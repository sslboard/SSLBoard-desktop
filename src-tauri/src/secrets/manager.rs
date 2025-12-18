use std::sync::Arc;

use anyhow::{anyhow, Result};
use chrono::Utc;
use uuid::Uuid;

use super::{
    keyring_store::{KeyringSecretStore, MasterKeyStore},
    metadata::SecretMetadataStore,
    store::{EncryptedSecretStore, SecretStore, SecretStoreError},
    types::{SecretKind, SecretMetadata},
    vault::MasterKeyVault,
};

#[derive(thiserror::Error, Debug)]
pub enum SecretError {
    #[error("secret not found: {0}")]
    NotFound(String),
    #[error("secret store unavailable: {0}")]
    Unavailable(String),
    #[error("secret storage error: {0}")]
    Store(String),
    #[error("secret metadata error: {0}")]
    Metadata(String),
    #[error("invalid secret reference: {0}")]
    InvalidReference(String),
    #[error("secret vault is locked: {0}")]
    Locked(String),
}

impl From<SecretStoreError> for SecretError {
    fn from(value: SecretStoreError) -> Self {
        match value {
            SecretStoreError::NotFound(id) => SecretError::NotFound(id),
            SecretStoreError::Unavailable(msg) => SecretError::Unavailable(msg),
            SecretStoreError::Store(msg) => SecretError::Store(msg),
            SecretStoreError::Locked(msg) => SecretError::Locked(msg),
        }
    }
}

#[derive(Clone)]
pub struct SecretManager {
    store: Arc<dyn SecretStore>,
    metadata: SecretMetadataStore,
    vault: Arc<MasterKeyVault>,
    legacy_store: Arc<KeyringSecretStore>,
    prefix: String,
}

impl SecretManager {
    pub fn initialize(app: tauri::AppHandle) -> Result<Self> {
        let metadata = SecretMetadataStore::initialize(app.clone())?;
        let master_key_store = MasterKeyStore::new("sslboard-desktop");
        let vault = Arc::new(MasterKeyVault::new(master_key_store));
        let encrypted_store: Arc<dyn SecretStore> =
            Arc::new(EncryptedSecretStore::new(metadata.clone(), vault.clone()));
        let legacy_store = Arc::new(KeyringSecretStore::new("sslboard-desktop"));

        let manager = Self {
            store: encrypted_store,
            metadata,
            vault,
            legacy_store,
            prefix: "sec_".to_string(),
        };

        manager.migrate_legacy_secrets()?;

        if let Err(err) = manager.vault.unlock() {
            eprintln!(
                "[secrets] warning: failed to unlock vault on startup, staying locked: {err}"
            );
        } else {
            eprintln!("[secrets] vault unlocked on startup");
        }

        Ok(manager)
    }

    pub fn list(&self) -> Result<Vec<SecretMetadata>, SecretError> {
        self.metadata
            .list()
            .map_err(|err| SecretError::Metadata(err.to_string()))
    }

    pub fn create_secret(
        &self,
        kind: SecretKind,
        label: String,
        secret_value: String,
    ) -> Result<SecretMetadata, SecretError> {
        self.ensure_unlocked()?;
        let id = self.generate_ref();

        let record = SecretMetadata {
            id,
            kind,
            label,
            created_at: Utc::now(),
        };
        eprintln!(
            "[secrets] create_secret kind={} id={} label={}",
            record.kind.as_str(),
            record.id,
            record.label
        );

        self.metadata
            .insert(&record)
            .map_err(|err| SecretError::Metadata(err.to_string()))?;

        if let Err(err) = self.store_secret(&record.id, secret_value.as_bytes()) {
            let _ = self.metadata.delete(&record.id);
            return Err(err);
        }

        Ok(record)
    }

    pub fn update_secret(
        &self,
        id: &str,
        secret_value: String,
        label: Option<String>,
    ) -> Result<SecretMetadata, SecretError> {
        self.ensure_prefix(id)?;
        self.ensure_unlocked()?;
        let Some(existing) = self
            .metadata
            .get(id)
            .map_err(|err| SecretError::Metadata(err.to_string()))?
        else {
            return Err(SecretError::NotFound(id.to_string()));
        };

        self.store_secret(id, secret_value.as_bytes())?;

        if let Some(ref new_label) = label {
            self.metadata
                .update_label(id, new_label)
                .map_err(|err| SecretError::Metadata(err.to_string()))?;
        }

        let updated_label = label.unwrap_or(existing.label);

        Ok(SecretMetadata {
            label: updated_label,
            ..existing
        })
    }

