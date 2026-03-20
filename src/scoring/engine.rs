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
