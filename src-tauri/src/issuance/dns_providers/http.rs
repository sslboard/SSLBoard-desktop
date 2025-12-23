use std::sync::OnceLock;
use std::time::Duration;

use anyhow::anyhow;
use log::warn;
use reqwest::blocking::Client;
use reqwest::StatusCode;

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
    const DEFAULT_TIMEOUT_SECS: u64 = 15;
    let timeout = std::env::var("SSLBOARD_HTTP_TIMEOUT_SECS")
        .ok()
        .and_then(|raw| raw.parse::<u64>().ok())
        .unwrap_or(DEFAULT_TIMEOUT_SECS);
    if timeout == 0 {
        warn!("[dns-http] invalid timeout value; using default");
        return Duration::from_secs(DEFAULT_TIMEOUT_SECS);
    }
    Duration::from_secs(timeout)
}

pub fn status_error(
    provider: &str,
    status: StatusCode,
    body: Option<String>,
) -> anyhow::Error {
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
    use super::{resolve_timeout, status_error};
    use reqwest::StatusCode;
    use std::sync::{Mutex, OnceLock};
    use std::time::Duration;

    fn with_timeout_env<T>(value: Option<&str>, f: impl FnOnce() -> T) -> T {
        static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        let _guard = ENV_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();
        let key = "SSLBOARD_HTTP_TIMEOUT_SECS";
        let previous = std::env::var(key).ok();
        match value {
            Some(value) => std::env::set_var(key, value),
            None => std::env::remove_var(key),
        }
        let result = f();
        match previous {
            Some(value) => std::env::set_var(key, value),
            None => std::env::remove_var(key),
        }
        result
    }

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
    fn resolve_timeout_defaults() {
        let timeout = with_timeout_env(None, resolve_timeout);
        assert_eq!(timeout, Duration::from_secs(15));
    }

    #[test]
    fn resolve_timeout_parses_env() {
        let timeout = with_timeout_env(Some("20"), resolve_timeout);
        assert_eq!(timeout, Duration::from_secs(20));
    }

    #[test]
    fn resolve_timeout_rejects_zero() {
        let timeout = with_timeout_env(Some("0"), resolve_timeout);
        assert_eq!(timeout, Duration::from_secs(15));
    }

    #[test]
    fn resolve_timeout_rejects_invalid() {
        let timeout = with_timeout_env(Some("nope"), resolve_timeout);
        assert_eq!(timeout, Duration::from_secs(15));
    }
}
