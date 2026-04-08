use sdkwork_api_app_credential::CredentialSecretManager;
use sdkwork_api_app_runtime::{
    build_admin_store_and_commercial_billing_from_config, build_cache_runtime_from_config,
    resolve_service_runtime_node_id, start_standalone_runtime_supervision,
    StandaloneListenerHost, StandaloneServiceKind, StandaloneServiceReloadHandles,
};
use sdkwork_api_config::StandaloneConfigLoader;
use sdkwork_api_interface_portal::{portal_router_with_state_and_http_exposure, PortalApiState};
use sdkwork_api_observability::init_tracing;
use sdkwork_api_storage_core::Reloadable;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing("portal-api-service");
    let (config_loader, config) = StandaloneConfigLoader::from_env()?;
    config.validate_security_posture()?;
    config.apply_to_process_env();
    let _cache_runtime = build_cache_runtime_from_config(&config).await?;
    let (store, commercial_billing) =
        build_admin_store_and_commercial_billing_from_config(&config).await?;
    let live_store = Reloadable::new(store);
    let live_commercial_billing = Reloadable::new(commercial_billing);
    let live_portal_jwt = Reloadable::new(config.portal_jwt_signing_secret.clone());
    let live_secret_manager =
        Reloadable::new(CredentialSecretManager::new_with_legacy_master_keys(
            config.secret_backend,
            config.credential_master_key.clone(),
            config.credential_legacy_master_keys.clone(),
            config.secret_local_file.clone(),
            config.secret_keyring_service.clone(),
        ));
    let state =
        PortalApiState::with_live_store_secret_manager_commercial_billing_and_jwt_secret_handle(
        live_store.clone(),
        live_secret_manager.clone(),
        Some(live_commercial_billing.clone()),
        live_portal_jwt.clone(),
    );
    let listener_host =
        StandaloneListenerHost::bind(
            config.portal_bind.clone(),
            portal_router_with_state_and_http_exposure(state, config.http_exposure_config()),
        )
        .await?;
    let node_id = resolve_service_runtime_node_id(StandaloneServiceKind::Portal);
    let _runtime_supervision = start_standalone_runtime_supervision(
        StandaloneServiceKind::Portal,
        config_loader,
        config.clone(),
        StandaloneServiceReloadHandles::portal(live_store, live_portal_jwt)
            .with_live_commercial_billing(live_commercial_billing)
            .with_secret_manager(live_secret_manager)
            .with_listener(listener_host.reload_handle())
            .with_node_id(node_id),
    );
    listener_host.wait().await?;
    Ok(())
}
