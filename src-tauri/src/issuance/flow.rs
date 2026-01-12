use std::{
    collections::HashMap,
    sync::{Mutex, OnceLock},
    time::Duration,
};

use anyhow::{Result, anyhow};
use chrono::{TimeZone, Utc};
use instant_acme::{ChallengeType, Identifier, Order, OrderStatus, RetryPolicy};
use sha2::{Digest, Sha256};
use uuid::Uuid;
use x509_parser::parse_x509_certificate;

use crate::{
    core::types::{
        CertificateRecord, CertificateSource, CsrMetadata, CsrSource, KeyAlgorithm, KeyCurve,
    },
    issuance::{acme_workflow, csr as csr_tools},
    issuance::dns::DnsRecordInstruction,
    issuance::dns_providers::adapter_for_provider,
    secrets::{manager::SecretManager, types::SecretKind},
    storage::{dns::DnsConfigStore, inventory::InventoryStore, issuer::IssuerConfigStore},
};

struct PendingIssuance {
    order: Order,
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
static CSR_SESSIONS: OnceLock<Mutex<HashMap<String, PendingCsrIssuance>>> = OnceLock::new();

fn sessions() -> &'static Mutex<HashMap<String, PendingIssuance>> {
    SESSIONS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn csr_sessions() -> &'static Mutex<HashMap<String, PendingCsrIssuance>> {
    CSR_SESSIONS.get_or_init(|| Mutex::new(HashMap::new()))
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
    log::info!(
        "[acme] starting managed issuance issuer_id={} domains={:?}",
        issuer_id,
        domains
    );
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
    log::debug!(
        "[acme] resolved key params algorithm={:?} size={:?} curve={:?}",
        key_algorithm,
        key_size,
        key_curve
    );

    let account = tauri::async_runtime::block_on(acme_workflow::setup_acme_account(
        &issuer.directory_url,
        &contact_email,
        &account_key_pem,
    ))?;

    let mut new_order = tauri::async_runtime::block_on(acme_workflow::create_acme_order(
        &account,
        &normalized,
    ))?;

    let (dns_records, dns_records_to_cleanup) =
        tauri::async_runtime::block_on(acme_workflow::prepare_dns_challenges(
            &mut new_order,
            dns_store,
            secrets,
        ))?;
    log::info!(
        "[acme] prepared {} DNS-01 record(s) for issuance",
        dns_records.len()
    );

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
    log::info!("[acme] completing issuance request_id={}", request_id);
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

    let csr_der = acme_workflow::build_csr_der(&managed_key_pem, &domains)?;
    let retry_policy = RetryPolicy::new().timeout(Duration::from_secs(60));

    let chain_pem = tauri::async_runtime::block_on(async {
        log::debug!("[acme] validating DNS-01 challenges");
        let mut authorizations = order.authorizations();
        while let Some(result) = authorizations.next().await {
            let mut authz = result?;
            if matches!(authz.status, instant_acme::AuthorizationStatus::Valid) {
                continue;
            }
            if !matches!(authz.status, instant_acme::AuthorizationStatus::Pending) {
                return Err(anyhow!(
                    "unexpected authorization status for {}",
                    authz.identifier()
                ));
            }

            let mut challenge = authz
                .challenge(ChallengeType::Dns01)
                .ok_or_else(|| anyhow!("no dns01 challenge found"))?;
            let domain = authorized_dns_name(challenge.identifier())?;
            let proof = challenge.key_authorization().dns_value();
            log::debug!("[acme] checking propagation for {}", domain);
            acme_workflow::check_dns_propagation(&domain, &proof)?;

            challenge.set_ready().await?;
        }

        log::debug!("[acme] polling order readiness");
        let status = order.poll_ready(&retry_policy).await?;
        if status != OrderStatus::Ready {
            return Err(anyhow!("unexpected order status: {status:?}"));
        }

        log::debug!("[acme] finalizing order with CSR");
        order.finalize_csr(&csr_der).await?;
        log::debug!("[acme] polling for certificate chain");
        order.poll_certificate(&retry_policy).await.map_err(|e| e.into())
    })?;
    log::info!("[acme] certificate chain received");

