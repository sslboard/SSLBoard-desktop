use std::time::Duration;

use acme_lib::{
    Certificate, Directory, DirectoryUrl, create_p256_key, create_p384_key,
    create_rsa_key,
    order::{Auth, NewOrder},
};
use anyhow::{Result, anyhow};

use crate::{
    core::types::{KeyAlgorithm, KeyCurve},
    issuance::dns::{record_name, DnsAdapter, DnsChallengeRequest, DnsRecordInstruction, ManualDnsAdapter, PropagationState},
    issuance::dns_providers::adapter_for_provider,
    secrets::manager::SecretManager,
    storage::dns::DnsConfigStore,
};

use super::flow::EphemeralPersist;

/// Validates and normalizes domain names for certificate issuance.
/// Returns normalized domains or an error if validation fails.
pub fn validate_and_normalize_domains(domains: Vec<String>) -> Result<Vec<String>> {
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

    Ok(normalized)
}

/// Validates key algorithm and size/curve parameters.
/// Returns the resolved parameters or an error if invalid.
pub fn resolve_key_params(
    key_algorithm: Option<KeyAlgorithm>,
    key_size: Option<u16>,
    key_curve: Option<KeyCurve>,
) -> Result<(KeyAlgorithm, Option<u16>, Option<KeyCurve>)> {
    match key_algorithm {
        None => {
            if key_size.is_some() || key_curve.is_some() {
                return Err(anyhow!(
                    "Key parameters must include key_algorithm when size/curve is provided"
                ));
            }
            Ok((KeyAlgorithm::Rsa, Some(2048), None))
        }
        Some(KeyAlgorithm::Rsa) => {
            let size = key_size.ok_or_else(|| anyhow!("RSA key_size is required"))?;
            if !matches!(size, 2048 | 3072 | 4096) {
                return Err(anyhow!(
                    "Unsupported RSA key size {size}. Allowed: 2048, 3072, 4096"
                ));
            }
            if key_curve.is_some() {
                return Err(anyhow!("RSA issuance does not accept key_curve"));
            }
            Ok((KeyAlgorithm::Rsa, Some(size), None))
        }
        Some(KeyAlgorithm::Ecdsa) => {
            if key_size.is_some() {
                return Err(anyhow!("ECDSA issuance does not accept key_size"));
            }
            let curve = key_curve.ok_or_else(|| anyhow!("ECDSA key_curve is required"))?;
            match curve {
                KeyCurve::P256 | KeyCurve::P384 => Ok((KeyAlgorithm::Ecdsa, None, Some(curve))),
            }
        }
    }
}

/// Generates a private key based on the specified algorithm and parameters.
pub fn generate_private_key(
    key_algorithm: &KeyAlgorithm,
    key_size: Option<u16>,
    key_curve: Option<&KeyCurve>,
) -> Result<String> {
    let key = match key_algorithm {
        KeyAlgorithm::Rsa => {
            let size = key_size.unwrap_or(2048);
            create_rsa_key(u32::from(size))
        }
        KeyAlgorithm::Ecdsa => match key_curve {
            Some(KeyCurve::P256) => create_p256_key(),
            Some(KeyCurve::P384) => create_p384_key(),
            None => return Err(anyhow!("ECDSA key_curve is required")),
        },
    };

    let key_pem = key
        .private_key_to_pem_pkcs8()
        .map_err(|e| anyhow!("failed to serialize private key: {e}"))?;

    String::from_utf8(key_pem)
        .map_err(|_| anyhow!("managed key PEM contained invalid UTF-8"))
}

/// Creates an ACME directory connection and account.
/// Returns the directory and account objects.
pub fn setup_acme_account(
    issuer_directory_url: &str,
    contact_email: &str,
    account_key_pem: &str,
) -> Result<(Directory<EphemeralPersist>, acme_lib::Account<EphemeralPersist>)> {
    let persist = EphemeralPersist::new();
    persist.seed_account_key(contact_email, account_key_pem.as_bytes())?;

    let directory = Directory::from_url(persist.clone(), DirectoryUrl::Other(issuer_directory_url))
        .map_err(|e: acme_lib::Error| anyhow!(e.to_string()))?;

    let account = directory
        .account_with_realm(
            contact_email,
            Some(vec![format!("mailto:{}", contact_email)]),
        )
        .map_err(|e: acme_lib::Error| anyhow!(e.to_string()))?;

    Ok((directory, account))
}

