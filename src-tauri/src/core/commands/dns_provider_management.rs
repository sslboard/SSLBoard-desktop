use log::warn;
use tauri::{async_runtime::spawn_blocking, State};

use crate::core::types::{
    DeleteDnsProviderRequest, DnsProviderDto, DnsProviderResolutionDto, DnsProviderType,
    ResolveDnsProviderRequest, UpdateDnsProviderRequest,
};
use crate::domain::normalize_domain_for_display;
use crate::secrets::manager::{SecretError, SecretManager};
use crate::storage::dns::{DnsConfigStore, DnsProvider};

use super::dns_provider_helpers::{validate_domain_suffixes, validate_label};
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

/// Updates a DNS provider configuration.
#[tauri::command]
pub async fn dns_provider_update(
    store: State<'_, DnsConfigStore>,
    secrets: State<'_, SecretManager>,
    update_req: UpdateDnsProviderRequest,
) -> Result<DnsProviderDto, String> {
    let store = store.inner().clone();
    let secrets = secrets.inner().clone();
    spawn_blocking(move || -> Result<DnsProviderDto, anyhow::Error> {
        let label = update_req.label.trim();
        validate_label(label)?;
        let domain_suffixes = validate_domain_suffixes(&update_req.domain_suffixes)?;

        let existing = store
            .get_provider(&update_req.provider_id)?
            .ok_or_else(|| anyhow::anyhow!("provider not found: {}", update_req.provider_id))?;

        let mut secret_refs = existing.secret_refs.clone();
        let provider_type = provider_type_from_string(&existing.provider_type);

        if matches!(provider_type, DnsProviderType::Route53) {
            if let (Some(access_key), Some(secret_key)) = (
                update_req
                    .route53_access_key
                    .clone()
                    .filter(|value| !value.trim().is_empty()),
                update_req
                    .route53_secret_key
                    .clone()
                    .filter(|value| !value.trim().is_empty()),
            ) {
                for secret_ref in &secret_refs {
                    match secrets.delete_secret(secret_ref) {
                        Ok(()) => {}
                        Err(SecretError::NotFound(_)) => {}
                        Err(err) => return Err(anyhow::anyhow!(err.to_string())),
                    }
                }
                secret_refs.clear();

                let access_key_label = format!("Route 53 access key: {}", label);
                let secret_key_label = format!("Route 53 secret key: {}", label);

                let access_key_record = secrets
                    .create_secret(
                        crate::secrets::types::SecretKind::DnsProviderAccessKey,
                        access_key_label,
                        access_key,
                    )
                    .map_err(|err| anyhow::anyhow!(err.to_string()))?;
                let secret_key_record = secrets
                    .create_secret(
                        crate::secrets::types::SecretKind::DnsProviderSecretKey,
                        secret_key_label,
                        secret_key,
                    )
                    .map_err(|err| anyhow::anyhow!(err.to_string()))?;

                secret_refs.push(access_key_record.id);
                secret_refs.push(secret_key_record.id);
            }
        } else if let Some(token) = update_req
            .api_token
            .clone()
            .filter(|value| !value.trim().is_empty())
        {
            let secret_label = format!("DNS provider token: {}", label);
            if let Some(secret_ref) = secret_refs.first() {
                secrets
                    .update_secret(secret_ref, token, Some(secret_label.clone()))
                    .map_err(|err| anyhow::anyhow!(err.to_string()))?;
            } else {
                let record = secrets
                    .create_secret(
                        crate::secrets::types::SecretKind::DnsProviderToken,
                        secret_label.clone(),
                        token,
                    )
                    .map_err(|err| anyhow::anyhow!(err.to_string()))?;
                secret_refs.push(record.id);
            }
        }

        if !secret_refs.is_empty() {
            store.update_provider_secret_refs(&update_req.provider_id, secret_refs)?;
        }

        let record = store.update_provider(
            &update_req.provider_id,
            label.to_string(),
            domain_suffixes,
            update_req.config.clone(),
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
    delete_req: DeleteDnsProviderRequest,
) -> Result<String, String> {
    let store = store.inner().clone();
    let secrets = secrets.inner().clone();
    spawn_blocking(move || -> Result<String, anyhow::Error> {
        let record = store
            .get_provider(&delete_req.provider_id)?
            .ok_or_else(|| anyhow::anyhow!("provider not found: {}", delete_req.provider_id))?;
        for secret_ref in &record.secret_refs {
            match secrets.delete_secret(secret_ref) {
                Ok(()) => {}
                Err(SecretError::NotFound(_)) => {}
                Err(err) => return Err(anyhow::anyhow!(err.to_string())),
            }
        }
        store.delete_provider(&delete_req.provider_id)?;
        Ok(delete_req.provider_id)
    })
    .await
    .map_err(|err| format!("DNS provider delete join error: {err}"))?
    .map_err(|err: anyhow::Error| err.to_string())
}

/// Resolves a DNS provider for a hostname.
#[tauri::command]
pub async fn dns_resolve_provider(
    store: State<'_, DnsConfigStore>,
    resolve_req: ResolveDnsProviderRequest,
) -> Result<DnsProviderResolutionDto, String> {
    let store = store.inner().clone();
    spawn_blocking(move || -> Result<DnsProviderResolutionDto, anyhow::Error> {
        let resolution = store.resolve_provider_for_domain(&resolve_req.hostname)?;
        Ok(DnsProviderResolutionDto {
            provider: resolution.provider.map(provider_record_to_dto),
            matched_suffix: resolution
                .matched_suffix
                .map(|suffix| normalize_domain_for_display(&suffix)),
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

pub(crate) fn provider_record_to_dto(record: DnsProvider) -> DnsProviderDto {
    let provider_type = provider_type_from_string(&record.provider_type);
    let config = match record.config_json.as_ref() {
        Some(raw) => match serde_json::from_str::<serde_json::Value>(raw) {
            Ok(value) => Some(value),
            Err(err) => {
                warn!(
                    "[dns] invalid provider config_json for {}: {}",
                    record.id, err
                );
                None
            }
        },
        None => None,
    };
    DnsProviderDto {
        id: record.id,
        provider_type,
        label: record.label,
        domain_suffixes: record
            .domain_suffixes
            .into_iter()
            .map(|suffix| normalize_domain_for_display(&suffix))
            .collect(),
        config,
        created_at: record.created_at,
        updated_at: record.updated_at,
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
