use anyhow::{anyhow, Result};

use crate::{secrets::manager::SecretManager, storage::dns::DnsProvider};

pub trait DnsProviderAdapter: Send + Sync {
    fn create_txt(&self, record_name: &str, value: &str) -> Result<()>;
    fn cleanup_txt(&self, record_name: &str) -> Result<()>;
}

pub struct UnsupportedDnsProviderAdapter {
    reason: String,
}

impl UnsupportedDnsProviderAdapter {
    pub fn new(reason: String) -> Self {
        Self { reason }
    }
}

impl DnsProviderAdapter for UnsupportedDnsProviderAdapter {
    fn create_txt(&self, _record_name: &str, _value: &str) -> Result<()> {
        Err(anyhow!(self.reason.clone()))
    }

    fn cleanup_txt(&self, _record_name: &str) -> Result<()> {
        Err(anyhow!(self.reason.clone()))
    }
}

pub fn adapter_for_provider(
    provider: &DnsProvider,
    _secrets: &SecretManager,
) -> Box<dyn DnsProviderAdapter> {
    let reason = if provider.provider_type == "manual" {
        "manual DNS providers do not support automated test connections".to_string()
    } else {
        format!(
            "provider type '{}' does not have an adapter yet",
            provider.provider_type
        )
    };
    Box::new(UnsupportedDnsProviderAdapter::new(reason))
}
