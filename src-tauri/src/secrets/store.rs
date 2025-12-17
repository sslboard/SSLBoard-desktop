use thiserror::Error;

/// Errors produced by secret storage backends.
#[derive(Debug, Error)]
pub enum SecretStoreError {
    #[error("secret not found: {0}")]
    NotFound(String),
    #[error("secret store unavailable: {0}")]
    Unavailable(String),
    #[error("secret store error: {0}")]
    Store(String),
}

/// Abstraction for storing and retrieving secrets inside the trusted core.
pub trait SecretStore: Send + Sync {
    fn store(&self, id: &str, value: &[u8]) -> Result<(), SecretStoreError>;
    fn retrieve(&self, id: &str) -> Result<Vec<u8>, SecretStoreError>;
    fn delete(&self, id: &str) -> Result<(), SecretStoreError>;
}
