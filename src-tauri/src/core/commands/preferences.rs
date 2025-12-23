use tauri::{async_runtime::spawn_blocking, State};

use crate::core::types::{GetPreferenceRequest, PreferenceEntry, SetPreferenceRequest};
use crate::storage::preferences::PreferencesStore;

#[tauri::command]
pub async fn get_preference(
    store: State<'_, PreferencesStore>,
    get_req: GetPreferenceRequest,
) -> Result<Option<PreferenceEntry>, String> {
    let store = store.inner().clone();
    spawn_blocking(move || {
        let record = store.get(&get_req.name)?;
        Ok(record.map(|pref| PreferenceEntry {
            name: pref.name,
            value: pref.value,
        }))
    })
    .await
    .map_err(|err| format!("Get preference join error: {err}"))?
    .map_err(|err: anyhow::Error| err.to_string())
}

#[tauri::command]
pub async fn set_preference(
    store: State<'_, PreferencesStore>,
    set_req: SetPreferenceRequest,
) -> Result<PreferenceEntry, String> {
    let store = store.inner().clone();
    spawn_blocking(move || {
        if set_req.name.trim().is_empty() {
            return Err(anyhow::anyhow!("preference name is required"));
        }

        let record = store.set(&set_req.name, &set_req.value)?;
        Ok(PreferenceEntry {
            name: record.name,
            value: record.value,
        })
    })
    .await
    .map_err(|err| format!("Set preference join error: {err}"))?
    .map_err(|err: anyhow::Error| err.to_string())
}
