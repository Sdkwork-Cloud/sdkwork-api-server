use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExtensionKind {
    Channel,
    Provider,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExtensionRuntime {
    Builtin,
    NativeDynamic,
    Connector,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompatibilityLevel {
    Native,
    Relay,
    Translated,
    Emulated,
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityDescriptor {
    pub operation: String,
    pub compatibility: CompatibilityLevel,
}

impl CapabilityDescriptor {
    pub fn new(operation: impl Into<String>, compatibility: CompatibilityLevel) -> Self {
        Self {
            operation: operation.into(),
            compatibility,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtensionManifest {
    pub api_version: String,
    pub id: String,
    pub kind: ExtensionKind,
    pub version: String,
    pub runtime: ExtensionRuntime,
    pub channel_bindings: Vec<String>,
    pub capabilities: Vec<CapabilityDescriptor>,
}

impl ExtensionManifest {
    pub fn new(
        id: impl Into<String>,
        kind: ExtensionKind,
        version: impl Into<String>,
        runtime: ExtensionRuntime,
    ) -> Self {
        Self {
            api_version: "sdkwork.extension/v1".to_owned(),
            id: id.into(),
            kind,
            version: version.into(),
            runtime,
            channel_bindings: Vec::new(),
            capabilities: Vec::new(),
        }
    }

    pub fn with_capability(mut self, capability: CapabilityDescriptor) -> Self {
        self.capabilities.push(capability);
        self
    }

    pub fn with_channel_binding(mut self, channel_id: impl Into<String>) -> Self {
        self.channel_bindings.push(channel_id.into());
        self
    }
}
