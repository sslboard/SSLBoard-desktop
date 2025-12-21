use std::{
    collections::HashMap,
    sync::{Mutex, OnceLock},
};

use acme_lib::{
    create_rsa_key,
    order::{Auth, NewOrder},
    persist::{Persist, PersistKey, PersistKind},
    Certificate, Directory, DirectoryUrl, Error as AcmeError,
};
use anyhow::{anyhow, Result};
use chrono::{TimeZone, Utc};
use sha2::{Digest, Sha256};
use uuid::Uuid;
use x509_parser::pem::parse_x509_pem;

use crate::{
    core::types::{CertificateRecord, CertificateSource},
    issuance::dns::{DnsAdapter, DnsChallengeRequest, DnsRecordInstruction, ManualDnsAdapter},
    secrets::{manager::SecretManager, types::SecretKind},
    storage::{dns::DnsConfigStore, inventory::InventoryStore, issuer::IssuerConfigStore},
};

/// In-memory persistence for acme-lib that avoids disk I/O and lets us seed the ACME account key.
#[derive(Clone, Default)]
pub struct EphemeralPersist {
    inner: std::sync::Arc<Mutex<HashMap<String, Vec<u8>>>>,
}

impl EphemeralPersist {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn seed_account_key(&self, realm: &str, pem: &[u8]) -> Result<()> {
        let key = PersistKey::new(realm, PersistKind::AccountPrivateKey, "acme_account");
        self.put(&key, pem).map_err(|e| anyhow!(e.to_string()))
    }
}

impl Persist for EphemeralPersist {
    fn put(&self, key: &PersistKey, value: &[u8]) -> acme_lib::Result<()> {
        let mut lock = self
            .inner
            .lock()
            .map_err(|e| AcmeError::Other(e.to_string()))?;
        lock.insert(key.to_string(), value.to_vec());
        Ok(())
    }

    fn get(&self, key: &PersistKey) -> acme_lib::Result<Option<Vec<u8>>> {
        let lock = self
            .inner
            .lock()
            .map_err(|e| AcmeError::Other(e.to_string()))?;
        Ok(lock.get(&key.to_string()).cloned())
    }
}

struct PendingIssuance {
    order: NewOrder<EphemeralPersist>,
    domains: Vec<String>,
    managed_key_ref: String,
    managed_key_pem: String,
}

static SESSIONS: OnceLock<Mutex<HashMap<String, PendingIssuance>>> = OnceLock::new();

fn sessions() -> &'static Mutex<HashMap<String, PendingIssuance>> {
    SESSIONS.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Starts a managed-key ACME DNS-01 issuance and returns DNS instructions plus a request id.
pub fn start_managed_dns01(
    domains: Vec<String>,
    issuer_id: String,
    issuer_store: &IssuerConfigStore,
    dns_store: &DnsConfigStore,
    secrets: &SecretManager,
) -> Result<(String, Vec<DnsRecordInstruction>)> {
    if domains.is_empty() {
        return Err(anyhow!("At least one domain is required"));
    }
    let mut normalized: Vec<String> = domains
        .into_iter()
        .map(|d| d.trim().trim_end_matches('.').to_lowercase())
        .filter(|d| !d.is_empty())
        .collect();
    normalized.sort();
    normalized.dedup();
    if normalized.is_empty() {
        return Err(anyhow!("No valid domains provided"));
    }

    let issuer = issuer_store
        .get(&issuer_id)?
        .ok_or_else(|| anyhow!("Issuer not found: {}", issuer_id))?;
    if issuer.disabled {
        return Err(anyhow!("Issuer is disabled: {}", issuer_id));
    }
    if !issuer.tos_agreed {
        return Err(anyhow!(
            "Issuer requires Terms of Service acceptance before issuance"
        ));
    }

    let contact_email = issuer
        .contact_email
        .clone()
        .ok_or_else(|| anyhow!("Issuer contact email is required"))?;
    let account_key_ref = issuer
        .account_key_ref
        .clone()
        .ok_or_else(|| anyhow!("Issuer account key ref is missing"))?;
    let account_key_pem = secrets
        .resolve_secret(&account_key_ref)
        .map_err(|e| anyhow!(e.to_string()))?;
    let account_key_pem = String::from_utf8(account_key_pem)
        .map_err(|_| anyhow!("Stored ACME account key is not valid UTF-8"))?;

    let persist = EphemeralPersist::new();
    persist.seed_account_key(&contact_email, account_key_pem.as_bytes())?;

    let directory =
        Directory::from_url(persist.clone(), DirectoryUrl::Other(&issuer.directory_url))
            .map_err(|e: acme_lib::Error| anyhow!(e.to_string()))?;
    let account = directory
        .account_with_realm(
            &contact_email,
            Some(vec![format!("mailto:{}", contact_email)]),
        )
        .map_err(|e: acme_lib::Error| anyhow!(e.to_string()))?;

    let primary = normalized
        .get(0)
        .cloned()
        .ok_or_else(|| anyhow!("primary domain missing"))?;
    let alt_names: Vec<&str> = normalized.iter().skip(1).map(|s| s.as_str()).collect();
    let new_order = account
        .new_order(&primary, &alt_names)
        .map_err(|e: acme_lib::Error| anyhow!(e.to_string()))?;

    let auths: Vec<Auth<EphemeralPersist>> = new_order
        .authorizations()
        .map_err(|e: acme_lib::Error| anyhow!(e.to_string()))?;

    let mut dns_records = Vec::new();
    let adapter = ManualDnsAdapter::new();
    for auth in auths {
        let dns = auth.dns_challenge();
        let proof = dns.dns_proof();
        let domain = auth.domain_name().to_string();

        let mapping = dns_store.find_for_hostname(&domain)?;
        if let Some(mapping) = &mapping {
            if mapping.adapter_id != "manual" {
                return Err(anyhow!(
                    "Only manual DNS adapter is supported for this flow (found {})",
                    mapping.adapter_id
                ));
            }
        }
        let request = DnsChallengeRequest {
            domain: domain.clone(),
            value: proof.clone(),
            zone: mapping.and_then(|m| m.zone),
        };
        let record = adapter.present_txt(&request)?;
        dns_records.push(record);
    }

    let key = create_rsa_key(2048);
    let key_pem = key
        .private_key_to_pem_pkcs8()
        .map_err(|e| anyhow!("failed to serialize private key: {e}"))?;
    let key_pem_str = String::from_utf8(key_pem)
        .map_err(|_| anyhow!("managed key PEM contained invalid UTF-8"))?;
    let key_label = format!("Managed key for {}", primary);
    let managed_key = secrets
        .create_secret(
            SecretKind::ManagedPrivateKey,
            key_label,
            key_pem_str.clone(),
        )
        .map_err(|e| anyhow!(e.to_string()))?;

    let request_id = Uuid::new_v4().to_string();
    let pending = PendingIssuance {
        order: new_order,
        domains: normalized,
        managed_key_ref: managed_key.id.clone(),
        managed_key_pem: key_pem_str,
    };

    sessions()
        .lock()
        .map_err(|e| anyhow!(e.to_string()))?
        .insert(request_id.clone(), pending);

    Ok((request_id, dns_records))
}

