use tauri::{async_runtime::spawn_blocking, State};

use crate::{
    core::types::{
        CertificateRecord, CreateSecretRequest, EnsureAcmeAccountRequest, IssuerConfigDto,
        IssuerEnvironment, SecretRefRecord, SelectIssuerRequest, UpdateSecretRequest,
        PrepareDnsChallengeRequest, PreparedDnsChallenge, CheckPropagationRequest,
        PropagationDto,
    },
    issuance::{
        acme::AcmeIssuer,
        dns::{DnsAdapter, DnsChallengeRequest, ManualDnsAdapter},
    },
    secrets::manager::SecretManager,
    storage::{inventory::InventoryStore, issuer::IssuerConfigStore, dns::DnsConfigStore},
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

/// Ensures an ACME account exists and stores the account key reference.
#[tauri::command]
pub async fn ensure_acme_account(
    store: State<'_, IssuerConfigStore>,
    secrets: State<'_, SecretManager>,
    req: EnsureAcmeAccountRequest,
) -> Result<IssuerConfigDto, String> {
    eprintln!(
        "[ensure_acme_account] start issuer={} email_present={} key_ref_present={} generate_new={}",
        req.issuer_id,
        req.contact_email.is_some(),
        req.account_key_ref.is_some(),
        req.generate_new_account_key
    );
    let store = store.inner().clone();
    let secrets = secrets.inner().clone();
    spawn_blocking(move || {
        let config = store
            .get(&req.issuer_id)?
            .ok_or_else(|| anyhow::anyhow!("issuer not found: {}", req.issuer_id))?;

        let issuer = AcmeIssuer {
            config: config.clone(),
            secrets: &secrets,
        };

        let account_state = issuer.ensure_account(
            req.contact_email.clone(),
            req.account_key_ref.clone(),
            req.generate_new_account_key,
        )?;

        let updated = store.upsert_account_state(
            &req.issuer_id,
            Some(account_state.contact_email),
            Some(account_state.account_key_ref),
        )?;

        eprintln!(
            "[ensure_acme_account] updated issuer={} key_ref_present={} email_present={}",
            updated.issuer_id,
            updated.account_key_ref.is_some(),
            updated.contact_email.is_some()
        );
        Ok(issuer_record_to_dto(updated))
    })
    .await
    .map_err(|err| format!("Ensure account join error: {err}"))?
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

fn issuer_record_to_dto(record: crate::storage::issuer::IssuerConfigRecord) -> IssuerConfigDto {
    let environment = match record.environment.as_str() {
        "production" => IssuerEnvironment::Production,
        _ => IssuerEnvironment::Staging,
    };

    IssuerConfigDto {
        issuer_id: record.issuer_id,
        label: record.label,
        directory_url: record.directory_url,
        environment,
        contact_email: record.contact_email,
        account_key_ref: record.account_key_ref,
        is_selected: record.is_selected,
        disabled: record.disabled,
    }
}
