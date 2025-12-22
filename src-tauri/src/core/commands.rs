use anyhow::Context;
use tauri::{async_runtime::spawn_blocking, State};

use crate::{
    core::types::{
        CertificateRecord, CheckPropagationRequest, CompleteIssuanceRequest, CreateIssuerRequest,
        CreateSecretRequest, CreateDnsProviderRequest, DeleteDnsProviderRequest, DeleteIssuerRequest,
        DnsProviderDto, DnsProviderErrorCategory, DnsProviderResolutionDto,
        DnsProviderTestResult, DnsProviderTokenValidationResult, DnsProviderType,
        IssuerConfigDto, IssuerEnvironment, IssuerType, PrepareDnsChallengeRequest,
        PreparedDnsChallenge, PropagationDto, ResolveDnsProviderRequest, SecretRefRecord,
        SelectIssuerRequest, SetIssuerDisabledRequest, StartIssuanceRequest,
        StartIssuanceResponse, TestDnsProviderRequest, UpdateDnsProviderRequest,
        UpdateIssuerRequest, UpdateSecretRequest, ValidateDnsProviderTokenRequest,
    },
    issuance::{
        acme::generate_account_key_pem,
        dns::{check_txt_record, DnsAdapter, DnsChallengeRequest, ManualDnsAdapter},
        dns_providers::adapter_for_provider,
        flow::{complete_managed_dns01, start_managed_dns01},
    },
    secrets::{manager::SecretManager, types::SecretKind},
    storage::{
        dns::{parse_domain_suffixes, DnsConfigStore, DnsProvider},
        inventory::InventoryStore,
        issuer::IssuerConfigStore,
    },
};
use std::time::{Duration, Instant};
use uuid::Uuid;
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
        let resolution = store.resolve_provider_for_domain(&req.domain)?;
        let zone_override = resolution
            .provider
            .as_ref()
            .and_then(provider_zone_override);
        let challenge = DnsChallengeRequest {
            domain: req.domain.clone(),
            value: req.txt_value.clone(),
            zone: zone_override,
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

/// Lists DNS providers.
#[tauri::command]
pub async fn dns_provider_list(
    store: State<'_, DnsConfigStore>,
) -> Result<Vec<DnsProviderDto>, String> {
    let store = store.inner().clone();
    spawn_blocking(move || -> Result<Vec<DnsProviderDto>, anyhow::Error> {
        let records = store.list_providers()?;
        Ok(records.into_iter().map(provider_record_to_dto).collect())
    })
    .await
    .map_err(|err| format!("DNS provider list join error: {err}"))?
    .map_err(|err: anyhow::Error| err.to_string())
}

/// Creates a DNS provider configuration.
#[tauri::command]
pub async fn dns_provider_create(
    store: State<'_, DnsConfigStore>,
    secrets: State<'_, SecretManager>,
    req: CreateDnsProviderRequest,
) -> Result<DnsProviderDto, String> {
    let store = store.inner().clone();
    let secrets = secrets.inner().clone();
    spawn_blocking(move || -> Result<DnsProviderDto, anyhow::Error> {
        if req.label.trim().is_empty() {
            return Err(anyhow::anyhow!("provider label is required"));
        }
        let domain_suffixes = parse_domain_suffixes(&req.domain_suffixes);
        if domain_suffixes.is_empty() {
            return Err(anyhow::anyhow!("at least one domain suffix is required"));
        }
        let provider_type = provider_type_to_string(&req.provider_type);
        let needs_token = !matches!(req.provider_type, DnsProviderType::Manual);
        let mut secret_refs = Vec::new();
        
        if needs_token {
            match req.provider_type {
                DnsProviderType::Route53 => {
                    let access_key = req.route53_access_key
                        .clone()
                        .filter(|v| !v.trim().is_empty())
                        .ok_or_else(|| anyhow::anyhow!("Route 53 access key is required"))?;
                    let secret_key = req.route53_secret_key
                        .clone()
                        .filter(|v| !v.trim().is_empty())
                        .ok_or_else(|| anyhow::anyhow!("Route 53 secret key is required"))?;
                    
                    let access_key_label = format!("Route 53 access key: {}", req.label.trim());
                    let secret_key_label = format!("Route 53 secret key: {}", req.label.trim());
                    
                    let access_key_record = secrets
                        .create_secret(SecretKind::DnsProviderAccessKey, access_key_label, access_key)
                        .map_err(|err| anyhow::anyhow!(err.to_string()))?;
                    let secret_key_record = secrets
                        .create_secret(SecretKind::DnsProviderSecretKey, secret_key_label, secret_key)
                        .map_err(|err| anyhow::anyhow!(err.to_string()))?;
                    
                    secret_refs.push(access_key_record.id);
                    secret_refs.push(secret_key_record.id);
                }
                _ => {
                    let token = req
                        .api_token
                        .clone()
                        .filter(|value| !value.trim().is_empty())
                        .ok_or_else(|| anyhow::anyhow!("API token is required for this provider"))?;
                    let label = format!("DNS provider token: {}", req.label.trim());
                    let record = secrets
                        .create_secret(SecretKind::DnsProviderToken, label, token)
                        .map_err(|err| anyhow::anyhow!(err.to_string()))?;
                    secret_refs.push(record.id);
                }
            }
        }

        let record = store.create_provider(
            provider_type,
            req.label.trim().to_string(),
            domain_suffixes,
            secret_refs,
            req.config.clone(),
        )?;
        Ok(provider_record_to_dto(record))
    })
    .await
    .map_err(|err| format!("DNS provider create join error: {err}"))?
    .map_err(|err: anyhow::Error| err.to_string())
}

/// Updates a DNS provider configuration.
#[tauri::command]
pub async fn dns_provider_update(
    store: State<'_, DnsConfigStore>,
    secrets: State<'_, SecretManager>,
    req: UpdateDnsProviderRequest,
) -> Result<DnsProviderDto, String> {
    let store = store.inner().clone();
    let secrets = secrets.inner().clone();
    spawn_blocking(move || -> Result<DnsProviderDto, anyhow::Error> {
        if req.label.trim().is_empty() {
            return Err(anyhow::anyhow!("provider label is required"));
        }
        let domain_suffixes = parse_domain_suffixes(&req.domain_suffixes);
        if domain_suffixes.is_empty() {
            return Err(anyhow::anyhow!("at least one domain suffix is required"));
        }

        let existing = store
            .get_provider(&req.provider_id)?
            .ok_or_else(|| anyhow::anyhow!("provider not found: {}", req.provider_id))?;

        let mut secret_refs = existing.secret_refs.clone();
        let provider_type = provider_type_from_string(&existing.provider_type);
        
        // Handle Route 53 credentials
        if matches!(provider_type, DnsProviderType::Route53) {
            if let (Some(access_key), Some(secret_key)) = (
                req.route53_access_key.clone().filter(|v| !v.trim().is_empty()),
                req.route53_secret_key.clone().filter(|v| !v.trim().is_empty()),
            ) {
                // Delete old secrets if they exist
                for secret_ref in &secret_refs {
                    let _ = secrets.delete_secret(secret_ref);
                }
                secret_refs.clear();
                
                // Create new secrets
                let access_key_label = format!("Route 53 access key: {}", req.label.trim());
                let secret_key_label = format!("Route 53 secret key: {}", req.label.trim());
                
                let access_key_record = secrets
                    .create_secret(SecretKind::DnsProviderAccessKey, access_key_label, access_key)
                    .map_err(|err| anyhow::anyhow!(err.to_string()))?;
                let secret_key_record = secrets
                    .create_secret(SecretKind::DnsProviderSecretKey, secret_key_label, secret_key)
                    .map_err(|err| anyhow::anyhow!(err.to_string()))?;
                
                secret_refs.push(access_key_record.id);
                secret_refs.push(secret_key_record.id);
            }
        } else if let Some(token) = req
            .api_token
            .clone()
            .filter(|value| !value.trim().is_empty())
        {
            let secret_label = format!("DNS provider token: {}", req.label.trim());
            if let Some(secret_ref) = secret_refs.first() {
                secrets
                    .update_secret(secret_ref, token, Some(secret_label.clone()))
                    .map_err(|err| anyhow::anyhow!(err.to_string()))?;
            } else {
                let record = secrets
                    .create_secret(SecretKind::DnsProviderToken, secret_label.clone(), token)
                    .map_err(|err| anyhow::anyhow!(err.to_string()))?;
                secret_refs.push(record.id);
            }
        }
        
        if !secret_refs.is_empty() {
            store.update_provider_secret_refs(&req.provider_id, secret_refs)?;
        }

        let record = store.update_provider(
            &req.provider_id,
            req.label.trim().to_string(),
            domain_suffixes,
            req.config.clone(),
        )?;
        Ok(provider_record_to_dto(record))
    })
    .await
    .map_err(|err| format!("DNS provider update join error: {err}"))?
    .map_err(|err: anyhow::Error| err.to_string())
}

/// Deletes a DNS provider configuration.
#[tauri::command]
pub async fn dns_provider_delete(
    store: State<'_, DnsConfigStore>,
    secrets: State<'_, SecretManager>,
    req: DeleteDnsProviderRequest,
) -> Result<String, String> {
    let store = store.inner().clone();
    let secrets = secrets.inner().clone();
    spawn_blocking(move || -> Result<String, anyhow::Error> {
        let record = store.delete_provider(&req.provider_id)?;
        // Cleanup all secrets associated with this provider
        for secret_ref in &record.secret_refs {
            if let Err(err) = secrets.delete_secret(secret_ref) {
                eprintln!("Warning: failed to delete secret {}: {}", secret_ref, err);
            }
        }
        Ok(req.provider_id)
    })
    .await
    .map_err(|err| format!("DNS provider delete join error: {err}"))?
    .map_err(|err: anyhow::Error| err.to_string())
}

/// Tests a DNS provider configuration by creating a temporary TXT record.
#[tauri::command]
pub async fn dns_provider_test(
    store: State<'_, DnsConfigStore>,
    secrets: State<'_, SecretManager>,
    req: TestDnsProviderRequest,
) -> Result<DnsProviderTestResult, String> {
    let store = store.inner().clone();
    let secrets = secrets.inner().clone();
    spawn_blocking(move || -> Result<DnsProviderTestResult, anyhow::Error> {
        let started = Instant::now();
        let provider = store
            .get_provider(&req.provider_id)?
            .ok_or_else(|| anyhow::anyhow!("provider not found: {}", req.provider_id))?;

        let suffix = provider
            .domain_suffixes
            .get(0)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("provider has no domain suffixes"))?;
        let random = Uuid::new_v4().as_simple().to_string();
        let record_name = format!("_sslboard-test-{}.{}", &random[..10], suffix);
        let value = format!("sslboard-test-{}", &random[..10]);

        let adapter = adapter_for_provider(&provider, &secrets);

        let create_start = Instant::now();
        if let Err(err) = adapter.create_txt(&record_name, &value) {
            let error_category = categorize_dns_error(&err);
            return Ok(DnsProviderTestResult {
                success: false,
                record_name: Some(record_name),
                value: Some(value),
                propagation: None,
                error: Some(err.to_string()),
                error_category: Some(error_category),
                error_stage: Some("create".to_string()),
                elapsed_ms: started.elapsed().as_millis() as u64,
                create_ms: Some(create_start.elapsed().as_millis() as u64),
                propagation_ms: None,
                cleanup_ms: None,
            });
        }
        let create_ms = create_start.elapsed().as_millis() as u64;

        let propagation_start = Instant::now();
        let propagation = match poll_txt_propagation(&record_name, &value) {
            Ok(result) => result,
            Err(err) => {
                let propagation_ms = propagation_start.elapsed().as_millis() as u64;
                let cleanup_start = Instant::now();
                let cleanup_result = adapter.cleanup_txt(&record_name);
                let cleanup_ms = cleanup_start.elapsed().as_millis() as u64;
                if let Err(cleanup_err) = cleanup_result {
                    let error_category = categorize_dns_error(&cleanup_err);
                    return Ok(DnsProviderTestResult {
                        success: false,
                        record_name: Some(record_name),
                        value: Some(value),
                        propagation: None,
                        error: Some(cleanup_err.to_string()),
                        error_category: Some(error_category),
                        error_stage: Some("cleanup".to_string()),
                        elapsed_ms: started.elapsed().as_millis() as u64,
                        create_ms: Some(create_ms),
                        propagation_ms: Some(propagation_ms),
                        cleanup_ms: Some(cleanup_ms),
                    });
                }
                let error_category = categorize_dns_error(&err);
                return Ok(DnsProviderTestResult {
                    success: false,
                    record_name: Some(record_name),
                    value: Some(value),
                    propagation: None,
                    error: Some(err.to_string()),
                    error_category: Some(error_category),
                    error_stage: Some("propagation".to_string()),
                    elapsed_ms: started.elapsed().as_millis() as u64,
                    create_ms: Some(create_ms),
                    propagation_ms: Some(propagation_ms),
                    cleanup_ms: Some(cleanup_ms),
                });
            }
        };
        let propagation_ms = propagation_start.elapsed().as_millis() as u64;
        let cleanup_start = Instant::now();
        let cleanup_result = adapter.cleanup_txt(&record_name);
        let cleanup_ms = cleanup_start.elapsed().as_millis() as u64;

        if let Err(err) = cleanup_result {
            let error_category = categorize_dns_error(&err);
            return Ok(DnsProviderTestResult {
                success: false,
                record_name: Some(record_name),
                value: Some(value),
                propagation: Some(propagation),
                error: Some(err.to_string()),
                error_category: Some(error_category),
                error_stage: Some("cleanup".to_string()),
                elapsed_ms: started.elapsed().as_millis() as u64,
                create_ms: Some(create_ms),
                propagation_ms: Some(propagation_ms),
                cleanup_ms: Some(cleanup_ms),
            });
        }

        let success = matches!(
            propagation.state,
            crate::issuance::dns::PropagationState::Found
        );

        Ok(DnsProviderTestResult {
            success,
            record_name: Some(record_name),
            value: Some(value),
            propagation: Some(propagation),
            error: None,
            error_category: None,
            error_stage: None,
            elapsed_ms: started.elapsed().as_millis() as u64,
            create_ms: Some(create_ms),
            propagation_ms: Some(propagation_ms),
            cleanup_ms: Some(cleanup_ms),
        })
    })
    .await
    .map_err(|err| format!("DNS provider test join error: {err}"))?
    .map_err(|err: anyhow::Error| err.to_string())
}

