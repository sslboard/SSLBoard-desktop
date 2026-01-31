use tauri::{async_runtime::spawn_blocking, State};
use log::debug;

use crate::secrets::manager::SecretManager;

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
