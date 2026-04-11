use super::gateway_cache::{
    cache_routing_decision, routing_decision_cache_key, take_cached_routing_decision,
};
use super::gateway_extension_host::configured_extension_discovery_policy;
use super::gateway_runtime_execution::{
    provider_request_metric_capability, record_gateway_execution_context_failure,
    record_gateway_execution_failover, record_gateway_provider_health,
    record_gateway_provider_health_persist_failure, record_gateway_provider_health_recovery_probe,
    record_gateway_upstream_outcome, record_gateway_upstream_retry_with_detail,
};
use super::{
    configure_route_decision_cache_store, current_request_api_key_group_id,
    with_request_api_key_group_id, with_request_routing_region, RoutingDecision,
    ROUTING_DECISION_CACHE_NAMESPACE,
};
use sdkwork_api_cache_core::CacheStore;
use sdkwork_api_cache_memory::MemoryCacheStore;
use sdkwork_api_observability::HttpMetricsRegistry;
use sdkwork_api_provider_core::ProviderRequest;
use std::path::Path;
use std::sync::Arc;

#[test]
fn configured_extension_discovery_policy_reads_native_dynamic_env_configuration() {
    let temp_root = std::env::temp_dir().join(format!(
        "sdkwork-app-gateway-policy-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("unix time")
            .as_millis()
    ));
    std::fs::create_dir_all(&temp_root).unwrap();
    let _guard = ExtensionEnvGuard::set(
        &[
            (
                "SDKWORK_EXTENSION_PATHS",
                std::env::join_paths([temp_root.as_path()])
                    .unwrap()
                    .to_string_lossy()
                    .as_ref(),
            ),
            ("SDKWORK_EXTENSION_ENABLE_CONNECTOR_EXTENSIONS", "false"),
            ("SDKWORK_EXTENSION_ENABLE_NATIVE_DYNAMIC_EXTENSIONS", "true"),
            (
                "SDKWORK_EXTENSION_REQUIRE_SIGNATURE_FOR_NATIVE_DYNAMIC_EXTENSIONS",
                "true",
            ),
        ],
        &temp_root,
    );

    let policy = configured_extension_discovery_policy();

    assert_eq!(policy.search_paths, vec![temp_root]);
    assert!(!policy.enable_connector_extensions);
    assert!(policy.enable_native_dynamic_extensions);
    assert!(policy.require_signed_native_dynamic_extensions);
}

#[test]
fn provider_request_metric_capability_groups_requests_by_upstream_surface() {
    assert_eq!(
        provider_request_metric_capability(&ProviderRequest::ChatCompletionsList),
        "chat_completion"
    );
    assert_eq!(
        provider_request_metric_capability(&ProviderRequest::ResponsesRetrieve("resp-1")),
        "responses"
    );
    assert_eq!(
        provider_request_metric_capability(&ProviderRequest::AudioVoicesList),
        "audio"
    );
}

