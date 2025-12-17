use std::{
    fs,
    sync::{Arc, Mutex, MutexGuard},
};

use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OpenFlags, Row};
use tauri::{AppHandle, Manager};

#[derive(Clone, Debug)]
pub struct DnsZoneMapping {
    pub hostname_pattern: String,
    pub zone: Option<String>,
    pub adapter_id: String,
    pub secret_ref: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Stores hostname → zone → adapter mappings for DNS challenges.
#[derive(Clone)]
pub struct DnsConfigStore {
    conn: Arc<Mutex<Connection>>,
}

impl DnsConfigStore {
    pub fn initialize(app: AppHandle) -> Result<Self> {
        let data_dir = app
            .path()
            .app_data_dir()
            .context("failed to resolve app data dir")?;
        fs::create_dir_all(&data_dir)?;

        let db_path = data_dir.join("issuance.sqlite");
        let conn = Connection::open_with_flags(
            &db_path,
            OpenFlags::SQLITE_OPEN_CREATE
                | OpenFlags::SQLITE_OPEN_READ_WRITE
                | OpenFlags::SQLITE_OPEN_FULL_MUTEX,
        )
        .with_context(|| format!("failed to open SQLite database at {}", db_path.display()))?;

        Self::configure_connection(&conn)?;
        Self::init_schema(&conn)?;
        Self::seed_defaults(&conn)?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn upsert_mapping(
        &self,
        hostname_pattern: &str,
        zone: Option<String>,
        adapter_id: &str,
        secret_ref: Option<String>,
    ) -> Result<DnsZoneMapping> {
        let conn = self.lock_conn()?;
        let now = Utc::now().to_rfc3339();

        conn.execute(
            r#"
            INSERT INTO dns_zone_mappings (hostname_pattern, zone, adapter_id, secret_ref, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?5)
            ON CONFLICT(hostname_pattern) DO UPDATE SET
                zone = excluded.zone,
                adapter_id = excluded.adapter_id,
                secret_ref = excluded.secret_ref,
                updated_at = excluded.updated_at
            "#,
            params![hostname_pattern, zone, adapter_id, secret_ref, now],
        )?;

        self.get(hostname_pattern)?
            .ok_or_else(|| anyhow!("mapping not found after upsert: {hostname_pattern}"))
    }

    pub fn get(&self, hostname_pattern: &str) -> Result<Option<DnsZoneMapping>> {
        let conn = self.lock_conn()?;
        let mut stmt = conn.prepare(
            r#"
            SELECT hostname_pattern, zone, adapter_id, secret_ref, created_at, updated_at
            FROM dns_zone_mappings
            WHERE hostname_pattern = ?1
            "#,
        )?;

        let mut rows = stmt.query(params![hostname_pattern])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_record(row)?))
        } else {
            Ok(None)
        }
    }

    /// Finds the longest-suffix match for a given hostname.
    pub fn find_for_hostname(&self, hostname: &str) -> Result<Option<DnsZoneMapping>> {
        let conn = self.lock_conn()?;
        let mut stmt = conn.prepare(
            r#"
            SELECT hostname_pattern, zone, adapter_id, secret_ref, created_at, updated_at
            FROM dns_zone_mappings
            ORDER BY length(hostname_pattern) DESC
            "#,
        )?;

        let mut rows = stmt.query([])?;
        while let Some(row) = rows.next()? {
            let record = Self::row_to_record(row)?;
            if record.hostname_pattern == "*" || hostname.ends_with(&record.hostname_pattern) {
                return Ok(Some(record));
            }
        }
        Ok(None)
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
            CREATE TABLE IF NOT EXISTS dns_zone_mappings (
                hostname_pattern TEXT PRIMARY KEY,
                zone TEXT,
                adapter_id TEXT NOT NULL,
                secret_ref TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            "#,
        )?;
        Ok(())
    }

    fn seed_defaults(conn: &Connection) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        conn.execute(
            r#"
            INSERT OR IGNORE INTO dns_zone_mappings (
                hostname_pattern, zone, adapter_id, secret_ref, created_at, updated_at
            ) VALUES ('*', NULL, 'manual', NULL, ?1, ?1)
            "#,
            params![now],
        )?;
        Ok(())
    }

    fn row_to_record(row: &Row<'_>) -> Result<DnsZoneMapping> {
        let created_at_raw: String = row.get(4)?;
        let updated_at_raw: String = row.get(5)?;
        Ok(DnsZoneMapping {
            hostname_pattern: row.get(0)?,
            zone: row.get(1)?,
            adapter_id: row.get(2)?,
            secret_ref: row.get(3)?,
            created_at: chrono::DateTime::parse_from_rfc3339(&created_at_raw)
                .map(|dt| dt.with_timezone(&Utc))
                .context("failed to parse created_at")?,
            updated_at: chrono::DateTime::parse_from_rfc3339(&updated_at_raw)
                .map(|dt| dt.with_timezone(&Utc))
                .context("failed to parse updated_at")?,
        })
    }

    fn lock_conn(&self) -> Result<MutexGuard<'_, Connection>> {
        self.conn
            .lock()
            .map_err(|err| anyhow!("SQLite connection poisoned: {err}"))
    }
}
