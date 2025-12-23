use std::time::{Duration, Instant};

use tauri::{async_runtime::spawn_blocking, State};
use uuid::Uuid;

use crate::core::types::{DnsProviderTestResult, PropagationDto, TestDnsProviderRequest};
use crate::issuance::dns::{check_txt_record, PropagationState};
use crate::issuance::dns_providers::adapter_for_provider;
use crate::secrets::manager::SecretManager;
use crate::storage::dns::DnsConfigStore;

use super::dns_provider_validation::categorize_dns_error;

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
        let provider = store
            .get_provider(&test_req.provider_id)?
            .ok_or_else(|| anyhow::anyhow!("provider not found: {}", test_req.provider_id))?;

        let suffix = provider
            .domain_suffixes
            .get(0)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("provider has no domain suffixes"))?;
        let random = Uuid::new_v4().as_simple().to_string();
        let record_name = format!("_sslboard-test-{}.{}", &random[..10], suffix);
        let value = format!("sslboard-test-{}", &random[..10]);

        let provider_adapter = adapter_for_provider(&provider, &secrets);

        let create_start = Instant::now();
        if let Err(err) = provider_adapter.create_txt(&record_name, &value) {
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
        let cleanup_start = Instant::now();
        let cleanup_result = provider_adapter.cleanup_txt(&record_name);
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

        let success = matches!(propagation.state, PropagationState::Found);

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

fn poll_txt_propagation(record_name: &str, value: &str) -> Result<PropagationDto, anyhow::Error> {
    let timeout = Duration::from_secs(30);
    let interval = Duration::from_secs(2);
    let started = Instant::now();
    let mut last = check_txt_record(record_name, value)?;
    loop {
        if matches!(last.state, PropagationState::Found) {
            return Ok(last);
        }
        if started.elapsed() >= timeout {
            return Ok(last);
        }
        std::thread::sleep(interval);
        last = check_txt_record(record_name, value)?;
    }
}
