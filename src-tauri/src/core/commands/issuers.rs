use tauri::{async_runtime::spawn_blocking, State};

use crate::core::types::{
    CreateIssuerRequest, DeleteIssuerRequest, IssuerConfigDto, IssuerEnvironment, IssuerType,
    SelectIssuerRequest, SetIssuerDisabledRequest, UpdateIssuerRequest,
};
use crate::issuance::acme::generate_account_key_pem;
use crate::secrets::{manager::SecretManager, types::SecretKind};
use crate::storage::issuer::IssuerConfigStore;

/// Lists issuer configurations, including the selected issuer.
#[tauri::command]
pub async fn list_issuers(
    store: State<'_, IssuerConfigStore>,
) -> Result<Vec<IssuerConfigDto>, String> {
    eprintln!("[list_issuers] start");
    let store = store.inner().clone();
    spawn_blocking(move || -> Result<Vec<IssuerConfigDto>, anyhow::Error> {
        eprintln!("[list_issuers] querying store");
        let records = store.list()?;
        let result: Vec<IssuerConfigDto> = records.into_iter().map(issuer_record_to_dto).collect();
        eprintln!("[list_issuers] returning {} issuers", result.len());
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
    req: SelectIssuerRequest,
) -> Result<IssuerConfigDto, String> {
    let store = store.inner().clone();
    spawn_blocking(move || {
        let record = store.set_selected(&req.issuer_id)?;
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
    req: CreateIssuerRequest,
) -> Result<IssuerConfigDto, String> {
    let store = store.inner().clone();
    let secrets = secrets.inner().clone();
    spawn_blocking(move || {
        if req.label.trim().is_empty() {
            return Err(anyhow::anyhow!("issuer label is required"));
        }
        validate_acme_requirements(&req.issuer_type, req.contact_email.as_ref(), req.tos_agreed)?;
        if req.directory_url.trim().is_empty() {
            return Err(anyhow::anyhow!("directory URL is required"));
        }

        let account_key_ref = match req.issuer_type {
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
            req.label,
            issuer_type_to_string(&req.issuer_type),
            environment_to_string(&req.environment),
            req.directory_url,
            req.contact_email,
            account_key_ref,
            req.tos_agreed,
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
    req: UpdateIssuerRequest,
) -> Result<IssuerConfigDto, String> {
    let store = store.inner().clone();
    let secrets = secrets.inner().clone();
    spawn_blocking(move || {
        if req.label.trim().is_empty() {
            return Err(anyhow::anyhow!("issuer label is required"));
        }
        validate_acme_requirements(&IssuerType::Acme, req.contact_email.as_ref(), req.tos_agreed)?;
        if req.directory_url.trim().is_empty() {
            return Err(anyhow::anyhow!("directory URL is required"));
        }

        let existing = store
            .get(&req.issuer_id)?
            .ok_or_else(|| anyhow::anyhow!("issuer not found: {}", req.issuer_id))?;
        let record = store.update(
            &req.issuer_id,
            req.label,
            environment_to_string(&req.environment),
            req.directory_url,
            req.contact_email,
            req.tos_agreed,
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
            store.set_account_key_ref(&req.issuer_id, secret_record.id)?
        } else {
            record
        };
        Ok(issuer_record_to_dto(record))
    })
    .await
    .map_err(|err| format!("Update issuer join error: {err}"))?
    .map_err(|err: anyhow::Error| err.to_string())
}

/// Enables or disables an issuer entry.
#[tauri::command]
pub async fn set_issuer_disabled(
    store: State<'_, IssuerConfigStore>,
    req: SetIssuerDisabledRequest,
) -> Result<IssuerConfigDto, String> {
    let store = store.inner().clone();
    spawn_blocking(move || {
        let record = store.set_disabled(&req.issuer_id, req.disabled)?;
        Ok(issuer_record_to_dto(record))
    })
    .await
    .map_err(|err| format!("Set issuer disabled join error: {err}"))?
    .map_err(|err: anyhow::Error| err.to_string())
}

/// Deletes an issuer entry and its associated account key if present.
#[tauri::command]
pub async fn delete_issuer(
    store: State<'_, IssuerConfigStore>,
    secrets: State<'_, SecretManager>,
    req: DeleteIssuerRequest,
) -> Result<String, String> {
    let store = store.inner().clone();
    let secrets = secrets.inner().clone();
    spawn_blocking(move || {
        let record = store
            .get(&req.issuer_id)?
            .ok_or_else(|| anyhow::anyhow!("issuer not found: {}", req.issuer_id))?;
        store.delete(&req.issuer_id)?;
        if let Some(account_key_ref) = record.account_key_ref {
            secrets
                .delete_secret(&account_key_ref)
                .map_err(|err| anyhow::anyhow!(err.to_string()))?;
        }
        Ok(req.issuer_id)
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
        disabled: record.disabled,
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
