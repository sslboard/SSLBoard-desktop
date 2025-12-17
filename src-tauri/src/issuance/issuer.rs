use thiserror::Error;

/// Core issuer operations that any certificate issuer must support.
pub trait Issuer {
    fn ensure_account(&self) -> Result<(), IssuerError>;
    fn begin_order(&self, domains: &[String]) -> Result<OrderHandle, IssuerError>;
    fn get_challenges(&self, order: &OrderHandle) -> Result<Vec<IssuanceChallenge>, IssuerError>;
    fn finalize(&self, order: &OrderHandle, csr_pem: &str) -> Result<(), IssuerError>;
    fn download_certificate(&self, order: &OrderHandle) -> Result<Vec<u8>, IssuerError>;
}

#[derive(Debug, Clone)]
pub struct OrderHandle {
    pub id: String,
}

#[derive(Debug, Clone)]
pub enum ChallengeKind {
    Dns01,
}

#[derive(Debug, Clone)]
pub struct IssuanceChallenge {
    pub domain: String,
    pub token: String,
    pub key_auth: String,
    pub kind: ChallengeKind,
}

#[derive(Error, Debug)]
pub enum IssuerError {
    #[error("issuer unavailable: {0}")]
    Unavailable(String),
    #[error("issuer configuration invalid: {0}")]
    InvalidConfig(String),
    #[error("issuer operation failed: {0}")]
    Operation(String),
}
