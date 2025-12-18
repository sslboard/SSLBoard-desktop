use std::{
    fs,
    path::PathBuf,
    sync::{Arc, Mutex, MutexGuard},
};

use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OpenFlags, OptionalExtension, Row};
use tauri::{AppHandle, Manager};

use super::types::SecretMetadata;

#[derive(Clone)]
pub struct SecretMetadataStore {
    conn: Arc<Mutex<Connection>>,
}

impl SecretMetadataStore {
    pub fn initialize(app: AppHandle) -> Result<Self> {
        let data_dir = app
            .path()
            .app_data_dir()
            .context("failed to resolve app data dir")?;
        fs::create_dir_all(&data_dir)?;

        let db_path = data_dir.join("secrets.sqlite");
        let created = !db_path.exists();
        let conn = Connection::open_with_flags(
            &db_path,
            OpenFlags::SQLITE_OPEN_CREATE
                | OpenFlags::SQLITE_OPEN_READ_WRITE
                | OpenFlags::SQLITE_OPEN_FULL_MUTEX,
        )
        .with_context(|| format!("failed to open secrets db at {}", db_path.display()))?;

        Self::configure_connection(&conn)?;
        Self::init_schema(&conn)?;
        Self::enforce_permissions(&db_path, created)?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
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
            CREATE TABLE IF NOT EXISTS secret_metadata (
                id TEXT PRIMARY KEY,
                kind TEXT NOT NULL,
                label TEXT NOT NULL,
                created_at TEXT NOT NULL,
                ciphertext BLOB
            );
            "#,
        )?;
        if !Self::column_exists(conn, "secret_metadata", "ciphertext")? {
            conn.execute("ALTER TABLE secret_metadata ADD COLUMN ciphertext BLOB", [])?;
        }
        Ok(())
    }

    fn column_exists(conn: &Connection, table: &str, column: &str) -> Result<bool> {
        let mut stmt = conn.prepare(&format!("PRAGMA table_info({table})"))?;
        let mut rows = stmt.query([])?;
        while let Some(row) = rows.next()? {
            let name: String = row.get(1)?;
            if name.eq_ignore_ascii_case(column) {
                return Ok(true);
            }
        }
        Ok(false)
    }

    #[cfg(unix)]
    fn enforce_permissions(db_path: &PathBuf, created: bool) -> Result<()> {
        use std::os::unix::fs::PermissionsExt;

        let desired = fs::Permissions::from_mode(0o600);
        if created {
            fs::set_permissions(db_path, desired)?;
        } else {
            let metadata = fs::metadata(db_path)?;
            let current = metadata.permissions();
            if current.mode() & 0o177 != 0 {
                fs::set_permissions(db_path, desired)?;
            }
        }
        Ok(())
    }

    #[cfg(not(unix))]
    fn enforce_permissions(_db_path: &PathBuf, _created: bool) -> Result<()> {
        Ok(())
    }

    pub fn has_missing_ciphertext(&self) -> Result<bool> {
        let conn = self.lock_conn()?;
        let mut stmt = conn.prepare(
            r#"
            SELECT EXISTS(SELECT 1 FROM secret_metadata WHERE ciphertext IS NULL)
            "#,
        )?;
        let exists: i64 = stmt.query_row([], |row| row.get(0))?;
        Ok(exists == 1)
    }

    pub fn has_ciphertext(&self, id: &str) -> Result<bool> {
        let conn = self.lock_conn()?;
        let mut stmt = conn.prepare(
            r#"
            SELECT ciphertext IS NOT NULL FROM secret_metadata WHERE id = ?1
            "#,
        )?;
        let present: Option<i64> = stmt.query_row(params![id], |row| row.get(0)).optional()?;
        Ok(present.unwrap_or(0) == 1)
    }

    pub fn store_ciphertext(&self, id: &str, ciphertext: &[u8]) -> Result<()> {
        let conn = self.lock_conn()?;
        conn.execute(
            r#"
            UPDATE secret_metadata
            SET ciphertext = ?2
            WHERE id = ?1
            "#,
            params![id, ciphertext],
        )?;
        Ok(())
    }

    pub fn get_ciphertext(&self, id: &str) -> Result<Option<Vec<u8>>> {
        let conn = self.lock_conn()?;
        let mut stmt = conn.prepare(
            r#"
            SELECT ciphertext
            FROM secret_metadata
            WHERE id = ?1
            "#,
        )?;
        let mut rows = stmt.query(params![id])?;
        if let Some(row) = rows.next()? {
            let data: Option<Vec<u8>> = row.get(0)?;
            Ok(data)
        } else {
            Ok(None)
        }
    }

    pub fn clear_ciphertext(&self, id: &str) -> Result<()> {
        let conn = self.lock_conn()?;
        conn.execute(
            r#"
            UPDATE secret_metadata
            SET ciphertext = NULL
            WHERE id = ?1
            "#,
            params![id],
        )?;
        Ok(())
    }

    pub fn list(&self) -> Result<Vec<SecretMetadata>> {
        let conn = self.lock_conn()?;
        let mut stmt = conn.prepare(
            r#"
            SELECT id, kind, label, created_at
            FROM secret_metadata
            ORDER BY datetime(created_at) DESC
            "#,
        )?;

        let mut rows = stmt.query([])?;
        let mut records = Vec::new();
        while let Some(row) = rows.next()? {
            records.push(Self::row_to_record(row)?);
        }
        Ok(records)
    }

    pub fn get(&self, id: &str) -> Result<Option<SecretMetadata>> {
        let conn = self.lock_conn()?;
        let mut stmt = conn.prepare(
            r#"
            SELECT id, kind, label, created_at
            FROM secret_metadata
            WHERE id = ?1
            "#,
        )?;

        let mut rows = stmt.query(params![id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_record(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn insert(&self, record: &SecretMetadata) -> Result<()> {
        let conn = self.lock_conn()?;
        conn.execute(
            r#"
            INSERT INTO secret_metadata (id, kind, label, created_at)
            VALUES (?1, ?2, ?3, ?4)
            "#,
            params![
                record.id,
                record.kind.as_str(),
                record.label,
                record.created_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn update_label(&self, id: &str, label: &str) -> Result<()> {
        let conn = self.lock_conn()?;
        conn.execute(
            r#"
            UPDATE secret_metadata
            SET label = ?2
            WHERE id = ?1
            "#,
            params![id, label],
        )?;
        Ok(())
    }

    pub fn delete(&self, id: &str) -> Result<()> {
        let conn = self.lock_conn()?;
        conn.execute(
            r#"
            DELETE FROM secret_metadata
            WHERE id = ?1
            "#,
            params![id],
        )?;
        Ok(())
    }

    fn row_to_record(row: &Row<'_>) -> Result<SecretMetadata> {
        let id: String = row.get(0)?;
        let kind_raw: String = row.get(1)?;
        let label: String = row.get(2)?;
        let created_raw: String = row.get(3)?;

        let kind = match kind_raw.as_str() {
            "dns_credential" => super::types::SecretKind::DnsCredential,
            "acme_account_key" => super::types::SecretKind::AcmeAccountKey,
            "managed_private_key" => super::types::SecretKind::ManagedPrivateKey,
            other => return Err(anyhow!("unknown secret kind: {other}")),
        };

        let created_at = DateTime::parse_from_rfc3339(&created_raw)
            .map(|dt| dt.with_timezone(&Utc))
            .context("failed to parse secret created_at")?;

        Ok(SecretMetadata {
            id,
            kind,
            label,
            created_at,
        })
    }

    fn lock_conn(&self) -> Result<MutexGuard<'_, Connection>> {
        self.conn
            .lock()
            .map_err(|err| anyhow!("secrets db mutex poisoned: {err}"))
    }
}
