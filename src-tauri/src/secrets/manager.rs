use std::sync::Arc;

use anyhow::Result;
use chrono::Utc;
use log::{error, info, warn};
use uuid::Uuid;

use super::{
    metadata::SecretMetadataStore,
    store::{EncryptedSecretStore, SecretStore, SecretStoreError},
    types::{SecretKind, SecretMetadata},
    vault::MasterKeyVault,
    create_master_key_store,
};
use tauri::Emitter;
use zeroize::Zeroizing;

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
    app: tauri::AppHandle,
    prefix: String,
}

impl SecretManager {
    pub fn initialize(app: tauri::AppHandle) -> Result<Self> {
        let metadata = SecretMetadataStore::initialize(app.clone())?;
        let master_key_store = create_master_key_store("sslboard-desktop");
        let vault = Arc::new(MasterKeyVault::new(master_key_store));
        let encrypted_store: Arc<dyn SecretStore> =
            Arc::new(EncryptedSecretStore::new(metadata.clone(), vault.clone()));

        Ok(Self {
            store: encrypted_store,
            metadata,
            vault,
            app: app.clone(),
            prefix: "sec_".to_string(),
        })
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
        let secret_bytes = Zeroizing::new(secret_value.into_bytes());
        let id = self.generate_ref();

        let record = SecretMetadata {
            id,
            kind,
            label,
            created_at: Utc::now(),
        };
        info!(
            "[secrets] create_secret kind={} id={}",
            record.kind.as_str(),
            record.id
        );

        self.metadata
            .insert(&record)
            .map_err(|err| SecretError::Metadata(err.to_string()))?;

        if let Err(err) = self.store_secret(&record.id, &secret_bytes) {
            if let Err(delete_err) = self.metadata.delete(&record.id) {
                warn!(
                    "[secrets] failed to rollback metadata for {}: {}",
                    record.id, delete_err
                );
            }
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
        let secret_bytes = Zeroizing::new(secret_value.into_bytes());
        let Some(existing) = self
            .metadata
            .get(id)
            .map_err(|err| SecretError::Metadata(err.to_string()))?
        else {
            return Err(SecretError::NotFound(id.to_string()));
        };

        self.store_secret(id, &secret_bytes)?;

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
        self.vault.unlock().map_err(SecretError::from)?;
        self.emit_vault_state(true);
        Ok(())
    }

    pub fn lock(&self) {
        self.vault.lock();
        self.emit_vault_state(false);
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

    /// Auto-unlocks the vault on demand when secrets are accessed.
    fn ensure_unlocked(&self) -> Result<(), SecretError> {
        if self.vault.is_unlocked() {
            return Ok(());
        }
        self.vault.unlock().map_err(|err| match err {
            SecretStoreError::Unavailable(msg) => SecretError::Unavailable(msg),
            SecretStoreError::Locked(msg) => SecretError::Locked(msg),
            _ => SecretError::Store(err.to_string()),
        })?;
        self.emit_vault_state(true);
        Ok(())
    }

    fn map_store_error(&self, err: SecretStoreError, id: &str) -> SecretError {
        match err {
            SecretStoreError::Unavailable(msg) => SecretError::Unavailable(msg),
            SecretStoreError::NotFound(_) => SecretError::NotFound(id.to_string()),
            SecretStoreError::Store(msg) => SecretError::Store(msg),
            SecretStoreError::Locked(msg) => SecretError::Locked(msg),
        }
    }

    fn emit_vault_state(&self, unlocked: bool) {
        let payload = serde_json::json!({ "unlocked": unlocked });
        if let Err(err) = self.app.emit("vault-state-changed", payload) {
            error!("[secrets] failed to emit vault state: {err}");
        }
    }
}
