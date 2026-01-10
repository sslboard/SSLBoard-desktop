use anyhow::anyhow;

use crate::storage::dns::parse_domain_suffixes;

pub(crate) fn validate_label(label: &str) -> Result<(), anyhow::Error> {
    if label.trim().is_empty() {
        return Err(anyhow!("provider label is required"));
    }
    Ok(())
}

pub(crate) fn validate_domain_suffixes(raw: &str) -> Result<Vec<String>, anyhow::Error> {
    let domain_suffixes = parse_domain_suffixes(raw)?;
    if domain_suffixes.is_empty() {
        return Err(anyhow!("at least one domain suffix is required"));
    }
    Ok(domain_suffixes)
}
