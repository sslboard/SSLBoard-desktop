use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Component, Path},
};

use anyhow::{anyhow, Context, Result};
use pem::Pem;

use crate::core::types::{ExportBundle, ExportCertificateResponse, ExportedFile};

#[cfg(unix)]
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
#[cfg(not(unix))]
use log::warn;

const CERT_FILENAME: &str = "cert.pem";
const CHAIN_FILENAME: &str = "chain.pem";
const FULLCHAIN_FILENAME: &str = "fullchain.pem";
const PRIVKEY_FILENAME: &str = "privkey.pem";

pub struct ExportOptions<'a> {
    pub destination_dir: &'a str,
    pub folder_name: &'a str,
    pub include_private_key: bool,
    pub overwrite: bool,
    pub bundle: ExportBundle,
}

pub fn export_pem_bundle(
    chain_pem: &str,
    private_key_pem: Option<&str>,
    options: ExportOptions<'_>,
) -> Result<ExportCertificateResponse> {
    validate_folder_name(options.folder_name)?;
    let output_dir = Path::new(options.destination_dir).join(options.folder_name);
    fs::create_dir_all(&output_dir).with_context(|| {
        format!(
            "failed to create export directory at {}",
            output_dir.display()
        )
    })?;

    let _selected_bundle = options.bundle;
    let (leaf_pem, chain_only_pem, fullchain_pem) = split_certificate_chain(chain_pem)?;

    let mut target_files = vec![
        output_dir.join(CERT_FILENAME),
        output_dir.join(CHAIN_FILENAME),
        output_dir.join(FULLCHAIN_FILENAME),
    ];
    if options.include_private_key {
        target_files.push(output_dir.join(PRIVKEY_FILENAME));
    }

    let existing: Vec<String> = target_files
        .iter()
        .filter(|path| path.exists())
        .map(|path| path.display().to_string())
        .collect();

    if !existing.is_empty() && !options.overwrite {
        return Ok(ExportCertificateResponse::OverwriteRequired {
            output_dir: output_dir.display().to_string(),
            existing_files: existing,
        });
    }

    write_secure_file(&output_dir.join(CERT_FILENAME), leaf_pem.as_bytes(), options.overwrite)?;
    write_secure_file(
        &output_dir.join(CHAIN_FILENAME),
        chain_only_pem.as_bytes(),
        options.overwrite,
    )?;
    write_secure_file(
        &output_dir.join(FULLCHAIN_FILENAME),
        fullchain_pem.as_bytes(),
        options.overwrite,
    )?;

    if options.include_private_key {
        let key_pem = private_key_pem.ok_or_else(|| {
            anyhow!("private key export requested but no key material was provided")
        })?;
        write_secure_file(
            &output_dir.join(PRIVKEY_FILENAME),
            key_pem.as_bytes(),
            options.overwrite,
        )?;
    }

    let mut files = vec![
        ExportedFile {
            label: "cert".to_string(),
            path: output_dir.join(CERT_FILENAME).display().to_string(),
        },
        ExportedFile {
            label: "chain".to_string(),
            path: output_dir.join(CHAIN_FILENAME).display().to_string(),
        },
        ExportedFile {
            label: "fullchain".to_string(),
            path: output_dir.join(FULLCHAIN_FILENAME).display().to_string(),
        },
    ];
    if options.include_private_key {
        files.push(ExportedFile {
            label: "privkey".to_string(),
            path: output_dir.join(PRIVKEY_FILENAME).display().to_string(),
        });
    }

    Ok(ExportCertificateResponse::Success {
        output_dir: output_dir.display().to_string(),
        files,
    })
}

fn split_certificate_chain(chain_pem: &str) -> Result<(String, String, String)> {
    let blocks = pem::parse_many(chain_pem)
        .map_err(|err| anyhow!("failed to parse certificate chain PEM: {err}"))?;
    let cert_blocks: Vec<Pem> = blocks
        .into_iter()
        .filter(|block| block.tag() == "CERTIFICATE")
        .collect();

    if cert_blocks.is_empty() {
        return Err(anyhow!("no certificate PEM blocks found"));
    }

    let encoded: Vec<String> = cert_blocks.into_iter().map(|block| pem::encode(&block)).collect();
    let leaf = encoded[0].clone();
    if encoded.len() < 2 {
        return Err(anyhow!("issuer chain is missing; cannot write chain.pem"));
    }
    let chain_only = encoded[1..].join("");
    let fullchain = encoded.join("");
    Ok((leaf, chain_only, fullchain))
}

fn validate_folder_name(name: &str) -> Result<()> {
    if name.trim().is_empty() {
        return Err(anyhow!("folder name cannot be empty"));
    }
    let mut components = Path::new(name).components();
    match (components.next(), components.next()) {
        (Some(Component::Normal(_)), None) => Ok(()),
        _ => Err(anyhow!("folder name must be a single path segment")),
    }
}

fn write_secure_file(path: &Path, content: &[u8], overwrite: bool) -> Result<()> {
    let mut options = OpenOptions::new();
    options.write(true);
    if overwrite {
        options.create(true).truncate(true);
    } else {
        options.create_new(true);
    }
    #[cfg(unix)]
    {
        options.mode(0o600);
    }
    let mut file = options
        .open(path)
        .with_context(|| format!("failed to open export file {}", path.display()))?;
    file.write_all(content)
        .with_context(|| format!("failed to write {}", path.display()))?;
    file.flush()
        .with_context(|| format!("failed to flush {}", path.display()))?;
    ensure_permissions(path)?;
    Ok(())
}

