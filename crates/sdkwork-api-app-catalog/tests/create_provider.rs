use sdkwork_api_app_catalog::{
    create_provider_with_bindings_and_extension_id,
    create_provider_with_config, create_provider_with_default_plugin_family,
    create_provider_with_default_plugin_family_and_bindings,
    create_provider_with_protocol_kind, normalize_provider_integration,
    provider_integration_view, ProviderIntegrationMode,
};
use sdkwork_api_domain_catalog::ProviderChannelBinding;

#[test]
fn creates_provider_for_channel() {
    let provider = create_provider_with_config(
        "provider-openai-official",
        "openai",
        "openai",
        "https://api.openai.com",
        "OpenAI Official",
    )
    .unwrap();
    assert_eq!(provider.channel_id, "openai");
    assert_eq!(provider.adapter_kind, "openai");
    assert_eq!(provider.base_url, "https://api.openai.com");
    assert_eq!(provider.protocol_kind(), "openai");
}

#[test]
fn creates_provider_with_explicit_protocol_kind() {
    let provider = create_provider_with_protocol_kind(
        "provider-claude-relay",
        "claude",
        "native-dynamic",
        "anthropic",
        "https://relay.example.com",
        "Claude Relay",
    )
    .unwrap();

    assert_eq!(provider.adapter_kind, "native-dynamic");
    assert_eq!(provider.protocol_kind(), "anthropic");
    assert_eq!(provider.extension_id, "sdkwork.provider.native-dynamic");
}

#[test]
fn creates_standard_passthrough_providers_with_derived_identity() {
    let anthropic = create_provider_with_config(
        "provider-anthropic-official",
        "anthropic",
        "anthropic",
        "https://api.anthropic.com",
        "Anthropic Official",
    )
    .unwrap();
    assert_eq!(anthropic.adapter_kind, "anthropic");
    assert_eq!(anthropic.protocol_kind(), "anthropic");
    assert_eq!(anthropic.extension_id, "sdkwork.provider.anthropic");

    let gemini = create_provider_with_config(
        "provider-gemini-official",
        "gemini",
        "gemini",
        "https://generativelanguage.googleapis.com",
        "Gemini Official",
    )
    .unwrap();
    assert_eq!(gemini.adapter_kind, "gemini");
    assert_eq!(gemini.protocol_kind(), "gemini");
    assert_eq!(gemini.extension_id, "sdkwork.provider.gemini");
}

#[test]
fn creates_default_plugin_provider_with_derived_identity() {
    let provider = create_provider_with_default_plugin_family(
        "provider-openrouter-main",
        "openrouter",
        "openrouter",
        "https://openrouter.ai/api/v1",
        "OpenRouter Main",
    )
    .unwrap();

    assert_eq!(provider.adapter_kind, "openrouter");
    assert_eq!(provider.protocol_kind(), "openai");
    assert_eq!(provider.extension_id, "sdkwork.provider.openrouter");
}

#[test]
fn creates_default_plugin_provider_with_additional_channel_bindings() {
    let provider = create_provider_with_default_plugin_family_and_bindings(
        "provider-openrouter-main",
        "openrouter",
        "openrouter",
        "https://openrouter.ai/api/v1",
        "OpenRouter Main",
        &[ProviderChannelBinding::new(
            "provider-openrouter-main",
            "openai",
        )],
    )
    .unwrap();

    assert_eq!(provider.channel_id, "openrouter");
    assert_eq!(provider.adapter_kind, "openrouter");
    assert_eq!(provider.protocol_kind(), "openai");
    assert_eq!(provider.extension_id, "sdkwork.provider.openrouter");
    assert!(provider.channel_bindings.contains(&ProviderChannelBinding::primary(
        "provider-openrouter-main",
        "openrouter",
    )));
    assert!(provider.channel_bindings.contains(&ProviderChannelBinding::new(
        "provider-openrouter-main",
        "openai",
    )));
}

#[test]
fn normalizes_default_plugin_family_to_canonical_provider_identity() {
    let openrouter = normalize_provider_integration(
        None,
        None,
        None,
        Some("openrouter-compatible"),
    )
    .unwrap();
    assert_eq!(openrouter.adapter_kind, "openrouter");
    assert_eq!(openrouter.protocol_kind.as_deref(), Some("openai"));
    assert_eq!(
        openrouter.extension_id.as_deref(),
        Some("sdkwork.provider.openrouter")
    );

    let ollama =
        normalize_provider_integration(None, None, None, Some("ollama")).unwrap();
    assert_eq!(ollama.adapter_kind, "ollama");
    assert_eq!(ollama.protocol_kind.as_deref(), Some("custom"));
    assert_eq!(
        ollama.extension_id.as_deref(),
        Some("sdkwork.provider.ollama")
    );

    let siliconflow = normalize_provider_integration(
        None,
        None,
        None,
        Some("siliconflow"),
    )
    .unwrap();
    assert_eq!(siliconflow.adapter_kind, "siliconflow");
    assert_eq!(siliconflow.protocol_kind.as_deref(), Some("openai"));
    assert_eq!(
        siliconflow.extension_id.as_deref(),
        Some("sdkwork.provider.siliconflow")
    );
}

#[test]
fn rejects_conflicting_default_plugin_family_identity() {
    let error = normalize_provider_integration(
        Some("openai"),
        None,
        None,
        Some("openrouter"),
    )
    .unwrap_err();

    assert!(error
        .to_string()
        .contains("default_plugin_family must match adapter_kind"));
}

#[test]
fn derives_provider_integration_view_from_provider_identity() {
    let official = create_provider_with_config(
        "provider-openai-official",
        "openai",
        "openai",
        "https://api.openai.com",
        "OpenAI Official",
    )
    .unwrap();
    assert_eq!(
        provider_integration_view(&official).mode,
        ProviderIntegrationMode::StandardPassthrough
    );
    assert_eq!(
        provider_integration_view(&official).default_plugin_family,
        None
    );

    let openrouter = create_provider_with_default_plugin_family(
        "provider-openrouter-main",
        "openrouter",
        "openrouter",
        "https://openrouter.ai/api/v1",
        "OpenRouter Main",
    )
    .unwrap();
    assert_eq!(
        provider_integration_view(&openrouter).mode,
        ProviderIntegrationMode::DefaultPlugin
    );
    assert_eq!(
        provider_integration_view(&openrouter).default_plugin_family.as_deref(),
        Some("openrouter")
    );

    let siliconflow = create_provider_with_default_plugin_family(
        "provider-siliconflow-main",
        "siliconflow",
        "siliconflow",
        "https://api.siliconflow.cn/v1",
        "SiliconFlow Main",
    )
    .unwrap();
    assert_eq!(
        provider_integration_view(&siliconflow).mode,
        ProviderIntegrationMode::DefaultPlugin
    );
    assert_eq!(
        provider_integration_view(&siliconflow)
            .default_plugin_family
            .as_deref(),
        Some("siliconflow")
    );

    let claude_relay = create_provider_with_bindings_and_extension_id(
        "provider-claude-relay",
        "claude",
        "native-dynamic",
        Some("anthropic"),
        Some("sdkwork.provider.claude.relay"),
        "https://relay.example.com",
        "Claude Relay",
        &[],
    )
    .unwrap();
    assert_eq!(
        provider_integration_view(&claude_relay).mode,
        ProviderIntegrationMode::CustomPlugin
    );
    assert_eq!(
        provider_integration_view(&claude_relay).default_plugin_family,
        None
    );
}
