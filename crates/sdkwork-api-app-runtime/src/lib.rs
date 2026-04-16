use anyhow::{Context, Result};
use async_trait::async_trait;
use axum::Router;
use futures_util::{StreamExt, TryStreamExt, stream};
use std::io;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use sdkwork_api_app_billing::{
    CommercialBillingAdminKernel, GatewayCommercialBillingKernel,
    synchronize_due_pricing_plan_lifecycle_with_report,
};
use sdkwork_api_app_credential::{CredentialSecretManager, resolve_credential_secret_with_manager};
use sdkwork_api_app_extension::{
    ExtensionDiscoveryPolicy, start_provider_health_snapshot_supervision,
};
use sdkwork_api_app_gateway::{
    ConfiguredExtensionHostReloadScope, reload_extension_host_with_policy,
    reload_extension_host_with_scope, start_configured_extension_hot_reload_supervision,
};
use sdkwork_api_cache_core::{CacheDriverFactory, CacheDriverRegistry, CacheRuntimeStores};
use sdkwork_api_cache_memory::MemoryCacheStore;
use sdkwork_api_cache_redis::RedisCacheStore;
use sdkwork_api_config::{
    CacheBackendKind, StandaloneConfig, StandaloneConfigLoader, StandaloneConfigWatchState,
    StandaloneRuntimeDynamicConfig,
};
use sdkwork_api_storage_core::{
    AdminStore, CommercialKernelStore, ExtensionRuntimeRolloutParticipantRecord,
    ExtensionRuntimeRolloutRecord, IdentityKernelStore, Reloadable, ServiceRuntimeNodeRecord,
    StandaloneConfigRolloutParticipantRecord, StandaloneConfigRolloutRecord, StorageDialect,
};
use sdkwork_api_storage_postgres::{PostgresAdminStore, run_migrations as run_postgres_migrations};
use sdkwork_api_storage_sqlite::{SqliteAdminStore, run_migrations as run_sqlite_migrations};
use tokio::net::TcpListener;
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;
use tokio::time::MissedTickBehavior;

pub(crate) const SERVICE_RUNTIME_NODE_HEARTBEAT_INTERVAL_MS: u64 = 2_000;

mod rollout_execution;
mod rollout_models;
mod runtime_builders;
mod bootstrap_data;
mod runtime_core;
mod runtime_reload;
mod standalone_listener;

#[cfg(test)]
mod tests;

pub(crate) use rollout_execution::{
    DEFAULT_EXTENSION_RUNTIME_ROLLOUT_TIMEOUT_SECS, DEFAULT_STANDALONE_CONFIG_ROLLOUT_TIMEOUT_SECS,
    NEXT_EXTENSION_RUNTIME_ROLLOUT_ID, NEXT_STANDALONE_CONFIG_ROLLOUT_ID,
    STANDALONE_CONFIG_ROLLOUT_NODE_FRESHNESS_WINDOW_MS,
};
pub(crate) use rollout_models::StandaloneRuntimeReloadOutcome;
pub(crate) use runtime_builders::{
    build_secret_manager_from_config, validate_secret_manager_for_store,
};
pub(crate) use runtime_core::{
    MemoryCacheStoreFactory, PendingStandaloneRuntimeRestartRequired, RedisCacheStoreFactory,
    StandaloneRuntimeState,
};
pub(crate) use runtime_reload::{
    AbortOnDropHandle, build_standalone_config_rollout_details, next_extension_runtime_rollout_id,
    next_standalone_config_rollout_id, normalize_extension_runtime_rollout_timeout_secs,
    normalize_standalone_config_rollout_timeout_secs,
    resolve_active_standalone_config_rollout_nodes, rollout_gateway_scope,
    rollout_request_fields_from_scope, rollout_scope_name, unix_timestamp_ms,
};

pub use rollout_execution::{
    create_extension_runtime_rollout, create_extension_runtime_rollout_with_request,
    create_standalone_config_rollout, find_extension_runtime_rollout,
    find_standalone_config_rollout, list_extension_runtime_rollouts,
    list_standalone_config_rollouts, resolve_service_runtime_node_id,
    start_extension_runtime_rollout_supervision,
};
pub use rollout_models::{
    CreateExtensionRuntimeRolloutRequest, CreateStandaloneConfigRolloutRequest,
    ExtensionRuntimeRolloutDetails, StandaloneConfigRolloutDetails,
};
pub use runtime_builders::{
    build_admin_payment_store_handles_from_config,
    build_admin_store_and_commercial_billing_from_config, build_admin_store_from_config,
    build_cache_runtime_from_config,
};
pub use sdkwork_api_app_billing::CommercialBillingReadKernel;
pub use runtime_core::{
    StandaloneRuntimeSupervision, StandaloneServiceKind, StandaloneServiceReloadHandles,
};
pub use runtime_reload::start_standalone_runtime_supervision;
pub use standalone_listener::{StandaloneListenerHandle, StandaloneListenerHost};

pub(crate) fn service_runtime_node_heartbeat_due(
    last_heartbeat_at_ms: Option<u64>,
    now_ms: u64,
) -> bool {
    match last_heartbeat_at_ms {
        None => true,
        Some(previous) => {
            now_ms.saturating_sub(previous) >= SERVICE_RUNTIME_NODE_HEARTBEAT_INTERVAL_MS
        }
    }
}
