pub fn base_score(play_count: i64, recent_7d: i64, days_since_last: i64, repetition_count: i64) -> f64 {
    let popularity = ((play_count as f64) + 1.0).ln();
    let recency_boost = ((recent_7d as f64) + 1.0).ln() * 0.8;
    let freshness = 1.0 / ((days_since_last.max(1) as f64).sqrt());
    let repetition_penalty = (repetition_count as f64) * 0.25;

    (1.3 * popularity) + (1.0 * recency_boost) + (0.7 * freshness) - repetition_penalty
}
