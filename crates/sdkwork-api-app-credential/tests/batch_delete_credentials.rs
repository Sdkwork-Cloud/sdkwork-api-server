use sdkwork_api_app_credential::{
    delete_provider_credentials_with_manager, delete_tenant_credentials_with_manager,
    list_credentials, persist_credential_with_secret_and_manager, CredentialSecretManager,
};
use sdkwork_api_storage_sqlite::{run_migrations, SqliteAdminStore};

#[tokio::test]
async fn delete_provider_credentials_removes_only_matching_provider_records() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);
    let manager = CredentialSecretManager::database_encrypted("batch-delete-master-key");

    persist_credential_with_secret_and_manager(
        &store,
        &manager,
        "tenant-1",
        "provider-a",
        "cred-a-1",
        "secret-a-1",
    )
    .await
    .unwrap();
    persist_credential_with_secret_and_manager(
        &store,
        &manager,
        "tenant-2",
        "provider-a",
        "cred-a-2",
        "secret-a-2",
    )
    .await
    .unwrap();
    persist_credential_with_secret_and_manager(
        &store,
        &manager,
        "tenant-1",
        "provider-b",
        "cred-b-1",
        "secret-b-1",
    )
    .await
    .unwrap();

    delete_provider_credentials_with_manager(&store, &manager, "provider-a")
        .await
        .unwrap();

    let remaining = list_credentials(&store).await.unwrap();
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0].provider_id, "provider-b");
}

#[tokio::test]
async fn delete_tenant_credentials_removes_only_matching_tenant_records() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);
    let manager = CredentialSecretManager::database_encrypted("batch-delete-master-key");

    persist_credential_with_secret_and_manager(
        &store,
        &manager,
        "tenant-1",
        "provider-a",
        "cred-a-1",
        "secret-a-1",
    )
    .await
    .unwrap();
    persist_credential_with_secret_and_manager(
        &store,
        &manager,
        "tenant-1",
        "provider-b",
        "cred-b-1",
        "secret-b-1",
    )
    .await
    .unwrap();
    persist_credential_with_secret_and_manager(
        &store,
        &manager,
        "tenant-2",
        "provider-a",
        "cred-a-2",
        "secret-a-2",
    )
    .await
    .unwrap();

    delete_tenant_credentials_with_manager(&store, &manager, "tenant-1")
        .await
        .unwrap();

    let remaining = list_credentials(&store).await.unwrap();
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0].tenant_id, "tenant-2");
}
