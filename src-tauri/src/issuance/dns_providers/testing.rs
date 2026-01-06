use anyhow::{Context, Result};
use log::{info, warn};
use serde::Deserialize;
use std::time::Duration;

use super::base::AtomicDnsOperations;
use crate::issuance::dns::{DnsPropagationResult, PropagationState};

/// Queries Google DNS via HTTPS for a TXT record.
/// This is a public function that can be used by other modules for DNS testing.
/// Uses default normalization (trim quotes and whitespace).
pub fn query_google_dns(record_name: &str, expected_value: &str) -> Result<DnsPropagationResult> {
    // Use default normalization for backward compatibility
    let normalized_expected = expected_value.trim().trim_matches('"').trim().to_string();
    query_google_dns_with_normalization(record_name, &normalized_expected, &DefaultNormalizer)
}

/// Internal function that queries Google DNS with provider-specific normalization.
pub fn query_google_dns_with_normalization(
    record_name: &str,
    normalized_expected: &str,
    normalizer: &dyn AtomicDnsOperations,
) -> Result<DnsPropagationResult> {
    let url = format!(
        "https://dns.google/resolve?name={}&type=TXT&random_padding=x",
        record_name
    );
    let timeout = resolve_dns_timeout();

    info!("[dns-test] Querying Google DNS for {}", record_name);

    let agent = ureq::AgentBuilder::new().timeout(timeout).build();
    let response = agent
        .get(&url)
        .set("Accept", "application/dns-json")
        .call()
        .context("Failed to query Google DNS")?;

    let body = response
        .into_string()
        .context("Failed to read Google DNS response body")?;

    let dns_response: GoogleDnsResponse =
        serde_json::from_str(&body).context("Failed to parse Google DNS response")?;

    info!(
        "[dns-test] Google DNS responded: status={}, has_answer={}",
        dns_response.status,
        dns_response.answer.is_some()
    );

    Ok(interpret_dns_response_with_normalization(
        &dns_response,
        record_name,
        normalized_expected,
        normalizer,
    ))
}

/// Default normalizer for backward compatibility (used by query_google_dns).
pub struct DefaultNormalizer;

impl AtomicDnsOperations for DefaultNormalizer {
    fn create_one_record(&mut self, _record_name: &str, _value: &str) -> Result<String> {
        unreachable!("DefaultNormalizer is only used for normalization")
    }

    fn delete_one_record(&mut self, _record_id: &str) -> Result<()> {
        unreachable!("DefaultNormalizer is only used for normalization")
    }

    fn list_records(&mut self, _record_name: &str) -> Result<Vec<super::base::DnsRecord>> {
        unreachable!("DefaultNormalizer is only used for normalization")
    }

    fn get_zone_id(&mut self, _domain: &str) -> Result<String> {
        unreachable!("DefaultNormalizer is only used for normalization")
    }
}

/// Interprets a Google DNS response into a DnsPropagationResult.
/// Uses provider-specific normalization for comparison.
pub fn interpret_dns_response_with_normalization(
    response: &GoogleDnsResponse,
    record_name: &str,
    normalized_expected: &str,
    normalizer: &dyn AtomicDnsOperations,
) -> DnsPropagationResult {
    let mut observed = Vec::new();
    let mut saw_nxdomain = false;
    let mut saw_ok = false;

    if let Some(answers) = &response.answer {
        for ans in answers {
            if let Some(data) = &ans.data {
                // Google DNS returns quoted values, normalize using provider logic
                let normalized = normalizer.normalize_value(data);
                observed.push(normalized);
            }
        }
    }

    if response.status == 3 {
        saw_nxdomain = true;
    }
    if response.status == 0 {
        saw_ok = true;
    }
    if response.status != 0 && response.status != 3 {
        warn!(
            "[dns-test] Unexpected status {} for {}",
            response.status, record_name
        );
    }

    if observed.iter().any(|val| val == normalized_expected) {
        return DnsPropagationResult {
            state: PropagationState::Found,
            reason: None,
            observed_values: observed,
        };
    }

    if !observed.is_empty() {
        return DnsPropagationResult {
            state: PropagationState::WrongContent,
            reason: Some("TXT record present with different value".to_string()),
            observed_values: observed,
        };
    }

    if saw_ok {
        return DnsPropagationResult {
            state: PropagationState::Pending,
            reason: Some("record not found yet".to_string()),
            observed_values: observed,
        };
    }

    if saw_nxdomain {
        return DnsPropagationResult {
            state: PropagationState::NxDomain,
            reason: Some("record not found (NXDOMAIN)".to_string()),
            observed_values: observed,
        };
    }

    DnsPropagationResult {
        state: PropagationState::Error,
        reason: Some("no response from DNS resolver".to_string()),
        observed_values: observed,
    }
}

/// Interprets a Google DNS response into a DnsPropagationResult.
/// Uses default normalization (for backward compatibility and tests).
#[allow(dead_code)] // Used in tests via interpret_dns_response
pub fn interpret_dns_response(
    response: &GoogleDnsResponse,
    record_name: &str,
    expected_value: &str,
) -> DnsPropagationResult {
    let normalized_expected = expected_value.trim().trim_matches('"').trim().to_string();
    interpret_dns_response_with_normalization(
        response,
        record_name,
        &normalized_expected,
        &DefaultNormalizer,
    )
}