/// Validates DNS provider credentials without storing them.
#[tauri::command]
pub async fn dns_provider_validate_token(
    req: ValidateDnsProviderTokenRequest,
) -> Result<DnsProviderTokenValidationResult, String> {
    spawn_blocking(move || -> Result<DnsProviderTokenValidationResult, anyhow::Error> {
        let result = match req.provider_type {
            DnsProviderType::Cloudflare => {
                let token = req
                    .api_token
                    .clone()
                    .filter(|value| !value.trim().is_empty())
                    .ok_or_else(|| anyhow::anyhow!("API token is required for Cloudflare"))?;
                validate_cloudflare_token(&token)
            }
            DnsProviderType::DigitalOcean => {
                let token = req
                    .api_token
                    .clone()
                    .filter(|value| !value.trim().is_empty())
                    .ok_or_else(|| anyhow::anyhow!("API token is required for DigitalOcean"))?;
                validate_digitalocean_token(&token)
            }
            DnsProviderType::Route53 => {
                let access_key = req
                    .route53_access_key
                    .clone()
                    .filter(|value| !value.trim().is_empty())
                    .ok_or_else(|| anyhow::anyhow!("Route 53 access key is required"))?;
                let secret_key = req
                    .route53_secret_key
                    .clone()
                    .filter(|value| !value.trim().is_empty())
                    .ok_or_else(|| anyhow::anyhow!("Route 53 secret key is required"))?;
                validate_route53_token(&access_key, &secret_key)
            }
            DnsProviderType::Manual => Err(anyhow::anyhow!(
                "manual DNS providers do not require token validation"
            )),
        };

        match result {
            Ok(()) => Ok(DnsProviderTokenValidationResult {
                success: true,
                error: None,
                error_category: None,
            }),
            Err(err) => {
                let category = categorize_dns_error(&err);
                Ok(DnsProviderTokenValidationResult {
                    success: false,
                    error: Some(err.to_string()),
                    error_category: Some(category),
                })
            }
        }
    })
    .await
    .map_err(|err| format!("DNS provider token validation join error: {err}"))?
    .map_err(|err: anyhow::Error| err.to_string())
}

