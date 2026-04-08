use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration as StdDuration;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::{routing::get, Router};
use reqwest::Client;
use sdkwork_api_app_credential::{
    persist_credential_with_secret_and_manager, resolve_provider_secret_with_manager,
    CredentialSecretManager,
};
use sdkwork_api_app_gateway::reload_configured_extension_host;
use sdkwork_api_app_runtime::{
    create_extension_runtime_rollout, create_standalone_config_rollout,
    find_extension_runtime_rollout, find_standalone_config_rollout,
    start_extension_runtime_rollout_supervision, start_standalone_runtime_supervision,
    CreateStandaloneConfigRolloutRequest, StandaloneListenerHost, StandaloneServiceKind,
    StandaloneServiceReloadHandles,
};
use sdkwork_api_config::{CacheBackendKind, StandaloneConfigLoader};
use sdkwork_api_domain_billing::{PricingPlanRecord, PricingRateRecord};
use sdkwork_api_domain_catalog::{Channel, ModelCatalogEntry, ProxyProvider};
use sdkwork_api_ext_provider_native_mock::FIXTURE_EXTENSION_ID;
use sdkwork_api_extension_core::{
    CompatibilityLevel, ExtensionKind, ExtensionManifest, ExtensionPermission, ExtensionProtocol,
    ExtensionRuntime,
};
use sdkwork_api_extension_host::shutdown_all_native_dynamic_runtimes;
use sdkwork_api_storage_core::{
    AccountKernelStore, AdminStore, ExtensionRuntimeRolloutParticipantRecord,
    ExtensionRuntimeRolloutRecord, Reloadable, ServiceRuntimeNodeRecord,
    StandaloneConfigRolloutParticipantRecord, StandaloneConfigRolloutRecord,
};
use sdkwork_api_storage_sqlite::{run_migrations, SqliteAdminStore};
use serial_test::serial;
use tokio::time::{sleep, Duration};

mod cluster_rollouts;
mod listener_reload;
mod pricing_and_secrets;
mod support;

use support::*;