/// Creates a new ACME order for the given domains.
/// Returns the new order object.
pub fn create_acme_order(
    account: &acme_lib::Account<EphemeralPersist>,
    domains: &[String],
) -> Result<NewOrder<EphemeralPersist>> {
    let primary = domains
        .first()
        .cloned()
        .ok_or_else(|| anyhow!("primary domain missing"))?;

    let alt_names: Vec<&str> = domains.iter().skip(1).map(|s| s.as_str()).collect();

    account
        .new_order(&primary, &alt_names)
        .map_err(|e: acme_lib::Error| anyhow!(e.to_string()))
}

/// Prepares DNS challenge records for the ACME order.
/// Returns DNS record instructions, authorizations, and records to cleanup.
#[allow(clippy::type_complexity)]
pub fn prepare_dns_challenges(
    order: &NewOrder<EphemeralPersist>,
    dns_store: &DnsConfigStore,
    secrets: &SecretManager,
) -> Result<(Vec<DnsRecordInstruction>, Vec<Auth<EphemeralPersist>>, Vec<(String, String)>)> {
    let auths: Vec<Auth<EphemeralPersist>> = order
        .authorizations()
        .map_err(|e: acme_lib::Error| anyhow!(e.to_string()))?;

    let mut dns_records = Vec::new();
    let mut dns_records_to_cleanup = Vec::new();
    let adapter = ManualDnsAdapter::new();

    for auth in &auths {
        let dns = auth.dns_challenge();
        let proof = dns.dns_proof();
        let domain = auth.domain_name().to_string();

        let resolution = dns_store.resolve_provider_for_domain(&domain)?;
        let zone_override = resolution
            .provider
            .as_ref()
            .and_then(provider_zone_override);

        let request = DnsChallengeRequest {
            domain: domain.clone(),
            value: proof.clone(),
            zone: zone_override,
        };

        let mut record = adapter.present_txt(&request)?;

        if let Some(provider) = resolution.provider.as_ref()
            && resolution.ambiguous.len() <= 1 {
            let provider_adapter = adapter_for_provider(provider, secrets);
            provider_adapter.create_txt(&record.record_name, &record.value)?;
            record.adapter = provider.provider_type.clone();
            // Store for cleanup after successful issuance
            dns_records_to_cleanup.push((domain.clone(), record.record_name.clone()));
        }

        dns_records.push(record);
    }

    Ok((dns_records, auths, dns_records_to_cleanup))
}

/// Validates DNS propagation for all ACME challenges.
/// Returns successfully if all challenges are validated.
pub fn validate_acme_challenges(
    auths: &[Auth<EphemeralPersist>],
) -> Result<()> {
    for auth in auths {
        let dns = auth.dns_challenge();
        dns.validate(2000)
            .map_err(|e| anyhow!(e.to_string()))?;
    }
    Ok(())
}

/// Finalizes the ACME certificate order.
/// Returns the certificate object.
pub fn finalize_acme_certificate(
    mut order: NewOrder<EphemeralPersist>,
    private_key_pem: &str,
) -> Result<Certificate> {
    let csr_order = loop {
        if let Some(csr) = order.confirm_validations() {
            break csr;
        }
        order.refresh()
            .map_err(|e| anyhow!(e.to_string()))?;
    };

    let cert_order = csr_order
        .finalize(private_key_pem, 5000)
        .map_err(|e| anyhow!(e.to_string()))?;

    cert_order
        .download_and_save_cert()
        .map_err(|e| anyhow!(e.to_string()))
}

/// Checks DNS propagation for all challenge records.
/// Returns successfully if all records are propagated.
pub fn check_dns_propagation(
    auths: &[Auth<EphemeralPersist>],
) -> Result<()> {
    for auth in auths {
        let dns = auth.dns_challenge();
        let proof = dns.dns_proof();
        let domain = auth.domain_name().to_string();

        // Poll for DNS propagation with retries
        let timeout = Duration::from_secs(30);
        let interval = Duration::from_secs(2);
        let record_name = record_name(&domain);

        let propagation_result = super::dns_providers::poll_dns_propagation(&record_name, &proof, timeout, interval)?;

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
            super::dns::PropagationState::Pending => {
                return Err(anyhow!(
                    "TXT record not found at {} after {}s. Please wait for DNS propagation and try again.",
                    record_name,
                    timeout.as_secs()
                ));
            }
            super::dns::PropagationState::WrongContent => {
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

    Ok(())
}

fn provider_zone_override(provider: &crate::storage::dns::DnsProvider) -> Option<String> {
    let raw = provider.config_json.as_ref()?;
    match serde_json::from_str::<serde_json::Value>(raw) {
        Ok(value) => value
            .get("zone")
            .and_then(|zone| zone.as_str().map(|s| s.to_string())),
        Err(err) => {
            log::warn!(
                "[dns] invalid provider config_json for {}: {}",
                provider.id,
                err
            );
            None
        }
    }
}
