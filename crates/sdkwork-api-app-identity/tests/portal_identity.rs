use sdkwork_api_app_identity::{
    change_portal_password, create_portal_api_key, list_portal_api_keys,
    load_portal_workspace_summary, login_portal_user, register_portal_user, verify_portal_jwt,
    PortalIdentityError,
};
use sdkwork_api_storage_sqlite::{run_migrations, SqliteAdminStore};

async fn memory_store() -> SqliteAdminStore {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    SqliteAdminStore::new(pool)
}

#[tokio::test]
async fn portal_registration_login_and_workspace_summary_work() {
    let store = memory_store().await;

    let session = register_portal_user(
        &store,
        "portal@example.com",
        "hunter2!",
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
        "hunter2!",
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
        "hunter2!",
        "Alice",
        "portal-test-secret",
    )
    .await
    .unwrap();
    let bob = register_portal_user(
        &store,
        "bob@example.com",
        "hunter2!",
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
async fn default_portal_login_bootstraps_a_local_demo_user() {
    let store = memory_store().await;

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
