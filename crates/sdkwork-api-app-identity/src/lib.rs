use anyhow::{anyhow, Result};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use sdkwork_api_domain_identity::GatewayApiKeyRecord;
use sdkwork_api_storage_core::AdminStore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

const ADMIN_JWT_ISSUER: &str = "sdkwork-admin";
const ADMIN_JWT_AUDIENCE: &str = "sdkwork-admin-ui";
const ADMIN_JWT_TTL_SECS: u64 = 60 * 60 * 12;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub iss: String,
    pub aud: String,
    pub exp: usize,
    pub iat: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GatewayRequestContext {
    pub tenant_id: String,
    pub project_id: String,
    pub environment: String,
}

impl GatewayRequestContext {
    pub fn tenant_id(&self) -> &str {
        &self.tenant_id
    }

    pub fn project_id(&self) -> &str {
        &self.project_id
    }
}

pub fn hash_gateway_api_key(value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub fn issue_jwt(subject: &str, signing_secret: &str) -> Result<String> {
    let issued_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| anyhow!("system clock error"))?
        .as_secs();
    let claims = Claims {
        sub: subject.to_owned(),
        iss: ADMIN_JWT_ISSUER.to_owned(),
        aud: ADMIN_JWT_AUDIENCE.to_owned(),
        exp: (issued_at + ADMIN_JWT_TTL_SECS) as usize,
        iat: issued_at as usize,
    };
    Ok(encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(signing_secret.as_bytes()),
    )?)
}

pub fn verify_jwt(token: &str, signing_secret: &str) -> Result<Claims> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.set_audience(&[ADMIN_JWT_AUDIENCE]);
    validation.set_issuer(&[ADMIN_JWT_ISSUER]);
    Ok(decode::<Claims>(
        token,
        &DecodingKey::from_secret(signing_secret.as_bytes()),
        &validation,
    )?
    .claims)
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

pub async fn resolve_gateway_request_context(
    store: &dyn AdminStore,
    plaintext_key: &str,
) -> Result<Option<GatewayRequestContext>> {
    let hashed_key = hash_gateway_api_key(plaintext_key);
    let Some(record) = store.find_gateway_api_key(&hashed_key).await? else {
        return Ok(None);
    };

    if !record.active {
        return Ok(None);
    }

    Ok(Some(GatewayRequestContext {
        tenant_id: record.tenant_id,
        project_id: record.project_id,
        environment: record.environment,
    }))
}
