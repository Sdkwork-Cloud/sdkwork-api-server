use sdkwork_api_app_identity::CreateGatewayApiKey;
use sdkwork_api_app_identity::{
    hash_gateway_api_key, list_gateway_api_keys, persist_gateway_api_key_with_metadata,
    resolve_gateway_request_context,
};
use sdkwork_api_domain_identity::GatewayApiKeyRecord;
use sdkwork_api_storage_sqlite::{run_migrations, SqliteAdminStore};

#[test]
fn generated_key_has_sdkwork_prefix() {
    let created = CreateGatewayApiKey::execute_with_metadata(
        "tenant-1",
        "project-1",
        "live",
        "Production rollout",
        Some(1_900_000_000_000),
    )
    .unwrap();
    assert!(created.plaintext.starts_with("skw_live_"));
    assert_eq!(created.label, "Production rollout");
    assert_eq!(created.expires_at_ms, Some(1_900_000_000_000));
    assert!(created.created_at_ms > 0);
}

#[tokio::test]
async fn persisted_gateway_api_keys_can_be_listed_with_governance_metadata() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    let created = persist_gateway_api_key_with_metadata(
        &store,
        "tenant-1",
        "project-1",
        "live",
        "Production rollout",
        Some(1_900_000_000_000),
    )
    .await
    .unwrap();

    let keys = list_gateway_api_keys(&store).await.unwrap();

    assert_eq!(keys.len(), 1);
    assert_eq!(keys[0].hashed_key, created.hashed);
    assert_eq!(keys[0].label, "Production rollout");
    assert_eq!(keys[0].created_at_ms, created.created_at_ms);
    assert_eq!(keys[0].expires_at_ms, Some(1_900_000_000_000));
    assert_eq!(keys[0].last_used_at_ms, None);
    assert!(keys[0].active);
}

#[tokio::test]
async fn resolving_gateway_context_updates_last_used_and_rejects_expired_keys() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    let created = persist_gateway_api_key_with_metadata(
        &store,
        "tenant-1",
        "project-1",
        "live",
        "Production rollout",
        None,
    )
    .await
    .unwrap();

    let request_context = resolve_gateway_request_context(&store, &created.plaintext)
        .await
        .unwrap();
    assert!(request_context.is_some());

    let active_record = store
        .find_gateway_api_key(&created.hashed)
        .await
        .unwrap()
        .unwrap();
    assert!(active_record.last_used_at_ms.is_some());

    let expired_plaintext = "expired-plaintext";
    let expired_record = GatewayApiKeyRecord::new(
        "tenant-1",
        "project-1",
        "staging",
        hash_gateway_api_key(expired_plaintext),
    )
    .with_label("Expired staging key")
    .with_created_at_ms(1_700_000_000_000)
    .with_expires_at_ms(1);
    store.insert_gateway_api_key(&expired_record).await.unwrap();

    let expired_request_context = resolve_gateway_request_context(&store, expired_plaintext)
        .await
        .unwrap();
    assert!(expired_request_context.is_none());
}
