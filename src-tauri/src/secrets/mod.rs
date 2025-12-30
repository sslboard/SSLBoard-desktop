pub mod keyring_store;
pub mod manager;
pub mod metadata;
pub mod store;
pub mod types;
pub mod vault;

#[cfg(target_os = "macos")]
pub mod biometric_store;

/// Trait for master key storage backends.
pub trait MasterKeyStoreTrait: Send + Sync {
    fn get_or_create(&self) -> Result<zeroize::Zeroizing<Vec<u8>>, store::SecretStoreError>;
}

/// Create a master key store appropriate for the current platform.
///
/// On macOS, this will use biometric protection if available.
/// On other platforms, it uses the standard OS keyring.
pub fn create_master_key_store(service: &str) -> Box<dyn MasterKeyStoreTrait> {
    #[cfg(target_os = "macos")]
    {
        // On macOS, try to use biometric store if available
        if biometric_store::BiometricKeyringStore::check_biometric_available() {
            log::info!("Using biometric-protected master key store on macOS (framework: keyring crate for dev, security framework for production)");
            Box::new(biometric_store::BiometricKeyringStore::new(service))
        } else {
            log::info!("Biometric authentication not available, using standard keyring store (framework: keyring crate)");
            Box::new(keyring_store::MasterKeyStore::new(service))
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        log::info!("Using standard keyring store (framework: keyring crate)");
        Box::new(keyring_store::MasterKeyStore::new(service))
    }
}
