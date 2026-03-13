use std::collections::HashMap;

use sdkwork_api_extension_core::ExtensionManifest;

#[derive(Debug, Clone)]
pub struct BuiltinExtensionFactory {
    manifest: ExtensionManifest,
}

impl BuiltinExtensionFactory {
    pub fn new(manifest: ExtensionManifest) -> Self {
        Self { manifest }
    }
}

#[derive(Default, Debug, Clone)]
pub struct ExtensionHost {
    manifests: HashMap<String, ExtensionManifest>,
}

impl ExtensionHost {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_builtin(&mut self, factory: BuiltinExtensionFactory) {
        self.manifests
            .insert(factory.manifest.id.clone(), factory.manifest);
    }

    pub fn manifest(&self, id: &str) -> Option<&ExtensionManifest> {
        self.manifests.get(id)
    }
}
