use anyhow::anyhow;
use tauri::{async_runtime::spawn_blocking, State};

use crate::core::types::{
    CertificateRecord, CompleteCsrIssuanceRequest, CompleteIssuanceRequest, CsrValidationResult,
    GenerateCsrRequest, GenerateCsrResponse, InspectCsrRequest, StartCsrIssuanceRequest,
    StartCsrIssuanceResponse, StartIssuanceRequest, StartIssuanceResponse,
};
use crate::domain::normalize_domains_for_display;
use crate::issuance::{acme_workflow, csr as csr_tools};
use crate::issuance::flow::{complete_csr_dns01, complete_managed_dns01, start_csr_dns01, start_managed_dns01};
use crate::secrets::manager::SecretManager;
use crate::secrets::types::SecretKind;
use crate::storage::{dns::DnsConfigStore, inventory::InventoryStore, issuer::IssuerConfigStore};

/// Starts a managed-key ACME issuance and returns DNS-01 instructions plus a request id.
#[tauri::command]
pub async fn start_managed_issuance(
    issuer_store: State<'_, IssuerConfigStore>,
    dns_store: State<'_, DnsConfigStore>,
    secrets: State<'_, SecretManager>,
    start_req: StartIssuanceRequest,
) -> Result<StartIssuanceResponse, String> {
    let issuer_store = issuer_store.inner().clone();
    let dns_store = dns_store.inner().clone();
    let secrets = secrets.inner().clone();
    spawn_blocking(move || {
        start_managed_dns01(
            start_req.domains,
            start_req.issuer_id,
            start_req.key_algorithm,
            start_req.key_size,
            start_req.key_curve,
            &issuer_store,
            &dns_store,
            &secrets,
        )
        .map(|(request_id, dns_records)| StartIssuanceResponse {
            request_id,
            dns_records,
        })
    })
    .await
    .map_err(|err| format!("Start issuance join error: {err}"))?
    .map_err(|err: anyhow::Error| err.to_string())
}

/// Completes a managed-key ACME issuance after DNS-01 is satisfied.
#[tauri::command]
pub async fn complete_managed_issuance(
    inventory: State<'_, InventoryStore>,
    secrets: State<'_, SecretManager>,
    dns_store: State<'_, DnsConfigStore>,
    complete_req: CompleteIssuanceRequest,
) -> Result<CertificateRecord, String> {
    let inventory = inventory.inner().clone();
    let secrets = secrets.inner().clone();
    let dns_store = dns_store.inner().clone();
    spawn_blocking(move || {
        complete_managed_dns01(&complete_req.request_id, &inventory, &secrets, &dns_store)
    })
        .await
        .map_err(|err| format!("Complete issuance join error: {err}"))?
        .map_err(|err: anyhow::Error| err.to_string())
        .map(record_for_display)
}

#[tauri::command]
pub async fn inspect_csr(
    inspect_req: InspectCsrRequest,
) -> Result<CsrValidationResult, String> {
    spawn_blocking(move || {
        let parsed = csr_tools::load_and_validate_csr(&inspect_req.csr_path)?;
        Ok(CsrValidationResult {
            metadata: parsed.metadata,
            identifiers: parsed.identifiers,
            warnings: parsed.warnings,
        })
    })
    .await
    .map_err(|err| format!("Inspect CSR join error: {err}"))?
    .map_err(|err: anyhow::Error| err.to_string())
}

#[tauri::command]
pub async fn generate_csr(
    secrets: State<'_, SecretManager>,
    generate_req: GenerateCsrRequest,
) -> Result<GenerateCsrResponse, String> {
    let secrets = secrets.inner().clone();
    spawn_blocking(move || {
        let (key_algorithm, key_size, key_curve) = acme_workflow::resolve_key_params(
            generate_req.key_algorithm,
            generate_req.key_size,
            generate_req.key_curve,
        )?;
        let key_pem = acme_workflow::generate_private_key(
            &key_algorithm,
            key_size,
            key_curve.as_ref(),
        )?;
        let csr_der = acme_workflow::build_csr_der_with_subject(
            &key_pem,
            &generate_req.subject,
            &generate_req.sans,
        )?;
        let csr_pem = pem::encode(&pem::Pem::new("CERTIFICATE REQUEST", csr_der.clone()));
        std::fs::write(&generate_req.output_path, csr_pem)
            .map_err(|err| anyhow!("Failed to write CSR file: {err}"))?;

        let key_label = format!("Managed CSR key for {}", generate_req.subject.trim());
        let managed_key = secrets
            .create_secret(SecretKind::ManagedPrivateKey, key_label, key_pem)
            .map_err(|err| anyhow!(err.to_string()))?;

        let parsed = csr_tools::parse_and_validate_csr_bytes(&csr_der)?;
        Ok(GenerateCsrResponse {
            csr_path: generate_req.output_path,
            managed_key_ref: managed_key.id,
            result: CsrValidationResult {
                metadata: parsed.metadata,
                identifiers: parsed.identifiers,
                warnings: parsed.warnings,
            },
        })
    })
    .await
    .map_err(|err| format!("Generate CSR join error: {err}"))?
    .map_err(|err: anyhow::Error| err.to_string())
}

/// Starts a CSR-based ACME issuance and returns DNS-01 instructions plus a request id.
#[tauri::command]
pub async fn start_csr_issuance(
    issuer_store: State<'_, IssuerConfigStore>,
    dns_store: State<'_, DnsConfigStore>,
    secrets: State<'_, SecretManager>,
    start_req: StartCsrIssuanceRequest,
) -> Result<StartCsrIssuanceResponse, String> {
    let issuer_store = issuer_store.inner().clone();
    let dns_store = dns_store.inner().clone();
    let secrets = secrets.inner().clone();
    spawn_blocking(move || {
        start_csr_dns01(
            start_req.issuer_id,
            start_req.csr_path,
            start_req.csr_source,
            start_req.managed_key_ref,
            &issuer_store,
            &dns_store,
            &secrets,
        )
        .map(|(request_id, dns_records, metadata, identifiers, warnings)| {
            StartCsrIssuanceResponse {
                request_id,
                dns_records,
                csr_result: CsrValidationResult {
                    metadata,
                    identifiers,
                    warnings,
                },
            }
        })
    })
    .await
    .map_err(|err| format!("Start CSR issuance join error: {err}"))?
    .map_err(|err: anyhow::Error| err.to_string())
}

/// Completes a CSR-based ACME issuance after DNS-01 is satisfied.
#[tauri::command]
pub async fn complete_csr_issuance(
    inventory: State<'_, InventoryStore>,
    secrets: State<'_, SecretManager>,
    dns_store: State<'_, DnsConfigStore>,
    complete_req: CompleteCsrIssuanceRequest,
) -> Result<CertificateRecord, String> {
    let inventory = inventory.inner().clone();
    let secrets = secrets.inner().clone();
    let dns_store = dns_store.inner().clone();
    spawn_blocking(move || {
        complete_csr_dns01(&complete_req.request_id, &inventory, &secrets, &dns_store)
    })
        .await
        .map_err(|err| format!("Complete CSR issuance join error: {err}"))?
        .map_err(|err: anyhow::Error| err.to_string())
        .map(record_for_display)
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
