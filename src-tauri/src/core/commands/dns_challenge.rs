use log::warn;
use tauri::{async_runtime::spawn_blocking, State};

use crate::core::types::{
    CheckPropagationRequest, PrepareDnsChallengeRequest, PreparedDnsChallenge, PropagationDto,
};
use crate::issuance::dns::{DnsAdapter, DnsChallengeRequest, ManualDnsAdapter};
use crate::storage::dns::{DnsConfigStore, DnsProvider};

/// Computes manual DNS instructions for a DNS-01 challenge.
#[tauri::command]
pub async fn prepare_dns_challenge(
    store: State<'_, DnsConfigStore>,
    prepare_req: PrepareDnsChallengeRequest,
) -> Result<PreparedDnsChallenge, String> {
    let store = store.inner().clone();
    spawn_blocking(move || -> Result<PreparedDnsChallenge, anyhow::Error> {
        let dns_adapter = ManualDnsAdapter::new();
        let resolution = store.resolve_provider_for_domain(&prepare_req.domain)?;
        let zone_override = resolution
            .provider
            .as_ref()
            .and_then(provider_zone_override);
        let challenge = DnsChallengeRequest {
            domain: prepare_req.domain.clone(),
            value: prepare_req.txt_value.clone(),
            zone: zone_override,
        };
        let record = dns_adapter.present_txt(&challenge)?;
        Ok(PreparedDnsChallenge { record })
    })
    .await
    .map_err(|err| format!("Prepare DNS join error: {err}"))?
    .map_err(|err: anyhow::Error| err.to_string())
}

/// Checks TXT record propagation for a DNS-01 challenge.
#[tauri::command]
pub async fn check_dns_propagation(
    check_req: CheckPropagationRequest,
) -> Result<PropagationDto, String> {
    spawn_blocking(move || -> Result<PropagationDto, anyhow::Error> {
        let dns_adapter = ManualDnsAdapter::new();
        let challenge = DnsChallengeRequest {
            domain: check_req.domain.clone(),
            value: check_req.txt_value.clone(),
            zone: None,
        };
        let result = dns_adapter.check_propagation(&challenge)?;
        Ok(result)
    })
    .await
    .map_err(|err| format!("Propagation join error: {err}"))?
    .map_err(|err: anyhow::Error| err.to_string())
}

fn provider_zone_override(provider: &DnsProvider) -> Option<String> {
    let raw = provider.config_json.as_ref()?;
    match serde_json::from_str::<serde_json::Value>(raw) {
        Ok(value) => value
            .get("zone")
            .and_then(|zone| zone.as_str().map(|s| s.to_string())),
        Err(err) => {
            warn!(
                "[dns] invalid provider config_json for {}: {}",
                provider.id, err
            );
            None
        }
    }
}
