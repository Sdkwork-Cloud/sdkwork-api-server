use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use sdkwork_api_extension_core::{ExtensionInstallation, ExtensionInstance, ExtensionManifest};
use sdkwork_api_provider_core::ProviderExecutionAdapter;
use serde_json::Value;

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
    provider_aliases: HashMap<String, String>,
    installations: HashMap<String, ExtensionInstallation>,
    instances_by_id: HashMap<String, ExtensionInstance>,
    instances_by_extension: HashMap<String, Vec<ExtensionInstance>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExtensionLoadPlan {
    pub instance_id: String,
    pub installation_id: String,
    pub extension_id: String,
    pub runtime: sdkwork_api_extension_core::ExtensionRuntime,
    pub display_name: String,
    pub entrypoint: Option<String>,
    pub base_url: Option<String>,
    pub credential_ref: Option<String>,
    pub config_schema: Option<String>,
    pub credential_schema: Option<String>,
    pub config: Value,
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
        let extension_id = factory.manifest.id.clone();
        self.manifests
            .insert(extension_id.clone(), factory.manifest);
        self.provider_factories
            .insert(extension_id.clone(), factory.factory);
        self.provider_aliases
            .insert(factory.adapter_kind, extension_id);
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
            *existing = instance.clone();
        } else {
            instances.push(instance.clone());
        }

        self.instances_by_id
            .insert(instance.instance_id.clone(), instance);
        Ok(())
    }

    pub fn instances(&self, extension_id: &str) -> Vec<ExtensionInstance> {
        self.instances_by_extension
            .get(extension_id)
            .cloned()
            .unwrap_or_default()
    }

    pub fn load_plan(&self, instance_id: &str) -> Result<ExtensionLoadPlan, ExtensionHostError> {
        let Some(instance) = self.instances_by_id.get(instance_id) else {
            return Err(ExtensionHostError::InstanceNotFound {
                instance_id: instance_id.to_owned(),
            });
        };
        let Some(installation) = self.installations.get(&instance.installation_id) else {
            return Err(ExtensionHostError::InstallationNotFound {
                installation_id: instance.installation_id.clone(),
            });
        };
        let Some(manifest) = self.manifests.get(&instance.extension_id) else {
            return Err(ExtensionHostError::ManifestNotFound {
                extension_id: instance.extension_id.clone(),
            });
        };

        if installation.runtime != manifest.runtime {
            return Err(ExtensionHostError::RuntimeMismatch {
                extension_id: manifest.id.clone(),
                manifest_runtime: manifest.runtime.clone(),
                installation_runtime: installation.runtime.clone(),
            });
        }

        Ok(ExtensionLoadPlan {
            instance_id: instance.instance_id.clone(),
            installation_id: installation.installation_id.clone(),
            extension_id: manifest.id.clone(),
            runtime: installation.runtime.clone(),
            display_name: manifest.display_name.clone(),
            entrypoint: installation
                .entrypoint
                .clone()
                .or_else(|| manifest.entrypoint.clone()),
            base_url: instance.base_url.clone(),
            credential_ref: instance.credential_ref.clone(),
            config_schema: manifest.config_schema.clone(),
            credential_schema: manifest.credential_schema.clone(),
            config: merge_config(&installation.config, &instance.config),
        })
    }

    pub fn resolve_provider(
        &self,
        runtime_key: &str,
        base_url: impl Into<String>,
    ) -> Option<Box<dyn ProviderExecutionAdapter>> {
        let base_url = base_url.into();

        self.provider_factories
            .get(runtime_key)
            .or_else(|| {
                self.provider_aliases
                    .get(runtime_key)
                    .and_then(|extension_id| self.provider_factories.get(extension_id))
            })
            .map(|factory| factory(base_url))
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
    InstanceNotFound {
        instance_id: String,
    },
    InstallationExtensionMismatch {
        installation_id: String,
        installation_extension_id: String,
        instance_extension_id: String,
    },
    RuntimeMismatch {
        extension_id: String,
        manifest_runtime: sdkwork_api_extension_core::ExtensionRuntime,
        installation_runtime: sdkwork_api_extension_core::ExtensionRuntime,
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
            Self::InstanceNotFound { instance_id } => {
                write!(f, "extension instance not found: {instance_id}")
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
            Self::RuntimeMismatch {
                extension_id,
                manifest_runtime,
                installation_runtime,
            } => write!(
                f,
                "extension {} manifest runtime {:?} does not match installation runtime {:?}",
                extension_id, manifest_runtime, installation_runtime
            ),
        }
    }
}

impl std::error::Error for ExtensionHostError {}

fn merge_config(base: &Value, overlay: &Value) -> Value {
    match (base, overlay) {
        (Value::Object(base_map), Value::Object(overlay_map)) => {
            let mut merged = base_map.clone();
            for (key, overlay_value) in overlay_map {
                let value = match merged.get(key) {
                    Some(base_value) => merge_config(base_value, overlay_value),
                    None => overlay_value.clone(),
                };
                merged.insert(key.clone(), value);
            }
            Value::Object(merged)
        }
        (_, overlay) => overlay.clone(),
    }
}
