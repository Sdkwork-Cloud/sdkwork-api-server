use sdkwork_api_app_gateway::list_models;
use sdkwork_api_app_gateway::list_models_from_store;
use sdkwork_api_storage_sqlite::{run_migrations, SqliteAdminStore};

#[test]
fn returns_platform_models() {
    let response = list_models("tenant-1", "project-1").unwrap();
    assert_eq!(response.object, "list");
}

#[tokio::test]
async fn returns_catalog_models_from_store() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);
    store
        .insert_model(&sdkwork_api_domain_catalog::ModelCatalogEntry::new(
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
