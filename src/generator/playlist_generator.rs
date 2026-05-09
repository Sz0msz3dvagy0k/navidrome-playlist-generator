use chrono::NaiveDate;
use rand::{rngs::StdRng, seq::SliceRandom, SeedableRng};
use sqlx::PgPool;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use crate::models::PlaylistKind;
use crate::scoring::engine::{calculate_score_for_kind, SongFeatures};

#[derive(Debug, Clone)]
pub struct PlaylistDraft {
    pub date: NaiveDate,
    pub kind: PlaylistKind,
    pub name: String,
    pub song_ids: Vec<Uuid>,
}

pub fn playlist_name(_date: NaiveDate, kind: PlaylistKind) -> String {
    match kind {
        PlaylistKind::Favorites => "Favorites mix".to_string(),
        PlaylistKind::Rediscovery => "Rediscovery mix".to_string(),
        PlaylistKind::Genre => "Genre mix".to_string(),
        PlaylistKind::Artist => "Artist mix".to_string(),
        PlaylistKind::SmartShuffle => "Smart shuffle mix".to_string(),
    }
}

#[derive(Debug, Clone, sqlx::FromRow)]
struct SongCandidate {
    id: Uuid,
    artist_id: Uuid,
    genre: Option<String>,
    total_play_count: i64,
    recent_7d_count: i64,
    days_since_last_play: i64,
    repetition_14d_count: i64,
}

#[derive(Debug, Clone)]
struct RankedCandidate {
    song_id: Uuid,
    artist_id: Uuid,
    genre: Option<String>,
    score: f64,
}

pub async fn generate_daily_playlists(
    pool: &PgPool,
    date: NaiveDate,
    playlist_size: usize,
) -> anyhow::Result<Vec<PlaylistDraft>> {
    let base_candidates: Vec<SongCandidate> = sqlx::query_as(
        r#"
        SELECT
            s.id,
            s.artist_id,
            s.genre,
            COALESCE(ast.total_play_count, 0) AS total_play_count,
            COALESCE(ast.recent_7d_count, 0) AS recent_7d_count,
            COALESCE(EXTRACT(DAY FROM NOW() - ast.last_played_at), 999)::BIGINT AS days_since_last_play,
            COALESCE(rep.play_count_14d, 0) AS repetition_14d_count
        FROM songs s
        LEFT JOIN aggregated_stats ast ON ast.song_id = s.id
        LEFT JOIN (
            SELECT song_id, COUNT(*)::BIGINT AS play_count_14d
            FROM play_history
            WHERE played_at >= NOW() - INTERVAL '14 days'
            GROUP BY song_id
        ) rep ON rep.song_id = s.id
        "#,
    )
    .fetch_all(pool)
    .await?;

    tracing::info!(
        "loaded {} candidate songs for daily generation (target size: {})",
        base_candidates.len(),
        playlist_size
    );

    let mut global_use_count: HashMap<Uuid, usize> = HashMap::new();

    let mut playlists = Vec::new();
    for kind in [
        PlaylistKind::Favorites,
        PlaylistKind::Rediscovery,
        PlaylistKind::Genre,
        PlaylistKind::Artist,
        PlaylistKind::SmartShuffle,
    ] {
        let mut ranked = rank_candidates_for_kind(&base_candidates, date, kind);
        let selected = pick_playlist_songs(&mut ranked, playlist_size, &mut global_use_count, date, kind);
        tracing::info!(
            "playlist {:?} selected {} songs",
            kind,
            selected.len()
        );
        playlists.push(PlaylistDraft {
            date,
            kind,
            name: playlist_name(date, kind),
            song_ids: selected,
        });
    }

    Ok(playlists)
}

fn rank_candidates_for_kind(
    candidates: &[SongCandidate],
    date: NaiveDate,
    kind: PlaylistKind,
) -> Vec<RankedCandidate> {
    let seed = date
        .to_string()
        .bytes()
        .fold(0_u64, |acc, b| acc.wrapping_mul(131).wrapping_add(b as u64))
        ^ (kind as u64 + 17);
    let mut rng = StdRng::seed_from_u64(seed);

    let mut ranked = candidates
        .iter()
        .map(|song| {
            let metadata_similarity = match kind {
                PlaylistKind::Favorites => 0.85,
                PlaylistKind::Rediscovery => 0.55,
                PlaylistKind::Genre => {
                    if song.genre.as_deref().is_some() {
                        0.8
                    } else {
                        0.4
                    }
                }
                PlaylistKind::Artist => 0.75,
                PlaylistKind::SmartShuffle => 0.65,
            };

            let features = SongFeatures {
                metadata_similarity,
                total_play_count: song.total_play_count,
                recent_7d_count: song.recent_7d_count,
                days_since_last_play: song.days_since_last_play,
                repetition_14d_count: song.repetition_14d_count,
                deterministic_jitter: rand::Rng::random_range(&mut rng, -0.015..0.015),
            };

            RankedCandidate {
                song_id: song.id,
                artist_id: song.artist_id,
                genre: song.genre.clone(),
                score: calculate_score_for_kind(kind, &features)
                    + match kind {
                        PlaylistKind::Favorites => song.total_play_count as f64 * 0.01,
                        PlaylistKind::Rediscovery => {
                            let stale_bonus = song.days_since_last_play.min(120) as f64 * 0.006;
                            let very_recent_penalty = if song.days_since_last_play <= 3 { 0.25 } else { 0.0 };
                            stale_bonus - very_recent_penalty
                        }
                        PlaylistKind::Genre => {
                            if song.genre.is_some() {
                                0.2
                            } else {
                                -0.1
                            }
                        }
                        PlaylistKind::Artist => song.recent_7d_count as f64 * 0.012,
                        PlaylistKind::SmartShuffle => {
                            if song.genre.is_some() { 0.05 } else { 0.0 }
                        }
                    },
            }
        })
        .collect::<Vec<_>>();

    ranked.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    ranked
}

