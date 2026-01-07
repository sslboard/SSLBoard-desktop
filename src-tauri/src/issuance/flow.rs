use std::{
    collections::HashMap,
    sync::{Mutex, OnceLock},
    time::Duration,
};

use acme_lib::{
    Certificate, Error as AcmeError,
    order::NewOrder,
    persist::{Persist, PersistKey, PersistKind},
};
use anyhow::{Result, anyhow};
use chrono::{TimeZone, Utc};
use sha2::{Digest, Sha256};
use uuid::Uuid;
use x509_parser::pem::parse_x509_pem;

use crate::{
    core::types::{CertificateRecord, CertificateSource, KeyAlgorithm, KeyCurve},
    issuance::acme_workflow,
    issuance::dns::{record_name, DnsRecordInstruction, PropagationState},
    issuance::dns_providers::{adapter_for_provider, poll_dns_propagation},
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
    key_algorithm: KeyAlgorithm,
    key_size: Option<u16>,
    key_curve: Option<KeyCurve>,
    /// DNS records that were automatically created and need cleanup after issuance
    dns_records_to_cleanup: Vec<(String, String)>, // (domain, record_name)
}

static SESSIONS: OnceLock<Mutex<HashMap<String, PendingIssuance>>> = OnceLock::new();

fn sessions() -> &'static Mutex<HashMap<String, PendingIssuance>> {
    SESSIONS.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Starts a managed-key ACME DNS-01 issuance and returns DNS instructions plus a request id.
#[allow(clippy::too_many_arguments)]
pub fn start_managed_dns01(
    domains: Vec<String>,
    issuer_id: String,
    key_algorithm: Option<KeyAlgorithm>,
    key_size: Option<u16>,
    key_curve: Option<KeyCurve>,
    issuer_store: &IssuerConfigStore,
    dns_store: &DnsConfigStore,
    secrets: &SecretManager,
) -> Result<(String, Vec<DnsRecordInstruction>)> {
    let normalized = acme_workflow::validate_and_normalize_domains(domains)?;

    let issuer = issuer_store
        .get(&issuer_id)?
        .ok_or_else(|| anyhow!("Issuer not found: {}", issuer_id))?;
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

    let (key_algorithm, key_size, key_curve) =
        acme_workflow::resolve_key_params(key_algorithm, key_size, key_curve)?;

    let (_directory, account) =
        acme_workflow::setup_acme_account(&issuer.directory_url, &contact_email, &account_key_pem)?;

    let new_order = acme_workflow::create_acme_order(&account, &normalized)?;

    let (dns_records, _auths, dns_records_to_cleanup) =
        acme_workflow::prepare_dns_challenges(&new_order, dns_store, secrets)?;

    let primary = normalized
        .first()
        .cloned()
        .ok_or_else(|| anyhow!("primary domain missing"))?;
    let key_pem_str =
        acme_workflow::generate_private_key(&key_algorithm, key_size, key_curve.as_ref())?;
    let key_label = format!(
        "Managed {} key for {}",
        format_key_label(&key_algorithm, key_size, key_curve.as_ref()),
        primary
    );
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
        key_algorithm,
        key_size,
        key_curve,
        dns_records_to_cleanup,
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
    dns_store: &DnsConfigStore,
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
        key_algorithm,
        key_size,
        key_curve,
        dns_records_to_cleanup,
    } = pending;

    let auths = order.authorizations().map_err(|e| anyhow!(e.to_string()))?;
    for auth in &auths {
        let dns = auth.dns_challenge();
        let proof = dns.dns_proof();
        let domain = auth.domain_name().to_string();

        // Poll for DNS propagation with retries using unified retry logic
        let timeout = Duration::from_secs(30);
        let interval = Duration::from_secs(2);
        let record_name = record_name(&domain);

        let propagation_result = poll_dns_propagation(&record_name, &proof, timeout, interval)?;

        // Check final state after polling
        match propagation_result.state {
            PropagationState::Found => {
                // Already handled in loop, continue to next domain
            }
            PropagationState::NxDomain => {
                return Err(anyhow!(
                    "No TXT record found at {} after {}s. Please ensure the DNS record is created and propagated.",
                    record_name,
                    timeout.as_secs()
                ));
            }
            PropagationState::Pending => {
                return Err(anyhow!(
                    "TXT record not found at {} after {}s. Please wait for DNS propagation and try again.",
                    record_name,
                    timeout.as_secs()
                ));
            }
            PropagationState::WrongContent => {
                // Should have been caught in loop, but handle just in case
                return Err(anyhow!(
                    "TXT record at {} has wrong value. Expected: {}. Observed: {:?}",
                    record_name,
                    proof,
                    propagation_result.observed_values
                ));
            }
            PropagationState::Error => {
                return Err(anyhow!(
                    "Failed to check DNS propagation for {}: {}",
                    record_name,
                    propagation_result
                        .reason
                        .unwrap_or_else(|| "Unknown error".to_string())
                ));
            }
        }
    }

    // All DNS records are present, proceed with ACME validation
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

    let record = build_record(
        &certificate,
        domains,
        managed_key_ref.clone(),
        key_algorithm,
        key_size,
        key_curve,
    )?;
    inventory.insert_certificate(&record)?;

    // Best-effort check the key still resolves
    if let Err(err) = secrets.resolve_secret(&managed_key_ref) {
        log::warn!(
            "[issuance] managed key ref {} failed to resolve after issuance: {}",
            managed_key_ref,
            err
        );
    }

    // Clean up DNS challenge records after successful issuance
    for (domain, record_name) in dns_records_to_cleanup {
        match dns_store.resolve_provider_for_domain(&domain) {
            Ok(resolution) => {
                if let Some(provider) = resolution.provider.as_ref()
                    && resolution.ambiguous.len() <= 1 {
                    let provider_adapter = adapter_for_provider(provider, secrets);
                    if let Err(err) = provider_adapter.cleanup_txt(&record_name) {
                        // Log but don't fail issuance if cleanup fails
                        log::warn!(
                            "[dns] Failed to cleanup TXT record {} for domain {}: {}",
                            record_name,
                            domain,
                            err
                        );
                    } else {
                        log::debug!(
                            "[dns] Successfully cleaned up TXT record {} for domain {}",
                            record_name,
                            domain
                        );
                    }
                }
            }
            Err(err) => {
                log::warn!(
                    "[dns] Failed to resolve provider for cleanup {}: {}",
                    domain,
                    err
                );
            }
        }
    }

    Ok(record)
}

