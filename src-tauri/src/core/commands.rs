use tauri::{async_runtime::spawn_blocking, State};

use crate::{
    core::types::{
        CertificateRecord, CheckPropagationRequest, CompleteIssuanceRequest, CreateIssuerRequest,
        CreateSecretRequest, DeleteIssuerRequest, IssuerConfigDto, IssuerEnvironment,
        IssuerType, PrepareDnsChallengeRequest, PreparedDnsChallenge, PropagationDto,
        SecretRefRecord, SelectIssuerRequest, SetIssuerDisabledRequest, StartIssuanceRequest,
        StartIssuanceResponse, UpdateIssuerRequest, UpdateSecretRequest,
    },
    issuance::{
        acme::generate_account_key_pem,
        dns::{DnsAdapter, DnsChallengeRequest, ManualDnsAdapter},
        flow::{complete_managed_dns01, start_managed_dns01},
    },
    secrets::{manager::SecretManager, types::SecretKind},
    storage::{dns::DnsConfigStore, inventory::InventoryStore, issuer::IssuerConfigStore},
};
/// A simple greeting command for testing the Tauri-Rust bridge.
/// This command demonstrates basic string processing and command invocation.
///
/// # Arguments
/// * `name` - The name to greet
///
/// # Returns
/// A greeting string from Rust
#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

/// Retrieves all certificate records from the inventory.
/// This command fetches all stored certificate data and returns it as a vector.
///
/// # Returns
/// A Result containing either a vector of CertificateRecord or an error string
#[tauri::command]
pub async fn list_certificates(
    store: State<'_, InventoryStore>,
) -> Result<Vec<CertificateRecord>, String> {
    let store = store.inner().clone();
    spawn_blocking(move || store.list_certificates())
        .await
        .map_err(|err| format!("List join error: {err}"))?
        .map_err(|err| err.to_string())
}

/// Retrieves a specific certificate record by its ID.
/// This command looks up a single certificate in the inventory by its unique identifier.
///
/// # Arguments
/// * `store` - The inventory store state
/// * `id` - The unique identifier of the certificate to retrieve
///
/// # Returns
/// A Result containing either the CertificateRecord or an error string if not found
#[tauri::command]
pub async fn get_certificate(
    store: State<'_, InventoryStore>,
    id: String,
) -> Result<CertificateRecord, String> {
    let store = store.inner().clone();
    let missing_id = id.clone();
    spawn_blocking(move || store.get_certificate(&id))
        .await
        .map_err(|err| format!("Get join error: {err}"))?
        .map_err(|err| err.to_string())?
        .ok_or_else(|| format!("Certificate not found: {missing_id}"))
}

/// Seeds the database with a sample development certificate.
/// This command is used for development and testing purposes to populate
/// the inventory with a fake certificate record. It only adds the sample
/// certificate if the inventory is empty.
///
/// # Returns
/// A Result indicating success or an error string
#[tauri::command]
pub async fn seed_fake_certificate(store: State<'_, InventoryStore>) -> Result<(), String> {
    let store = store.inner().clone();
    spawn_blocking(move || store.seed_dev_certificate())
        .await
        .map_err(|err| format!("Seed join error: {err}"))?
        .map_err(|err| err.to_string())
}

/// Lists secret references (metadata only, no secret bytes).
#[tauri::command]
pub async fn list_secret_refs(
    manager: State<'_, SecretManager>,
) -> Result<Vec<SecretRefRecord>, String> {
    let manager = manager.inner().clone();
    spawn_blocking(move || manager.list())
        .await
        .map_err(|err| format!("List join error: {err}"))?
        .map_err(|err| err.to_string())
}

/// Creates a new secret reference by sending the secret value into the trusted core once.
#[tauri::command]
pub async fn create_secret_ref(
    manager: State<'_, SecretManager>,
    req: CreateSecretRequest,
) -> Result<SecretRefRecord, String> {
    let manager = manager.inner().clone();
    spawn_blocking(move || manager.create_secret(req.kind, req.label, req.secret_value))
        .await
        .map_err(|err| format!("Create join error: {err}"))?
        .map_err(|err| err.to_string())
}

/// Updates an existing secret while keeping the reference id stable.
#[tauri::command]
pub async fn update_secret_ref(
    manager: State<'_, SecretManager>,
    req: UpdateSecretRequest,
) -> Result<SecretRefRecord, String> {
    let manager = manager.inner().clone();
    spawn_blocking(move || manager.update_secret(&req.id, req.secret_value, req.label))
        .await
        .map_err(|err| format!("Update join error: {err}"))?
        .map_err(|err| err.to_string())
}

/// Removes a secret reference and deletes the underlying secret from the OS store.
#[tauri::command]
pub async fn delete_secret_ref(
    manager: State<'_, SecretManager>,
    id: String,
) -> Result<(), String> {
    let manager = manager.inner().clone();
    spawn_blocking(move || manager.delete_secret(&id))
        .await
        .map_err(|err| format!("Delete join error: {err}"))?
        .map_err(|err| err.to_string())
}

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

