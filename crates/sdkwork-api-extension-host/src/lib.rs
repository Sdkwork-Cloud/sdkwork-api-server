use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::io;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::os::raw::{c_char, c_void};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Condvar, Mutex, OnceLock};
use std::time::{Duration, Instant};

use anyhow::{Result as AnyhowResult, anyhow};
use async_trait::async_trait;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use bytes::Bytes;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use futures_util::StreamExt;
use futures_util::stream;
use libloading::Library;
use sdkwork_api_extension_abi::{
    ExtensionHealthCheckResult, ExtensionLifecycleContext, ExtensionLifecycleResult,
    ProviderInvocation, ProviderInvocationResult, ProviderStreamInvocationResult,
    ProviderStreamWriter, SDKWORK_EXTENSION_ABI_VERSION, SDKWORK_EXTENSION_ABI_VERSION_SYMBOL,
    SDKWORK_EXTENSION_FREE_STRING_SYMBOL, SDKWORK_EXTENSION_HEALTH_CHECK_JSON_SYMBOL,
    SDKWORK_EXTENSION_INIT_JSON_SYMBOL, SDKWORK_EXTENSION_MANIFEST_JSON_SYMBOL,
    SDKWORK_EXTENSION_PROVIDER_EXECUTE_JSON_SYMBOL,
    SDKWORK_EXTENSION_PROVIDER_EXECUTE_STREAM_JSON_SYMBOL, SDKWORK_EXTENSION_SHUTDOWN_JSON_SYMBOL,
    free_raw_c_string, from_raw_c_str, into_raw_c_string,
};
use sdkwork_api_extension_core::{
    ExtensionInstallation, ExtensionInstance, ExtensionManifest, ExtensionRuntime,
    ExtensionSignatureAlgorithm,
};
use sdkwork_api_provider_core::{
    ProviderAdapter, ProviderExecutionAdapter, ProviderOutput, ProviderRequest,
    ProviderRequestOptions, ProviderStreamOutput,
};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;
use sha2::{Digest, Sha256};
use tokio::sync::mpsc::{self, UnboundedSender};
use tokio_stream::wrappers::UnboundedReceiverStream;

mod host_types;
mod extension_discovery;
mod extension_trust;
mod connector_runtime;
mod native_dynamic_runtime;
mod host_impl;
mod provider_invocation;
mod errors;

#[cfg(test)]
mod tests;

use host_types::{
    provider_runtime_aliases, AbiVersionFn, ConnectorLaunchConfig, ExecuteJsonFn,
    ExecuteStreamJsonFn, FreeStringFn, HostStreamWriterContext, LifecycleJsonFn,
    ManifestJsonFn,
    ManagedConnectorProcess, NativeDynamicLifecycleState, NativeDynamicProviderAdapter,
    NativeDynamicRuntime, NativeDynamicStreamEvent,
    SDKWORK_NATIVE_DYNAMIC_SHUTDOWN_DRAIN_TIMEOUT_MS, CONNECTOR_PROCESS_REGISTRY,
    NATIVE_DYNAMIC_RUNTIME_REGISTRY,
};
use connector_runtime::{native_dynamic_runtime_registry, resolve_entrypoint};
use native_dynamic_runtime::{
    ensure_native_dynamic_manifest_matches, load_or_reuse_native_dynamic_runtime, merge_config,
};
use provider_invocation::{
    provider_invocation_from_request, provider_invocation_from_request_with_options,
};

pub use connector_runtime::{
    ensure_connector_runtime_started, list_connector_runtime_statuses,
    shutdown_all_connector_runtimes, shutdown_connector_runtime,
    shutdown_connector_runtimes_for_extension,
};
pub use errors::ExtensionHostError;
pub use extension_discovery::{
    discover_extension_packages, validate_discovered_extension_package, validate_extension_manifest,
};
pub use extension_trust::verify_discovered_extension_package_trust;
pub use host_types::{
    BuiltinExtensionFactory, BuiltinProviderExtensionFactory, ConnectorRuntimeStatus,
    DiscoveredExtensionPackage, ExtensionDiscoveryPolicy, ExtensionHost, ExtensionLoadPlan,
    ExtensionTrustIssue, ExtensionTrustReport, ExtensionTrustState, ManifestValidationIssue,
    ManifestValidationReport, ManifestValidationSeverity, NativeDynamicRuntimeStatus,
};
pub use native_dynamic_runtime::{
    list_native_dynamic_runtime_statuses, load_native_dynamic_library_manifest,
    load_native_dynamic_provider_adapter, shutdown_all_native_dynamic_runtimes,
    shutdown_native_dynamic_runtimes_for_extension,
};
