use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::time::Duration;

use super::{http, DnsProviderAdapter};

pub struct DigitalOceanAdapter {
    api_token: String,
    domain: String,
}

#[derive(Serialize)]
struct DigitalOceanDnsRecord {
    #[serde(rename = "type")]
    record_type: String,
    name: String,
    data: String,
    ttl: u32,
}

#[derive(Deserialize)]
struct DigitalOceanDnsRecordResponse {
    domain_record: DigitalOceanDnsRecordResult,
}

#[derive(Deserialize)]
struct DigitalOceanDnsRecordResult {
    id: u64,
}

#[derive(Deserialize)]
struct DigitalOceanDnsRecordDetail {
    #[serde(default)]
    data: Option<String>,
}

#[derive(Deserialize)]
struct DigitalOceanDnsRecordDetailResponse {
    domain_record: DigitalOceanDnsRecordDetail,
}

impl DigitalOceanAdapter {
    pub fn new(api_token: String, domain: String) -> Self {
        Self { api_token, domain }
    }

    /// Converts a full record name (FQDN) to a relative name for DigitalOcean API.
    /// Example: "_acme-challenge.example.com" with domain "example.com" -> "_acme-challenge"
    fn to_relative_name(&self, record_name: &str) -> String {
        let record_name = record_name.trim_end_matches('.');
        let domain = self.domain.trim_end_matches('.');

        if record_name == domain {
            // Root domain record
            "@".to_string()
        } else if record_name.ends_with(&format!(".{}", domain)) {
            // Subdomain record - extract the relative part
            let relative = record_name
                .strip_suffix(&format!(".{}", domain))
                .unwrap_or(record_name);
            relative.to_string()
        } else {
            // If it doesn't match the domain, return as-is (might be a bug upstream)
            record_name.to_string()
        }
    }

    /// Formats TXT content. DigitalOcean API handles quoting automatically,
    /// so we just trim whitespace and remove any existing quotes.
    fn format_txt_content(value: &str) -> String {
        value.trim().trim_matches('"').trim().to_string()
    }

    /// Normalizes TXT content for comparison (removes quotes and whitespace).
    fn normalize_txt_content(value: &str) -> String {
        value.trim().trim_matches('"').trim().to_string()
    }

    fn list_txt_records(&self, record_name: &str) -> Result<Vec<DigitalOceanDnsRecordListItem>> {
        let relative_name = self.to_relative_name(record_name);
        let client = http::HttpClient::shared();
        let response = client
            .get(&format!(
                "https://api.digitalocean.com/v2/domains/{}/records?type=TXT&name={}",
                self.domain, relative_name
            ))
            .header("Authorization", format!("Bearer {}", self.api_token))
            .send()
            .context("Failed to list DigitalOcean DNS records")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().unwrap_or_default();
            return Err(http::status_error("DigitalOcean", status, Some(body)));
        }

        let list_result: DigitalOceanDnsRecordListResponse = response
            .json()
            .context("Failed to parse DigitalOcean DNS record list")?;

