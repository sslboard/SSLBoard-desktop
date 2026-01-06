mod core;
mod distribution;
pub mod issuance;
mod secrets;
mod storage;

use core::commands::{
    complete_managed_issuance, create_issuer, delete_issuer, dns_provider_create,
    dns_provider_delete, dns_provider_list, dns_provider_test, dns_provider_update,
    dns_resolve_provider, export_certificate_pem, get_certificate, get_preference,
    list_certificates, list_issuers, list_secret_refs, lock_vault, select_issuer, set_preference,
    start_managed_issuance, update_issuer,
};
use secrets::manager::SecretManager;
use std::sync::Once;
use storage::{
    dns::DnsConfigStore, inventory::InventoryStore, issuer::IssuerConfigStore,
    preferences::PreferencesStore,
};
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    init_logging();
    if let Err(err) = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
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

            let preferences_store = PreferencesStore::initialize(app.handle().clone())?;
            app.manage(preferences_store);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_certificates,
            get_certificate,
            export_certificate_pem,
            list_secret_refs,
            lock_vault,
            list_issuers,
            select_issuer,
            create_issuer,
            update_issuer,
            delete_issuer,
            dns_provider_list,
            dns_provider_create,
            dns_provider_update,
            dns_provider_delete,
            dns_provider_test,
            dns_resolve_provider,
            start_managed_issuance,
            complete_managed_issuance,
            get_preference,
            set_preference
        ])
        .run(tauri::generate_context!())
    {
        log::error!("[tauri] error while running tauri application: {err}");
    }
}

fn init_logging() {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        let env = env_logger::Env::default().default_filter_or("info");
        let mut builder = env_logger::Builder::from_env(env);
        // Reduce verbosity of rustls and related TLS libraries by default
        // Can be overridden via RUST_LOG env var (e.g., RUST_LOG=debug,rustls=debug)
        builder.filter_module("rustls", log::LevelFilter::Warn);
        builder.filter_module("rustls::client", log::LevelFilter::Warn);
        builder.filter_module("rustls::client::hs", log::LevelFilter::Warn);
        builder.filter_module("rustls::client::tls13", log::LevelFilter::Warn);
        // Reduce verbosity of ureq HTTP client by default
        builder.filter_module("ureq", log::LevelFilter::Warn);
        builder.filter_module("ureq::stream", log::LevelFilter::Warn);
        builder.filter_module("ureq::unit", log::LevelFilter::Warn);
        builder.filter_module("ureq::pool", log::LevelFilter::Warn);
        builder.filter_module("ureq::response", log::LevelFilter::Warn);
        // Reduce verbosity of reqwest HTTP client by default
        builder.filter_module("reqwest", log::LevelFilter::Warn);
        builder.filter_module("reqwest::connect", log::LevelFilter::Warn);
        // Reduce verbosity of hyper_util HTTP client by default
        builder.filter_module("hyper_util", log::LevelFilter::Warn);
        builder.filter_module("hyper_util::client", log::LevelFilter::Warn);
        builder.filter_module("hyper_util::client::legacy", log::LevelFilter::Warn);
        builder.filter_module(
            "hyper_util::client::legacy::connect",
            log::LevelFilter::Warn,
        );
        builder.filter_module("hyper_util::client::legacy::pool", log::LevelFilter::Warn);
        builder.format_timestamp_millis().init();
    });
}
