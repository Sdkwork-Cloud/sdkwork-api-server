use sdkwork_api_domain_catalog::{
    derive_provider_default_plugin_family, normalize_provider_default_plugin_family,
    ModelCapability, ModelVariant, ProviderChannelBinding, ProxyProvider,
};

#[test]
fn provider_can_bind_to_multiple_channels() {
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

    assert_eq!(provider.channel_id, "openrouter");
    assert_eq!(provider.channel_bindings.len(), 2);
    assert_eq!(
        provider.channel_bindings[0].provider_id,
        "provider-openrouter-main"
    );
    assert_eq!(provider.channel_bindings[0].channel_id, "openrouter");
    assert!(provider.channel_bindings[0].is_primary);
    assert_eq!(provider.channel_bindings[1].channel_id, "openai");
    assert!(!provider.channel_bindings[1].is_primary);
}

#[test]
fn provider_tracks_extension_runtime_identity() {
    let derived = ProxyProvider::new(
        "provider-openrouter-main",
        "openrouter",
        "openrouter",
        "https://openrouter.ai/api/v1",
        "OpenRouter Main",
    );
    assert_eq!(derived.extension_id, "sdkwork.provider.openrouter");

    let explicit = ProxyProvider::new(
        "provider-openrouter-main",
        "openrouter",
        "openrouter-compatible",
        "https://openrouter.ai/api/v1",
        "OpenRouter Main",
    )
    .with_extension_id("sdkwork.provider.openrouter");
    assert_eq!(explicit.extension_id, "sdkwork.provider.openrouter");
}

#[test]
fn provider_derives_protocol_kind_from_adapter_kind() {
    let openrouter = ProxyProvider::new(
        "provider-openrouter-main",
        "openrouter",
        "openrouter",
        "https://openrouter.ai/api/v1",
        "OpenRouter Main",
    );
    assert_eq!(openrouter.protocol_kind(), "openai");

    let ollama = ProxyProvider::new(
        "provider-ollama-local",
        "ollama",
        "ollama",
        "http://localhost:11434/v1",
        "Ollama Local",
    );
    assert_eq!(ollama.protocol_kind(), "custom");

    let native_dynamic = ProxyProvider::new(
        "provider-native-bridge",
        "claude",
        "native-dynamic",
        "https://relay.example.com",
        "Claude Relay",
    );
    assert_eq!(native_dynamic.protocol_kind(), "custom");
}

#[test]
fn provider_normalizes_default_plugin_families_for_builtin_nonstandard_families() {
    assert_eq!(
        derive_provider_default_plugin_family("openrouter-compatible"),
        Some("openrouter")
    );
    assert_eq!(derive_provider_default_plugin_family("ollama"), Some("ollama"));
    assert_eq!(
        derive_provider_default_plugin_family("siliconflow"),
        Some("siliconflow")
    );
    assert_eq!(
        normalize_provider_default_plugin_family(" OpenRouter "),
        Some("openrouter")
    );
    assert_eq!(
        normalize_provider_default_plugin_family("ollama-compatible"),
        Some("ollama")
    );
    assert_eq!(
        normalize_provider_default_plugin_family("SiliconFlow"),
        Some("siliconflow")
    );
    assert_eq!(normalize_provider_default_plugin_family("openai"), None);
}

#[test]
fn provider_can_override_protocol_kind_without_changing_runtime_identity() {
    let provider = ProxyProvider::new(
        "provider-claude-relay",
        "claude",
        "native-dynamic",
        "https://relay.example.com",
        "Claude Relay",
    )
    .with_extension_id("sdkwork.provider.claude.relay")
    .with_protocol_kind("anthropic");

    assert_eq!(provider.extension_id, "sdkwork.provider.claude.relay");
    assert_eq!(provider.adapter_kind, "native-dynamic");
    assert_eq!(provider.protocol_kind(), "anthropic");
}

#[test]
fn model_variant_tracks_capabilities_and_streaming() {
    let model = ModelVariant::new("gpt-4.1", "provider-openai-official")
        .with_capability(ModelCapability::Responses)
        .with_capability(ModelCapability::ChatCompletions)
        .with_streaming(true)
        .with_context_window(128_000);

    assert!(model.streaming);
    assert_eq!(model.capabilities.len(), 2);
    assert_eq!(model.context_window, Some(128_000));
}
