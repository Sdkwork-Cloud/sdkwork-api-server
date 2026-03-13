use sdkwork_api_extension_core::{ExtensionKind, ExtensionManifest, ExtensionRuntime};

#[test]
fn manifest_derives_distribution_and_crate_names_from_runtime_id() {
    let manifest = ExtensionManifest::new(
        "sdkwork.provider.openrouter",
        ExtensionKind::Provider,
        "0.1.0",
        ExtensionRuntime::Connector,
    )
    .with_display_name("OpenRouter")
    .with_entrypoint("bin/sdkwork-provider-openrouter")
    .with_config_schema("schemas/config.schema.json")
    .with_credential_schema("schemas/credential.schema.json");

    assert_eq!(manifest.distribution_name(), "sdkwork-provider-openrouter");
    assert_eq!(manifest.crate_name(), "sdkwork-api-ext-provider-openrouter");
    assert_eq!(manifest.display_name, "OpenRouter");
    assert_eq!(
        manifest.entrypoint.as_deref(),
        Some("bin/sdkwork-provider-openrouter")
    );
    assert_eq!(
        manifest.config_schema.as_deref(),
        Some("schemas/config.schema.json")
    );
    assert_eq!(
        manifest.credential_schema.as_deref(),
        Some("schemas/credential.schema.json")
    );
}
