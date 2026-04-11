use std::str::FromStr;

use sdkwork_api_extension_core::{
    CapabilityDescriptor, CompatibilityLevel, ExtensionKind, ExtensionManifest, ExtensionModality,
    ExtensionProtocol, ExtensionRuntime,
};

#[test]
fn manifest_tracks_kind_runtime_and_capabilities() {
    let manifest = ExtensionManifest::new(
        "sdkwork.provider.openrouter",
        ExtensionKind::Provider,
        "0.1.0",
        ExtensionRuntime::Builtin,
    )
    .with_capability(CapabilityDescriptor::new(
        "responses.create",
        CompatibilityLevel::Relay,
    ))
    .with_channel_binding("sdkwork.channel.openai");

    assert_eq!(manifest.id, "sdkwork.provider.openrouter");
    assert_eq!(manifest.kind, ExtensionKind::Provider);
    assert_eq!(manifest.runtime, ExtensionRuntime::Builtin);
    assert_eq!(manifest.capabilities[0].operation, "responses.create");
    assert_eq!(
        manifest.capabilities[0].compatibility,
        CompatibilityLevel::Relay
    );
    assert_eq!(manifest.channel_bindings, vec!["sdkwork.channel.openai"]);
}

#[test]
fn manifest_tracks_runtime_contract_versions_and_supported_modalities() {
    let manifest = ExtensionManifest::new(
        "sdkwork.provider.openrouter",
        ExtensionKind::Provider,
        "0.1.0",
        ExtensionRuntime::Builtin,
    )
    .with_supported_modality(ExtensionModality::Image)
    .with_supported_modality(ExtensionModality::Audio);

    assert_eq!(
        manifest.runtime_compat_version.as_deref(),
        Some("sdkwork.runtime/v1")
    );
    assert_eq!(manifest.config_schema_version.as_deref(), Some("1.0"));
    assert_eq!(
        manifest.supported_modalities,
        vec![
            ExtensionModality::Text,
            ExtensionModality::Image,
            ExtensionModality::Audio,
        ]
    );
}

#[test]
fn manifest_normalizes_protocol_capabilities() {
    assert_eq!(
        ExtensionProtocol::from_str("openai")
            .unwrap()
            .capability_key(),
        "openai"
    );
    assert_eq!(
        ExtensionProtocol::from_str("anthropic")
            .unwrap()
            .capability_key(),
        "anthropic"
    );
    assert_eq!(
        ExtensionProtocol::from_str("gemini")
            .unwrap()
            .capability_key(),
        "gemini"
    );
    assert_eq!(
        ExtensionProtocol::from_str("custom")
            .unwrap()
            .capability_key(),
        "custom"
    );
    assert_eq!(
        ExtensionProtocol::from_str("openrouter")
            .unwrap()
            .capability_key(),
        "openai"
    );
    assert_eq!(
        ExtensionProtocol::from_str("ollama")
            .unwrap()
            .capability_key(),
        "custom"
    );
}

#[test]
fn manifest_reports_canonical_protocol_capability() {
    let manifest = ExtensionManifest::new(
        "sdkwork.provider.custom-openrouter",
        ExtensionKind::Provider,
        "0.1.0",
        ExtensionRuntime::Connector,
    )
    .with_protocol(ExtensionProtocol::from_str("openrouter").unwrap());

    assert_eq!(
        manifest
            .protocol_capability()
            .map(|protocol| protocol.capability_key()),
        Some("openai")
    );
}

#[test]
fn runtime_capability_helpers_keep_connector_off_raw_plugin_surface() {
    assert!(ExtensionRuntime::NativeDynamic.supports_raw_provider_execution());
    assert!(ExtensionRuntime::NativeDynamic.supports_structured_retry_hints());

    assert!(!ExtensionRuntime::Connector.supports_raw_provider_execution());
    assert!(!ExtensionRuntime::Connector.supports_structured_retry_hints());

    assert!(!ExtensionRuntime::Builtin.supports_raw_provider_execution());
    assert!(!ExtensionRuntime::Builtin.supports_structured_retry_hints());
}
