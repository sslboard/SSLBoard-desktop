use std::sync::Arc;

use aes_gcm::{aead::Aead, Aes256Gcm, KeyInit, Nonce};
use rand::{rngs::OsRng, RngCore};
use thiserror::Error;

use super::{metadata::SecretMetadataStore, vault::MasterKeyVault};

/// Errors produced by secret storage backends.
#[derive(Debug, Error)]
pub enum SecretStoreError {
    #[error("secret not found: {0}")]
    NotFound(String),
    #[error("secret store unavailable: {0}")]
    Unavailable(String),
    #[error("secret store error: {0}")]
    Store(String),
    #[error("secret vault is locked: {0}")]
    Locked(String),
    #[error("master key mismatch - stored data was encrypted with a different key. This usually happens when switching storage methods. Clear all stored data and start fresh.")]
    MasterKeyMismatch,
}

/// Abstraction for storing and retrieving secrets inside the trusted core.
pub trait SecretStore: Send + Sync {
    fn store(&self, id: &str, value: &[u8]) -> Result<(), SecretStoreError>;
    fn retrieve(&self, id: &str) -> Result<Vec<u8>, SecretStoreError>;
    fn delete(&self, id: &str) -> Result<(), SecretStoreError>;
}

/// AES-256-GCM encrypted secret storage backed by SQLite.
pub struct EncryptedSecretStore {
    metadata: SecretMetadataStore,
    vault: Arc<MasterKeyVault>,
}

impl EncryptedSecretStore {
    pub fn new(metadata: SecretMetadataStore, vault: Arc<MasterKeyVault>) -> Self {
        Self { metadata, vault }
    }
}

impl SecretStore for EncryptedSecretStore {
    fn store(&self, id: &str, value: &[u8]) -> Result<(), SecretStoreError> {
        self.vault.with_key(|key| {
            let cipher = Aes256Gcm::new_from_slice(key)
                .map_err(|err| SecretStoreError::Store(err.to_string()))?;

            let mut nonce_bytes = [0u8; 12];
            OsRng.fill_bytes(&mut nonce_bytes);
            let nonce = Nonce::from_slice(&nonce_bytes);

            let mut ciphertext = cipher
                .encrypt(nonce, value)
                .map_err(|err| SecretStoreError::Store(err.to_string()))?;

            let mut payload = Vec::with_capacity(nonce_bytes.len() + ciphertext.len());
            payload.extend_from_slice(&nonce_bytes);
            payload.append(&mut ciphertext);

            self.metadata
                .store_ciphertext(id, &payload)
                .map_err(|err| SecretStoreError::Store(err.to_string()))
        })
    }

    fn retrieve(&self, id: &str) -> Result<Vec<u8>, SecretStoreError> {
        let ciphertext = self
            .metadata
            .get_ciphertext(id)
            .map_err(|err| SecretStoreError::Store(err.to_string()))?;
        let Some(ciphertext) = ciphertext else {
            return Err(SecretStoreError::NotFound(id.to_string()));
        };
        if ciphertext.len() < 12 {
            return Err(SecretStoreError::Store(
                "stored ciphertext missing nonce".into(),
            ));
        }

        self.vault.with_key(|key| {
            let cipher = Aes256Gcm::new_from_slice(key)
                .map_err(|err| SecretStoreError::Store(err.to_string()))?;
            let (nonce_bytes, data) = ciphertext.split_at(12);
            let nonce = Nonce::from_slice(nonce_bytes);
            cipher
                .decrypt(nonce, data)
                .map_err(|_err| {
                    // AEAD decryption failures almost always mean master key mismatch
                    // (data was encrypted with a different key than we're trying to decrypt with)
                    SecretStoreError::MasterKeyMismatch
                })
        })
    }

    fn delete(&self, id: &str) -> Result<(), SecretStoreError> {
        self.metadata
            .clear_ciphertext(id)
            .map_err(|err| SecretStoreError::Store(err.to_string()))
    }
}
