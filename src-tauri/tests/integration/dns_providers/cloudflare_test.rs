use anyhow::{anyhow, Result};

use sslboard_desktop_lib::issuance::dns_providers::{
    AtomicDnsOperations, CloudflareAdapter, DnsProviderAdapter,
};

use super::test_utils::{
    ensure_record_cleanup, expected_txt_content, list_txt_records, load_cloudflare_config,
    record_name, wait_for_record_content,
};

#[test]
fn cloudflare_record_name_format_fqdn() -> Result<()> {
    let config = load_cloudflare_config()?;
    let name = record_name(&config.zone, "format");
    let _cleanup = ensure_record_cleanup(config.clone(), &name)?;

    let adapter = CloudflareAdapter::new(config.token.clone(), config.zone.clone());
    adapter.create_txt(&name, "integration-test-format")?;

    let records = list_txt_records(&config, &name)?;
    let record = records
        .first()
        .ok_or_else(|| anyhow!("Expected Cloudflare TXT record to exist"))?;
    if record.name != name {
        return Err(anyhow!(
            "Expected record name {}, got {}",
            name,
            record.name
        ));
    }

    Ok(())
}

#[test]
fn cloudflare_txt_value_formatting() -> Result<()> {
    let config = load_cloudflare_config()?;
    let name = record_name(&config.zone, "txt");
    let _cleanup = ensure_record_cleanup(config.clone(), &name)?;

    let value = "integration-test-txt";
    let adapter = CloudflareAdapter::new(config.token.clone(), config.zone.clone());
    adapter.create_txt(&name, value)?;

    let expected = expected_txt_content(value);
    let record = wait_for_record_content(&config, &name, &expected)?;
    if record.content.as_deref() != Some(expected.as_str()) {
        return Err(anyhow!(
            "Expected Cloudflare TXT content {}, got {:?}",
            expected,
            record.content
        ));
    }

    Ok(())
}

#[test]
fn cloudflare_upsert_behavior() -> Result<()> {
    let config = load_cloudflare_config()?;
    let name = record_name(&config.zone, "upsert");
    let _cleanup = ensure_record_cleanup(config.clone(), &name)?;

    let adapter = CloudflareAdapter::new(config.token.clone(), config.zone.clone());
    adapter.create_txt(&name, "integration-test-upsert-1")?;
    adapter.create_txt(&name, "integration-test-upsert-2")?;

    let expected = expected_txt_content("integration-test-upsert-2");
    let record = wait_for_record_content(&config, &name, &expected)?;
    if record.content.as_deref() != Some(expected.as_str()) {
        return Err(anyhow!(
            "Expected Cloudflare TXT content {}, got {:?}",
            expected,
            record.content
        ));
    }

    Ok(())
}

#[test]
fn cloudflare_verification_logic() -> Result<()> {
    let config = load_cloudflare_config()?;
    let name = record_name(&config.zone, "verify");
    let _cleanup = ensure_record_cleanup(config.clone(), &name)?;

    let value = "integration-test-verify";
    let adapter = CloudflareAdapter::new(config.token.clone(), config.zone.clone());
    adapter.create_txt(&name, value)?;

    let expected = expected_txt_content(value);
    let record = wait_for_record_content(&config, &name, &expected)?;
    if record.content.as_deref() != Some(expected.as_str()) {
        return Err(anyhow!(
            "Expected Cloudflare TXT content {}, got {:?}",
            expected,
            record.content
        ));
    }

    Ok(())
}

#[test]
fn cloudflare_nested_subdomain_handling() -> Result<()> {
    let config = load_cloudflare_config()?;
    let name = record_name(&config.zone, "www");
    let _cleanup = ensure_record_cleanup(config.clone(), &name)?;

    let adapter = CloudflareAdapter::new(config.token.clone(), config.zone.clone());
    adapter.create_txt(&name, "integration-test-nested")?;

    let records = list_txt_records(&config, &name)?;
    if records.is_empty() {
        return Err(anyhow!("Expected nested Cloudflare TXT record to exist"));
    }

    Ok(())
}

#[test]
fn cloudflare_error_invalid_token() -> Result<()> {
    let config = load_cloudflare_config()?;
    let name = record_name(&config.zone, "invalid-token");

    let adapter = CloudflareAdapter::new("invalid-token".to_string(), config.zone);
    let err = adapter
        .create_txt(&name, "integration-test-error")
        .expect_err("expected invalid token error");
    let message = err.to_string();
    let message_lower = message.to_lowercase();
    if !message_lower.contains("authentication")
        && !message_lower.contains("401")
        && !message_lower.contains("403")
        && !message_lower.contains("400 bad request")
    {
        return Err(anyhow!("Unexpected error for invalid token: {}", message));
    }

    Ok(())
}

#[test]
fn cloudflare_error_zone_not_found() -> Result<()> {
    let config = load_cloudflare_config()?;
    let name = record_name("invalid-example.test", "zone-not-found");

    let adapter = CloudflareAdapter::new(config.token, "invalid-example.test".to_string());
    let err = adapter
        .create_txt(&name, "integration-test-error")
        .expect_err("expected zone not found error");
    let message = err.to_string();
    if !message.to_lowercase().contains("no cloudflare zone found") {
        return Err(anyhow!("Unexpected error for missing zone: {}", message));
    }

    Ok(())
}

