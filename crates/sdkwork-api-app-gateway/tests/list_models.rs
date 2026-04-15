use sdkwork_api_app_credential::CredentialSecretManager;
use sdkwork_api_app_gateway::{
    clear_capability_catalog_cache_store, configure_capability_catalog_cache_store, delete_model,
    delete_model_from_store, get_model, get_model_from_store, invalidate_capability_catalog_cache,
    list_models, list_models_from_store,
};
use sdkwork_api_cache_core::CacheStore;
use sdkwork_api_cache_memory::MemoryCacheStore;
use sdkwork_api_domain_catalog::{Channel, ModelCatalogEntry, ProxyProvider};
use sdkwork_api_storage_sqlite::{run_migrations, SqliteAdminStore};
use serial_test::serial;
use std::sync::Arc;

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

struct CapabilityCatalogCacheResetGuard;

impl Drop for CapabilityCatalogCacheResetGuard {
    fn drop(&mut self) {
        clear_capability_catalog_cache_store();
    }
}

fn capability_catalog_cache_reset_guard() -> CapabilityCatalogCacheResetGuard {
    clear_capability_catalog_cache_store();
    CapabilityCatalogCacheResetGuard
}

fn assert_error_contains<T: std::fmt::Debug, E: std::fmt::Display>(
    result: Result<T, E>,
    expected: &str,
) {
    let error = result.expect_err("expected error");
    assert!(
        error.to_string().contains(expected),
        "expected error containing `{expected}`, got `{error}`"
    );
}

#[test]
fn local_model_catalog_fallback_requires_configured_store() {
    assert_error_contains(
        list_models("tenant-1", "project-1"),
        "Local model catalog fallback is not supported",
    );
    assert_error_contains(
        get_model("tenant-1", "project-1", "gpt-4.1"),
        "Local model catalog fallback is not supported",
    );
    assert_error_contains(
        delete_model("tenant-1", "project-1", "ft:gpt-4.1:sdkwork"),
        "Local model catalog fallback is not supported",
    );
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

#[tokio::test]
#[serial]
async fn catalog_model_reads_use_configured_capability_catalog_cache_until_invalidated() {
    let _cache_guard = capability_catalog_cache_reset_guard();
    let cache_store: Arc<dyn CacheStore> = Arc::new(MemoryCacheStore::default());
    configure_capability_catalog_cache_store(cache_store);

    let store = create_store_with_registered_provider().await;
    store
        .insert_model(&ModelCatalogEntry::new(
            "gpt-4.1",
            "provider-openai-official",
        ))
        .await
        .unwrap();

    let first = list_models_from_store(&store, "tenant-1", "project-1")
        .await
        .unwrap();
    assert_eq!(first.data.len(), 1);
    assert_eq!(first.data[0].id, "gpt-4.1");

    store
        .insert_model(&ModelCatalogEntry::new(
            "gpt-4.1-mini",
            "provider-openai-official",
        ))
        .await
        .unwrap();

    let cached = list_models_from_store(&store, "tenant-1", "project-1")
        .await
        .unwrap();
    assert_eq!(cached.data.len(), 1);
    assert!(cached.data.iter().all(|model| model.id != "gpt-4.1-mini"));

    invalidate_capability_catalog_cache().await;

    let refreshed = list_models_from_store(&store, "tenant-1", "project-1")
        .await
        .unwrap();
    assert_eq!(refreshed.data.len(), 2);
    assert!(refreshed.data.iter().any(|model| model.id == "gpt-4.1"));
    assert!(refreshed
        .data
        .iter()
        .any(|model| model.id == "gpt-4.1-mini"));
}

#[tokio::test]
#[serial]
async fn deleting_catalog_model_from_store_invalidates_capability_catalog_cache() {
    let _cache_guard = capability_catalog_cache_reset_guard();
    let cache_store: Arc<dyn CacheStore> = Arc::new(MemoryCacheStore::default());
    configure_capability_catalog_cache_store(cache_store);

    let store = create_store_with_registered_provider().await;
    store
        .insert_model(&ModelCatalogEntry::new(
            "ft:gpt-4.1:sdkwork",
            "provider-openai-official",
        ))
        .await
        .unwrap();

    let cached = get_model_from_store(&store, "tenant-1", "project-1", "ft:gpt-4.1:sdkwork")
        .await
        .unwrap()
        .expect("catalog model");
    assert_eq!(cached.id, "ft:gpt-4.1:sdkwork");

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

    let reloaded = get_model_from_store(&store, "tenant-1", "project-1", "ft:gpt-4.1:sdkwork")
        .await
        .unwrap();
    assert!(reloaded.is_none());
}

#[tokio::test]
#[serial]
async fn capability_catalog_cache_store_can_be_cleared_for_runtime_isolation() {
    let _cache_guard = capability_catalog_cache_reset_guard();
    let cache_store: Arc<dyn CacheStore> = Arc::new(MemoryCacheStore::default());
    configure_capability_catalog_cache_store(cache_store);

    let store = create_store_with_registered_provider().await;
    store
        .insert_model(&ModelCatalogEntry::new(
            "gpt-4.1",
            "provider-openai-official",
        ))
        .await
        .unwrap();

    let cached = list_models_from_store(&store, "tenant-1", "project-1")
        .await
        .unwrap();
    assert_eq!(cached.data.len(), 1);

    clear_capability_catalog_cache_store();

    store
        .insert_model(&ModelCatalogEntry::new(
            "gpt-4.1-mini",
            "provider-openai-official",
        ))
        .await
        .unwrap();

    let uncached = list_models_from_store(&store, "tenant-1", "project-1")
        .await
        .unwrap();
    assert_eq!(uncached.data.len(), 2);
    assert!(uncached.data.iter().any(|model| model.id == "gpt-4.1"));
    assert!(uncached.data.iter().any(|model| model.id == "gpt-4.1-mini"));
}
