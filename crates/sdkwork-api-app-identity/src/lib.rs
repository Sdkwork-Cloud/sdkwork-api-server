use anyhow::{anyhow, Result};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use sdkwork_api_domain_identity::GatewayApiKeyRecord;
use sdkwork_api_storage_core::AdminStore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
}

pub fn hash_gateway_api_key(value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub fn issue_jwt(subject: &str) -> Result<String> {
    let claims = Claims {
        sub: subject.to_owned(),
    };
    let payload = serde_json::to_vec(&claims)?;
    Ok(format!("sdkwork.{}", URL_SAFE_NO_PAD.encode(payload)))
}

pub fn verify_jwt(token: &str) -> Result<Claims> {
    let payload = token
        .strip_prefix("sdkwork.")
        .ok_or_else(|| anyhow!("invalid token prefix"))?;
    let decoded = URL_SAFE_NO_PAD.decode(payload)?;
    Ok(serde_json::from_slice(&decoded)?)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreatedGatewayApiKey {
    pub plaintext: String,
    pub hashed: String,
    pub tenant_id: String,
    pub project_id: String,
    pub environment: String,
}

pub struct CreateGatewayApiKey;

impl CreateGatewayApiKey {
    pub fn execute(
        tenant_id: &str,
        project_id: &str,
        environment: &str,
    ) -> Result<CreatedGatewayApiKey> {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| anyhow!("system clock error"))?
            .as_nanos();
        let plaintext = format!("skw_{environment}_{nonce:x}");
        let hashed = hash_gateway_api_key(&plaintext);
        Ok(CreatedGatewayApiKey {
            plaintext,
            hashed,
            tenant_id: tenant_id.to_owned(),
            project_id: project_id.to_owned(),
            environment: environment.to_owned(),
        })
    }
}

pub async fn persist_gateway_api_key(
    store: &dyn AdminStore,
    tenant_id: &str,
    project_id: &str,
    environment: &str,
) -> Result<CreatedGatewayApiKey> {
    let created = CreateGatewayApiKey::execute(tenant_id, project_id, environment)?;
    let record =
        GatewayApiKeyRecord::new(tenant_id, project_id, environment, created.hashed.clone());
    store.insert_gateway_api_key(&record).await?;
    Ok(created)
}

pub async fn list_gateway_api_keys(store: &dyn AdminStore) -> Result<Vec<GatewayApiKeyRecord>> {
    store.list_gateway_api_keys().await
}
