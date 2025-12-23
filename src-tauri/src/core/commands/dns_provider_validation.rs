use anyhow::Context;
use tauri::async_runtime::spawn_blocking;

use crate::core::types::{
    DnsProviderErrorCategory, DnsProviderTokenValidationResult, DnsProviderType,
    ValidateDnsProviderTokenRequest,
};
use crate::issuance::dns_providers::http;

/// Validates DNS provider credentials without storing them.
#[tauri::command]
pub async fn dns_provider_validate_token(
    validate_req: ValidateDnsProviderTokenRequest,
) -> Result<DnsProviderTokenValidationResult, String> {
    spawn_blocking(move || -> Result<DnsProviderTokenValidationResult, anyhow::Error> {
        let result = match validate_req.provider_type {
            DnsProviderType::Cloudflare => {
                let token = validate_req
                    .api_token
                    .clone()
                    .filter(|value| !value.trim().is_empty())
                    .ok_or_else(|| anyhow::anyhow!("API token is required for Cloudflare"))?;
                validate_cloudflare_token(&token)
            }
            DnsProviderType::DigitalOcean => {
                let token = validate_req
                    .api_token
                    .clone()
                    .filter(|value| !value.trim().is_empty())
                    .ok_or_else(|| anyhow::anyhow!("API token is required for DigitalOcean"))?;
                validate_digitalocean_token(&token)
            }
            DnsProviderType::Route53 => {
                let access_key = validate_req
                    .route53_access_key
                    .clone()
                    .filter(|value| !value.trim().is_empty())
                    .ok_or_else(|| anyhow::anyhow!("Route 53 access key is required"))?;
                let secret_key = validate_req
                    .route53_secret_key
                    .clone()
                    .filter(|value| !value.trim().is_empty())
                    .ok_or_else(|| anyhow::anyhow!("Route 53 secret key is required"))?;
                validate_route53_token(&access_key, &secret_key)
            }
            DnsProviderType::Manual => Err(anyhow::anyhow!(
                "manual DNS providers do not require token validation"
            )),
        };

        match result {
            Ok(()) => Ok(DnsProviderTokenValidationResult {
                success: true,
                error: None,
                error_category: None,
            }),
            Err(err) => {
                let category = categorize_dns_error(&err);
                Ok(DnsProviderTokenValidationResult {
                    success: false,
                    error: Some(err.to_string()),
                    error_category: Some(category),
                })
            }
        }
    })
    .await
    .map_err(|err| format!("DNS provider token validation join error: {err}"))?
    .map_err(|err: anyhow::Error| err.to_string())
}

fn validate_cloudflare_token(token: &str) -> Result<(), anyhow::Error> {
    #[derive(serde::Deserialize)]
    struct CloudflareZoneListResponse {
        success: bool,
    }

    let client = http::HttpClient::shared();
    let response = client
        .get("https://api.cloudflare.com/client/v4/zones")
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .send()
        .context("Failed to list Cloudflare zones")?;

    if !response.status().is_success() {
        if response.status() == 401 || response.status() == 403 {
            return Err(anyhow::anyhow!(
                "Cloudflare authentication failed: invalid API token"
            ));
        }
        if response.status() == 429 {
            return Err(anyhow::anyhow!("Cloudflare rate limit exceeded"));
        }
        return Err(http::status_error("Cloudflare", response.status(), None));
    }

    let payload: CloudflareZoneListResponse = response
        .json()
        .context("Failed to parse Cloudflare zone list response")?;
    if !payload.success {
        return Err(anyhow::anyhow!(
            "Cloudflare API returned unsuccessful response"
        ));
    }
    Ok(())
}

fn validate_digitalocean_token(token: &str) -> Result<(), anyhow::Error> {
    let client = http::HttpClient::shared();
    let response = client
        .get("https://api.digitalocean.com/v2/domains")
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .context("Failed to list DigitalOcean domains")?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().unwrap_or_default();
        return Err(http::status_error("DigitalOcean", status, Some(body)));
    }
    Ok(())
}

fn validate_route53_token(access_key: &str, secret_key: &str) -> Result<(), anyhow::Error> {
    let rt = tokio::runtime::Runtime::new().context("Failed to create tokio runtime")?;
    rt.block_on(async move {
        use aws_config::BehaviorVersion;
        use aws_sdk_route53::config::Credentials;
        use aws_sdk_route53::Client;

        let credentials = Credentials::new(access_key, secret_key, None, None, "sslboard");
        let config = aws_config::defaults(BehaviorVersion::latest())
            .credentials_provider(credentials)
            .load()
            .await;
        let client = Client::new(&config);
        client
            .list_hosted_zones()
            .send()
            .await
            .context("Failed to list Route 53 hosted zones")?;
        Ok(())
    })
}

pub(crate) fn categorize_dns_error(err: &anyhow::Error) -> DnsProviderErrorCategory {
    let err_str = err.to_string().to_lowercase();
    if err_str.contains("auth")
        || err_str.contains("unauthorized")
        || err_str.contains("forbidden")
        || err_str.contains("invalid credentials")
        || err_str.contains("access denied")
    {
        DnsProviderErrorCategory::AuthError
    } else if err_str.contains("not found")
        || err_str.contains("404")
        || err_str.contains("no such")
    {
        DnsProviderErrorCategory::NotFound
    } else if err_str.contains("rate limit")
        || err_str.contains("429")
        || err_str.contains("too many requests")
    {
        DnsProviderErrorCategory::RateLimited
    } else if err_str.contains("network")
        || err_str.contains("timeout")
        || err_str.contains("connection")
        || err_str.contains("dns")
    {
        DnsProviderErrorCategory::NetworkError
    } else {
        DnsProviderErrorCategory::Unknown
    }
}

#[cfg(test)]
mod tests {
    use super::{categorize_dns_error, DnsProviderErrorCategory};

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
