//! Issuer configuration storage.
//!
//! Persists issuer selections and ACME account metadata in SQLite so issuer
//! choice and account references survive restarts. This keeps issuer state
//! alongside other local metadata without exposing secrets.

use std::{
    sync::MutexGuard,
};

use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use log::debug;
use rusqlite::{params, Connection, Row};
use serde_json::json;
use uuid::Uuid;

use crate::storage::db::Db;

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
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// SQLite-backed issuer configuration store.
#[derive(Clone)]
pub struct IssuerConfigStore {
    db: Db,
}

impl IssuerConfigStore {
    pub fn initialize(db: Db) -> Result<Self> {
        Ok(Self { db })
    }

    pub fn list(&self) -> Result<Vec<IssuerConfigRecord>> {
        debug!("[issuer_store] list() begin");
        let conn = self.lock_conn()?;
        let records = Self::query_all(&conn)?;

        debug!(
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
                contact_email, account_key_ref, tos_agreed, is_selected, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 0, ?10, ?10)
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

    /// Sets the selected issuer, ensuring only one issuer is marked selected.
    pub fn set_selected(&self, issuer_id: &str) -> Result<IssuerConfigRecord> {
        debug!("[issuer_store] set_selected({issuer_id})");
        let mut conn = self.lock_conn()?;
        let tx = conn.transaction()?;

        let exists: i64 =
            tx.query_row("SELECT COUNT(1) FROM issuer_configs WHERE issuer_id = ?1", params![issuer_id], |row| {
                row.get(0)
            })?;
        if exists == 0 {
            return Err(anyhow!("issuer not found: {issuer_id}"));
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

    fn query_all(conn: &Connection) -> Result<Vec<IssuerConfigRecord>> {
        let mut stmt = conn.prepare(
            r#"
            SELECT issuer_id, label, directory_url, environment, issuer_type, params_json,
                   contact_email, account_key_ref, tos_agreed, is_selected, created_at, updated_at
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
                   contact_email, account_key_ref, tos_agreed, is_selected, created_at, updated_at
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

    fn row_to_record(row: &Row<'_>) -> Result<IssuerConfigRecord> {
        let created_at_raw: String = row.get(10)?;
        let updated_at_raw: String = row.get(11)?;

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
        self.db.lock_conn()
    }
}
