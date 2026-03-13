use anyhow::Result;
use sdkwork_api_extension_core::{ExtensionInstallation, ExtensionInstance, ExtensionRuntime};
use sdkwork_api_storage_core::AdminStore;
use serde_json::Value;

pub struct PersistExtensionInstanceInput<'a> {
    pub instance_id: &'a str,
    pub installation_id: &'a str,
    pub extension_id: &'a str,
    pub enabled: bool,
    pub base_url: Option<&'a str>,
    pub credential_ref: Option<&'a str>,
    pub config: Value,
}

pub async fn list_extension_installations(
    store: &dyn AdminStore,
) -> Result<Vec<ExtensionInstallation>> {
    store.list_extension_installations().await
}

pub async fn persist_extension_installation(
    store: &dyn AdminStore,
    installation_id: &str,
    extension_id: &str,
    runtime: ExtensionRuntime,
    enabled: bool,
    entrypoint: Option<&str>,
    config: Value,
) -> Result<ExtensionInstallation> {
    let mut installation =
        ExtensionInstallation::new(installation_id, extension_id, runtime).with_enabled(enabled);
    if let Some(entrypoint) = entrypoint {
        installation = installation.with_entrypoint(entrypoint);
    }
    installation = installation.with_config(config);
    store.insert_extension_installation(&installation).await
}

pub async fn list_extension_instances(store: &dyn AdminStore) -> Result<Vec<ExtensionInstance>> {
    store.list_extension_instances().await
}

pub async fn persist_extension_instance(
    store: &dyn AdminStore,
    input: PersistExtensionInstanceInput<'_>,
) -> Result<ExtensionInstance> {
    let mut instance =
        ExtensionInstance::new(input.instance_id, input.installation_id, input.extension_id)
            .with_enabled(input.enabled);
    if let Some(base_url) = input.base_url {
        instance = instance.with_base_url(base_url);
    }
    if let Some(credential_ref) = input.credential_ref {
        instance = instance.with_credential_ref(credential_ref);
    }
    instance = instance.with_config(input.config);
    store.insert_extension_instance(&instance).await
}
