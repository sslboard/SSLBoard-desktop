use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::issuance::dns::{DnsPropagationResult, DnsRecordInstruction};
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum KeyAlgorithm {
    Rsa,
    Ecdsa,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum KeyCurve {
    P256,
    P384,
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
    /// Key algorithm metadata for managed issuance (rsa/ecdsa)
    pub key_algorithm: Option<KeyAlgorithm>,
    /// RSA key size when applicable
    pub key_size: Option<u16>,
    /// ECDSA curve when applicable
    pub key_curve: Option<KeyCurve>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExportBundle {
    Cert,
    Chain,
    Fullchain,
}

impl ExportBundle {
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportCertificateRequest {
    pub certificate_id: String,
    pub destination_dir: String,
    pub folder_name: String,
    pub include_private_key: bool,
    pub bundle: ExportBundle,
    pub overwrite: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedFile {
    pub label: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum ExportCertificateResponse {
    Success {
        output_dir: String,
        files: Vec<ExportedFile>,
    },
    OverwriteRequired {
        output_dir: String,
        existing_files: Vec<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreferenceEntry {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetPreferenceRequest {
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SetPreferenceRequest {
    pub name: String,
    pub value: String,
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
#[serde(rename_all = "lowercase")]
pub enum IssuerType {
    Acme,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssuerConfigDto {
    pub issuer_id: String,
    pub label: String,
    pub directory_url: String,
    pub environment: IssuerEnvironment,
    pub issuer_type: IssuerType,
    pub contact_email: Option<String>,
    pub account_key_ref: Option<String>,
    pub tos_agreed: bool,
    pub is_selected: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SelectIssuerRequest {
    pub issuer_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateIssuerRequest {
    pub label: String,
    pub issuer_type: IssuerType,
    pub environment: IssuerEnvironment,
    pub directory_url: String,
    pub contact_email: Option<String>,
    pub tos_agreed: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateIssuerRequest {
    pub issuer_id: String,
    pub label: String,
    pub environment: IssuerEnvironment,
    pub directory_url: String,
    pub contact_email: Option<String>,
    pub tos_agreed: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeleteIssuerRequest {
    pub issuer_id: String,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DnsProviderType {
    Cloudflare,
    #[serde(rename = "digitalocean", alias = "digital_ocean")]
    DigitalOcean,
    Route53,
    Manual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsProviderDto {
    pub id: String,
    pub provider_type: DnsProviderType,
    pub label: String,
    pub domain_suffixes: Vec<String>,
    pub config: Option<Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateDnsProviderRequest {
    pub provider_type: DnsProviderType,
    pub label: String,
    pub domain_suffixes: String,
    pub api_token: Option<String>,
    #[serde(rename = "route53_access_key")]
    pub route53_access_key: Option<String>,
    #[serde(rename = "route53_secret_key")]
    pub route53_secret_key: Option<String>,
    pub config: Option<Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateDnsProviderRequest {
    pub provider_id: String,
    pub label: String,
    pub domain_suffixes: String,
    pub api_token: Option<String>,
    #[serde(rename = "route53_access_key")]
    pub route53_access_key: Option<String>,
    #[serde(rename = "route53_secret_key")]
    pub route53_secret_key: Option<String>,
    pub config: Option<Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeleteDnsProviderRequest {
    pub provider_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ResolveDnsProviderRequest {
    pub hostname: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DnsProviderResolutionDto {
    pub provider: Option<DnsProviderDto>,
    pub matched_suffix: Option<String>,
    pub ambiguous: Vec<DnsProviderDto>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TestDnsProviderRequest {
    pub provider_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ValidateDnsProviderTokenRequest {
    pub provider_type: DnsProviderType,
    pub api_token: Option<String>,
    #[serde(rename = "route53_access_key")]
    pub route53_access_key: Option<String>,
    #[serde(rename = "route53_secret_key")]
    pub route53_secret_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DnsProviderErrorCategory {
    AuthError,
    NotFound,
    RateLimited,
    NetworkError,
    Unknown,
}

#[derive(Debug, Clone, Serialize)]
pub struct DnsProviderTestResult {
    pub success: bool,
    pub record_name: Option<String>,
    pub value: Option<String>,
    pub propagation: Option<DnsPropagationResult>,
    pub error: Option<String>,
    pub error_category: Option<DnsProviderErrorCategory>,
    pub error_stage: Option<String>,
    pub elapsed_ms: u64,
    pub create_ms: Option<u64>,
    pub propagation_ms: Option<u64>,
    pub cleanup_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DnsProviderTokenValidationResult {
    pub success: bool,
    pub error: Option<String>,
    pub error_category: Option<DnsProviderErrorCategory>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StartIssuanceRequest {
    pub domains: Vec<String>,
    pub issuer_id: String,
    pub key_algorithm: Option<KeyAlgorithm>,
    pub key_size: Option<u16>,
    pub key_curve: Option<KeyCurve>,
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
