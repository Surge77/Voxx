use crate::modes::DictationMode;
use crate::state::Preferences;
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryEntry {
    pub id: i64,
    pub raw_text: String,
    pub processed_text: String,
    pub mode: DictationMode,
    pub created_at: String,
    pub duration_ms: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DictionaryEntry {
    pub id: i64,
    pub wrong: String,
    pub right: String,
    pub created_at: String,
}

#[derive(Clone)]
pub struct Database {
    path: PathBuf,
}

impl Database {
    pub fn open(path: PathBuf) -> rusqlite::Result<Self> {
        let db = Self { path };
        db.migrate()?;
        Ok(db)
    }

    fn connect(&self) -> rusqlite::Result<Connection> {
        Connection::open(&self.path)
    }

    fn migrate(&self) -> rusqlite::Result<()> {
        let conn = self.connect()?;
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS history (
              id INTEGER PRIMARY KEY AUTOINCREMENT,
              raw_text TEXT NOT NULL,
              processed_text TEXT NOT NULL,
              mode TEXT NOT NULL,
              created_at TEXT NOT NULL,
              duration_ms INTEGER NOT NULL DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS dictionary_entries (
              id INTEGER PRIMARY KEY AUTOINCREMENT,
              wrong TEXT NOT NULL,
              right TEXT NOT NULL,
              created_at TEXT NOT NULL,
              UNIQUE(wrong, right)
            );

            CREATE TABLE IF NOT EXISTS preferences (
              key TEXT PRIMARY KEY,
              value TEXT NOT NULL
            );
            ",
        )
    }

    pub fn insert_history(
        &self,
        raw_text: &str,
        processed_text: &str,
        mode: DictationMode,
        duration_ms: i64,
    ) -> rusqlite::Result<HistoryEntry> {
        let created_at = Utc::now().to_rfc3339();
        let conn = self.connect()?;
        conn.execute(
            "INSERT INTO history (raw_text, processed_text, mode, created_at, duration_ms) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![raw_text, processed_text, mode_to_str(mode), created_at, duration_ms],
        )?;
        let id = conn.last_insert_rowid();
        Ok(HistoryEntry {
            id,
            raw_text: raw_text.to_string(),
            processed_text: processed_text.to_string(),
            mode,
            created_at,
            duration_ms,
        })
    }

    pub fn search_history(&self, query: &str) -> rusqlite::Result<Vec<HistoryEntry>> {
        let like = format!("%{}%", query.trim());
        let conn = self.connect()?;
        let mut stmt = if query.trim().is_empty() {
            conn.prepare("SELECT id, raw_text, processed_text, mode, created_at, duration_ms FROM history ORDER BY id DESC")?
        } else {
            conn.prepare(
                "SELECT id, raw_text, processed_text, mode, created_at, duration_ms
                 FROM history
                 WHERE raw_text LIKE ?1 OR processed_text LIKE ?1 OR mode LIKE ?1
                 ORDER BY id DESC",
            )?
        };

        let rows = if query.trim().is_empty() {
            stmt.query_map([], history_from_row)?.collect::<rusqlite::Result<Vec<_>>>()?
        } else {
            stmt.query_map(params![like], history_from_row)?.collect::<rusqlite::Result<Vec<_>>>()?
        };

        Ok(rows)
    }

    pub fn get_history_entry(&self, id: i64) -> rusqlite::Result<Option<HistoryEntry>> {
        let conn = self.connect()?;
        conn.query_row(
            "SELECT id, raw_text, processed_text, mode, created_at, duration_ms FROM history WHERE id = ?1",
            params![id],
            history_from_row,
        )
        .optional()
    }

    pub fn update_history_entry(&self, id: i64, processed_text: &str) -> rusqlite::Result<Option<HistoryEntry>> {
        let conn = self.connect()?;
        conn.execute("UPDATE history SET processed_text = ?1 WHERE id = ?2", params![processed_text, id])?;
        drop(conn);
        self.get_history_entry(id)
    }

    pub fn delete_history_entry(&self, id: i64) -> rusqlite::Result<()> {
        let conn = self.connect()?;
        conn.execute("DELETE FROM history WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn list_dictionary(&self) -> rusqlite::Result<Vec<DictionaryEntry>> {
        let conn = self.connect()?;
        let mut stmt = conn.prepare("SELECT id, wrong, right, created_at FROM dictionary_entries ORDER BY id DESC")?;
        stmt.query_map([], dictionary_from_row)?.collect()
    }

    pub fn insert_dictionary(&self, wrong: &str, right: &str) -> rusqlite::Result<DictionaryEntry> {
        let created_at = Utc::now().to_rfc3339();
        let conn = self.connect()?;
        conn.execute(
            "INSERT OR IGNORE INTO dictionary_entries (wrong, right, created_at) VALUES (?1, ?2, ?3)",
            params![wrong.trim(), right.trim(), created_at],
        )?;
        let id = conn.query_row(
            "SELECT id FROM dictionary_entries WHERE wrong = ?1 AND right = ?2",
            params![wrong.trim(), right.trim()],
            |row| row.get::<_, i64>(0),
        )?;
        Ok(DictionaryEntry {
            id,
            wrong: wrong.trim().to_string(),
            right: right.trim().to_string(),
            created_at,
        })
    }

    pub fn delete_dictionary_entry(&self, id: i64) -> rusqlite::Result<()> {
        let conn = self.connect()?;
        conn.execute("DELETE FROM dictionary_entries WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn load_preferences(&self) -> rusqlite::Result<Preferences> {
        let conn = self.connect()?;
        let value: Option<String> = conn
            .query_row("SELECT value FROM preferences WHERE key = 'app'", [], |row| row.get(0))
            .optional()?;

        Ok(value
            .and_then(|json| serde_json::from_str::<Preferences>(&json).ok())
            .unwrap_or_default())
    }

    pub fn save_preferences(&self, preferences: &Preferences) -> rusqlite::Result<()> {
        let conn = self.connect()?;
        let json = serde_json::to_string(preferences).unwrap_or_else(|_| "{}".to_string());
        conn.execute(
            "INSERT INTO preferences (key, value) VALUES ('app', ?1)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![json],
        )?;
        Ok(())
    }

    pub fn dictionary_prompt_lines(&self) -> rusqlite::Result<Vec<String>> {
        Ok(self
            .list_dictionary()?
            .into_iter()
            .map(|entry| format!("Replace '{}' with '{}'.", entry.wrong, entry.right))
            .collect())
    }
}

fn history_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<HistoryEntry> {
    let mode: String = row.get(3)?;
    Ok(HistoryEntry {
        id: row.get(0)?,
        raw_text: row.get(1)?,
        processed_text: row.get(2)?,
        mode: mode_from_str(&mode),
        created_at: row.get(4)?,
        duration_ms: row.get(5)?,
    })
}

fn dictionary_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<DictionaryEntry> {
    Ok(DictionaryEntry {
        id: row.get(0)?,
        wrong: row.get(1)?,
        right: row.get(2)?,
        created_at: row.get(3)?,
    })
}

pub fn mode_to_str(mode: DictationMode) -> &'static str {
    match mode {
        DictationMode::General => "general",
        DictationMode::Code => "code",
        DictationMode::Command => "command",
        DictationMode::Email => "email",
    }
}

pub fn mode_from_str(value: &str) -> DictationMode {
    match value {
        "code" => DictationMode::Code,
        "command" => DictationMode::Command,
        "email" => DictationMode::Email,
        _ => DictationMode::General,
    }
}

#[cfg(test)]
mod tests {
    use super::Database;
    use crate::modes::DictationMode;

    #[test]
    fn stores_and_searches_history() {
        let path = std::env::temp_dir().join(format!("voxx-test-{}.db", uuid_like()));
        let db = Database::open(path.clone()).expect("open db");
        db.insert_history("react query", "React Query", DictationMode::Code, 500)
            .expect("insert");

        let results = db.search_history("query").expect("search");

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].processed_text, "React Query");
        let _ = std::fs::remove_file(path);
    }

    fn uuid_like() -> String {
        format!("{:?}", std::time::SystemTime::now())
            .replace(':', "")
            .replace('\\', "")
            .replace('/', "")
            .replace(' ', "-")
    }
}

