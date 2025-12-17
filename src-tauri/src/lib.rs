mod core;
mod storage;

use core::commands::{get_certificate, greet, list_certificates, seed_fake_certificate};
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
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            list_certificates,
            get_certificate,
            seed_fake_certificate
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
