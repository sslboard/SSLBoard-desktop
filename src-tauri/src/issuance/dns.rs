use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
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

    fn record_name(domain: &str) -> String {
        let trimmed = domain.trim_end_matches('.');
        if trimmed.starts_with("_acme-challenge.") {
            trimmed.to_string()
        } else {
            format!("_acme-challenge.{trimmed}")
        }
    }

    fn query_txt(record_name: &str) -> Result<Vec<GoogleDnsResponse>> {
        let urls = [
            (
                format!("https://dns.google/resolve?name={record_name}&type=TXT&random_padding=x"),
                Some("application/dns-json"),
            ),
            (
                format!("https://cloudflare-dns.com/dns-query?name={record_name}&type=TXT"),
                Some("application/dns-json"),
            ),
        ];
        let agent = ureq::AgentBuilder::new()
            .timeout(Duration::from_secs(10))
            .build();
        let mut results = Vec::new();
        for (url, accept) in urls {
            let mut req = agent.get(&url);
            if let Some(accept) = accept {
                req = req.set("Accept", accept);
            }
            match req.call() {
                Ok(resp) => {
                    if let Ok(body) = resp.into_string() {
                        match serde_json::from_str::<GoogleDnsResponse>(&body) {
                            Ok(parsed) => results.push(parsed),
                            Err(err) => eprintln!("[dns] parse failed for {record_name} via {url}: {err}"),
                        }
                    }
                }
                Err(err) => {
                    eprintln!("[dns] resolver query failed for {record_name} via {url}: {err}");
                }
            }
        }
        if results.is_empty() {
            Err(anyhow!("dns query failed for {record_name} across resolvers"))
        } else {
            Ok(results)
        }
    }

}

impl DnsAdapter for ManualDnsAdapter {
    fn id(&self) -> &'static str {
        "manual"
    }

    fn present_txt(&self, req: &DnsChallengeRequest) -> Result<DnsRecordInstruction> {
        let record_name = Self::record_name(&req.domain);
        let zone = req
            .zone
            .clone()
            .unwrap_or_else(|| derive_zone(&req.domain));
        Ok(DnsRecordInstruction {
            adapter: self.id().to_string(),
            record_name,
            value: req.value.clone(),
            zone,
        })
    }

    fn check_propagation(&self, req: &DnsChallengeRequest) -> Result<DnsPropagationResult> {
        let record_name = Self::record_name(&req.domain);
        let responses = Self::query_txt(&record_name)?;
        eprintln!(
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
    value
        .trim_matches('"')
        .trim_matches(' ')
        .trim()
        .to_string()
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
            eprintln!("[dns] unexpected status {} for {}", response.status, req.domain);
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
        assert!(result.observed_values.contains(&"expected-value".to_string()));
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

    /// Live-check against real DNS for debugging the ezs3.net TXT record.
    /// Ignored by default because it requires network access.
    #[test]
    #[ignore]
    fn resolves_live_txt_for_ezs3_net() {
        let record_name = ManualDnsAdapter::record_name("test.ezs3.net");
        let responses =
            ManualDnsAdapter::query_txt(&record_name).expect("dns query should succeed");
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
    }
}
