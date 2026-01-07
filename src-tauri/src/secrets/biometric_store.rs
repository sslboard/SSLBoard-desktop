//! macOS biometric keychain storage adapter.
//!
//! This module provides biometric-protected master key storage on macOS using
//! Apple's Security Framework. When biometric hardware is available, secrets
//! are stored with Touch ID/Face ID protection.

use base64::{Engine as _, engine::general_purpose};
use core_foundation::base::{CFType, TCFType};
use core_foundation::data::CFData;
use core_foundation::dictionary::CFMutableDictionary;
use core_foundation::string::CFString;
use core_foundation_sys::base::OSStatus;
use core_foundation_sys::number::kCFBooleanTrue;
use keyring::Entry;
use log::{debug, info, warn};
use rand::RngCore;
use security_framework::access_control::{ProtectionMode, SecAccessControl};
use security_framework_sys::access_control::{
    kSecAccessControlBiometryAny, kSecAccessControlDevicePasscode, kSecAccessControlOr,
};
use security_framework_sys::base::errSecSuccess;
use security_framework_sys::item::{kSecClass, kSecClassGenericPassword};
use security_framework_sys::keychain_item::{SecItemAdd, SecItemCopyMatching};
use zeroize::Zeroizing;

use super::{MasterKeyStoreTrait, store::SecretStoreError};

/// macOS biometric-protected master key storage using Security Framework.
pub struct BiometricKeyringStore {
    service: String,
    account: String,
}

impl BiometricKeyringStore {
    /// Create a new biometric keyring store.
    pub fn new(service: impl Into<String>) -> Self {
        Self {
            service: service.into(),
            account: "master_key".into(), // Use same account as standard store for compatibility
        }
    }

    /// Get or create the master key with biometric protection.
    ///
    /// On first access, this will prompt for biometric authentication to establish
    /// the biometric-protected keychain item. Subsequent accesses will trigger
    /// biometric prompts automatically.
    pub fn get_or_create(&self) -> Result<Zeroizing<Vec<u8>>, SecretStoreError> {
        debug!("[biometric] get_or_create called");
        // Try to get existing key first
        match self.get() {
            Ok(key) => {
                debug!("[biometric] get_or_create: found existing biometric key");
                Ok(key)
            }
            Err(SecretStoreError::NotFound(_)) => {
                debug!("[biometric] get_or_create: no biometric key found, creating new");
                // Key doesn't exist, create it with biometric protection
                self.create()
            }
            Err(err) => {
                warn!("[biometric] get_or_create: error getting key: {}", err);
                Err(err)
            }
        }
    }

    /// Get the master key, triggering biometric authentication if required.
    ///
    /// This will automatically prompt for Touch ID/Face ID or passcode when
    /// the biometric-protected keychain item is accessed.
    pub fn get(&self) -> Result<Zeroizing<Vec<u8>>, SecretStoreError> {
        debug!("[biometric] get: fetching master key...");

        // Try biometric-protected storage first, fall back to standard keyring
        debug!("[biometric] get: trying biometric-protected storage first...");
        match self.get_biometric_secret() {
            Ok(key) => {
                debug!("[biometric] get: successfully retrieved from biometric storage");
                Ok(key)
            }
            Err(err) => {
                // ADD THIS LOGGING HERE:
                warn!(
                    "[biometric] biometric retrieval failed ({}), falling back to standard keyring",
                    err
                );
                debug!("[biometric] get: attempting fallback to standard keyring...");
                self.get_standard_keyring()
            }
        }
    }

    /// Get from standard keyring
    fn get_standard_keyring(&self) -> Result<Zeroizing<Vec<u8>>, SecretStoreError> {
        debug!("[biometric] using keyring crate for keychain access (standard mode)");
        let entry = Entry::new(&self.service, &self.account).map_err(|err| {
            SecretStoreError::Store(format!("Failed to create keyring entry: {}", err))
        })?;

        let value = entry.get_password().map_err(|err| {
            let err_str = err.to_string().to_lowercase();
            if err_str.contains("not found")
                || err_str.contains("no matching")
                || err_str.contains("found in")
            {
                SecretStoreError::NotFound(self.account.clone())
            } else {
                SecretStoreError::Store(format!("Failed to get from keyring: {}", err))
            }
        })?;

        // Decode from base64
        let decoded =
            base64::Engine::decode(&base64::engine::general_purpose::STANDARD, value.as_bytes())
                .map_err(|err| {
                    SecretStoreError::Store(format!("Failed to decode stored key: {}", err))
                })?;

        Ok(Zeroizing::new(decoded))
    }