#[test]
#[ignore = "manual rate limit trigger required"]
fn cloudflare_error_rate_limit() -> Result<()> {
    let config = load_cloudflare_config()?;
    let name = record_name(&config.zone, "rate-limit");
    let _cleanup = ensure_record_cleanup(config.clone(), &name)?;

    let adapter = CloudflareAdapter::new(config.token, config.zone);
    let err = adapter
        .create_txt(&name, "integration-test-rate-limit")
        .expect_err("expected rate limit error");
    let message = err.to_string();
    if !message.to_lowercase().contains("rate limit") {
        return Err(anyhow!("Unexpected error for rate limit: {}", message));
    }

    Ok(())
}

// Atomic operations tests

#[test]
fn cloudflare_atomic_create_one_record() -> Result<()> {
    let config = load_cloudflare_config()?;
    let name = record_name(&config.zone, "atomic-create");
    let _cleanup = ensure_record_cleanup(config.clone(), &name)?;

    let mut adapter = CloudflareAdapter::new(config.token.clone(), config.zone.clone());
    let record_id = adapter.create_one_record(&name, "integration-test-atomic-create")?;

    // Verify record was created
    let records = list_txt_records(&config, &name)?;
    let record = records
        .iter()
        .find(|r| r.id == record_id)
        .ok_or_else(|| anyhow!("Created record not found"))?;

    let expected = expected_txt_content("integration-test-atomic-create");
    if record.content.as_deref() != Some(expected.as_str()) {
        return Err(anyhow!(
            "Expected TXT content {}, got {:?}",
            expected,
            record.content
        ));
    }

    Ok(())
}

#[test]
fn cloudflare_atomic_delete_one_record() -> Result<()> {
    let config = load_cloudflare_config()?;
    let name = record_name(&config.zone, "atomic-delete");
    let _cleanup = ensure_record_cleanup(config.clone(), &name)?;

    // Create a record first
    let mut adapter = CloudflareAdapter::new(config.token.clone(), config.zone.clone());
    let record_id = adapter.create_one_record(&name, "integration-test-atomic-delete")?;

    // Wait for record to be visible
    let _record = wait_for_record_content(
        &config,
        &name,
        &expected_txt_content("integration-test-atomic-delete"),
    )?;

    // Delete it using atomic operation
    adapter.delete_one_record(&record_id)?;

    // Verify it's gone
    std::thread::sleep(std::time::Duration::from_millis(500));
    let records = list_txt_records(&config, &name)?;
    if records.iter().any(|r| r.id == record_id) {
        return Err(anyhow!("Record was not deleted"));
    }

    Ok(())
}

#[test]
fn cloudflare_atomic_list_records() -> Result<()> {
    let config = load_cloudflare_config()?;
    let name = record_name(&config.zone, "atomic-list");
    let _cleanup = ensure_record_cleanup(config.clone(), &name)?;

    // Create a record first
    let mut adapter = CloudflareAdapter::new(config.token.clone(), config.zone.clone());
    let record_id = adapter.create_one_record(&name, "integration-test-atomic-list")?;

    // Wait for record to be visible
    let _record = wait_for_record_content(
        &config,
        &name,
        &expected_txt_content("integration-test-atomic-list"),
    )?;

    // List records using atomic operation
    let records = adapter.list_records(&name)?;
    let found = records.iter().any(|r| r.id == record_id);
    if !found {
        return Err(anyhow!("Created record not found in list"));
    }

    Ok(())
}

#[test]
fn cloudflare_atomic_get_zone_id() -> Result<()> {
    let config = load_cloudflare_config()?;
    let mut adapter = CloudflareAdapter::new(config.token.clone(), config.zone.clone());

    // Get zone ID using atomic operation
    let zone_id = adapter.get_zone_id(&config.zone)?;

    // Verify it's a valid zone ID (Cloudflare zone IDs are alphanumeric)
    if zone_id.is_empty() {
        return Err(anyhow!("Zone ID is empty"));
    }

    // Verify it's cached (second call should use cache)
    let zone_id2 = adapter.get_zone_id(&config.zone)?;
    if zone_id != zone_id2 {
        return Err(anyhow!("Zone ID changed between calls (cache not working)"));
    }

    Ok(())
}

#[test]
fn cloudflare_atomic_normalize_value() -> Result<()> {
    // This test doesn't require API access, just test the normalization logic
    // Create adapter with dummy values since we only need the normalize_value method
    let adapter = CloudflareAdapter::new("dummy-token".to_string(), "example.com".to_string());

    // Test normalization removes quotes
    assert_eq!(adapter.normalize_value("\"test\""), "test");
    assert_eq!(adapter.normalize_value(" \"test\" "), "test");
    assert_eq!(adapter.normalize_value("test"), "test");

    Ok(())
}
