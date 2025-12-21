mod core;
mod issuance;
mod secrets;
mod storage;

use core::commands::{
    check_dns_propagation, complete_managed_issuance, create_issuer, create_secret_ref,
    delete_issuer, delete_secret_ref, dns_provider_create, dns_provider_delete, dns_provider_list,
    dns_provider_test, dns_provider_update, dns_resolve_provider, get_certificate, greet,
    is_vault_unlocked, list_certificates, list_issuers, list_secret_refs, lock_vault,
    prepare_dns_challenge, seed_fake_certificate, select_issuer, set_issuer_disabled,
    start_managed_issuance, unlock_vault, update_issuer, update_secret_ref,
};
use secrets::manager::SecretManager;
use storage::{dns::DnsConfigStore, inventory::InventoryStore, issuer::IssuerConfigStore};
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

            let dns_store = DnsConfigStore::initialize(app.handle().clone())?;
            app.manage(dns_store);
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
            unlock_vault,
            lock_vault,
            is_vault_unlocked,
            list_issuers,
            select_issuer,
            create_issuer,
            update_issuer,
            set_issuer_disabled,
            delete_issuer,
            prepare_dns_challenge,
            check_dns_propagation,
            dns_provider_list,
            dns_provider_create,
            dns_provider_update,
            dns_provider_delete,
            dns_provider_test,
            dns_resolve_provider,
            start_managed_issuance,
            complete_managed_issuance
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
