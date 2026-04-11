use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use base64::{engine::general_purpose::STANDARD, Engine as _};
use ed25519_dalek::SigningKey;
use sdkwork_api_app_credential::{
    persist_credential_with_secret_and_manager, CredentialSecretManager,
};
use sdkwork_api_app_gateway::{
    execute_raw_json_provider_operation_from_planned_execution_context_with_options,
    execute_raw_stream_provider_operation_from_planned_execution_context_with_options,
    planned_execution_provider_context_for_route_without_log, PlannedExecutionProviderContext,
};
use sdkwork_api_app_routing::persist_routing_policy;
use sdkwork_api_domain_catalog::{Channel, ModelCatalogEntry, ProxyProvider};
use sdkwork_api_domain_routing::RoutingPolicy;
use sdkwork_api_ext_provider_native_mock::FIXTURE_EXTENSION_ID;
use sdkwork_api_extension_core::{
    CapabilityDescriptor, CompatibilityLevel, ExtensionInstallation, ExtensionInstance,
    ExtensionKind, ExtensionManifest, ExtensionModality, ExtensionPermission, ExtensionProtocol,
    ExtensionRuntime, ExtensionSignature, ExtensionSignatureAlgorithm, ExtensionTrustDeclaration,
};
use sdkwork_api_extension_host::shutdown_all_native_dynamic_runtimes;
use sdkwork_api_observability::HttpMetricsRegistry;
use sdkwork_api_provider_core::ProviderRequestOptions;
use sdkwork_api_storage_sqlite::{run_migrations, SqliteAdminStore};
use serde_json::{json, Value};
use serial_test::serial;
use sha2::{Digest, Sha256};

const MODEL_ID: &str = "claude-3-7-sonnet";
const PROVIDER_BASE_URL: &str = "https://native-dynamic.invalid/v1";
const JSON_DELAY_SEQUENCE_MS_ENV: &str = "SDKWORK_NATIVE_MOCK_JSON_DELAY_SEQUENCE_MS";
const JSON_RESULT_SEQUENCE_ENV: &str = "SDKWORK_NATIVE_MOCK_JSON_RESULT_SEQUENCE";
const STREAM_RESULT_SEQUENCE_ENV: &str = "SDKWORK_NATIVE_MOCK_STREAM_RESULT_SEQUENCE";

#[serial(extension_env)]
#[tokio::test]
async fn raw_json_planned_execution_persists_healthy_snapshot_on_success() {
    let fixture = raw_native_dynamic_fixture("json-success").await;

    let response = execute_raw_json_provider_operation_from_planned_execution_context_with_options(
        &fixture.store,
        &fixture.planned,
        "chat_completion",
        "anthropic.messages.create",
        vec![MODEL_ID.to_owned()],
        anthropic_payload(MODEL_ID),
        HashMap::new(),
        &ProviderRequestOptions::default(),
    )
    .await
    .unwrap()
    .expect("raw json response");

    assert_eq!(response["id"], "msg_native_dynamic");
    let snapshots = fixture
        .store
        .list_provider_health_snapshots()
        .await
        .unwrap();
    assert_eq!(snapshots.len(), 1);
    assert_eq!(snapshots[0].provider_id, fixture.provider_id);
    assert_eq!(snapshots[0].extension_id, FIXTURE_EXTENSION_ID);
    assert_eq!(snapshots[0].runtime, "native_dynamic");
    assert!(snapshots[0].running);
    assert!(snapshots[0].healthy);
    assert!(snapshots[0]
        .message
        .as_deref()
        .is_some_and(|message| message.contains("gateway execution succeeded")));

    let metrics = HttpMetricsRegistry::new("gateway").render_prometheus();
    assert!(metrics.contains(&format!(
        "sdkwork_upstream_requests_total{{service=\"gateway\",capability=\"chat_completion\",provider=\"{}\",outcome=\"attempt\"}} 1",
        fixture.provider_id
    )));
    assert!(metrics.contains(&format!(
        "sdkwork_upstream_requests_total{{service=\"gateway\",capability=\"chat_completion\",provider=\"{}\",outcome=\"success\"}} 1",
        fixture.provider_id
    )));
    assert!(metrics.contains(&format!(
        "sdkwork_provider_health_status{{service=\"gateway\",provider=\"{}\",runtime=\"native_dynamic\"}} 1",
        fixture.provider_id
    )));
}

