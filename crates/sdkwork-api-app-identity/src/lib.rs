use anyhow::{anyhow, Result};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use hmac::{Hmac, Mac};
use jsonwebtoken::{
    crypto::{CryptoProvider, JwkUtils, JwtSigner, JwtVerifier},
    errors::{new_error, ErrorKind as JwtErrorKind, Result as JwtResult},
    signature::{Signer, Verifier},
    decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation,
};
use sdkwork_api_domain_billing::BillingAccountingMode;
use sdkwork_api_domain_identity::{
    AdminUserProfile, AdminUserRecord, AdminUserRole, ApiKeyGroupRecord, GatewayApiKeyRecord,
    GatewayAuthSubject, PortalUserProfile, PortalUserRecord,
};
use sdkwork_api_domain_tenant::{Project, Tenant};
use sdkwork_api_storage_core::{AdminStore, IdentityKernelStore};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sha2_010::Sha256 as JwtSha256;
use std::str::FromStr;
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};
use utoipa::ToSchema;

mod admin_users;
mod api_key_groups;
mod gateway_api_keys;
mod identity_support;
mod identity_types;
mod jwt_support;
mod portal_api_keys;
mod portal_users;

#[cfg(test)]
mod tests;

pub(crate) use identity_support::*;
pub(crate) use identity_types::{AdminResult, PortalResult};
pub(crate) use jwt_support::*;

pub use admin_users::*;
pub use api_key_groups::*;
pub use gateway_api_keys::*;
pub use identity_types::*;
pub use jwt_support::*;
pub use portal_api_keys::*;
pub use portal_users::*;