    pub fn delete_secret(&self, id: &str) -> Result<(), SecretError> {
        self.ensure_prefix(id)?;
        self.store
            .delete(id)
            .map_err(|err| self.map_store_error(err, id))?;
        self.metadata
            .delete(id)
            .map_err(|err| SecretError::Metadata(err.to_string()))
    }

    pub fn get_metadata(&self, id: &str) -> Result<Option<SecretMetadata>, SecretError> {
        self.ensure_prefix(id)?;
        self.metadata
            .get(id)
            .map_err(|err| SecretError::Metadata(err.to_string()))
    }

    /// Internal helper for other Rust modules to resolve a secret by reference.
    pub fn resolve_secret(&self, id: &str) -> Result<Vec<u8>, SecretError> {
        self.ensure_prefix(id)?;
        self.ensure_unlocked()?;
        self.store.retrieve(id).map_err(Into::into)
    }

    pub fn unlock(&self) -> Result<(), SecretError> {
        self.vault.unlock().map_err(Into::into)
    }

    pub fn lock(&self) {
        self.vault.lock();
    }

    pub fn is_unlocked(&self) -> bool {
        self.vault.is_unlocked()
    }

    fn store_secret(&self, id: &str, secret_value: &[u8]) -> Result<(), SecretError> {
        self.store
            .store(id, secret_value)
            .map_err(|err| self.map_store_error(err, id))
    }

    fn generate_ref(&self) -> String {
        format!("{}{}", self.prefix, Uuid::new_v4().as_simple())
    }

    fn ensure_prefix(&self, id: &str) -> Result<(), SecretError> {
        if id.starts_with(&self.prefix) {
            Ok(())
        } else {
            Err(SecretError::InvalidReference(format!(
                "expected prefix {}",
                self.prefix
            )))
        }
    }

    fn ensure_unlocked(&self) -> Result<(), SecretError> {
        if self.vault.is_unlocked() {
            Ok(())
        } else {
            Err(SecretError::Locked(
                "vault is locked; call unlock first".into(),
            ))
        }
    }

    fn migrate_legacy_secrets(&self) -> Result<()> {
        if !self.metadata.has_missing_ciphertext()? {
            return Ok(());
        }

        eprintln!("[secrets] migrating legacy keyring secrets into encrypted store");

        let migration = || -> Result<()> {
            self.vault.unlock()?;
            let records = self.metadata.list()?;
            for record in records {
                if self.metadata.has_ciphertext(&record.id)? {
                    continue;
                }
                match self.legacy_store.retrieve(&record.id) {
                    Ok(bytes) => {
                        self.store_secret(&record.id, &bytes)?;
                        if let Err(err) = self.legacy_store.delete(&record.id) {
                            eprintln!(
                                "[secrets] warning: failed to delete legacy keyring entry {}: {}",
                                record.id, err
                            );
                        }
                    }
                    Err(SecretStoreError::NotFound(_)) => {
                        eprintln!(
                            "[secrets] removing orphaned metadata row without keyring entry: {}",
                            record.id
                        );
                        let _ = self.metadata.delete(&record.id);
                    }
                    Err(err) => {
                        return Err(anyhow!(
                            "failed migrating secret {} from keyring: {}",
                            record.id,
                            err
                        ))
                    }
                }
            }
            Ok(())
        };

        let result = migration();
        self.vault.lock();
        result
    }

    fn map_store_error(&self, err: SecretStoreError, id: &str) -> SecretError {
        match err {
            SecretStoreError::Unavailable(msg) => SecretError::Unavailable(msg),
            SecretStoreError::NotFound(_) => SecretError::NotFound(id.to_string()),
            SecretStoreError::Store(msg) => SecretError::Store(msg),
            SecretStoreError::Locked(msg) => SecretError::Locked(msg),
        }
    }
}
