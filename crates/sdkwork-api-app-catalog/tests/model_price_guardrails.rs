use sdkwork_api_app_catalog::{
    list_model_prices, list_provider_models, persist_channel, persist_channel_model_with_metadata,
    persist_model_price_with_rates_and_metadata, persist_model_with_metadata,
    persist_provider_model_with_metadata, persist_provider_with_bindings_and_extension_id,
    PersistProviderWithBindingsRequest,
};
use sdkwork_api_domain_catalog::ModelCapability;
use sdkwork_api_storage_sqlite::{run_migrations, SqliteAdminStore};

#[tokio::test]
async fn model_prices_require_matching_provider_model_support() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    persist_channel(&store, "openai", "OpenAI").await.unwrap();
    persist_provider_with_bindings_and_extension_id(
        &store,
        PersistProviderWithBindingsRequest {
            id: "provider-openai-official",
            channel_id: "openai",
            adapter_kind: "openai",
            protocol_kind: Some("openai"),
            extension_id: None,
            base_url: "https://api.openai.com/v1",
            display_name: "OpenAI Official",
            channel_bindings: &[],
        },
    )
    .await
    .unwrap();
    persist_channel_model_with_metadata(
        &store,
        "openai",
        "gpt-4.1",
        "GPT-4.1",
        &[ModelCapability::Responses],
        true,
        Some(128_000),
        Some("Official flagship model."),
    )
    .await
    .unwrap();

    let error = persist_model_price_with_rates_and_metadata(
        &store,
        "openai",
        "gpt-4.1",
        "provider-openai-official",
        "USD",
        "per_1m_tokens",
        2.0,
        8.0,
        0.5,
        0.0,
        0.0,
        "official",
        Some("Official pricing snapshot."),
        Vec::new(),
        true,
    )
    .await
    .unwrap_err();

    assert!(
        error
            .to_string()
            .contains("provider-model must exist before pricing can be saved"),
        "unexpected error: {error}"
    );

    persist_provider_model_with_metadata(
        &store,
        "provider-openai-official",
        "openai",
        "gpt-4.1",
        Some("gpt-4.1"),
        None,
        Some(&[ModelCapability::Responses]),
        Some(true),
        Some(128_000),
        Some(32_768),
        true,
        false,
        true,
        true,
        true,
    )
    .await
    .unwrap();

    let price = persist_model_price_with_rates_and_metadata(
        &store,
        "openai",
        "gpt-4.1",
        "provider-openai-official",
        "USD",
        "per_1m_tokens",
        2.0,
        8.0,
        0.5,
        0.0,
        0.0,
        "official",
        Some("Official pricing snapshot."),
        Vec::new(),
        true,
    )
    .await
    .unwrap();

    assert_eq!(price.proxy_provider_id, "provider-openai-official");
    assert_eq!(price.channel_id, "openai");
    assert_eq!(price.model_id, "gpt-4.1");
}

#[tokio::test]
async fn persisting_provider_model_variants_creates_provider_model_and_price_records_together() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    persist_channel(&store, "anthropic", "Anthropic")
        .await
        .unwrap();
    persist_provider_with_bindings_and_extension_id(
        &store,
        PersistProviderWithBindingsRequest {
            id: "provider-anthropic-official",
            channel_id: "anthropic",
            adapter_kind: "anthropic",
            protocol_kind: Some("anthropic"),
            extension_id: None,
            base_url: "https://api.anthropic.com",
            display_name: "Anthropic Official",
            channel_bindings: &[],
        },
    )
    .await
    .unwrap();

    let model = persist_model_with_metadata(
        &store,
        "claude-3-7-sonnet",
        "provider-anthropic-official",
        &[ModelCapability::Responses, ModelCapability::ChatCompletions],
        true,
        Some(200_000),
    )
    .await
    .unwrap();

    assert_eq!(model.external_name, "claude-3-7-sonnet");

    let provider_models = list_provider_models(&store).await.unwrap();
    assert!(provider_models.iter().any(|record| {
        record.proxy_provider_id == "provider-anthropic-official"
            && record.channel_id == "anthropic"
            && record.model_id == "claude-3-7-sonnet"
            && record.provider_model_id == "claude-3-7-sonnet"
    }));

    let model_prices = list_model_prices(&store).await.unwrap();
    assert!(model_prices.iter().any(|record| {
        record.proxy_provider_id == "provider-anthropic-official"
            && record.channel_id == "anthropic"
            && record.model_id == "claude-3-7-sonnet"
    }));
}