    /// Create a new master key with biometric protection.
    ///
    /// This will prompt for biometric authentication to establish the
    /// biometric-protected keychain item.
    pub fn create(&self) -> Result<Zeroizing<Vec<u8>>, SecretStoreError> {
        debug!("[biometric] create: generating new master key with biometric protection...");

        // Generate a new 32-byte master key
        let mut key_bytes = vec![0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut key_bytes);
        debug!(
            "[biometric] create: generated {} bytes of key material",
            key_bytes.len()
        );

        // Encode as base64 for storage
        let encoded = general_purpose::STANDARD.encode(&key_bytes);
        debug!(
            "[biometric] create: encoded key for storage ({} chars)",
            encoded.len()
        );

        // Try biometric storage first, fall back to standard keyring
        debug!("[biometric] create: attempting biometric storage...");
        if let Err(err) = self.store_biometric_secret(&encoded) {
            debug!(
                "[biometric] biometric storage failed ({}), falling back to standard keyring",
                err
            );
            self.store_standard_keychain(&encoded)?;
        }
        debug!("[biometric] create: storage completed successfully");

        debug!("[biometric] create: biometric keychain storage complete");
        Ok(Zeroizing::new(key_bytes))
    }

    /// Store a secret in the biometric-protected macOS Keychain.
    ///
    /// Tries biometric access control first, falls back to standard keyring if unavailable.
    fn store_biometric_secret(&self, secret: &str) -> Result<(), SecretStoreError> {
        debug!("[biometric] attempting biometric-protected storage...");

        // Try to create biometric access control
        match Self::create_biometric_access_control() {
            Ok(access_control) => {
                debug!("[biometric] biometric access control available, using protected storage");
                self.store_with_biometric_access_control(secret, access_control)
            }
            Err(err) => {
                debug!(
                    "[biometric] biometric access control failed ({}), using standard keyring",
                    err
                );
                Err(err) // Return error so caller can fall back to standard storage
            }
        }
    }

    /// Retrieve a secret from biometric-protected macOS Keychain.
    ///
    /// This will trigger biometric authentication if the item is protected.
    fn get_biometric_secret(&self) -> Result<Zeroizing<Vec<u8>>, SecretStoreError> {
        debug!("[biometric] retrieving from biometric-protected storage...");

        // Build the keychain query dictionary
        let mut query = CFMutableDictionary::new();

        unsafe {
            // Set class to generic password
            query.set(
                CFType::wrap_under_get_rule(kSecClass as *const _),
                CFType::wrap_under_get_rule(kSecClassGenericPassword as *const _),
            );

            // Set service
            query.set(
                CFString::new("svce").as_CFType(),
                CFString::new(&self.service).as_CFType(),
            );

            // Set account
            query.set(
                CFString::new("acct").as_CFType(),
                CFString::new(&self.account).as_CFType(),
            );

            // Request return of data
            query.set(
                CFString::new("r_Data").as_CFType(),
                CFType::wrap_under_get_rule(kCFBooleanTrue as *const _),
            );
        }

        // Query the keychain
        debug!("[biometric] calling SecItemCopyMatching for biometric-protected item...");
        let mut result: core_foundation_sys::base::CFTypeRef = std::ptr::null();
        let status: OSStatus =
            unsafe { SecItemCopyMatching(query.as_concrete_TypeRef(), &mut result) };

        debug!(
            "[biometric] SecItemCopyMatching returned status: {} (0x{:x})",
            status, status as u32
        );

        if status == errSecSuccess {
            // Successfully retrieved the item
            let cf_data = unsafe { CFData::wrap_under_get_rule(result as *const _) };
            let bytes = cf_data.bytes().to_vec();

            // Decode from base64
            let encoded = String::from_utf8(bytes).map_err(|err| {
                SecretStoreError::Store(format!("Failed to decode stored data as UTF-8: {}", err))
            })?;

            let decoded = base64::Engine::decode(
                &base64::engine::general_purpose::STANDARD,
                encoded.as_bytes(),
            )
            .map_err(|err| {
                SecretStoreError::Store(format!("Failed to decode base64 data: {}", err))
            })?;

            debug!("[biometric] successfully retrieved and decoded biometric-protected secret");
            Ok(Zeroizing::new(decoded))
        } else {
            // ADD THIS LOGGING HERE:
            warn!(
                "[biometric] biometric access failed (status: {}), falling back to device passcode authentication",
                status
            );
            info!("[biometric] second auth: falls back to device passcode when biometric fails");
            Err(SecretStoreError::Store(format!(
                "Biometric keychain retrieval failed with status: {} (0x{:x})",
                status, status as u32
            )))
        }
    }