#[serial(extension_env)]
#[tokio::test]
async fn raw_json_planned_execution_records_timeout_failure_and_unhealthy_snapshot() {
    let fixture = raw_native_dynamic_fixture("json-timeout").await;
    let _delay_guard = RawDelayGuard::json(200);

    let error = execute_raw_json_provider_operation_from_planned_execution_context_with_options(
        &fixture.store,
        &fixture.planned,
        "chat_completion",
        "anthropic.messages.create",
        vec![MODEL_ID.to_owned()],
        anthropic_payload(MODEL_ID),
        HashMap::new(),
        &ProviderRequestOptions::new().with_request_timeout_ms(25),
    )
    .await
    .expect_err("raw json timeout should fail");

    assert!(
        error.to_string().contains("timed out"),
        "unexpected timeout error: {error}"
    );

    let snapshots = fixture
        .store
        .list_provider_health_snapshots()
        .await
        .unwrap();
    assert_eq!(snapshots.len(), 1);
    assert_eq!(snapshots[0].provider_id, fixture.provider_id);
    assert!(!snapshots[0].healthy);
    assert!(snapshots[0]
        .message
        .as_deref()
        .is_some_and(|message| message.contains("timed out")));

    let metrics = HttpMetricsRegistry::new("gateway").render_prometheus();
    assert!(metrics.contains(&format!(
        "sdkwork_upstream_requests_total{{service=\"gateway\",capability=\"chat_completion\",provider=\"{}\",outcome=\"attempt\"}} 1",
        fixture.provider_id
    )));
    assert!(metrics.contains(&format!(
        "sdkwork_upstream_requests_total{{service=\"gateway\",capability=\"chat_completion\",provider=\"{}\",outcome=\"failure\"}} 1",
        fixture.provider_id
    )));
    assert!(metrics.contains(&format!(
        "sdkwork_gateway_execution_context_failures_total{{service=\"gateway\",capability=\"chat_completion\",provider=\"{}\",reason=\"request_timeout\"}} 1",
        fixture.provider_id
    )));
    assert!(metrics.contains(&format!(
        "sdkwork_provider_health_status{{service=\"gateway\",provider=\"{}\",runtime=\"native_dynamic\"}} 0",
        fixture.provider_id
    )));
}

#[serial(extension_env)]
#[tokio::test]
async fn raw_json_planned_execution_retries_timeout_and_succeeds_when_policy_allows() {
    let fixture = raw_native_dynamic_fixture_with_retry("json-timeout-retry", 2).await;
    let _delay_guard = RawDelayGuard::json_sequence("200,0");

    let response = execute_raw_json_provider_operation_from_planned_execution_context_with_options(
        &fixture.store,
        &fixture.planned,
        "chat_completion",
        "anthropic.messages.create",
        vec![MODEL_ID.to_owned()],
        anthropic_payload(MODEL_ID),
        HashMap::new(),
        &ProviderRequestOptions::new().with_request_timeout_ms(25),
    )
    .await
    .unwrap()
    .expect("raw json response after retry");

    assert_eq!(response["id"], "msg_native_dynamic");

    let snapshots = fixture
        .store
        .list_provider_health_snapshots()
        .await
        .unwrap();
    assert_eq!(snapshots.len(), 1);
    assert_eq!(snapshots[0].provider_id, fixture.provider_id);
    assert!(snapshots[0].healthy);

    let metrics = HttpMetricsRegistry::new("gateway").render_prometheus();
    assert!(metrics.contains(&format!(
        "sdkwork_upstream_requests_total{{service=\"gateway\",capability=\"chat_completion\",provider=\"{}\",outcome=\"attempt\"}} 2",
        fixture.provider_id
    )));
    assert!(metrics.contains(&format!(
        "sdkwork_upstream_requests_total{{service=\"gateway\",capability=\"chat_completion\",provider=\"{}\",outcome=\"success\"}} 1",
        fixture.provider_id
    )));
    assert!(metrics.contains(&format!(
        "sdkwork_gateway_execution_context_failures_total{{service=\"gateway\",capability=\"chat_completion\",provider=\"{}\",reason=\"request_timeout\"}} 1",
        fixture.provider_id
    )));
    assert!(metrics.contains(&format!(
        "sdkwork_upstream_retries_total{{service=\"gateway\",capability=\"chat_completion\",provider=\"{}\",outcome=\"scheduled\"}} 1",
        fixture.provider_id
    )));
    assert!(metrics.contains(&format!(
        "sdkwork_upstream_retry_reasons_total{{service=\"gateway\",capability=\"chat_completion\",provider=\"{}\",outcome=\"scheduled\",reason=\"execution_timeout\"}} 1",
        fixture.provider_id
    )));
}

#[serial(extension_env)]
#[tokio::test]
async fn raw_json_planned_execution_retries_retryable_plugin_error_and_succeeds() {
    let fixture = raw_native_dynamic_fixture_with_retry("json-plugin-retry", 2).await;
    let _result_guard = RawResultGuard::json_sequence("retryable@1,success");

    let response = execute_raw_json_provider_operation_from_planned_execution_context_with_options(
        &fixture.store,
        &fixture.planned,
        "chat_completion",
        "anthropic.messages.create",
        vec![MODEL_ID.to_owned()],
        anthropic_payload(MODEL_ID),
        HashMap::new(),
        &ProviderRequestOptions::default(),
    )
    .await
    .unwrap()
    .expect("raw json response after plugin retry");

    assert_eq!(response["id"], "msg_native_dynamic");

    let metrics = HttpMetricsRegistry::new("gateway").render_prometheus();
    assert!(metrics.contains(&format!(
        "sdkwork_upstream_requests_total{{service=\"gateway\",capability=\"chat_completion\",provider=\"{}\",outcome=\"attempt\"}} 2",
        fixture.provider_id
    )));
    assert!(metrics.contains(&format!(
        "sdkwork_upstream_retries_total{{service=\"gateway\",capability=\"chat_completion\",provider=\"{}\",outcome=\"scheduled\"}} 1",
        fixture.provider_id
    )));
    assert!(metrics.contains(&format!(
        "sdkwork_upstream_retry_reasons_total{{service=\"gateway\",capability=\"chat_completion\",provider=\"{}\",outcome=\"scheduled\",reason=\"plugin_retryable\"}} 1",
        fixture.provider_id
    )));
    assert!(metrics.contains(&format!(
        "sdkwork_upstream_retry_delay_ms_total{{service=\"gateway\",capability=\"chat_completion\",provider=\"{}\",source=\"plugin_retry_after_ms\"}} 1",
        fixture.provider_id
    )));
}

