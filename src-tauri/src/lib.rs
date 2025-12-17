mod core;
mod issuance;
mod secrets;
mod storage;

use core::commands::{
    create_secret_ref, delete_secret_ref, get_certificate, greet, list_certificates,
    list_issuers, list_secret_refs, seed_fake_certificate, select_issuer, update_secret_ref,
    ensure_acme_account,
};
use secrets::manager::SecretManager;
use storage::{inventory::InventoryStore, issuer::IssuerConfigStore};
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

            let issuer_store = IssuerConfigStore::initialize(app.handle().clone())?;
            app.manage(issuer_store);
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
            delete_secret_ref,
            list_issuers,
            select_issuer,
            ensure_acme_account
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
