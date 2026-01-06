//! Certificate inventory storage module.
//!
//! This module provides persistent storage for SSL/TLS certificate records
//! using SQLite as the backend. It handles certificate metadata storage,
//! retrieval, and basic inventory management operations.

use std::{
    sync::MutexGuard,
};

use anyhow::{anyhow, Context, Result};
use chrono::{Duration, Utc};
use rusqlite::{params, Connection, Row};

use crate::core::types::{CertificateRecord, CertificateSource, KeyAlgorithm, KeyCurve};
use crate::storage::db::Db;

/// SQLite-based storage for certificate inventory data.
/// Provides thread-safe access to certificate records with CRUD operations.
///
/// The store uses a single SQLite database file stored in the application's
/// data directory. All operations are protected by a mutex to ensure
/// thread safety across async operations.
#[derive(Clone)]
pub struct InventoryStore {
    db: Db,
}

impl InventoryStore {
    pub fn initialize(db: Db) -> Result<Self> {
        Ok(Self { db })
    }

    /// Retrieves all certificate records from the inventory.
    ///
    /// Returns all certificate records ordered by expiration date (newest first).
    /// This provides a complete view of all tracked certificates.
    ///
    /// # Returns
    /// A Result containing a vector of all CertificateRecord instances or an error
    ///
    /// # Errors
    /// Returns an error if the database query fails or record deserialization fails
    pub fn list_certificates(&self) -> Result<Vec<CertificateRecord>> {
        let conn = self.lock_conn()?;
        let mut stmt = conn.prepare(
            r#"
            SELECT id, subjects, sans, issuer, serial, not_before, not_after, fingerprint, source, domain_roots, tags, managed_key_ref, chain_pem
            , key_algorithm, key_size, key_curve FROM certificate_records
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

    /// Retrieves a specific certificate record by its unique ID.
    ///
    /// Looks up a single certificate record in the database using its ID.
    /// Returns None if no certificate with the given ID exists.
    ///
    /// # Arguments
    /// * `id` - The unique identifier of the certificate to retrieve
    ///
    /// # Returns
    /// A Result containing Some(CertificateRecord) if found, None if not found, or an error
    ///
    /// # Errors
    /// Returns an error if the database query fails or record deserialization fails
    pub fn get_certificate(&self, id: &str) -> Result<Option<CertificateRecord>> {
        let conn = self.lock_conn()?;
        let mut stmt = conn.prepare(
            r#"
            SELECT id, subjects, sans, issuer, serial, not_before, not_after, fingerprint, source, domain_roots, tags, managed_key_ref, chain_pem
            , key_algorithm, key_size, key_curve FROM certificate_records
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

    /// Inserts or replaces a certificate record in the inventory.
    ///
    /// Stores a certificate record in the database. If a record with the same ID
    /// already exists, it will be replaced (upsert behavior).
    ///
    /// # Arguments
    /// * `record` - The certificate record to insert
    ///
    /// # Returns
    /// A Result indicating success or failure of the insertion
    ///
    /// # Errors
    /// Returns an error if the database operation fails or serialization fails
    pub fn insert_certificate(&self, record: &CertificateRecord) -> Result<()> {
        let mut conn = self.lock_conn()?;
        Self::insert_with_conn(&mut conn, record)
    }

    /// Seeds the database with a sample development certificate.
    ///
    /// Inserts a fake certificate record for development and testing purposes.
    /// Only adds the sample certificate if the inventory is currently empty.
    /// This helps with UI development and testing without requiring real certificates.
    ///
    /// # Returns
    /// A Result indicating success or failure of the seeding operation
    ///
    /// # Errors
    /// Returns an error if the database operation fails or if insertion fails
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
            managed_key_ref: None,
            chain_pem: None,
            key_algorithm: None,
            key_size: None,
            key_curve: None,
        };

        Self::insert_with_conn(&mut conn, &sample)
    }

    /// Inserts a certificate record using an existing database connection.
    ///
    /// Internal helper method that performs the actual database insertion.
    /// Serializes complex fields (vectors) to JSON for storage.
    ///
    /// # Arguments
    /// * `conn` - Mutable reference to the SQLite connection
    /// * `record` - The certificate record to insert
    ///
    /// # Returns
    /// A Result indicating success or failure of the insertion
    ///
    /// # Errors
    /// Returns an error if serialization fails or the database operation fails
    fn insert_with_conn(conn: &mut Connection, record: &CertificateRecord) -> Result<()> {
        conn.execute(
            r#"
            INSERT OR REPLACE INTO certificate_records (
                id, subjects, sans, issuer, serial, not_before, not_after, fingerprint, source, domain_roots, tags, managed_key_ref, chain_pem, key_algorithm, key_size, key_curve
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)
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
                record.managed_key_ref,
                record.chain_pem,
                key_algorithm_to_db(&record.key_algorithm),
                record.key_size,
                key_curve_to_db(&record.key_curve),
            ],
        )?;
        Ok(())
    }

    /// Converts a database row into a CertificateRecord struct.
    ///
    /// Deserializes JSON fields and parses timestamps from RFC3339 format.
    /// Handles conversion from database column types to Rust types.
    ///
    /// # Arguments
    /// * `row` - The SQLite row to convert
    ///
    /// # Returns
    /// A Result containing the deserialized CertificateRecord or an error
    ///
    /// # Errors
    /// Returns an error if deserialization fails, timestamp parsing fails,
    /// or unknown enum values are encountered
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
        let managed_key_ref: Option<String> = row.get(11).unwrap_or(None);
        let chain_pem: Option<String> = row.get(12).unwrap_or(None);
        let key_algorithm_raw: Option<String> = row.get(13).unwrap_or(None);
        let key_size: Option<u16> = row.get(14).unwrap_or(None);
        let key_curve_raw: Option<String> = row.get(15).unwrap_or(None);

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
            managed_key_ref,
            chain_pem,
            key_algorithm: parse_key_algorithm(key_algorithm_raw)?,
            key_size,
            key_curve: parse_key_curve(key_curve_raw)?,
        })
    }

    /// Acquires a lock on the database connection for thread-safe access.
    ///
    /// Returns a mutex guard that provides exclusive access to the SQLite connection.
    /// This ensures thread safety when performing database operations.
    ///
    /// # Returns
    /// A Result containing a MutexGuard for the connection or an error if the mutex is poisoned
    ///
    /// # Errors
    /// Returns an error if the mutex has been poisoned by a previous panic
    fn lock_conn(&self) -> Result<MutexGuard<'_, Connection>> {
        self.db.lock_conn()
    }
}