fn pick_playlist_songs(
    ranked: &mut [RankedCandidate],
    playlist_size: usize,
    global_use_count: &mut HashMap<Uuid, usize>,
    date: NaiveDate,
    kind: PlaylistKind,
) -> Vec<Uuid> {
    let seed = date
        .to_string()
        .bytes()
        .fold(0_u64, |acc, b| acc.wrapping_mul(67).wrapping_add(b as u64))
        ^ (kind as u64 + 991);
    let mut rng = StdRng::seed_from_u64(seed);

    let mut selected = Vec::with_capacity(playlist_size);
    let mut selected_set: HashSet<Uuid> = HashSet::new();
    let mut recent_artists: Vec<Uuid> = Vec::new();
    let mut genre_counts: HashMap<String, usize> = HashMap::new();

    ranked.shuffle(&mut rng);
    ranked.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

    if !ranked.is_empty() {
        tracing::debug!(
            "top 3 scored candidates for {:?}: [{}, {}, {}]",
            kind,
            ranked.get(0).map(|c| c.score).unwrap_or(0.0),
            ranked.get(1).map(|c| c.score).unwrap_or(0.0),
            ranked.get(2).map(|c| c.score).unwrap_or(0.0),
        );
    }

    let mut skipped_by_reason = std::collections::HashMap::new();

    for item in ranked.iter() {
        if selected.len() >= playlist_size {
            break;
        }
        if selected_set.contains(&item.song_id) {
            continue;
        }

        let global_repeats = *global_use_count.get(&item.song_id).unwrap_or(&0);
        if global_repeats >= 2 {
            *skipped_by_reason.entry("global_repeat_limit").or_insert(0) += 1;
            continue;
        }

        // Only enforce artist streak if we have at least 2 recent artists
        if recent_artists.len() >= 2 {
            let last_two_same_artist = recent_artists.iter().rev().take(2)
                .all(|a| *a == item.artist_id);
            if last_two_same_artist {
                *skipped_by_reason.entry("artist_streak").or_insert(0) += 1;
                continue;
            }
        }

        if let Some(genre) = item.genre.as_ref() {
            let count = *genre_counts.get(genre).unwrap_or(&0);
            if count > playlist_size / 3 {
                *skipped_by_reason.entry("genre_limit").or_insert(0) += 1;
                continue;
            }
            genre_counts.insert(genre.clone(), count + 1);
        }

        selected.push(item.song_id);
        selected_set.insert(item.song_id);
        recent_artists.push(item.artist_id);
        *global_use_count.entry(item.song_id).or_insert(0) += 1;
    }

    for (reason, count) in skipped_by_reason {
        tracing::debug!("playlist {:?}: {} songs skipped ({})", kind, count, reason);
    }

    reorder_for_smoother_transitions(&selected, ranked, kind)
}

fn reorder_for_smoother_transitions(
    selected_ids: &[Uuid],
    ranked: &[RankedCandidate],
    kind: PlaylistKind,
) -> Vec<Uuid> {
    if selected_ids.len() <= 2 {
        return selected_ids.to_vec();
    }

    let mut by_id: HashMap<Uuid, &RankedCandidate> = HashMap::new();
    for candidate in ranked {
        by_id.insert(candidate.song_id, candidate);
    }

    let mut remaining: Vec<Uuid> = selected_ids.to_vec();
    remaining.sort_by(|a, b| {
        let a_score = by_id.get(a).map(|s| s.score).unwrap_or(0.0);
        let b_score = by_id.get(b).map(|s| s.score).unwrap_or(0.0);
        b_score
            .partial_cmp(&a_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let first = remaining.remove(0);
    let mut ordered = vec![first];

    while let Some(last) = ordered.last().copied() {
        if remaining.is_empty() {
            break;
        }

        let mut best_idx = 0usize;
        let mut best_cost = f64::MAX;
        for (idx, candidate_id) in remaining.iter().enumerate() {
            let transition_cost = transition_cost(
                by_id.get(&last).copied(),
                by_id.get(candidate_id).copied(),
                kind,
            );
            if transition_cost < best_cost {
                best_cost = transition_cost;
                best_idx = idx;
            }
        }

        ordered.push(remaining.remove(best_idx));
    }

    ordered
}

fn transition_cost(
    from: Option<&RankedCandidate>,
    to: Option<&RankedCandidate>,
    kind: PlaylistKind,
) -> f64 {
    match (from, to) {
        (Some(from), Some(to)) => {
            let mut cost = (from.score - to.score).abs();
            let (genre_penalty, artist_penalty) = match kind {
                PlaylistKind::Genre => (1.2, 0.25),
                PlaylistKind::Artist => (0.55, 0.9),
                PlaylistKind::Rediscovery => (0.7, 0.3),
                PlaylistKind::Favorites => (0.65, 0.45),
                PlaylistKind::SmartShuffle => (0.8, 0.4),
            };
            if from.genre != to.genre {
                cost += genre_penalty;
            }
            if from.artist_id != to.artist_id {
                cost += artist_penalty;
            }
            cost
        }
        _ => 10.0,
    }
}
