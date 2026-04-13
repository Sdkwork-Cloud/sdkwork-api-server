use sdkwork_api_app_identity::{
    create_api_key_group, delete_api_key_group, list_api_key_groups,
    persist_gateway_api_key_with_metadata, set_api_key_group_active, update_api_key_group,
    ApiKeyGroupInput, PersistGatewayApiKeyInput,
};
use sdkwork_api_domain_routing::{RoutingProfileRecord, RoutingStrategy};
use sdkwork_api_storage_sqlite::{run_migrations, SqliteAdminStore};

#[tokio::test]
async fn create_and_list_api_key_groups_normalize_group_metadata() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    let created = create_api_key_group(
        &store,
        ApiKeyGroupInput {
            tenant_id: "tenant-1".to_owned(),
            project_id: "project-1".to_owned(),
            environment: "live".to_owned(),
            name: " Production Keys ".to_owned(),
            slug: None,
            description: Some(" Primary production pool ".to_owned()),
            color: Some("#2563eb".to_owned()),
            default_capability_scope: Some("chat,responses".to_owned()),
            default_routing_profile_id: None,
            default_accounting_mode: None,
        },
    )
    .await
    .unwrap();

    assert_eq!(created.tenant_id, "tenant-1");
    assert_eq!(created.project_id, "project-1");
    assert_eq!(created.environment, "live");
    assert_eq!(created.name, "Production Keys");
    assert_eq!(created.slug, "production-keys");
    assert_eq!(
        created.description.as_deref(),
        Some("Primary production pool")
    );
    assert_eq!(created.color.as_deref(), Some("#2563eb"));
    assert_eq!(
        created.default_capability_scope.as_deref(),
        Some("chat,responses")
    );
    assert!(created.active);
    assert!(created.created_at_ms > 0);
    assert_eq!(created.updated_at_ms, created.created_at_ms);

    let groups = list_api_key_groups(&store).await.unwrap();
    assert_eq!(groups, vec![created]);
}

#[tokio::test]
async fn create_api_key_group_persists_default_accounting_mode() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    let created = create_api_key_group(
        &store,
        ApiKeyGroupInput {
            tenant_id: "tenant-1".to_owned(),
            project_id: "project-1".to_owned(),
            environment: "live".to_owned(),
            name: "BYOK Keys".to_owned(),
            slug: Some("byok-keys".to_owned()),
            description: None,
            color: None,
            default_capability_scope: None,
            default_routing_profile_id: None,
            default_accounting_mode: Some(" byok ".to_owned()),
        },
    )
    .await
    .unwrap();

    assert_eq!(created.default_accounting_mode.as_deref(), Some("byok"));

    let groups = list_api_key_groups(&store).await.unwrap();
    assert_eq!(groups[0].default_accounting_mode.as_deref(), Some("byok"));
}

#[tokio::test]
async fn api_key_group_rejects_invalid_default_accounting_mode() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    let error = create_api_key_group(
        &store,
        ApiKeyGroupInput {
            tenant_id: "tenant-1".to_owned(),
            project_id: "project-1".to_owned(),
            environment: "live".to_owned(),
            name: "Invalid Accounting".to_owned(),
            slug: Some("invalid-accounting".to_owned()),
            description: None,
            color: None,
            default_capability_scope: None,
            default_routing_profile_id: None,
            default_accounting_mode: Some("credits-plus".to_owned()),
        },
    )
    .await
    .unwrap_err();

    assert_eq!(
        error.to_string(),
        "default_accounting_mode must be one of: platform_credit, byok, passthrough"
    );
}

#[tokio::test]
async fn inactive_api_key_groups_cannot_receive_new_gateway_keys() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    let group = create_api_key_group(
        &store,
        ApiKeyGroupInput {
            tenant_id: "tenant-1".to_owned(),
            project_id: "project-1".to_owned(),
            environment: "live".to_owned(),
            name: "Production Keys".to_owned(),
            slug: Some("production-keys".to_owned()),
            description: None,
            color: None,
            default_capability_scope: None,
            default_routing_profile_id: None,
            default_accounting_mode: None,
        },
    )
    .await
    .unwrap();

    let updated = set_api_key_group_active(&store, &group.group_id, false)
        .await
        .unwrap()
        .unwrap();
    assert!(!updated.active);

    let error = persist_gateway_api_key_with_metadata(
        &store,
        PersistGatewayApiKeyInput {
            tenant_id: "tenant-1",
            project_id: "project-1",
            environment: "live",
            label: "Production rollout",
            expires_at_ms: None,
            plaintext_key: None,
            notes: None,
            api_key_group_id: Some(&group.group_id),
        },
    )
    .await
    .unwrap_err();

    assert_eq!(error.to_string(), "api key group is inactive");
}