#[test]
fn gateway_upstream_outcomes_are_recorded_to_shared_gateway_metrics() {
    record_gateway_upstream_outcome("chat_completion", "provider-metrics-test", "attempt");
    record_gateway_upstream_outcome("chat_completion", "provider-metrics-test", "success");
    record_gateway_upstream_outcome("chat_completion", "provider-metrics-test", "failure");
    record_gateway_upstream_retry_with_detail(
        "chat_completion",
        "provider-metrics-test",
        "exhausted",
        "status_503",
        None,
        None,
    );
    record_gateway_upstream_retry_with_detail(
        "chat_completion",
        "provider-metrics-test",
        "scheduled",
        "status_429",
        Some("retry_after_seconds"),
        Some(1000),
    );
    record_gateway_execution_failover(
        "chat_completion",
        "provider-primary",
        "provider-backup",
        "success",
    );
    record_gateway_provider_health("provider-health-failed", "builtin", false, 1234);
    record_gateway_provider_health("provider-health-healthy", "builtin", true, 5678);
    record_gateway_provider_health_persist_failure("provider-health-failed", "builtin");
    record_gateway_provider_health_recovery_probe("provider-health-recovery", "selected");
    record_gateway_provider_health_recovery_probe("provider-health-recovery", "lease_contended");
    record_gateway_provider_health_recovery_probe("provider-health-recovery", "lease_error");
    record_gateway_execution_context_failure(
        "chat_completion",
        "provider-context-timeout",
        "request_timeout",
    );
    record_gateway_execution_context_failure(
        "chat_completion",
        "provider-context-overload",
        "provider_overloaded",
    );
    record_gateway_execution_context_failure(
        "chat_completion",
        "provider-context-deadline",
        "deadline_exceeded",
    );

    let output = HttpMetricsRegistry::new("gateway").render_prometheus();

    assert!(output.contains(
            "sdkwork_upstream_requests_total{service=\"gateway\",capability=\"chat_completion\",provider=\"provider-metrics-test\",outcome=\"attempt\"} 1"
        ));
    assert!(output.contains(
            "sdkwork_upstream_requests_total{service=\"gateway\",capability=\"chat_completion\",provider=\"provider-metrics-test\",outcome=\"success\"} 1"
        ));
    assert!(output.contains(
            "sdkwork_upstream_requests_total{service=\"gateway\",capability=\"chat_completion\",provider=\"provider-metrics-test\",outcome=\"failure\"} 1"
        ));
    assert!(output.contains(
            "sdkwork_upstream_retries_total{service=\"gateway\",capability=\"chat_completion\",provider=\"provider-metrics-test\",outcome=\"scheduled\"} 1"
        ));
    assert!(output.contains(
            "sdkwork_upstream_retries_total{service=\"gateway\",capability=\"chat_completion\",provider=\"provider-metrics-test\",outcome=\"exhausted\"} 1"
        ));
    assert!(output.contains(
            "sdkwork_upstream_retry_reasons_total{service=\"gateway\",capability=\"chat_completion\",provider=\"provider-metrics-test\",outcome=\"scheduled\",reason=\"status_429\"} 1"
        ));
    assert!(output.contains(
            "sdkwork_upstream_retry_reasons_total{service=\"gateway\",capability=\"chat_completion\",provider=\"provider-metrics-test\",outcome=\"exhausted\",reason=\"status_503\"} 1"
        ));
    assert!(output.contains(
            "sdkwork_upstream_retry_delay_ms_total{service=\"gateway\",capability=\"chat_completion\",provider=\"provider-metrics-test\",source=\"retry_after_seconds\"} 1000"
        ));
    assert!(output.contains(
            "sdkwork_gateway_failovers_total{service=\"gateway\",capability=\"chat_completion\",from_provider=\"provider-primary\",to_provider=\"provider-backup\",outcome=\"success\"} 1"
        ));
    assert!(output.contains(
            "sdkwork_provider_health_status{service=\"gateway\",provider=\"provider-health-failed\",runtime=\"builtin\"} 0"
        ));
    assert!(output.contains(
            "sdkwork_provider_health_status{service=\"gateway\",provider=\"provider-health-healthy\",runtime=\"builtin\"} 1"
        ));
    assert!(output.contains(
            "sdkwork_provider_health_observed_at_ms{service=\"gateway\",provider=\"provider-health-healthy\",runtime=\"builtin\"} 5678"
        ));
    assert!(output.contains(
            "sdkwork_provider_health_persist_failures_total{service=\"gateway\",provider=\"provider-health-failed\",runtime=\"builtin\"} 1"
        ));
    assert!(output.contains(
            "sdkwork_provider_health_recovery_probes_total{service=\"gateway\",provider=\"provider-health-recovery\",outcome=\"selected\"} 1"
        ));
    assert!(output.contains(
            "sdkwork_provider_health_recovery_probes_total{service=\"gateway\",provider=\"provider-health-recovery\",outcome=\"lease_contended\"} 1"
        ));
    assert!(output.contains(
            "sdkwork_provider_health_recovery_probes_total{service=\"gateway\",provider=\"provider-health-recovery\",outcome=\"lease_error\"} 1"
        ));
    assert!(output.contains(
            "sdkwork_gateway_execution_context_failures_total{service=\"gateway\",capability=\"chat_completion\",provider=\"provider-context-timeout\",reason=\"request_timeout\"} 1"
        ));
    assert!(output.contains(
            "sdkwork_gateway_execution_context_failures_total{service=\"gateway\",capability=\"chat_completion\",provider=\"provider-context-overload\",reason=\"provider_overloaded\"} 1"
        ));
    assert!(output.contains(
            "sdkwork_gateway_execution_context_failures_total{service=\"gateway\",capability=\"chat_completion\",provider=\"provider-context-deadline\",reason=\"deadline_exceeded\"} 1"
        ));
}

#[tokio::test]
async fn request_api_key_group_scope_exposes_group_id_inside_gateway_execution() {
    let value = with_request_api_key_group_id(Some("group-live".to_owned()), async {
        current_request_api_key_group_id()
    })
    .await;

    assert_eq!(value.as_deref(), Some("group-live"));
    assert_eq!(current_request_api_key_group_id(), None);
}

