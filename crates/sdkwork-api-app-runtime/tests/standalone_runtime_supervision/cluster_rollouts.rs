use super::*;

#[tokio::test]
async fn cluster_standalone_config_rollout_rejects_insecure_dev_defaults_on_non_loopback_bind() {
    let shared_store = empty_store().await;
    let gateway_root = temp_root("cluster-config-rollout-gateway-security-posture");
    let gateway_bind = available_bind().replacen("127.0.0.1", "0.0.0.0", 1);

    write_gateway_security_posture_runtime_config(
        &gateway_root,
        &gateway_bind,
        "admin-secret-initial",
        "portal-secret-initial",
        "gateway-master-key-initial",
        false,
    );

    let (gateway_loader, gateway_initial_config) =
        StandaloneConfigLoader::from_local_root_and_pairs(
            &gateway_root,
            std::iter::empty::<(&str, &str)>(),
        )
        .unwrap();

    let gateway_live_store = Reloadable::new(empty_store().await);
    let gateway_supervision = start_standalone_runtime_supervision(
        StandaloneServiceKind::Gateway,
        gateway_loader,
        gateway_initial_config,
        StandaloneServiceReloadHandles::gateway(gateway_live_store)
            .with_coordination_store(shared_store.clone())
            .with_node_id("gateway-node-security-posture"),
    );

    wait_for_service_runtime_node(shared_store.as_ref(), "gateway-node-security-posture").await;

    write_gateway_security_posture_runtime_config(
        &gateway_root,
        &gateway_bind,
        "local-dev-admin-jwt-secret",
        "local-dev-portal-jwt-secret",
        "local-dev-master-key",
        false,
    );

    let rollout = create_standalone_config_rollout(
        shared_store.as_ref(),
        "admin-user",
        CreateStandaloneConfigRolloutRequest::new(Some("gateway".to_owned()), 30),
    )
    .await
    .unwrap();

    wait_for_standalone_config_rollout_status(shared_store.as_ref(), &rollout.rollout_id, "failed")
        .await;

    let rollout = find_standalone_config_rollout(shared_store.as_ref(), &rollout.rollout_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(rollout.participant_count, 1);
    assert_eq!(rollout.participants.len(), 1);
    assert_eq!(rollout.participants[0].status, "failed");
    let message = rollout.participants[0]
        .message
        .as_deref()
        .unwrap_or_default();
    assert!(message.contains("admin_jwt_signing_secret"));
    assert!(message.contains("portal_jwt_signing_secret"));
    assert!(message.contains("credential_master_key"));

    drop(gateway_supervision);
    cleanup_dir(&gateway_root);
}

#[serial(extension_env)]
#[tokio::test]
async fn cluster_runtime_rollout_workers_complete_shared_rollout() {
    let store = empty_store().await;
    let live_store = Reloadable::new(store.clone());
    let now_ms = unix_timestamp_ms();
    store
        .upsert_service_runtime_node(&ServiceRuntimeNodeRecord::new(
            "gateway-node-a",
            "gateway",
            now_ms,
        ))
        .await
        .unwrap();
    store
        .upsert_service_runtime_node(&ServiceRuntimeNodeRecord::new(
            "admin-node-a",
            "admin",
            now_ms,
        ))
        .await
        .unwrap();

    let rollout = create_extension_runtime_rollout(
        store.as_ref(),
        "admin-user",
        sdkwork_api_app_gateway::ConfiguredExtensionHostReloadScope::All,
        30,
    )
    .await
    .unwrap();

    let gateway_worker = start_extension_runtime_rollout_supervision(
        StandaloneServiceKind::Gateway,
        "gateway-node-a",
        live_store.clone(),
    )
    .unwrap();
    let admin_worker = start_extension_runtime_rollout_supervision(
        StandaloneServiceKind::Admin,
        "admin-node-a",
        live_store,
    )
    .unwrap();

    wait_for_extension_runtime_rollout_status(store.as_ref(), &rollout.rollout_id, "succeeded")
        .await;

    let rollout = find_extension_runtime_rollout(store.as_ref(), &rollout.rollout_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(rollout.participant_count, 2);
    assert_eq!(rollout.participants.len(), 2);
    assert_eq!(rollout.participants[0].status, "succeeded");
    assert_eq!(rollout.participants[1].status, "succeeded");

    gateway_worker.abort();
    admin_worker.abort();
}

#[tokio::test]
async fn cluster_runtime_rollout_times_out_when_participant_stays_pending() {
    let store = empty_store().await;
    let now_ms = unix_timestamp_ms();
    store
        .insert_extension_runtime_rollout(&ExtensionRuntimeRolloutRecord::new(
            "rollout-timeout",
            "all",
            None,
            None,
            None,
            "admin-user",
            now_ms - 5_000,
            now_ms - 1,
        ))
        .await
        .unwrap();
    store
        .insert_extension_runtime_rollout_participant(
            &ExtensionRuntimeRolloutParticipantRecord::new(
                "rollout-timeout",
                "gateway-node-a",
                "gateway",
                "pending",
                now_ms - 5_000,
            ),
        )
        .await
        .unwrap();

    let rollout = find_extension_runtime_rollout(store.as_ref(), "rollout-timeout")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(rollout.status, "timed_out");
    assert_eq!(rollout.participant_count, 1);
}

#[serial(runtime_config_env)]
#[tokio::test]
async fn cluster_standalone_config_rollout_workers_apply_shared_reload() {
    let shared_store = empty_store().await;

    let portal_a_root = temp_root("cluster-config-rollout-portal-a");
    let portal_b_root = temp_root("cluster-config-rollout-portal-b");
    let portal_a_initial_db = sqlite_url_for_path(&portal_a_root.join("initial.db"));
    let portal_a_rotated_db = sqlite_url_for_path(&portal_a_root.join("rotated.db"));
    let portal_b_initial_db = sqlite_url_for_path(&portal_b_root.join("initial.db"));
    let portal_b_rotated_db = sqlite_url_for_path(&portal_b_root.join("rotated.db"));

    seed_model_store(&portal_a_initial_db, "portal-a-initial").await;
    seed_model_store(&portal_a_rotated_db, "portal-a-rotated").await;
    seed_model_store(&portal_b_initial_db, "portal-b-initial").await;
    seed_model_store(&portal_b_rotated_db, "portal-b-rotated").await;
    write_portal_runtime_config(
        &portal_a_root,
        &portal_a_initial_db,
        "portal-secret-a-initial",
    );
    write_portal_runtime_config(
        &portal_b_root,
        &portal_b_initial_db,
        "portal-secret-b-initial",
    );

    let (portal_a_loader, portal_a_initial_config) =
        StandaloneConfigLoader::from_local_root_and_pairs(
            &portal_a_root,
            std::iter::empty::<(&str, &str)>(),
        )
        .unwrap();
    let (portal_b_loader, portal_b_initial_config) =
        StandaloneConfigLoader::from_local_root_and_pairs(
            &portal_b_root,
            std::iter::empty::<(&str, &str)>(),
        )
        .unwrap();

    let portal_a_live_store =
        Reloadable::new(seed_model_store(&portal_a_initial_db, "portal-a-initial").await);
    let portal_b_live_store =
        Reloadable::new(seed_model_store(&portal_b_initial_db, "portal-b-initial").await);
    let portal_a_live_jwt = Reloadable::new("portal-secret-a-initial".to_owned());
    let portal_b_live_jwt = Reloadable::new("portal-secret-b-initial".to_owned());

    let portal_a_supervision = start_standalone_runtime_supervision(
        StandaloneServiceKind::Portal,
        portal_a_loader,
        portal_a_initial_config,
        StandaloneServiceReloadHandles::portal(
            portal_a_live_store.clone(),
            portal_a_live_jwt.clone(),
        )
        .with_coordination_store(shared_store.clone())
        .with_node_id("portal-node-a"),
    );
    let portal_b_supervision = start_standalone_runtime_supervision(
        StandaloneServiceKind::Portal,
        portal_b_loader,
        portal_b_initial_config,
        StandaloneServiceReloadHandles::portal(
            portal_b_live_store.clone(),
            portal_b_live_jwt.clone(),
        )
        .with_coordination_store(shared_store.clone())
        .with_node_id("portal-node-b"),
    );

    wait_for_service_runtime_node(shared_store.as_ref(), "portal-node-a").await;
    wait_for_service_runtime_node(shared_store.as_ref(), "portal-node-b").await;

    write_portal_runtime_config(
        &portal_a_root,
        &portal_a_rotated_db,
        "portal-secret-a-rotated",
    );
    write_portal_runtime_config(
        &portal_b_root,
        &portal_b_rotated_db,
        "portal-secret-b-rotated",
    );

    let rollout = create_standalone_config_rollout(
        shared_store.as_ref(),
        "admin-user",
        CreateStandaloneConfigRolloutRequest::new(Some("portal".to_owned()), 30),
    )
    .await
    .unwrap();

    wait_for_standalone_config_rollout_status(
        shared_store.as_ref(),
        &rollout.rollout_id,
        "succeeded",
    )
    .await;
    wait_for_models(&portal_a_live_store, &["portal-a-rotated"]).await;
    wait_for_models(&portal_b_live_store, &["portal-b-rotated"]).await;
    wait_for_reloadable_string(&portal_a_live_jwt, "portal-secret-a-rotated").await;
    wait_for_reloadable_string(&portal_b_live_jwt, "portal-secret-b-rotated").await;

    let rollout = find_standalone_config_rollout(shared_store.as_ref(), &rollout.rollout_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(rollout.participant_count, 2);
    assert_eq!(rollout.participants.len(), 2);
    assert_eq!(rollout.participants[0].status, "succeeded");
    assert_eq!(rollout.participants[1].status, "succeeded");

    drop(portal_a_supervision);
    drop(portal_b_supervision);
    cleanup_dir(&portal_a_root);
    cleanup_dir(&portal_b_root);
}

#[serial(runtime_config_env)]
#[tokio::test]
async fn cluster_standalone_config_rollout_fails_when_only_cache_backend_change_requires_restart() {
    let shared_store = empty_store().await;
    let gateway_root = temp_root("cluster-config-rollout-gateway-cache-restart");
    let gateway_bind = available_bind();

    write_gateway_runtime_config(&gateway_root, &gateway_bind);

    let (gateway_loader, gateway_initial_config) =
        StandaloneConfigLoader::from_local_root_and_pairs(
            &gateway_root,
            std::iter::empty::<(&str, &str)>(),
        )
        .unwrap();

    let gateway_live_store = Reloadable::new(empty_store().await);
    let gateway_supervision = start_standalone_runtime_supervision(
        StandaloneServiceKind::Gateway,
        gateway_loader,
        gateway_initial_config,
        StandaloneServiceReloadHandles::gateway(gateway_live_store)
            .with_coordination_store(shared_store.clone())
            .with_node_id("gateway-node-cache-restart"),
    );

    wait_for_service_runtime_node(shared_store.as_ref(), "gateway-node-cache-restart").await;

    write_gateway_runtime_config_with_cache(
        &gateway_root,
        &gateway_bind,
        CacheBackendKind::Redis,
        Some("redis://127.0.0.1:6379/12"),
    );

    let rollout = create_standalone_config_rollout(
        shared_store.as_ref(),
        "admin-user",
        CreateStandaloneConfigRolloutRequest::new(Some("gateway".to_owned()), 30),
    )
    .await
    .unwrap();

    wait_for_standalone_config_rollout_status(shared_store.as_ref(), &rollout.rollout_id, "failed")
        .await;

    let rollout = find_standalone_config_rollout(shared_store.as_ref(), &rollout.rollout_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(rollout.participant_count, 1);
    assert_eq!(rollout.participants.len(), 1);
    assert_eq!(rollout.participants[0].status, "failed");
    let message = rollout.participants[0]
        .message
        .as_deref()
        .unwrap_or_default();
    assert!(message.contains("restart required"));
    assert!(message.contains("cache_backend"));
    assert!(message.contains("cache_url"));

    drop(gateway_supervision);
    cleanup_dir(&gateway_root);
}

#[serial(runtime_config_env)]
#[tokio::test]
async fn cluster_standalone_config_rollout_applies_database_reload_but_fails_when_cache_backend_also_requires_restart(
) {
    let shared_store = empty_store().await;
    let gateway_root = temp_root("cluster-config-rollout-gateway-store-and-cache-restart");
    let gateway_bind = available_bind();
    let initial_db_url = sqlite_url_for_path(&gateway_root.join("initial.db"));
    let rotated_db_url = sqlite_url_for_path(&gateway_root.join("rotated.db"));

    seed_model_store(&initial_db_url, "gateway-initial").await;
    seed_model_store(&rotated_db_url, "gateway-rotated").await;
    write_gateway_store_runtime_config_with_cache(
        &gateway_root,
        &gateway_bind,
        &initial_db_url,
        CacheBackendKind::Memory,
        None,
    );

    let (gateway_loader, gateway_initial_config) =
        StandaloneConfigLoader::from_local_root_and_pairs(
            &gateway_root,
            std::iter::empty::<(&str, &str)>(),
        )
        .unwrap();

    let gateway_live_store =
        Reloadable::new(seed_model_store(&initial_db_url, "gateway-initial").await);
    let gateway_supervision = start_standalone_runtime_supervision(
        StandaloneServiceKind::Gateway,
        gateway_loader,
        gateway_initial_config,
        StandaloneServiceReloadHandles::gateway(gateway_live_store.clone())
            .with_coordination_store(shared_store.clone())
            .with_node_id("gateway-node-store-and-cache-restart"),
    );

    wait_for_service_runtime_node(
        shared_store.as_ref(),
        "gateway-node-store-and-cache-restart",
    )
    .await;

    write_gateway_store_runtime_config_with_cache(
        &gateway_root,
        &gateway_bind,
        &rotated_db_url,
        CacheBackendKind::Redis,
        Some("redis://127.0.0.1:6379/13"),
    );

    let rollout = create_standalone_config_rollout(
        shared_store.as_ref(),
        "admin-user",
        CreateStandaloneConfigRolloutRequest::new(Some("gateway".to_owned()), 30),
    )
    .await
    .unwrap();

    wait_for_standalone_config_rollout_status(shared_store.as_ref(), &rollout.rollout_id, "failed")
        .await;
    wait_for_models(&gateway_live_store, &["gateway-rotated"]).await;

    let rollout = find_standalone_config_rollout(shared_store.as_ref(), &rollout.rollout_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(rollout.participant_count, 1);
    assert_eq!(rollout.participants.len(), 1);
    assert_eq!(rollout.participants[0].status, "failed");
    let message = rollout.participants[0]
        .message
        .as_deref()
        .unwrap_or_default();
    assert!(message.contains("database_changed=true"));
    assert!(message.contains("restart required"));
    assert!(message.contains("cache_backend"));

    drop(gateway_supervision);
    cleanup_dir(&gateway_root);
}

#[tokio::test]
async fn cluster_standalone_config_rollout_times_out_when_participant_stays_pending() {
    let store = empty_store().await;
    let now_ms = unix_timestamp_ms();
    store
        .insert_standalone_config_rollout(&StandaloneConfigRolloutRecord::new(
            "config-rollout-timeout",
            Some("portal".to_owned()),
            "admin-user",
            now_ms - 5_000,
            now_ms - 1,
        ))
        .await
        .unwrap();
    store
        .insert_standalone_config_rollout_participant(
            &StandaloneConfigRolloutParticipantRecord::new(
                "config-rollout-timeout",
                "portal-node-a",
                "portal",
                "pending",
                now_ms - 5_000,
            ),
        )
        .await
        .unwrap();

    let rollout = find_standalone_config_rollout(store.as_ref(), "config-rollout-timeout")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(rollout.status, "timed_out");
    assert_eq!(rollout.participant_count, 1);
}
