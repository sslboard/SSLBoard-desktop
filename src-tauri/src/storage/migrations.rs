use anyhow::{Context, Result};
use rusqlite::Connection;

/// Runs all schema creation and migrations for the unified SQLite database.
pub fn run_all(conn: &Connection) -> Result<()> {
    create_tables(conn)?;
    migrate_tables(conn)?;
    Ok(())
}

fn create_tables(conn: &Connection) -> Result<()> {
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
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS dns_providers (
            id TEXT PRIMARY KEY,
            provider_type TEXT NOT NULL,
            label TEXT NOT NULL,
            domain_suffixes TEXT NOT NULL,
            secret_ref TEXT,
            config_json TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        -- Legacy-only table for older DNS versions; kept to enable migration and import.
        CREATE TABLE IF NOT EXISTS dns_zone_mappings (
            hostname_pattern TEXT NOT NULL,
            zone TEXT,
            adapter_id TEXT NOT NULL,
            secret_ref TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

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
            tags TEXT NOT NULL,
            managed_key_ref TEXT,
            chain_pem TEXT,
            key_algorithm TEXT,
            key_size INTEGER,
            key_curve TEXT
        );

        CREATE TABLE IF NOT EXISTS preferences (
            name TEXT PRIMARY KEY,
            value TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS secret_metadata (
            id TEXT PRIMARY KEY,
            kind TEXT NOT NULL,
            label TEXT NOT NULL,
            created_at TEXT NOT NULL,
            ciphertext BLOB
        );
        "#,
    )?;
    Ok(())
}

fn migrate_tables(conn: &Connection) -> Result<()> {
    ensure_columns(conn, "issuer_configs", &[
        ("issuer_type", "ALTER TABLE issuer_configs ADD COLUMN issuer_type TEXT NOT NULL DEFAULT 'acme'"),
        ("params_json", "ALTER TABLE issuer_configs ADD COLUMN params_json TEXT NOT NULL DEFAULT '{}'"),
    ])?;
    ensure_columns(conn, "certificate_records", &[
        ("managed_key_ref", "ALTER TABLE certificate_records ADD COLUMN managed_key_ref TEXT"),
        ("chain_pem", "ALTER TABLE certificate_records ADD COLUMN chain_pem TEXT"),
        ("key_algorithm", "ALTER TABLE certificate_records ADD COLUMN key_algorithm TEXT"),
        ("key_size", "ALTER TABLE certificate_records ADD COLUMN key_size INTEGER"),
        ("key_curve", "ALTER TABLE certificate_records ADD COLUMN key_curve TEXT"),
    ])?;
    ensure_columns(conn, "secret_metadata", &[(
        "ciphertext",
        "ALTER TABLE secret_metadata ADD COLUMN ciphertext BLOB",
    )])?;

    backfill_issuer_params_json(conn)?;
    migrate_dns_credential_kind(conn)?;

    Ok(())
}

fn ensure_columns(conn: &Connection, table: &str, alters: &[(&str, &str)]) -> Result<()> {
    let mut stmt = conn
        .prepare(&format!("PRAGMA table_info({table})"))
        .with_context(|| format!("failed to introspect table {table}"))?;
    let mut rows = stmt.query([])?;
    let mut existing = Vec::new();
    while let Some(row) = rows.next()? {
        let name: String = row.get(1)?;
        existing.push(name);
    }

    for (column, alter_sql) in alters {
        if !existing.iter().any(|c| c == column) {
            conn.execute(alter_sql, [])
                .with_context(|| format!("failed to apply migration for {table}.{column}"))?;
        }
    }

    Ok(())
}

fn backfill_issuer_params_json(conn: &Connection) -> Result<()> {
    // Keep this lightweight and safe: if params_json is missing/empty, fill it from existing fields.
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
        let params_json = serde_json::to_string(&serde_json::json!({
            "directory_url": directory_url,
            "environment": environment,
        }))
        .context("failed to serialize issuer params")?;
        conn.execute(
            "UPDATE issuer_configs SET params_json = ?2 WHERE issuer_id = ?1",
            rusqlite::params![issuer_id, params_json],
        )?;
    }
    Ok(())
}

fn migrate_dns_credential_kind(conn: &Connection) -> Result<()> {
    conn.execute(
        "UPDATE secret_metadata SET kind = 'dns_provider_token' WHERE kind = 'dns_credential'",
        [],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::OpenFlags;

    #[test]
    fn runs_on_empty_database() -> Result<()> {
        let conn = Connection::open_with_flags(
            ":memory:",
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE,
        )?;
        run_all(&conn)?;
        Ok(())
    }
}