#[tokio::test]
async fn route_decision_cache_uses_configured_cache_store_and_consumes_entries_once() {
    let cache_store: Arc<dyn CacheStore> = Arc::new(MemoryCacheStore::default());
    configure_route_decision_cache_store(cache_store.clone());

    with_request_routing_region(Some("us-east".to_owned()), async {
        let decision = RoutingDecision::new(
            "provider-openrouter",
            vec!["provider-openrouter".to_owned()],
        );
        let cache_key = routing_decision_cache_key(
            "tenant-1",
            Some("project-1"),
            Some("group-1"),
            "chat",
            "gpt-4.1",
            Some("us-east"),
        );

        cache_routing_decision(
            "tenant-1",
            Some("project-1"),
            Some("group-1"),
            "chat",
            "gpt-4.1",
            Some("us-east"),
            &decision,
        )
        .await;

        assert!(cache_store
            .get(ROUTING_DECISION_CACHE_NAMESPACE, &cache_key)
            .await
            .unwrap()
            .is_some());

        let cached = take_cached_routing_decision(
            "tenant-1",
            Some("project-1"),
            Some("group-1"),
            "chat",
            "gpt-4.1",
            Some("us-east"),
        )
        .await
        .expect("cached routing decision");
        let second = take_cached_routing_decision(
            "tenant-1",
            Some("project-1"),
            Some("group-1"),
            "chat",
            "gpt-4.1",
            Some("us-east"),
        )
        .await;

        assert_eq!(cached.selected_provider_id, "provider-openrouter");
        assert!(second.is_none());
    })
    .await;
}

#[test]
fn relay_files_uploads_does_not_define_media_local_fallbacks() {
    let relay_files_uploads = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("src/relay_files_uploads.rs"),
    )
    .expect("read relay_files_uploads source");
    let relay_music_video = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("src/relay_music_video.rs"),
    )
    .expect("read relay_music_video source");

    for signature in [
        "pub fn create_music(",
        "pub fn list_music(",
        "pub fn get_music(",
        "pub fn delete_music(",
        "pub fn music_content(",
        "pub fn create_music_lyrics(",
        "pub fn create_video(",
        "pub fn list_videos(",
        "pub fn get_video(",
        "pub fn delete_video(",
        "pub fn video_content(",
        "pub fn remix_video(",
        "pub fn create_video_character(",
        "pub fn list_video_characters(",
        "pub fn get_video_character(",
        "pub fn get_video_character_canonical(",
        "pub fn update_video_character(",
        "pub fn extend_video(",
        "pub fn edit_video(",
        "pub fn extensions_video(",
    ] {
        assert!(
            !relay_files_uploads.contains(signature),
            "relay_files_uploads should not define legacy media fallback `{signature}`",
        );
        assert!(
            relay_music_video.contains(signature),
            "relay_music_video should define canonical media fallback `{signature}`",
        );
    }
}

struct ExtensionEnvGuard {
    previous: Vec<(&'static str, Option<String>)>,
    cleanup_dir: std::path::PathBuf,
}

impl ExtensionEnvGuard {
    fn set(overrides: &[(&'static str, &str)], cleanup_dir: &Path) -> Self {
        let keys = [
            "SDKWORK_EXTENSION_PATHS",
            "SDKWORK_EXTENSION_ENABLE_CONNECTOR_EXTENSIONS",
            "SDKWORK_EXTENSION_ENABLE_NATIVE_DYNAMIC_EXTENSIONS",
            "SDKWORK_EXTENSION_REQUIRE_SIGNATURE_FOR_NATIVE_DYNAMIC_EXTENSIONS",
        ];
        let previous = keys
            .into_iter()
            .map(|key| (key, std::env::var(key).ok()))
            .collect::<Vec<_>>();

        for key in keys {
            std::env::remove_var(key);
        }
        for (key, value) in overrides {
            std::env::set_var(key, value);
        }

        Self {
            previous,
            cleanup_dir: cleanup_dir.to_path_buf(),
        }
    }
}

impl Drop for ExtensionEnvGuard {
    fn drop(&mut self) {
        for (key, value) in &self.previous {
            match value {
                Some(value) => std::env::set_var(key, value),
                None => std::env::remove_var(key),
            }
        }
        let _ = std::fs::remove_dir_all(&self.cleanup_dir);
    }
}
