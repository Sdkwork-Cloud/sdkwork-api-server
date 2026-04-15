use sdkwork_api_app_identity::{
    change_portal_password, create_portal_api_key, delete_portal_user, list_portal_api_keys,
    list_portal_user_profiles, load_portal_workspace_summary, login_portal_user,
    register_portal_user, upsert_portal_user, verify_portal_jwt, PortalIdentityError,
    UpsertPortalUserInput,
};
use sdkwork_api_domain_tenant::{Project, Tenant};
use sdkwork_api_storage_sqlite::{run_migrations, SqliteAdminStore};

async fn memory_store() -> SqliteAdminStore {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    SqliteAdminStore::new(pool)
}

async fn seed_portal_user(store: &SqliteAdminStore) {
    let _ = register_portal_user(
        store,
        "portal@sdkwork.local",
        "ChangeMe123!",
        "Portal Demo",
        "portal-test-secret",
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn empty_store_does_not_lazy_create_default_portal_user() {
    let store = memory_store().await;

    let error = login_portal_user(
        &store,
        "portal@sdkwork.local",
        "ChangeMe123!",
        "portal-test-secret",
    )
    .await
    .unwrap_err();

    assert!(matches!(error, PortalIdentityError::InvalidCredentials));
    assert!(store.list_portal_users().await.unwrap().is_empty());
}

#[tokio::test]
async fn listing_portal_users_does_not_materialize_default_identity_on_empty_store() {
    let store = memory_store().await;

    let users = list_portal_user_profiles(&store).await.unwrap();

    assert!(users.is_empty());
    assert!(store.list_portal_users().await.unwrap().is_empty());
}

#[tokio::test]
async fn portal_registration_login_and_workspace_summary_work() {
    let store = memory_store().await;

    let session = register_portal_user(
        &store,
        "portal@example.com",
        "PortalPass123!",
        "Portal User",
        "portal-test-secret",
    )
    .await
    .unwrap();

    assert_eq!(session.user.email, "portal@example.com");
    assert_eq!(session.user.display_name, "Portal User");
    assert!(session.workspace.tenant_id.starts_with("tenant_"));
    assert!(session.workspace.project_id.starts_with("project_"));
    assert!(session.token.len() > 10);

    let claims = verify_portal_jwt(&session.token, "portal-test-secret").unwrap();
    assert_eq!(claims.sub, session.user.id);
    assert_eq!(claims.email, "portal@example.com");

    let workspace = load_portal_workspace_summary(&store, &session.user.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(workspace.user.email, "portal@example.com");
    assert_eq!(workspace.tenant.id, session.workspace.tenant_id);
    assert_eq!(workspace.project.id, session.workspace.project_id);

    let login = login_portal_user(
        &store,
        "portal@example.com",
        "PortalPass123!",
        "portal-test-secret",
    )
    .await
    .unwrap();
    assert_eq!(login.user.id, session.user.id);
    assert_eq!(login.workspace, session.workspace);
}

#[tokio::test]
async fn portal_api_key_listing_is_workspace_scoped_and_login_rejects_invalid_password() {
    let store = memory_store().await;

    let alice = register_portal_user(
        &store,
        "alice@example.com",
        "PortalPass123!",
        "Alice",
        "portal-test-secret",
    )
    .await
    .unwrap();
    let bob = register_portal_user(
        &store,
        "bob@example.com",
        "PortalPass123!",
        "Bob",
        "portal-test-secret",
    )
    .await
    .unwrap();

    let alice_error = login_portal_user(
        &store,
        "alice@example.com",
        "wrong-password",
        "portal-test-secret",
    )
    .await
    .unwrap_err();
    assert!(matches!(
        alice_error,
        PortalIdentityError::InvalidCredentials
    ));

    let alice_key = create_portal_api_key(&store, &alice.user.id, "live")
        .await
        .unwrap();
    let bob_key = create_portal_api_key(&store, &bob.user.id, "test")
        .await
        .unwrap();
    assert!(alice_key.plaintext.starts_with("skw_live_"));
    assert!(bob_key.plaintext.starts_with("skw_test_"));

    let alice_keys = list_portal_api_keys(&store, &alice.user.id).await.unwrap();
    assert_eq!(alice_keys.len(), 1);
    assert_eq!(alice_keys[0].environment, "live");
    assert_eq!(alice_keys[0].tenant_id, alice.workspace.tenant_id);
    assert_eq!(alice_keys[0].project_id, alice.workspace.project_id);

    let bob_keys = list_portal_api_keys(&store, &bob.user.id).await.unwrap();
    assert_eq!(bob_keys.len(), 1);
    assert_eq!(bob_keys[0].environment, "test");
    assert_eq!(bob_keys[0].tenant_id, bob.workspace.tenant_id);
    assert_eq!(bob_keys[0].project_id, bob.workspace.project_id);
}

#[tokio::test]
async fn portal_login_returns_explicitly_seeded_demo_user() {
    let store = memory_store().await;
    seed_portal_user(&store).await;

    let session = login_portal_user(
        &store,
        "portal@sdkwork.local",
        "ChangeMe123!",
        "portal-test-secret",
    )
    .await
    .unwrap();

    assert_eq!(session.user.email, "portal@sdkwork.local");
    assert_eq!(session.user.display_name, "Portal Demo");
    assert!(session.workspace.tenant_id.starts_with("tenant_"));
    assert!(session.workspace.project_id.starts_with("project_"));
}

#[tokio::test]
async fn portal_password_change_rejects_old_password_and_accepts_new_password() {
    let store = memory_store().await;
    seed_portal_user(&store).await;

    let session = login_portal_user(
        &store,
        "portal@sdkwork.local",
        "ChangeMe123!",
        "portal-test-secret",
    )
    .await
    .unwrap();

    let updated = change_portal_password(
        &store,
        &session.user.id,
        "ChangeMe123!",
        "PortalPassword456!",
    )
    .await
    .unwrap();
    assert_eq!(updated.email, "portal@sdkwork.local");

    let old_password_error = login_portal_user(
        &store,
        "portal@sdkwork.local",
        "ChangeMe123!",
        "portal-test-secret",
    )
    .await
    .unwrap_err();
    assert!(matches!(
        old_password_error,
        PortalIdentityError::InvalidCredentials
    ));

    let new_session = login_portal_user(
        &store,
        "portal@sdkwork.local",
        "PortalPassword456!",
        "portal-test-secret",
    )
    .await
    .unwrap();
    assert_eq!(new_session.user.id, session.user.id);
}

#[tokio::test]
async fn portal_registration_rejects_weak_passwords() {
    let store = memory_store().await;

    let error = register_portal_user(
        &store,
        "weak@example.com",
        "password1234",
        "Weak User",
        "portal-test-secret",
    )
    .await
    .unwrap_err();

    assert!(matches!(
        error,
        PortalIdentityError::InvalidInput(message)
            if message == "password must include an uppercase letter"
    ));
}

#[tokio::test]
async fn production_bootstrap_workspace_does_not_lazy_create_default_portal_user() {
    let store = memory_store().await;
    store
        .insert_tenant(&Tenant::new(
            "tenant_global_default",
            "Global Default Workspace",
        ))
        .await
        .unwrap();
    store
        .insert_project(&Project::new(
            "tenant_global_default",
            "project_global_default",
            "production-default",
        ))
        .await
        .unwrap();

    let error = login_portal_user(
        &store,
        "portal@sdkwork.local",
        "ChangeMe123!",
        "portal-test-secret",
    )
    .await
    .unwrap_err();

    assert!(matches!(error, PortalIdentityError::InvalidCredentials));
    assert!(store.list_portal_users().await.unwrap().is_empty());
}

#[tokio::test]
async fn explicit_local_demo_portal_user_is_protected_from_deletion() {
    let store = memory_store().await;
    store
        .insert_tenant(&Tenant::new("tenant_local_demo", "Local Demo Workspace"))
        .await
        .unwrap();
    store
        .insert_project(&Project::new(
            "tenant_local_demo",
            "project_local_demo",
            "default",
        ))
        .await
        .unwrap();
    upsert_portal_user(
        &store,
        UpsertPortalUserInput {
            user_id: Some("user_local_demo"),
            email: "portal@sdkwork.local",
            display_name: "Portal Demo",
            password: Some("ChangeMe123!"),
            workspace_tenant_id: "tenant_local_demo",
            workspace_project_id: "project_local_demo",
            active: true,
        },
    )
    .await
    .unwrap();

    let error = delete_portal_user(&store, "user_local_demo")
        .await
        .unwrap_err();

    assert!(matches!(
        error,
        PortalIdentityError::Protected(message)
            if message == "default portal demo user cannot be deleted"
    ));
}
