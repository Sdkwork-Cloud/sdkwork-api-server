use std::sync::Arc;

use sdkwork_api_app_credential::CredentialSecretManager;
use sdkwork_api_app_gateway::configure_capability_catalog_cache_store;
use sdkwork_api_app_runtime::{
    build_admin_store_and_commercial_billing_from_config, build_cache_runtime_from_config,
    resolve_service_runtime_node_id, start_extension_runtime_rollout_supervision,
    start_standalone_runtime_supervision, StandaloneListenerHost, StandaloneServiceKind,
    StandaloneServiceReloadHandles,
};
use sdkwork_api_config::{StandaloneConfig, StandaloneConfigLoader};
use sdkwork_api_interface_admin::{admin_router_with_state_and_http_exposure, AdminApiState};
use sdkwork_api_observability::init_tracing;
use sdkwork_api_storage_core::{AdminStore, Reloadable};

struct AdminServiceRuntime {
    live_store: Reloadable<Arc<dyn AdminStore>>,
    state: AdminApiState,
    reload_handles: StandaloneServiceReloadHandles,
}

async fn build_admin_service_runtime(config: &StandaloneConfig) -> anyhow::Result<AdminServiceRuntime> {
    let (store, commercial_billing) =
        build_admin_store_and_commercial_billing_from_config(config).await?;
    let live_store = Reloadable::new(store);
    let live_commercial_billing = Reloadable::new(commercial_billing);
    let live_admin_jwt = Reloadable::new(config.admin_jwt_signing_secret.clone());
    let live_secret_manager =
        Reloadable::new(CredentialSecretManager::new_with_legacy_master_keys(
            config.secret_backend,
            config.credential_master_key.clone(),
            config.credential_legacy_master_keys.clone(),
            config.secret_local_file.clone(),
            config.secret_keyring_service.clone(),
        ));

    Ok(AdminServiceRuntime {
        live_store: live_store.clone(),
        state:
            AdminApiState::with_live_store_and_secret_manager_handle_and_commercial_billing_and_jwt_secret_handle(
                live_store.clone(),
                live_secret_manager.clone(),
                Some(live_commercial_billing.clone()),
                live_admin_jwt.clone(),
            ),
        reload_handles: StandaloneServiceReloadHandles::admin(live_store, live_admin_jwt)
            .with_live_commercial_billing(live_commercial_billing)
            .with_secret_manager(live_secret_manager),
    })
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing("admin-api-service");
    let (config_loader, config) = StandaloneConfigLoader::from_env()?;
    config.validate_security_posture()?;
    config.apply_to_process_env();
    let cache_runtime = build_cache_runtime_from_config(&config).await?;
    if config.cache_backend.supports_shared_cache_coherence() {
        configure_capability_catalog_cache_store(cache_runtime.cache_store());
    }
    let runtime = build_admin_service_runtime(&config).await?;
    let listener_host =
        StandaloneListenerHost::bind(
            config.admin_bind.clone(),
            admin_router_with_state_and_http_exposure(
                runtime.state.clone(),
                config.http_exposure_config(),
            ),
        )
        .await?;
    let node_id = resolve_service_runtime_node_id(StandaloneServiceKind::Admin);
    let _rollout_supervision = start_extension_runtime_rollout_supervision(
        StandaloneServiceKind::Admin,
        node_id.clone(),
        runtime.live_store.clone(),
    )?;
    let _runtime_supervision = start_standalone_runtime_supervision(
        StandaloneServiceKind::Admin,
        config_loader,
        config.clone(),
        runtime
            .reload_handles
            .with_listener(listener_host.reload_handle())
            .with_node_id(node_id),
    );
    listener_host.wait().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use serde_json::Value;
    use tower::ServiceExt;

    use super::*;

    async fn read_body(response: axum::response::Response) -> String {
        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        String::from_utf8(bytes.to_vec()).unwrap()
    }

    fn metric_counter_value(body: &str, key: &str) -> u64 {
        body.lines()
            .find_map(|line| {
                let (metric_key, value) = line.split_once(' ')?;
                if metric_key != key {
                    return None;
                }
                value.trim().parse::<u64>().ok()
            })
            .unwrap_or(0)
    }

    async fn login_token(app: axum::Router) -> String {
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/admin/auth/login")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        "{\"email\":\"admin@sdkwork.local\",\"password\":\"ChangeMe123!\"}",
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let payload: Value = serde_json::from_slice(&bytes).unwrap();
        payload["token"].as_str().unwrap().to_owned()
    }

    #[tokio::test]
    async fn build_admin_service_runtime_exposes_commercial_billing_control_plane_routes() {
        let mut config = sdkwork_api_config::StandaloneConfig::default();
        config.database_url = "sqlite::memory:".to_owned();
        config.bootstrap_profile = "dev".to_owned();

        let runtime = build_admin_service_runtime(&config).await.unwrap();
        let app = admin_router_with_state_and_http_exposure(
            runtime.state.clone(),
            config.http_exposure_config(),
        );
        let token = login_token(app.clone()).await;

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/admin/billing/account-holds")
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn build_admin_service_runtime_exposes_pricing_lifecycle_synchronization_route() {
        let mut config = sdkwork_api_config::StandaloneConfig::default();
        config.database_url = "sqlite::memory:".to_owned();
        config.bootstrap_profile = "dev".to_owned();

        let runtime = build_admin_service_runtime(&config).await.unwrap();
        let app = admin_router_with_state_and_http_exposure(
            runtime.state.clone(),
            config.http_exposure_config(),
        );
        let token = login_token(app.clone()).await;

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/admin/billing/pricing-lifecycle/synchronize")
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn build_admin_service_runtime_reports_success_metrics_for_commercial_billing_routes() {
        let mut config = sdkwork_api_config::StandaloneConfig::default();
        config.database_url = "sqlite::memory:".to_owned();
        config.bootstrap_profile = "dev".to_owned();

        let runtime = build_admin_service_runtime(&config).await.unwrap();
        let app = admin_router_with_state_and_http_exposure(
            runtime.state.clone(),
            config.http_exposure_config(),
        );

        let account_holds_ok_key =
            "sdkwork_http_requests_total{service=\"admin\",method=\"GET\",route=\"/admin/billing/account-holds\",status=\"200\"}";
        let request_settlements_ok_key =
            "sdkwork_http_requests_total{service=\"admin\",method=\"GET\",route=\"/admin/billing/request-settlements\",status=\"200\"}";
        let pricing_sync_ok_key =
            "sdkwork_http_requests_total{service=\"admin\",method=\"POST\",route=\"/admin/billing/pricing-lifecycle/synchronize\",status=\"200\"}";
        let account_holds_not_implemented_key =
            "sdkwork_http_requests_total{service=\"admin\",method=\"GET\",route=\"/admin/billing/account-holds\",status=\"501\"}";
        let request_settlements_not_implemented_key =
            "sdkwork_http_requests_total{service=\"admin\",method=\"GET\",route=\"/admin/billing/request-settlements\",status=\"501\"}";
        let pricing_sync_not_implemented_key =
            "sdkwork_http_requests_total{service=\"admin\",method=\"POST\",route=\"/admin/billing/pricing-lifecycle/synchronize\",status=\"501\"}";

        let initial_metrics = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/metrics")
                    .header("authorization", "Bearer local-dev-metrics-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(initial_metrics.status(), StatusCode::OK);
        let initial_metrics_body = read_body(initial_metrics).await;

        let token = login_token(app.clone()).await;

        let summary = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/admin/billing/summary")
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(summary.status(), StatusCode::OK);

        let account_holds = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/admin/billing/account-holds")
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(account_holds.status(), StatusCode::OK);

        let request_settlements = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/admin/billing/request-settlements")
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(request_settlements.status(), StatusCode::OK);

        let pricing_sync = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/admin/billing/pricing-lifecycle/synchronize")
                    .header("authorization", format!("Bearer {token}"))
                    .header("content-type", "application/json")
                    .body(Body::from("{}"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(pricing_sync.status(), StatusCode::OK);

        let metrics = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/metrics")
                    .header("authorization", "Bearer local-dev-metrics-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(metrics.status(), StatusCode::OK);
        let metrics_body = read_body(metrics).await;

        assert_eq!(
            metric_counter_value(&metrics_body, account_holds_ok_key),
            metric_counter_value(&initial_metrics_body, account_holds_ok_key) + 1
        );
        assert_eq!(
            metric_counter_value(&metrics_body, request_settlements_ok_key),
            metric_counter_value(&initial_metrics_body, request_settlements_ok_key) + 1
        );
        assert_eq!(
            metric_counter_value(&metrics_body, pricing_sync_ok_key),
            metric_counter_value(&initial_metrics_body, pricing_sync_ok_key) + 1
        );
        assert_eq!(
            metric_counter_value(&metrics_body, account_holds_not_implemented_key),
            0
        );
        assert_eq!(
            metric_counter_value(&metrics_body, request_settlements_not_implemented_key),
            0
        );
        assert_eq!(
            metric_counter_value(&metrics_body, pricing_sync_not_implemented_key),
            0
        );
    }
}
