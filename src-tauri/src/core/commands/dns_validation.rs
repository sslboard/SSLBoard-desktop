use crate::core::types::DnsProviderErrorCategory;

/// Categorizes a DNS error for consistent error reporting.
pub fn categorize_dns_error(error: &anyhow::Error) -> DnsProviderErrorCategory {
    let error_msg = error.to_string().to_lowercase();

    if error_msg.contains("auth")
        || error_msg.contains("unauthorized")
        || error_msg.contains("forbidden")
        || error_msg.contains("invalid credentials")
        || error_msg.contains("access denied")
    {
        DnsProviderErrorCategory::AuthError
    } else if error_msg.contains("not found")
        || error_msg.contains("404")
        || error_msg.contains("no such")
    {
        DnsProviderErrorCategory::NotFound
    } else if error_msg.contains("rate limit")
        || error_msg.contains("429")
        || error_msg.contains("too many requests")
    {
        DnsProviderErrorCategory::RateLimited
    } else if error_msg.contains("network")
        || error_msg.contains("timeout")
        || error_msg.contains("connection")
        || error_msg.contains("dns")
    {
        DnsProviderErrorCategory::NetworkError
    } else {
        DnsProviderErrorCategory::Unknown
    }
}

#[cfg(test)]
mod tests {
    use super::{DnsProviderErrorCategory, categorize_dns_error};

    #[test]
    fn categorizes_auth_errors() {
        let err = anyhow::anyhow!("authentication failed");
        assert!(matches!(
            categorize_dns_error(&err),
            DnsProviderErrorCategory::AuthError
        ));
    }

    #[test]
    fn categorizes_not_found_errors() {
        let err = anyhow::anyhow!("zone not found");
        assert!(matches!(
            categorize_dns_error(&err),
            DnsProviderErrorCategory::NotFound
        ));
    }

    #[test]
    fn categorizes_rate_limit_errors() {
        let err = anyhow::anyhow!("429 too many requests");
        assert!(matches!(
            categorize_dns_error(&err),
            DnsProviderErrorCategory::RateLimited
        ));
    }

    #[test]
    fn categorizes_network_errors() {
        let err = anyhow::anyhow!("network timeout");
        assert!(matches!(
            categorize_dns_error(&err),
            DnsProviderErrorCategory::NetworkError
        ));
    }

    #[test]
    fn categorizes_unknown_errors() {
        let err = anyhow::anyhow!("unexpected issue");
        assert!(matches!(
            categorize_dns_error(&err),
            DnsProviderErrorCategory::Unknown
        ));
    }
}
