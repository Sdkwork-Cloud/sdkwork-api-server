use axum::extract::State;
use axum::routing::{get, post};
use axum::{Json, Router};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use ed25519_dalek::SigningKey;
use sdkwork_api_app_credential::{
    persist_credential_with_secret_and_manager, CredentialSecretManager,
};
use sdkwork_api_app_gateway::{
    builtin_extension_host, execute_json_provider_request_with_runtime,
    planned_execution_provider_context_for_route_without_log, relay_chat_completion_from_store,
    reload_configured_extension_host, reload_extension_host_with_scope,
    start_configured_extension_hot_reload_supervision, ConfiguredExtensionHostReloadScope,
};
use sdkwork_api_contract_openai::chat_completions::{
    ChatMessageInput, CreateChatCompletionRequest,
};
use sdkwork_api_domain_catalog::{Channel, ModelCatalogEntry, ProxyProvider};
use sdkwork_api_ext_provider_native_mock::FIXTURE_EXTENSION_ID;
use sdkwork_api_extension_core::{
    ExtensionInstallation, ExtensionInstance, ExtensionRuntime, ExtensionSignature,
    ExtensionSignatureAlgorithm, ExtensionTrustDeclaration,
};
use sdkwork_api_extension_host::{
    discover_extension_packages, load_native_dynamic_provider_adapter,
    shutdown_all_native_dynamic_runtimes, ExtensionDiscoveryPolicy,
};
use sdkwork_api_provider_core::ProviderRequest;
use sdkwork_api_storage_sqlite::{run_migrations, SqliteAdminStore};
use serde_json::{json, Value};
use serial_test::serial;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::time::{sleep, Duration};

mod builtin_host;
mod connector_relay;
mod native_dynamic_reload;
mod support;

use support::*;
