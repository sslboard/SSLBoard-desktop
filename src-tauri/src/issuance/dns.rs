use anyhow::{anyhow, Context, Result};
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
        format!("_acme-challenge.{trimmed}")
    }

    fn query_txt(record_name: &str) -> Result<GoogleDnsResponse> {
        let url = format!(
            "https://dns.google/resolve?name={record_name}&type=TXT&random_padding=x"
        );
        let agent = ureq::AgentBuilder::new()
            .timeout(Duration::from_secs(10))
            .build();
        let response = agent
            .get(&url)
            .call()
            .with_context(|| format!("dns query failed for {record_name}"))?;
        let body = response.into_string()?;
        let parsed: GoogleDnsResponse = serde_json::from_str(&body)
            .with_context(|| format!("failed to parse dns response for {record_name}"))?;
        Ok(parsed)
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
        let response = Self::query_txt(&record_name)?;

        match response.status {
            0 => {
                let mut observed = Vec::new();
                if let Some(answers) = response.answer {
                    for ans in answers {
                        if let Some(data) = ans.data {
                            observed.push(trim_txt_quotes(&data));
                        }
                    }
                }
                if observed.iter().any(|val| val == &req.value) {
                    return Ok(DnsPropagationResult {
                        state: PropagationState::Found,
                        reason: None,
                        observed_values: observed,
                    });
                }
                if observed.is_empty() {
                    Ok(DnsPropagationResult {
                        state: PropagationState::Pending,
                        reason: Some("record not found yet".to_string()),
                        observed_values: observed,
                    })
                } else {
                    Ok(DnsPropagationResult {
                        state: PropagationState::WrongContent,
                        reason: Some("TXT record present with different value".to_string()),
                        observed_values: observed,
                    })
                }
            }
            3 => Ok(DnsPropagationResult {
                state: PropagationState::NxDomain,
                reason: Some("record not found (NXDOMAIN)".to_string()),
                observed_values: Vec::new(),
            }),
            code => Err(anyhow!("unexpected DNS status {code} for {record_name}")),
        }
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

#[derive(Debug, Deserialize)]
struct GoogleDnsAnswer {
    #[serde(rename = "data")]
    data: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GoogleDnsResponse {
    #[serde(rename = "Status")]
    status: u32,
    #[serde(rename = "Answer")]
    answer: Option<Vec<GoogleDnsAnswer>>,
}
