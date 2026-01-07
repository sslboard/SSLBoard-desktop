use anyhow::{Result, anyhow};
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

/// Represents a DNS-01 challenge request.
#[derive(Debug, Clone)]
pub struct DnsChallengeRequest {
    pub domain: String,
    pub value: String,
    pub zone: Option<String>,
}

/// The computed TXT record instructions for the UI.
#[derive(Debug, Clone, Serialize)]
pub struct DnsRecordInstruction {
    pub adapter: String,
    pub record_name: String,
    pub value: String,
    pub zone: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PropagationState {
    Pending,
    Found,
    NxDomain,
    WrongContent,
    Error,
}

#[derive(Debug, Clone, Serialize)]
pub struct DnsPropagationResult {
    pub state: PropagationState,
    pub reason: Option<String>,
    pub observed_values: Vec<String>,
}

pub trait DnsAdapter: Send + Sync {
    fn id(&self) -> &'static str;
    fn present_txt(&self, req: &DnsChallengeRequest) -> Result<DnsRecordInstruction>;
    fn cleanup_txt(&self, _req: &DnsChallengeRequest) -> Result<()> {
        Ok(())
    }
    fn check_propagation(&self, req: &DnsChallengeRequest) -> Result<DnsPropagationResult>;
}

/// Manual DNS adapter that emits instructions and checks propagation via DNS over HTTPS.
pub struct ManualDnsAdapter;

impl ManualDnsAdapter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ManualDnsAdapter {
    fn default() -> Self {
        Self::new()
    }
}

pub fn record_name(domain: &str) -> String {
    let trimmed = domain.trim_end_matches('.');
    if trimmed.starts_with("_acme-challenge.") {
        trimmed.to_string()
    } else {
        format!("_acme-challenge.{trimmed}")
    }
}

fn query_txt(record_name: &str, expected_value: Option<&str>) -> Result<Vec<GoogleDnsResponse>> {
    info!(
        "[dns-test] Starting parallel DNS queries for {}",
        record_name
    );
    let urls = [
        (
            "Google DNS",
            format!("https://dns.google/resolve?name={record_name}&type=TXT&random_padding=x"),
            Some("application/dns-json"),
        ),
        (
            "Cloudflare DNS",
            format!("https://cloudflare-dns.com/dns-query?name={record_name}&type=TXT"),
            Some("application/dns-json"),
        ),
    ];

    let timeout = resolve_dns_timeout();
    let (tx, rx) = mpsc::channel();

    // Spawn parallel queries
    for (resolver_name, url, accept) in urls {
        let tx = tx.clone();
        let url_clone = url.clone();
        let record_name_clone = record_name.to_string();
        let resolver_name_clone = resolver_name.to_string();
        let expected_value_clone = expected_value.map(|s| s.to_string());

        thread::spawn(move || {
            info!(
                "[dns-test] Querying {} for {}",
                resolver_name_clone, record_name_clone
            );
            let agent = ureq::AgentBuilder::new().timeout(timeout).build();

            let mut req = agent.get(&url_clone);
            if let Some(accept) = accept {
                req = req.set("Accept", accept);
            }

            let result = match req.call() {
                Ok(resp) => {
                    match resp.into_string() {
                        Ok(body) => {
                            match serde_json::from_str::<GoogleDnsResponse>(&body) {
                                Ok(parsed) => {
                                    info!(
                                        "[dns-test] {} responded: status={}, has_answer={}",
                                        resolver_name_clone,
                                        parsed.status,
                                        parsed.answer.is_some()
                                    );

                                    // Check if this response contains the expected value
                                    if let Some(expected) = &expected_value_clone
                                        && let Some(answers) = &parsed.answer
                                    {
                                        for ans in answers {
                                            if let Some(data) = &ans.data {
                                                let trimmed = trim_txt_quotes(data);
                                                if trimmed == *expected {
                                                    info!(
                                                        "[dns-test] {} found expected value!",
                                                        resolver_name_clone
                                                    );
                                                    // Send success immediately
                                                    let _ =
                                                        tx.send(Ok((resolver_name_clone, parsed)));
                                                    return;
                                                }
                                            }
                                        }
                                    }

                                    Ok((resolver_name_clone, parsed))
                                }
                                Err(err) => {
                                    warn!(
                                        "[dns-test] {} parse failed for {} via {}: {}",
                                        resolver_name_clone, record_name_clone, url_clone, err
                                    );
                                    Err(anyhow!("parse failed: {}", err))
                                }
                            }
                        }
                        Err(err) => {
                            warn!(
                                "[dns-test] {} body read failed for {}: {}",
                                resolver_name_clone, record_name_clone, err
                            );
                            Err(anyhow!("body read failed: {}", err))
                        }
                    }
                }
                Err(err) => {
                    warn!(
                        "[dns-test] {} query failed for {} via {}: {}",
                        resolver_name_clone, record_name_clone, url_clone, err
                    );
                    Err(anyhow!("query failed: {}", err))
                }
            };

            let _ = tx.send(result);
        });
    }

    drop(tx); // Close sender so receiver knows when all threads are done

    let mut results = Vec::new();

    // Collect results, returning early if we find the expected value
    for received in rx {
        match received {
            Ok((resolver_name, response)) => {
                // Check if this response has the expected value
                if let Some(expected) = expected_value
                    && let Some(answers) = &response.answer
                {
                    for ans in answers {
                        if let Some(data) = &ans.data {
                            let trimmed = trim_txt_quotes(data);
                            if trimmed == expected {
                                info!(
                                    "[dns-test] Found expected value via {}, returning immediately",
                                    resolver_name
                                );
                                return Ok(vec![response]);
                            }
                        }
                    }
                }
                results.push(response);
            }
            Err(err) => {
                debug!("[dns-test] One resolver failed: {}", err);
            }
        }
    }

    if results.is_empty() {
        warn!("[dns-test] All DNS queries failed for {}", record_name);
        Err(anyhow!(
            "dns query failed for {record_name} across resolvers"
        ))
    } else {
        info!(
            "[dns-test] Collected {} response(s) for {}",
            results.len(),
            record_name
        );
        Ok(results)
    }
}

pub fn check_txt_record(record_name: &str, expected_value: &str) -> Result<DnsPropagationResult> {
    info!(
        "[dns-test] Checking TXT record {} for value {}",
        record_name, expected_value
    );
    let responses = query_txt(record_name, Some(expected_value))?;
    let req = DnsChallengeRequest {
        domain: record_name.to_string(),
        value: expected_value.to_string(),
        zone: None,
    };
    let result = interpret_dns_results(&responses, &req);
    info!(
        "[dns-test] DNS check result for {}: state={:?}, observed={:?}",
        record_name, result.state, result.observed_values
    );
    Ok(result)
}

impl DnsAdapter for ManualDnsAdapter {
    fn id(&self) -> &'static str {
        "manual"
    }