fn build_record(
    certificate: &Certificate,
    domains: Vec<String>,
    managed_key_ref: String,
    key_algorithm: KeyAlgorithm,
    key_size: Option<u16>,
    key_curve: Option<KeyCurve>,
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
        key_algorithm: Some(key_algorithm),
        key_size,
        key_curve,
    })
}

fn format_key_label(
    key_algorithm: &KeyAlgorithm,
    key_size: Option<u16>,
    key_curve: Option<&KeyCurve>,
) -> String {
    match key_algorithm {
        KeyAlgorithm::Rsa => {
            let size = key_size.unwrap_or(2048);
            format!("RSA {}", size)
        }
        KeyAlgorithm::Ecdsa => match key_curve {
            Some(KeyCurve::P256) => "ECDSA P-256".to_string(),
            Some(KeyCurve::P384) => "ECDSA P-384".to_string(),
            None => "ECDSA".to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{KeyAlgorithm, KeyCurve};
    use crate::issuance::acme_workflow;

    #[test]
    fn defaults_to_rsa_2048_when_missing() {
        let (algo, size, curve) = acme_workflow::resolve_key_params(None, None, None).unwrap();
        assert!(matches!(algo, KeyAlgorithm::Rsa));
        assert_eq!(size, Some(2048));
        assert!(curve.is_none());
    }

    #[test]
    fn accepts_rsa_3072() {
        let (algo, size, curve) =
            acme_workflow::resolve_key_params(Some(KeyAlgorithm::Rsa), Some(3072), None).unwrap();
        assert!(matches!(algo, KeyAlgorithm::Rsa));
        assert_eq!(size, Some(3072));
        assert!(curve.is_none());
    }

    #[test]
    fn accepts_ecdsa_p384() {
        let (algo, size, curve) = acme_workflow::resolve_key_params(
            Some(KeyAlgorithm::Ecdsa),
            None,
            Some(KeyCurve::P384),
        )
        .unwrap();
        assert!(matches!(algo, KeyAlgorithm::Ecdsa));
        assert!(size.is_none());
        assert!(matches!(curve, Some(KeyCurve::P384)));
    }

    #[test]
    fn rejects_invalid_size() {
        let err = acme_workflow::resolve_key_params(Some(KeyAlgorithm::Rsa), Some(1024), None)
            .unwrap_err();
        assert!(err.to_string().contains("Unsupported RSA key size"));
    }
}

fn root_from_hostname(hostname: &str) -> String {
    let parts: Vec<&str> = hostname.trim_end_matches('.').split('.').collect();
    if parts.len() >= 2 {
        format!("{}.{}", parts[parts.len() - 2], parts[parts.len() - 1])
    } else {
        hostname.to_string()
    }
}
