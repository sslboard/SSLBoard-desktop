use std::time::Duration;

use anyhow::{Result, anyhow};
use instant_acme::{
    Account, AuthorizationStatus, AuthorizedIdentifier, ChallengeType, Identifier, NewOrder, Order,
};
use rcgen::{CertificateParams, DnType, KeyPair, RsaKeySize, PKCS_ECDSA_P256_SHA256, PKCS_ECDSA_P384_SHA384, PKCS_RSA_SHA256};
use rustls_pki_types::{PrivateKeyDer, PrivatePkcs8KeyDer};
use tauri::async_runtime::spawn_blocking;

use crate::{
    core::types::{KeyAlgorithm, KeyCurve},
    domain::normalize_domain_for_storage,
    issuance::dns::{
        record_name, DnsAdapter, DnsChallengeRequest, DnsRecordInstruction, ManualDnsAdapter,
        PropagationState,
    },
    issuance::dns_providers::adapter_for_provider,
    secrets::manager::SecretManager,
    storage::dns::DnsConfigStore,
};

/// Validates and normalizes domain names for certificate issuance.
/// Returns normalized domains or an error if validation fails.
pub fn validate_and_normalize_domains(domains: Vec<String>) -> Result<Vec<String>> {
    if domains.is_empty() {
        return Err(anyhow!("At least one domain is required"));
    }

    let mut normalized: Vec<String> = Vec::new();
    for domain in domains {
        let trimmed = domain.trim();
        if trimmed.is_empty() {
            continue;
        }
        let ascii = normalize_domain_for_storage(trimmed)
            .map_err(|err| anyhow!("Invalid domain name \"{trimmed}\": {err}"))?;
        if !ascii.is_empty() {
            normalized.push(ascii);
        }
    }

    normalized.sort();
    normalized.dedup();

    if normalized.is_empty() {
        return Err(anyhow!("No valid domains provided"));
    }

    log::debug!(
        "[acme] normalized {} domain(s) for issuance",
        normalized.len()
    );
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
    log::debug!(
        "[acme] generating managed key algorithm={:?} size={:?} curve={:?}",
        key_algorithm,
        key_size,
        key_curve
    );
    let key_pair = match key_algorithm {
        KeyAlgorithm::Rsa => {
            let size = key_size.unwrap_or(2048);
            let rsa_size = match size {
                2048 => RsaKeySize::_2048,
                3072 => RsaKeySize::_3072,
                4096 => RsaKeySize::_4096,
                _ => return Err(anyhow!("Unsupported RSA key size {size}. Allowed: 2048, 3072, 4096")),
            };
            KeyPair::generate_rsa_for(&PKCS_RSA_SHA256, rsa_size)
                .map_err(|err| anyhow!("failed to generate RSA key: {err}"))?
        }
        KeyAlgorithm::Ecdsa => {
            let alg = match key_curve {
                Some(KeyCurve::P256) => &PKCS_ECDSA_P256_SHA256,
                Some(KeyCurve::P384) => &PKCS_ECDSA_P384_SHA384,
                None => return Err(anyhow!("ECDSA key_curve is required")),
            };
            KeyPair::generate_for(alg)
                .map_err(|err| anyhow!("failed to generate ECDSA key: {err}"))?
        }
    };

    let key_pem = key_pair.serialize_pem();
    Ok(key_pem)
}

/// Builds a CSR DER for the provided key and SANs.
pub fn build_csr_der(private_key_pem: &str, sans: &[String]) -> Result<Vec<u8>> {
    let key_pair = KeyPair::from_pem(private_key_pem)
        .map_err(|err| anyhow!("failed to parse private key PEM: {err}"))?;
    let primary = sans
        .first()
        .ok_or_else(|| anyhow!("At least one domain is required for CSR"))?;
    log::debug!(
        "[acme] building CSR with primary CN {} and {} SAN(s)",
        primary,
        sans.len()
    );

    let primary_normalized = normalize_domain_for_storage(primary)
        .map_err(|err| anyhow!("Invalid primary domain \"{primary}\": {err}"))?;

    let mut params = CertificateParams::new(vec![primary_normalized.clone()])
        .map_err(|err| anyhow!("failed to create CSR parameters: {err}"))?;
    params
        .distinguished_name
        .push(DnType::CommonName, primary_normalized.clone());

    // Normalize and deduplicate SAN names
    let mut san_names = Vec::new();
    for name in sans {
        let normalized = normalize_domain_for_storage(name)
            .map_err(|err| anyhow!("Invalid SAN entry \"{name}\": {err}"))?;
        if !normalized.is_empty() && normalized != primary_normalized {
            san_names.push(normalized);
        }
    }
    san_names.sort();
    san_names.dedup();

    if !san_names.is_empty() {
        let mut san_vec = Vec::new();
        for name in &san_names {
            san_vec.push(rcgen::SanType::DnsName(
                rcgen::string::Ia5String::try_from(name.as_str())
                    .map_err(|err| anyhow!("Invalid DNS name for SAN \"{name}\": {err}"))?
            ));
        }
        params.subject_alt_names = san_vec;
    }

    log::debug!(
        "[acme] CSR CN={} SANs={:?}",
        primary_normalized,
        san_names
    );

    let csr = params.serialize_request(&key_pair)
        .map_err(|err| anyhow!("failed to generate CSR: {err}"))?;
    Ok(csr.der().to_vec())
}

