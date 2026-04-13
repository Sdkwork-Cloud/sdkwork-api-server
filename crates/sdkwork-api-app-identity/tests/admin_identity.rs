use sdkwork_api_app_identity::{
    change_admin_password, load_admin_user_profile, login_admin_user, verify_jwt,
};
use sdkwork_api_domain_identity::AdminUserRole;
use sdkwork_api_domain_tenant::{Project, Tenant};
use sdkwork_api_storage_sqlite::{run_migrations, SqliteAdminStore};

async fn memory_store() -> SqliteAdminStore {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    SqliteAdminStore::new(pool)
}

#[tokio::test]
async fn default_admin_login_bootstraps_profile_and_jwt() {
    let store = memory_store().await;

    let session = login_admin_user(
        &store,
        "admin@sdkwork.local",
        "ChangeMe123!",
        "admin-test-secret",
    )
    .await
    .unwrap();

    assert_eq!(session.user.email, "admin@sdkwork.local");
    assert_eq!(session.user.display_name, "Admin Operator");
    assert_eq!(session.user.role, AdminUserRole::SuperAdmin);
    assert!(session.user.active);
    assert!(session.token.len() > 10);

    let claims = verify_jwt(&session.token, "admin-test-secret").unwrap();
    assert_eq!(claims.sub, session.user.id);
    assert_eq!(claims.role, AdminUserRole::SuperAdmin);

    let user = load_admin_user_profile(&store, &session.user.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(user.email, "admin@sdkwork.local");
    assert_eq!(user.role, AdminUserRole::SuperAdmin);
}

#[tokio::test]
async fn admin_password_change_rejects_old_password_and_accepts_new_password() {
    let store = memory_store().await;

    let session = login_admin_user(
        &store,
        "admin@sdkwork.local",
        "ChangeMe123!",
        "admin-test-secret",
    )
    .await
    .unwrap();

    let updated = change_admin_password(
        &store,
        &session.user.id,
        "ChangeMe123!",
        "AdminPassword456!",
    )
    .await
    .unwrap();
    assert_eq!(updated.email, "admin@sdkwork.local");

    let old_password_error = login_admin_user(
        &store,
        "admin@sdkwork.local",
        "ChangeMe123!",
        "admin-test-secret",
    )
    .await
    .unwrap_err();
    assert_eq!(old_password_error.to_string(), "invalid email or password");

    let new_session = login_admin_user(
        &store,
        "admin@sdkwork.local",
        "AdminPassword456!",
        "admin-test-secret",
    )
    .await
    .unwrap();
    assert_eq!(new_session.user.id, session.user.id);
}

#[tokio::test]
async fn admin_password_change_rejects_weak_passwords() {
    let store = memory_store().await;

    let session = login_admin_user(
        &store,
        "admin@sdkwork.local",
        "ChangeMe123!",
        "admin-test-secret",
    )
    .await
    .unwrap();

    let error = change_admin_password(
        &store,
        &session.user.id,
        "ChangeMe123!",
        "password1234",
    )
    .await
    .unwrap_err();

    assert_eq!(
        error.to_string(),
        "password must include an uppercase letter"
    );
}

#[tokio::test]
async fn production_bootstrap_workspace_does_not_lazy_create_default_admin_user() {
    let store = memory_store().await;
    store
        .insert_tenant(&Tenant::new("tenant_global_default", "Global Default Workspace"))
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

    let error = login_admin_user(
        &store,
        "admin@sdkwork.local",
        "ChangeMe123!",
        "admin-test-secret",
    )
    .await
    .unwrap_err();

    assert_eq!(error.to_string(), "invalid email or password");
    assert!(store.list_admin_users().await.unwrap().is_empty());
}
