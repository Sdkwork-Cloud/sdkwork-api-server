use std::collections::HashMap;
use std::sync::Arc;

use sdkwork_api_extension_core::ExtensionManifest;
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
