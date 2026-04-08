#[cfg(windows)]
use std::fs;
#[cfg(windows)]
use std::net::TcpListener;
#[cfg(windows)]
use std::path::{Path, PathBuf};
#[cfg(windows)]
use std::time::{SystemTime, UNIX_EPOCH};

use sdkwork_api_app_routing::RouteSelectionContext;
use sdkwork_api_app_routing::{
    persist_routing_policy, select_route_with_store, select_route_with_store_context,
    simulate_route, simulate_route_with_store, simulate_route_with_store_context,
    simulate_route_with_store_seeded, simulate_route_with_store_selection_context,
};
use sdkwork_api_cache_core::DistributedLockStore;
use sdkwork_api_cache_memory::MemoryCacheStore;
use sdkwork_api_domain_catalog::{Channel, ModelCatalogEntry, ProxyProvider};
use sdkwork_api_domain_identity::ApiKeyGroupRecord;
use sdkwork_api_domain_routing::{
    ProjectRoutingPreferences, ProviderHealthSnapshot, RoutingDecisionSource, RoutingPolicy,
    RoutingProfileRecord, RoutingStrategy,
};
use sdkwork_api_extension_core::{ExtensionInstallation, ExtensionInstance, ExtensionRuntime};
#[cfg(windows)]
use sdkwork_api_extension_host::{
    ensure_connector_runtime_started, load_native_dynamic_provider_adapter, ExtensionLoadPlan,
};
use sdkwork_api_extension_host::{
    shutdown_all_connector_runtimes, shutdown_all_native_dynamic_runtimes,
};
use sdkwork_api_storage_sqlite::{run_migrations, SqliteAdminStore};
use serial_test::serial;

mod basic_selection;
mod geo_slo_context;
mod provider_health;
mod runtime_backed;
mod support;

use support::*;