    fn present_txt(&self, req: &DnsChallengeRequest) -> Result<DnsRecordInstruction> {
        let record_name = record_name(&req.domain);
        let zone = req.zone.clone().unwrap_or_else(|| derive_zone(&req.domain));
        Ok(DnsRecordInstruction {
            adapter: self.id().to_string(),
            record_name,
            value: req.value.clone(),
            zone,
        })
    }

    fn check_propagation(&self, req: &DnsChallengeRequest) -> Result<DnsPropagationResult> {
        let record_name = record_name(&req.domain);
        let responses = query_txt(&record_name, Some(&req.value))?;
        debug!(
            "[dns] checked {record_name}: statuses={:?} answers={:?}",
            responses.iter().map(|r| r.status).collect::<Vec<_>>(),
            responses
                .iter()
                .filter_map(|r| r.answer.as_ref())
                .flatten()
                .filter_map(|a| a.data.clone())
                .collect::<Vec<_>>()
        );
        Ok(interpret_dns_results(&responses, req))
    }
}

fn trim_txt_quotes(value: &str) -> String {
    value.trim_matches('"').trim_matches(' ').trim().to_string()
}

fn resolve_dns_timeout() -> Duration {
    const DEFAULT_TIMEOUT_SECS: u64 = 10;
    let timeout = std::env::var("SSLBOARD_HTTP_TIMEOUT_SECS")
        .ok()
        .and_then(|raw| raw.parse::<u64>().ok())
        .unwrap_or(DEFAULT_TIMEOUT_SECS);
    if timeout == 0 {
        warn!("[dns] invalid timeout value; using default");
        return Duration::from_secs(DEFAULT_TIMEOUT_SECS);
    }
    Duration::from_secs(timeout)
}

fn derive_zone(hostname: &str) -> String {
    let parts: Vec<&str> = hostname.trim_end_matches('.').split('.').collect();
    if parts.len() >= 2 {
        format!("{}.{}", parts[parts.len() - 2], parts[parts.len() - 1])
    } else {
        hostname.to_string()
    }
}

