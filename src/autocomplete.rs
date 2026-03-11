/// Fuzzy autocomplete over the bundled German dictionary.
/// Uses nucleo-matcher (same engine as the Helix editor).
///
/// The wordlist is compiled directly into the binary via include_str!
/// — zero file I/O at runtime.

// The dictionary file is expected at assets/german_words.txt
// (one word per line, ~170k entries).
static GERMAN_WORDS: &str = include_str!("../assets/german_words.txt");

pub struct Autocomplete {
    words: Vec<&'static str>,
    limit: usize,
}

impl Autocomplete {
    pub fn new(limit: usize) -> Self {
        let words: Vec<&'static str> = GERMAN_WORDS
            .lines()
            .filter(|l| !l.trim().is_empty())
            .collect();
        Self { words, limit }
    }

    /// Return up to `self.limit` words that fuzzy-match `query`.
    /// Falls back to simple prefix matching until nucleo-matcher is wired up.
    pub fn search(&self, query: &str) -> Vec<String> {
        if query.is_empty() {
            return Vec::new();
        }

        // ── TODO (Phase 4): replace this with nucleo-matcher scoring ─────────
        // use nucleo_matcher::{Matcher, Config, pattern::{Pattern, CaseMatching}};
        // let mut matcher = Matcher::new(Config::DEFAULT);
        // let pattern = Pattern::parse(query, CaseMatching::Ignore);
        // Then score each word and sort by score descending.
        // ─────────────────────────────────────────────────────────────────────

        // MVP fallback: case-insensitive prefix + substring filter
        let q = query.to_lowercase();
        let mut results: Vec<String> = self
            .words
            .iter()
            .filter(|w| w.to_lowercase().contains(&q))
            .take(self.limit)
            .map(|w| w.to_string())
            .collect();

        // Prefer prefix matches at the top
        results.sort_by_key(|w| {
            if w.to_lowercase().starts_with(&q) { 0usize } else { 1 }
        });

        results
    }
}
