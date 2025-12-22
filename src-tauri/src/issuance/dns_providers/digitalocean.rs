use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

use super::DnsProviderAdapter;

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
        let client = reqwest::blocking::Client::new();
        let response = client
            .get(&format!(
                "https://api.digitalocean.com/v2/domains/{}/records?type=TXT&name={}",
                self.domain, relative_name
            ))
            .header("Authorization", format!("Bearer {}", self.api_token))
            .send()
            .context("Failed to list DigitalOcean DNS records")?;

        if !response.status().is_success() {
            if response.status() == 401 || response.status() == 403 {
                return Err(anyhow!("DigitalOcean authentication failed"));
            }
            if response.status() == 429 {
                return Err(anyhow!("DigitalOcean rate limit exceeded"));
            }
            return Err(anyhow!("DigitalOcean API error: {}", response.status()));
        }

        let list_result: DigitalOceanDnsRecordListResponse = response
            .json()
            .context("Failed to parse DigitalOcean DNS record list")?;

        Ok(list_result.domain_records)
    }

    fn create_txt_record(&self, record_name: &str, value: &str) -> Result<u64> {
        let client = reqwest::blocking::Client::new();
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
            if response.status() == 401 || response.status() == 403 {
                return Err(anyhow!("DigitalOcean authentication failed"));
            }
            if response.status() == 429 {
                return Err(anyhow!("DigitalOcean rate limit exceeded"));
            }
            let error_text = response.text().unwrap_or_default();
            return Err(anyhow!("DigitalOcean API error: {}", error_text));
        }

        let result: DigitalOceanDnsRecordResponse = response
            .json()
            .context("Failed to parse DigitalOcean DNS record response")?;

        Ok(result.domain_record.id)
    }

    fn update_txt_record(&self, record_id: u64, record_name: &str, value: &str) -> Result<()> {
        let client = reqwest::blocking::Client::new();
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
            if response.status() == 401 || response.status() == 403 {
                return Err(anyhow!("DigitalOcean authentication failed"));
            }
            if response.status() == 429 {
                return Err(anyhow!("DigitalOcean rate limit exceeded"));
            }
            let error_text = response.text().unwrap_or_default();
            return Err(anyhow!("DigitalOcean API error: {}", error_text));
        }

        Ok(())
    }

    fn upsert_txt_record(&self, record_name: &str, value: &str) -> Result<()> {
        let existing = self.list_txt_records(record_name)?;
        if existing.is_empty() {
            self.create_txt_record(record_name, value)?;
        } else {
            for record in &existing {
                self.update_txt_record(record.id, record_name, value)?;
            }
        }

        let updated = self.list_txt_records(record_name)?;
        let expected_normalized = Self::normalize_txt_content(value);
        let matched = updated.iter().any(|record| {
            record
                .data
                .as_deref()
                .map(|d| Self::normalize_txt_content(d) == expected_normalized)
                .unwrap_or(false)
        });
        if !matched {
            return Err(anyhow!(
                "DigitalOcean record verification failed for {}",
                record_name
            ));
        }
        Ok(())
    }

    fn delete_txt_record(&self, record_name: &str) -> Result<()> {
        let client = reqwest::blocking::Client::new();

        let list_result = self.list_txt_records(record_name)?;
        let record_ids: Vec<u64> = list_result.iter().map(|record| record.id).collect();
        if record_ids.is_empty() {
            return Err(anyhow!("TXT record not found: {}", record_name));
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
                return Err(anyhow!(
                    "Failed to delete DigitalOcean DNS record: {}",
                    delete_response.status()
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
    #[serde(default)]
    data: Option<String>,
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
