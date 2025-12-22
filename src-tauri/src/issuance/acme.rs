use anyhow::Result;

pub fn generate_account_key_pem() -> Result<String> {
    let key = rcgen::KeyPair::generate()?;
    Ok(key.serialize_pem())
}