#[serial(extension_env)]
#[tokio::test]
async fn raw_json_planned_execution_does_not_retry_opaque_plugin_error() {
    let fixture = raw_native_dynamic_fixture_with_retry("json-plugin-opaque", 2).await;
    let _result_guard = RawResultGuard::json_sequence("error");

    let error = execute_raw_json_provider_operation_from_planned_execution_context_with_options(
        &fixture.store,
        &fixture.planned,
        "chat_completion",
        "anthropic.messages.create",
        vec![MODEL_ID.to_owned()],
        anthropic_payload(MODEL_ID),
        HashMap::new(),
        &ProviderRequestOptions::default(),
    )
    .await
    .expect_err("opaque plugin error should fail without retry");

    assert!(
        error.to_string().contains("non-retryable error"),
        "unexpected opaque plugin error: {error}"
    );

    let metrics = HttpMetricsRegistry::new("gateway").render_prometheus();
    assert!(metrics.contains(&format!(
        "sdkwork_upstream_requests_total{{service=\"gateway\",capability=\"chat_completion\",provider=\"{}\",outcome=\"attempt\"}} 1",
        fixture.provider_id
    )));
    assert!(metrics.contains(&format!(
        "sdkwork_upstream_requests_total{{service=\"gateway\",capability=\"chat_completion\",provider=\"{}\",outcome=\"failure\"}} 1",
        fixture.provider_id
    )));
    assert!(!metrics.contains(&format!(
        "sdkwork_upstream_retries_total{{service=\"gateway\",capability=\"chat_completion\",provider=\"{}\",outcome=\"scheduled\"}}",
        fixture.provider_id
    )));
}

#[serial(extension_env)]
#[tokio::test]
async fn raw_json_planned_execution_none_is_accounting_neutral_when_raw_runtime_is_not_applicable()
{
    let fixture = raw_native_dynamic_fixture("json-none-neutral").await;
    let planned = planned_context_with_runtime(&fixture.planned, ExtensionRuntime::Builtin);

    let response = execute_raw_json_provider_operation_from_planned_execution_context_with_options(
        &fixture.store,
        &planned,
        "chat_completion",
        "anthropic.messages.create",
        vec![MODEL_ID.to_owned()],
        anthropic_payload(MODEL_ID),
        HashMap::new(),
        &ProviderRequestOptions::default(),
    )
    .await
    .unwrap();

    assert!(response.is_none(), "raw json path should fall through");

    let snapshots = fixture
        .store
        .list_provider_health_snapshots()
        .await
        .unwrap();
    assert!(
        snapshots.is_empty(),
        "raw json fallthrough should not persist provider health"
    );

    let metrics = HttpMetricsRegistry::new("gateway").render_prometheus();
    assert!(!metrics.contains(&format!(
        "sdkwork_upstream_requests_total{{service=\"gateway\",capability=\"chat_completion\",provider=\"{}\",outcome=\"attempt\"}}",
        fixture.provider_id
    )));
    assert!(!metrics.contains(&format!(
        "sdkwork_upstream_requests_total{{service=\"gateway\",capability=\"chat_completion\",provider=\"{}\",outcome=\"success\"}}",
        fixture.provider_id
    )));
    assert!(!metrics.contains(&format!(
        "sdkwork_provider_health_status{{service=\"gateway\",provider=\"{}\"",
        fixture.provider_id
    )));
}

#[serial(extension_env)]
#[tokio::test]
async fn raw_json_planned_execution_is_accounting_neutral_for_connector_runtime() {
    let fixture = raw_native_dynamic_fixture("json-connector-neutral").await;
    let planned = planned_context_with_runtime(&fixture.planned, ExtensionRuntime::Connector);

    let response = execute_raw_json_provider_operation_from_planned_execution_context_with_options(
        &fixture.store,
        &planned,
        "chat_completion",
        "anthropic.messages.create",
        vec![MODEL_ID.to_owned()],
        anthropic_payload(MODEL_ID),
        HashMap::new(),
        &ProviderRequestOptions::default(),
    )
    .await
    .unwrap();

    assert!(
        response.is_none(),
        "connector runtime should stay off raw json surface"
    );

    let snapshots = fixture
        .store
        .list_provider_health_snapshots()
        .await
        .unwrap();
    assert!(
        snapshots.is_empty(),
        "connector raw json fallthrough should not persist provider health"
    );

    let metrics = HttpMetricsRegistry::new("gateway").render_prometheus();
    assert!(!metrics.contains(&format!(
        "sdkwork_upstream_requests_total{{service=\"gateway\",capability=\"chat_completion\",provider=\"{}\",outcome=\"attempt\"}}",
        fixture.provider_id
    )));
}

