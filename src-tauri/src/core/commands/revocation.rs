use anyhow::{Result, anyhow};
use chrono::Utc;
use instant_acme::{RevocationReason, RevocationRequest};
use rustls_pki_types::CertificateDer;
use tauri::{State, async_runtime::spawn_blocking};

use crate::core::types::{CertificateRecord, CertificateSource, RevokeCertificateRequest};
use crate::domain::normalize_domains_for_display;
use crate::issuance::acme_workflow;
use crate::secrets::manager::SecretManager;
use crate::storage::{inventory::InventoryStore, issuer::IssuerConfigStore};

#[tauri::command]
pub async fn revoke_certificate(
    inventory: State<'_, InventoryStore>,
    issuer_store: State<'_, IssuerConfigStore>,
    secrets: State<'_, SecretManager>,
    revoke_req: RevokeCertificateRequest,
) -> Result<CertificateRecord, String> {
    let inventory = inventory.inner().clone();
    let issuer_store = issuer_store.inner().clone();
    let secrets = secrets.inner().clone();
    spawn_blocking(move || {
        revoke_certificate_inner(&inventory, &issuer_store, &secrets, revoke_req)
    })
    .await
    .map_err(|err| format!("Revoke join error: {err}"))?
    .map_err(|err| err.to_string())
}

fn revoke_certificate_inner(
    inventory: &InventoryStore,
    issuer_store: &IssuerConfigStore,
    secrets: &SecretManager,
    revoke_req: RevokeCertificateRequest,
) -> Result<CertificateRecord> {
    log::info!(
        "[revocation] request certificate_id={} reason={}",
        revoke_req.certificate_id,
        revoke_req
            .revocation_reason
            .as_deref()
            .unwrap_or("unspecified")
    );
    let mut record = inventory
        .get_certificate(&revoke_req.certificate_id)?
        .ok_or_else(|| anyhow!("Certificate not found: {}", revoke_req.certificate_id))?;

    log::debug!(
        "[revocation] record source={:?} issuer_id={:?} revoked_at={:?} managed_key_ref_present={} chain_pem_present={}",
        record.source,
        record.issuer_id,
        record.revoked_at,
        record.managed_key_ref.is_some(),
        record.chain_pem.is_some()
    );

    if !matches!(record.source, CertificateSource::Managed) {
        return Err(anyhow!("Only Managed certificates can be revoked"));
    }
    if record.revoked_at.is_some() {
        return Err(anyhow!("Certificate is already revoked"));
    }

    let issuer_id = record
        .issuer_id
        .clone()
        .ok_or_else(|| anyhow!("Certificate issuer could not be determined"))?;
    let issuer = issuer_store
        .get(&issuer_id)?
        .ok_or_else(|| anyhow!("Issuer not found: {}", issuer_id))?;

    log::debug!(
        "[revocation] issuer loaded issuer_id={} account_key_ref_present={}",
        issuer_id,
        issuer.account_key_ref.is_some()
    );

    if record.managed_key_ref.is_none() && issuer.account_key_ref.is_none() {
        return Err(anyhow!(
            "Revocation requires either the certificate private key or issuer account key"
        ));
    }

    let account_key_ref = issuer
        .account_key_ref
        .clone()
        .ok_or_else(|| anyhow!("Issuer account key ref is missing"))?;
    let account_key_pem = secrets
        .resolve_secret(&account_key_ref)
        .map_err(|e| anyhow!(e.to_string()))?;
    let account_key_pem = String::from_utf8(account_key_pem)
        .map_err(|_| anyhow!("Stored ACME account key is not valid UTF-8"))?;

    let chain_pem = record
        .chain_pem
        .as_ref()
        .ok_or_else(|| anyhow!("Certificate chain PEM is missing"))?;
    let pem_blocks = pem::parse_many(chain_pem)
        .map_err(|err| anyhow!("failed to parse certificate chain: {err}"))?;
    let leaf = pem_blocks
        .first()
        .ok_or_else(|| anyhow!("certificate chain is empty"))?;
    let cert_der = CertificateDer::from(leaf.contents().to_vec());

    let (reason_label, reason) = parse_revocation_reason(revoke_req.revocation_reason)?;
    let payload = RevocationRequest {
        certificate: &cert_der,
        reason: Some(reason),
    };

    let contact_email = issuer.contact_email.clone().unwrap_or_default();
    let account = tauri::async_runtime::block_on(acme_workflow::setup_acme_account(
        &issuer.directory_url,
        &contact_email,
        &account_key_pem,
    ))?;
    log::info!("[revocation] submitting revoke request to ACME");
    tauri::async_runtime::block_on(account.revoke(&payload))?;

    record.revoked_at = Some(Utc::now());
    record.revocation_reason = Some(reason_label);
    inventory.insert_certificate(&record)?;
    log::info!(
        "[revocation] certificate revoked certificate_id={}",
        record.id
    );

    Ok(record_for_display(record))
}

fn parse_revocation_reason(reason: Option<String>) -> Result<(String, RevocationReason)> {
    let normalized = reason.unwrap_or_else(|| "unspecified".to_string());
    let parsed = match normalized.as_str() {
        "keyCompromise" => RevocationReason::KeyCompromise,
        "superseded" => RevocationReason::Superseded,
        "cessationOfOperation" => RevocationReason::CessationOfOperation,
        "unspecified" => RevocationReason::Unspecified,
        _ => {
            return Err(anyhow!(
                "Invalid revocation reason: {}",
                normalized
            ));
        }
    };
    Ok((normalized, parsed))
}

fn record_for_display(mut record: CertificateRecord) -> CertificateRecord {
    record.subjects = normalize_domains_for_display(&record.subjects);
    record.sans = normalize_domains_for_display(&record.sans);
    record.domain_roots = normalize_domains_for_display(&record.domain_roots);
    if let Some(csr_sans) = record.csr_sans.as_ref() {
        record.csr_sans = Some(normalize_domains_for_display(csr_sans));
    }
    record
}
