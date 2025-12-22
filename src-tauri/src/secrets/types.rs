use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Enumerates supported secret categories for v0.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SecretKind {
    DnsProviderToken,
    DnsProviderAccessKey,
    DnsProviderSecretKey,
    AcmeAccountKey,
    ManagedPrivateKey,
}

impl SecretKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            SecretKind::DnsProviderToken => "dns_provider_token",
            SecretKind::DnsProviderAccessKey => "dns_provider_access_key",
            SecretKind::DnsProviderSecretKey => "dns_provider_secret_key",
            SecretKind::AcmeAccountKey => "acme_account_key",
            SecretKind::ManagedPrivateKey => "managed_private_key",
        }
    }
}

/// Non-secret metadata stored locally so the UI can list secret references.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretMetadata {
    pub id: String,
    pub kind: SecretKind,
    pub label: String,
    pub created_at: DateTime<Utc>,
}