fn key_algorithm_to_db(value: &Option<KeyAlgorithm>) -> Option<String> {
    value.as_ref().map(|alg| match alg {
        KeyAlgorithm::Rsa => "rsa".to_string(),
        KeyAlgorithm::Ecdsa => "ecdsa".to_string(),
    })
}

fn key_curve_to_db(value: &Option<KeyCurve>) -> Option<String> {
    value.as_ref().map(|curve| match curve {
        KeyCurve::P256 => "p256".to_string(),
        KeyCurve::P384 => "p384".to_string(),
    })
}

fn parse_key_algorithm(raw: Option<String>) -> Result<Option<KeyAlgorithm>> {
    match raw {
        None => Ok(None),
        Some(value) => match value.as_str() {
            "rsa" => Ok(Some(KeyAlgorithm::Rsa)),
            "ecdsa" => Ok(Some(KeyAlgorithm::Ecdsa)),
            _ => Err(anyhow!("Unknown key algorithm: {}", value)),
        },
    }
}

fn parse_key_curve(raw: Option<String>) -> Result<Option<KeyCurve>> {
    match raw {
        None => Ok(None),
        Some(value) => match value.as_str() {
            "p256" => Ok(Some(KeyCurve::P256)),
            "p384" => Ok(Some(KeyCurve::P384)),
            _ => Err(anyhow!("Unknown key curve: {}", value)),
        },
    }
}
