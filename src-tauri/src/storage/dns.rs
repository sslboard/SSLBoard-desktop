use std::{collections::HashMap, sync::MutexGuard};

use anyhow::{Context, Result, anyhow};
use chrono::{DateTime, Utc};
use log::warn;
use rusqlite::{Connection, Row, params};
use serde_json::Value;
use uuid::Uuid;

use crate::domain::{normalize_domain_for_storage, normalize_domain_suffix_for_storage};
use crate::storage::db::Db;

#[derive(Clone, Debug)]
pub struct DnsProvider {
    pub id: String,
    pub provider_type: String,
    pub label: String,
    pub domain_suffixes: Vec<String>,
    pub secret_refs: Vec<String>, // Changed from Option<String> to Vec<String> to support multiple secrets
    pub config_json: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug)]
pub struct DnsProviderResolution {
    pub provider: Option<DnsProvider>,
    pub matched_suffix: Option<String>,
    pub ambiguous: Vec<DnsProvider>,
}

/// Stores DNS provider configurations and resolves providers for DNS challenges.
#[derive(Clone)]
pub struct DnsConfigStore {
    db: Db,
}

impl DnsConfigStore {
    pub fn initialize(db: Db) -> Result<Self> {
        {
            let conn = db.lock_conn()?;
            Self::migrate_zone_mappings(&conn)?;
        }
        Ok(Self { db })
    }

    pub fn list_providers(&self) -> Result<Vec<DnsProvider>> {
        let conn = self.lock_conn()?;
        let mut stmt = conn.prepare(
            r#"
            SELECT id, provider_type, label, domain_suffixes, secret_ref, config_json, created_at, updated_at
            FROM dns_providers
            ORDER BY created_at DESC
            "#,
        )?;

        let mut rows = stmt.query([])?;
        let mut providers = Vec::new();
        while let Some(row) = rows.next()? {
            providers.push(Self::row_to_provider(row)?);
        }
        Ok(providers)
    }

    pub fn get_provider(&self, provider_id: &str) -> Result<Option<DnsProvider>> {
        let conn = self.lock_conn()?;
        Self::get_provider_with_conn(&conn, provider_id)
    }

    pub fn create_provider(
        &self,
        provider_type: String,
        label: String,
        domain_suffixes: Vec<String>,
        secret_refs: Vec<String>,
        config: Option<Value>,
    ) -> Result<DnsProvider> {
        let conn = self.lock_conn()?;
        let now = Utc::now().to_rfc3339();
        let provider_id = format!("dns_prov_{}", Uuid::new_v4().as_simple());
        let domain_suffixes_json =
            serde_json::to_string(&domain_suffixes).context("failed to serialize suffixes")?;
        let secret_refs_json = if secret_refs.is_empty() {
            None
        } else {
            Some(serde_json::to_string(&secret_refs).context("failed to serialize secret refs")?)
        };
        let config_json = match config {
            Some(value) => {
                Some(serde_json::to_string(&value).context("failed to serialize provider config")?)
            }
            None => None,
        };

        conn.execute(
            r#"
            INSERT INTO dns_providers (
                id, provider_type, label, domain_suffixes, secret_ref, config_json, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)
            "#,
            params![
                provider_id,
                provider_type,
                label,
                domain_suffixes_json,
                secret_refs_json,
                config_json,
                now
            ],
        )?;

        Self::get_provider_with_conn(&conn, &provider_id)?
            .ok_or_else(|| anyhow!("provider not found after create: {provider_id}"))
    }

    pub fn update_provider(
        &self,
        provider_id: &str,
        label: String,
        domain_suffixes: Vec<String>,
        config: Option<Value>,
    ) -> Result<DnsProvider> {
        let conn = self.lock_conn()?;
        let now = Utc::now().to_rfc3339();
        let domain_suffixes_json =
            serde_json::to_string(&domain_suffixes).context("failed to serialize suffixes")?;
        let config_json = match config {
            Some(value) => {
                Some(serde_json::to_string(&value).context("failed to serialize provider config")?)
            }
            None => None,
        };

        let updated = conn.execute(
            r#"
            UPDATE dns_providers
            SET label = ?2,
                domain_suffixes = ?3,
                config_json = ?4,
                updated_at = ?5
            WHERE id = ?1
            "#,
            params![provider_id, label, domain_suffixes_json, config_json, now],
        )?;

        if updated == 0 {
            return Err(anyhow!("provider not found when updating: {provider_id}"));
        }

        Self::get_provider_with_conn(&conn, provider_id)?
            .ok_or_else(|| anyhow!("provider not found after update: {provider_id}"))
    }