#[serial(extension_env)]
#[tokio::test]
async fn raw_stream_planned_execution_persists_healthy_snapshot_on_success() {
    let fixture = raw_native_dynamic_fixture("stream-success").await;

    let response =
        execute_raw_stream_provider_operation_from_planned_execution_context_with_options(
            &fixture.store,
            &fixture.planned,
            "chat_completion",
            "gemini.stream_generate_content",
            vec![MODEL_ID.to_owned()],
            gemini_payload(MODEL_ID),
            HashMap::new(),
            &ProviderRequestOptions::default(),
        )
        .await
        .unwrap()
        .expect("raw stream response");

    assert_eq!(response.content_type(), "text/event-stream");
    let snapshots = fixture
        .store
        .list_provider_health_snapshots()
        .await
        .unwrap();
    assert_eq!(snapshots.len(), 1);
    assert_eq!(snapshots[0].provider_id, fixture.provider_id);
    assert!(snapshots[0].healthy);

    let metrics = HttpMetricsRegistry::new("gateway").render_prometheus();
    assert!(metrics.contains(&format!(
        "sdkwork_upstream_requests_total{{service=\"gateway\",capability=\"chat_completion\",provider=\"{}\",outcome=\"attempt\"}} 1",
        fixture.provider_id
    )));
    assert!(metrics.contains(&format!(
        "sdkwork_upstream_requests_total{{service=\"gateway\",capability=\"chat_completion\",provider=\"{}\",outcome=\"success\"}} 1",
        fixture.provider_id
    )));
}

#[serial(extension_env)]
#[tokio::test]
async fn raw_stream_planned_execution_records_deadline_failure_before_start() {
    let fixture = raw_native_dynamic_fixture("stream-deadline").await;

    let error =
        match execute_raw_stream_provider_operation_from_planned_execution_context_with_options(
            &fixture.store,
            &fixture.planned,
            "chat_completion",
            "gemini.stream_generate_content",
            vec![MODEL_ID.to_owned()],
            gemini_payload(MODEL_ID),
            HashMap::new(),
            &ProviderRequestOptions::new().with_deadline_at_ms(1),
        )
        .await
        {
            Ok(_) => panic!("raw stream deadline should fail"),
            Err(error) => error,
        };

    assert!(
        error.to_string().contains("deadline"),
        "unexpected deadline error: {error}"
    );

    let snapshots = fixture
        .store
        .list_provider_health_snapshots()
        .await
        .unwrap();
    assert!(
        snapshots.is_empty(),
        "deadline-exceeded stream start should not mark provider unhealthy"
    );

    let metrics = HttpMetricsRegistry::new("gateway").render_prometheus();
    assert!(metrics.contains(&format!(
        "sdkwork_upstream_requests_total{{service=\"gateway\",capability=\"chat_completion\",provider=\"{}\",outcome=\"attempt\"}} 1",
        fixture.provider_id
    )));
    assert!(metrics.contains(&format!(
        "sdkwork_upstream_requests_total{{service=\"gateway\",capability=\"chat_completion\",provider=\"{}\",outcome=\"failure\"}} 1",
        fixture.provider_id
    )));
    assert!(metrics.contains(&format!(
        "sdkwork_gateway_execution_context_failures_total{{service=\"gateway\",capability=\"chat_completion\",provider=\"{}\",reason=\"deadline_exceeded\"}} 1",
        fixture.provider_id
    )));
}

#[serial(extension_env)]
#[tokio::test]
async fn raw_stream_planned_execution_retries_retryable_plugin_startup_error_and_succeeds() {
    let fixture = raw_native_dynamic_fixture_with_retry("stream-plugin-retry", 2).await;
    let _result_guard = RawResultGuard::stream_sequence("retryable@1,success");

    let response =
        execute_raw_stream_provider_operation_from_planned_execution_context_with_options(
            &fixture.store,
            &fixture.planned,
            "chat_completion",
            "gemini.stream_generate_content",
            vec![MODEL_ID.to_owned()],
            gemini_payload(MODEL_ID),
            HashMap::new(),
            &ProviderRequestOptions::default(),
        )
        .await
        .unwrap()
        .expect("raw stream response after retry");

    assert_eq!(response.content_type(), "text/event-stream");

    let metrics = HttpMetricsRegistry::new("gateway").render_prometheus();
    assert!(metrics.contains(&format!(
        "sdkwork_upstream_requests_total{{service=\"gateway\",capability=\"chat_completion\",provider=\"{}\",outcome=\"attempt\"}} 2",
        fixture.provider_id
    )));
    assert!(metrics.contains(&format!(
        "sdkwork_upstream_retries_total{{service=\"gateway\",capability=\"chat_completion\",provider=\"{}\",outcome=\"scheduled\"}} 1",
        fixture.provider_id
    )));
    assert!(metrics.contains(&format!(
        "sdkwork_upstream_retry_reasons_total{{service=\"gateway\",capability=\"chat_completion\",provider=\"{}\",outcome=\"scheduled\",reason=\"plugin_retryable\"}} 1",
        fixture.provider_id
    )));
}

#[serial(extension_env)]
#[tokio::test]
async fn raw_stream_planned_execution_does_not_retry_opaque_plugin_startup_error() {
    let fixture = raw_native_dynamic_fixture_with_retry("stream-plugin-opaque", 2).await;
    let _result_guard = RawResultGuard::stream_sequence("error");

    let error =
        match execute_raw_stream_provider_operation_from_planned_execution_context_with_options(
            &fixture.store,
            &fixture.planned,
            "chat_completion",
            "gemini.stream_generate_content",
            vec![MODEL_ID.to_owned()],
            gemini_payload(MODEL_ID),
            HashMap::new(),
            &ProviderRequestOptions::default(),
        )
        .await
        {
            Ok(_) => panic!("opaque plugin startup error should fail without retry"),
            Err(error) => error,
        };

    assert!(
        error.to_string().contains("non-retryable error"),
        "unexpected opaque plugin error: {error}"
    );

    let metrics = HttpMetricsRegistry::new("gateway").render_prometheus();
    assert!(metrics.contains(&format!(
        "sdkwork_upstream_requests_total{{service=\"gateway\",capability=\"chat_completion\",provider=\"{}\",outcome=\"attempt\"}} 1",
        fixture.provider_id
    )));
    assert!(metrics.contains(&format!(
        "sdkwork_upstream_requests_total{{service=\"gateway\",capability=\"chat_completion\",provider=\"{}\",outcome=\"failure\"}} 1",
        fixture.provider_id
    )));
    assert!(!metrics.contains(&format!(
        "sdkwork_upstream_retries_total{{service=\"gateway\",capability=\"chat_completion\",provider=\"{}\",outcome=\"scheduled\"}}",
        fixture.provider_id
    )));
}

