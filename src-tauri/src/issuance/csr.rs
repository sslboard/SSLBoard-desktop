use std::fs;

use anyhow::{Result, anyhow};
use x509_parser::oid_registry::{
    OID_EC_P256, OID_KEY_TYPE_EC_PUBLIC_KEY, OID_NIST_EC_P384, OID_PKCS1_RSAENCRYPTION,
};
use x509_parser::{
    certification_request::X509CertificationRequest,
    extensions::{GeneralName, ParsedExtension},
    prelude::FromDer,
};

use crate::{
    core::types::{CsrMetadata, KeyAlgorithm, KeyCurve},
    domain::normalize_domain_for_storage,
};

pub struct ParsedCsr {
    pub der: Vec<u8>,
    pub metadata: CsrMetadata,
    pub identifiers: Vec<String>,
    pub warnings: Vec<String>,
}

pub fn load_and_validate_csr(path: &str) -> Result<ParsedCsr> {
    let bytes = fs::read(path).map_err(|err| anyhow!("Failed to read CSR file: {err}"))?;
    parse_and_validate_csr_bytes(&bytes)
}

pub fn parse_and_validate_csr_bytes(bytes: &[u8]) -> Result<ParsedCsr> {
    let der = parse_csr_der(bytes)?;
    let (_, csr) = X509CertificationRequest::from_der(&der)
        .map_err(|_| anyhow!("CSR is not valid DER data"))?;

    csr.verify_signature()
        .map_err(|err| anyhow!("CSR signature validation failed: {err}"))?;

    let (key_algorithm, key_size, key_curve) = resolve_csr_key_params(&csr)?;
    let subject = csr.certification_request_info.subject.to_string();
    let common_name = csr
        .certification_request_info
        .subject
        .iter_common_name()
        .next()
        .and_then(|cn| cn.as_str().ok())
        .map(|cn| cn.to_string());

    let sans = extract_dns_sans(&csr)?;
    let (identifiers, warnings) = derive_identifiers(sans.clone(), common_name.as_deref())?;

    Ok(ParsedCsr {
        der,
        metadata: CsrMetadata {
            subject,
            sans,
            key_algorithm,
            key_size,
            key_curve,
        },
        identifiers,
        warnings,
    })
}

fn parse_csr_der(bytes: &[u8]) -> Result<Vec<u8>> {
    if let Ok(text) = std::str::from_utf8(bytes)
        && let Ok(pem) = pem::parse(text) {
        if !pem.tag().contains("REQUEST") {
            return Err(anyhow!("CSR PEM must have CERTIFICATE REQUEST tag"));
        }
        return Ok(pem.contents().to_vec());
    }

    Ok(bytes.to_vec())
}

fn extract_dns_sans(csr: &X509CertificationRequest<'_>) -> Result<Vec<String>> {
    let mut sans = Vec::new();
    if let Some(extensions) = csr.requested_extensions() {
        for ext in extensions {
            if let ParsedExtension::SubjectAlternativeName(san) = ext {
                for name in &san.general_names {
                    if let GeneralName::DNSName(dns) = name {
                        sans.push(dns.to_string());
                    }
                }
            }
        }
    }

    let mut normalized = Vec::new();
    for name in sans {
        let ascii = normalize_domain_for_storage(&name)
            .map_err(|err| anyhow!("Invalid SAN entry \"{name}\": {err}"))?;
        if !ascii.is_empty() {
            normalized.push(ascii);
        }
    }
    normalized.sort();
    normalized.dedup();
    Ok(normalized)
}

fn derive_identifiers(sans: Vec<String>, common_name: Option<&str>) -> Result<(Vec<String>, Vec<String>)> {
    if !sans.is_empty() {
        return Ok((sans, Vec::new()));
    }

    let Some(cn) = common_name else {
        return Err(anyhow!("CSR must include at least one DNS SAN or a Common Name"));
    };
    let normalized = normalize_domain_for_storage(cn)
        .map_err(|err| anyhow!("Invalid CSR common name \"{cn}\": {err}"))?;
    let warning = "CSR has no SAN entries; falling back to CN-only issuance.".to_string();
    Ok((vec![normalized], vec![warning]))
}

fn resolve_csr_key_params(csr: &X509CertificationRequest<'_>) -> Result<(KeyAlgorithm, Option<u16>, Option<KeyCurve>)> {
    let spki = &csr.certification_request_info.subject_pki;
    if spki.algorithm.algorithm == OID_PKCS1_RSAENCRYPTION {
        let key_size = spki
            .parsed()
            .map(|key| key.key_size())
            .map_err(|_| anyhow!("Unable to parse RSA public key"))?;
        let key_size = u16::try_from(key_size)
            .map_err(|_| anyhow!("RSA key size is too large"))?;
        if !matches!(key_size, 2048 | 3072 | 4096) {
            return Err(anyhow!(
                "Unsupported RSA key size {key_size}. Allowed: 2048, 3072, 4096"
            ));
        }
        return Ok((KeyAlgorithm::Rsa, Some(key_size), None));
    }

    if spki.algorithm.algorithm == OID_KEY_TYPE_EC_PUBLIC_KEY {
        let curve_oid = spki
            .algorithm
            .parameters
            .as_ref()
            .and_then(|params| params.as_oid().ok())
            .ok_or_else(|| anyhow!("ECDSA curve parameters missing"))?;
        if curve_oid == OID_EC_P256 {
            return Ok((KeyAlgorithm::Ecdsa, None, Some(KeyCurve::P256)));
        }
        if curve_oid == OID_NIST_EC_P384 {
            return Ok((KeyAlgorithm::Ecdsa, None, Some(KeyCurve::P384)));
        }
        return Err(anyhow!("Unsupported ECDSA curve in CSR"));
    }

    Err(anyhow!("Unsupported CSR key algorithm"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::issuance::acme_workflow;

    #[test]
    fn parses_generated_csr_with_sans() {
        let key = acme_workflow::generate_private_key(&KeyAlgorithm::Rsa, Some(2048), None)
            .expect("key");
        let csr_der = acme_workflow::build_csr_der(
            &key,
            &vec!["example.com".to_string(), "www.example.com".to_string()],
        )
        .expect("csr");

        let parsed = parse_and_validate_csr_bytes(&csr_der).expect("parsed");
        assert_eq!(parsed.identifiers.len(), 1);
        assert_eq!(parsed.identifiers[0], "www.example.com");
        assert!(parsed.warnings.is_empty());
    }

    #[test]
    fn warns_on_cn_only_csr() {
        let key = acme_workflow::generate_private_key(&KeyAlgorithm::Rsa, Some(2048), None)
            .expect("key");
        let csr_der = acme_workflow::build_csr_der_with_subject(&key, "example.com", &[])
            .expect("csr");

        let parsed = parse_and_validate_csr_bytes(&csr_der).expect("parsed");
        assert_eq!(parsed.identifiers, vec!["example.com".to_string()]);
        assert_eq!(parsed.warnings.len(), 1);
    }
}
