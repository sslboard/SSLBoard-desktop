use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};

use super::{
    DnsProviderAdapter,
    base::{AtomicDnsOperations, DnsProviderBase, DnsRecord},
    http,
};

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
            .get(format!(
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

    fn fetch_record_data(&self, record_id: u64) -> Result<Option<String>> {
        let client = http::HttpClient::shared();
        let response = client
            .get(format!(
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

    /// Atomic operation: Creates a single TXT record via DigitalOcean API.
    /// Returns the record ID. Does not check for existing records or verify.
    fn create_txt_record_atomic(&self, record_name: &str, value: &str) -> Result<u64> {
        let client = http::HttpClient::shared();
        let relative_name = self.to_relative_name(record_name);

        let record = DigitalOceanDnsRecord {
            record_type: "TXT".to_string(),
            name: relative_name,
            data: Self::format_txt_content(value),
            ttl: 300,
        };

        let response = client
            .post(format!(
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

    /// Atomic operation: Deletes a single TXT record by ID via DigitalOcean API.
    /// Does not handle listing or retries - just a single DELETE call.
    fn delete_txt_record_atomic(&self, record_id: u64) -> Result<()> {
        let client = http::HttpClient::shared();
        let delete_response = client
            .delete(format!(
                "https://api.digitalocean.com/v2/domains/{}/records/{}",
                self.domain, record_id
            ))
            .header("Authorization", format!("Bearer {}", self.api_token))
            .send()
            .context("Failed to delete DigitalOcean DNS record")?;

        if delete_response.status().is_success() || delete_response.status() == 404 {
            // 404 is fine, record already gone
            Ok(())
        } else {
            Err(http::status_error(
                "DigitalOcean",
                delete_response.status(),
                None,
            ))
        }
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

impl AtomicDnsOperations for DigitalOceanAdapter {
    fn normalize_value(&self, value: &str) -> String {
        // DigitalOcean uses normalize_txt_content logic
        Self::normalize_txt_content(value)
    }

    fn create_one_record(&mut self, record_name: &str, value: &str) -> Result<String> {
        // Truly atomic: just create the record
        let record_id = self.create_txt_record_atomic(record_name, value)?;
        Ok(record_id.to_string())
    }

    fn delete_one_record(&mut self, record_id: &str) -> Result<()> {
        let record_id = record_id
            .parse::<u64>()
            .map_err(|_| anyhow!("Invalid record ID: {}", record_id))?;

        // Truly atomic: just delete the record
        self.delete_txt_record_atomic(record_id)
    }

    fn list_records(&mut self, record_name: &str) -> Result<Vec<DnsRecord>> {
        let existing = self.list_txt_records(record_name)?;
        let mut records = Vec::new();

        for item in existing {
            if let Ok(Some(data)) = self.fetch_record_data(item.id) {
                records.push(DnsRecord {
                    id: item.id.to_string(),
                    name: item.name.clone(),
                    value: data,
                });
            }
        }

        Ok(records)
    }

    fn get_zone_id(&mut self, _domain: &str) -> Result<String> {
        // DigitalOcean uses domain name as the zone identifier
        Ok(self.domain.clone())
    }
}

impl DnsProviderBase for DigitalOceanAdapter {
    fn atomic_ops(&mut self) -> &mut dyn AtomicDnsOperations {
        self
    }
}

impl DnsProviderAdapter for DigitalOceanAdapter {
    fn create_txt(&self, record_name: &str, value: &str) -> Result<()> {
        // Use DnsProviderBase for backward compatibility
        let mut adapter = DigitalOceanAdapter::new(self.api_token.clone(), self.domain.clone());
        adapter.set_txt_record(record_name, value)?;
        Ok(())
    }

    fn cleanup_txt(&self, record_name: &str) -> Result<()> {
        // Use DnsProviderBase for backward compatibility
        let mut adapter = DigitalOceanAdapter::new(self.api_token.clone(), self.domain.clone());
        adapter.delete_txt_record(record_name)?;
        Ok(())
    }
}
