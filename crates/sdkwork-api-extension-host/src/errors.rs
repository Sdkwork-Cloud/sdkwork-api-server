use super::*;

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
        manifest_runtime: ExtensionRuntime,
        installation_runtime: ExtensionRuntime,
    },
    ManifestReadFailed {
        path: String,
        message: String,
    },
    ManifestParseFailed {
        path: String,
        message: String,
    },
    NativeDynamicLibraryLoadFailed {
        entrypoint: String,
        message: String,
    },
    NativeDynamicSymbolMissing {
        entrypoint: String,
        symbol: String,
        message: String,
    },
    NativeDynamicAbiVersionUnsupported {
        entrypoint: String,
        actual_version: u32,
    },
    NativeDynamicManifestExportMissing {
        entrypoint: String,
    },
    NativeDynamicManifestMismatch {
        entrypoint: String,
        message: String,
    },
    NativeDynamicInvocationSerializeFailed {
        operation: String,
        message: String,
    },
    NativeDynamicInvocationFailed {
        entrypoint: String,
        message: String,
    },
    NativeDynamicResponseParseFailed {
        entrypoint: String,
        message: String,
    },
    NativeDynamicLifecycleFailed {
        entrypoint: String,
        phase: String,
        message: String,
    },
    NativeDynamicShutdownDrainTimedOut {
        entrypoint: String,
        timeout_ms: u64,
    },
    NativeDynamicShutdownDrainTimeoutInvalid {
        value: String,
    },
    NativeDynamicRuntimeStatePoisoned {
        entrypoint: String,
    },
    ConnectorRuntimeUnsupported {
        instance_id: String,
        runtime: ExtensionRuntime,
    },
    ConnectorRuntimeEntrypointMissing {
        instance_id: String,
    },
    ConnectorRuntimeBaseUrlInvalid {
        instance_id: String,
        base_url: String,
    },
    ConnectorRuntimeSpawnFailed {
        instance_id: String,
        entrypoint: String,
        message: String,
    },
    ConnectorRuntimeStatePoisoned,
    ConnectorRuntimeExited {
        instance_id: String,
        status: Option<i32>,
    },
    ConnectorRuntimeHealthTimedOut {
        instance_id: String,
        health_url: String,
        timeout_ms: u64,
    },
    ConnectorRuntimeShutdownFailed {
        instance_id: String,
        message: String,
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
            Self::ManifestReadFailed { path, message } => {
                write!(f, "failed to read extension manifest {}: {}", path, message)
            }
            Self::ManifestParseFailed { path, message } => {
                write!(
                    f,
                    "failed to parse extension manifest {}: {}",
                    path, message
                )
            }
            Self::NativeDynamicLibraryLoadFailed {
                entrypoint,
                message,
            } => write!(
                f,
                "failed to load native dynamic extension library {}: {}",
                entrypoint, message
            ),
            Self::NativeDynamicSymbolMissing {
                entrypoint,
                symbol,
                message,
            } => write!(
                f,
                "native dynamic extension library {} is missing symbol {}: {}",
                entrypoint, symbol, message
            ),
            Self::NativeDynamicAbiVersionUnsupported {
                entrypoint,
                actual_version,
            } => write!(
                f,
                "native dynamic extension library {} reported unsupported ABI version {}",
                entrypoint, actual_version
            ),
            Self::NativeDynamicManifestExportMissing { entrypoint } => write!(
                f,
                "native dynamic extension library {} returned no manifest export",
                entrypoint
            ),
            Self::NativeDynamicManifestMismatch {
                entrypoint,
                message,
            } => write!(
                f,
                "native dynamic extension library {} manifest does not match package manifest: {}",
                entrypoint, message
            ),
            Self::NativeDynamicInvocationSerializeFailed { operation, message } => write!(
                f,
                "failed to serialize native dynamic provider invocation {}: {}",
                operation, message
            ),
            Self::NativeDynamicInvocationFailed {
                entrypoint,
                message,
            } => write!(
                f,
                "native dynamic extension library {} failed during invocation: {}",
                entrypoint, message
            ),
            Self::NativeDynamicResponseParseFailed {
                entrypoint,
                message,
            } => write!(
                f,
                "native dynamic extension library {} returned invalid response payload: {}",
                entrypoint, message
            ),
            Self::NativeDynamicLifecycleFailed {
                entrypoint,
                phase,
                message,
            } => write!(
                f,
                "native dynamic extension library {} failed during {}: {}",
                entrypoint, phase, message
            ),
            Self::NativeDynamicShutdownDrainTimedOut {
                entrypoint,
                timeout_ms,
            } => write!(
                f,
                "native dynamic extension library {} drain timed out after {}ms; runtime kept running",
                entrypoint, timeout_ms
            ),
            Self::NativeDynamicShutdownDrainTimeoutInvalid { value } => write!(
                f,
                "native dynamic shutdown drain timeout is invalid: {}",
                value
            ),
            Self::NativeDynamicRuntimeStatePoisoned { entrypoint } => write!(
                f,
                "native dynamic extension library {} runtime state is poisoned",
                entrypoint
            ),
            Self::ConnectorRuntimeUnsupported {
                instance_id,
                runtime,
            } => write!(
                f,
                "extension instance {} uses unsupported connector runtime {:?}",
                instance_id, runtime
            ),
            Self::ConnectorRuntimeEntrypointMissing { instance_id } => write!(
                f,
                "connector extension instance {} has no executable entrypoint",
                instance_id
            ),
            Self::ConnectorRuntimeBaseUrlInvalid {
                instance_id,
                base_url,
            } => write!(
                f,
                "connector extension instance {} has invalid base url {}",
                instance_id, base_url
            ),
            Self::ConnectorRuntimeSpawnFailed {
                instance_id,
                entrypoint,
                message,
            } => write!(
                f,
                "failed to spawn connector extension instance {} from {}: {}",
                instance_id, entrypoint, message
            ),
            Self::ConnectorRuntimeStatePoisoned => {
                write!(f, "connector runtime state is poisoned")
            }
            Self::ConnectorRuntimeExited {
                instance_id,
                status,
            } => write!(
                f,
                "connector extension instance {} exited before becoming ready with status {:?}",
                instance_id, status
            ),
            Self::ConnectorRuntimeHealthTimedOut {
                instance_id,
                health_url,
                timeout_ms,
            } => write!(
                f,
                "connector extension instance {} did not become healthy at {} within {}ms",
                instance_id, health_url, timeout_ms
            ),
            Self::ConnectorRuntimeShutdownFailed {
                instance_id,
                message,
            } => write!(
                f,
                "failed to shut down connector extension instance {}: {}",
                instance_id, message
            ),
        }
    }
}

impl std::error::Error for ExtensionHostError {}
