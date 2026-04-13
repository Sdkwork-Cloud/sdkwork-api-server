use sdkwork_api_app_identity::{
    change_admin_password, load_admin_user_profile, login_admin_user,
    login_admin_user_with_bootstrap, verify_jwt,
};
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
    assert_eq!(session.user.role.as_str(), "super_admin");
    assert!(session.user.active);
    assert!(session.token.len() > 10);

    let claims = verify_jwt(&session.token, "admin-test-secret").unwrap();
    assert_eq!(claims.sub, session.user.id);

    let user = load_admin_user_profile(&store, &session.user.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(user.email, "admin@sdkwork.local");
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
async fn disabled_admin_bootstrap_does_not_seed_default_credentials() {
    let store = memory_store().await;

    let error = login_admin_user_with_bootstrap(
        &store,
        "admin@sdkwork.local",
        "ChangeMe123!",
        "admin-test-secret",
        false,
    )
    .await
    .unwrap_err();
    assert_eq!(error.to_string(), "invalid email or password");
    assert!(store
        .find_admin_user_by_email("admin@sdkwork.local")
        .await
        .unwrap()
        .is_none());
}