/// Finalizes a pending issuance by validating DNS-01, finalizing the order, and persisting metadata.
pub fn complete_managed_dns01(
    request_id: &str,
    inventory: &InventoryStore,
    secrets: &SecretManager,
) -> Result<CertificateRecord> {
    let pending = sessions()
        .lock()
        .map_err(|e| anyhow!(e.to_string()))?
        .remove(request_id)
        .ok_or_else(|| anyhow!("Issuance session not found or already finalized"))?;

    let PendingIssuance {
        mut order,
        domains,
        managed_key_ref,
        managed_key_pem,
    } = pending;

    let auths = order.authorizations().map_err(|e| anyhow!(e.to_string()))?;
    for auth in auths {
        let dns = auth.dns_challenge();
        dns.validate(2000).map_err(|e| anyhow!(e.to_string()))?;
    }

    let csr_order = loop {
        if let Some(csr) = order.confirm_validations() {
            break csr;
        }
        order.refresh().map_err(|e| anyhow!(e.to_string()))?;
    };

    let cert_order = csr_order
        .finalize(&managed_key_pem, 5000)
        .map_err(|e| anyhow!(e.to_string()))?;
    let certificate = cert_order
        .download_and_save_cert()
        .map_err(|e| anyhow!(e.to_string()))?;

    let record = build_record(&certificate, domains, managed_key_ref.clone())?;
    inventory.insert_certificate(&record)?;

    // Best-effort check the key still resolves
    let _ = secrets.resolve_secret(&managed_key_ref);

    Ok(record)
}

fn build_record(
    certificate: &Certificate,
    domains: Vec<String>,
    managed_key_ref: String,
) -> Result<CertificateRecord> {
    let pem = certificate.certificate();
    let (_, pem_block) = parse_x509_pem(pem.as_bytes())
        .map_err(|e| anyhow!("failed to parse issued certificate PEM: {e}"))?;
    let cert = pem_block.parse_x509().map_err(|e| anyhow!(e.to_string()))?;
    let not_before = Utc
        .timestamp_opt(cert.validity().not_before.timestamp(), 0)
        .single()
        .unwrap_or_else(Utc::now);
    let not_after = Utc
        .timestamp_opt(cert.validity().not_after.timestamp(), 0)
        .single()
        .unwrap_or_else(Utc::now);
    let serial = cert.raw_serial_as_string();
    let fingerprint = {
        let mut hasher = Sha256::new();
        hasher.update(cert.as_raw());
        hex::encode(hasher.finalize())
    };

    let sans: Vec<String> = domains.clone();
    let issuer_name = cert.issuer().to_string();

    Ok(CertificateRecord {
        id: format!("cert_{}", Uuid::new_v4().as_simple()),
        subjects: sans.clone(),
        sans,
        issuer: if issuer_name.is_empty() {
            "ACME Issuer".into()
        } else {
            issuer_name
        },
        serial,
        not_before,
        not_after,
        fingerprint,
        source: CertificateSource::Managed,
        domain_roots: domains.iter().map(|d| root_from_hostname(d)).collect(),
        tags: vec![],
        chain_pem: Some(pem.to_string()),
        managed_key_ref: Some(managed_key_ref),
    })
}

fn root_from_hostname(hostname: &str) -> String {
    let parts: Vec<&str> = hostname.trim_end_matches('.').split('.').collect();
    if parts.len() >= 2 {
        format!("{}.{}", parts[parts.len() - 2], parts[parts.len() - 1])
    } else {
        hostname.to_string()
    }
}
