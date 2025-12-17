use tauri::{async_runtime::spawn_blocking, State};

use crate::{
    core::types::{CertificateRecord, CreateSecretRequest, SecretRefRecord, UpdateSecretRequest},
    secrets::manager::SecretManager,
    storage::inventory::InventoryStore,
};

/// A simple greeting command for testing the Tauri-Rust bridge.
/// This command demonstrates basic string processing and command invocation.
///
/// # Arguments
/// * `name` - The name to greet
///
/// # Returns
/// A greeting string from Rust
#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

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

/// Lists secret references (metadata only, no secret bytes).
#[tauri::command]
pub async fn list_secret_refs(
    manager: State<'_, SecretManager>,
) -> Result<Vec<SecretRefRecord>, String> {
    let manager = manager.inner().clone();
    spawn_blocking(move || manager.list())
        .await
        .map_err(|err| format!("List join error: {err}"))?
        .map_err(|err| err.to_string())
}

/// Creates a new secret reference by sending the secret value into the trusted core once.
#[tauri::command]
pub async fn create_secret_ref(
    manager: State<'_, SecretManager>,
    req: CreateSecretRequest,
) -> Result<SecretRefRecord, String> {
    let manager = manager.inner().clone();
    spawn_blocking(move || manager.create_secret(req.kind, req.label, req.secret_value))
        .await
        .map_err(|err| format!("Create join error: {err}"))?
        .map_err(|err| err.to_string())
}

/// Updates an existing secret while keeping the reference id stable.
#[tauri::command]
pub async fn update_secret_ref(
    manager: State<'_, SecretManager>,
    req: UpdateSecretRequest,
) -> Result<SecretRefRecord, String> {
    let manager = manager.inner().clone();
    spawn_blocking(move || manager.update_secret(&req.id, req.secret_value, req.label))
        .await
        .map_err(|err| format!("Update join error: {err}"))?
        .map_err(|err| err.to_string())
}

/// Removes a secret reference and deletes the underlying secret from the OS store.
#[tauri::command]
pub async fn delete_secret_ref(
    manager: State<'_, SecretManager>,
    id: String,
) -> Result<(), String> {
    let manager = manager.inner().clone();
    spawn_blocking(move || manager.delete_secret(&id))
        .await
        .map_err(|err| format!("Delete join error: {err}"))?
        .map_err(|err| err.to_string())
}