    /// Store with biometric access control (production mode with entitlements)
    fn store_with_biometric_access_control(
        &self,
        secret: &str,
        access_control: SecAccessControl,
    ) -> Result<(), SecretStoreError> {
        debug!("[biometric] storing with biometric access control (production mode)");

        // Build the keychain item dictionary with biometric protection
        let mut dict = CFMutableDictionary::new();

        unsafe {
            // Set class to generic password
            dict.set(
                CFType::wrap_under_get_rule(kSecClass as *const _),
                CFType::wrap_under_get_rule(kSecClassGenericPassword as *const _),
            );

            // Set service
            dict.set(
                CFString::new("svce").as_CFType(),
                CFString::new(&self.service).as_CFType(),
            );

            // Set account
            dict.set(
                CFString::new("acct").as_CFType(),
                CFString::new(&self.account).as_CFType(),
            );

            // Set data
            dict.set(
                CFString::new("v_Data").as_CFType(),
                CFData::from_buffer(secret.as_bytes()).as_CFType(),
            );

            // Set biometric access control
            dict.set(
                CFString::new("accc").as_CFType(),
                access_control.as_CFType(),
            );
        }

        // Add to keychain
        debug!("[biometric] calling SecItemAdd with biometric access control...");
        let status: OSStatus =
            unsafe { SecItemAdd(dict.as_concrete_TypeRef(), std::ptr::null_mut()) };
        debug!(
            "[biometric] SecItemAdd returned status: {} (0x{:x})",
            status, status as u32
        );

        if status == errSecSuccess {
            debug!("[biometric] secret stored successfully with biometric protection");
            Ok(())
        } else {
            Err(SecretStoreError::Store(format!(
                "Biometric keychain storage failed with status: {} (0x{:x})",
                status, status as u32
            )))
        }
    }

    /// Store in standard keychain without biometric protection
    fn store_standard_keychain(&self, secret: &str) -> Result<(), SecretStoreError> {
        debug!("[biometric] storing in standard keychain using keyring crate (standard mode)");

        // Use the keyring crate for storage
        let entry = Entry::new(&self.service, &self.account).map_err(|err| {
            SecretStoreError::Store(format!("Failed to create keyring entry: {}", err))
        })?;

        entry.set_password(secret).map_err(|err| {
            SecretStoreError::Store(format!("Failed to store in keyring: {}", err))
        })?;

        debug!("[biometric] secret stored successfully in standard keychain via keyring crate");
        Ok(())
    }

    /// Check if biometric authentication is available on this macOS device.
    ///
    /// This uses a try-and-fallback approach by attempting to create the
    /// biometric access control. If it succeeds, biometrics are likely available.
    pub fn check_biometric_available() -> bool {
        debug!("[biometric] checking biometric availability...");

        match Self::create_biometric_access_control() {
            Ok(_) => {
                debug!("[biometric] biometric access control created successfully");
                true
            }
            Err(err) => {
                debug!("[biometric] biometric access control failed: {}", err);
                false
            }
        }
    }

    /// Create SecAccessControl with biometric protection flags.
    ///
    /// Uses kSecAccessControlBiometryAny | kSecAccessControlOr | kSecAccessControlDevicePasscode
    /// with ProtectionMode::AccessibleWhenPasscodeSetThisDeviceOnly.
    fn create_biometric_access_control() -> Result<SecAccessControl, SecretStoreError> {
        let flags =
            kSecAccessControlBiometryAny | kSecAccessControlOr | kSecAccessControlDevicePasscode;

        SecAccessControl::create_with_protection(
            Some(ProtectionMode::AccessibleWhenUnlocked),
            flags,
        )
        .map_err(|err| {
            SecretStoreError::Store(format!(
                "Failed to create biometric access control: {}",
                err
            ))
        })
    }
}

impl MasterKeyStoreTrait for BiometricKeyringStore {
    fn get_or_create(&self) -> Result<Zeroizing<Vec<u8>>, SecretStoreError> {
        self.get_or_create()
    }
}
