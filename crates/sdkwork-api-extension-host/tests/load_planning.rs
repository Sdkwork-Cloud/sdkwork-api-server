use sdkwork_api_extension_core::{
    ExtensionInstallation, ExtensionInstance, ExtensionKind, ExtensionManifest, ExtensionRuntime,
};
use sdkwork_api_extension_host::{BuiltinExtensionFactory, ExtensionHost};

#[test]
fn host_builds_runtime_load_plan_from_manifest_installation_and_instance() {
    let mut host = ExtensionHost::new();
    host.register_builtin(BuiltinExtensionFactory::new(
        ExtensionManifest::new(
            "sdkwork.provider.openrouter",
            ExtensionKind::Provider,
            "0.1.0",
            ExtensionRuntime::Connector,
        )
        .with_entrypoint("bin/default-openrouter")
        .with_config_schema("schemas/config.schema.json"),
    ));

    host.install(
        ExtensionInstallation::new(
            "openrouter-installation",
            "sdkwork.provider.openrouter",
            ExtensionRuntime::Connector,
        )
        .with_entrypoint("bin/sdkwork-provider-openrouter")
        .with_config(serde_json::json!({"timeout_secs": 30, "region": "global"})),
    )
    .unwrap();

    host.mount_instance(
        ExtensionInstance::new(
            "provider-openrouter-main",
            "openrouter-installation",
            "sdkwork.provider.openrouter",
        )
        .with_base_url("https://openrouter.ai/api/v1")
        .with_credential_ref("cred-openrouter")
        .with_config(serde_json::json!({"region": "us", "weight": 100})),
    )
    .unwrap();

    let plan = host
        .load_plan("provider-openrouter-main")
        .expect("plan should build");

    assert_eq!(plan.extension_id, "sdkwork.provider.openrouter");
    assert_eq!(plan.runtime, ExtensionRuntime::Connector);
    assert_eq!(
        plan.entrypoint.as_deref(),
        Some("bin/sdkwork-provider-openrouter")
    );
    assert_eq!(
        plan.base_url.as_deref(),
        Some("https://openrouter.ai/api/v1")
    );
    assert_eq!(plan.credential_ref.as_deref(), Some("cred-openrouter"));
    assert_eq!(plan.config["timeout_secs"], 30);
    assert_eq!(plan.config["region"], "us");
    assert_eq!(plan.config["weight"], 100);
}
