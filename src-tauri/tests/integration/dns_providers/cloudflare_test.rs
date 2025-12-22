use anyhow::{anyhow, Result};

use sslboard_desktop_lib::issuance::dns_providers::{CloudflareAdapter, DnsProviderAdapter};

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