/// Resolves a DNS provider for a hostname.
#[tauri::command]
pub async fn dns_resolve_provider(
    store: State<'_, DnsConfigStore>,
    req: ResolveDnsProviderRequest,
) -> Result<DnsProviderResolutionDto, String> {
    let store = store.inner().clone();
    spawn_blocking(move || -> Result<DnsProviderResolutionDto, anyhow::Error> {
        let resolution = store.resolve_provider_for_domain(&req.hostname)?;
        Ok(DnsProviderResolutionDto {
            provider: resolution.provider.map(provider_record_to_dto),
            matched_suffix: resolution.matched_suffix,
            ambiguous: resolution
                .ambiguous
                .into_iter()
                .map(provider_record_to_dto)
                .collect(),
        })
    })
    .await
    .map_err(|err| format!("DNS resolve provider join error: {err}"))?
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

fn provider_record_to_dto(record: DnsProvider) -> DnsProviderDto {
    let provider_type = provider_type_from_string(&record.provider_type);
    let config = record
        .config_json
        .as_ref()
        .and_then(|raw| serde_json::from_str::<serde_json::Value>(raw).ok());
    DnsProviderDto {
        id: record.id,
        provider_type,
        label: record.label,
        domain_suffixes: record.domain_suffixes,
        config,
        created_at: record.created_at,
        updated_at: record.updated_at,
    }
}

fn provider_type_to_string(provider_type: &DnsProviderType) -> String {
    match provider_type {
        DnsProviderType::Cloudflare => "cloudflare".to_string(),
        DnsProviderType::DigitalOcean => "digitalocean".to_string(),
        DnsProviderType::Route53 => "route53".to_string(),
        DnsProviderType::Manual => "manual".to_string(),
    }
}

fn provider_type_from_string(raw: &str) -> DnsProviderType {
    match raw {
        "cloudflare" => DnsProviderType::Cloudflare,
        "digitalocean" => DnsProviderType::DigitalOcean,
        "route53" => DnsProviderType::Route53,
        _ => DnsProviderType::Manual,
    }
}

fn provider_zone_override(provider: &DnsProvider) -> Option<String> {
    provider
        .config_json
        .as_ref()
        .and_then(|raw| serde_json::from_str::<serde_json::Value>(raw).ok())
        .and_then(|value| value.get("zone").and_then(|zone| zone.as_str().map(|s| s.to_string())))
}

fn poll_txt_propagation(record_name: &str, value: &str) -> Result<PropagationDto, anyhow::Error> {
    let timeout = Duration::from_secs(30);
    let interval = Duration::from_secs(2);
    let started = Instant::now();
    let mut last = check_txt_record(record_name, value)?;
    loop {
        if matches!(last.state, crate::issuance::dns::PropagationState::Found) {
            return Ok(last);
        }
        if started.elapsed() >= timeout {
            return Ok(last);
        }
        std::thread::sleep(interval);
        last = check_txt_record(record_name, value)?;
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

fn validate_cloudflare_token(token: &str) -> Result<(), anyhow::Error> {
    #[derive(serde::Deserialize)]
    struct CloudflareZoneListResponse {
        success: bool,
    }

    let client = reqwest::blocking::Client::new();
    let response = client
        .get("https://api.cloudflare.com/client/v4/zones")
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .send()
        .context("Failed to list Cloudflare zones")?;

    if !response.status().is_success() {
        if response.status() == 401 || response.status() == 403 {
            return Err(anyhow::anyhow!(
                "Cloudflare authentication failed: invalid API token"
            ));
        }
        if response.status() == 429 {
            return Err(anyhow::anyhow!("Cloudflare rate limit exceeded"));
        }
        return Err(anyhow::anyhow!(
            "Cloudflare API error: {}",
            response.status()
        ));
    }

    let payload: CloudflareZoneListResponse = response
        .json()
        .context("Failed to parse Cloudflare zone list response")?;
    if !payload.success {
        return Err(anyhow::anyhow!(
            "Cloudflare API returned unsuccessful response"
        ));
    }
    Ok(())
}

fn validate_digitalocean_token(token: &str) -> Result<(), anyhow::Error> {
    let client = reqwest::blocking::Client::new();
    let response = client
        .get("https://api.digitalocean.com/v2/domains")
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .context("Failed to list DigitalOcean domains")?;

    if !response.status().is_success() {
        if response.status() == 401 || response.status() == 403 {
            return Err(anyhow::anyhow!("DigitalOcean authentication failed"));
        }
        if response.status() == 429 {
            return Err(anyhow::anyhow!("DigitalOcean rate limit exceeded"));
        }
        return Err(anyhow::anyhow!(
            "DigitalOcean API error: {}",
            response.status()
        ));
    }
    Ok(())
}

fn validate_route53_token(access_key: &str, secret_key: &str) -> Result<(), anyhow::Error> {
    let rt = tokio::runtime::Runtime::new().context("Failed to create tokio runtime")?;
    rt.block_on(async move {
        use aws_config::BehaviorVersion;
        use aws_sdk_route53::config::Credentials;
        use aws_sdk_route53::Client;

        let credentials = Credentials::new(access_key, secret_key, None, None, "sslboard");
        let config = aws_config::defaults(BehaviorVersion::latest())
            .credentials_provider(credentials)
            .load()
            .await;
        let client = Client::new(&config);
        client
            .list_hosted_zones()
            .send()
            .await
            .context("Failed to list Route 53 hosted zones")?;
        Ok(())
    })
}

fn categorize_dns_error(err: &anyhow::Error) -> DnsProviderErrorCategory {
    let err_str = err.to_string().to_lowercase();
    if err_str.contains("auth") || err_str.contains("unauthorized") || err_str.contains("forbidden") || err_str.contains("invalid credentials") || err_str.contains("access denied") {
        DnsProviderErrorCategory::AuthError
    } else if err_str.contains("not found") || err_str.contains("404") || err_str.contains("no such") {
        DnsProviderErrorCategory::NotFound
    } else if err_str.contains("rate limit") || err_str.contains("429") || err_str.contains("too many requests") {
        DnsProviderErrorCategory::RateLimited
    } else if err_str.contains("network") || err_str.contains("timeout") || err_str.contains("connection") || err_str.contains("dns") {
        DnsProviderErrorCategory::NetworkError
    } else {
        DnsProviderErrorCategory::Unknown
    }
}

#[cfg(test)]
mod tests {
    use super::{categorize_dns_error, DnsProviderErrorCategory};

    #[test]
    fn categorizes_auth_errors() {
        let err = anyhow::anyhow!("authentication failed");
        assert!(matches!(
            categorize_dns_error(&err),
            DnsProviderErrorCategory::AuthError
        ));
    }

    #[test]
    fn categorizes_not_found_errors() {
        let err = anyhow::anyhow!("zone not found");
        assert!(matches!(
            categorize_dns_error(&err),
            DnsProviderErrorCategory::NotFound
        ));
    }

    #[test]
    fn categorizes_rate_limit_errors() {
        let err = anyhow::anyhow!("429 too many requests");
        assert!(matches!(
            categorize_dns_error(&err),
            DnsProviderErrorCategory::RateLimited
        ));
    }

    #[test]
    fn categorizes_network_errors() {
        let err = anyhow::anyhow!("network timeout");
        assert!(matches!(
            categorize_dns_error(&err),
            DnsProviderErrorCategory::NetworkError
        ));
    }

    #[test]
    fn categorizes_unknown_errors() {
        let err = anyhow::anyhow!("unexpected issue");
        assert!(matches!(
            categorize_dns_error(&err),
            DnsProviderErrorCategory::Unknown
        ));
    }
}
