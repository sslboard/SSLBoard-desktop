use std::{
    fs,
    sync::{Arc, Mutex, MutexGuard},
};

use anyhow::{anyhow, Context, Result};
use chrono::{Duration, Utc};
use rusqlite::{params, Connection, OpenFlags, Row};
use tauri::{AppHandle, Manager};

use crate::core::types::{CertificateRecord, CertificateSource};

#[derive(Clone)]
pub struct InventoryStore {
    conn: Arc<Mutex<Connection>>,
}

impl InventoryStore {
    pub fn initialize(app: AppHandle) -> Result<Self> {
        let data_dir = app
            .path()
            .app_data_dir()
            .context("failed to resolve app data dir")?;
        fs::create_dir_all(&data_dir)?;

        let db_path = data_dir.join("inventory.sqlite");
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
            CREATE TABLE IF NOT EXISTS certificate_records (
                id TEXT PRIMARY KEY,
                subjects TEXT NOT NULL,
                sans TEXT NOT NULL,
                issuer TEXT NOT NULL,
                serial TEXT NOT NULL,
                not_before TEXT NOT NULL,
                not_after TEXT NOT NULL,
                fingerprint TEXT NOT NULL,
                source TEXT NOT NULL,
                domain_roots TEXT NOT NULL,
                tags TEXT NOT NULL
            );
            "#,
        )?;
        Ok(())
    }

    pub fn list_certificates(&self) -> Result<Vec<CertificateRecord>> {
        let conn = self.lock_conn()?;
        let mut stmt = conn.prepare(
            r#"
            SELECT id, subjects, sans, issuer, serial, not_before, not_after, fingerprint, source, domain_roots, tags
            FROM certificate_records
            ORDER BY not_after DESC
            "#,
        )?;

        let mut rows = stmt.query([])?;
        let mut records = Vec::new();
        while let Some(row) = rows.next()? {
            records.push(Self::row_to_record(row)?);
        }
        Ok(records)
    }

    pub fn get_certificate(&self, id: &str) -> Result<Option<CertificateRecord>> {
        let conn = self.lock_conn()?;
        let mut stmt = conn.prepare(
            r#"
            SELECT id, subjects, sans, issuer, serial, not_before, not_after, fingerprint, source, domain_roots, tags
            FROM certificate_records
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

    pub fn insert_certificate(&self, record: &CertificateRecord) -> Result<()> {
        let mut conn = self.lock_conn()?;
        Self::insert_with_conn(&mut conn, record)
    }

    pub fn seed_dev_certificate(&self) -> Result<()> {
        let mut conn = self.lock_conn()?;
        let count: i64 = conn.query_row("SELECT COUNT(1) FROM certificate_records", [], |row| {
            row.get(0)
        })?;

        if count > 0 {
            return Ok(());
        }

        let now = Utc::now();
        let sample = CertificateRecord {
            id: "cert_demo_edge_001".to_string(),
            subjects: vec![
                "edge.sslboard.test".to_string(),
                "api.sslboard.test".to_string(),
            ],
            sans: vec![
                "edge.sslboard.test".to_string(),
                "api.sslboard.test".to_string(),
            ],
            issuer: "Let's Encrypt (Sandbox)".to_string(),
            serial: "04C8F8A8DFE1B2C7".to_string(),
            not_before: now - Duration::days(30),
            not_after: now + Duration::days(330),
            fingerprint: "15:9A:53:1E:72:2B:B3:91:DD:41:18:52:73:AF:35:A4:10:AC:9C:0A:68:F3:1C:90:E2:8B:F4:0C:CB:12:EF".to_string(),
            source: CertificateSource::Managed,
            domain_roots: vec!["sslboard.test".to_string()],
            tags: vec!["demo".to_string(), "sandbox".to_string()],
        };

        Self::insert_with_conn(&mut conn, &sample)
    }

    fn insert_with_conn(conn: &mut Connection, record: &CertificateRecord) -> Result<()> {
        conn.execute(
            r#"
            INSERT OR REPLACE INTO certificate_records (
                id, subjects, sans, issuer, serial, not_before, not_after, fingerprint, source, domain_roots, tags
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
            "#,
            params![
                record.id,
                serde_json::to_string(&record.subjects)?,
                serde_json::to_string(&record.sans)?,
                record.issuer,
                record.serial,
                record.not_before.to_rfc3339(),
                record.not_after.to_rfc3339(),
                record.fingerprint,
                match record.source {
                    CertificateSource::External => "External",
                    CertificateSource::Managed => "Managed",
                },
                serde_json::to_string(&record.domain_roots)?,
                serde_json::to_string(&record.tags)?,
            ],
        )?;
        Ok(())
    }

    fn row_to_record(row: &Row<'_>) -> Result<CertificateRecord> {
        let id: String = row.get(0)?;
        let subjects_raw: String = row.get(1)?;
        let sans_raw: String = row.get(2)?;
        let issuer: String = row.get(3)?;
        let serial: String = row.get(4)?;
        let not_before_raw: String = row.get(5)?;
        let not_after_raw: String = row.get(6)?;
        let fingerprint: String = row.get(7)?;
        let source_raw: String = row.get(8)?;
        let domain_roots_raw: String = row.get(9)?;
        let tags_raw: String = row.get(10)?;

        let source = match source_raw.as_str() {
            "External" => CertificateSource::External,
            "Managed" => CertificateSource::Managed,
            _ => return Err(anyhow!("Unknown certificate source: {}", source_raw)),
        };

        let not_before = chrono::DateTime::parse_from_rfc3339(&not_before_raw)
            .map(|dt| dt.with_timezone(&Utc))
            .context("failed to parse not_before timestamp")?;
        let not_after = chrono::DateTime::parse_from_rfc3339(&not_after_raw)
            .map(|dt| dt.with_timezone(&Utc))
            .context("failed to parse not_after timestamp")?;

        Ok(CertificateRecord {
            id,
            subjects: serde_json::from_str(&subjects_raw)
                .context("failed to deserialize subjects")?,
            sans: serde_json::from_str(&sans_raw).context("failed to deserialize sans")?,
            issuer,
            serial,
            not_before,
            not_after,
            fingerprint,
            source,
            domain_roots: serde_json::from_str(&domain_roots_raw)
                .context("failed to deserialize domain_roots")?,
            tags: serde_json::from_str(&tags_raw).context("failed to deserialize tags")?,
        })
    }

    fn lock_conn(&self) -> Result<MutexGuard<'_, Connection>> {
        self.conn
            .lock()
            .map_err(|err| anyhow!("SQLite connection poisoned: {err}"))
    }
}