    let record = build_record(
        &chain_pem,
        domains,
        Some(managed_key_ref.clone()),
        Some(key_algorithm),
        key_size,
        key_curve,
        None,
        None,
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
                    && resolution.ambiguous.len() <= 1
                {
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

struct PendingCsrIssuance {
    order: Order,
    identifiers: Vec<String>,
    csr_der: Vec<u8>,
    csr_metadata: CsrMetadata,
    csr_source: CsrSource,
    managed_key_ref: Option<String>,
    dns_records_to_cleanup: Vec<(String, String)>,
}

#[allow(clippy::too_many_arguments)]
pub fn start_csr_dns01(
    issuer_id: String,
    csr_path: String,
    csr_source: CsrSource,
    managed_key_ref: Option<String>,
    issuer_store: &IssuerConfigStore,
    dns_store: &DnsConfigStore,
    secrets: &SecretManager,
) -> Result<(String, Vec<DnsRecordInstruction>, CsrMetadata, Vec<String>, Vec<String>)> {
    log::info!(
        "[acme] starting CSR issuance issuer_id={} csr_path={}",
        issuer_id,
        csr_path
    );

    let parsed = csr_tools::load_and_validate_csr(&csr_path)?;

    if let Some(ref key_ref) = managed_key_ref {
        let metadata = secrets
            .get_metadata(key_ref)
            .map_err(|err| anyhow!(err.to_string()))?
            .ok_or_else(|| anyhow!("Managed key reference not found: {}", key_ref))?;
        if !matches!(metadata.kind, SecretKind::ManagedPrivateKey) {
            return Err(anyhow!(
                "CSR managed key reference {} is not a managed private key",
                key_ref
            ));
        }
    }

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

    let account = tauri::async_runtime::block_on(acme_workflow::setup_acme_account(
        &issuer.directory_url,
        &contact_email,
        &account_key_pem,
    ))?;

    let mut new_order = tauri::async_runtime::block_on(acme_workflow::create_acme_order(
        &account,
        &parsed.identifiers,
    ))?;

    let (dns_records, dns_records_to_cleanup) =
        tauri::async_runtime::block_on(acme_workflow::prepare_dns_challenges(
            &mut new_order,
            dns_store,
            secrets,
        ))?;

    let request_id = Uuid::new_v4().to_string();
    let pending = PendingCsrIssuance {
        order: new_order,
        identifiers: parsed.identifiers.clone(),
        csr_der: parsed.der,
        csr_metadata: parsed.metadata.clone(),
        csr_source,
        managed_key_ref,
        dns_records_to_cleanup,
    };

    csr_sessions()
        .lock()
        .map_err(|e| anyhow!(e.to_string()))?
        .insert(request_id.clone(), pending);

    Ok((
        request_id,
        dns_records,
        parsed.metadata,
        parsed.identifiers,
        parsed.warnings,
    ))
}

pub fn complete_csr_dns01(
    request_id: &str,
    inventory: &InventoryStore,
    secrets: &SecretManager,
    dns_store: &DnsConfigStore,
) -> Result<CertificateRecord> {
    log::info!("[acme] completing CSR issuance request_id={}", request_id);
    let pending = csr_sessions()
        .lock()
        .map_err(|e| anyhow!(e.to_string()))?
        .remove(request_id)
        .ok_or_else(|| anyhow!("CSR issuance session not found or already finalized"))?;

    let PendingCsrIssuance {
        mut order,
        identifiers,
        csr_der,
        csr_metadata,
        csr_source,
        managed_key_ref,
        dns_records_to_cleanup,
    } = pending;

    let retry_policy = RetryPolicy::new().timeout(Duration::from_secs(60));
    let chain_pem = tauri::async_runtime::block_on(async {
        log::debug!("[acme] validating DNS-01 challenges for CSR issuance");
        let mut authorizations = order.authorizations();
        while let Some(result) = authorizations.next().await {
            let mut authz = result?;
            if matches!(authz.status, instant_acme::AuthorizationStatus::Valid) {
                continue;
            }
            if !matches!(authz.status, instant_acme::AuthorizationStatus::Pending) {
                return Err(anyhow!(
                    "unexpected authorization status for {}",
                    authz.identifier()
                ));
            }

            let mut challenge = authz
                .challenge(ChallengeType::Dns01)
                .ok_or_else(|| anyhow!("no dns01 challenge found"))?;
            let domain = authorized_dns_name(challenge.identifier())?;
            let proof = challenge.key_authorization().dns_value();
            log::debug!("[acme] checking propagation for {}", domain);
            acme_workflow::check_dns_propagation(&domain, &proof)?;

            challenge.set_ready().await?;
        }

        log::debug!("[acme] polling CSR order readiness");
        let status = order.poll_ready(&retry_policy).await?;
        if status != OrderStatus::Ready {
            return Err(anyhow!("unexpected order status: {status:?}"));
        }

        log::debug!("[acme] finalizing CSR order");
        order.finalize_csr(&csr_der).await?;
        log::debug!("[acme] polling CSR certificate chain");
        order.poll_certificate(&retry_policy).await.map_err(|e| e.into())
    })?;

    let record = build_record(
        &chain_pem,
        identifiers,
        managed_key_ref.clone(),
        Some(csr_metadata.key_algorithm.clone()),
        csr_metadata.key_size,
        csr_metadata.key_curve.clone(),
        Some(csr_metadata),
        Some(csr_source),
    )?;
    inventory.insert_certificate(&record)?;

    if let Some(ref key_ref) = managed_key_ref {
        if let Err(err) = secrets.resolve_secret(key_ref) {
            log::warn!(
                "[issuance] managed key ref {} failed to resolve after CSR issuance: {}",
                key_ref,
                err
            );
        }
    }

    for (domain, record_name) in dns_records_to_cleanup {
        match dns_store.resolve_provider_for_domain(&domain) {
            Ok(resolution) => {
                if let Some(provider) = resolution.provider.as_ref()
                    && resolution.ambiguous.len() <= 1
                {
                    let provider_adapter = adapter_for_provider(provider, secrets);
                    if let Err(err) = provider_adapter.cleanup_txt(&record_name) {
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
    chain_pem: &str,
    domains: Vec<String>,
    managed_key_ref: Option<String>,
    key_algorithm: Option<KeyAlgorithm>,
    key_size: Option<u16>,
    key_curve: Option<KeyCurve>,
    csr_metadata: Option<CsrMetadata>,
    csr_source: Option<CsrSource>,
) -> Result<CertificateRecord> {
    let pem_blocks = pem::parse_many(chain_pem)
        .map_err(|err| anyhow!("failed to parse certificate chain: {err}"))?;
    let first = pem_blocks
        .first()
        .ok_or_else(|| anyhow!("issued certificate chain is empty"))?;
    let (_, cert) = parse_x509_certificate(first.contents())
        .map_err(|e| anyhow!("failed to parse issued certificate DER: {e}"))?;
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
        chain_pem: Some(chain_pem.to_string()),
        managed_key_ref,
        key_algorithm,
        key_size,
        key_curve,
        csr_subject: csr_metadata.as_ref().map(|meta| meta.subject.clone()),
        csr_sans: csr_metadata.as_ref().map(|meta| meta.sans.clone()),
        csr_key_algorithm: csr_metadata.as_ref().map(|meta| meta.key_algorithm.clone()),
        csr_key_size: csr_metadata.as_ref().and_then(|meta| meta.key_size),
        csr_key_curve: csr_metadata.as_ref().and_then(|meta| meta.key_curve.clone()),
        csr_source,
    })
}

fn authorized_dns_name(identifier: &instant_acme::AuthorizedIdentifier<'_>) -> Result<String> {
    match identifier.identifier {
        Identifier::Dns(name) => Ok(name.to_string()),
        _ => Err(anyhow!("Only DNS identifiers are supported for DNS-01")),
    }
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
