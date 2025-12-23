use tauri::{async_runtime::spawn_blocking, State};
use log::debug;

use crate::core::types::{
    CreateIssuerRequest, DeleteIssuerRequest, IssuerConfigDto, IssuerEnvironment, IssuerType,
    SelectIssuerRequest, UpdateIssuerRequest,
};
use crate::issuance::acme::generate_account_key_pem;
use crate::secrets::{
    manager::{SecretError, SecretManager},
    types::SecretKind,
};
use crate::storage::issuer::IssuerConfigStore;

/// Lists issuer configurations, including the selected issuer.
#[tauri::command]
pub async fn list_issuers(
    store: State<'_, IssuerConfigStore>,
) -> Result<Vec<IssuerConfigDto>, String> {
    debug!("[list_issuers] start");
    let store = store.inner().clone();
    spawn_blocking(move || -> Result<Vec<IssuerConfigDto>, anyhow::Error> {
        debug!("[list_issuers] querying store");
        let records = store.list()?;
        let result: Vec<IssuerConfigDto> = records.into_iter().map(issuer_record_to_dto).collect();
        debug!("[list_issuers] returning {} issuers", result.len());
        Ok(result)
    })
    .await
    .map_err(|err| format!("List issuers join error: {err}"))?
    .map_err(|err: anyhow::Error| err.to_string())
}

/// Sets the selected issuer id.
#[tauri::command]
pub async fn select_issuer(
    store: State<'_, IssuerConfigStore>,
    select_req: SelectIssuerRequest,
) -> Result<IssuerConfigDto, String> {
    let store = store.inner().clone();
    spawn_blocking(move || {
        let record = store.set_selected(&select_req.issuer_id)?;
        Ok(issuer_record_to_dto(record))
    })
    .await
    .map_err(|err| format!("Select issuer join error: {err}"))?
    .map_err(|err: anyhow::Error| err.to_string())
}

/// Creates a new issuer entry.
#[tauri::command]
pub async fn create_issuer(
    store: State<'_, IssuerConfigStore>,
    secrets: State<'_, SecretManager>,
    create_req: CreateIssuerRequest,
) -> Result<IssuerConfigDto, String> {
    let store = store.inner().clone();
    let secrets = secrets.inner().clone();
    spawn_blocking(move || {
        if create_req.label.trim().is_empty() {
            return Err(anyhow::anyhow!("issuer label is required"));
        }
        validate_acme_requirements(
            &create_req.issuer_type,
            create_req.contact_email.as_ref(),
            create_req.tos_agreed,
        )?;
        if create_req.directory_url.trim().is_empty() {
            return Err(anyhow::anyhow!("directory URL is required"));
        }

        let account_key_ref = match create_req.issuer_type {
            IssuerType::Acme => {
                let pem = generate_account_key_pem()
                    .map_err(|err| anyhow::anyhow!("failed to generate ACME account key: {err}"))?;
                let record = secrets
                    .create_secret(
                        SecretKind::AcmeAccountKey,
                        "ACME account key".into(),
                        pem,
                    )
                    .map_err(|err| anyhow::anyhow!(err.to_string()))?;
                Some(record.id)
            }
        };

        let record = store.create(
            create_req.label,
            issuer_type_to_string(&create_req.issuer_type),
            environment_to_string(&create_req.environment),
            create_req.directory_url,
            create_req.contact_email,
            account_key_ref,
            create_req.tos_agreed,
        )?;
        Ok(issuer_record_to_dto(record))
    })
    .await
    .map_err(|err| format!("Create issuer join error: {err}"))?
    .map_err(|err: anyhow::Error| err.to_string())
}

