use sdkwork_api_app_credential::CredentialSecretManager;
use sdkwork_api_app_gateway::{
    delete_model, delete_model_from_store, get_model, get_model_from_store, list_models,
    list_models_from_store,
};
use sdkwork_api_domain_catalog::{Channel, ModelCatalogEntry, ProxyProvider};
use sdkwork_api_storage_sqlite::{run_migrations, SqliteAdminStore};

async fn create_store_with_registered_provider() -> SqliteAdminStore {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);
    store
        .insert_channel(&Channel::new("openai", "OpenAI"))
        .await
        .unwrap();
    store
        .insert_provider(
            &ProxyProvider::new(
                "provider-openai-official",
                "openai",
                "openai",
                "http://127.0.0.1:1",
                "OpenAI Official",
            )
            .with_extension_id("sdkwork.provider.openai.official"),
        )
        .await
        .unwrap();
    store
}

#[test]
fn returns_platform_models() {
    let response = list_models("tenant-1", "project-1").unwrap();
    assert_eq!(response.object, "list");
}

#[test]
fn returns_platform_model() {
    let response = get_model("tenant-1", "project-1", "gpt-4.1").unwrap();
    assert_eq!(response.id, "gpt-4.1");
    assert_eq!(response.object, "model");
}

#[test]
fn deletes_platform_model() {
    let response = delete_model("tenant-1", "project-1", "ft:gpt-4.1:sdkwork").unwrap();
    assert_eq!(response.id, "ft:gpt-4.1:sdkwork");
    assert!(response.deleted);
}

#[tokio::test]
async fn returns_catalog_models_from_store() {
    let store = create_store_with_registered_provider().await;
    store
        .insert_model(&ModelCatalogEntry::new(
            "gpt-4.1",
            "provider-openai-official",
        ))
        .await
        .unwrap();

    let response = list_models_from_store(&store, "tenant-1", "project-1")
        .await
        .unwrap();
    assert_eq!(response.data[0].id, "gpt-4.1");
}

#[tokio::test]
async fn returns_catalog_model_from_store() {
    let store = create_store_with_registered_provider().await;
    store
        .insert_model(&ModelCatalogEntry::new(
            "gpt-4.1",
            "provider-openai-official",
        ))
        .await
        .unwrap();

    let response = get_model_from_store(&store, "tenant-1", "project-1", "gpt-4.1")
        .await
        .unwrap()
        .expect("catalog model");
    assert_eq!(response.id, "gpt-4.1");
    assert_eq!(response.owned_by, "provider-openai-official");
}

#[tokio::test]
async fn deletes_catalog_model_from_store() {
    let store = create_store_with_registered_provider().await;
    store
        .insert_model(&ModelCatalogEntry::new(
            "ft:gpt-4.1:sdkwork",
            "provider-openai-official",
        ))
        .await
        .unwrap();

    let response = delete_model_from_store(
        &store,
        &CredentialSecretManager::database_encrypted("local-dev-master-key"),
        "tenant-1",
        "project-1",
        "ft:gpt-4.1:sdkwork",
    )
    .await
    .unwrap()
    .expect("deleted model");

    assert_eq!(response["id"], "ft:gpt-4.1:sdkwork");
    assert_eq!(response["deleted"], true);
    assert!(store
        .find_model("ft:gpt-4.1:sdkwork")
        .await
        .unwrap()
        .is_none());
}
