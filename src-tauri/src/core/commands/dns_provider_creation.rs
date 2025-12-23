use anyhow::anyhow;
use tauri::{async_runtime::spawn_blocking, State};

use crate::core::types::{CreateDnsProviderRequest, DnsProviderDto, DnsProviderType};
use crate::secrets::{manager::SecretManager, types::SecretKind};
use crate::storage::dns::DnsConfigStore;

use super::dns_provider_helpers::{validate_domain_suffixes, validate_label};
use super::dns_provider_management::provider_record_to_dto;

/// Creates a DNS provider configuration.
#[tauri::command]
pub async fn dns_provider_create(
    store: State<'_, DnsConfigStore>,
    secrets: State<'_, SecretManager>,
    create_req: CreateDnsProviderRequest,
) -> Result<DnsProviderDto, String> {
    let store = store.inner().clone();
    let secrets = secrets.inner().clone();
    spawn_blocking(move || -> Result<DnsProviderDto, anyhow::Error> {
        let label = create_req.label.trim();
        validate_label(label)?;
        let domain_suffixes = validate_domain_suffixes(&create_req.domain_suffixes)?;
        let provider_type = provider_type_to_string(&create_req.provider_type);
        let needs_token = !matches!(create_req.provider_type, DnsProviderType::Manual);
        let mut secret_refs = Vec::new();

        if needs_token {
            match create_req.provider_type {
                DnsProviderType::Route53 => {
                    let access_key = create_req
                        .route53_access_key
                        .clone()
                        .filter(|value| !value.trim().is_empty())
                        .ok_or_else(|| anyhow!("Route 53 access key is required"))?;
                    let secret_key = create_req
                        .route53_secret_key
                        .clone()
                        .filter(|value| !value.trim().is_empty())
                        .ok_or_else(|| anyhow!("Route 53 secret key is required"))?;
                    let mut route53_refs = create_route53_credentials(
                        &secrets,
                        label,
                        access_key,
                        secret_key,
                    )?;
                    secret_refs.append(&mut route53_refs);
                }
                _ => {
                    let token = create_req
                        .api_token
                        .clone()
                        .filter(|value| !value.trim().is_empty())
                        .ok_or_else(|| anyhow!("API token is required for this provider"))?;
                    let mut token_refs = create_api_token_credential(&secrets, label, token)?;
                    secret_refs.append(&mut token_refs);
                }
            }
        }

        let record = store.create_provider(
            provider_type,
            label.to_string(),
            domain_suffixes,
            secret_refs,
            create_req.config.clone(),
        )?;
        Ok(provider_record_to_dto(record))
    })
    .await
    .map_err(|err| format!("DNS provider create join error: {err}"))?
    .map_err(|err: anyhow::Error| err.to_string())
}

fn provider_type_to_string(provider_type: &DnsProviderType) -> String {
    match provider_type {
        DnsProviderType::Cloudflare => "cloudflare".to_string(),
        DnsProviderType::DigitalOcean => "digitalocean".to_string(),
        DnsProviderType::Route53 => "route53".to_string(),
        DnsProviderType::Manual => "manual".to_string(),
    }
}

fn create_route53_credentials(
    secrets: &SecretManager,
    label: &str,
    access_key: String,
    secret_key: String,
) -> Result<Vec<String>, anyhow::Error> {
    let access_key_label = format!("Route 53 access key: {}", label.trim());
    let secret_key_label = format!("Route 53 secret key: {}", label.trim());

    let access_key_record = secrets
        .create_secret(
            SecretKind::DnsProviderAccessKey,
            access_key_label,
            access_key,
        )
        .map_err(|err| anyhow!(err.to_string()))?;
    let secret_key_record = secrets
        .create_secret(
            SecretKind::DnsProviderSecretKey,
            secret_key_label,
            secret_key,
        )
        .map_err(|err| anyhow!(err.to_string()))?;

    Ok(vec![access_key_record.id, secret_key_record.id])
}

fn create_api_token_credential(
    secrets: &SecretManager,
    label: &str,
    token: String,
) -> Result<Vec<String>, anyhow::Error> {
    let token_label = format!("DNS provider token: {}", label.trim());
    let record = secrets
        .create_secret(SecretKind::DnsProviderToken, token_label, token)
        .map_err(|err| anyhow!(err.to_string()))?;
    Ok(vec![record.id])
}
