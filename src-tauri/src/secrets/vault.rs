use std::sync::{Arc, PoisonError, RwLock};

use zeroize::{Zeroize, Zeroizing};
use log::debug;

use super::{store::SecretStoreError, MasterKeyStoreTrait};

/// Caches the master key in memory and provides explicit lock/unlock control.
pub struct MasterKeyVault {
    store: Box<dyn MasterKeyStoreTrait>,
    cached: Arc<RwLock<Option<Zeroizing<Vec<u8>>>>>,
}

impl MasterKeyVault {
    pub fn new(store: Box<dyn MasterKeyStoreTrait>) -> Self {
        Self {
            store,
            cached: Arc::new(RwLock::new(None)),
        }
    }

    pub fn is_unlocked(&self) -> bool {
        self.cached
            .read()
            .map(|guard| guard.is_some())
            .unwrap_or(false)
    }

    pub fn unlock(&self) -> Result<(), SecretStoreError> {
        // Skip keyring access if already unlocked
        if self.is_unlocked() {
            debug!("[vault] unlock called but already unlocked, skipping keyring");
            return Ok(());
        }
        debug!("[vault] unlock: accessing keyring via get_or_create...");
        let key = self.store.get_or_create()?;
        debug!("[vault] unlock: keyring access complete, caching key");
        let mut guard = self.cached.write().map_err(map_poison)?;
        if let Some(mut existing) = guard.take() {
            existing.zeroize();
        }
        *guard = Some(key);
        debug!("[vault] unlock: done, vault is now unlocked");
        Ok(())
    }

    pub fn lock(&self) {
        if let Ok(mut guard) = self.cached.write() {
            if let Some(mut key) = guard.take() {
                key.zeroize();
            }
        }
    }

    pub fn with_key<T, F>(&self, f: F) -> Result<T, SecretStoreError>
    where
        F: FnOnce(&[u8]) -> Result<T, SecretStoreError>,
    {
        let guard = self.cached.read().map_err(map_poison)?;
        let key = guard
            .as_ref()
            .ok_or_else(|| SecretStoreError::Locked("vault is locked".into()))?;
        f(key)
    }
}

fn map_poison<T>(err: PoisonError<T>) -> SecretStoreError {
    SecretStoreError::Store(format!("vault state poisoned: {err}"))
}
