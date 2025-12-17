mod core;
mod secrets;
mod storage;

use core::commands::{
    create_secret_ref, delete_secret_ref, get_certificate, greet, list_certificates,
    list_secret_refs, seed_fake_certificate, update_secret_ref,
};
use secrets::manager::SecretManager;
use storage::inventory::InventoryStore;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let inventory_store = InventoryStore::initialize(app.handle().clone())?;
            if cfg!(debug_assertions) {
                // Seed a demo record for manual testing in development builds.
                inventory_store.seed_dev_certificate()?;
            }
            app.manage(inventory_store);

            let secret_manager = SecretManager::initialize(app.handle().clone())?;
            app.manage(secret_manager);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            list_certificates,
            get_certificate,
            seed_fake_certificate,
            list_secret_refs,
            create_secret_ref,
            update_secret_ref,
            delete_secret_ref
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
