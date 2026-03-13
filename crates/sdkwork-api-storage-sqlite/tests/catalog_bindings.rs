use sdkwork_api_domain_catalog::{
    ModelCapability, ModelCatalogEntry, ProviderChannelBinding, ProxyProvider,
};
use sdkwork_api_storage_sqlite::{run_migrations, SqliteAdminStore};

#[tokio::test]
async fn sqlite_store_persists_provider_bindings_and_model_metadata() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    let provider = ProxyProvider::new(
        "provider-openrouter-main",
        "openrouter",
        "openrouter",
        "https://openrouter.ai/api/v1",
        "OpenRouter Main",
    )
    .with_channel_binding(ProviderChannelBinding::new(
        "provider-openrouter-main",
        "openai",
    ));

    store.insert_provider(&provider).await.unwrap();

    let model = ModelCatalogEntry::new("gpt-4.1", "provider-openrouter-main")
        .with_capability(ModelCapability::Responses)
        .with_capability(ModelCapability::ChatCompletions)
        .with_streaming(true)
        .with_context_window(128_000);

    store.insert_model(&model).await.unwrap();

    let providers = store.list_providers().await.unwrap();
    assert_eq!(providers[0].channel_bindings.len(), 2);
    assert_eq!(providers[0].channel_bindings[1].channel_id, "openai");

    let models = store.list_models().await.unwrap();
    assert_eq!(models[0].capabilities.len(), 2);
    assert!(models[0].streaming);
    assert_eq!(models[0].context_window, Some(128_000));
}
