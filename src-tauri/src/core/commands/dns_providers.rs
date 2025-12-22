use anyhow::Context;
use tauri::{async_runtime::spawn_blocking, State};

use crate::core::types::{
    CreateDnsProviderRequest, DeleteDnsProviderRequest, DnsProviderDto, DnsProviderErrorCategory,
    DnsProviderResolutionDto, DnsProviderTestResult, DnsProviderTokenValidationResult,
    DnsProviderType, PropagationDto, ResolveDnsProviderRequest, TestDnsProviderRequest,
    UpdateDnsProviderRequest, ValidateDnsProviderTokenRequest,
};
use crate::issuance::dns::check_txt_record;
use crate::issuance::dns_providers::adapter_for_provider;
use crate::secrets::{
    manager::{SecretError, SecretManager},
    types::SecretKind,
};
use crate::storage::dns::{parse_domain_suffixes, DnsConfigStore, DnsProvider};
use std::time::{Duration, Instant};
use uuid::Uuid;

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
                    let access_key = req
                        .route53_access_key
                        .clone()
                        .filter(|v| !v.trim().is_empty())
                        .ok_or_else(|| anyhow::anyhow!("Route 53 access key is required"))?;
                    let secret_key = req
                        .route53_secret_key
                        .clone()
                        .filter(|v| !v.trim().is_empty())
                        .ok_or_else(|| anyhow::anyhow!("Route 53 secret key is required"))?;

                    let access_key_label = format!("Route 53 access key: {}", req.label.trim());
                    let secret_key_label = format!("Route 53 secret key: {}", req.label.trim());

                    let access_key_record = secrets
                        .create_secret(
                            SecretKind::DnsProviderAccessKey,
                            access_key_label,
                            access_key,
                        )
                        .map_err(|err| anyhow::anyhow!(err.to_string()))?;
                    let secret_key_record = secrets
                        .create_secret(
                            SecretKind::DnsProviderSecretKey,
                            secret_key_label,
                            secret_key,
                        )
                        .map_err(|err| anyhow::anyhow!(err.to_string()))?;

                    secret_refs.push(access_key_record.id);
                    secret_refs.push(secret_key_record.id);
                }
                _ => {
                    let token = req
                        .api_token
                        .clone()
                        .filter(|value| !value.trim().is_empty())
                        .ok_or_else(|| {
                            anyhow::anyhow!("API token is required for this provider")
                        })?;
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

        if matches!(provider_type, DnsProviderType::Route53) {
            if let (Some(access_key), Some(secret_key)) = (
                req.route53_access_key.clone().filter(|v| !v.trim().is_empty()),
                req.route53_secret_key
                    .clone()
                    .filter(|v| !v.trim().is_empty()),
            ) {
                for secret_ref in &secret_refs {
                    let _ = secrets.delete_secret(secret_ref);
                }
                secret_refs.clear();

                let access_key_label = format!("Route 53 access key: {}", req.label.trim());
                let secret_key_label = format!("Route 53 secret key: {}", req.label.trim());

                let access_key_record = secrets
                    .create_secret(
                        SecretKind::DnsProviderAccessKey,
                        access_key_label,
                        access_key,
                    )
                    .map_err(|err| anyhow::anyhow!(err.to_string()))?;
                let secret_key_record = secrets
                    .create_secret(
                        SecretKind::DnsProviderSecretKey,
                        secret_key_label,
                        secret_key,
                    )
                    .map_err(|err| anyhow::anyhow!(err.to_string()))?;

                secret_refs.push(access_key_record.id);
                secret_refs.push(secret_key_record.id);
            }
        } else if let Some(token) = req.api_token.clone().filter(|value| !value.trim().is_empty())
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
        let record = store
            .get_provider(&req.provider_id)?
            .ok_or_else(|| anyhow::anyhow!("provider not found: {}", req.provider_id))?;
        for secret_ref in &record.secret_refs {
            match secrets.delete_secret(secret_ref) {
                Ok(()) => {}
                Err(SecretError::NotFound(_)) => {}
                Err(err) => return Err(anyhow::anyhow!(err.to_string())),
            }
        }
        store.delete_provider(&req.provider_id)?;
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
    if err_str.contains("auth")
        || err_str.contains("unauthorized")
        || err_str.contains("forbidden")
        || err_str.contains("invalid credentials")
        || err_str.contains("access denied")
    {
        DnsProviderErrorCategory::AuthError
    } else if err_str.contains("not found")
        || err_str.contains("404")
        || err_str.contains("no such")
    {
        DnsProviderErrorCategory::NotFound
    } else if err_str.contains("rate limit")
        || err_str.contains("429")
        || err_str.contains("too many requests")
    {
        DnsProviderErrorCategory::RateLimited
    } else if err_str.contains("network")
        || err_str.contains("timeout")
        || err_str.contains("connection")
        || err_str.contains("dns")
    {
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
