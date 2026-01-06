use std::{
    sync::MutexGuard,
};

use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, Row};

use super::types::SecretMetadata;
use crate::storage::db::Db;

#[derive(Clone)]
pub struct SecretMetadataStore {
    db: Db,
}

impl SecretMetadataStore {
    pub fn initialize(db: Db) -> Result<Self> {
        Ok(Self { db })
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
            "dns_credential" | "dns_provider_token" => super::types::SecretKind::DnsProviderToken,
            "dns_provider_access_key" => super::types::SecretKind::DnsProviderAccessKey,
            "dns_provider_secret_key" => super::types::SecretKind::DnsProviderSecretKey,
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
        self.db
            .lock_conn()
            .map_err(|err| anyhow!("secrets db mutex poisoned: {err}"))
    }
}
