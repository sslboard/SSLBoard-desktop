use std::{
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, MutexGuard},
    thread,
    time::Duration,
};

use anyhow::{anyhow, Context, Result};
use rusqlite::{params, Connection, OpenFlags};
use tauri::{AppHandle, Manager};

use super::migrations;

#[cfg(windows)]
unsafe extern "system" {
    fn SetFileAttributesW(path: *const u16, attributes: u32) -> i32;
}

#[derive(Clone)]
pub struct Db {
    conn: Arc<Mutex<Connection>>,
    db_path: PathBuf,
}

impl Db {
    pub fn initialize(app: AppHandle) -> Result<Self> {
        let data_dir = app
            .path()
            .app_data_dir()
            .context("failed to resolve app data dir")?;
        Self::initialize_with_path(&data_dir)
    }

    pub fn initialize_with_path(data_dir: &Path) -> Result<Self> {
        fs::create_dir_all(data_dir)?;

        let db_path = data_dir.join("sslboard.sqlite");
        let created = !db_path.exists();
        let mut conn = Connection::open_with_flags(
            &db_path,
            OpenFlags::SQLITE_OPEN_CREATE
                | OpenFlags::SQLITE_OPEN_READ_WRITE
                | OpenFlags::SQLITE_OPEN_FULL_MUTEX,
        )
        .with_context(|| format!("failed to open SQLite database at {}", db_path.display()))?;

        Self::configure_connection(&conn)?;
        migrations::run_all(&conn)?;
        Self::import_legacy_databases(data_dir, &mut conn)?;
        // Re-run lightweight migrations/backfills after importing legacy data.
        migrations::run_all(&conn)?;
        Self::enforce_permissions(&db_path, created)?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            db_path,
        })
    }

    pub fn db_path(&self) -> &Path {
        &self.db_path
    }

    pub fn lock_conn(&self) -> Result<MutexGuard<'_, Connection>> {
        self.conn
            .lock()
            .map_err(|err| anyhow!("SQLite connection poisoned: {err}"))
    }

    fn configure_connection(conn: &Connection) -> Result<()> {
        conn.execute_batch(
            r#"
            PRAGMA journal_mode = WAL;
            PRAGMA foreign_keys = ON;
            "#,
        )?;
        // Ensure we don't crash on transient locks during startup migrations/import.
        conn.busy_timeout(Duration::from_secs(5))
            .context("failed to set SQLite busy timeout")?;
        Ok(())
    }

    fn import_legacy_databases(data_dir: &Path, conn: &mut Connection) -> Result<()> {
        Self::import_legacy_db(conn, &data_dir.join("issuance.sqlite"), &[
            "issuer_configs",
            "dns_providers",
            "dns_zone_mappings",
        ])?;
        Self::import_legacy_db(conn, &data_dir.join("inventory.sqlite"), &["certificate_records"])?;
        Self::import_legacy_db(conn, &data_dir.join("preferences.sqlite"), &["preferences"])?;
        Self::import_legacy_db(conn, &data_dir.join("secrets.sqlite"), &["secret_metadata"])?;
        Ok(())
    }

    fn import_legacy_db(conn: &mut Connection, legacy_path: &Path, tables: &[&str]) -> Result<()> {
        if !legacy_path.exists() {
            return Ok(());
        }

        let legacy_str = legacy_path
            .to_str()
            .ok_or_else(|| anyhow!("legacy db path is not valid utf-8: {}", legacy_path.display()))?;

        let schema = format!(
            "legacy_{}",
            legacy_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("db")
        );

        Self::attach_with_retry(conn, &schema, legacy_str)
            .with_context(|| format!("failed to attach legacy db {}", legacy_path.display()))?;

        let tx = conn
            .transaction()
            .context("failed to start transaction for legacy db import")?;

        for table in tables {
            if !Self::table_exists_in_schema(&tx, &schema, table)? {
                continue;
            }

            match *table {
                "issuer_configs" => {
                    tx.execute_batch(&format!(
                        r#"
                        INSERT OR IGNORE INTO issuer_configs (
                            issuer_id, label, directory_url, environment, issuer_type, params_json,
                            contact_email, account_key_ref, tos_agreed, is_selected, created_at, updated_at
                        )
                        SELECT issuer_id, label, directory_url, environment, issuer_type, params_json,
                               contact_email, account_key_ref, tos_agreed, is_selected, created_at, updated_at
                        FROM {schema}.issuer_configs;
                        "#
                    ))?;
                }
                "dns_providers" => {
                    tx.execute_batch(&format!(
                        r#"
                        INSERT OR IGNORE INTO dns_providers (
                            id, provider_type, label, domain_suffixes, secret_ref, config_json, created_at, updated_at
                        )
                        SELECT id, provider_type, label, domain_suffixes, secret_ref, config_json, created_at, updated_at
                        FROM {schema}.dns_providers;
                        "#
                    ))?;
                }
                "dns_zone_mappings" => {
                    // Imported only to support migrations from older versions. Table is created in unified DB
                    // so the DNS store can perform its existing migration and then ignore it.
                    tx.execute_batch(&format!(
                        r#"
                        INSERT OR IGNORE INTO dns_zone_mappings (
                            hostname_pattern, zone, adapter_id, secret_ref, created_at, updated_at
                        )
                        SELECT hostname_pattern, zone, adapter_id, secret_ref, created_at, updated_at
                        FROM {schema}.dns_zone_mappings;
                        "#
                    ))?;
                }
                "certificate_records" => {
                    tx.execute_batch(&format!(
                        r#"
                        INSERT OR IGNORE INTO certificate_records (
                            id, subjects, sans, issuer, serial, not_before, not_after, fingerprint, source,
                            domain_roots, tags, managed_key_ref, chain_pem, key_algorithm, key_size, key_curve
                        )
                        SELECT id, subjects, sans, issuer, serial, not_before, not_after, fingerprint, source,
                               domain_roots, tags, managed_key_ref, chain_pem, key_algorithm, key_size, key_curve
                        FROM {schema}.certificate_records;
                        "#
                    ))?;
                }
                "preferences" => {
                    tx.execute_batch(&format!(
                        r#"
                        INSERT OR IGNORE INTO preferences (name, value, updated_at)
                        SELECT name, value, updated_at
                        FROM {schema}.preferences;
                        "#
                    ))?;
                }
                "secret_metadata" => {
                    tx.execute_batch(&format!(
                        r#"
                        INSERT OR IGNORE INTO secret_metadata (id, kind, label, created_at, ciphertext)
                        SELECT id, kind, label, created_at, ciphertext
                        FROM {schema}.secret_metadata;
                        "#
                    ))?;
                }
                _ => {}
            }
        }
        tx.commit()?;

        conn.execute(&format!("DETACH DATABASE {schema}"), [])
            .context("failed to detach legacy db")?;

        Self::backup_legacy_db_file(legacy_path)?;
        Ok(())
    }

    fn attach_with_retry(conn: &Connection, schema: &str, legacy_path: &str) -> Result<()> {
        let attach_sql = format!("ATTACH DATABASE ?1 AS {schema}");
        let mut delay = Duration::from_millis(50);

        for attempt in 1..=8 {
            match conn.execute(&attach_sql, params![legacy_path]) {
                Ok(_) => return Ok(()),
                Err(err) => {
                    let is_locked = matches!(
                        err,
                        rusqlite::Error::SqliteFailure(
                            rusqlite::ffi::Error {
                                code: rusqlite::ErrorCode::DatabaseBusy
                                    | rusqlite::ErrorCode::DatabaseLocked,
                                ..
                            },
                            _
                        )
                    );
                    if !is_locked || attempt == 8 {
                        return Err(err.into());
                    }
                    thread::sleep(delay);
                    delay = (delay * 2).min(Duration::from_millis(400));
                }
            }
        }

        Err(anyhow!("failed to attach legacy db after retries"))
    }

    fn table_exists_in_schema(conn: &Connection, schema: &str, table: &str) -> Result<bool> {
        let mut stmt = conn.prepare(&format!(
            r#"
            SELECT name FROM {schema}.sqlite_master
            WHERE type = 'table' AND name = ?1
            "#
        ))?;
        let mut rows = stmt.query(params![table])?;
        Ok(rows.next()?.is_some())
    }

    fn backup_legacy_db_file(path: &Path) -> Result<()> {
        if !path.exists() {
            return Ok(());
        }
        let backup = Self::next_backup_path(path);
        fs::rename(path, &backup)
            .with_context(|| format!("failed to rename {} to {}", path.display(), backup.display()))?;
        Ok(())
    }

    fn next_backup_path(path: &Path) -> PathBuf {
        let base = PathBuf::from(format!("{}.bak", path.display()));
        if !base.exists() {
            return base;
        }
        for idx in 1..1000 {
            let candidate = PathBuf::from(format!("{}.bak{idx}", path.display()));
            if !candidate.exists() {
                return candidate;
            }
        }
        base
    }

    #[cfg(unix)]
    fn enforce_permissions(db_path: &Path, created: bool) -> Result<()> {
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
    fn enforce_permissions(db_path: &Path, _created: bool) -> Result<()> {
        #[cfg(windows)]
        {
            use std::os::windows::ffi::OsStrExt;

            const FILE_ATTRIBUTE_HIDDEN: u32 = 0x2;
            const FILE_ATTRIBUTE_ARCHIVE: u32 = 0x20;
            const FILE_ATTRIBUTE_NOT_CONTENT_INDEXED: u32 = 0x2000;

            let wide: Vec<u16> = db_path
                .as_os_str()
                .encode_wide()
                .chain(std::iter::once(0))
                .collect();

            unsafe {
                if SetFileAttributesW(
                    wide.as_ptr(),
                    FILE_ATTRIBUTE_HIDDEN
                        | FILE_ATTRIBUTE_ARCHIVE
                        | FILE_ATTRIBUTE_NOT_CONTENT_INDEXED,
                ) == 0
                {
                    log::warn!(
                        "[db] warning: failed to harden Windows file attributes for {}: {}",
                        db_path.display(),
                        std::io::Error::last_os_error()
                    );
                }
            }
        }

        Ok(())
    }
}
