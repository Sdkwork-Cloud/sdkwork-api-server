use sdkwork_api_extension_core::{
    ExtensionInstallation, ExtensionInstance, ExtensionKind, ExtensionManifest, ExtensionRuntime,
};
use sdkwork_api_extension_host::{BuiltinExtensionFactory, ExtensionHost};
use serde_json::json;

#[test]
fn installation_can_mount_multiple_instances_from_one_extension() {
    let mut host = ExtensionHost::new();
    host.register_builtin(BuiltinExtensionFactory::new(ExtensionManifest::new(
        "sdkwork.provider.openrouter",
        ExtensionKind::Provider,
        "0.1.0",
        ExtensionRuntime::Builtin,
    )));

    host.install(
        ExtensionInstallation::new(
            "openrouter-builtin",
            "sdkwork.provider.openrouter",
            ExtensionRuntime::Builtin,
        )
        .with_config(json!({"trust_mode":"builtin"})),
    )
    .unwrap();

    host.mount_instance(
        ExtensionInstance::new(
            "provider-openrouter-main",
            "openrouter-builtin",
            "sdkwork.provider.openrouter",
        )
        .with_base_url("https://openrouter.ai/api/v1")
        .with_config(json!({"region":"global"})),
    )
    .unwrap();

    host.mount_instance(
        ExtensionInstance::new(
            "provider-openrouter-backup",
            "openrouter-builtin",
            "sdkwork.provider.openrouter",
        )
        .with_base_url("https://openrouter.ai/api/v1")
        .with_config(json!({"region":"backup"})),
    )
    .unwrap();

    assert_eq!(host.instances("sdkwork.provider.openrouter").len(), 2);
}