#[serial(extension_env)]
#[tokio::test]
async fn raw_stream_planned_execution_none_is_accounting_neutral_when_raw_runtime_is_not_applicable(
) {
    let fixture = raw_native_dynamic_fixture("stream-none-neutral").await;
    let planned = planned_context_with_runtime(&fixture.planned, ExtensionRuntime::Builtin);

    let response =
        execute_raw_stream_provider_operation_from_planned_execution_context_with_options(
            &fixture.store,
            &planned,
            "chat_completion",
            "gemini.stream_generate_content",
            vec![MODEL_ID.to_owned()],
            gemini_payload(MODEL_ID),
            HashMap::new(),
            &ProviderRequestOptions::default(),
        )
        .await
        .unwrap();

    assert!(response.is_none(), "raw stream path should fall through");

    let snapshots = fixture
        .store
        .list_provider_health_snapshots()
        .await
        .unwrap();
    assert!(
        snapshots.is_empty(),
        "raw stream fallthrough should not persist provider health"
    );

    let metrics = HttpMetricsRegistry::new("gateway").render_prometheus();
    assert!(!metrics.contains(&format!(
        "sdkwork_upstream_requests_total{{service=\"gateway\",capability=\"chat_completion\",provider=\"{}\",outcome=\"attempt\"}}",
        fixture.provider_id
    )));
    assert!(!metrics.contains(&format!(
        "sdkwork_upstream_requests_total{{service=\"gateway\",capability=\"chat_completion\",provider=\"{}\",outcome=\"success\"}}",
        fixture.provider_id
    )));
    assert!(!metrics.contains(&format!(
        "sdkwork_provider_health_status{{service=\"gateway\",provider=\"{}\"",
        fixture.provider_id
    )));
}

#[serial(extension_env)]
#[tokio::test]
async fn raw_stream_planned_execution_is_accounting_neutral_for_connector_runtime() {
    let fixture = raw_native_dynamic_fixture("stream-connector-neutral").await;
    let planned = planned_context_with_runtime(&fixture.planned, ExtensionRuntime::Connector);

    let response =
        execute_raw_stream_provider_operation_from_planned_execution_context_with_options(
            &fixture.store,
            &planned,
            "chat_completion",
            "gemini.stream_generate_content",
            vec![MODEL_ID.to_owned()],
            gemini_payload(MODEL_ID),
            HashMap::new(),
            &ProviderRequestOptions::default(),
        )
        .await
        .unwrap();

    assert!(
        response.is_none(),
        "connector runtime should stay off raw stream surface"
    );

    let snapshots = fixture
        .store
        .list_provider_health_snapshots()
        .await
        .unwrap();
    assert!(
        snapshots.is_empty(),
        "connector raw stream fallthrough should not persist provider health"
    );

    let metrics = HttpMetricsRegistry::new("gateway").render_prometheus();
    assert!(!metrics.contains(&format!(
        "sdkwork_upstream_requests_total{{service=\"gateway\",capability=\"chat_completion\",provider=\"{}\",outcome=\"attempt\"}}",
        fixture.provider_id
    )));
}

struct RawNativeDynamicFixture {
    provider_id: String,
    store: SqliteAdminStore,
    planned: PlannedExecutionProviderContext,
    _guard: RawNativeDynamicFixtureGuard,
}

struct RawNativeDynamicFixtureGuard {
    extension_root: PathBuf,
    _extension_env_guard: ExtensionEnvGuard,
}

impl Drop for RawNativeDynamicFixtureGuard {
    fn drop(&mut self) {
        let _ = shutdown_all_native_dynamic_runtimes();
        cleanup_dir(&self.extension_root);
    }
}

struct RawDelayGuard {
    previous_json_delay_ms: Option<String>,
    previous_stream_delay_ms: Option<String>,
}

struct RawResultGuard {
    previous_json_result_sequence: Option<String>,
    previous_stream_result_sequence: Option<String>,
}

impl RawDelayGuard {
    fn json(delay_ms: u64) -> Self {
        let previous_json_delay_ms = std::env::var("SDKWORK_NATIVE_MOCK_JSON_DELAY_MS").ok();
        let previous_stream_delay_ms = std::env::var("SDKWORK_NATIVE_MOCK_STREAM_DELAY_MS").ok();
        std::env::set_var("SDKWORK_NATIVE_MOCK_JSON_DELAY_MS", delay_ms.to_string());
        std::env::remove_var(JSON_DELAY_SEQUENCE_MS_ENV);
        std::env::remove_var("SDKWORK_NATIVE_MOCK_STREAM_DELAY_MS");
        Self {
            previous_json_delay_ms,
            previous_stream_delay_ms,
        }
    }