        Ok(list_result.domain_records)
    }

    fn list_all_txt_records(&self) -> Result<Vec<DigitalOceanDnsRecordListItem>> {
        let client = http::HttpClient::shared();
        let response = client
            .get(&format!(
                "https://api.digitalocean.com/v2/domains/{}/records?type=TXT",
                self.domain
            ))
            .header("Authorization", format!("Bearer {}", self.api_token))
            .send()
            .context("Failed to list DigitalOcean DNS records")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().unwrap_or_default();
            return Err(http::status_error("DigitalOcean", status, Some(body)));
        }

        let list_result: DigitalOceanDnsRecordListResponse = response
            .json()
            .context("Failed to parse DigitalOcean DNS record list")?;

        Ok(list_result.domain_records)
    }

    fn create_txt_record(&self, record_name: &str, value: &str) -> Result<u64> {
        let client = http::HttpClient::shared();
        let relative_name = self.to_relative_name(record_name);

        let record = DigitalOceanDnsRecord {
            record_type: "TXT".to_string(),
            name: relative_name,
            data: Self::format_txt_content(value),
            ttl: 300,
        };

        let response = client
            .post(&format!(
                "https://api.digitalocean.com/v2/domains/{}/records",
                self.domain
            ))
            .header("Authorization", format!("Bearer {}", self.api_token))
            .header("Content-Type", "application/json")
            .json(&record)
            .send()
            .context("Failed to create DigitalOcean DNS record")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().unwrap_or_default();
            return Err(http::status_error("DigitalOcean", status, Some(error_text)));
        }

        let result: DigitalOceanDnsRecordResponse = response
            .json()
            .context("Failed to parse DigitalOcean DNS record response")?;

        Ok(result.domain_record.id)
    }

    fn update_txt_record(&self, record_id: u64, record_name: &str, value: &str) -> Result<()> {
        let client = http::HttpClient::shared();
        let relative_name = self.to_relative_name(record_name);
        let record = DigitalOceanDnsRecord {
            record_type: "TXT".to_string(),
            name: relative_name,
            data: Self::format_txt_content(value),
            ttl: 300,
        };
        let response = client
            .put(&format!(
                "https://api.digitalocean.com/v2/domains/{}/records/{}",
                self.domain, record_id
            ))
            .header("Authorization", format!("Bearer {}", self.api_token))
            .header("Content-Type", "application/json")
            .json(&record)
            .send()
            .context("Failed to update DigitalOcean DNS record")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().unwrap_or_default();
            return Err(http::status_error("DigitalOcean", status, Some(error_text)));
        }

        Ok(())
    }

    fn fetch_record_data(&self, record_id: u64) -> Result<Option<String>> {
        let client = http::HttpClient::shared();
        let response = client
            .get(&format!(
                "https://api.digitalocean.com/v2/domains/{}/records/{}",
                self.domain, record_id
            ))
            .header("Authorization", format!("Bearer {}", self.api_token))
            .send()
            .context("Failed to fetch DigitalOcean DNS record")?;

        if response.status() == 404 {
            return Ok(None);
        }
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().unwrap_or_default();
            return Err(http::status_error("DigitalOcean", status, Some(error_text)));
        }

        let record: DigitalOceanDnsRecordDetailResponse = response
            .json()
            .context("Failed to parse DigitalOcean DNS record response")?;
        Ok(record.domain_record.data)
    }

    fn upsert_txt_record(&self, record_name: &str, value: &str) -> Result<()> {
        let existing = self.list_txt_records(record_name)?;
        let mut record_ids = Vec::new();
        if existing.is_empty() {
            let record_id = self.create_txt_record(record_name, value)?;
            record_ids.push(record_id);
        } else {
            for record in &existing {
                self.update_txt_record(record.id, record_name, value)?;
                record_ids.push(record.id);
            }
        }
        let expected_normalized = Self::normalize_txt_content(value);
        let mut matched = false;
        for _ in 0..5 {
            for record_id in &record_ids {
                if let Some(data) = self.fetch_record_data(*record_id)? {
                    if Self::normalize_txt_content(&data) == expected_normalized {
                        matched = true;
                        break;
                    }
                }
            }
            if matched {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
        if !matched {
            return Err(anyhow!(
                "DigitalOcean record verification failed for {}",
                record_name
            ));
        }
        Ok(())
    }

    fn delete_txt_record(&self, record_name: &str) -> Result<()> {
        let client = http::HttpClient::shared();

        let relative_name = self.to_relative_name(record_name);
        let mut record_ids: Vec<u64> = Vec::new();
        for attempt in 0..4 {
            let list_result = self.list_txt_records(record_name)?;
            record_ids = list_result.iter().map(|record| record.id).collect();
            if !record_ids.is_empty() {
                break;
            }
            if attempt == 2 {
                let list_result = self.list_all_txt_records()?;
                record_ids = list_result
                    .iter()
                    .filter(|record| record.name == relative_name)
                    .map(|record| record.id)
                    .collect();
                if !record_ids.is_empty() {
                    break;
                }
            }
            std::thread::sleep(Duration::from_millis(400));
        }
        if record_ids.is_empty() {
            return Ok(());
        }

        for record_id in record_ids {
            let delete_response = client
                .delete(&format!(
                    "https://api.digitalocean.com/v2/domains/{}/records/{}",
                    self.domain, record_id
            ))
            .header("Authorization", format!("Bearer {}", self.api_token))
            .send()
            .context("Failed to delete DigitalOcean DNS record")?;

            if !delete_response.status().is_success() {
                if delete_response.status() == 404 {
                    continue;
                }
                return Err(http::status_error(
                    "DigitalOcean",
                    delete_response.status(),
                    None,
                ));
            }
        }

        Ok(())
    }
}

#[derive(Deserialize)]
struct DigitalOceanDnsRecordListResponse {
    domain_records: Vec<DigitalOceanDnsRecordListItem>,
}

#[derive(Deserialize)]
struct DigitalOceanDnsRecordListItem {
    id: u64,
    #[serde(default)]
    name: String,
}

impl DnsProviderAdapter for DigitalOceanAdapter {
    fn create_txt(&self, record_name: &str, value: &str) -> Result<()> {
        self.upsert_txt_record(record_name, value)?;
        Ok(())
    }

    fn cleanup_txt(&self, record_name: &str) -> Result<()> {
        self.delete_txt_record(record_name)?;
        Ok(())
    }
}
