use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::issuance::dns::{DnsPropagationResult, DnsRecordInstruction, PropagationState};
use crate::secrets::types::{SecretKind, SecretMetadata};

/// Represents the source of a certificate record, indicating whether it was
/// discovered externally or is managed by the application.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CertificateSource {
    /// Certificate was discovered from external sources (e.g., system certificate stores)
    External,
    /// Certificate is managed and tracked by this application
    Managed,
}

/// Represents a complete certificate record with all metadata and validation information.
/// This structure is used for storing, retrieving, and displaying SSL/TLS certificate data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateRecord {
    /// Unique identifier for the certificate record
    pub id: String,
    /// Canonical subjects or SAN entries associated with the certificate.
    pub subjects: Vec<String>,
    /// Subject Alternative Names; for now mirrors `subjects` for clarity in the UI.
    pub sans: Vec<String>,
    /// Certificate issuer information (e.g., "Let's Encrypt Authority X3")
    pub issuer: String,
    /// Certificate serial number as a hex string
    pub serial: String,
    /// Certificate validity start date
    pub not_before: DateTime<Utc>,
    /// Certificate validity end date
    pub not_after: DateTime<Utc>,
    /// Certificate fingerprint (SHA-256 hash) for uniqueness verification
    pub fingerprint: String,
    /// Source of this certificate record
    pub source: CertificateSource,
    /// Root domains extracted from subjects/SANs (e.g., ["example.com"])
    pub domain_roots: Vec<String>,
    /// User-defined tags for organization and filtering
    pub tags: Vec<String>,
    /// Optional managed key reference if the private key is stored locally
    pub managed_key_ref: Option<String>,
    /// PEM-encoded certificate chain for export
    pub chain_pem: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSecretRequest {
    pub label: String,
    pub kind: SecretKind,
    pub secret_value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSecretRequest {
    pub id: String,
    pub secret_value: String,
    pub label: Option<String>,
}

pub type SecretRefRecord = SecretMetadata;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IssuerEnvironment {
    Staging,
    Production,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssuerConfigDto {
    pub issuer_id: String,
    pub label: String,
    pub directory_url: String,
    pub environment: IssuerEnvironment,
    pub contact_email: Option<String>,
    pub account_key_ref: Option<String>,
    pub is_selected: bool,
    pub disabled: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SelectIssuerRequest {
    pub issuer_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EnsureAcmeAccountRequest {
    pub issuer_id: String,
    pub contact_email: Option<String>,
    pub account_key_ref: Option<String>,
    #[serde(default)]
    pub generate_new_account_key: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PrepareDnsChallengeRequest {
    pub domain: String,
    pub txt_value: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PreparedDnsChallenge {
    pub record: DnsRecordInstruction,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CheckPropagationRequest {
    pub domain: String,
    pub txt_value: String,
}

pub type PropagationDto = DnsPropagationResult;
pub type PropagationStateDto = PropagationState;

#[derive(Debug, Clone, Deserialize)]
pub struct StartIssuanceRequest {
    pub domains: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StartIssuanceResponse {
    pub request_id: String,
    pub dns_records: Vec<DnsRecordInstruction>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CompleteIssuanceRequest {
    pub request_id: String,
}