fn ensure_permissions(path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        let desired = fs::Permissions::from_mode(0o600);
        let metadata = fs::metadata(path)?;
        let current = metadata.permissions();
        if current.mode() & 0o777 != 0o600 {
            fs::set_permissions(path, desired).with_context(|| {
                format!(
                    "failed to set restrictive permissions on {}",
                    path.display()
                )
            })?;
        }
    }
    #[cfg(not(unix))]
    {
        if let Err(err) = fs::metadata(path) {
            warn!(
                "[export] warning: unable to confirm permissions for {}: {}",
                path.display(),
                err
            );
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rcgen::{BasicConstraints, CertificateParams, IsCa, KeyPair};
    use std::path::PathBuf;
    use uuid::Uuid;

    fn temp_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("sslboard-export-{}", Uuid::new_v4()));
        fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    fn sample_chain() -> (String, String) {
        let mut ca_params =
            CertificateParams::new(vec!["example.com".to_string()]).expect("ca params");
        ca_params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
        let ca_key = KeyPair::generate().expect("ca key");
        let ca = ca_params.self_signed(&ca_key).expect("create ca cert");

        let mut leaf_params =
            CertificateParams::new(vec!["example.com".to_string()]).expect("leaf params");
        leaf_params.is_ca = IsCa::NoCa;
        let leaf_key = KeyPair::generate().expect("leaf key");
        let leaf = leaf_params
            .signed_by(&leaf_key, &ca, &ca_key)
            .expect("create leaf cert");

        let leaf_pem = leaf.pem();
        let ca_pem = ca.pem();
        let chain_pem = format!("{leaf_pem}{ca_pem}");
        let key_pem = leaf_key.serialize_pem();
        (chain_pem, key_pem)
    }

    #[test]
    fn writes_expected_pem_files() {
        let (chain_pem, key_pem) = sample_chain();
        let dir = temp_dir();
        let options = ExportOptions {
            destination_dir: dir.to_str().expect("dir str"),
            folder_name: "example.com",
            include_private_key: true,
            overwrite: false,
            bundle: ExportBundle::Fullchain,
        };
        let result = export_pem_bundle(&chain_pem, Some(&key_pem), options)
            .expect("export pem");

        match result {
            ExportCertificateResponse::Success { output_dir, .. } => {
                let cert_path = Path::new(&output_dir).join(CERT_FILENAME);
                let chain_path = Path::new(&output_dir).join(CHAIN_FILENAME);
                let full_path = Path::new(&output_dir).join(FULLCHAIN_FILENAME);
                let key_path = Path::new(&output_dir).join(PRIVKEY_FILENAME);

                let cert_blocks = pem::parse_many(
                    &fs::read_to_string(cert_path).expect("read cert"),
                )
                .expect("parse cert");
                let chain_blocks = pem::parse_many(
                    &fs::read_to_string(chain_path).expect("read chain"),
                )
                .expect("parse chain");
                let full_blocks = pem::parse_many(
                    &fs::read_to_string(full_path).expect("read full"),
                )
                .expect("parse fullchain");
                let key_blocks = pem::parse_many(
                    &fs::read_to_string(key_path).expect("read key"),
                )
                .expect("parse key");

                assert_eq!(cert_blocks.len(), 1);
                assert_eq!(chain_blocks.len(), 1);
                assert_eq!(full_blocks.len(), 2);
                assert_eq!(key_blocks.len(), 1);
            }
            ExportCertificateResponse::OverwriteRequired { .. } => {
                panic!("unexpected overwrite requirement");
            }
        }
    }

    #[test]
    fn detects_existing_files_without_overwrite() {
        let (chain_pem, _) = sample_chain();
        let dir = temp_dir();
        let folder = dir.join("example.com");
        fs::create_dir_all(&folder).expect("create folder");
        fs::write(folder.join(CERT_FILENAME), "existing").expect("seed file");

        let options = ExportOptions {
            destination_dir: dir.to_str().expect("dir str"),
            folder_name: "example.com",
            include_private_key: false,
            overwrite: false,
            bundle: ExportBundle::Cert,
        };
        let result = export_pem_bundle(&chain_pem, None, options).expect("export");

        match result {
            ExportCertificateResponse::OverwriteRequired { existing_files, .. } => {
                assert!(!existing_files.is_empty());
            }
            ExportCertificateResponse::Success { .. } => {
                panic!("expected overwrite required");
            }
        }
    }

    #[test]
    fn exports_without_private_key() {
        let (chain_pem, _) = sample_chain();
        let dir = temp_dir();
        let options = ExportOptions {
            destination_dir: dir.to_str().expect("dir str"),
            folder_name: "no-key",
            include_private_key: false,
            overwrite: false,
            bundle: ExportBundle::Fullchain,
        };
        let result = export_pem_bundle(&chain_pem, None, options).expect("export");

        match result {
            ExportCertificateResponse::Success { output_dir, .. } => {
                let key_path = Path::new(&output_dir).join(PRIVKEY_FILENAME);
                assert!(!key_path.exists());
            }
            ExportCertificateResponse::OverwriteRequired { .. } => {
                panic!("unexpected overwrite requirement");
            }
        }
    }

    #[test]
    fn rejects_nested_folder_names() {
        let (chain_pem, _) = sample_chain();
        let dir = temp_dir();
        let options = ExportOptions {
            destination_dir: dir.to_str().expect("dir str"),
            folder_name: "../oops",
            include_private_key: false,
            overwrite: false,
            bundle: ExportBundle::Cert,
        };
        let err = export_pem_bundle(&chain_pem, None, options)
            .expect_err("expected error");
        assert!(err.to_string().contains("folder name"));
    }
}
