use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::time::Duration;

use super::{http, matches_zone, DnsProviderAdapter};

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

    fn create_txt_record(&mut self, record_name: &str, value: &str) -> Result<String> {
        let zone_id = self.discover_zone_id()?;
        let client = http::HttpClient::shared();
        let formatted_value = Self::format_txt_content(value);

        let record = CloudflareDnsRecord {
            record_type: "TXT".to_string(),
            name: record_name.to_string(),
            content: formatted_value.clone(),
            ttl: 120, // Auto TTL
        };

        let response = client
            .post(&format!(
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
            if let Ok(parsed) = serde_json::from_str::<CloudflareDnsRecordResponse>(&error_text) {
                if parsed
                    .errors
                    .as_ref()
                    .map(|errs| errs.iter().any(|err| err.code == 81058))
                    .unwrap_or(false)
                {
                    return self.update_existing_txt_record(&client, &zone_id, record_name, value);
                }
            }
            return Err(http::status_error(
                "Cloudflare",
                status,
                Some(error_text.clone()),
            ));
        }

        let result: CloudflareDnsRecordResponse = response
            .json()
            .context("Failed to parse Cloudflare DNS record response")?;

        if !result.success {
            if result
                .errors
                .as_ref()
                .map(|errs| errs.iter().any(|err| err.code == 81058))
                .unwrap_or(false)
            {
                return self.update_existing_txt_record(&client, &zone_id, record_name, value);
            }
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
            .and_then(|r| Some(r.id))
            .ok_or_else(|| anyhow!("Cloudflare API did not return record ID"))?;

        self.verify_record_content(&client, &zone_id, &record_id, &formatted_value)?;
        Ok(record_id)
    }

    fn update_existing_txt_record(
        &self,
        client: &reqwest::blocking::Client,
        zone_id: &str,
        record_name: &str,
        value: &str,
    ) -> Result<String> {
        let response = client
            .get(&format!(
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

        let formatted_value = Self::format_txt_content(value);
        let records = list_result.result;
        if records.is_empty() {
            return Err(anyhow!("TXT record not found: {}", record_name));
        }

        let mut updated_ids = Vec::new();
        for record in records {
            let record_id = record.id.clone();
            let record = CloudflareDnsRecord {
                record_type: "TXT".to_string(),
                name: record_name.to_string(),
                content: formatted_value.clone(),
                ttl: 120,
            };

            let update_response = client
                .put(&format!(
                    "https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}",
                    zone_id, record_id
                ))
                .header("Authorization", format!("Bearer {}", self.api_token))
                .header("Content-Type", "application/json")
                .json(&record)
                .send()
                .context("Failed to update Cloudflare DNS record")?;

            if !update_response.status().is_success() {
                let status = update_response.status();
                let error_text = update_response.text().unwrap_or_default();
                return Err(http::status_error("Cloudflare", status, Some(error_text)));
            }

            let result: CloudflareDnsRecordResponse = update_response
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

            let updated_id = result
                .result
                .and_then(|r| Some(r.id))
                .ok_or_else(|| anyhow!("Cloudflare API did not return record ID"))?;
            updated_ids.push(updated_id);
        }

        let first_id = updated_ids
            .first()
            .cloned()
            .ok_or_else(|| anyhow!("Cloudflare API did not return record ID"))?;
        self.verify_record_content(client, zone_id, &first_id, &formatted_value)?;

        Ok(first_id)
    }

    fn delete_txt_record(&mut self, record_name: &str) -> Result<()> {
        let zone_id = self.discover_zone_id()?;
        let client = http::HttpClient::shared();

        // First, find the record ID
        let response = client
            .get(&format!(
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

        let record_id = list_result
            .result
            .first()
            .ok_or_else(|| anyhow!("TXT record not found: {}", record_name))?
            .id
            .clone();

        // Delete the record
        let delete_response = client
            .delete(&format!(
                "https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}",
                zone_id, record_id
            ))
            .header("Authorization", format!("Bearer {}", self.api_token))
            .send()
            .context("Failed to delete Cloudflare DNS record")?;

        if !delete_response.status().is_success() {
            return Err(http::status_error(
                "Cloudflare",
                delete_response.status(),
                None,
            ));
        }

        Ok(())
    }

    fn verify_record_content(
        &self,
        client: &reqwest::blocking::Client,
        zone_id: &str,
        record_id: &str,
        expected_content: &str,
    ) -> Result<()> {
        for _ in 0..3 {
            let check_response = client
                .get(&format!(
                    "https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}",
                    zone_id, record_id
                ))
                .header("Authorization", format!("Bearer {}", self.api_token))
                .header("Content-Type", "application/json")
                .send()
                .context("Failed to fetch Cloudflare DNS record")?;

            if check_response.status().is_success() {
                let check_result: CloudflareDnsRecordResponse = check_response
                    .json()
                    .context("Failed to parse Cloudflare DNS record response")?;
                if let Some(record) = check_result.result {
                    if record
                        .content
                        .as_deref()
                        .map(|content| content == expected_content)
                        .unwrap_or(false)
                    {
                        return Ok(());
                    }
                }
            }
            std::thread::sleep(Duration::from_millis(300));
        }
        Err(anyhow!("Cloudflare record verification failed"))
    }
}

impl DnsProviderAdapter for CloudflareAdapter {
    fn create_txt(&self, record_name: &str, value: &str) -> Result<()> {
        // Need mutable access, so clone and create new adapter
        let mut adapter =
            CloudflareAdapter::new(self.api_token.clone(), self.domain_suffix.clone());
        adapter.create_txt_record(record_name, value)?;
        Ok(())
    }

    fn cleanup_txt(&self, record_name: &str) -> Result<()> {
        let mut adapter =
            CloudflareAdapter::new(self.api_token.clone(), self.domain_suffix.clone());
        adapter.delete_txt_record(record_name)?;
        Ok(())
    }
}
