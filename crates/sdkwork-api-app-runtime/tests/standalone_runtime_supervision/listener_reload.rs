use super::*;
use sdkwork_api_app_billing::GatewayCommercialBillingKernel;
use sdkwork_api_domain_billing::{
    AccountBenefitLotRecord, AccountBenefitSourceType, AccountBenefitType, AccountRecord,
    AccountStatus, AccountType,
};
use sdkwork_api_storage_core::AccountKernelStore;

#[tokio::test]
async fn standalone_listener_host_rebinds_requests_to_new_bind() {
    let initial_bind = available_bind();
    let next_bind = available_bind();
    let host =
        StandaloneListenerHost::bind(initial_bind.clone(), health_router("listener-host-initial"))
            .await
            .unwrap();

    wait_for_health_response(&initial_bind, "listener-host-initial").await;
    host.reload_handle()
        .rebind(next_bind.clone())
        .await
        .unwrap();

    wait_for_health_response(&next_bind, "listener-host-initial").await;
    wait_for_health_unreachable(&initial_bind).await;

    host.shutdown().await.unwrap();
}

#[tokio::test]
async fn standalone_listener_host_reports_actual_ephemeral_bind() {
    let host =
        StandaloneListenerHost::bind("127.0.0.1:0", health_router("listener-host-ephemeral"))
            .await
            .unwrap();

    let actual_bind = host.current_bind().unwrap();
    assert_ne!(actual_bind, "127.0.0.1:0");
    assert!(actual_bind.starts_with("127.0.0.1:"));

    wait_for_health_response(&actual_bind, "listener-host-ephemeral").await;

    host.shutdown().await.unwrap();
}

#[serial(extension_env)]
#[tokio::test]
async fn standalone_runtime_supervision_reloads_extension_runtime_after_config_file_change() {
    shutdown_all_native_dynamic_runtimes().unwrap();

    let log_guard = NativeDynamicLifecycleLogGuard::new();
    let config_root = temp_root("runtime-config-reload");
    let extension_root = config_root.join("extensions");
    let package_dir = extension_root.join("sdkwork-provider-native-mock");
    fs::create_dir_all(&package_dir).unwrap();

    let library_path = native_dynamic_fixture_library_path();
    let manifest = native_dynamic_manifest(&library_path);
    let manifest_text = toml::to_string(&manifest).unwrap();
    fs::write(package_dir.join("sdkwork-extension.toml"), &manifest_text).unwrap();

    write_runtime_config(&config_root, true, &extension_root);

    let (loader, initial_config) = StandaloneConfigLoader::from_local_root_and_pairs(
        &config_root,
        std::iter::empty::<(&str, &str)>(),
    )
    .unwrap();
    initial_config.apply_to_process_env();

    let first = reload_configured_extension_host().unwrap();
    assert_eq!(first.discovered_package_count, 1);
    assert_eq!(first.loadable_package_count, 1);
    wait_for_lifecycle_log(log_guard.path(), &["init"]).await;

    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store: Arc<dyn AdminStore> = Arc::new(SqliteAdminStore::new(pool));
    let supervision = start_standalone_runtime_supervision(
        StandaloneServiceKind::Gateway,
        loader,
        initial_config,
        StandaloneServiceReloadHandles::gateway(Reloadable::new(store)),
    );

    write_runtime_config(&config_root, false, &extension_root);
    wait_for_lifecycle_log(log_guard.path(), &["init", "shutdown"]).await;

    write_runtime_config(&config_root, true, &extension_root);
    wait_for_lifecycle_log(log_guard.path(), &["init", "shutdown", "init"]).await;

    drop(supervision);
    shutdown_all_native_dynamic_runtimes().unwrap();
    cleanup_dir(&config_root);
}

#[tokio::test]
async fn standalone_runtime_supervision_rebinds_listener_after_config_file_change() {
    let config_root = temp_root("runtime-listener-rebind");
    let initial_bind = available_bind();
    let next_bind = available_bind();

    write_gateway_runtime_config(&config_root, &initial_bind);

    let (loader, initial_config) = StandaloneConfigLoader::from_local_root_and_pairs(
        &config_root,
        std::iter::empty::<(&str, &str)>(),
    )
    .unwrap();
    initial_config.apply_to_process_env();

    let live_store = Reloadable::new(empty_store().await);
    let listener_host =
        StandaloneListenerHost::bind(initial_bind.clone(), health_router("gateway-runtime"))
            .await
            .unwrap();
    let supervision = start_standalone_runtime_supervision(
        StandaloneServiceKind::Gateway,
        loader,
        initial_config,
        StandaloneServiceReloadHandles::gateway(live_store)
            .with_listener(listener_host.reload_handle()),
    );

    wait_for_health_response(&initial_bind, "gateway-runtime").await;
    write_gateway_runtime_config(&config_root, &next_bind);

    wait_for_health_response(&next_bind, "gateway-runtime").await;
    wait_for_health_unreachable(&initial_bind).await;

    drop(supervision);
    listener_host.shutdown().await.unwrap();
    cleanup_dir(&config_root);
}