#[derive(Debug, Deserialize, Clone)]
struct GoogleDnsAnswer {
    #[serde(rename = "data")]
    data: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
struct GoogleDnsResponse {
    #[serde(rename = "Status")]
    status: u32,
    #[serde(rename = "Answer")]
    answer: Option<Vec<GoogleDnsAnswer>>,
}

fn interpret_dns_results(
    responses: &[GoogleDnsResponse],
    req: &DnsChallengeRequest,
) -> DnsPropagationResult {
    let mut observed = Vec::new();
    let mut saw_nxdomain = false;
    let mut saw_ok = false;

    for response in responses {
        if let Some(answers) = &response.answer {
            for ans in answers {
                if let Some(data) = &ans.data {
                    observed.push(trim_txt_quotes(data));
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
                "[dns] unexpected status {} for {}",
                response.status, req.domain
            );
        }
    }

    if observed.iter().any(|val| val == &req.value) {
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
        reason: Some("no responses from DNS resolvers".to_string()),
        observed_values: observed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_req() -> DnsChallengeRequest {
        DnsChallengeRequest {
            domain: "test.example.com".to_string(),
            value: "expected-value".to_string(),
            zone: None,
        }
    }

    #[test]
    fn interprets_found_when_value_matches() {
        let responses = vec![GoogleDnsResponse {
            status: 0,
            answer: Some(vec![GoogleDnsAnswer {
                data: Some("\"expected-value\"".to_string()),
            }]),
        }];
        let result = interpret_dns_results(&responses, &make_req());
        assert!(matches!(result.state, PropagationState::Found));
        assert!(
            result
                .observed_values
                .contains(&"expected-value".to_string())
        );
    }

    #[test]
    fn interprets_wrong_content_when_value_differs() {
        let responses = vec![GoogleDnsResponse {
            status: 0,
            answer: Some(vec![GoogleDnsAnswer {
                data: Some("\"other-value\"".to_string()),
            }]),
        }];
        let result = interpret_dns_results(&responses, &make_req());
        assert!(matches!(result.state, PropagationState::WrongContent));
        assert!(result.observed_values.contains(&"other-value".to_string()));
    }

    #[test]
    fn interprets_nxdomain_when_only_nxdomain_seen() {
        let responses = vec![GoogleDnsResponse {
            status: 3,
            answer: None,
        }];
        let result = interpret_dns_results(&responses, &make_req());
        assert!(matches!(result.state, PropagationState::NxDomain));
    }

    #[test]
    fn found_beats_nxdomain_mix() {
        let responses = vec![
            GoogleDnsResponse {
                status: 3,
                answer: None,
            },
            GoogleDnsResponse {
                status: 0,
                answer: Some(vec![GoogleDnsAnswer {
                    data: Some("\"expected-value\"".to_string()),
                }]),
            },
        ];
        let result = interpret_dns_results(&responses, &make_req());
        assert!(matches!(result.state, PropagationState::Found));
    }

    #[test]
    fn pending_when_ok_but_empty() {
        let responses = vec![GoogleDnsResponse {
            status: 0,
            answer: None,
        }];
        let result = interpret_dns_results(&responses, &make_req());
        assert!(matches!(result.state, PropagationState::Pending));
    }

    #[test]
    fn record_name_adds_acme_prefix() {
        assert_eq!(
            record_name("example.com"),
            "_acme-challenge.example.com"
        );
    }

    #[test]
    fn record_name_preserves_existing_prefix() {
        assert_eq!(
            record_name("_acme-challenge.example.com"),
            "_acme-challenge.example.com"
        );
    }

    /// Live-check against real DNS for debugging the ezs3.net TXT record.
    /// Ignored by default because it requires network access.
    #[test]
    #[ignore]
    fn resolves_live_txt_for_ezs3_net() -> Result<()> {
        let record_name = record_name("test.ezs3.net");
        let responses = query_txt(&record_name, None)?;
        let req = DnsChallengeRequest {
            domain: "test.ezs3.net".into(),
            value: "cj9WcLQwCB5xDkgRQ312yXLGko4p9WY-oQjML_T7DIQ".into(),
            zone: None,
        };
        let result = interpret_dns_results(&responses, &req);
        assert!(
            matches!(result.state, PropagationState::Found),
            "expected Found, got {:?} with observed {:?}",
            result.state,
            result.observed_values
        );
        Ok(())
    }
}
