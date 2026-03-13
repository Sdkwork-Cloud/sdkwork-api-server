use std::str::FromStr;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExtensionKind {
    Channel,
    Provider,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExtensionRuntime {
    Builtin,
    NativeDynamic,
    Connector,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
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

impl ExtensionRuntime {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Builtin => "builtin",
            Self::NativeDynamic => "native_dynamic",
            Self::Connector => "connector",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseExtensionRuntimeError(String);

impl std::fmt::Display for ParseExtensionRuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for ParseExtensionRuntimeError {}

impl FromStr for ExtensionRuntime {
    type Err = ParseExtensionRuntimeError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "builtin" => Ok(Self::Builtin),
            "native_dynamic" => Ok(Self::NativeDynamic),
            "connector" => Ok(Self::Connector),
            other => Err(ParseExtensionRuntimeError(format!(
                "unknown extension runtime: {other}"
            ))),
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExtensionInstallation {
    pub installation_id: String,
    pub extension_id: String,
    pub runtime: ExtensionRuntime,
    pub enabled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entrypoint: Option<String>,
    #[serde(default = "default_config")]
    pub config: Value,
}

impl ExtensionInstallation {
    pub fn new(
        installation_id: impl Into<String>,
        extension_id: impl Into<String>,
        runtime: ExtensionRuntime,
    ) -> Self {
        Self {
            installation_id: installation_id.into(),
            extension_id: extension_id.into(),
            runtime,
            enabled: true,
            entrypoint: None,
            config: default_config(),
        }
    }

    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn with_entrypoint(mut self, entrypoint: impl Into<String>) -> Self {
        self.entrypoint = Some(entrypoint.into());
        self
    }

    pub fn with_config(mut self, config: Value) -> Self {
        self.config = config;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExtensionInstance {
    pub instance_id: String,
    pub installation_id: String,
    pub extension_id: String,
    pub enabled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub credential_ref: Option<String>,
    #[serde(default = "default_config")]
    pub config: Value,
}

impl ExtensionInstance {
    pub fn new(
        instance_id: impl Into<String>,
        installation_id: impl Into<String>,
        extension_id: impl Into<String>,
    ) -> Self {
        Self {
            instance_id: instance_id.into(),
            installation_id: installation_id.into(),
            extension_id: extension_id.into(),
            enabled: true,
            base_url: None,
            credential_ref: None,
            config: default_config(),
        }
    }

    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = Some(base_url.into());
        self
    }

    pub fn with_credential_ref(mut self, credential_ref: impl Into<String>) -> Self {
        self.credential_ref = Some(credential_ref.into());
        self
    }

    pub fn with_config(mut self, config: Value) -> Self {
        self.config = config;
        self
    }
}

fn default_config() -> Value {
    Value::Object(Default::default())
}
