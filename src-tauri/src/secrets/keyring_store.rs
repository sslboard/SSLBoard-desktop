use super::store::{SecretStore, SecretStoreError};

/// OS-backed secret storage using the `keyring` crate (Keychain/Credential Manager/Secret Service).
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
        Entry::new(&self.service, id).map_err(|err| self.map_error(id, err))
    }

    fn map_error(&self, id: &str, err: keyring::Error) -> SecretStoreError {
        let msg = err.to_string();
        if msg.to_lowercase().contains("no entry") || msg.to_lowercase().contains("no password") {
            SecretStoreError::NotFound(id.to_string())
        } else if msg.to_lowercase().contains("no backend")
            || msg.to_lowercase().contains("unsupported")
        {
            SecretStoreError::Unavailable(msg)
        } else {
            SecretStoreError::Store(msg)
        }
    }
}

impl SecretStore for KeyringSecretStore {
    fn store(&self, id: &str, value: &[u8]) -> Result<(), SecretStoreError> {
        // Safety: value is sensitive; rely on OS keyring to store securely.
        self.entry(id).and_then(|entry| {
            entry
                .set_password(&String::from_utf8_lossy(value))
                .map_err(|err| self.map_error(id, err))
        })
    }

    fn retrieve(&self, id: &str) -> Result<Vec<u8>, SecretStoreError> {
        let secret = self
            .entry(id)?
            .get_password()
            .map_err(|err| self.map_error(id, err))?;
        Ok(secret.into_bytes())
    }

    fn delete(&self, id: &str) -> Result<(), SecretStoreError> {
        self.entry(id)?
            .delete_password()
            .map_err(|err| self.map_error(id, err))
    }
}
use keyring::Entry;
