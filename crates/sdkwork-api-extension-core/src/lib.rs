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
pub enum ExtensionModality {
    Text,
    Image,
    Audio,
    Video,
    File,
    Embedding,
    Music,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExtensionProtocol {
    #[serde(rename = "openai")]
    OpenAi,
    #[serde(rename = "anthropic")]
    Anthropic,
    #[serde(rename = "gemini")]
    Gemini,
    #[serde(rename = "custom")]
    Custom,
    #[serde(rename = "openrouter")]
    OpenRouter,
    #[serde(rename = "ollama")]
    Ollama,
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
#[serde(rename_all = "snake_case")]
pub enum ExtensionPermission {
    NetworkOutbound,
    FilesystemRead,
    FilesystemWrite,
    SpawnProcess,
    LoopbackBind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExtensionSignatureAlgorithm {
    Ed25519,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtensionSignature {
    pub algorithm: ExtensionSignatureAlgorithm,
    pub public_key: String,
    pub signature: String,
}

impl ExtensionSignature {
    pub fn new(
        algorithm: ExtensionSignatureAlgorithm,
        public_key: impl Into<String>,
        signature: impl Into<String>,
    ) -> Self {
        Self {
            algorithm,
            public_key: public_key.into(),
            signature: signature.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtensionTrustDeclaration {
    pub publisher: String,
    pub signature: ExtensionSignature,
}

impl ExtensionTrustDeclaration {
    pub fn signed(publisher: impl Into<String>, signature: ExtensionSignature) -> Self {
        Self {
            publisher: publisher.into(),
            signature,
        }
    }
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
pub struct ExtensionHealthContract {
    pub path: String,
    pub interval_secs: u64,
}

impl ExtensionHealthContract {
    pub fn new(path: impl Into<String>, interval_secs: u64) -> Self {
        Self {
            path: path.into(),
            interval_secs,
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

    pub fn supports_raw_provider_execution(&self) -> bool {
        matches!(self, Self::NativeDynamic)
    }

    pub fn supports_structured_retry_hints(&self) -> bool {
        self.supports_raw_provider_execution()
    }
}

impl ExtensionProtocol {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::OpenAi => "openai",
            Self::Anthropic => "anthropic",
            Self::Gemini => "gemini",
            Self::Custom => "custom",
            Self::OpenRouter => "openrouter",
            Self::Ollama => "ollama",
        }
    }

    pub fn protocol_capability(self) -> Self {
        match self {
            Self::OpenRouter => Self::OpenAi,
            Self::Ollama => Self::Custom,
            other => other,
        }
    }

    pub fn capability_key(self) -> &'static str {
        self.protocol_capability().as_str()
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseExtensionProtocolError(String);

impl std::fmt::Display for ParseExtensionProtocolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for ParseExtensionProtocolError {}

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

impl FromStr for ExtensionProtocol {
    type Err = ParseExtensionProtocolError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "openai" => Ok(Self::OpenAi),
            "anthropic" => Ok(Self::Anthropic),
            "gemini" => Ok(Self::Gemini),
            "custom" => Ok(Self::Custom),
            "openrouter" => Ok(Self::OpenRouter),
            "ollama" => Ok(Self::Ollama),
            other => Err(ParseExtensionProtocolError(format!(
                "unknown extension protocol: {other}"
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
    pub display_name: String,
    pub runtime: ExtensionRuntime,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub supported_modalities: Vec<ExtensionModality>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protocol: Option<ExtensionProtocol>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entrypoint: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime_compat_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub config_schema: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub config_schema_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub credential_schema: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub permissions: Vec<ExtensionPermission>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub health: Option<ExtensionHealthContract>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trust: Option<ExtensionTrustDeclaration>,
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
        let id = id.into();
        Self {
            api_version: "sdkwork.extension/v1".to_owned(),
            display_name: id.clone(),
            id,
            kind,
            version: version.into(),
            runtime,
            supported_modalities: vec![ExtensionModality::Text],
            protocol: None,
            entrypoint: None,
            runtime_compat_version: Some(default_runtime_compat_version().to_owned()),
            config_schema: None,
            config_schema_version: Some(default_config_schema_version().to_owned()),
            credential_schema: None,
            permissions: Vec::new(),
            health: None,
            trust: None,
            channel_bindings: Vec::new(),
            capabilities: Vec::new(),
        }
    }

    pub fn distribution_name(&self) -> String {
        self.id.replace('.', "-")
    }

    pub fn crate_name(&self) -> String {
        let suffix = self.id.strip_prefix("sdkwork.").unwrap_or(&self.id);
        format!("sdkwork-api-ext-{}", suffix.replace('.', "-"))
    }

    pub fn with_display_name(mut self, display_name: impl Into<String>) -> Self {
        self.display_name = display_name.into();
        self
    }

    pub fn with_entrypoint(mut self, entrypoint: impl Into<String>) -> Self {
        self.entrypoint = Some(entrypoint.into());
        self
    }

    pub fn with_supported_modality(mut self, modality: ExtensionModality) -> Self {
        if !self.supported_modalities.contains(&modality) {
            self.supported_modalities.push(modality);
        }
        self
    }

    pub fn with_protocol(mut self, protocol: ExtensionProtocol) -> Self {
        self.protocol = Some(protocol);
        self
    }

    pub fn protocol_capability(&self) -> Option<ExtensionProtocol> {
        self.protocol.map(ExtensionProtocol::protocol_capability)
    }

    pub fn with_runtime_compat_version(
        mut self,
        runtime_compat_version: impl Into<String>,
    ) -> Self {
        self.runtime_compat_version = Some(runtime_compat_version.into());
        self
    }

    pub fn with_config_schema(mut self, config_schema: impl Into<String>) -> Self {
        self.config_schema = Some(config_schema.into());
        self
    }

    pub fn with_config_schema_version(mut self, config_schema_version: impl Into<String>) -> Self {
        self.config_schema_version = Some(config_schema_version.into());
        self
    }

    pub fn with_credential_schema(mut self, credential_schema: impl Into<String>) -> Self {
        self.credential_schema = Some(credential_schema.into());
        self
    }

    pub fn with_permission(mut self, permission: ExtensionPermission) -> Self {
        self.permissions.push(permission);
        self
    }

    pub fn with_health_contract(mut self, health: ExtensionHealthContract) -> Self {
        self.health = Some(health);
        self
    }

    pub fn with_trust(mut self, trust: ExtensionTrustDeclaration) -> Self {
        self.trust = Some(trust);
        self
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

fn default_runtime_compat_version() -> &'static str {
    "sdkwork.runtime/v1"
}

fn default_config_schema_version() -> &'static str {
    "1.0"
}