/// Builds a CSR DER for the provided key, subject, and SANs.
pub fn build_csr_der_with_subject(
    private_key_pem: &str,
    subject: &str,
    sans: &[String],
) -> Result<Vec<u8>> {
    let key_pair = KeyPair::from_pem(private_key_pem)
        .map_err(|err| anyhow!("failed to parse private key PEM: {err}"))?;
    let subject_normalized = normalize_domain_for_storage(subject)
        .map_err(|err| anyhow!("Invalid CSR subject \"{subject}\": {err}"))?;

    let mut params = CertificateParams::new(Vec::<String>::new())
        .map_err(|err| anyhow!("failed to create CSR parameters: {err}"))?;
    params
        .distinguished_name
        .push(DnType::CommonName, subject_normalized.clone());

    let mut san_names = Vec::new();
    for name in sans {
        let normalized = normalize_domain_for_storage(name)
            .map_err(|err| anyhow!("Invalid SAN entry \"{name}\": {err}"))?;
        if !normalized.is_empty() {
            san_names.push(normalized);
        }
    }
    san_names.sort();
    san_names.dedup();

    if !san_names.is_empty() {
        let mut san_vec = Vec::new();
        for name in &san_names {
            san_vec.push(rcgen::SanType::DnsName(
                rcgen::string::Ia5String::try_from(name.as_str())
                    .map_err(|err| anyhow!("Invalid DNS name for SAN \"{name}\": {err}"))?,
            ));
        }
        params.subject_alt_names = san_vec;
    }

    let csr = params
        .serialize_request(&key_pair)
        .map_err(|err| anyhow!("failed to generate CSR: {err}"))?;
    Ok(csr.der().to_vec())
}

/// Creates an ACME account based on the issuer and account key.
pub async fn setup_acme_account(
    issuer_directory_url: &str,
    contact_email: &str,
    account_key_pem: &str,
) -> Result<Account> {
    log::info!(
        "[acme] initializing account against directory {}",
        issuer_directory_url
    );
    let (key, key_der) = account_key_from_pem(account_key_pem)?;
    let builder = Account::builder()?;
    let (account, _credentials) = builder
        .create_from_key((key, key_der), issuer_directory_url.to_string())
        .await?;

    if !contact_email.trim().is_empty() {
        let contact = format!("mailto:{contact_email}");
        account.update_contacts(&[contact.as_str()]).await?;
        log::debug!("[acme] updated account contact email");
    }

    Ok(account)
}

/// Creates a new ACME order for the given domains.
pub async fn create_acme_order(account: &Account, domains: &[String]) -> Result<Order> {
    let identifiers: Vec<Identifier> = domains
        .iter()
        .map(|domain| Identifier::Dns(domain.clone()))
        .collect();
    log::info!(
        "[acme] creating order for {} identifier(s)",
        identifiers.len()
    );
    let order = account
        .new_order(&NewOrder::new(identifiers.as_slice()))
        .await?;
    Ok(order)
}

