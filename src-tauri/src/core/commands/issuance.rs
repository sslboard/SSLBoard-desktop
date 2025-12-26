use tauri::{async_runtime::spawn_blocking, State};

use crate::core::types::{
    CertificateRecord, CompleteIssuanceRequest, StartIssuanceRequest, StartIssuanceResponse,
};
use crate::issuance::flow::{complete_managed_dns01, start_managed_dns01};
use crate::secrets::manager::SecretManager;
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
}
