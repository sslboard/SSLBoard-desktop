use tauri::{async_runtime::spawn_blocking, State};

use crate::core::types::{CreateSecretRequest, SecretRefRecord, UpdateSecretRequest};
use crate::secrets::manager::SecretManager;

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

/// Unlocks the secret vault, loading the master key into memory.
#[tauri::command]
pub async fn unlock_vault(manager: State<'_, SecretManager>) -> Result<bool, String> {
    eprintln!(
        "[vault-cmd] unlock_vault called, is_unlocked={}",
        manager.is_unlocked()
    );
    let manager = manager.inner().clone();
    let result = spawn_blocking(move || manager.unlock())
        .await
        .map_err(|err| format!("Unlock vault join error: {err}"))?
        .map(|_| true)
        .map_err(|err| err.to_string());
    eprintln!("[vault-cmd] unlock_vault result={:?}", result);
    result
}

/// Locks the secret vault, zeroizing the cached master key.
#[tauri::command]
pub async fn lock_vault(manager: State<'_, SecretManager>) -> Result<bool, String> {
    eprintln!(
        "[vault-cmd] lock_vault called, is_unlocked={}",
        manager.is_unlocked()
    );
    let manager = manager.inner().clone();
    let result = spawn_blocking(move || {
        manager.lock();
        Ok(false)
    })
    .await
    .map_err(|err| format!("Lock vault join error: {err}"))?;
    eprintln!("[vault-cmd] lock_vault result={:?}", result);
    result
}

/// Returns whether the vault is currently unlocked.
#[tauri::command]
pub async fn is_vault_unlocked(manager: State<'_, SecretManager>) -> Result<bool, String> {
    eprintln!("[vault-cmd] is_vault_unlocked called");
    let manager = manager.inner().clone();
    let result = spawn_blocking(move || Ok(manager.is_unlocked()))
        .await
        .map_err(|err| format!("Vault status join error: {err}"))?;
    eprintln!("[vault-cmd] is_vault_unlocked result={:?}", result);
    result
}
