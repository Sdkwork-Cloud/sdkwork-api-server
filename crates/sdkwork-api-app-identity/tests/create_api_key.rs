use sdkwork_api_app_identity::CreateGatewayApiKey;
use sdkwork_api_app_identity::{
    hash_gateway_api_key, list_gateway_api_keys, persist_gateway_api_key_with_metadata,
    resolve_gateway_auth_subject_from_api_key, resolve_gateway_request_context,
};
use sdkwork_api_domain_identity::{
    ApiKeyGroupRecord, CanonicalApiKeyRecord, GatewayApiKeyRecord, GatewayAuthType,
    IdentityUserRecord,
};
use sdkwork_api_storage_core::IdentityKernelStore;
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
        None,
        Some("Gateway launch credential"),
        None,
    )
    .await
    .unwrap();

    let keys = list_gateway_api_keys(&store).await.unwrap();

    assert_eq!(keys.len(), 1);
    assert_eq!(keys[0].hashed_key, created.hashed);
    assert_eq!(keys[0].label, "Production rollout");
    assert_eq!(keys[0].notes.as_deref(), Some("Gateway launch credential"));
    assert_eq!(keys[0].created_at_ms, created.created_at_ms);
    assert_eq!(keys[0].expires_at_ms, Some(1_900_000_000_000));
    assert_eq!(keys[0].last_used_at_ms, None);
    assert!(keys[0].active);
}

#[tokio::test]
async fn persisted_gateway_api_keys_do_not_retain_plaintext_after_create() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    let created = persist_gateway_api_key_with_metadata(
        &store,
        "tenant-1",
        "project-1",
        "live",
        "Production rollout",
        Some(1_900_000_000_000),
        None,
        Some("Gateway launch credential"),
        None,
    )
    .await
    .unwrap();

    let persisted = store
        .find_gateway_api_key(&created.hashed)
        .await
        .unwrap()
        .unwrap();
    let persisted_json = serde_json::to_value(&persisted).unwrap();
    assert!(persisted_json.get("raw_key").is_none());

    let listed = list_gateway_api_keys(&store).await.unwrap();
    let listed_json = serde_json::to_value(&listed[0]).unwrap();
    assert!(listed_json.get("raw_key").is_none());
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
        None,
        None,
        None,
    )
    .await
    .unwrap();

    let request_context = resolve_gateway_request_context(&store, &created.plaintext)
        .await
        .unwrap();
    assert!(request_context.is_some());
    assert_eq!(
        request_context
            .as_ref()
            .and_then(|context| context.api_key_group_id.as_deref()),
        None
    );

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

#[test]
fn custom_plaintext_key_can_be_preserved_during_creation() {
    let created = CreateGatewayApiKey::execute_with_optional_plaintext(
        "tenant-1",
        "project-1",
        "live",
        "Custom rollout",
        Some(1_900_000_000_000),
        Some("skw_live_custom_portal_secret"),
        Some("Operator supplied migration key"),
    )
    .unwrap();

    assert_eq!(created.plaintext, "skw_live_custom_portal_secret");
    assert_eq!(
        created.hashed,
        hash_gateway_api_key("skw_live_custom_portal_secret")
    );
    assert_eq!(created.label, "Custom rollout");
    assert_eq!(
        created.notes.as_deref(),
        Some("Operator supplied migration key")
    );
}

#[tokio::test]
async fn persisted_gateway_api_key_can_be_bound_to_an_api_key_group() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    let group = ApiKeyGroupRecord::new(
        "group-live",
        "tenant-1",
        "project-1",
        "live",
        "Production keys",
        "production-keys",
    )
    .with_description("Primary production key pool")
    .with_created_at_ms(1_700_000_000_000)
    .with_updated_at_ms(1_700_000_000_000);
    store.insert_api_key_group(&group).await.unwrap();

    let created = persist_gateway_api_key_with_metadata(
        &store,
        "tenant-1",
        "project-1",
        "live",
        "Production rollout",
        Some(1_900_000_000_000),
        None,
        Some("Gateway launch credential"),
        Some(&group.group_id),
    )
    .await
    .unwrap();

    assert_eq!(created.api_key_group_id.as_deref(), Some("group-live"));

    let persisted = store
        .find_gateway_api_key(&created.hashed)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(persisted.api_key_group_id.as_deref(), Some("group-live"));

    let request_context = resolve_gateway_request_context(&store, &created.plaintext)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        request_context.api_key_group_id.as_deref(),
        Some("group-live")
    );
}

#[tokio::test]
async fn persisted_gateway_api_key_rejects_api_key_group_with_mismatched_environment() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    let group = ApiKeyGroupRecord::new(
        "group-staging",
        "tenant-1",
        "project-1",
        "staging",
        "Staging keys",
        "staging-keys",
    )
    .with_created_at_ms(1_700_000_000_000)
    .with_updated_at_ms(1_700_000_000_000);
    store.insert_api_key_group(&group).await.unwrap();

    let error = persist_gateway_api_key_with_metadata(
        &store,
        "tenant-1",
        "project-1",
        "live",
        "Production rollout",
        None,
        None,
        None,
        Some(&group.group_id),
    )
    .await
    .unwrap_err();

    assert_eq!(
        error.to_string(),
        "api key group environment does not match"
    );
}

#[tokio::test]
async fn resolving_gateway_auth_subject_uses_canonical_api_key_scope() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);
    let plaintext = "skw_live_canonical_identity";
    let hashed_key = hash_gateway_api_key(plaintext);

    store
        .insert_identity_user_record(
            &IdentityUserRecord::new(9001, 1001, 2002)
                .with_display_name(Some("Portal User".to_owned()))
                .with_created_at_ms(10)
                .with_updated_at_ms(10),
        )
        .await
        .unwrap();
    store
        .insert_canonical_api_key_record(
            &CanonicalApiKeyRecord::new(778899, 1001, 2002, 9001, &hashed_key)
                .with_key_prefix("skw_live")
                .with_display_name("Canonical key")
                .with_created_at_ms(20)
                .with_updated_at_ms(20),
        )
        .await
        .unwrap();

    let subject = resolve_gateway_auth_subject_from_api_key(&store, plaintext)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(subject.tenant_id, 1001);
    assert_eq!(subject.organization_id, 2002);
    assert_eq!(subject.user_id, 9001);
    assert_eq!(subject.api_key_id, Some(778899));
    assert_eq!(subject.api_key_hash.as_deref(), Some(hashed_key.as_str()));
    assert_eq!(subject.auth_type, GatewayAuthType::ApiKey);

    let persisted = store
        .find_canonical_api_key_record_by_hash(&hashed_key)
        .await
        .unwrap()
        .unwrap();
    assert!(persisted.last_used_at_ms.is_some());
}