    pub fn update_provider_secret_refs(
        &self,
        provider_id: &str,
        secret_refs: Vec<String>,
    ) -> Result<DnsProvider> {
        let conn = self.lock_conn()?;
        let now = Utc::now().to_rfc3339();
        let secret_refs_json = if secret_refs.is_empty() {
            None
        } else {
            Some(serde_json::to_string(&secret_refs).context("failed to serialize secret refs")?)
        };
        let updated = conn.execute(
            r#"
            UPDATE dns_providers
            SET secret_ref = ?2,
                updated_at = ?3
            WHERE id = ?1
            "#,
            params![provider_id, secret_refs_json, now],
        )?;
        if updated == 0 {
            return Err(anyhow!(
                "provider not found when updating secret: {provider_id}"
            ));
        }
        Self::get_provider_with_conn(&conn, provider_id)?
            .ok_or_else(|| anyhow!("provider not found after secret update: {provider_id}"))
    }

    pub fn delete_provider(&self, provider_id: &str) -> Result<DnsProvider> {
        let conn = self.lock_conn()?;
        let existing = Self::get_provider_with_conn(&conn, provider_id)?
            .ok_or_else(|| anyhow!("provider not found when deleting: {provider_id}"))?;
        conn.execute(
            "DELETE FROM dns_providers WHERE id = ?1",
            params![provider_id],
        )?;
        Ok(existing)
    }

    pub fn resolve_provider_for_domain(&self, hostname: &str) -> Result<DnsProviderResolution> {
        let providers = self.list_providers()?;
        let normalized = normalize_hostname(hostname)?;
        let mut matches: Vec<(DnsProvider, String)> = Vec::new();

        for provider in providers {
            for suffix in &provider.domain_suffixes {
                if matches_suffix(&normalized, suffix) {
                    matches.push((provider.clone(), suffix.clone()));
                }
            }
        }

        if matches.is_empty() {
            return Ok(DnsProviderResolution {
                provider: None,
                matched_suffix: None,
                ambiguous: vec![],
            });
        }

        matches.sort_by(|(a_provider, a_suffix), (b_provider, b_suffix)| {
            b_suffix
                .len()
                .cmp(&a_suffix.len())
                .then_with(|| a_provider.id.cmp(&b_provider.id))
        });

        let first_match = matches
            .first()
            .expect("matches should not be empty after empty check");
        let best_suffix_len = first_match.1.len();

        let ambiguous: Vec<DnsProvider> = matches
            .iter()
            .filter(|(_, suffix)| suffix.len() == best_suffix_len)
            .map(|(provider, _)| provider.clone())
            .collect();

        let (provider, matched_suffix) = (first_match.0.clone(), first_match.1.clone());

        Ok(DnsProviderResolution {
            provider: Some(provider),
            matched_suffix: Some(matched_suffix),
            ambiguous,
        })
    }

