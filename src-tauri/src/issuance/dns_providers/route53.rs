use anyhow::{anyhow, Context, Result};

use super::{
    base::{AtomicDnsOperations, DnsProviderBase, DnsRecord},
    matches_zone, DnsProviderAdapter,
};

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

    fn format_txt_content(value: &str) -> String {
        let trimmed = value.trim();
        if trimmed.starts_with('"') && trimmed.ends_with('"') {
            trimmed.to_string()
        } else {
            format!("\"{}\"", trimmed.trim_matches('"'))
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
                if matches_zone(&self.domain_suffix, zone_name) {
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

    /// Atomic operation: Creates a single TXT record via Route53 API.
    /// Returns the record name as ID (Route53 doesn't return a separate ID).
    /// Does not check for existing records or verify.
    async fn create_txt_record_atomic(&mut self, record_name: &str, value: &str) -> Result<String> {
        use aws_config::BehaviorVersion;
        use aws_sdk_route53::config::Credentials;
        use aws_sdk_route53::Client;
        use aws_sdk_route53::types::{Change, ChangeBatch, ResourceRecord, ResourceRecordSet, RrType};

        let hosted_zone_id = self.discover_hosted_zone_id().await?;
        let formatted_value = Self::format_txt_content(value);

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
                .value(formatted_value)
                .build()
                .map_err(|e| anyhow!("Failed to build ResourceRecord: {}", e))?]))
            .ttl(300)
            .set_type(Some(RrType::Txt))
            .build()
            .map_err(|e| anyhow!("Failed to build ResourceRecordSet: {}", e))?;

        let change = Change::builder()
            .action(aws_sdk_route53::types::ChangeAction::Upsert)
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

        // Route53 doesn't return a record ID, so we use the record name
        Ok(format!("route53:{}", record_name))
    }

    /// Atomic operation: Deletes a single TXT record via Route53 API.
    /// Does not handle listing - expects the record_set to be provided.
    async fn delete_txt_record_atomic(&mut self, record_set: aws_sdk_route53::types::ResourceRecordSet) -> Result<()> {
        use aws_config::BehaviorVersion;
        use aws_sdk_route53::config::Credentials;
        use aws_sdk_route53::Client;
        use aws_sdk_route53::types::{Change, ChangeBatch};

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

impl AtomicDnsOperations for Route53Adapter {
    fn normalize_value(&self, value: &str) -> String {
        // Route53 stores values with quotes, normalize by removing quotes and trimming
        value.trim().trim_matches('"').trim().to_string()
    }

    fn create_one_record(&mut self, record_name: &str, value: &str) -> Result<String> {
        // Truly atomic: just create the record
        let rt = tokio::runtime::Runtime::new().context("Failed to create tokio runtime")?;
        rt.block_on(self.create_txt_record_atomic(record_name, value))
    }

    fn delete_one_record(&mut self, record_id: &str) -> Result<()> {
        // Extract record name from our ID format
        let record_name = record_id
            .strip_prefix("route53:")
            .ok_or_else(|| anyhow!("Invalid Route53 record ID format"))?;
        
        // For Route53, we need to get the record_set first to delete it
        // This is a limitation of Route53 API - deletion requires the full record_set
        // So we need to list first, then delete
        use aws_config::BehaviorVersion;
        use aws_sdk_route53::config::Credentials;
        use aws_sdk_route53::Client;
        use aws_sdk_route53::types::RrType;

        let rt = tokio::runtime::Runtime::new().context("Failed to create tokio runtime")?;
        let hosted_zone_id = rt.block_on(self.discover_hosted_zone_id())?;
        
        let credentials = Credentials::new(
            &self.access_key,
            &self.secret_key,
            None,
            None,
            "sslboard",
        );

        let config = rt.block_on(
            aws_config::defaults(BehaviorVersion::latest())
                .credentials_provider(credentials)
                .load()
        );

        let client = Client::new(&config);

        let response = rt.block_on(
            client
                .list_resource_record_sets()
                .hosted_zone_id(&hosted_zone_id)
                .send()
        )
        .context("Failed to list Route 53 DNS records")?;

        let record_set = response
            .resource_record_sets()
            .iter()
            .find(|rs| rs.name() == record_name && rs.r#type() == &RrType::Txt)
            .cloned()
            .ok_or_else(|| anyhow!("TXT record not found: {}", record_name))?;

        // Now delete using the record_set
        rt.block_on(self.delete_txt_record_atomic(record_set))?;
        Ok(())
    }

    fn list_records(&mut self, record_name: &str) -> Result<Vec<DnsRecord>> {
        use aws_config::BehaviorVersion;
        use aws_sdk_route53::config::Credentials;
        use aws_sdk_route53::Client;
        use aws_sdk_route53::types::RrType;

        let rt = tokio::runtime::Runtime::new().context("Failed to create tokio runtime")?;
        
        let hosted_zone_id = rt.block_on(self.discover_hosted_zone_id())?;
        
        let credentials = Credentials::new(
            &self.access_key,
            &self.secret_key,
            None,
            None,
            "sslboard",
        );

        let config = rt.block_on(
            aws_config::defaults(BehaviorVersion::latest())
                .credentials_provider(credentials)
                .load()
        );

        let client = Client::new(&config);

        let response = rt.block_on(
            client
                .list_resource_record_sets()
                .hosted_zone_id(&hosted_zone_id)
                .send()
        )
        .context("Failed to list Route 53 DNS records")?;

        let mut records = Vec::new();
        for record_set in response.resource_record_sets() {
            if record_set.name() == record_name && record_set.r#type() == &RrType::Txt {
                for record in record_set.resource_records() {
                    records.push(DnsRecord {
                        id: format!("route53:{}", record_name),
                        name: record_name.to_string(),
                        value: record.value().to_string(),
                    });
                }
            }
        }

        Ok(records)
    }

    fn get_zone_id(&mut self, _domain: &str) -> Result<String> {
        let rt = tokio::runtime::Runtime::new().context("Failed to create tokio runtime")?;
        rt.block_on(self.discover_hosted_zone_id())
    }
}

impl DnsProviderBase for Route53Adapter {
    fn atomic_ops(&mut self) -> &mut dyn AtomicDnsOperations {
        self
    }
}

impl DnsProviderAdapter for Route53Adapter {
    fn create_txt(&self, record_name: &str, value: &str) -> Result<()> {
        // Use DnsProviderBase for backward compatibility
        let mut adapter = Route53Adapter::new(
            self.access_key.clone(),
            self.secret_key.clone(),
            self.domain_suffix.clone(),
        );
        adapter.set_txt_record(record_name, value)?;
        Ok(())
    }

    fn cleanup_txt(&self, record_name: &str) -> Result<()> {
        // Use DnsProviderBase for backward compatibility
        let mut adapter = Route53Adapter::new(
            self.access_key.clone(),
            self.secret_key.clone(),
            self.domain_suffix.clone(),
        );
        adapter.delete_txt_record(record_name)?;
        Ok(())
    }
}
