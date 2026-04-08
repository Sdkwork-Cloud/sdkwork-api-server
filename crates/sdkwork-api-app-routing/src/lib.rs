use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Result, ensure};
use sdkwork_api_app_extension::{
    ExtensionRuntimeStatusRecord, list_extension_runtime_statuses,
    matching_runtime_statuses_for_provider,
};
use sdkwork_api_cache_core::DistributedLockStore;
use sdkwork_api_domain_catalog::ProxyProvider;
use sdkwork_api_domain_routing::{
    CompiledRoutingSnapshotRecord, ProjectRoutingPreferences, ProviderHealthRecoveryProbe,
    ProviderHealthRecoveryProbeOutcome, ProviderHealthSnapshot, RoutingCandidateAssessment,
    RoutingCandidateHealth, RoutingDecision, RoutingDecisionLog, RoutingDecisionSource,
    RoutingPolicy, RoutingProfileRecord, RoutingStrategy, select_policy,
};
use sdkwork_api_extension_core::{ExtensionInstallation, ExtensionInstance};
use sdkwork_api_policy_routing::{
    RoutingStrategyExecutionInput, RoutingStrategyExecutionResult,
    builtin_routing_strategy_registry,
};
use sdkwork_api_storage_core::AdminStore;
use serde_json::Value;

const DEFAULT_WEIGHT: u64 = 100;
const STRATEGY_STATIC_FALLBACK: &str = "static_fallback";
const DEFAULT_PERSISTED_PROVIDER_HEALTH_FRESHNESS_TTL_MS: u64 = 60_000;
const PROVIDER_HEALTH_FRESHNESS_TTL_ENV: &str = "SDKWORK_ROUTING_PROVIDER_HEALTH_FRESHNESS_TTL_MS";
const DEFAULT_PROVIDER_HEALTH_RECOVERY_PROBE_PERCENT: u8 = 5;
const PROVIDER_HEALTH_RECOVERY_PROBE_PERCENT_ENV: &str =
    "SDKWORK_ROUTING_PROVIDER_HEALTH_RECOVERY_PROBE_PERCENT";
const DEFAULT_PROVIDER_HEALTH_RECOVERY_PROBE_LOCK_TTL_MS: u64 = 30_000;
const PROVIDER_HEALTH_RECOVERY_PROBE_LOCK_TTL_ENV: &str =
    "SDKWORK_ROUTING_PROVIDER_HEALTH_RECOVERY_PROBE_LOCK_TTL_MS";
const PROVIDER_HEALTH_RECOVERY_PROBE_FALLBACK_REASON: &str = "provider_health_recovery_probe";


mod route_inputs;
mod route_management;
mod route_selection;
mod candidate_selection;
mod routing_support;

pub use route_inputs::*;
pub use route_management::*;
pub use route_selection::*;

pub(crate) use candidate_selection::*;
pub(crate) use routing_support::*;