#[allow(dead_code)] // Used in tests via interpret_dns_response
fn trim_txt_quotes(value: &str) -> String {
    value.trim().trim_matches('"').trim().to_string()
}

fn resolve_dns_timeout() -> Duration {
    const DEFAULT_TIMEOUT_SECS: u64 = 10;
    let timeout = std::env::var("SSLBOARD_HTTP_TIMEOUT_SECS")
        .ok()
        .and_then(|raw| raw.parse::<u64>().ok())
        .unwrap_or(DEFAULT_TIMEOUT_SECS);
    if timeout == 0 {
        warn!("[dns-test] invalid timeout value; using default");
        return Duration::from_secs(DEFAULT_TIMEOUT_SECS);
    }
    Duration::from_secs(timeout)
}

#[derive(Debug, Deserialize, Clone)]
pub struct GoogleDnsAnswer {
    #[serde(rename = "data")]
    pub data: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GoogleDnsResponse {
    #[serde(rename = "Status")]
    pub status: u32,
    #[serde(rename = "Answer")]
    pub answer: Option<Vec<GoogleDnsAnswer>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interpret_dns_response_found_when_value_matches() {
        let response = GoogleDnsResponse {
            status: 0,
            answer: Some(vec![GoogleDnsAnswer {
                data: Some("\"expected-value\"".to_string()),
            }]),
        };
        let result = interpret_dns_response(&response, "test.example.com", "expected-value");
        assert!(matches!(result.state, PropagationState::Found));
        assert!(
            result
                .observed_values
                .contains(&"expected-value".to_string())
        );
    }

    #[test]
    fn interpret_dns_response_wrong_content_when_value_differs() {
        let response = GoogleDnsResponse {
            status: 0,
            answer: Some(vec![GoogleDnsAnswer {
                data: Some("\"other-value\"".to_string()),
            }]),
        };
        let result = interpret_dns_response(&response, "test.example.com", "expected-value");
        assert!(matches!(result.state, PropagationState::WrongContent));
        assert!(result.observed_values.contains(&"other-value".to_string()));
    }

    #[test]
    fn interpret_dns_response_nxdomain_when_only_nxdomain_seen() {
        let response = GoogleDnsResponse {
            status: 3,
            answer: None,
        };
        let result = interpret_dns_response(&response, "test.example.com", "expected-value");
        assert!(matches!(result.state, PropagationState::NxDomain));
    }

    #[test]
    fn interpret_dns_response_found_beats_nxdomain_mix() {
        let response = GoogleDnsResponse {
            status: 0,
            answer: Some(vec![GoogleDnsAnswer {
                data: Some("\"expected-value\"".to_string()),
            }]),
        };
        let result = interpret_dns_response(&response, "test.example.com", "expected-value");
        assert!(matches!(result.state, PropagationState::Found));
    }

    #[test]
    fn interpret_dns_response_pending_when_ok_but_empty() {
        let response = GoogleDnsResponse {
            status: 0,
            answer: None,
        };
        let result = interpret_dns_response(&response, "test.example.com", "expected-value");
        assert!(matches!(result.state, PropagationState::Pending));
    }

    #[test]
    fn normalize_value_removes_quotes_and_whitespace() {
        let normalizer = DefaultNormalizer;
        assert_eq!(normalizer.normalize_value("\"test\""), "test");
        assert_eq!(normalizer.normalize_value(" \"test\" "), "test");
        assert_eq!(normalizer.normalize_value("test"), "test");
        assert_eq!(normalizer.normalize_value("\"test value\""), "test value");
    }

    #[test]
    fn interpret_dns_response_handles_multiple_answers() {
        let response = GoogleDnsResponse {
            status: 0,
            answer: Some(vec![
                GoogleDnsAnswer {
                    data: Some("\"value1\"".to_string()),
                },
                GoogleDnsAnswer {
                    data: Some("\"value2\"".to_string()),
                },
            ]),
        };
        let result = interpret_dns_response(&response, "test.example.com", "value1");
        assert!(matches!(result.state, PropagationState::Found));
        assert_eq!(result.observed_values.len(), 2);
        assert!(result.observed_values.contains(&"value1".to_string()));
        assert!(result.observed_values.contains(&"value2".to_string()));
    }

    #[test]
    fn interpret_dns_response_wrong_content_with_multiple_answers() {
        let response = GoogleDnsResponse {
            status: 0,
            answer: Some(vec![
                GoogleDnsAnswer {
                    data: Some("\"value1\"".to_string()),
                },
                GoogleDnsAnswer {
                    data: Some("\"value2\"".to_string()),
                },
            ]),
        };
        let result = interpret_dns_response(&response, "test.example.com", "expected-value");
        assert!(matches!(result.state, PropagationState::WrongContent));
        assert_eq!(result.observed_values.len(), 2);
    }
}
