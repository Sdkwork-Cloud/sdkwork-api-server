use std::sync::Arc;

use sdkwork_api_app_credential::CredentialSecretManager;
use sdkwork_api_app_gateway::{
    configure_capability_catalog_cache_store, configure_route_decision_cache_store,
    configure_route_recovery_probe_lock_store,
};
use sdkwork_api_app_rate_limit::InMemoryGatewayTrafficController;
use sdkwork_api_app_runtime::{
    StandaloneListenerHost, StandaloneServiceKind, StandaloneServiceReloadHandles,
    build_admin_payment_store_handles_from_config, build_cache_runtime_from_config,
    resolve_service_runtime_node_id, start_extension_runtime_rollout_supervision,
    start_standalone_runtime_supervision,
};
use sdkwork_api_config::StandaloneConfigLoader;
use sdkwork_api_interface_http::{
    gateway_router_with_state_and_http_exposure, GatewayApiState,
};
use sdkwork_api_observability::init_tracing;
use sdkwork_api_storage_core::Reloadable;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing("gateway-service");
    let (config_loader, config) = StandaloneConfigLoader::from_env()?;
    config.validate_security_posture()?;
    config.apply_to_process_env();
    let cache_runtime = build_cache_runtime_from_config(&config).await?;
    configure_route_decision_cache_store(cache_runtime.cache_store());
    configure_route_recovery_probe_lock_store(cache_runtime.distributed_lock_store());
    if config.cache_backend.supports_shared_cache_coherence() {
        configure_capability_catalog_cache_store(cache_runtime.cache_store());
    }
    let initial_store_handles = build_admin_payment_store_handles_from_config(&config).await?;
    let live_store = Reloadable::new(initial_store_handles.admin_store);
    let live_commercial_billing = Reloadable::new(initial_store_handles.commercial_billing);
    let live_payment_store = Reloadable::new(initial_store_handles.payment_store);
    let live_secret_manager =
        Reloadable::new(CredentialSecretManager::new_with_legacy_master_keys(
            config.secret_backend,
            config.credential_master_key.clone(),
            config.credential_legacy_master_keys.clone(),
            config.secret_local_file.clone(),
            config.secret_keyring_service.clone(),
        ));
    let state =
        GatewayApiState::with_live_store_commercial_billing_payment_store_and_secret_manager_handle(
        live_store.clone(),
        live_commercial_billing.clone(),
        live_payment_store.clone(),
        live_secret_manager.clone(),
        Arc::new(InMemoryGatewayTrafficController::new()),
    );
    let listener_host = StandaloneListenerHost::bind(
        config.gateway_bind.clone(),
        gateway_router_with_state_and_http_exposure(state, config.http_exposure_config()),
    )
    .await?;
    let node_id = resolve_service_runtime_node_id(StandaloneServiceKind::Gateway);
    let _rollout_supervision = start_extension_runtime_rollout_supervision(
        StandaloneServiceKind::Gateway,
        node_id.clone(),
        live_store.clone(),
    )?;
    let _runtime_supervision = start_standalone_runtime_supervision(
        StandaloneServiceKind::Gateway,
        config_loader,
        config.clone(),
        StandaloneServiceReloadHandles::gateway(live_store)
            .with_payment_store(live_payment_store)
            .with_secret_manager(live_secret_manager)
            .with_listener(listener_host.reload_handle())
            .with_node_id(node_id),
    );
    listener_host.wait().await?;
    Ok(())
}
