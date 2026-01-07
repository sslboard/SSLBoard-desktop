use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};

use super::{
    DnsProviderAdapter,
    base::{AtomicDnsOperations, DnsProviderBase, DnsRecord},
    http, matches_zone,
};

pub struct CloudflareAdapter {
    api_token: String,
    zone_cache: Option<String>,
    domain_suffix: String,
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

#[derive(Serialize)]
struct CloudflareDnsRecord {
    #[serde(rename = "type")]
    record_type: String,
    name: String,
    content: String,
    ttl: u32,
}

#[derive(Deserialize)]
struct CloudflareDnsRecordResponse {
    result: Option<CloudflareDnsRecordResult>,
    success: bool,
    errors: Option<Vec<CloudflareError>>,
}

#[derive(Deserialize)]
struct CloudflareDnsRecordResult {
    id: String,
    #[serde(default)]
    content: Option<String>,
}

#[derive(Deserialize)]
struct CloudflareError {
    code: u32,
    message: String,
}

impl CloudflareAdapter {
    pub fn new(api_token: String, domain_suffix: String) -> Self {
        Self {
            api_token,
            zone_cache: None,
            domain_suffix,
        }
    }

    fn format_txt_content(value: &str) -> String {
        let trimmed = value.trim();
        if trimmed.starts_with('"') && trimmed.ends_with('"') {
            trimmed.to_string()
        } else {
            format!("\"{}\"", trimmed.trim_matches('"'))
        }
    }

    fn discover_zone_id(&mut self) -> Result<String> {
        if let Some(ref zone_id) = self.zone_cache {
            return Ok(zone_id.clone());
        }

        let client = http::HttpClient::shared();
        let response = client
            .get("https://api.cloudflare.com/client/v4/zones")
            .header("Authorization", format!("Bearer {}", self.api_token))
            .header("Content-Type", "application/json")
            .send()
            .context("Failed to list Cloudflare zones")?;

        if !response.status().is_success() {
            if response.status() == 401 || response.status() == 403 {
                return Err(anyhow!(
                    "Cloudflare authentication failed: invalid API token"
                ));
            }
            return Err(http::status_error("Cloudflare", response.status(), None));
        }

        let zone_list: CloudflareZoneListResponse = response
            .json()
            .context("Failed to parse Cloudflare zone list response")?;

        if !zone_list.success {
            return Err(anyhow!("Cloudflare API returned unsuccessful response"));
        }

        // Find zone matching domain suffix
        let zone = zone_list
            .result
            .iter()
            .find(|z| matches_zone(&self.domain_suffix, &z.name))
            .ok_or_else(|| {
                anyhow!(
                    "No Cloudflare zone found for domain suffix: {}",
                    self.domain_suffix
                )
            })?;

        self.zone_cache = Some(zone.id.clone());
        Ok(zone.id.clone())
    }

    fn list_existing_txt_records(
        &self,
        client: &reqwest::blocking::Client,
        zone_id: &str,
        record_name: &str,
    ) -> Result<Vec<CloudflareDnsRecordResult>> {
        let response = client
            .get(format!(
                "https://api.cloudflare.com/client/v4/zones/{}/dns_records?type=TXT&name={}",
                zone_id, record_name
            ))
            .header("Authorization", format!("Bearer {}", self.api_token))
            .header("Content-Type", "application/json")
            .send()
            .context("Failed to list Cloudflare DNS records")?;

        if !response.status().is_success() {
            return Err(anyhow!("Failed to list Cloudflare DNS records"));
        }

        #[derive(Deserialize)]
        struct CloudflareDnsRecordListResponse {
            result: Vec<CloudflareDnsRecordResult>,
            success: bool,
        }

        let list_result: CloudflareDnsRecordListResponse = response
            .json()
            .context("Failed to parse Cloudflare DNS record list")?;

        if !list_result.success {
            return Err(anyhow!("Cloudflare API returned unsuccessful response"));
        }

        Ok(list_result.result)
    }

