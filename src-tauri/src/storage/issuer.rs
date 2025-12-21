//! Issuer configuration storage.
//!
//! Persists issuer selections and ACME account metadata in SQLite so issuer
//! choice and account references survive restarts. This keeps issuer state
//! alongside other local metadata without exposing secrets.

use std::{
    fs,
    sync::{Arc, Mutex, MutexGuard},
};

use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OpenFlags, Row};
use serde_json::json;
use tauri::{AppHandle, Manager};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct IssuerConfigRecord {
    pub issuer_id: String,
    pub label: String,
    pub directory_url: String,
    pub environment: String,
    pub issuer_type: String,
    pub params_json: String,
    pub contact_email: Option<String>,
    pub account_key_ref: Option<String>,
    pub tos_agreed: bool,
    pub is_selected: bool,
    pub disabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// SQLite-backed issuer configuration store.
#[derive(Clone)]
pub struct IssuerConfigStore {
    conn: Arc<Mutex<Connection>>,
}

impl IssuerConfigStore {
    const STAGING_ID: &'static str = "acme_le_staging";
    const PROD_ID: &'static str = "acme_le_prod";

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
        Self::ensure_columns(&conn)?;
        Self::backfill_params_json(&conn)?;
        Self::bootstrap_defaults(&conn)?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn list(&self) -> Result<Vec<IssuerConfigRecord>> {
        eprintln!("[issuer_store] list() begin");
        let conn = self.lock_conn()?;
        let mut records = Self::query_all(&conn)?;

        // Repair/reset defaults if the table was wiped or no issuers exist.
        if records.is_empty() {
            eprintln!("[issuer_store] no rows found, bootstrapping defaults");
            Self::bootstrap_defaults(&conn)?;
            records = Self::query_all(&conn)?;
        }

        eprintln!(
            "[issuer_store] list() returning {} records (selected: {:?})",
            records.len(),
            records
                .iter()
                .find(|r| r.is_selected)
                .map(|r| r.issuer_id.clone())
        );
        Ok(records)
    }

    pub fn get(&self, issuer_id: &str) -> Result<Option<IssuerConfigRecord>> {
        let conn = self.lock_conn()?;
        Self::get_with_conn(&conn, issuer_id)
    }

    pub fn create(
        &self,
        label: String,
        issuer_type: String,
        environment: String,
        directory_url: String,
        contact_email: Option<String>,
        account_key_ref: Option<String>,
        tos_agreed: bool,
    ) -> Result<IssuerConfigRecord> {
        let conn = self.lock_conn()?;
        let now = Utc::now().to_rfc3339();
        let issuer_id = format!("{}_{}", issuer_type, Uuid::new_v4());
        let params_json = Self::build_params_json(&directory_url, &environment)?;

        conn.execute(
            r#"
            INSERT INTO issuer_configs (
                issuer_id, label, directory_url, environment, issuer_type, params_json,
                contact_email, account_key_ref, tos_agreed, is_selected, disabled, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 0, 0, ?10, ?10)
            "#,
            params![
                issuer_id,
                label,
                directory_url,
                environment,
                issuer_type,
                params_json,
                contact_email,
                account_key_ref,
                if tos_agreed { 1 } else { 0 },
                now
            ],
        )?;

        Self::get_with_conn(&conn, &issuer_id)?
            .ok_or_else(|| anyhow!("issuer not found after create: {issuer_id}"))
    }

    pub fn update(
        &self,
        issuer_id: &str,
        label: String,
        environment: String,
        directory_url: String,
        contact_email: Option<String>,
        tos_agreed: bool,
    ) -> Result<IssuerConfigRecord> {
        let conn = self.lock_conn()?;
        let now = Utc::now().to_rfc3339();
        let params_json = Self::build_params_json(&directory_url, &environment)?;

        let updated = conn.execute(
            r#"
            UPDATE issuer_configs
            SET label = ?2,
                environment = ?3,
                directory_url = ?4,
                params_json = ?5,
                contact_email = ?6,
                tos_agreed = ?7,
                updated_at = ?8
            WHERE issuer_id = ?1
            "#,
            params![
                issuer_id,
                label,
                environment,
                directory_url,
                params_json,
                contact_email,
                if tos_agreed { 1 } else { 0 },
                now
            ],
        )?;

        if updated == 0 {
            return Err(anyhow!("issuer not found when updating: {issuer_id}"));
        }

        Self::get_with_conn(&conn, issuer_id)?
            .ok_or_else(|| anyhow!("issuer not found after update: {issuer_id}"))
    }

    pub fn set_disabled(&self, issuer_id: &str, disabled: bool) -> Result<IssuerConfigRecord> {
        let conn = self.lock_conn()?;
        let updated = conn.execute(
            "UPDATE issuer_configs SET disabled = ?2, updated_at = ?3 WHERE issuer_id = ?1",
            params![issuer_id, if disabled { 1 } else { 0 }, Utc::now().to_rfc3339()],
        )?;
        if updated == 0 {
            return Err(anyhow!("issuer not found when updating disabled state: {issuer_id}"));
        }

        Self::get_with_conn(&conn, issuer_id)?
            .ok_or_else(|| anyhow!("issuer not found after disabled update: {issuer_id}"))
    }

    /// Sets the selected issuer, ensuring only one issuer is marked selected.
    pub fn set_selected(&self, issuer_id: &str) -> Result<IssuerConfigRecord> {
        eprintln!("[issuer_store] set_selected({issuer_id})");
        let mut conn = self.lock_conn()?;
        let tx = conn.transaction()?;

        let exists: i64 = tx.query_row(
            "SELECT COUNT(1) FROM issuer_configs WHERE issuer_id = ?1 AND disabled = 0",
            params![issuer_id],
            |row| row.get(0),
        )?;
        if exists == 0 {
            return Err(anyhow!("issuer not found or disabled: {issuer_id}"));
        }

        tx.execute("UPDATE issuer_configs SET is_selected = 0", [])?;
        tx.execute(
            "UPDATE issuer_configs SET is_selected = 1, updated_at = ?2 WHERE issuer_id = ?1",
            params![issuer_id, Utc::now().to_rfc3339()],
        )?;

        tx.commit()?;
        Self::get_with_conn(&conn, issuer_id)?
            .ok_or_else(|| anyhow!("issuer not found after select: {issuer_id}"))
    }

    pub fn upsert_account_state(
        &self,
        issuer_id: &str,
        contact_email: Option<String>,
        account_key_ref: Option<String>,
    ) -> Result<IssuerConfigRecord> {
        eprintln!(
            "[issuer_store] upsert_account_state({}, email_present={}, key_ref_present={})",
            issuer_id,
            contact_email.is_some(),
            account_key_ref.is_some()
        );
        let conn = self.lock_conn()?;
        let now = Utc::now().to_rfc3339();

        let updated = conn.execute(
            r#"
            UPDATE issuer_configs
            SET contact_email = COALESCE(?2, contact_email),
                account_key_ref = COALESCE(?3, account_key_ref),
                updated_at = ?4
            WHERE issuer_id = ?1
            "#,
            params![issuer_id, contact_email, account_key_ref, now],
        )?;

        if updated == 0 {
            return Err(anyhow!("issuer not found when updating account: {issuer_id}"));
        }

        Self::get_with_conn(&conn, issuer_id)?
            .ok_or_else(|| anyhow!("issuer not found after update: {issuer_id}"))
    }

    pub fn set_account_key_ref(
        &self,
        issuer_id: &str,
        account_key_ref: String,
    ) -> Result<IssuerConfigRecord> {
        let conn = self.lock_conn()?;
        let now = Utc::now().to_rfc3339();
        let updated = conn.execute(
            "UPDATE issuer_configs SET account_key_ref = ?2, updated_at = ?3 WHERE issuer_id = ?1",
            params![issuer_id, account_key_ref, now],
        )?;
        if updated == 0 {
            return Err(anyhow!("issuer not found when setting account key: {issuer_id}"));
        }
        Self::get_with_conn(&conn, issuer_id)?
            .ok_or_else(|| anyhow!("issuer not found after account key update: {issuer_id}"))
    }

    pub fn delete(&self, issuer_id: &str) -> Result<()> {
        let conn = self.lock_conn()?;
        let updated = conn.execute(
            "DELETE FROM issuer_configs WHERE issuer_id = ?1",
            params![issuer_id],
        )?;
        if updated == 0 {
            return Err(anyhow!("issuer not found when deleting: {issuer_id}"));
        }
        Ok(())
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

    fn query_all(conn: &Connection) -> Result<Vec<IssuerConfigRecord>> {
        let mut stmt = conn.prepare(
            r#"
            SELECT issuer_id, label, directory_url, environment, issuer_type, params_json,
                   contact_email, account_key_ref, tos_agreed, is_selected, disabled,
                   created_at, updated_at
            FROM issuer_configs
            ORDER BY created_at ASC
            "#,
        )?;

        let mut rows = stmt.query([])?;
        let mut records = Vec::new();
        while let Some(row) = rows.next()? {
            records.push(Self::row_to_record(row)?);
        }
        Ok(records)
    }

    fn get_with_conn(conn: &Connection, issuer_id: &str) -> Result<Option<IssuerConfigRecord>> {
        let mut stmt = conn.prepare(
            r#"
            SELECT issuer_id, label, directory_url, environment, issuer_type, params_json,
                   contact_email, account_key_ref, tos_agreed, is_selected, disabled,
                   created_at, updated_at
            FROM issuer_configs
            WHERE issuer_id = ?1
            "#,
        )?;

        let mut rows = stmt.query(params![issuer_id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_record(row)?))
        } else {
            Ok(None)
        }
    }

    fn init_schema(conn: &Connection) -> Result<()> {
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS issuer_configs (
                issuer_id TEXT PRIMARY KEY,
                label TEXT NOT NULL,
                directory_url TEXT NOT NULL,
                environment TEXT NOT NULL,
                issuer_type TEXT NOT NULL DEFAULT 'acme',
                params_json TEXT NOT NULL DEFAULT '{}',
                contact_email TEXT,
                account_key_ref TEXT,
                tos_agreed INTEGER NOT NULL DEFAULT 0,
                is_selected INTEGER NOT NULL DEFAULT 0,
                disabled INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            "#,
        )?;
        Ok(())
    }

    fn ensure_columns(conn: &Connection) -> Result<()> {
        let mut stmt = conn.prepare("PRAGMA table_info(issuer_configs)")?;
        let columns = stmt
            .query_map([], |row| row.get::<_, String>(1))?
            .collect::<std::result::Result<Vec<String>, _>>()?;

        if !columns.iter().any(|col| col == "issuer_type") {
            conn.execute(
                "ALTER TABLE issuer_configs ADD COLUMN issuer_type TEXT NOT NULL DEFAULT 'acme'",
                [],
            )?;
        }
        if !columns.iter().any(|col| col == "params_json") {
            conn.execute(
                "ALTER TABLE issuer_configs ADD COLUMN params_json TEXT NOT NULL DEFAULT '{}'",
                [],
            )?;
        }

        Ok(())
    }

    fn backfill_params_json(conn: &Connection) -> Result<()> {
        let mut stmt = conn.prepare(
            r#"
            SELECT issuer_id, directory_url, environment
            FROM issuer_configs
            WHERE params_json IS NULL OR params_json = ''
            "#,
        )?;
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })?;

        for row in rows {
            let (issuer_id, directory_url, environment) = row?;
            let params_json = Self::build_params_json(&directory_url, &environment)?;
            conn.execute(
                "UPDATE issuer_configs SET params_json = ?2 WHERE issuer_id = ?1",
                params![issuer_id, params_json],
            )?;
        }
        Ok(())
    }

    fn bootstrap_defaults(conn: &Connection) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        let staging_url = "https://acme-staging-v02.api.letsencrypt.org/directory";
        let prod_url = "https://acme-v02.api.letsencrypt.org/directory";
        let staging_params = Self::build_params_json(staging_url, "staging")?;
        let prod_params = Self::build_params_json(prod_url, "production")?;

        // Ensure staging row exists.
        conn.execute(
            r#"
            INSERT OR IGNORE INTO issuer_configs (
                issuer_id, label, directory_url, environment, issuer_type, params_json,
                contact_email, account_key_ref, tos_agreed, is_selected, disabled, created_at, updated_at
            ) VALUES (?1, ?2, ?3, 'staging', 'acme', ?4, NULL, NULL, 0, 1, 0, ?5, ?5)
            "#,
            params![
                Self::STAGING_ID,
                "Let's Encrypt (Staging)",
                staging_url,
                staging_params,
                now
            ],
        )?;

        // Ensure production row exists but disabled by default.
        conn.execute(
            r#"
            INSERT OR IGNORE INTO issuer_configs (
                issuer_id, label, directory_url, environment, issuer_type, params_json,
                contact_email, account_key_ref, tos_agreed, is_selected, disabled, created_at, updated_at
            ) VALUES (?1, ?2, ?3, 'production', 'acme', ?4, NULL, NULL, 0, 0, 1, ?5, ?5)
            "#,
            params![Self::PROD_ID, "Let's Encrypt (Production)", prod_url, prod_params, now],
        )?;

        // Ensure at least one issuer is marked selected.
        let selected_count: i64 = conn.query_row(
            "SELECT COUNT(1) FROM issuer_configs WHERE is_selected = 1 AND disabled = 0",
            [],
            |row| row.get(0),
        )?;
        if selected_count == 0 {
            conn.execute(
                "UPDATE issuer_configs SET is_selected = 1 WHERE issuer_id = ?1",
                params![Self::STAGING_ID],
            )?;
        }

        Ok(())
    }

    fn row_to_record(row: &Row<'_>) -> Result<IssuerConfigRecord> {
        let created_at_raw: String = row.get(11)?;
        let updated_at_raw: String = row.get(12)?;

        Ok(IssuerConfigRecord {
            issuer_id: row.get(0)?,
            label: row.get(1)?,
            directory_url: row.get(2)?,
            environment: row.get(3)?,
            issuer_type: row.get(4)?,
            params_json: row.get(5)?,
            contact_email: row.get(6)?,
            account_key_ref: row.get(7)?,
            tos_agreed: row.get::<_, i64>(8)? != 0,
            is_selected: row.get::<_, i64>(9)? != 0,
            disabled: row.get::<_, i64>(10)? != 0,
            created_at: chrono::DateTime::parse_from_rfc3339(&created_at_raw)
                .map(|dt| dt.with_timezone(&Utc))
                .context("failed to parse created_at")?,
            updated_at: chrono::DateTime::parse_from_rfc3339(&updated_at_raw)
                .map(|dt| dt.with_timezone(&Utc))
                .context("failed to parse updated_at")?,
        })
    }

    fn build_params_json(directory_url: &str, environment: &str) -> Result<String> {
        serde_json::to_string(&json!({
            "directory_url": directory_url,
            "environment": environment,
        }))
        .context("failed to serialize issuer params")
    }

    fn lock_conn(&self) -> Result<MutexGuard<'_, Connection>> {
        self.conn
            .lock()
            .map_err(|err| anyhow!("SQLite connection poisoned: {err}"))
    }
}
