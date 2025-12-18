use base64::{engine::general_purpose, Engine as _};
use rand::{rngs::OsRng, RngCore};
use zeroize::Zeroizing;

use super::store::{SecretStore, SecretStoreError};
use keyring::Entry;

/// Legacy OS-backed secret storage using the `keyring` crate (Keychain/Credential Manager/Secret Service).
#[derive(Clone)]
pub struct KeyringSecretStore {
    service: String,
}

impl KeyringSecretStore {
    pub fn new(service: impl Into<String>) -> Self {
        Self {
            service: service.into(),
        }
    }

    fn entry(&self, id: &str) -> Result<keyring::Entry, SecretStoreError> {
        Entry::new(&self.service, id).map_err(|err| map_error(id, err))
    }
}

impl SecretStore for KeyringSecretStore {
    fn store(&self, id: &str, value: &[u8]) -> Result<(), SecretStoreError> {
        // Safety: value is sensitive; rely on OS keyring to store securely.
        self.entry(id).and_then(|entry| {
            entry
                .set_password(&String::from_utf8_lossy(value))
                .map_err(|err| map_error(id, err))
        })
    }

    fn retrieve(&self, id: &str) -> Result<Vec<u8>, SecretStoreError> {
        let secret = self
            .entry(id)?
            .get_password()
            .map_err(|err| map_error(id, err))?;
        Ok(secret.into_bytes())
    }

    fn delete(&self, id: &str) -> Result<(), SecretStoreError> {
        self.entry(id)?
            .delete_password()
            .map_err(|err| map_error(id, err))
    }
}

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
        match self.get() {
            Ok(key) => Ok(key),
            Err(SecretStoreError::NotFound(_)) => self.create(),
            Err(err) => Err(err),
        }
    }

    pub fn get(&self) -> Result<Zeroizing<Vec<u8>>, SecretStoreError> {
        let entry =
            Entry::new(&self.service, &self.user).map_err(|err| map_error(&self.user, err))?;
        let value = entry
            .get_password()
            .map_err(|err| map_error(&self.user, err))?;
        let decoded = general_purpose::STANDARD
            .decode(value.as_bytes())
            .map_err(|err| {
                SecretStoreError::Store(format!("failed to decode master key: {err}"))
            })?;
        Ok(Zeroizing::new(decoded))
    }

    pub fn create(&self) -> Result<Zeroizing<Vec<u8>>, SecretStoreError> {
        let mut key_bytes = vec![0u8; 32];
        OsRng.fill_bytes(&mut key_bytes);
        let encoded = general_purpose::STANDARD.encode(&key_bytes);

        let entry =
            Entry::new(&self.service, &self.user).map_err(|err| map_error(&self.user, err))?;
        entry
            .set_password(&encoded)
            .map_err(|err| map_error(&self.user, err))?;

        Ok(Zeroizing::new(key_bytes))
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
