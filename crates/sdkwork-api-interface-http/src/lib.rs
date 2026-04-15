#![allow(clippy::result_large_err)]

mod assistants_handlers;
mod assistants_stateless_handlers;
mod batches_handlers;
mod batches_stateless_handlers;
mod chat_completion_handlers;
mod chat_completion_stateless_handlers;
mod compat_anthropic;
mod compat_gemini;
mod compat_gemini_handlers;
mod compat_streaming;
mod conversation_handlers;
mod conversation_stateless_handlers;
mod eval_handlers;
mod eval_stateless_handlers;
mod fine_tuning_handlers;
mod fine_tuning_stateless_handlers;
mod gateway_auth;
mod gateway_commercial;
mod gateway_compat_handlers;
mod gateway_docs;
mod gateway_http;
mod gateway_market;
mod gateway_openapi;
mod gateway_payments;
mod gateway_prelude;
mod gateway_response_helpers;
mod gateway_router_common;
mod gateway_state;
mod gateway_stateful_route_groups;
mod gateway_stateful_router;
mod gateway_stateless_route_groups;
mod gateway_stateless_router;
mod gateway_usage;
mod inference_handlers;
mod inference_stateless_handlers;
mod models_handlers;
mod models_stateless_handlers;
mod multipart_parsers;
mod realtime_handlers;
mod realtime_stateless_handlers;
mod response_handlers;
mod response_stateless_handlers;
mod stateless_gateway;
mod stateless_relay;
mod storage_handlers;
mod storage_stateless_handlers;
mod thread_handlers;
mod thread_stateless_handlers;
mod vector_store_handlers;
mod vector_store_stateless_handlers;
mod video_handlers;
mod video_stateless_handlers;
mod webhooks_handlers;
mod webhooks_stateless_handlers;

use gateway_prelude::*;

pub use gateway_state::GatewayApiState;
pub use stateless_gateway::{StatelessGatewayConfig, StatelessGatewayUpstream};

pub fn try_gateway_router() -> anyhow::Result<Router> {
    try_gateway_router_with_stateless_config(StatelessGatewayConfig::default())
}

pub fn gateway_router() -> Router {
    try_gateway_router().expect("http exposure config should load from process env")
}

pub fn try_gateway_router_with_stateless_config(
    config: StatelessGatewayConfig,
) -> anyhow::Result<Router> {
    Ok(gateway_router_with_stateless_config_and_http_exposure(
        config,
        gateway_http::http_exposure_config()?,
    ))
}

pub fn gateway_router_with_stateless_config(config: StatelessGatewayConfig) -> Router {
    try_gateway_router_with_stateless_config(config)
        .expect("http exposure config should load from process env")
}

pub fn gateway_router_with_stateless_config_and_http_exposure(
    config: StatelessGatewayConfig,
    http_exposure: sdkwork_api_config::HttpExposureConfig,
) -> Router {
    build_stateless_gateway_router_with_http_exposure(config, http_exposure)
}

pub fn try_gateway_router_with_pool(pool: SqlitePool) -> anyhow::Result<Router> {
    try_gateway_router_with_pool_and_master_key(pool, "local-dev-master-key")
}

pub fn gateway_router_with_pool(pool: SqlitePool) -> Router {
    try_gateway_router_with_pool(pool).expect("http exposure config should load from process env")
}

pub fn try_gateway_router_with_store(store: Arc<dyn AdminStore>) -> anyhow::Result<Router> {
    try_gateway_router_with_store_and_secret_manager(
        store,
        CredentialSecretManager::database_encrypted("local-dev-master-key"),
    )
}

pub fn gateway_router_with_store(store: Arc<dyn AdminStore>) -> Router {
    try_gateway_router_with_store(store).expect("http exposure config should load from process env")
}

pub fn try_gateway_router_with_pool_and_master_key(
    pool: SqlitePool,
    credential_master_key: impl Into<String>,
) -> anyhow::Result<Router> {
    try_gateway_router_with_state(GatewayApiState::with_master_key(
        pool,
        credential_master_key,
    ))
}

pub fn gateway_router_with_pool_and_master_key(
    pool: SqlitePool,
    credential_master_key: impl Into<String>,
) -> Router {
    try_gateway_router_with_pool_and_master_key(pool, credential_master_key)
        .expect("http exposure config should load from process env")
}

pub fn try_gateway_router_with_pool_and_secret_manager(
    pool: SqlitePool,
    secret_manager: CredentialSecretManager,
) -> anyhow::Result<Router> {
    try_gateway_router_with_state(GatewayApiState::with_secret_manager(pool, secret_manager))
}

pub fn gateway_router_with_pool_and_secret_manager(
    pool: SqlitePool,
    secret_manager: CredentialSecretManager,
) -> Router {
    try_gateway_router_with_pool_and_secret_manager(pool, secret_manager)
        .expect("http exposure config should load from process env")
}

pub fn try_gateway_router_with_store_and_secret_manager(
    store: Arc<dyn AdminStore>,
    secret_manager: CredentialSecretManager,
) -> anyhow::Result<Router> {
    try_gateway_router_with_state(GatewayApiState::with_store_and_secret_manager(
        store,
        secret_manager,
    ))
}

pub fn gateway_router_with_store_and_secret_manager(
    store: Arc<dyn AdminStore>,
    secret_manager: CredentialSecretManager,
) -> Router {
    try_gateway_router_with_store_and_secret_manager(store, secret_manager)
        .expect("http exposure config should load from process env")
}

pub fn try_gateway_router_with_state(state: GatewayApiState) -> anyhow::Result<Router> {
    Ok(gateway_router_with_state_and_http_exposure(
        state,
        gateway_http::http_exposure_config()?,
    ))
}

pub fn gateway_router_with_state(state: GatewayApiState) -> Router {
    try_gateway_router_with_state(state).expect("http exposure config should load from process env")
}

pub fn gateway_router_with_state_and_http_exposure(
    state: GatewayApiState,
    http_exposure: sdkwork_api_config::HttpExposureConfig,
) -> Router {
    build_stateful_gateway_router_with_http_exposure(state, http_exposure)
}
