use tauri::{async_runtime::spawn_blocking, State};
use log::debug;

use crate::core::types::SecretRefRecord;
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

/// Locks the secret vault, zeroizing the cached master key.
/// Used internally for auto-lock functionality (idle timeout, window blur).
#[tauri::command]
pub async fn lock_vault(manager: State<'_, SecretManager>) -> Result<(), String> {
    debug!(
        "[vault-cmd] lock_vault called, is_unlocked={}",
        manager.is_unlocked()
    );
    let manager = manager.inner().clone();
    spawn_blocking(move || {
        manager.lock();
        Ok(())
    })
    .await
    .map_err(|err| format!("Lock vault join error: {err}"))?
}
