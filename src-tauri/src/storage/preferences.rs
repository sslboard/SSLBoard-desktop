use std::{
    fs,
    path::Path,
    sync::{Arc, Mutex, MutexGuard},
};

use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OpenFlags, OptionalExtension, Row};
use tauri::{AppHandle, Manager};

#[derive(Clone, Debug)]
pub struct PreferenceRecord {
    pub name: String,
    pub value: String,
    #[allow(dead_code)]
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct PreferencesStore {
    conn: Arc<Mutex<Connection>>,
}

impl PreferencesStore {
    pub fn initialize(app: AppHandle) -> Result<Self> {
        let data_dir = app
            .path()
            .app_data_dir()
            .context("failed to resolve app data dir")?;
        Self::initialize_with_path(&data_dir)
    }

    fn initialize_with_path(data_dir: &Path) -> Result<Self> {
        fs::create_dir_all(data_dir)?;

        let db_path = data_dir.join("preferences.sqlite");
        let conn = Connection::open_with_flags(
            &db_path,
            OpenFlags::SQLITE_OPEN_CREATE
                | OpenFlags::SQLITE_OPEN_READ_WRITE
                | OpenFlags::SQLITE_OPEN_FULL_MUTEX,
        )
        .with_context(|| format!("failed to open SQLite database at {}", db_path.display()))?;

        Self::configure_connection(&conn)?;
        Self::init_schema(&conn)?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn get(&self, name: &str) -> Result<Option<PreferenceRecord>> {
        let conn = self.lock_conn()?;
        let mut stmt = conn.prepare(
            r#"
            SELECT name, value, updated_at
            FROM preferences
            WHERE name = ?1
            "#,
        )?;

        let record = stmt
            .query_row(params![name], |row| Self::row_to_record(row))
            .optional()?;
        Ok(record)
    }

    pub fn set(&self, name: &str, value: &str) -> Result<PreferenceRecord> {
        let conn = self.lock_conn()?;
        let now = Utc::now().to_rfc3339();

        let updated = conn.execute(
            r#"
            INSERT INTO preferences (name, value, updated_at)
            VALUES (?1, ?2, ?3)
            ON CONFLICT(name) DO UPDATE
            SET value = excluded.value,
                updated_at = excluded.updated_at
            "#,
            params![name, value, now],
        )?;

        if updated == 0 {
            return Err(anyhow!("failed to upsert preference: {name}"));
        }

        Self::get_with_conn(&conn, name)?
            .ok_or_else(|| anyhow!("preference not found after upsert: {name}"))
    }

    fn configure_connection(conn: &Connection) -> Result<()> {
        conn.execute_batch(
            r#"
            PRAGMA journal_mode = WAL;
            PRAGMA foreign_keys = ON;
            "#,
        )?;
        Ok(())
    }

    fn init_schema(conn: &Connection) -> Result<()> {
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS preferences (
                name TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            "#,
        )?;
        Ok(())
    }

    fn get_with_conn(conn: &Connection, name: &str) -> Result<Option<PreferenceRecord>> {
        let mut stmt = conn.prepare(
            r#"
            SELECT name, value, updated_at
            FROM preferences
            WHERE name = ?1
            "#,
        )?;

        let record = stmt
            .query_row(params![name], |row| Self::row_to_record(row))
            .optional()?;
        Ok(record)
    }

    fn row_to_record(row: &Row<'_>) -> rusqlite::Result<PreferenceRecord> {
        let updated_at_str: String = row.get(2)?;
        let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
            .map_err(|err| {
                rusqlite::Error::FromSqlConversionFailure(
                    2,
                    rusqlite::types::Type::Text,
                    Box::new(err),
                )
            })?
            .with_timezone(&Utc);
        Ok(PreferenceRecord {
            name: row.get(0)?,
            value: row.get(1)?,
            updated_at,
        })
    }

    fn lock_conn(&self) -> Result<MutexGuard<'_, Connection>> {
        self.conn
            .lock()
            .map_err(|err| anyhow!("preferences db mutex poisoned: {err}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use uuid::Uuid;

    fn create_temp_dir() -> Result<std::path::PathBuf> {
        let mut path = std::env::temp_dir();
        path.push(format!("sslboard_pref_test_{}", Uuid::new_v4().as_simple()));
        fs::create_dir_all(&path)?;
        Ok(path)
    }

    #[test]
    fn preference_upsert_and_get() -> Result<()> {
        let temp_dir = create_temp_dir()?;
        let store = PreferencesStore::initialize_with_path(&temp_dir)?;

        assert!(store.get("export_destination")?.is_none());

        let first = store.set("export_destination", "/tmp/first")?;
        assert_eq!(first.name, "export_destination");
        assert_eq!(first.value, "/tmp/first");

        let updated = store.set("export_destination", "/tmp/second")?;
        assert_eq!(updated.value, "/tmp/second");

        let fetched = store
            .get("export_destination")?
            .ok_or_else(|| anyhow!("expected preference to exist"))?;
        assert_eq!(fetched.value, "/tmp/second");

        drop(store);
        fs::remove_dir_all(&temp_dir)?;
        Ok(())
    }
}