/// Updates an existing issuer entry.
#[tauri::command]
pub async fn update_issuer(
    store: State<'_, IssuerConfigStore>,
    secrets: State<'_, SecretManager>,
    update_req: UpdateIssuerRequest,
) -> Result<IssuerConfigDto, String> {
    let store = store.inner().clone();
    let secrets = secrets.inner().clone();
    spawn_blocking(move || {
        if update_req.label.trim().is_empty() {
            return Err(anyhow::anyhow!("issuer label is required"));
        }
        validate_acme_requirements(
            &IssuerType::Acme,
            update_req.contact_email.as_ref(),
            update_req.tos_agreed,
        )?;
        if update_req.directory_url.trim().is_empty() {
            return Err(anyhow::anyhow!("directory URL is required"));
        }

        let existing = store
            .get(&update_req.issuer_id)?
            .ok_or_else(|| anyhow::anyhow!("issuer not found: {}", update_req.issuer_id))?;
        let record = store.update(
            &update_req.issuer_id,
            update_req.label,
            environment_to_string(&update_req.environment),
            update_req.directory_url,
            update_req.contact_email,
            update_req.tos_agreed,
        )?;
        let record = if existing.account_key_ref.is_none() {
            let pem = generate_account_key_pem()
                .map_err(|err| anyhow::anyhow!("failed to generate ACME account key: {err}"))?;
            let secret_record = secrets
                .create_secret(
                    SecretKind::AcmeAccountKey,
                    "ACME account key".into(),
                    pem,
                )
                .map_err(|err| anyhow::anyhow!(err.to_string()))?;
            store.set_account_key_ref(&update_req.issuer_id, secret_record.id)?
        } else {
            record
        };
        Ok(issuer_record_to_dto(record))
    })
    .await
    .map_err(|err| format!("Update issuer join error: {err}"))?
    .map_err(|err: anyhow::Error| err.to_string())
}

/// Deletes an issuer entry and its associated account key if present.
#[tauri::command]
pub async fn delete_issuer(
    store: State<'_, IssuerConfigStore>,
    secrets: State<'_, SecretManager>,
    delete_req: DeleteIssuerRequest,
) -> Result<String, String> {
    let store = store.inner().clone();
    let secrets = secrets.inner().clone();
    spawn_blocking(move || {
        let record = store
            .get(&delete_req.issuer_id)?
            .ok_or_else(|| anyhow::anyhow!("issuer not found: {}", delete_req.issuer_id))?;
        if let Some(account_key_ref) = record.account_key_ref {
            match secrets.delete_secret(&account_key_ref) {
                Ok(()) => {}
                Err(SecretError::NotFound(_)) => {}
                Err(err) => return Err(anyhow::anyhow!(err.to_string())),
            }
        }
        store.delete(&delete_req.issuer_id)?;
        Ok(delete_req.issuer_id)
    })
    .await
    .map_err(|err| format!("Delete issuer join error: {err}"))?
    .map_err(|err: anyhow::Error| err.to_string())
}

fn issuer_record_to_dto(record: crate::storage::issuer::IssuerConfigRecord) -> IssuerConfigDto {
    let environment = match record.environment.as_str() {
        "production" => IssuerEnvironment::Production,
        _ => IssuerEnvironment::Staging,
    };
    let issuer_type = match record.issuer_type.as_str() {
        "acme" => IssuerType::Acme,
        _ => IssuerType::Acme,
    };

    IssuerConfigDto {
        issuer_id: record.issuer_id,
        label: record.label,
        directory_url: record.directory_url,
        environment,
        issuer_type,
        contact_email: record.contact_email,
        account_key_ref: record.account_key_ref,
        tos_agreed: record.tos_agreed,
        is_selected: record.is_selected,
    }
}

fn environment_to_string(environment: &IssuerEnvironment) -> String {
    match environment {
        IssuerEnvironment::Production => "production".to_string(),
        IssuerEnvironment::Staging => "staging".to_string(),
    }
}

fn issuer_type_to_string(issuer_type: &IssuerType) -> String {
    match issuer_type {
        IssuerType::Acme => "acme".to_string(),
    }
}

fn validate_acme_requirements(
    issuer_type: &IssuerType,
    contact_email: Option<&String>,
    tos_agreed: bool,
) -> Result<(), anyhow::Error> {
    if matches!(issuer_type, IssuerType::Acme) {
        if contact_email.map_or(true, |email| email.trim().is_empty()) {
            return Err(anyhow::anyhow!("contact email is required for ACME issuers"));
        }
        if !tos_agreed {
            return Err(anyhow::anyhow!(
                "Terms of Service acceptance is required for ACME issuers"
            ));
        }
    }
    Ok(())
}
