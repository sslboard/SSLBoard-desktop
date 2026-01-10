use anyhow::Result;

use crate::{domain::normalize_domain_for_storage, secrets::manager::SecretManager, storage::dns::DnsProvider};

mod base;
mod cloudflare;
mod digitalocean;
pub(crate) mod http;
mod retry;
mod route53;
mod testing;

pub use base::{AtomicDnsOperations, DnsProviderBase, DnsRecord};
pub use testing::query_google_dns;
pub use retry::{poll_dns_propagation, retry_provider_verification};

pub use cloudflare::CloudflareAdapter;
pub use digitalocean::DigitalOceanAdapter;
pub use route53::Route53Adapter;

pub trait DnsProviderAdapter: Send + Sync {
    fn create_txt(&self, record_name: &str, value: &str) -> Result<()>;
    fn cleanup_txt(&self, record_name: &str) -> Result<()>;
}

pub(crate) fn matches_zone(domain_suffix: &str, zone_name: &str) -> bool {
    let domain_suffix = match normalize_domain_for_storage(domain_suffix) {
        Ok(value) => value,
        Err(_) => return false,
    };
    let zone_name = match normalize_domain_for_storage(zone_name) {
        Ok(value) => value,
        Err(_) => return false,
    };
    zone_name == domain_suffix || domain_suffix.ends_with(&format!(".{}", zone_name))
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
        Err(anyhow::anyhow!(self.reason.clone()))
    }

    fn cleanup_txt(&self, _record_name: &str) -> Result<()> {
        Err(anyhow::anyhow!(self.reason.clone()))
    }
}

pub fn adapter_for_provider(
    provider: &DnsProvider,
    secrets: &SecretManager,
) -> Box<dyn DnsProviderAdapter> {
    match provider.provider_type.as_str() {
        "cloudflare" => {
            if provider.secret_refs.is_empty() {
                return Box::new(UnsupportedDnsProviderAdapter::new(
                    "Cloudflare provider missing API token".to_string(),
                ));
            }
            let token_ref = &provider.secret_refs[0];
            match secrets.resolve_secret(token_ref) {
                Ok(token_bytes) => {
                    if let Ok(token) = String::from_utf8(token_bytes) {
                        let domain_suffix = provider
                            .domain_suffixes
                            .first()
                            .cloned()
                            .unwrap_or_default();
                        Box::new(CloudflareAdapter::new(token, domain_suffix))
                    } else {
                        Box::new(UnsupportedDnsProviderAdapter::new(
                            "Failed to decode Cloudflare API token".to_string(),
                        ))
                    }
                }
                Err(err) => Box::new(UnsupportedDnsProviderAdapter::new(format!(
                    "Failed to resolve Cloudflare API token: {}",
                    err
                ))),
            }
        }
        "digitalocean" => {
            if provider.secret_refs.is_empty() {
                return Box::new(UnsupportedDnsProviderAdapter::new(
                    "DigitalOcean provider missing API token".to_string(),
                ));
            }
            let token_ref = &provider.secret_refs[0];
            match secrets.resolve_secret(token_ref) {
                Ok(token_bytes) => {
                    if let Ok(token) = String::from_utf8(token_bytes) {
                        let domain = provider
                            .domain_suffixes
                            .first()
                            .cloned()
                            .unwrap_or_default();
                        Box::new(DigitalOceanAdapter::new(token, domain))
                    } else {
                        Box::new(UnsupportedDnsProviderAdapter::new(
                            "Failed to decode DigitalOcean API token".to_string(),
                        ))
                    }
                }
                Err(err) => Box::new(UnsupportedDnsProviderAdapter::new(format!(
                    "Failed to resolve DigitalOcean API token: {}",
                    err
                ))),
            }
        }
        "route53" => {
            if provider.secret_refs.len() < 2 {
                return Box::new(UnsupportedDnsProviderAdapter::new(
                    "Route 53 provider missing access key or secret key".to_string(),
                ));
            }
            let access_key_ref = &provider.secret_refs[0];
            let secret_key_ref = &provider.secret_refs[1];
            match (
                secrets.resolve_secret(access_key_ref),
                secrets.resolve_secret(secret_key_ref),
            ) {
                (Ok(access_key_bytes), Ok(secret_key_bytes)) => {
                    match (
                        String::from_utf8(access_key_bytes),
                        String::from_utf8(secret_key_bytes),
                    ) {
                        (Ok(access_key), Ok(secret_key)) => {
                            let domain_suffix = provider
                                .domain_suffixes
                                .first()
                                .cloned()
                                .unwrap_or_default();
                            Box::new(Route53Adapter::new(access_key, secret_key, domain_suffix))
                        }
                        _ => Box::new(UnsupportedDnsProviderAdapter::new(
                            "Failed to decode Route 53 credentials".to_string(),
                        )),
                    }
                }
                _ => Box::new(UnsupportedDnsProviderAdapter::new(
                    "Failed to resolve Route 53 credentials".to_string(),
                )),
            }
        }
        "manual" => Box::new(UnsupportedDnsProviderAdapter::new(
            "manual DNS providers do not support automated test connections".to_string(),
        )),
        _ => Box::new(UnsupportedDnsProviderAdapter::new(format!(
            "provider type '{}' does not have an adapter yet",
            provider.provider_type
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::matches_zone;

    #[test]
    fn matches_exact_zone_name() {
        assert!(matches_zone("example.com", "example.com"));
        assert!(!matches_zone("example.com", "other.com"));
    }

    #[test]
    fn matches_subdomain_suffix() {
        assert!(matches_zone("sub.example.com", "example.com"));
        assert!(!matches_zone("example.com", "sub.example.com"));
    }

    #[test]
    fn matches_idn_suffix() {
        assert!(matches_zone("testé.ezs3.net", "ezs3.net"));
        assert!(matches_zone("xn--test-epa.ezs3.net", "ezs3.net"));
        assert!(matches_zone("testé.fr", "xn--test-epa.fr"));
        assert!(!matches_zone("example.com", "xn--test-epa.fr"));
    }
}
