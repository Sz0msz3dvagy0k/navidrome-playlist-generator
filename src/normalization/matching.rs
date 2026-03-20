use strsim::normalized_levenshtein;

use crate::utils::text::normalize_text;

pub fn match_score(a_artist: &str, a_title: &str, b_artist: &str, b_title: &str) -> f64 {
    let artist_sim = normalized_levenshtein(&normalize_text(a_artist), &normalize_text(b_artist));
    let title_sim = normalized_levenshtein(&normalize_text(a_title), &normalize_text(b_title));
    (artist_sim * 0.55) + (title_sim * 0.45)
}
