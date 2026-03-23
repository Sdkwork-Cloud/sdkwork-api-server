use sdkwork_api_storage_core::{
    AdminStore, ExtensionRuntimeRolloutParticipantRecord, ExtensionRuntimeRolloutRecord,
    ServiceRuntimeNodeRecord, StandaloneConfigRolloutParticipantRecord,
    StandaloneConfigRolloutRecord, StorageDialect,
};
use sdkwork_api_storage_sqlite::{run_migrations, SqliteAdminStore};

#[tokio::test]
async fn sqlite_store_implements_admin_store_trait() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);
    let trait_store: &dyn AdminStore = &store;

    assert_eq!(trait_store.dialect(), StorageDialect::Sqlite);
    let channels = trait_store.list_channels().await.unwrap();
    assert!(channels.len() >= 5);
    assert!(channels.iter().any(|channel| channel.id == "openai"));
}

#[tokio::test]
async fn sqlite_store_round_trips_runtime_rollout_records() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);
    let now_ms = 1_234_567_u64;

    store
        .upsert_service_runtime_node(&ServiceRuntimeNodeRecord::new(
            "gateway-node-a",
            "gateway",
            now_ms,
        ))
        .await
        .unwrap();
    store
        .insert_extension_runtime_rollout(&ExtensionRuntimeRolloutRecord::new(
            "rollout-1",
            "extension",
            Some("sdkwork.provider.native.mock".to_owned()),
            None,
            Some("sdkwork.provider.native.mock".to_owned()),
            "admin-user",
            now_ms,
            now_ms + 30_000,
        ))
        .await
        .unwrap();
    store
        .insert_extension_runtime_rollout_participant(
            &ExtensionRuntimeRolloutParticipantRecord::new(
                "rollout-1",
                "gateway-node-a",
                "gateway",
                "pending",
                now_ms,
            ),
        )
        .await
        .unwrap();

    let nodes = store.list_service_runtime_nodes().await.unwrap();
    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0].node_id, "gateway-node-a");

    let rollouts = store.list_extension_runtime_rollouts().await.unwrap();
    assert_eq!(rollouts.len(), 1);
    assert_eq!(rollouts[0].rollout_id, "rollout-1");

    let participants = store
        .list_extension_runtime_rollout_participants("rollout-1")
        .await
        .unwrap();
    assert_eq!(participants.len(), 1);
    assert_eq!(participants[0].node_id, "gateway-node-a");
}

#[tokio::test]
async fn sqlite_store_round_trips_standalone_config_rollout_records() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);
    let now_ms = 7_654_321_u64;

    store
        .insert_standalone_config_rollout(&StandaloneConfigRolloutRecord::new(
            "config-rollout-1",
            Some("portal".to_owned()),
            "admin-user",
            now_ms,
            now_ms + 30_000,
        ))
        .await
        .unwrap();
    store
        .insert_standalone_config_rollout_participant(
            &StandaloneConfigRolloutParticipantRecord::new(
                "config-rollout-1",
                "portal-node-a",
                "portal",
                "pending",
                now_ms,
            ),
        )
        .await
        .unwrap();

    let rollouts = store.list_standalone_config_rollouts().await.unwrap();
    assert_eq!(rollouts.len(), 1);
    assert_eq!(rollouts[0].rollout_id, "config-rollout-1");
    assert_eq!(
        rollouts[0].requested_service_kind.as_deref(),
        Some("portal")
    );

    let participants = store
        .list_standalone_config_rollout_participants("config-rollout-1")
        .await
        .unwrap();
    assert_eq!(participants.len(), 1);
    assert_eq!(participants[0].node_id, "portal-node-a");
}
