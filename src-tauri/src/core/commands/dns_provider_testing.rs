use std::time::{Duration, Instant};

use log::{info, warn};
use tauri::{async_runtime::spawn_blocking, State};
use uuid::Uuid;

use crate::core::types::{DnsProviderTestResult, TestDnsProviderRequest};
use crate::issuance::dns::PropagationState;
use crate::issuance::dns_providers::{adapter_for_provider, poll_dns_propagation};
use crate::secrets::manager::SecretManager;
use crate::storage::dns::DnsConfigStore;

use super::dns_validation::categorize_dns_error;

/// Tests a DNS provider configuration by creating a temporary TXT record.
#[tauri::command]
pub async fn dns_provider_test(
    store: State<'_, DnsConfigStore>,
    secrets: State<'_, SecretManager>,
    test_req: TestDnsProviderRequest,
) -> Result<DnsProviderTestResult, String> {
    let store = store.inner().clone();
    let secrets = secrets.inner().clone();
    spawn_blocking(move || -> Result<DnsProviderTestResult, anyhow::Error> {
        let started = Instant::now();
        info!("[dns-test] Starting DNS provider test for provider_id: {}", test_req.provider_id);
        let provider = store
            .get_provider(&test_req.provider_id)?
            .ok_or_else(|| anyhow::anyhow!("provider not found: {}", test_req.provider_id))?;
        info!("[dns-test] Found provider: type={}, label={}", provider.provider_type, provider.label);

        // Generate test record details
        let suffix = provider
            .domain_suffixes
            .first()
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("provider has no domain suffixes"))?;
        let random = Uuid::new_v4().as_simple().to_string();
        let record_name = format!("_sslboard-test-{}.{}", &random[..10], suffix);
        let value = format!("sslboard-test-{}", &random[..10]);

        info!("[dns-test] Creating test TXT record: {} = {}", record_name, value);
        let provider_adapter = adapter_for_provider(&provider, &secrets);

        let create_start = Instant::now();
        if let Err(err) = provider_adapter.create_txt(&record_name, &value) {
            warn!("[dns-test] Failed to create TXT record: {}", err);
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
        info!("[dns-test] TXT record created in {}ms", create_ms);

        let propagation_start = Instant::now();
        info!("[dns-test] Starting propagation polling for {}", record_name);
        let timeout = Duration::from_secs(30);
        let interval = Duration::from_secs(2);
        let propagation = match poll_dns_propagation(&record_name, &value, timeout, interval) {
            Ok(result) => result,
            Err(err) => {
                let propagation_ms = propagation_start.elapsed().as_millis() as u64;
                let cleanup_start = Instant::now();
                let cleanup_result = provider_adapter.cleanup_txt(&record_name);
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
        info!(
            "[dns-test] Propagation check completed in {}ms: state={:?}",
            propagation_ms, propagation.state
        );
        
        // Spawn cleanup in background thread - don't block on it
        let record_name_clone = record_name.clone();
        let provider_adapter_clone = adapter_for_provider(&provider, &secrets);
        info!("[dns-test] Starting background cleanup for {}", record_name);
        std::thread::spawn(move || {
            let cleanup_start = Instant::now();
            if let Err(err) = provider_adapter_clone.cleanup_txt(&record_name_clone) {
                warn!("[dns-test] Background cleanup failed for {}: {}", record_name_clone, err);
            } else {
                info!(
                    "[dns-test] Background cleanup completed for {} in {}ms",
                    record_name_clone,
                    cleanup_start.elapsed().as_millis()
                );
            }
        });

        let success = matches!(propagation.state, PropagationState::Found);
        info!(
            "[dns-test] Test completed: success={}, total_elapsed={}ms, create={}ms, propagation={}ms, cleanup=async",
            success, started.elapsed().as_millis(), create_ms, propagation_ms
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
            cleanup_ms: None, // Cleanup is async, no timing available
        })
    })
    .await
    .map_err(|err| format!("DNS provider test join error: {err}"))?
    .map_err(|err: anyhow::Error| err.to_string())
}
