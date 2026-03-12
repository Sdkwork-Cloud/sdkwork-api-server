use anyhow::{anyhow, Result};
use base64::{engine::general_purpose::STANDARD, Engine as _};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecretEnvelope {
    pub ciphertext: String,
    pub key_version: u32,
}

pub fn encrypt(master_key: &str, plaintext: &str) -> Result<SecretEnvelope> {
    let payload = format!("{master_key}:{plaintext}");
    Ok(SecretEnvelope {
        ciphertext: STANDARD.encode(payload),
        key_version: 1,
    })
}

pub fn decrypt(master_key: &str, envelope: &SecretEnvelope) -> Result<String> {
    let decoded = STANDARD.decode(&envelope.ciphertext)?;
    let decoded = String::from_utf8(decoded)?;
    let prefix = format!("{master_key}:");
    decoded
        .strip_prefix(&prefix)
        .map(ToOwned::to_owned)
        .ok_or_else(|| anyhow!("invalid master key"))
}