/// Prepares DNS challenge records for the ACME order.
/// Returns DNS record instructions and records to cleanup.
#[allow(clippy::type_complexity)]
pub async fn prepare_dns_challenges(
    order: &mut Order,
    dns_store: &DnsConfigStore,
    secrets: &SecretManager,
) -> Result<(Vec<DnsRecordInstruction>, Vec<(String, String)>)> {
    let mut dns_records = Vec::new();
    let mut dns_records_to_cleanup = Vec::new();
    let adapter = ManualDnsAdapter::new();

    log::debug!("[acme] preparing DNS-01 challenges");
    let mut authorizations = order.authorizations();
    while let Some(result) = authorizations.next().await {
        let mut authz = result?;
        match authz.status {
            AuthorizationStatus::Pending => {}
            AuthorizationStatus::Valid => continue,
            _ => {
                return Err(anyhow!("unexpected authorization status for {}", authz.identifier()));
            }
        }

        let challenge = authz
            .challenge(ChallengeType::Dns01)
            .ok_or_else(|| anyhow!("no dns01 challenge found"))?;

        let domain = authorized_dns_name(challenge.identifier())?;
        log::debug!("[acme] preparing challenge for {}", domain);
        let proof = challenge.key_authorization().dns_value();

        let resolution = dns_store.resolve_provider_for_domain(&domain)?;
        let zone_override = resolution
            .provider
            .as_ref()
            .and_then(provider_zone_override);

        let request = DnsChallengeRequest {
            domain: domain.clone(),
            value: proof,
            zone: zone_override,
        };

        let mut record = adapter.present_txt(&request)?;

        if let Some(provider) = resolution.provider.as_ref()
            && resolution.ambiguous.len() <= 1
        {
            log::info!(
                "[dns] creating TXT record via provider {} for {}",
                provider.provider_type,
                domain
            );
            let provider_adapter = adapter_for_provider(provider, secrets);
            let record_name = record.record_name.clone();
            let record_value = record.value.clone();
            let provider_type = provider.provider_type.clone();
            spawn_blocking(move || provider_adapter.create_txt(&record_name, &record_value))
                .await
                .map_err(|err| anyhow!("DNS provider task failed: {err}"))?
                .map_err(|err| anyhow!("Failed to create TXT record via {provider_type}: {err}"))?;
            record.adapter = provider_type;
            dns_records_to_cleanup.push((domain.clone(), record.record_name.clone()));
        }

        dns_records.push(record);
    }

    Ok((dns_records, dns_records_to_cleanup))
}

/// Checks DNS propagation for all challenge records.
/// Returns successfully if all records are propagated.
pub fn check_dns_propagation(
    domain: &str,
    proof: &str,
) -> Result<()> {
    let timeout = Duration::from_secs(30);
    let interval = Duration::from_secs(2);
    let record_name = record_name(domain);

    let propagation_result =
        super::dns_providers::poll_dns_propagation(&record_name, proof, timeout, interval)?;

    match propagation_result.state {
        PropagationState::Found => Ok(()),
        PropagationState::NxDomain => Err(anyhow!(
            "No TXT record found at {} after {}s. Please ensure the DNS record is created and propagated.",
            record_name,
            timeout.as_secs()
        )),
        PropagationState::Pending => Err(anyhow!(
            "TXT record not found at {} after {}s. Please wait for DNS propagation and try again.",
            record_name,
            timeout.as_secs()
        )),
        PropagationState::WrongContent => Err(anyhow!(
            "TXT record at {} has wrong value. Expected: {}. Observed: {:?}",
            record_name,
            proof,
            propagation_result.observed_values
        )),
        PropagationState::Error => Err(anyhow!(
            "Failed to check DNS propagation for {}: {}",
            record_name,
            propagation_result
                .reason
                .unwrap_or_else(|| "Unknown error".to_string())
        )),
    }
}

fn account_key_from_pem(
    account_key_pem: &str,
) -> Result<(instant_acme::Key, PrivateKeyDer<'static>)> {
    let key_pair = KeyPair::from_pem(account_key_pem)
        .map_err(|err| anyhow!("failed to parse account key PEM: {err}"))?;
    
    // Extract PKCS#8 DER from the key pair
    let pkcs8_der = key_pair.serialize_der();

    let key = instant_acme::Key::from_pkcs8_der(PrivatePkcs8KeyDer::from(pkcs8_der.clone()))?;

    Ok((key, PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(pkcs8_der))))
}

fn authorized_dns_name(identifier: &AuthorizedIdentifier<'_>) -> Result<String> {
    match identifier.identifier {
        Identifier::Dns(name) => Ok(name.to_string()),
        _ => Err(anyhow!("Only DNS identifiers are supported for DNS-01")),
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_csr_der_for_domains() {
        let key = generate_private_key(&KeyAlgorithm::Rsa, Some(2048), None).unwrap();
        let csr_der = build_csr_der(
            &key,
            &vec!["example.com".to_string(), "www.example.com".to_string()],
        )
        .unwrap();
        // Verify the CSR DER is non-empty and has reasonable size
        assert!(!csr_der.is_empty());
        assert!(csr_der.len() > 100); // CSRs are typically several hundred bytes
    }

    #[test]
    fn generates_ecdsa_p256_key() {
        let key = generate_private_key(&KeyAlgorithm::Ecdsa, None, Some(&KeyCurve::P256)).unwrap();
        assert!(!key.is_empty());
        assert!(key.contains("BEGIN PRIVATE KEY"));
    }

    #[test]
    fn generates_ecdsa_p384_key() {
        let key = generate_private_key(&KeyAlgorithm::Ecdsa, None, Some(&KeyCurve::P384)).unwrap();
        assert!(!key.is_empty());
        assert!(key.contains("BEGIN PRIVATE KEY"));
    }
}
