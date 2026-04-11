#[test]
fn legacy_adapter_constructor_derives_protocol_kind() {
    let upstream = sdkwork_api_interface_http::StatelessGatewayUpstream::from_adapter_kind(
        "custom-openai",
        "https://api.openai.com/v1",
        "sk-test",
    );

    assert_eq!(upstream.runtime_key(), "custom-openai");
    assert_eq!(upstream.protocol_kind(), "openai");
}

#[test]
fn openrouter_legacy_adapter_constructor_derives_openai_protocol_kind() {
    let upstream = sdkwork_api_interface_http::StatelessGatewayUpstream::from_adapter_kind(
        "openrouter",
        "https://openrouter.ai/api/v1",
        "sk-or-v1-test",
    );

    assert_eq!(upstream.runtime_key(), "openrouter");
    assert_eq!(upstream.protocol_kind(), "openai");
}

#[test]
fn ollama_legacy_adapter_constructor_derives_custom_protocol_kind() {
    let upstream = sdkwork_api_interface_http::StatelessGatewayUpstream::from_adapter_kind(
        "ollama",
        "http://localhost:11434/v1",
        "ollama-local-token",
    );

    assert_eq!(upstream.runtime_key(), "ollama");
    assert_eq!(upstream.protocol_kind(), "custom");
}

#[test]
fn explicit_protocol_constructor_preserves_runtime_runtime_key() {
    let upstream = sdkwork_api_interface_http::StatelessGatewayUpstream::new_with_protocol_kind(
        "native-dynamic",
        "anthropic",
        "https://relay.example.com",
        "sk-test",
    );

    assert_eq!(upstream.runtime_key(), "native-dynamic");
    assert_eq!(upstream.protocol_kind(), "anthropic");
    assert_eq!(upstream.base_url(), "https://relay.example.com");
}

#[test]
fn default_plugin_family_constructor_derives_openrouter_identity() {
    let upstream =
        sdkwork_api_interface_http::StatelessGatewayUpstream::from_default_plugin_family(
            "openrouter",
            "https://openrouter.ai/api/v1",
            "sk-or-v1-test",
        )
        .expect("openrouter default plugin upstream");

    assert_eq!(upstream.runtime_key(), "openrouter");
    assert_eq!(upstream.protocol_kind(), "openai");
    assert_eq!(upstream.base_url(), "https://openrouter.ai/api/v1");
}

#[test]
fn default_plugin_family_constructor_derives_ollama_identity() {
    let upstream =
        sdkwork_api_interface_http::StatelessGatewayUpstream::from_default_plugin_family(
            "ollama",
            "http://localhost:11434/v1",
            "ollama-local-token",
        )
        .expect("ollama default plugin upstream");

    assert_eq!(upstream.runtime_key(), "ollama");
    assert_eq!(upstream.protocol_kind(), "custom");
    assert_eq!(upstream.base_url(), "http://localhost:11434/v1");
}

#[test]
fn default_plugin_family_constructor_rejects_unsupported_family() {
    let error = sdkwork_api_interface_http::StatelessGatewayUpstream::from_default_plugin_family(
        "openai",
        "https://api.openai.com/v1",
        "sk-test",
    )
    .expect_err("unsupported family should fail");

    assert!(error
        .to_string()
        .contains("unsupported default_plugin_family"));
}