    fn migrate_zone_mappings(conn: &Connection) -> Result<()> {
        if !Self::table_exists(conn, "dns_zone_mappings")? {
            return Ok(());
        }
        let provider_count = Self::table_count(conn, "dns_providers")?;
        if provider_count > 0 {
            return Ok(());
        }

        let mut stmt = conn.prepare(
            r#"
            SELECT hostname_pattern, zone, adapter_id, secret_ref, created_at, updated_at
            FROM dns_zone_mappings
            "#,
        )?;
        let mut rows = stmt.query([])?;
        let mut groups: HashMap<(String, Option<String>), LegacyProvider> = HashMap::new();

        while let Some(row) = rows.next()? {
            let pattern: String = row.get(0)?;
            let zone: Option<String> = row.get(1)?;
            let adapter_id: String = row.get(2)?;
            let secret_ref: Option<String> = row.get(3)?;
            let created_at_raw: String = row.get(4)?;
            let updated_at_raw: String = row.get(5)?;

            if pattern.trim() == "*" {
                continue;
            }

            let suffix = match normalize_suffix(&pattern) {
                Ok(value) => value,
                Err(err) => {
                    warn!(
                        "[dns] invalid legacy suffix for adapter {}: {}",
                        adapter_id, err
                    );
                    continue;
                }
            };

            let created_at = match DateTime::parse_from_rfc3339(&created_at_raw) {
                Ok(dt) => dt.with_timezone(&Utc),
                Err(err) => {
                    warn!(
                        "[dns] failed to parse legacy created_at for adapter {}: {} (raw={})",
                        adapter_id, err, created_at_raw
                    );
                    Utc::now()
                }
            };
            let updated_at = match DateTime::parse_from_rfc3339(&updated_at_raw) {
                Ok(dt) => dt.with_timezone(&Utc),
                Err(err) => {
                    warn!(
                        "[dns] failed to parse legacy updated_at for adapter {}: {} (raw={})",
                        adapter_id, err, updated_at_raw
                    );
                    Utc::now()
                }
            };

            let entry = groups
                .entry((adapter_id.clone(), secret_ref.clone()))
                .or_insert_with(|| LegacyProvider::new(adapter_id.clone(), secret_ref.clone()));
            entry.domain_suffixes.push(suffix);
            entry.created_at = entry.created_at.min(created_at);
            entry.updated_at = entry.updated_at.max(updated_at);
            if entry.zone_override.is_none() {
                entry.zone_override = zone.clone();
            }
        }

        for (index, (_, legacy)) in groups.into_iter().enumerate() {
            let mut suffixes = legacy.domain_suffixes;
            suffixes.sort();
            suffixes.dedup();
            if suffixes.is_empty() {
                continue;
            }

            let label = if index == 0 {
                format!("Migrated {}", legacy.adapter_id)
            } else {
                format!("Migrated {} ({})", legacy.adapter_id, index + 1)
            };

            let config_json = if let Some(zone) = legacy.zone_override {
                let value = serde_json::json!({ "zone": zone });
                match serde_json::to_string(&value) {
                    Ok(raw) => Some(raw),
                    Err(err) => {
                        warn!(
                            "[dns] failed to serialize legacy zone override for adapter {}: {}",
                            legacy.adapter_id, err
                        );
                        None
                    }
                }
            } else {
                None
            };

            let suffixes_json =
                serde_json::to_string(&suffixes).context("failed to serialize suffixes")?;
            let secret_refs_json = legacy
                .secret_ref
                .map(|ref_id| {
                    serde_json::to_string(&vec![ref_id]).context("failed to serialize secret refs")
                })
                .transpose()?;
            conn.execute(
                r#"
                INSERT INTO dns_providers (
                    id, provider_type, label, domain_suffixes, secret_ref, config_json, created_at, updated_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                "#,
                params![
                    format!("dns_prov_{}", Uuid::new_v4().as_simple()),
                    legacy.adapter_id,
                    label,
                    suffixes_json,
                    secret_refs_json,
                    config_json,
                    legacy.created_at.to_rfc3339(),
                    legacy.updated_at.to_rfc3339()
                ],
            )?;
        }

        Ok(())
    }

    fn table_exists(conn: &Connection, table: &str) -> Result<bool> {
        let mut stmt = conn.prepare(
            r#"
            SELECT name FROM sqlite_master
            WHERE type = 'table' AND name = ?1
            "#,
        )?;
        let mut rows = stmt.query(params![table])?;
        Ok(rows.next()?.is_some())
    }

    fn table_count(conn: &Connection, table: &str) -> Result<i64> {
        let count = conn.query_row(&format!("SELECT COUNT(1) FROM {table}"), [], |row| {
            row.get(0)
        })?;
        Ok(count)
    }

    fn row_to_provider(row: &Row<'_>) -> Result<DnsProvider> {
        let id: String = row.get(0)?;
        let provider_type: String = row.get(1)?;
        let label: String = row.get(2)?;
        let domain_suffixes_raw: String = row.get(3)?;
        let secret_ref_raw: Option<String> = row.get(4)?;
        let config_json: Option<String> = row.get(5)?;
        let created_at_raw: String = row.get(6)?;
        let updated_at_raw: String = row.get(7)?;

        let domain_suffixes: Vec<String> = match serde_json::from_str(&domain_suffixes_raw) {
            Ok(suffixes) => suffixes,
            Err(err) => {
                warn!(
                    "[dns] failed to parse domain_suffixes for provider {}: {}",
                    id, err
                );
                return Err(anyhow!(
                    "failed to parse domain_suffixes for provider {id}: {err}"
                ));
            }
        };

        // Deserialize secret_refs from JSON array, with backward compatibility for old single secret_ref values
        let secret_refs: Vec<String> = match secret_ref_raw {
            Some(ref raw) => {
                let trimmed = raw.trim_start();
                if trimmed.starts_with('[') {
                    match serde_json::from_str::<Vec<String>>(raw) {
                        Ok(refs) => refs,
                        Err(err) => {
                            warn!(
                                "[dns] failed to parse secret_ref list for provider {}: {}",
                                id, err
                            );
                            vec![raw.clone()]
                        }
                    }
                } else {
                    vec![raw.clone()]
                }
            }
            None => Vec::new(),
        };

        let created_at = match chrono::DateTime::parse_from_rfc3339(&created_at_raw) {
            Ok(dt) => dt.with_timezone(&Utc),
            Err(err) => {
                warn!(
                    "[dns] failed to parse created_at for provider {}: {} (raw={})",
                    id, err, created_at_raw
                );
                return Err(anyhow!(
                    "failed to parse created_at for provider {id}: {err}"
                ));
            }
        };
        let updated_at = match chrono::DateTime::parse_from_rfc3339(&updated_at_raw) {
            Ok(dt) => dt.with_timezone(&Utc),
            Err(err) => {
                warn!(
                    "[dns] failed to parse updated_at for provider {}: {} (raw={})",
                    id, err, updated_at_raw
                );
                return Err(anyhow!(
                    "failed to parse updated_at for provider {id}: {err}"
                ));
            }
        };

        Ok(DnsProvider {
            id,
            provider_type,
            label,
            domain_suffixes,
            secret_refs,
            config_json,
            created_at,
            updated_at,
        })
    }

    fn get_provider_with_conn(conn: &Connection, provider_id: &str) -> Result<Option<DnsProvider>> {
        let mut stmt = conn.prepare(
            r#"
            SELECT id, provider_type, label, domain_suffixes, secret_ref, config_json, created_at, updated_at
            FROM dns_providers
            WHERE id = ?1
            "#,
        )?;
        let mut rows = stmt.query(params![provider_id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_provider(row)?))
        } else {
            Ok(None)
        }
    }

