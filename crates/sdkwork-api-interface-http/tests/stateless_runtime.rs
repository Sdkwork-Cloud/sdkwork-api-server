use sdkwork_api_interface_http::{StatelessGatewayConfig, StatelessGatewayUpstream};

#[test]
fn stateless_gateway_config_defaults_are_explicit() {
    let config = StatelessGatewayConfig::default();

    assert_eq!(config.tenant_id(), "sdkwork-stateless");
    assert_eq!(config.project_id(), "sdkwork-stateless-default");
    assert!(config.upstream().is_none());
}

#[test]
fn stateless_gateway_config_accepts_custom_identity_and_upstream() {
    let config = StatelessGatewayConfig::new()
        .with_identity("tenant-custom", "project-custom")
        .with_upstream(StatelessGatewayUpstream::from_adapter_kind(
            "openai",
            "https://example.com/v1",
            "sk-stateless",
        ));

    assert_eq!(config.tenant_id(), "tenant-custom");
    assert_eq!(config.project_id(), "project-custom");

    let upstream = config.upstream().expect("upstream should be configured");
    assert_eq!(upstream.runtime_key(), "openai");
    assert_eq!(upstream.base_url(), "https://example.com/v1");
    assert_eq!(upstream.api_key(), "sk-stateless");
}

#[test]
fn stateless_gateway_config_accepts_default_plugin_upstream_shortcut() {
    let config = StatelessGatewayConfig::new()
        .with_identity("tenant-openrouter", "project-openrouter")
        .try_with_default_plugin_upstream(
            "openrouter",
            "https://openrouter.ai/api/v1",
            "sk-openrouter",
        )
        .expect("default plugin upstream should be accepted");

    let upstream = config.upstream().expect("upstream should be configured");
    assert_eq!(upstream.runtime_key(), "openrouter");
    assert_eq!(upstream.protocol_kind(), "openai");
    assert_eq!(upstream.base_url(), "https://openrouter.ai/api/v1");
    assert_eq!(upstream.api_key(), "sk-openrouter");
}
