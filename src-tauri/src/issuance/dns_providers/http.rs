use std::sync::OnceLock;
use std::time::Duration;

use anyhow::anyhow;
use log::warn;
use reqwest::StatusCode;
use reqwest::blocking::Client;

pub struct HttpClient;

impl HttpClient {
    pub fn shared() -> &'static Client {
        static CLIENT: OnceLock<Client> = OnceLock::new();
        CLIENT.get_or_init(|| {
            let timeout = resolve_timeout();
            reqwest::blocking::Client::builder()
                .timeout(timeout)
                .build()
                .unwrap_or_else(|err| {
                    warn!("[dns-http] failed to build shared client: {err}");
                    reqwest::blocking::Client::new()
                })
        })
    }
}

fn resolve_timeout() -> Duration {
    let env_value = std::env::var("SSLBOARD_HTTP_TIMEOUT_SECS").ok();
    parse_timeout(env_value.as_deref())
}

fn parse_timeout(env_value: Option<&str>) -> Duration {
    const DEFAULT_TIMEOUT_SECS: u64 = 15;
    let timeout = env_value
        .and_then(|raw| raw.parse::<u64>().ok())
        .unwrap_or(DEFAULT_TIMEOUT_SECS);
    if timeout == 0 {
        warn!("[dns-http] invalid timeout value; using default");
        return Duration::from_secs(DEFAULT_TIMEOUT_SECS);
    }
    Duration::from_secs(timeout)
}

pub fn status_error(provider: &str, status: StatusCode, body: Option<String>) -> anyhow::Error {
    if status == StatusCode::UNAUTHORIZED || status == StatusCode::FORBIDDEN {
        return anyhow!("{provider} authentication failed");
    }
    if status == StatusCode::TOO_MANY_REQUESTS {
        return anyhow!("{provider} rate limit exceeded");
    }
    if let Some(body) = body {
        return anyhow!("{provider} API error: {body}");
    }
    anyhow!("{provider} API error: {status}")
}

#[cfg(test)]
mod tests {
    use super::{parse_timeout, status_error};
    use reqwest::StatusCode;
    use std::time::Duration;

    #[test]
    fn status_error_maps_auth() {
        let err = status_error("Cloudflare", StatusCode::UNAUTHORIZED, None);
        assert!(err.to_string().contains("Cloudflare authentication failed"));
    }

    #[test]
    fn status_error_maps_rate_limit() {
        let err = status_error("DigitalOcean", StatusCode::TOO_MANY_REQUESTS, None);
        assert!(err.to_string().contains("DigitalOcean rate limit exceeded"));
    }

    #[test]
    fn status_error_includes_body() {
        let err = status_error(
            "Cloudflare",
            StatusCode::BAD_REQUEST,
            Some("bad request".to_string()),
        );
        let msg = err.to_string();
        assert!(msg.contains("Cloudflare API error"));
        assert!(msg.contains("bad request"));
    }

    #[test]
    fn parse_timeout_defaults() {
        let timeout = parse_timeout(None);
        assert_eq!(timeout, Duration::from_secs(15));
    }

    #[test]
    fn parse_timeout_parses_valid() {
        let timeout = parse_timeout(Some("20"));
        assert_eq!(timeout, Duration::from_secs(20));
    }

    #[test]
    fn parse_timeout_rejects_zero() {
        let timeout = parse_timeout(Some("0"));
        assert_eq!(timeout, Duration::from_secs(15));
    }

    #[test]
    fn parse_timeout_rejects_invalid() {
        let timeout = parse_timeout(Some("nope"));
        assert_eq!(timeout, Duration::from_secs(15));
    }
}
