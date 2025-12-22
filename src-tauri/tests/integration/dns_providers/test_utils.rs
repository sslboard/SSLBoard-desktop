use anyhow::{anyhow, Context, Result};
use reqwest::blocking::Client;
use serde::Deserialize;
use std::{env, thread, time::Duration};

#[derive(Clone)]
pub struct CloudflareTestConfig {
    pub token: String,
    pub zone: String,
}

#[derive(Deserialize)]
struct CloudflareZone {
    id: String,
    name: String,
}

#[derive(Deserialize)]
struct CloudflareZoneListResponse {
    result: Vec<CloudflareZone>,
    success: bool,
}

#[derive(Deserialize, Clone)]
pub struct CloudflareDnsRecord {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub content: Option<String>,
}

#[derive(Deserialize)]
struct CloudflareDnsRecordListResponse {
    result: Vec<CloudflareDnsRecord>,
    success: bool,
}

#[derive(Deserialize)]
struct CloudflareDeleteResponse {
    success: bool,
}

pub struct CleanupRecord {
    config: CloudflareTestConfig,
    record_name: String,
}

impl Drop for CleanupRecord {
    fn drop(&mut self) {
        let _ = delete_txt_records(&self.config, &self.record_name);
    }
}

pub fn load_cloudflare_config() -> Result<CloudflareTestConfig> {
    let token = env::var("DNS_TEST_CLOUDFLARE_TOKEN")
        .context("DNS_TEST_CLOUDFLARE_TOKEN not set")?;
    let zone =
        env::var("DNS_TEST_CLOUDFLARE_ZONE").context("DNS_TEST_CLOUDFLARE_ZONE not set")?;
    Ok(CloudflareTestConfig { token, zone })
}

pub fn record_name(zone: &str, label: &str) -> String {
    format!("_acme-challenge.integration-test.{}.{}", label, zone)
}

pub fn expected_txt_content(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.starts_with('"') && trimmed.ends_with('"') {
        trimmed.to_string()
    } else {
        format!("\"{}\"", trimmed.trim_matches('"'))
    }
}

pub fn ensure_record_cleanup(
    config: CloudflareTestConfig,
    record_name: &str,
) -> Result<CleanupRecord> {
    delete_txt_records(&config, record_name)?;
    Ok(CleanupRecord {
        config,
        record_name: record_name.to_string(),
    })
}

pub fn wait_for_record_content(
    config: &CloudflareTestConfig,
    record_name: &str,
    expected: &str,
) -> Result<CloudflareDnsRecord> {
    for _ in 0..5 {
        let records = list_txt_records(config, record_name)?;
        if let Some(record) = records
            .into_iter()
            .find(|record| record.content.as_deref() == Some(expected))
        {
            return Ok(record);
        }
        thread::sleep(Duration::from_millis(400));
    }
    Err(anyhow!(
        "Cloudflare record content did not match expected value"
    ))
}

pub fn list_txt_records(
    config: &CloudflareTestConfig,
    record_name: &str,
) -> Result<Vec<CloudflareDnsRecord>> {
    let client = Client::new();
    let zone_id = resolve_zone_id(config)?;
    let response = client
        .get(&format!(
            "https://api.cloudflare.com/client/v4/zones/{}/dns_records",
            zone_id
        ))
        .query(&[("type", "TXT"), ("name", record_name)])
        .header("Authorization", format!("Bearer {}", config.token))
        .header("Content-Type", "application/json")
        .send()
        .context("Failed to list Cloudflare DNS records")?;

    if !response.status().is_success() {
        return Err(anyhow!(
            "Failed to list Cloudflare DNS records: {}",
            response.status()
        ));
    }

    let list_result: CloudflareDnsRecordListResponse = response
        .json()
        .context("Failed to parse Cloudflare DNS record list")?;

    if !list_result.success {
        return Err(anyhow!("Cloudflare API returned unsuccessful response"));
    }

    Ok(list_result.result)
}

pub fn delete_txt_records(config: &CloudflareTestConfig, record_name: &str) -> Result<()> {
    let client = Client::new();
    let zone_id = resolve_zone_id(config)?;
    let records = list_txt_records(config, record_name)?;

    for record in records {
        let response = client
            .delete(&format!(
                "https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}",
                zone_id, record.id
            ))
            .header("Authorization", format!("Bearer {}", config.token))
            .send()
            .context("Failed to delete Cloudflare DNS record")?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Failed to delete Cloudflare DNS record: {}",
                response.status()
            ));
        }

        let delete_result: CloudflareDeleteResponse = response
            .json()
            .context("Failed to parse Cloudflare delete response")?;

        if !delete_result.success {
            return Err(anyhow!("Cloudflare API returned unsuccessful delete response"));
        }
    }

    Ok(())
}

fn resolve_zone_id(config: &CloudflareTestConfig) -> Result<String> {
    let client = Client::new();
    let response = client
        .get("https://api.cloudflare.com/client/v4/zones")
        .header("Authorization", format!("Bearer {}", config.token))
        .header("Content-Type", "application/json")
        .send()
        .context("Failed to list Cloudflare zones")?;

    if !response.status().is_success() {
        return Err(anyhow!(
            "Cloudflare zone list failed: {}",
            response.status()
        ));
    }

    let zone_list: CloudflareZoneListResponse = response
        .json()
        .context("Failed to parse Cloudflare zone list response")?;

    if !zone_list.success {
        return Err(anyhow!("Cloudflare API returned unsuccessful response"));
    }

    let zone = zone_list
        .result
        .iter()
        .find(|zone| matches_zone(&config.zone, &zone.name))
        .ok_or_else(|| anyhow!("No Cloudflare zone found for domain suffix: {}", config.zone))?;

    Ok(zone.id.clone())
}

fn matches_zone(domain_suffix: &str, zone_name: &str) -> bool {
    zone_name == domain_suffix || domain_suffix.ends_with(&format!(".{}", zone_name))
}
