use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CertificateSource {
    External,
    Managed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateRecord {
    pub id: String,
    /// Canonical subjects or SAN entries associated with the certificate.
    pub subjects: Vec<String>,
    /// Subject Alternative Names; for now mirrors `subjects` for clarity in the UI.
    pub sans: Vec<String>,
    pub issuer: String,
    pub serial: String,
    pub not_before: DateTime<Utc>,
    pub not_after: DateTime<Utc>,
    pub fingerprint: String,
    pub source: CertificateSource,
    pub domain_roots: Vec<String>,
    pub tags: Vec<String>,
}
