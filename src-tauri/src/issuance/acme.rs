use anyhow::Result;
use instant_acme::Key;

pub fn generate_account_key_pem() -> Result<String> {
    let (_key, pkcs8) = Key::generate_pkcs8()?;
    let pkcs8_der: Vec<u8> = pkcs8.secret_pkcs8_der().to_vec();
    Ok(pem::encode(&pem::Pem::new("PRIVATE KEY", pkcs8_der)))
}
