use sdkwork_api_extension_core::{
    ExtensionHealthContract, ExtensionKind, ExtensionManifest, ExtensionPermission,
    ExtensionRuntime, ExtensionSignature, ExtensionSignatureAlgorithm, ExtensionTrustDeclaration,
};

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
    .with_credential_schema("schemas/credential.schema.json")
    .with_permission(ExtensionPermission::NetworkOutbound)
    .with_permission(ExtensionPermission::SpawnProcess)
    .with_health_contract(ExtensionHealthContract::new("/health", 30));

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
    assert_eq!(manifest.permissions.len(), 2);
    assert_eq!(
        manifest.permissions[0],
        ExtensionPermission::NetworkOutbound
    );
    assert_eq!(manifest.permissions[1], ExtensionPermission::SpawnProcess);
    assert_eq!(
        manifest.health.as_ref(),
        Some(&ExtensionHealthContract::new("/health", 30))
    );
}

#[test]
fn manifest_carries_extension_trust_metadata() {
    let manifest = ExtensionManifest::new(
        "sdkwork.provider.openrouter",
        ExtensionKind::Provider,
        "0.1.0",
        ExtensionRuntime::Connector,
    )
    .with_trust(ExtensionTrustDeclaration::signed(
        "sdkwork",
        ExtensionSignature::new(
            ExtensionSignatureAlgorithm::Ed25519,
            "cHVibGljLWtleS1iYXNlNjQ=",
            "c2lnbmF0dXJlLWJhc2U2NA==",
        ),
    ));

    let trust = manifest.trust.as_ref().expect("trust metadata");
    assert_eq!(trust.publisher, "sdkwork");
    assert_eq!(
        trust.signature.algorithm,
        ExtensionSignatureAlgorithm::Ed25519
    );
    assert_eq!(trust.signature.public_key, "cHVibGljLWtleS1iYXNlNjQ=");
    assert_eq!(trust.signature.signature, "c2lnbmF0dXJlLWJhc2U2NA==");
}
