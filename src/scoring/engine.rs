use crate::models::PlaylistKind;

#[derive(Debug, Clone)]
pub struct SongFeatures {
    pub metadata_similarity: f64,
    pub total_play_count: i64,
    pub recent_7d_count: i64,
    pub days_since_last_play: i64,
    pub repetition_14d_count: i64,
    pub deterministic_jitter: f64,
}

pub fn calculate_score(features: &SongFeatures) -> f64 {
    let metadata = features.metadata_similarity.clamp(0.0, 1.0);
    let log_play_count = ((features.total_play_count as f64) + 1.0).ln();
    let recency_boost = ((features.recent_7d_count as f64) + 1.0).ln();
    let freshness_decay = 1.0 / ((features.days_since_last_play.max(1) as f64).sqrt());
    let repetition_penalty = (features.repetition_14d_count as f64).powf(1.15);

    // Deterministic weighted score with very small jitter to avoid tie lock-in.
    (0.30 * metadata)
        + (0.28 * log_play_count)
        + (0.22 * recency_boost)
        + (0.12 * freshness_decay)
        - (0.08 * repetition_penalty)
        + features.deterministic_jitter
}

pub fn calculate_score_for_kind(kind: PlaylistKind, features: &SongFeatures) -> f64 {
    let metadata = features.metadata_similarity.clamp(0.0, 1.0);
    let log_play_count = ((features.total_play_count as f64) + 1.0).ln();
    let recency_boost = ((features.recent_7d_count as f64) + 1.0).ln();
    let freshness_decay = 1.0 / ((features.days_since_last_play.max(1) as f64).sqrt());
    let repetition_penalty = (features.repetition_14d_count as f64).powf(1.15);
    let very_recent_penalty = if features.days_since_last_play <= 2 {
        1.0
    } else {
        0.0
    };

    let weighted = match kind {
        PlaylistKind::Favorites => {
            (0.26 * metadata) + (0.40 * log_play_count) + (0.20 * recency_boost) + (0.08 * freshness_decay)
                - (0.06 * repetition_penalty)
        }
        PlaylistKind::Rediscovery => {
            (0.34 * metadata) + (0.12 * log_play_count) + (0.06 * recency_boost) + (0.40 * freshness_decay)
                - (0.10 * repetition_penalty)
                - (0.16 * very_recent_penalty)
        }
        PlaylistKind::Genre => {
            (0.50 * metadata) + (0.18 * log_play_count) + (0.12 * recency_boost) + (0.10 * freshness_decay)
                - (0.10 * repetition_penalty)
        }
        PlaylistKind::Artist => {
            (0.33 * metadata) + (0.24 * log_play_count) + (0.30 * recency_boost) + (0.05 * freshness_decay)
                - (0.08 * repetition_penalty)
        }
        PlaylistKind::SmartShuffle => {
            (0.30 * metadata) + (0.23 * log_play_count) + (0.20 * recency_boost) + (0.18 * freshness_decay)
                - (0.09 * repetition_penalty)
        }
    };

    weighted + features.deterministic_jitter
}