#[serial(runtime_config_env)]
#[tokio::test]
async fn standalone_runtime_supervision_retries_listener_rebind_after_bind_failure() {
    let config_root = temp_root("runtime-listener-rebind-retry");
    let initial_bind = available_bind();
    let occupied_bind = available_bind();
    let occupied_listener = std::net::TcpListener::bind(&occupied_bind).unwrap();

    write_gateway_runtime_config(&config_root, &initial_bind);

    let (loader, initial_config) = StandaloneConfigLoader::from_local_root_and_pairs(
        &config_root,
        std::iter::empty::<(&str, &str)>(),
    )
    .unwrap();
    initial_config.apply_to_process_env();

    let live_store = Reloadable::new(empty_store().await);
    let listener_host =
        StandaloneListenerHost::bind(initial_bind.clone(), health_router("gateway-runtime"))
            .await
            .unwrap();
    let supervision = start_standalone_runtime_supervision(
        StandaloneServiceKind::Gateway,
        loader,
        initial_config,
        StandaloneServiceReloadHandles::gateway(live_store)
            .with_listener(listener_host.reload_handle()),
    );

    wait_for_health_response(&initial_bind, "gateway-runtime").await;
    write_gateway_runtime_config(&config_root, &occupied_bind);

    sleep(Duration::from_millis(1200)).await;
    wait_for_health_response(&initial_bind, "gateway-runtime").await;

    drop(occupied_listener);

    wait_for_health_response(&occupied_bind, "gateway-runtime").await;
    wait_for_health_unreachable(&initial_bind).await;

    drop(supervision);
    listener_host.shutdown().await.unwrap();
    cleanup_dir(&config_root);
}

#[serial(runtime_config_env)]
#[tokio::test]
async fn standalone_runtime_supervision_reloads_store_and_jwt_after_config_file_change() {
    let config_root = temp_root("runtime-store-jwt-reload");
    let initial_db_path = config_root.join("initial.db");
    let rotated_db_path = config_root.join("rotated.db");
    let initial_db_url = sqlite_url_for_path(&initial_db_path);
    let rotated_db_url = sqlite_url_for_path(&rotated_db_path);

    seed_model_store(&initial_db_url, "gpt-4.1-initial").await;
    seed_model_store(&rotated_db_url, "gpt-4.1-rotated").await;
    write_portal_runtime_config(&config_root, &initial_db_url, "portal-secret-initial");

    let (loader, initial_config) = StandaloneConfigLoader::from_local_root_and_pairs(
        &config_root,
        std::iter::empty::<(&str, &str)>(),
    )
    .unwrap();
    initial_config.apply_to_process_env();

    let initial_store = seed_model_store(&initial_db_url, "gpt-4.1-initial").await;
    let live_store = Reloadable::new(initial_store);
    let live_portal_jwt = Reloadable::new("portal-secret-initial".to_owned());
    let supervision = start_standalone_runtime_supervision(
        StandaloneServiceKind::Portal,
        loader,
        initial_config,
        StandaloneServiceReloadHandles::portal(live_store.clone(), live_portal_jwt.clone()),
    );

    write_portal_runtime_config(&config_root, &rotated_db_url, "portal-secret-rotated");

    wait_for_models(&live_store, &["gpt-4.1-rotated"]).await;
    wait_for_reloadable_string(&live_portal_jwt, "portal-secret-rotated").await;

    drop(supervision);
    cleanup_dir(&config_root);
}

