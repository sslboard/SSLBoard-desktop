use base64::{Engine as _, engine::general_purpose};
use log::{debug, warn};
use rand::{RngCore, rngs::OsRng};
use zeroize::Zeroizing;

use super::{MasterKeyStoreTrait, store::SecretStoreError};
use keyring::Entry;

/// Master key storage adapter using the OS keyring.
#[derive(Clone)]
pub struct MasterKeyStore {
    service: String,
    user: String,
}

impl MasterKeyStore {
    pub fn new(service: impl Into<String>) -> Self {
        Self {
            service: service.into(),
            user: "master_key".into(),
        }
    }

    pub fn get_or_create(&self) -> Result<Zeroizing<Vec<u8>>, SecretStoreError> {
        debug!("[keyring] get_or_create called");
        // Try to get existing key first (most common case - avoids unnecessary key generation)
        match self.get() {
            Ok(key) => {
                debug!("[keyring] get_or_create: found existing key");
                return Ok(key);
            }
            Err(SecretStoreError::NotFound(_)) => {
                // Key doesn't exist, will create below
                debug!("[keyring] get_or_create: no key found, creating new");
            }
            Err(err) => {
                // If get() failed for a reason other than NotFound (e.g., keychain locked),
                // we can't proceed. Return the error.
                warn!("[keyring] get_or_create: error getting key: {}", err);
                return Err(err);
            }
        }

        // Key doesn't exist, create it.
        // NOTE: On macOS, you may see TWO password prompts even when the master key exists:
        // 1. One prompt when get() tries to read the keychain item
        // 2. Another prompt from macOS keychain security (even though the item exists)
        // This is a known macOS keychain behavior - macOS may prompt twice for the same
        // operation (once for keychain unlock, once for item access). This cannot be
        // avoided with the current keyring crate API. A future enhancement would use
        // security_framework directly with biometric authentication (Touch ID/Face ID)
        // to eliminate password prompts entirely.
        let new_key = self.create()?;
        debug!("[keyring] get_or_create: created new key successfully");
        Ok(new_key)
    }

    pub fn get(&self) -> Result<Zeroizing<Vec<u8>>, SecretStoreError> {
        debug!("[keyring] get: fetching master key from keyring...");
        let entry =
            Entry::new(&self.service, &self.user).map_err(|err| map_error(&self.user, err))?;
        let value = entry
            .get_password()
            .map_err(|err| map_error(&self.user, err))?;
        debug!("[keyring] get: keyring access complete");
        let decoded = general_purpose::STANDARD
            .decode(value.as_bytes())
            .map_err(|err| {
                SecretStoreError::Store(format!("failed to decode master key: {err}"))
            })?;
        Ok(Zeroizing::new(decoded))
    }

    pub fn create(&self) -> Result<Zeroizing<Vec<u8>>, SecretStoreError> {
        debug!("[keyring] create: generating new master key...");
        let mut key_bytes = vec![0u8; 32];
        OsRng.fill_bytes(&mut key_bytes);
        let encoded = general_purpose::STANDARD.encode(&key_bytes);

        let entry =
            Entry::new(&self.service, &self.user).map_err(|err| map_error(&self.user, err))?;
        debug!("[keyring] create: storing key in keyring...");
        entry
            .set_password(&encoded)
            .map_err(|err| map_error(&self.user, err))?;
        debug!("[keyring] create: keyring access complete");

        Ok(Zeroizing::new(key_bytes))
    }
}

impl MasterKeyStoreTrait for MasterKeyStore {
    fn get_or_create(&self) -> Result<Zeroizing<Vec<u8>>, SecretStoreError> {
        self.get_or_create()
    }
}

fn map_error(id: &str, err: keyring::Error) -> SecretStoreError {
    let msg = err.to_string();
    let msg_lc = msg.to_lowercase();
    if msg_lc.contains("no entry")
        || msg_lc.contains("no password")
        || msg_lc.contains("no matching entry")
        || msg_lc.contains("not found")
    {
        SecretStoreError::NotFound(id.to_string())
    } else if msg_lc.contains("no backend") || msg_lc.contains("unsupported") {
        SecretStoreError::Unavailable(msg)
    } else {
        SecretStoreError::Store(msg)
    }
}
