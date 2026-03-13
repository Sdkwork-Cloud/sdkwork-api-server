use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use sdkwork_api_extension_core::{ExtensionInstallation, ExtensionInstance, ExtensionManifest};
use sdkwork_api_provider_core::ProviderExecutionAdapter;

#[derive(Debug, Clone)]
pub struct BuiltinExtensionFactory {
    manifest: ExtensionManifest,
}

impl BuiltinExtensionFactory {
    pub fn new(manifest: ExtensionManifest) -> Self {
        Self { manifest }
    }
}

type ProviderFactory =
    Arc<dyn Fn(String) -> Box<dyn ProviderExecutionAdapter> + Send + Sync + 'static>;

#[derive(Clone)]
pub struct BuiltinProviderExtensionFactory {
    manifest: ExtensionManifest,
    adapter_kind: String,
    factory: ProviderFactory,
}

impl BuiltinProviderExtensionFactory {
    pub fn new<F>(manifest: ExtensionManifest, adapter_kind: impl Into<String>, factory: F) -> Self
    where
        F: Fn(String) -> Box<dyn ProviderExecutionAdapter> + Send + Sync + 'static,
    {
        Self {
            manifest,
            adapter_kind: adapter_kind.into(),
            factory: Arc::new(factory),
        }
    }
}

#[derive(Default, Clone)]
pub struct ExtensionHost {
    manifests: HashMap<String, ExtensionManifest>,
    provider_factories: HashMap<String, ProviderFactory>,
    installations: HashMap<String, ExtensionInstallation>,
    instances_by_extension: HashMap<String, Vec<ExtensionInstance>>,
}

impl ExtensionHost {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_builtin(&mut self, factory: BuiltinExtensionFactory) {
        self.manifests
            .insert(factory.manifest.id.clone(), factory.manifest);
    }

    pub fn register_builtin_provider(&mut self, factory: BuiltinProviderExtensionFactory) {
        self.manifests
            .insert(factory.manifest.id.clone(), factory.manifest);
        self.provider_factories
            .insert(factory.adapter_kind, factory.factory);
    }

    pub fn manifest(&self, id: &str) -> Option<&ExtensionManifest> {
        self.manifests.get(id)
    }

    pub fn install(
        &mut self,
        installation: ExtensionInstallation,
    ) -> Result<(), ExtensionHostError> {
        if !self.manifests.contains_key(&installation.extension_id) {
            return Err(ExtensionHostError::ManifestNotFound {
                extension_id: installation.extension_id,
            });
        }

        self.installations
            .insert(installation.installation_id.clone(), installation);
        Ok(())
    }

    pub fn installations(&self) -> Vec<ExtensionInstallation> {
        self.installations.values().cloned().collect()
    }

    pub fn mount_instance(
        &mut self,
        instance: ExtensionInstance,
    ) -> Result<(), ExtensionHostError> {
        let Some(installation) = self.installations.get(&instance.installation_id) else {
            return Err(ExtensionHostError::InstallationNotFound {
                installation_id: instance.installation_id,
            });
        };

        if installation.extension_id != instance.extension_id {
            return Err(ExtensionHostError::InstallationExtensionMismatch {
                installation_id: installation.installation_id.clone(),
                installation_extension_id: installation.extension_id.clone(),
                instance_extension_id: instance.extension_id,
            });
        }

        let instances = self
            .instances_by_extension
            .entry(installation.extension_id.clone())
            .or_default();

        if let Some(existing) = instances
            .iter_mut()
            .find(|existing| existing.instance_id == instance.instance_id)
        {
            *existing = instance;
        } else {
            instances.push(instance);
        }
        Ok(())
    }

    pub fn instances(&self, extension_id: &str) -> Vec<ExtensionInstance> {
        self.instances_by_extension
            .get(extension_id)
            .cloned()
            .unwrap_or_default()
    }

    pub fn resolve_provider(
        &self,
        adapter_kind: &str,
        base_url: impl Into<String>,
    ) -> Option<Box<dyn ProviderExecutionAdapter>> {
        self.provider_factories
            .get(adapter_kind)
            .map(|factory| factory(base_url.into()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExtensionHostError {
    ManifestNotFound {
        extension_id: String,
    },
    InstallationNotFound {
        installation_id: String,
    },
    InstallationExtensionMismatch {
        installation_id: String,
        installation_extension_id: String,
        instance_extension_id: String,
    },
}

impl fmt::Display for ExtensionHostError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ManifestNotFound { extension_id } => {
                write!(f, "extension manifest not found: {extension_id}")
            }
            Self::InstallationNotFound { installation_id } => {
                write!(f, "extension installation not found: {installation_id}")
            }
            Self::InstallationExtensionMismatch {
                installation_id,
                installation_extension_id,
                instance_extension_id,
            } => write!(
                f,
                "extension instance references {} but installation {} is bound to {}",
                instance_extension_id, installation_id, installation_extension_id
            ),
        }
    }
}

impl std::error::Error for ExtensionHostError {}
