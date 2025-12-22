use anyhow::{anyhow, Context, Result};

use super::DnsProviderAdapter;

pub struct Route53Adapter {
    access_key: String,
    secret_key: String,
    hosted_zone_cache: Option<String>,
    domain_suffix: String,
}

impl Route53Adapter {
    pub fn new(access_key: String, secret_key: String, domain_suffix: String) -> Self {
        Self {
            access_key,
            secret_key,
            hosted_zone_cache: None,
            domain_suffix,
        }
    }

    async fn discover_hosted_zone_id(&mut self) -> Result<String> {
        if let Some(ref zone_id) = self.hosted_zone_cache {
            return Ok(zone_id.clone());
        }

        use aws_config::BehaviorVersion;
        use aws_sdk_route53::config::Credentials;
        use aws_sdk_route53::Client;

        let credentials = Credentials::new(
            &self.access_key,
            &self.secret_key,
            None,
            None,
            "sslboard",
        );

        let config = aws_config::defaults(BehaviorVersion::latest())
            .credentials_provider(credentials)
            .load()
            .await;

        let client = Client::new(&config);

        let mut paginator = client
            .list_hosted_zones()
            .into_paginator()
            .page_size(100)
            .send();

        while let Some(page) = paginator.next().await {
            let page = page.context("Failed to list Route 53 hosted zones")?;
            
            // hosted_zones() returns &[HostedZone] directly (not Option)
            // name() and id() return &str directly (not Option)
            let zones = page.hosted_zones();
            for zone in zones {
                let name = zone.name();
                let id = zone.id();
                let zone_name = name.trim_end_matches('.');
                if zone_name == self.domain_suffix
                    || self.domain_suffix.ends_with(&format!(".{}", zone_name))
                {
                    let zone_id = id.to_string();
                    self.hosted_zone_cache = Some(zone_id.clone());
                    return Ok(zone_id);
                }
            }
        }

        Err(anyhow!(
            "No Route 53 hosted zone found for domain suffix: {}",
            self.domain_suffix
        ))
    }

    async fn create_txt_record(&mut self, record_name: &str, value: &str) -> Result<()> {
        use aws_config::BehaviorVersion;
        use aws_sdk_route53::config::Credentials;
        use aws_sdk_route53::Client;
        use aws_sdk_route53::types::{Change, ChangeBatch, ResourceRecord, ResourceRecordSet, RrType};

        let hosted_zone_id = self.discover_hosted_zone_id().await?;

        let credentials = Credentials::new(
            &self.access_key,
            &self.secret_key,
            None,
            None,
            "sslboard",
        );

        let config = aws_config::defaults(BehaviorVersion::latest())
            .credentials_provider(credentials)
            .load()
            .await;

        let client = Client::new(&config);

        let record_set = ResourceRecordSet::builder()
            .name(record_name)
            .set_resource_records(Some(vec![ResourceRecord::builder()
                .value(value)
                .build()
                .map_err(|e| anyhow!("Failed to build ResourceRecord: {}", e))?]))
            .ttl(300)
            .set_type(Some(RrType::Txt))
            .build()
            .map_err(|e| anyhow!("Failed to build ResourceRecordSet: {}", e))?;

        let change = Change::builder()
            .action(aws_sdk_route53::types::ChangeAction::Create)
            .resource_record_set(record_set)
            .build()
            .map_err(|e| anyhow!("Failed to build Change: {}", e))?;

        let change_batch = ChangeBatch::builder()
            .changes(change)
            .build()
            .map_err(|e| anyhow!("Failed to build ChangeBatch: {}", e))?;

        let _result = client
            .change_resource_record_sets()
            .hosted_zone_id(&hosted_zone_id)
            .change_batch(change_batch)
            .send()
            .await
            .context("Failed to create Route 53 DNS record")?;

        Ok(())
    }

    async fn delete_txt_record(&mut self, record_name: &str) -> Result<()> {
        use aws_config::BehaviorVersion;
        use aws_sdk_route53::config::Credentials;
        use aws_sdk_route53::Client;
        use aws_sdk_route53::types::{Change, ChangeBatch, RrType};

        let hosted_zone_id = self.discover_hosted_zone_id().await?;

        let credentials = Credentials::new(
            &self.access_key,
            &self.secret_key,
            None,
            None,
            "sslboard",
        );

        let config = aws_config::defaults(BehaviorVersion::latest())
            .credentials_provider(credentials)
            .load()
            .await;

        let client = Client::new(&config);

        // First, get the existing record to delete it
        let list_response = client
            .list_resource_record_sets()
            .hosted_zone_id(&hosted_zone_id)
            .send()
            .await
            .context("Failed to list Route 53 DNS records")?;

        // resource_record_sets() returns &[ResourceRecordSet] directly (not Option)
        let sets = list_response.resource_record_sets();
        let record_set = sets
            .iter()
            .find(|rs| {
                rs.name() == record_name && rs.r#type() == &RrType::Txt
            })
            .cloned()
            .ok_or_else(|| anyhow!("TXT record not found: {}", record_name))?;

        let change = Change::builder()
            .action(aws_sdk_route53::types::ChangeAction::Delete)
            .resource_record_set(record_set)
            .build()
            .map_err(|e| anyhow!("Failed to build Change: {}", e))?;

        let change_batch = ChangeBatch::builder()
            .changes(change)
            .build()
            .map_err(|e| anyhow!("Failed to build ChangeBatch: {}", e))?;

        let _result = client
            .change_resource_record_sets()
            .hosted_zone_id(&hosted_zone_id)
            .change_batch(change_batch)
            .send()
            .await
            .context("Failed to delete Route 53 DNS record")?;

        Ok(())
    }
}

impl DnsProviderAdapter for Route53Adapter {
    fn create_txt(&self, record_name: &str, value: &str) -> Result<()> {
        // Route 53 SDK is async, so we need to use tokio runtime
        let rt = tokio::runtime::Runtime::new().context("Failed to create tokio runtime")?;
        let mut adapter = Route53Adapter::new(
            self.access_key.clone(),
            self.secret_key.clone(),
            self.domain_suffix.clone(),
        );
        rt.block_on(adapter.create_txt_record(record_name, value))?;
        Ok(())
    }

    fn cleanup_txt(&self, record_name: &str) -> Result<()> {
        let rt = tokio::runtime::Runtime::new().context("Failed to create tokio runtime")?;
        let mut adapter = Route53Adapter::new(
            self.access_key.clone(),
            self.secret_key.clone(),
            self.domain_suffix.clone(),
        );
        rt.block_on(adapter.delete_txt_record(record_name))?;
        Ok(())
    }
}

