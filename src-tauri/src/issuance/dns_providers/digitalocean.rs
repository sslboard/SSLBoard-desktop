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

    fn create_txt_record(&self, record_name: &str, value: &str) -> Result<u64> {
        let client = reqwest::blocking::Client::new();

        let record = DigitalOceanDnsRecord {
            record_type: "TXT".to_string(),
            name: record_name.to_string(),
            data: value.to_string(),
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

    fn delete_txt_record(&self, record_name: &str) -> Result<()> {
        let client = reqwest::blocking::Client::new();

        // First, find the record ID
        let response = client
            .get(&format!(
                "https://api.digitalocean.com/v2/domains/{}/records?type=TXT&name={}",
                self.domain, record_name
            ))
            .header("Authorization", format!("Bearer {}", self.api_token))
            .send()
            .context("Failed to list DigitalOcean DNS records")?;

        if !response.status().is_success() {
            return Err(anyhow!("Failed to list DigitalOcean DNS records"));
        }

        #[derive(Deserialize)]
        struct DigitalOceanDnsRecordListResponse {
            domain_records: Vec<DigitalOceanDnsRecordListItem>,
        }

        #[derive(Deserialize)]
        struct DigitalOceanDnsRecordListItem {
            id: u64,
            name: String,
        }

        let list_result: DigitalOceanDnsRecordListResponse = response
            .json()
            .context("Failed to parse DigitalOcean DNS record list")?;

        let record_id = list_result
            .domain_records
            .iter()
            .find(|r| r.name == record_name)
            .ok_or_else(|| anyhow!("TXT record not found: {}", record_name))?
            .id;

        // Delete the record
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

        Ok(())
    }
}

impl DnsProviderAdapter for DigitalOceanAdapter {
    fn create_txt(&self, record_name: &str, value: &str) -> Result<()> {
        self.create_txt_record(record_name, value)?;
        Ok(())
    }

    fn cleanup_txt(&self, record_name: &str) -> Result<()> {
        self.delete_txt_record(record_name)?;
        Ok(())
    }
}

