use sdkwork_api_app_credential::{
    persist_credential_with_secret, resolve_credential_secret, save_credential,
};
use sdkwork_api_storage_sqlite::{run_migrations, SqliteAdminStore};

#[test]
fn saves_upstream_credential_binding() {
    let credential =
        save_credential("tenant-1", "provider-openai-official", "cred-openai").unwrap();
    assert_eq!(credential.provider_id, "provider-openai-official");
}

#[tokio::test]
async fn persists_encrypted_credential_and_resolves_plaintext_secret() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    let credential = persist_credential_with_secret(
        &store,
        "local-dev-master-key",
        "tenant-1",
        "provider-openai-official",
        "cred-openai",
        "sk-upstream-openai",
    )
    .await
    .unwrap();

    assert_eq!(credential.key_reference, "cred-openai");

    let secret = resolve_credential_secret(
        &store,
        "local-dev-master-key",
        "tenant-1",
        "provider-openai-official",
        "cred-openai",
    )
    .await
    .unwrap();

    assert_eq!(secret, "sk-upstream-openai");
}
