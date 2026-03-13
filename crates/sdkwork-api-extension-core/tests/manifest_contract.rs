use sdkwork_api_extension_core::{
    CapabilityDescriptor, CompatibilityLevel, ExtensionKind, ExtensionManifest, ExtensionRuntime,
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
