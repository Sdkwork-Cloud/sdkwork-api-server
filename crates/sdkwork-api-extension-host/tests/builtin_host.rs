use sdkwork_api_extension_core::{ExtensionKind, ExtensionManifest, ExtensionRuntime};
use sdkwork_api_extension_host::{BuiltinExtensionFactory, ExtensionHost};

#[test]
fn host_registers_and_resolves_builtin_provider_extensions() {
    let mut host = ExtensionHost::new();
    host.register_builtin(BuiltinExtensionFactory::new(ExtensionManifest::new(
        "sdkwork.provider.openai.official",
        ExtensionKind::Provider,
        "0.1.0",
        ExtensionRuntime::Builtin,
    )));

    let manifest = host
        .manifest("sdkwork.provider.openai.official")
        .expect("manifest");

    assert_eq!(manifest.id, "sdkwork.provider.openai.official");
}
