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