/// Computes manual DNS instructions for a DNS-01 challenge.
#[tauri::command]
pub async fn prepare_dns_challenge(
    store: State<'_, DnsConfigStore>,
    req: PrepareDnsChallengeRequest,
) -> Result<PreparedDnsChallenge, String> {
    let store = store.inner().clone();
    spawn_blocking(move || -> Result<PreparedDnsChallenge, anyhow::Error> {
        let adapter = ManualDnsAdapter::new();
        let mapping = store.find_for_hostname(&req.domain)?;
        let challenge = DnsChallengeRequest {
            domain: req.domain.clone(),
            value: req.txt_value.clone(),
            zone: mapping.and_then(|m| m.zone),
        };
        let record = adapter.present_txt(&challenge)?;
        Ok(PreparedDnsChallenge { record })
    })
    .await
    .map_err(|err| format!("Prepare DNS join error: {err}"))?
    .map_err(|err: anyhow::Error| err.to_string())
}

/// Checks TXT record propagation for a DNS-01 challenge.
#[tauri::command]
pub async fn check_dns_propagation(req: CheckPropagationRequest) -> Result<PropagationDto, String> {
    spawn_blocking(move || -> Result<PropagationDto, anyhow::Error> {
        let adapter = ManualDnsAdapter::new();
        let challenge = DnsChallengeRequest {
            domain: req.domain.clone(),
            value: req.txt_value.clone(),
            zone: None,
        };
        let result = adapter.check_propagation(&challenge)?;
        Ok(result)
    })
    .await
    .map_err(|err| format!("Propagation join error: {err}"))?
    .map_err(|err: anyhow::Error| err.to_string())
}

/// Starts a managed-key ACME issuance and returns DNS-01 instructions plus a request id.
#[tauri::command]
pub async fn start_managed_issuance(
    issuer_store: State<'_, IssuerConfigStore>,
    dns_store: State<'_, DnsConfigStore>,
    secrets: State<'_, SecretManager>,
    req: StartIssuanceRequest,
) -> Result<StartIssuanceResponse, String> {
    let issuer_store = issuer_store.inner().clone();
    let dns_store = dns_store.inner().clone();
    let secrets = secrets.inner().clone();
    spawn_blocking(move || {
        start_managed_dns01(req.domains, req.issuer_id, &issuer_store, &dns_store, &secrets).map(
            |(request_id, dns_records)| StartIssuanceResponse {
                request_id,
                dns_records,
            },
        )
    })
    .await
    .map_err(|err| format!("Start issuance join error: {err}"))?
    .map_err(|err: anyhow::Error| err.to_string())
}

/// Completes a managed-key ACME issuance after DNS-01 is satisfied.
#[tauri::command]
pub async fn complete_managed_issuance(
    inventory: State<'_, InventoryStore>,
    secrets: State<'_, SecretManager>,
    req: CompleteIssuanceRequest,
) -> Result<CertificateRecord, String> {
    let inventory = inventory.inner().clone();
    let secrets = secrets.inner().clone();
    spawn_blocking(move || complete_managed_dns01(&req.request_id, &inventory, &secrets))
        .await
        .map_err(|err| format!("Complete issuance join error: {err}"))?
        .map_err(|err: anyhow::Error| err.to_string())
}

/// Unlocks the secret vault, loading the master key into memory.
#[tauri::command]
pub async fn unlock_vault(manager: State<'_, SecretManager>) -> Result<bool, String> {
    eprintln!(
        "[vault-cmd] unlock_vault called, is_unlocked={}",
        manager.is_unlocked()
    );
    let manager = manager.inner().clone();
    let result = spawn_blocking(move || manager.unlock())
        .await
        .map_err(|err| format!("Unlock vault join error: {err}"))?
        .map(|_| true)
        .map_err(|err| err.to_string());
    eprintln!("[vault-cmd] unlock_vault result={:?}", result);
    result
}

/// Locks the secret vault, zeroizing the cached master key.
#[tauri::command]
pub async fn lock_vault(manager: State<'_, SecretManager>) -> Result<bool, String> {
    eprintln!(
        "[vault-cmd] lock_vault called, is_unlocked={}",
        manager.is_unlocked()
    );
    let manager = manager.inner().clone();
    let result = spawn_blocking(move || {
        manager.lock();
        Ok(false)
    })
    .await
    .map_err(|err| format!("Lock vault join error: {err}"))?;
    eprintln!("[vault-cmd] lock_vault result={:?}", result);
    result
}

/// Returns whether the vault is currently unlocked.
#[tauri::command]
pub async fn is_vault_unlocked(manager: State<'_, SecretManager>) -> Result<bool, String> {
    eprintln!("[vault-cmd] is_vault_unlocked called");
    let manager = manager.inner().clone();
    let result = spawn_blocking(move || Ok(manager.is_unlocked()))
        .await
        .map_err(|err| format!("Vault status join error: {err}"))?;
    eprintln!("[vault-cmd] is_vault_unlocked result={:?}", result);
    result
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