    fn json_sequence(sequence: &str) -> Self {
        let previous_json_delay_ms = std::env::var("SDKWORK_NATIVE_MOCK_JSON_DELAY_MS").ok();
        let previous_stream_delay_ms = std::env::var("SDKWORK_NATIVE_MOCK_STREAM_DELAY_MS").ok();
        std::env::remove_var("SDKWORK_NATIVE_MOCK_JSON_DELAY_MS");
        std::env::set_var(JSON_DELAY_SEQUENCE_MS_ENV, sequence);
        std::env::remove_var("SDKWORK_NATIVE_MOCK_STREAM_DELAY_MS");
        Self {
            previous_json_delay_ms,
            previous_stream_delay_ms,
        }
    }
}

impl Drop for RawDelayGuard {
    fn drop(&mut self) {
        restore_env_var(
            "SDKWORK_NATIVE_MOCK_JSON_DELAY_MS",
            self.previous_json_delay_ms.as_deref(),
        );
        std::env::remove_var(JSON_DELAY_SEQUENCE_MS_ENV);
        restore_env_var(
            "SDKWORK_NATIVE_MOCK_STREAM_DELAY_MS",
            self.previous_stream_delay_ms.as_deref(),
        );
    }
}

impl RawResultGuard {
    fn json_sequence(sequence: &str) -> Self {
        let previous_json_result_sequence = std::env::var(JSON_RESULT_SEQUENCE_ENV).ok();
        std::env::set_var(JSON_RESULT_SEQUENCE_ENV, sequence);
        Self {
            previous_json_result_sequence,
            previous_stream_result_sequence: std::env::var(STREAM_RESULT_SEQUENCE_ENV).ok(),
        }
    }

    fn stream_sequence(sequence: &str) -> Self {
        let previous_json_result_sequence = std::env::var(JSON_RESULT_SEQUENCE_ENV).ok();
        let previous_stream_result_sequence = std::env::var(STREAM_RESULT_SEQUENCE_ENV).ok();
        std::env::set_var(STREAM_RESULT_SEQUENCE_ENV, sequence);
        Self {
            previous_json_result_sequence,
            previous_stream_result_sequence,
        }
    }
}

impl Drop for RawResultGuard {
    fn drop(&mut self) {
        restore_env_var(
            JSON_RESULT_SEQUENCE_ENV,
            self.previous_json_result_sequence.as_deref(),
        );
        restore_env_var(
            STREAM_RESULT_SEQUENCE_ENV,
            self.previous_stream_result_sequence.as_deref(),
        );
    }
}

async fn raw_native_dynamic_fixture(suffix: &str) -> RawNativeDynamicFixture {
    raw_native_dynamic_fixture_with_retry(suffix, 1).await
}

async fn raw_native_dynamic_fixture_with_retry(
    suffix: &str,
    retry_max_attempts: u32,
) -> RawNativeDynamicFixture {
    let _ = shutdown_all_native_dynamic_runtimes();
    let provider_id = format!("provider-native-raw-{suffix}");
    let credential_id = format!("cred-native-raw-{suffix}");
    let installation_id = format!("native-raw-installation-{suffix}");
    let policy_id = format!("policy-native-raw-{suffix}");

    let extension_root = temp_extension_root(&format!("raw-execution-governance-{suffix}"));
    let package_dir = extension_root.join("sdkwork-provider-native-mock");
    fs::create_dir_all(&package_dir).unwrap();

    let signing_key = SigningKey::from_bytes(&[12_u8; 32]);
    let public_key = STANDARD.encode(signing_key.verifying_key().to_bytes());
    let library_path = native_dynamic_fixture_library_path();
    let manifest = native_dynamic_manifest(&library_path);
    let signature = sign_native_dynamic_package(&package_dir, &manifest, &signing_key);
    let manifest = manifest.with_trust(ExtensionTrustDeclaration::signed(
        "sdkwork",
        ExtensionSignature::new(
            ExtensionSignatureAlgorithm::Ed25519,
            public_key.clone(),
            signature,
        ),
    ));
    fs::write(
        package_dir.join("sdkwork-extension.toml"),
        toml::to_string(&manifest).unwrap(),
    )
    .unwrap();
    let extension_env_guard = native_dynamic_env_guard(&extension_root, &public_key);

    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);
    let secret_manager = CredentialSecretManager::database_encrypted("local-dev-master-key");

    store
        .insert_channel(&Channel::new("openai", "OpenAI"))
        .await
        .unwrap();
    store
        .insert_provider(
            &ProxyProvider::new(
                &provider_id,
                "openai",
                "native-dynamic",
                PROVIDER_BASE_URL,
                "Native Raw Provider",
            )
            .with_extension_id(FIXTURE_EXTENSION_ID)
            .with_protocol_kind("custom"),
        )
        .await
        .unwrap();
    store
        .insert_model(&ModelCatalogEntry::new(MODEL_ID, &provider_id).with_streaming(true))
        .await
        .unwrap();
    persist_credential_with_secret_and_manager(
        &store,
        &secret_manager,
        "tenant-1",
        &provider_id,
        &credential_id,
        "sk-native-raw",
    )
    .await
    .unwrap();
    store
        .insert_extension_installation(
            &ExtensionInstallation::new(
                &installation_id,
                FIXTURE_EXTENSION_ID,
                ExtensionRuntime::NativeDynamic,
            )
            .with_enabled(true)
            .with_entrypoint(library_path.to_string_lossy())
            .with_config(json!({})),
        )
        .await
        .unwrap();
    store
        .insert_extension_instance(
            &ExtensionInstance::new(&provider_id, &installation_id, FIXTURE_EXTENSION_ID)
                .with_enabled(true)
                .with_base_url(PROVIDER_BASE_URL)
                .with_config(json!({})),
        )
        .await
        .unwrap();
    persist_routing_policy(
        &store,
        &RoutingPolicy::new(&policy_id, "chat_completion", MODEL_ID)
            .with_priority(100)
            .with_ordered_provider_ids(vec![provider_id.clone()])
            .with_upstream_retry_max_attempts(retry_max_attempts)
            .with_upstream_retry_base_delay_ms(0)
            .with_upstream_retry_max_delay_ms(100)
            .with_execution_failover_enabled(false),
    )
    .await
    .unwrap();

    let planned = planned_execution_provider_context_for_route_without_log(
        &store,
        &secret_manager,
        "tenant-1",
        "project-1",
        "chat_completion",
        MODEL_ID,
    )
    .await
    .unwrap()
    .expect("planned provider context");
    assert_eq!(planned.provider.id, provider_id);
    assert_eq!(planned.execution.runtime_key, FIXTURE_EXTENSION_ID);
    assert_eq!(planned.execution.runtime, ExtensionRuntime::NativeDynamic);
    assert!(!planned.execution.local_fallback);

    RawNativeDynamicFixture {
        provider_id,
        store,
        planned,
        _guard: RawNativeDynamicFixtureGuard {
            extension_root,
            _extension_env_guard: extension_env_guard,
        },
    }
}