    fn lock_conn(&self) -> Result<MutexGuard<'_, Connection>> {
        self.db.lock_conn()
    }
}

#[derive(Clone, Debug)]
struct LegacyProvider {
    adapter_id: String,
    secret_ref: Option<String>,
    domain_suffixes: Vec<String>,
    zone_override: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl LegacyProvider {
    fn new(adapter_id: String, secret_ref: Option<String>) -> Self {
        let now = Utc::now();
        Self {
            adapter_id,
            secret_ref,
            domain_suffixes: Vec::new(),
            zone_override: None,
            created_at: now,
            updated_at: now,
        }
    }
}

pub fn parse_domain_suffixes(raw: &str) -> Result<Vec<String>> {
    let mut suffixes: Vec<String> = Vec::new();
    for entry in raw.split(|ch: char| ch == ',' || ch.is_whitespace()) {
        let normalized = normalize_suffix(entry)?;
        if !normalized.is_empty() {
            suffixes.push(normalized);
        }
    }
    suffixes.sort();
    suffixes.dedup();
    Ok(suffixes)
}

fn normalize_suffix(raw: &str) -> Result<String> {
    normalize_domain_suffix_for_storage(raw)
}

fn normalize_hostname(hostname: &str) -> Result<String> {
    normalize_domain_for_storage(hostname)
}

fn matches_suffix(hostname: &str, suffix: &str) -> bool {
    let suffix = match normalize_suffix(suffix) {
        Ok(value) => value,
        Err(_) => return false,
    };
    if suffix.is_empty() {
        return false;
    }
    hostname == suffix || hostname.ends_with(&format!(".{suffix}"))
}

#[cfg(test)]
mod tests {
    use super::{matches_suffix, normalize_hostname};

    #[test]
    fn matches_idn_suffix_with_unicode_input() {
        let hostname = normalize_hostname("testé.fr").expect("normalize hostname");
        assert!(matches_suffix(&hostname, "testé.fr"));
        assert!(matches_suffix(&hostname, "xn--test-epa.fr"));
    }

    #[test]
    fn matches_idn_suffix_for_subdomain() {
        let hostname = normalize_hostname("sub.testé.fr").expect("normalize hostname");
        assert!(matches_suffix(&hostname, "testé.fr"));
        assert!(matches_suffix(&hostname, "xn--test-epa.fr"));
    }
}
