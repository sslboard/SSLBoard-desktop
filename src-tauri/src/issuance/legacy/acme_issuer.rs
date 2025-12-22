// Archived ACME issuer abstraction kept for reference; not wired into the current flow.

use anyhow::Result;
use thiserror::Error;

use crate::{
    secrets::manager::{SecretError, SecretManager},
    secrets::types::SecretKind,
    storage::issuer::IssuerConfigRecord,
};

/// High-level operations for managing ACME issuer state.
pub struct AcmeIssuer<'a> {
    pub config: IssuerConfigRecord,
    pub secrets: &'a SecretManager,
}

pub struct AccountState {
    pub account_key_ref: String,
    pub contact_email: String,
}

#[derive(Error, Debug)]
pub enum AcmeIssuerError {
    #[error("account email is required to create or update an ACME account")]
    MissingEmail,
    #[error("secret error: {0}")]
    Secret(String),
    #[error("secret reference has wrong kind: expected {expected}, found {found}")]
    WrongSecretKind { expected: String, found: String },
    #[error("key generation failed: {0}")]
    KeyGeneration(String),
    #[error("acme operation failed: {0}")]
    Operation(String),
}

impl From<SecretError> for AcmeIssuerError {
    fn from(value: SecretError) -> Self {
        AcmeIssuerError::Secret(value.to_string())
    }
}

impl<'a> AcmeIssuer<'a> {
    pub fn ensure_account(
        &self,
        contact_email: Option<String>,
        account_key_ref: Option<String>,
        generate_new_account_key: bool,
    ) -> Result<AccountState, AcmeIssuerError> {
        let email = contact_email
            .or_else(|| self.config.contact_email.clone())
            .ok_or(AcmeIssuerError::MissingEmail)?;

        let key_ref = if let Some(existing_ref) = account_key_ref {
            // Validate the provided ref exists without exposing the value.
            let metadata = self
                .secrets
                .get_metadata(&existing_ref)?
                .ok_or_else(|| AcmeIssuerError::Secret("secret not found".to_string()))?;
            if metadata.kind != SecretKind::AcmeAccountKey {
                return Err(AcmeIssuerError::WrongSecretKind {
                    expected: "acme_account_key".into(),
                    found: metadata.kind.as_str().into(),
                });
            }
            let _ = self.secrets.resolve_secret(&existing_ref)?;
            existing_ref
        } else if let Some(existing_ref) = &self.config.account_key_ref {
            // Validate the persisted ref; if missing and allowed, generate a new one.
            match self.secrets.get_metadata(existing_ref)? {
                Some(metadata) if metadata.kind == SecretKind::AcmeAccountKey => {
                    let _ = self.secrets.resolve_secret(existing_ref)?;
                    existing_ref.clone()
                }
                _ if generate_new_account_key => {
                    let pem = crate::issuance::acme::generate_account_key_pem()
                        .map_err(|err| AcmeIssuerError::KeyGeneration(err.to_string()))?;
                    let record = self
                        .secrets
                        .create_secret(SecretKind::AcmeAccountKey, "ACME account key".into(), pem)
                        .map_err(AcmeIssuerError::from)?;
                    eprintln!(
                        "[acme_issuer] regenerated account key ref {} (persisted ref missing/invalid)",
                        record.id
                    );
                    record.id
                }
                _ => {
                    return Err(AcmeIssuerError::WrongSecretKind {
                        expected: "acme_account_key".into(),
                        found: "unknown".into(),
                    })
                }
            }
        } else if generate_new_account_key {
            let pem = crate::issuance::acme::generate_account_key_pem()
                .map_err(|err| AcmeIssuerError::KeyGeneration(err.to_string()))?;
            let record = self
                .secrets
                .create_secret(SecretKind::AcmeAccountKey, "ACME account key".into(), pem)
                .map_err(AcmeIssuerError::from)?;
            eprintln!(
                "[acme_issuer] generated new account key ref {} (kind acme_account_key)",
                record.id
            );
            record.id
        } else {
            return Err(AcmeIssuerError::KeyGeneration(
                "no account key provided; set generate_new_account_key or provide a reference"
                    .to_string(),
            ));
        };

        Ok(AccountState {
            account_key_ref: key_ref,
            contact_email: email,
        })
    }
}