fn anthropic_payload(model: &str) -> Value {
    json!({
        "model": model,
        "max_tokens": 32,
        "messages": [{
            "role": "user",
            "content": "hello"
        }]
    })
}

fn gemini_payload(model: &str) -> Value {
    json!({
        "model": model,
        "contents": [{
            "role": "user",
            "parts": [{
                "text": "hello"
            }]
        }]
    })
}

fn planned_context_with_runtime(
    planned: &PlannedExecutionProviderContext,
    runtime: ExtensionRuntime,
) -> PlannedExecutionProviderContext {
    let mut planned = planned.clone();
    planned.execution.runtime = runtime;
    planned.execution.runtime_key = format!(
        "{}-{}-non-raw",
        planned.execution.runtime_key,
        planned.execution.runtime.as_str()
    );
    planned
}

fn restore_env_var(key: &str, value: Option<&str>) {
    match value {
        Some(value) => std::env::set_var(key, value),
        None => std::env::remove_var(key),
    }
}

fn temp_extension_root(suffix: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    path.push(format!("sdkwork-app-gateway-{suffix}-{millis}"));
    path
}

fn cleanup_dir(path: &Path) {
    let _ = fs::remove_dir_all(path);
}

fn native_dynamic_env_guard(path: &Path, public_key: &str) -> ExtensionEnvGuard {
    let previous_paths = std::env::var("SDKWORK_EXTENSION_PATHS").ok();
    let previous_connector = std::env::var("SDKWORK_EXTENSION_ENABLE_CONNECTOR_EXTENSIONS").ok();
    let previous_native = std::env::var("SDKWORK_EXTENSION_ENABLE_NATIVE_DYNAMIC_EXTENSIONS").ok();
    let previous_connector_signature =
        std::env::var("SDKWORK_EXTENSION_REQUIRE_SIGNATURE_FOR_CONNECTOR_EXTENSIONS").ok();
    let previous_native_signature =
        std::env::var("SDKWORK_EXTENSION_REQUIRE_SIGNATURE_FOR_NATIVE_DYNAMIC_EXTENSIONS").ok();
    let previous_trusted_signers = std::env::var("SDKWORK_EXTENSION_TRUSTED_SIGNERS").ok();

    let joined_paths = std::env::join_paths([path]).unwrap();
    std::env::set_var("SDKWORK_EXTENSION_PATHS", joined_paths);
    std::env::set_var("SDKWORK_EXTENSION_ENABLE_CONNECTOR_EXTENSIONS", "false");
    std::env::set_var("SDKWORK_EXTENSION_ENABLE_NATIVE_DYNAMIC_EXTENSIONS", "true");
    std::env::set_var(
        "SDKWORK_EXTENSION_REQUIRE_SIGNATURE_FOR_CONNECTOR_EXTENSIONS",
        "false",
    );
    std::env::set_var(
        "SDKWORK_EXTENSION_REQUIRE_SIGNATURE_FOR_NATIVE_DYNAMIC_EXTENSIONS",
        "true",
    );
    std::env::set_var(
        "SDKWORK_EXTENSION_TRUSTED_SIGNERS",
        format!("sdkwork={public_key}"),
    );

    ExtensionEnvGuard {
        previous_paths,
        previous_connector,
        previous_native,
        previous_connector_signature,
        previous_native_signature,
        previous_trusted_signers,
    }
}

fn native_dynamic_fixture_library_path() -> PathBuf {
    let current_exe = std::env::current_exe().expect("current exe");
    let directory = current_exe.parent().expect("exe dir");
    let prefix = if cfg!(windows) {
        "sdkwork_api_ext_provider_native_mock"
    } else {
        "libsdkwork_api_ext_provider_native_mock"
    };
    let extension = if cfg!(windows) {
        "dll"
    } else if cfg!(target_os = "macos") {
        "dylib"
    } else {
        "so"
    };

    std::fs::read_dir(directory)
        .expect("deps dir")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .find(|path| {
            path.extension().and_then(|value| value.to_str()) == Some(extension)
                && path
                    .file_stem()
                    .and_then(|value| value.to_str())
                    .is_some_and(|stem| stem.starts_with(prefix))
        })
        .expect("native dynamic fixture library")
}

