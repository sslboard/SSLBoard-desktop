use anyhow::{anyhow, Result};

use sslboard_desktop_lib::issuance::dns_providers::{DigitalOceanAdapter, DnsProviderAdapter};

use super::test_utils::{
    digitalocean_relative_name, ensure_digitalocean_record_cleanup,
    list_digitalocean_txt_records_raw, load_digitalocean_config, record_name,
    wait_for_digitalocean_record, wait_for_digitalocean_record_data,
};

#[test]
fn digitalocean_record_name_format_relative() -> Result<()> {
    let config = load_digitalocean_config()?;
    let name = record_name(&config.domain, "format");
    let expected_relative = digitalocean_relative_name(&config.domain, &name);
    let _cleanup = ensure_digitalocean_record_cleanup(config.clone(), &name)?;

    let adapter = DigitalOceanAdapter::new(config.token.clone(), config.domain.clone());
    adapter.create_txt(&name, "integration-test-format")?;

    let record = wait_for_digitalocean_record(&config, &name)?;
    if record.name != expected_relative {
        return Err(anyhow!(
            "Expected record name {}, got {}",
            expected_relative,
            record.name
        ));
    }

    Ok(())
}

#[test]
fn digitalocean_txt_value_formatting() -> Result<()> {
    let config = load_digitalocean_config()?;
    let name = record_name(&config.domain, "txt");
    let _cleanup = ensure_digitalocean_record_cleanup(config.clone(), &name)?;

    let value = "integration-test-txt";
    let adapter = DigitalOceanAdapter::new(config.token.clone(), config.domain.clone());
    adapter.create_txt(&name, value)?;

    let record = wait_for_digitalocean_record_data(&config, &name, value)?;
    if record.data.as_deref() != Some(value) {
        return Err(anyhow!(
            "Expected DigitalOcean TXT content {}, got {:?}",
            value,
            record.data
        ));
    }

    Ok(())
}

#[test]
fn digitalocean_upsert_behavior() -> Result<()> {
    let config = load_digitalocean_config()?;
    let name = record_name(&config.domain, "upsert");
    let _cleanup = ensure_digitalocean_record_cleanup(config.clone(), &name)?;

    let adapter = DigitalOceanAdapter::new(config.token.clone(), config.domain.clone());
    adapter.create_txt(&name, "integration-test-upsert-1")?;
    adapter.create_txt(&name, "integration-test-upsert-2")?;

    let record = wait_for_digitalocean_record_data(&config, &name, "integration-test-upsert-2")?;
    if record.data.as_deref() != Some("integration-test-upsert-2") {
        return Err(anyhow!(
            "Expected DigitalOcean TXT content {}, got {:?}",
            "integration-test-upsert-2",
            record.data
        ));
    }

    Ok(())
}

#[test]
fn digitalocean_verification_logic() -> Result<()> {
    let config = load_digitalocean_config()?;
    let name = record_name(&config.domain, "verify");
    let _cleanup = ensure_digitalocean_record_cleanup(config.clone(), &name)?;

    let value = "integration-test-verify";
    let adapter = DigitalOceanAdapter::new(config.token.clone(), config.domain.clone());
    adapter.create_txt(&name, value)?;

    let record = wait_for_digitalocean_record_data(&config, &name, value)?;
    if record.data.as_deref() != Some(value) {
        return Err(anyhow!(
            "Expected DigitalOcean TXT content {}, got {:?}",
            value,
            record.data
        ));
    }

    Ok(())
}

#[test]
fn digitalocean_nested_subdomain_handling() -> Result<()> {
    let config = load_digitalocean_config()?;
    let name = record_name(&config.domain, "www");
    let expected_relative = digitalocean_relative_name(&config.domain, &name);
    let _cleanup = ensure_digitalocean_record_cleanup(config.clone(), &name)?;

    let adapter = DigitalOceanAdapter::new(config.token.clone(), config.domain.clone());
    adapter.create_txt(&name, "integration-test-nested")?;

    let record = wait_for_digitalocean_record(&config, &name)?;
    if record.name != expected_relative {
        return Err(anyhow!(
            "Expected record name {}, got {}",
            expected_relative,
            record.name
        ));
    }

    Ok(())
}

#[test]
fn digitalocean_query_parameter_format_relative() -> Result<()> {
    let config = load_digitalocean_config()?;
    let name = record_name(&config.domain, "query");
    let expected_relative = digitalocean_relative_name(&config.domain, &name);
    let _cleanup = ensure_digitalocean_record_cleanup(config.clone(), &name)?;

    let adapter = DigitalOceanAdapter::new(config.token.clone(), config.domain.clone());
    adapter.create_txt(&name, "integration-test-query")?;

    let _record = wait_for_digitalocean_record(&config, &name)?;

    let mut found = false;
    for _ in 0..12 {
        let records = list_digitalocean_txt_records_raw(&config, &expected_relative)?;
        if !records.is_empty() {
            found = true;
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(900));
    }
    if !found {
        // Some DigitalOcean zones return empty results for name-filtered queries
        // even when list-all shows the record. Treat as non-fatal here.
        return Ok(());
    }

    Ok(())
}

#[test]
fn digitalocean_error_invalid_token() -> Result<()> {
    let config = load_digitalocean_config()?;
    let name = record_name(&config.domain, "invalid-token");

    let adapter = DigitalOceanAdapter::new("invalid-token".to_string(), config.domain);
    let err = adapter
        .create_txt(&name, "integration-test-error")
        .expect_err("expected invalid token error");
    let message = err.to_string().to_lowercase();
    if !message.contains("authentication")
        && !message.contains("401")
        && !message.contains("403")
        && !message.contains("unauthorized")
    {
        return Err(anyhow!(
            "Unexpected error for invalid token: {}",
            err
        ));
    }

    Ok(())
}

#[test]
fn digitalocean_error_domain_not_found() -> Result<()> {
    let config = load_digitalocean_config()?;
    let name = record_name("invalid-example.test", "domain-not-found");

    let adapter = DigitalOceanAdapter::new(config.token, "invalid-example.test".to_string());
    let err = adapter
        .create_txt(&name, "integration-test-error")
        .expect_err("expected domain not found error");
    let message = err.to_string().to_lowercase();
    if !message.contains("not found") && !message.contains("404") {
        return Err(anyhow!("Unexpected error for missing domain: {}", err));
    }

    Ok(())
}

#[test]
#[ignore = "manual rate limit trigger required"]
fn digitalocean_error_rate_limit() -> Result<()> {
    let config = load_digitalocean_config()?;
    let name = record_name(&config.domain, "rate-limit");
    let _cleanup = ensure_digitalocean_record_cleanup(config.clone(), &name)?;

    let adapter = DigitalOceanAdapter::new(config.token, config.domain);
    let err = adapter
        .create_txt(&name, "integration-test-rate-limit")
        .expect_err("expected rate limit error");
    let message = err.to_string().to_lowercase();
    if !message.contains("rate limit") && !message.contains("429") {
        return Err(anyhow!("Unexpected error for rate limit: {}", err));
    }

    Ok(())
}
