use anyhow::{anyhow, Context, Result};
use reqwest::blocking::Client;
use serde::Deserialize;
use std::{env, thread, time::Duration};

#[derive(Clone)]
pub struct CloudflareTestConfig {
    pub token: String,
    pub zone: String,
}

#[derive(Clone)]
pub struct DigitalOceanTestConfig {
    pub token: String,
    pub domain: String,
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

pub fn load_digitalocean_config() -> Result<DigitalOceanTestConfig> {
    let token = env::var("DNS_TEST_DIGITALOCEAN_TOKEN")
        .context("DNS_TEST_DIGITALOCEAN_TOKEN not set")?;
    let domain = env::var("DNS_TEST_DIGITALOCEAN_DOMAIN")
        .context("DNS_TEST_DIGITALOCEAN_DOMAIN not set")?;
    Ok(DigitalOceanTestConfig { token, domain })
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

#[derive(Deserialize, Clone)]
pub struct DigitalOceanDnsRecord {
    pub id: u64,
    pub name: String,
    #[serde(default)]
    pub data: Option<String>,
}

#[derive(Deserialize)]
struct DigitalOceanDnsRecordListResponse {
    domain_records: Vec<DigitalOceanDnsRecord>,
}

pub struct DigitalOceanCleanupRecord {
    config: DigitalOceanTestConfig,
    record_name: String,
}

impl Drop for DigitalOceanCleanupRecord {
    fn drop(&mut self) {
        let _ = delete_digitalocean_txt_records(&self.config, &self.record_name);
    }
}

pub fn digitalocean_relative_name(domain: &str, record_name: &str) -> String {
    let record_name = record_name.trim_end_matches('.');
    let domain = domain.trim_end_matches('.');

    if record_name == domain {
        "@".to_string()
    } else if record_name.ends_with(&format!(".{}", domain)) {
        record_name
            .strip_suffix(&format!(".{}", domain))
            .unwrap_or(record_name)
            .to_string()
    } else {
        record_name.to_string()
    }
}

pub fn ensure_digitalocean_record_cleanup(
    config: DigitalOceanTestConfig,
    record_name: &str,
) -> Result<DigitalOceanCleanupRecord> {
    delete_digitalocean_txt_records(&config, record_name)?;
    Ok(DigitalOceanCleanupRecord {
        config,
        record_name: record_name.to_string(),
    })
}

pub fn list_digitalocean_txt_records(
    config: &DigitalOceanTestConfig,
    record_name: &str,
) -> Result<Vec<DigitalOceanDnsRecord>> {
    let relative_name = digitalocean_relative_name(&config.domain, record_name);
    list_digitalocean_txt_records_best_effort(config, &relative_name)
}

pub fn list_digitalocean_txt_records_raw(
    config: &DigitalOceanTestConfig,
    query_name: &str,
) -> Result<Vec<DigitalOceanDnsRecord>> {
    let client = Client::new();
    let response = client
        .get(&format!(
            "https://api.digitalocean.com/v2/domains/{}/records",
            config.domain
        ))
        .query(&[("type", "TXT"), ("name", query_name)])
        .header("Authorization", format!("Bearer {}", config.token))
        .send()
        .context("Failed to list DigitalOcean DNS records")?;

    if !response.status().is_success() {
        return Err(anyhow!(
            "Failed to list DigitalOcean DNS records: {}",
            response.status()
        ));
    }

    let list_result: DigitalOceanDnsRecordListResponse = response
        .json()
        .context("Failed to parse DigitalOcean DNS record list")?;

    Ok(list_result.domain_records)
}

pub fn list_digitalocean_txt_records_best_effort(
    config: &DigitalOceanTestConfig,
    query_name: &str,
) -> Result<Vec<DigitalOceanDnsRecord>> {
    let records = list_digitalocean_txt_records_raw(config, query_name)?;
    if !records.is_empty() {
        return Ok(records);
    }

    let all_records = list_digitalocean_txt_records_all(config)?;
    Ok(all_records
        .into_iter()
        .filter(|record| record.name == query_name)
        .collect())
}

pub fn list_digitalocean_txt_records_all(
    config: &DigitalOceanTestConfig,
) -> Result<Vec<DigitalOceanDnsRecord>> {
    let client = Client::new();
    let response = client
        .get(&format!(
            "https://api.digitalocean.com/v2/domains/{}/records",
            config.domain
        ))
        .query(&[("type", "TXT")])
        .header("Authorization", format!("Bearer {}", config.token))
        .send()
        .context("Failed to list DigitalOcean DNS records")?;

    if !response.status().is_success() {
        return Err(anyhow!(
            "Failed to list DigitalOcean DNS records: {}",
            response.status()
        ));
    }

    let list_result: DigitalOceanDnsRecordListResponse = response
        .json()
        .context("Failed to parse DigitalOcean DNS record list")?;

    Ok(list_result.domain_records)
}

pub fn wait_for_digitalocean_record_data(
    config: &DigitalOceanTestConfig,
    record_name: &str,
    expected: &str,
) -> Result<DigitalOceanDnsRecord> {
    for _ in 0..10 {
        let records = list_digitalocean_txt_records(config, record_name)?;
        if let Some(record) = records
            .into_iter()
            .find(|record| record.data.as_deref() == Some(expected))
        {
            return Ok(record);
        }
        thread::sleep(Duration::from_millis(700));
    }
    Err(anyhow!(
        "DigitalOcean record content did not match expected value"
    ))
}

pub fn wait_for_digitalocean_record(
    config: &DigitalOceanTestConfig,
    record_name: &str,
) -> Result<DigitalOceanDnsRecord> {
    let expected_name = digitalocean_relative_name(&config.domain, record_name);
    for _ in 0..10 {
        let records = list_digitalocean_txt_records_all(config)?;
        if let Some(record) = records
            .into_iter()
            .find(|record| record.name == expected_name)
        {
            return Ok(record);
        }
        thread::sleep(Duration::from_millis(700));
    }
    Err(anyhow!("DigitalOcean record was not listed by name"))
}

pub fn delete_digitalocean_txt_records(
    config: &DigitalOceanTestConfig,
    record_name: &str,
) -> Result<()> {
    let client = Client::new();
    let relative_name = digitalocean_relative_name(&config.domain, record_name);
    let mut records = list_digitalocean_txt_records_raw(config, &relative_name)?;
    if records.is_empty() {
        records = list_digitalocean_txt_records_all(config)?
            .into_iter()
            .filter(|record| record.name == relative_name)
            .collect();
    }

    for record in records {
        let response = client
            .delete(&format!(
                "https://api.digitalocean.com/v2/domains/{}/records/{}",
                config.domain, record.id
            ))
            .header("Authorization", format!("Bearer {}", config.token))
            .send()
            .context("Failed to delete DigitalOcean DNS record")?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Failed to delete DigitalOcean DNS record: {}",
                response.status()
            ));
        }
    }

    Ok(())
}
