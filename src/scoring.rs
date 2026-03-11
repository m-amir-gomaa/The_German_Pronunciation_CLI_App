/// Pronunciation accuracy scoring.
///
/// Compares the expected German word/phrase with what Whisper transcribed.
/// Uses Levenshtein distance (via `strsim`) on lowercased strings.
///
/// Returns a score from 0–100.
/// Phase 5+ upgrade: compare IPA strings instead of raw text for phoneme-level scoring.

use strsim::normalized_levenshtein;

/// Score how closely `heard` matches `expected`.
/// Returns 0–100 (100 = perfect match).
pub fn score(expected: &str, heard: &str) -> u8 {
    if heard.is_empty() || heard.contains("[STT not yet implemented]") {
        return 0;
    }

    let a = expected.to_lowercase();
    let b = heard.to_lowercase();

    // normalized_levenshtein returns 0.0–1.0
    let similarity = normalized_levenshtein(&a, &b);
    (similarity * 100.0).round() as u8
}

/// Upgrade path for Phase 5+:
/// Convert both strings to IPA first via espeak-ng, then score phoneme sequences.
/// This catches cases where the user said the right sounds but Whisper transcribed
/// them differently (e.g. "Entschuldigung" vs "Entschuldigung" with slight variance).
pub fn score_ipa(expected_ipa: &str, heard_ipa: &str) -> u8 {
    let similarity = normalized_levenshtein(expected_ipa, heard_ipa);
    (similarity * 100.0).round() as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn perfect_match() {
        assert_eq!(score("hallo", "hallo"), 100);
    }

    #[test]
    fn empty_heard() {
        assert_eq!(score("hallo", ""), 0);
    }

    #[test]
    fn close_match() {
        let s = score("entschuldigung", "entschuldigunk");
        assert!(s > 80, "score was {}", s);
    }
}
