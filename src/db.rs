/// SQLite persistence layer.
/// Stores saved words, IPA, attempt history, and best scores.
///
/// DB location: ~/.local/share/pronouncer/words.db

use anyhow::{Context, Result};
use rusqlite::{Connection, params};
use std::path::PathBuf;

pub struct Db {
    conn: Connection,
}

impl Db {
    pub fn open() -> Result<Self> {
        let path = db_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(&path)
            .with_context(|| format!("Failed to open DB at {}", path.display()))?;
        let db = Db { conn };
        db.migrate()?;
        Ok(db)
    }

    fn migrate(&self) -> Result<()> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS words (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                word        TEXT NOT NULL UNIQUE,
                ipa         TEXT NOT NULL,
                created_at  INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
            );

            CREATE TABLE IF NOT EXISTS attempts (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                word_id     INTEGER NOT NULL REFERENCES words(id),
                score       INTEGER NOT NULL,
                attempted_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
            );",
        )?;
        Ok(())
    }

    /// Save a word + its IPA to the word list (idempotent).
    pub fn save_word(&self, word: &str, ipa: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO words (word, ipa) VALUES (?1, ?2)
             ON CONFLICT(word) DO UPDATE SET ipa = excluded.ipa",
            params![word, ipa],
        )?;
        Ok(())
    }

    /// Record a pronunciation attempt and its score.
    pub fn record_attempt(&self, word: &str, score: u8) -> Result<()> {
        // Auto-save word if not already present
        let ipa = crate::ipa::get_ipa(word).unwrap_or_default();
        self.save_word(word, &ipa)?;

        let word_id: i64 = self.conn.query_row(
            "SELECT id FROM words WHERE word = ?1",
            params![word],
            |row| row.get(0),
        )?;

        self.conn.execute(
            "INSERT INTO attempts (word_id, score) VALUES (?1, ?2)",
            params![word_id, score],
        )?;
        Ok(())
    }

    /// List all saved words with best score.
    pub fn list_words(&self) -> Result<Vec<(String, String, Option<u8>)>> {
        let mut stmt = self.conn.prepare(
            "SELECT w.word, w.ipa, MAX(a.score) as best_score
             FROM words w
             LEFT JOIN attempts a ON a.word_id = w.id
             GROUP BY w.id
             ORDER BY w.word ASC",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<u8>>(2)?,
            ))
        })?;

        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    /// Pick a random saved word for drill/practice mode.
    pub fn random_word(&self) -> Result<String> {
        let count: i64 = self.conn
            .query_row("SELECT COUNT(*) FROM words", [], |r| r.get(0))?;

        if count == 0 {
            return Err(anyhow::anyhow!(
                "No saved words yet. Use `gp save <word>` to add some."
            ));
        }

        // Bias toward words with lower scores (spaced repetition lite)
        let word: String = self.conn.query_row(
            "SELECT w.word FROM words w
             LEFT JOIN (
               SELECT word_id, MAX(score) as best FROM attempts GROUP BY word_id
             ) a ON a.word_id = w.id
             ORDER BY COALESCE(a.best, 0) ASC, RANDOM()
             LIMIT 1",
            [],
            |row| row.get(0),
        )?;

        Ok(word)
    }
}

fn db_path() -> PathBuf {
    let base = std::env::var("XDG_DATA_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| "~".to_string());
            PathBuf::from(home).join(".local").join("share")
        });
    base.join("pronouncer").join("words.db")
}