#[tokio::test]
async fn updating_api_key_groups_rewrites_metadata_and_keeps_creation_timestamp() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    let created = create_api_key_group(
        &store,
        ApiKeyGroupInput {
            tenant_id: "tenant-1".to_owned(),
            project_id: "project-1".to_owned(),
            environment: "live".to_owned(),
            name: "Production Keys".to_owned(),
            slug: Some("production-keys".to_owned()),
            description: Some("Primary pool".to_owned()),
            color: Some("#2563eb".to_owned()),
            default_capability_scope: Some("chat".to_owned()),
            default_routing_profile_id: None,
            default_accounting_mode: None,
        },
    )
    .await
    .unwrap();

    let updated = update_api_key_group(
        &store,
        &created.group_id,
        ApiKeyGroupInput {
            tenant_id: "tenant-1".to_owned(),
            project_id: "project-1".to_owned(),
            environment: "live".to_owned(),
            name: "Enterprise Keys".to_owned(),
            slug: Some("enterprise-keys".to_owned()),
            description: Some(" Premium pool ".to_owned()),
            color: Some("#0f766e".to_owned()),
            default_capability_scope: Some("responses".to_owned()),
            default_routing_profile_id: None,
            default_accounting_mode: None,
        },
    )
    .await
    .unwrap()
    .unwrap();

    assert_eq!(updated.group_id, created.group_id);
    assert_eq!(updated.name, "Enterprise Keys");
    assert_eq!(updated.slug, "enterprise-keys");
    assert_eq!(updated.description.as_deref(), Some("Premium pool"));
    assert_eq!(updated.color.as_deref(), Some("#0f766e"));
    assert_eq!(
        updated.default_capability_scope.as_deref(),
        Some("responses")
    );
    assert_eq!(updated.created_at_ms, created.created_at_ms);
    assert!(updated.updated_at_ms >= created.updated_at_ms);
}

#[tokio::test]
async fn deleting_api_key_groups_rejects_groups_with_bound_keys() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    let group = create_api_key_group(
        &store,
        ApiKeyGroupInput {
            tenant_id: "tenant-1".to_owned(),
            project_id: "project-1".to_owned(),
            environment: "live".to_owned(),
            name: "Production Keys".to_owned(),
            slug: Some("production-keys".to_owned()),
            description: None,
            color: None,
            default_capability_scope: None,
            default_routing_profile_id: None,
            default_accounting_mode: None,
        },
    )
    .await
    .unwrap();

    persist_gateway_api_key_with_metadata(
        &store,
        PersistGatewayApiKeyInput {
            tenant_id: "tenant-1",
            project_id: "project-1",
            environment: "live",
            label: "Production rollout",
            expires_at_ms: None,
            plaintext_key: None,
            notes: None,
            api_key_group_id: Some(&group.group_id),
        },
    )
    .await
    .unwrap();

    let error = delete_api_key_group(&store, &group.group_id)
        .await
        .unwrap_err();

    assert_eq!(error.to_string(), "api key group has bound api keys");
}

#[tokio::test]
async fn api_key_group_rejects_missing_default_routing_profile_binding() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    let error = create_api_key_group(
        &store,
        ApiKeyGroupInput {
            tenant_id: "tenant-1".to_owned(),
            project_id: "project-1".to_owned(),
            environment: "live".to_owned(),
            name: "Production Keys".to_owned(),
            slug: Some("production-keys".to_owned()),
            description: None,
            color: None,
            default_capability_scope: None,
            default_routing_profile_id: Some("missing-profile".to_owned()),
            default_accounting_mode: None,
        },
    )
    .await
    .unwrap_err();

    assert_eq!(error.to_string(), "routing profile not found");
}

#[tokio::test]
async fn api_key_group_persists_default_routing_profile_binding_for_same_workspace_profile() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    let profile = RoutingProfileRecord::new(
        "profile-live",
        "tenant-1",
        "project-1",
        "Priority Live",
        "priority-live",
    )
    .with_strategy(RoutingStrategy::GeoAffinity)
    .with_created_at_ms(100)
    .with_updated_at_ms(200);
    store.insert_routing_profile(&profile).await.unwrap();

    let created = create_api_key_group(
        &store,
        ApiKeyGroupInput {
            tenant_id: "tenant-1".to_owned(),
            project_id: "project-1".to_owned(),
            environment: "live".to_owned(),
            name: "Production Keys".to_owned(),
            slug: Some("production-keys".to_owned()),
            description: None,
            color: None,
            default_capability_scope: None,
            default_routing_profile_id: Some("profile-live".to_owned()),
            default_accounting_mode: None,
        },
    )
    .await
    .unwrap();

    assert_eq!(
        created.default_routing_profile_id.as_deref(),
        Some("profile-live")
    );

    let updated = update_api_key_group(
        &store,
        &created.group_id,
        ApiKeyGroupInput {
            tenant_id: "tenant-1".to_owned(),
            project_id: "project-1".to_owned(),
            environment: "live".to_owned(),
            name: "Production Keys".to_owned(),
            slug: Some("production-keys".to_owned()),
            description: Some("Bound profile".to_owned()),
            color: None,
            default_capability_scope: None,
            default_routing_profile_id: Some("profile-live".to_owned()),
            default_accounting_mode: None,
        },
    )
    .await
    .unwrap()
    .unwrap();

    assert_eq!(
        updated.default_routing_profile_id.as_deref(),
        Some("profile-live")
    );
}