    /// Atomic operation: Creates a single TXT record via Cloudflare API.
    /// Returns the record ID. Does not check for existing records or verify.
    fn create_txt_record_atomic(&mut self, record_name: &str, value: &str) -> Result<String> {
        let zone_id = self.discover_zone_id()?;
        let client = http::HttpClient::shared();
        let formatted_value = Self::format_txt_content(value);

        let record = CloudflareDnsRecord {
            record_type: "TXT".to_string(),
            name: record_name.to_string(),
            content: formatted_value,
            ttl: 120, // Auto TTL
        };

        let response = client
            .post(format!(
                "https://api.cloudflare.com/client/v4/zones/{}/dns_records",
                zone_id
            ))
            .header("Authorization", format!("Bearer {}", self.api_token))
            .header("Content-Type", "application/json")
            .json(&record)
            .send()
            .context("Failed to create Cloudflare DNS record")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().unwrap_or_default();
            return Err(http::status_error("Cloudflare", status, Some(error_text)));
        }

        let result: CloudflareDnsRecordResponse = response
            .json()
            .context("Failed to parse Cloudflare DNS record response")?;

        if !result.success {
            let error_msg = result
                .errors
                .map(|e| {
                    e.iter()
                        .map(|err| format!("{}: {}", err.code, err.message))
                        .collect::<Vec<_>>()
                        .join(", ")
                })
                .unwrap_or_else(|| "Unknown error".to_string());
            return Err(anyhow!("Cloudflare API error: {}", error_msg));
        }

        let record_id = result
            .result
            .map(|r| r.id)
            .ok_or_else(|| anyhow!("Cloudflare API did not return record ID"))?;

        Ok(record_id)
    }

    /// Atomic operation: Deletes a single TXT record by ID via Cloudflare API.
    /// Does not handle parallelization or listing - just a single DELETE call.
    fn delete_txt_record_atomic(&mut self, record_id: &str) -> Result<()> {
        let zone_id = self.discover_zone_id()?;
        let client = http::HttpClient::shared();

        let delete_response = client
            .delete(format!(
                "https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}",
                zone_id, record_id
            ))
            .header("Authorization", format!("Bearer {}", self.api_token))
            .send()
            .context("Failed to delete Cloudflare DNS record")?;

        if delete_response.status().is_success() || delete_response.status() == 404 {
            // 404 is fine, record already gone
            Ok(())
        } else {
            Err(http::status_error(
                "Cloudflare",
                delete_response.status(),
                None,
            ))
        }
    }
}

impl AtomicDnsOperations for CloudflareAdapter {
    fn normalize_value(&self, value: &str) -> String {
        // Cloudflare stores values with quotes, normalize by removing quotes and trimming
        value.trim().trim_matches('"').trim().to_string()
    }

    fn create_one_record(&mut self, record_name: &str, value: &str) -> Result<String> {
        self.create_txt_record_atomic(record_name, value)
    }

    fn delete_one_record(&mut self, record_id: &str) -> Result<()> {
        self.delete_txt_record_atomic(record_id)
    }

    fn list_records(&mut self, record_name: &str) -> Result<Vec<DnsRecord>> {
        let zone_id = self.discover_zone_id()?;
        let client = http::HttpClient::shared();
        let existing_records = self.list_existing_txt_records(client, &zone_id, record_name)?;

        Ok(existing_records
            .into_iter()
            .map(|r| DnsRecord {
                id: r.id,
                name: record_name.to_string(),
                value: r.content.unwrap_or_default(),
            })
            .collect())
    }

    fn get_zone_id(&mut self, _domain: &str) -> Result<String> {
        // Cloudflare uses domain_suffix for zone discovery
        self.discover_zone_id()
    }
}

impl DnsProviderBase for CloudflareAdapter {
    fn atomic_ops(&mut self) -> &mut dyn AtomicDnsOperations {
        self
    }
}

impl DnsProviderAdapter for CloudflareAdapter {
    fn create_txt(&self, record_name: &str, value: &str) -> Result<()> {
        // Use DnsProviderBase for backward compatibility
        let mut adapter =
            CloudflareAdapter::new(self.api_token.clone(), self.domain_suffix.clone());
        adapter.set_txt_record(record_name, value)?;
        Ok(())
    }

    fn cleanup_txt(&self, record_name: &str) -> Result<()> {
        // Use DnsProviderBase for backward compatibility
        let mut adapter =
            CloudflareAdapter::new(self.api_token.clone(), self.domain_suffix.clone());
        adapter.delete_txt_record(record_name)?;
        Ok(())
    }
}