fn native_dynamic_manifest(library_path: &Path) -> ExtensionManifest {
    ExtensionManifest::new(
        FIXTURE_EXTENSION_ID,
        ExtensionKind::Provider,
        "0.1.0",
        ExtensionRuntime::NativeDynamic,
    )
    .with_display_name("Native Mock")
    .with_protocol(ExtensionProtocol::OpenAi)
    .with_entrypoint(library_path.to_string_lossy())
    .with_supported_modality(ExtensionModality::Audio)
    .with_supported_modality(ExtensionModality::Video)
    .with_supported_modality(ExtensionModality::File)
    .with_channel_binding("sdkwork.channel.openai")
    .with_permission(ExtensionPermission::NetworkOutbound)
    .with_capability(CapabilityDescriptor::new(
        "chat.completions.create",
        CompatibilityLevel::Native,
    ))
    .with_capability(CapabilityDescriptor::new(
        "chat.completions.stream",
        CompatibilityLevel::Native,
    ))
    .with_capability(CapabilityDescriptor::new(
        "responses.create",
        CompatibilityLevel::Native,
    ))
    .with_capability(CapabilityDescriptor::new(
        "responses.stream",
        CompatibilityLevel::Native,
    ))
    .with_capability(CapabilityDescriptor::new(
        "anthropic.messages.create",
        CompatibilityLevel::Native,
    ))
    .with_capability(CapabilityDescriptor::new(
        "anthropic.messages.count_tokens",
        CompatibilityLevel::Native,
    ))
    .with_capability(CapabilityDescriptor::new(
        "gemini.generate_content",
        CompatibilityLevel::Native,
    ))
    .with_capability(CapabilityDescriptor::new(
        "gemini.stream_generate_content",
        CompatibilityLevel::Native,
    ))
    .with_capability(CapabilityDescriptor::new(
        "gemini.count_tokens",
        CompatibilityLevel::Native,
    ))
    .with_capability(CapabilityDescriptor::new(
        "audio.speech.create",
        CompatibilityLevel::Native,
    ))
    .with_capability(CapabilityDescriptor::new(
        "files.content",
        CompatibilityLevel::Native,
    ))
    .with_capability(CapabilityDescriptor::new(
        "videos.content",
        CompatibilityLevel::Native,
    ))
}

fn sign_native_dynamic_package(
    package_dir: &Path,
    manifest: &ExtensionManifest,
    signing_key: &SigningKey,
) -> String {
    use ed25519_dalek::Signer;

    #[derive(serde::Serialize)]
    struct PackageSignaturePayload<'a> {
        manifest: &'a ExtensionManifest,
        files: Vec<PackageFileDigest>,
    }

    #[derive(serde::Serialize)]
    struct PackageFileDigest {
        path: String,
        sha256: String,
    }

    let files = std::fs::read_dir(package_dir)
        .unwrap()
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name().and_then(|value| value.to_str()) != Some("sdkwork-extension.toml")
        })
        .map(|path| PackageFileDigest {
            path: path
                .strip_prefix(package_dir)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/"),
            sha256: sha256_hex_path(&path),
        })
        .collect::<Vec<_>>();

    let payload = serde_json::to_vec(&PackageSignaturePayload { manifest, files }).unwrap();
    let signature = signing_key.sign(&payload);
    STANDARD.encode(signature.to_bytes())
}

fn sha256_hex_path(path: &Path) -> String {
    let digest = Sha256::digest(std::fs::read(path).unwrap());
    let mut encoded = String::with_capacity(digest.len() * 2);
    for byte in digest {
        encoded.push_str(&format!("{byte:02x}"));
    }
    encoded
}

struct ExtensionEnvGuard {
    previous_paths: Option<String>,
    previous_connector: Option<String>,
    previous_native: Option<String>,
    previous_connector_signature: Option<String>,
    previous_native_signature: Option<String>,
    previous_trusted_signers: Option<String>,
}

impl Drop for ExtensionEnvGuard {
    fn drop(&mut self) {
        restore_env_var("SDKWORK_EXTENSION_PATHS", self.previous_paths.as_deref());
        restore_env_var(
            "SDKWORK_EXTENSION_ENABLE_CONNECTOR_EXTENSIONS",
            self.previous_connector.as_deref(),
        );
        restore_env_var(
            "SDKWORK_EXTENSION_ENABLE_NATIVE_DYNAMIC_EXTENSIONS",
            self.previous_native.as_deref(),
        );
        restore_env_var(
            "SDKWORK_EXTENSION_REQUIRE_SIGNATURE_FOR_CONNECTOR_EXTENSIONS",
            self.previous_connector_signature.as_deref(),
        );
        restore_env_var(
            "SDKWORK_EXTENSION_REQUIRE_SIGNATURE_FOR_NATIVE_DYNAMIC_EXTENSIONS",
            self.previous_native_signature.as_deref(),
        );
        restore_env_var(
            "SDKWORK_EXTENSION_TRUSTED_SIGNERS",
            self.previous_trusted_signers.as_deref(),
        );
    }
}
