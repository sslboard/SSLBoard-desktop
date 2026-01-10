use anyhow::{Result, anyhow};

pub fn normalize_domain_for_storage(input: &str) -> Result<String> {
    let trimmed = input.trim().trim_end_matches('.');
    if trimmed.is_empty() {
        return Err(anyhow!("domain name is required"));
    }
    let ascii = idna::domain_to_ascii(trimmed)
        .map_err(|err| anyhow!("invalid domain name: {err}"))?;
    Ok(ascii.to_lowercase())
}

pub fn normalize_domain_suffix_for_storage(raw: &str) -> Result<String> {
    let stripped = raw
        .trim()
        .trim_start_matches("*.")
        .trim_start_matches('.')
        .trim_end_matches('.');
    if stripped.is_empty() {
        return Ok(String::new());
    }
    normalize_domain_for_storage(stripped)
}

pub fn normalize_domain_for_display(input: &str) -> String {
    normalize_unicode_domain(input)
}

pub fn normalize_domains_for_display(domains: &[String]) -> Vec<String> {
    domains
        .iter()
        .map(|domain| normalize_domain_for_display(domain))
        .collect()
}

fn normalize_unicode_domain(input: &str) -> String {
    let trimmed = input.trim().trim_end_matches('.');
    let (unicode, _) = idna::domain_to_unicode(trimmed);
    unicode.to_lowercase()
}