#[serial(runtime_config_env)]
#[tokio::test]
async fn standalone_runtime_supervision_reloads_gateway_billing_handle_after_database_change() {
    let config_root = temp_root("runtime-gateway-billing-reload");
    let initial_bind = available_bind();
    let initial_db_path = config_root.join("initial.db");
    let rotated_db_path = config_root.join("rotated.db");
    let initial_db_url = sqlite_url_for_path(&initial_db_path);
    let rotated_db_url = sqlite_url_for_path(&rotated_db_path);

    seed_gateway_billing_store(&initial_db_url, 8801, 9901).await;
    seed_gateway_billing_store(&rotated_db_url, 8802, 9902).await;
    write_gateway_store_runtime_config_with_cache(
        &config_root,
        &initial_bind,
        &initial_db_url,
        CacheBackendKind::Memory,
        None,
    );

    let (loader, initial_config) = StandaloneConfigLoader::from_local_root_and_pairs(
        &config_root,
        std::iter::empty::<(&str, &str)>(),
    )
    .unwrap();
    initial_config.apply_to_process_env();

    let initial_store_handles =
        sdkwork_api_app_runtime::build_admin_payment_store_handles_from_config(&initial_config)
            .await
            .unwrap();
    let live_store = Reloadable::new(initial_store_handles.admin_store);
    let live_gateway_commercial_billing =
        Reloadable::new(initial_store_handles.gateway_commercial_billing);
    let supervision = start_standalone_runtime_supervision(
        StandaloneServiceKind::Gateway,
        loader,
        initial_config,
        StandaloneServiceReloadHandles::gateway(live_store)
            .with_live_gateway_commercial_billing(live_gateway_commercial_billing.clone()),
    );

    wait_for_gateway_hold_plan(&live_gateway_commercial_billing, 8801).await;
    write_gateway_store_runtime_config_with_cache(
        &config_root,
        &initial_bind,
        &rotated_db_url,
        CacheBackendKind::Memory,
        None,
    );
    wait_for_gateway_hold_plan(&live_gateway_commercial_billing, 8802).await;

    drop(supervision);
    cleanup_dir(&config_root);
}

#[tokio::test]
async fn standalone_runtime_supervision_continues_heartbeat_in_reloaded_database() {
    let config_root = temp_root("runtime-heartbeat-store-reload");
    let initial_db_path = config_root.join("initial.db");
    let rotated_db_path = config_root.join("rotated.db");
    let initial_db_url = sqlite_url_for_path(&initial_db_path);
    let rotated_db_url = sqlite_url_for_path(&rotated_db_path);

    seed_model_store(&initial_db_url, "gpt-4.1-initial").await;
    seed_model_store(&rotated_db_url, "gpt-4.1-rotated").await;
    write_portal_runtime_config(&config_root, &initial_db_url, "portal-secret-initial");

    let (loader, initial_config) = StandaloneConfigLoader::from_local_root_and_pairs(
        &config_root,
        std::iter::empty::<(&str, &str)>(),
    )
    .unwrap();
    initial_config.apply_to_process_env();

    let live_store = Reloadable::new(seed_model_store(&initial_db_url, "gpt-4.1-initial").await);
    let live_portal_jwt = Reloadable::new("portal-secret-initial".to_owned());
    let supervision = start_standalone_runtime_supervision(
        StandaloneServiceKind::Portal,
        loader,
        initial_config,
        StandaloneServiceReloadHandles::portal(live_store.clone(), live_portal_jwt)
            .with_node_id("portal-node-rotating"),
    );

    let initial_store = live_store.snapshot();
    wait_for_service_runtime_node(initial_store.as_ref(), "portal-node-rotating").await;

    write_portal_runtime_config(&config_root, &rotated_db_url, "portal-secret-rotated");

    wait_for_models(&live_store, &["gpt-4.1-rotated"]).await;
    let rotated_store = live_store.snapshot();
    wait_for_service_runtime_node(rotated_store.as_ref(), "portal-node-rotating").await;

    drop(supervision);
    cleanup_dir(&config_root);
}

async fn seed_gateway_billing_store(database_url: &str, account_id: u64, lot_id: u64) {
    if let Some(path) = sqlite_path_from_url(database_url) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        let _ = fs::File::create(path).unwrap();
    }

    let pool = run_migrations(database_url).await.unwrap();
    let store = SqliteAdminStore::new(pool);
    let account = AccountRecord::new(account_id, 1001, 2002, 9001, AccountType::Primary)
        .with_status(AccountStatus::Active)
        .with_created_at_ms(10)
        .with_updated_at_ms(10);
    let lot = AccountBenefitLotRecord::new(
        lot_id,
        1001,
        2002,
        account_id,
        9001,
        AccountBenefitType::CashCredit,
    )
    .with_source_type(AccountBenefitSourceType::Recharge)
    .with_original_quantity(10.0)
    .with_remaining_quantity(10.0)
    .with_created_at_ms(11)
    .with_updated_at_ms(11);

    store.insert_account_record(&account).await.unwrap();
    store.insert_account_benefit_lot(&lot).await.unwrap();
}

async fn wait_for_gateway_hold_plan(
    live_gateway_commercial_billing: &Reloadable<Arc<dyn GatewayCommercialBillingKernel>>,
    account_id: u64,
) {
    for _ in 0..200 {
        match live_gateway_commercial_billing
            .snapshot()
            .plan_account_hold(account_id, 1.0, 100)
            .await
        {
            Ok(plan)
                if plan.account_id == account_id
                    && plan.sufficient_balance
                    && !plan.allocations.is_empty() =>
            {
                return;
            }
            _ => {}
        }
        sleep(Duration::from_millis(25)).await;
    }

    panic!("gateway billing handle did not reach expected account {account_id}");
}
