use tauri::{async_runtime::spawn_blocking, State};

use crate::core::types::{
    CertificateSource, ExportCertificateRequest, ExportCertificateResponse,
};
use crate::distribution::export::{export_pem_bundle, ExportOptions};
use crate::secrets::manager::SecretManager;
use crate::storage::inventory::InventoryStore;

#[tauri::command]
pub async fn export_certificate_pem(
    inventory: State<'_, InventoryStore>,
    secrets: State<'_, SecretManager>,
    export_req: ExportCertificateRequest,
) -> Result<ExportCertificateResponse, String> {
    let inventory = inventory.inner().clone();
    let secrets = secrets.inner().clone();
    spawn_blocking(move || {
        let record = inventory
            .get_certificate(&export_req.certificate_id)
            .map_err(|err| err.to_string())?
            .ok_or_else(|| format!("Certificate not found: {}", export_req.certificate_id))?;

        if !matches!(record.source, CertificateSource::Managed) {
            return Err("Export is only available for Managed certificates".to_string());
        }

        let chain_pem = record
            .chain_pem
            .ok_or_else(|| "Certificate chain PEM is missing for export".to_string())?;

        let key_pem = if export_req.include_private_key {
            let key_ref = record.managed_key_ref.ok_or_else(|| {
                "Certificate does not have a managed key reference".to_string()
            })?;
            let bytes = secrets
                .resolve_secret(&key_ref)
                .map_err(|err| err.to_string())?;
            Some(
                String::from_utf8(bytes)
                    .map_err(|_| "Managed key material was not valid UTF-8".to_string())?,
            )
        } else {
            None
        };

        export_pem_bundle(
            &chain_pem,
            key_pem.as_deref(),
            ExportOptions {
                destination_dir: &export_req.destination_dir,
                folder_name: &export_req.folder_name,
                include_private_key: export_req.include_private_key,
                overwrite: export_req.overwrite,
                bundle: export_req.bundle,
            },
        )
        .map_err(|err| err.to_string())
    })
    .await
    .map_err(|err| format!("Export join error: {err}"))?
}
