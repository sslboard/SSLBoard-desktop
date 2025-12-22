use tauri::{async_runtime::spawn_blocking, State};

use crate::core::types::CertificateRecord;
use crate::storage::inventory::InventoryStore;

/// Retrieves all certificate records from the inventory.
/// This command fetches all stored certificate data and returns it as a vector.
///
/// # Returns
/// A Result containing either a vector of CertificateRecord or an error string
#[tauri::command]
pub async fn list_certificates(
    store: State<'_, InventoryStore>,
) -> Result<Vec<CertificateRecord>, String> {
    let store = store.inner().clone();
    spawn_blocking(move || store.list_certificates())
        .await
        .map_err(|err| format!("List join error: {err}"))?
        .map_err(|err| err.to_string())
}

/// Retrieves a specific certificate record by its ID.
/// This command looks up a single certificate in the inventory by its unique identifier.
///
/// # Arguments
/// * `store` - The inventory store state
/// * `id` - The unique identifier of the certificate to retrieve
///
/// # Returns
/// A Result containing either the CertificateRecord or an error string if not found
#[tauri::command]
pub async fn get_certificate(
    store: State<'_, InventoryStore>,
    id: String,
) -> Result<CertificateRecord, String> {
    let store = store.inner().clone();
    let missing_id = id.clone();
    spawn_blocking(move || store.get_certificate(&id))
        .await
        .map_err(|err| format!("Get join error: {err}"))?
        .map_err(|err| err.to_string())?
        .ok_or_else(|| format!("Certificate not found: {missing_id}"))
}

/// Seeds the database with a sample development certificate.
/// This command is used for development and testing purposes to populate
/// the inventory with a fake certificate record. It only adds the sample
/// certificate if the inventory is empty.
///
/// # Returns
/// A Result indicating success or an error string
#[tauri::command]
pub async fn seed_fake_certificate(store: State<'_, InventoryStore>) -> Result<(), String> {
    let store = store.inner().clone();
    spawn_blocking(move || store.seed_dev_certificate())
        .await
        .map_err(|err| format!("Seed join error: {err}"))?
        .map_err(|err| err.to_string())
}
