use super::*;

pub fn discover_extension_packages(
    policy: &ExtensionDiscoveryPolicy,
) -> Result<Vec<DiscoveredExtensionPackage>, ExtensionHostError> {
    let mut packages = Vec::new();
    for search_path in &policy.search_paths {
        discover_in_path(search_path, policy, &mut packages)?;
    }
    packages.sort_by(|left, right| left.manifest_path.cmp(&right.manifest_path));
    Ok(packages)
}

pub fn validate_discovered_extension_package(
    package: &DiscoveredExtensionPackage,
) -> ManifestValidationReport {
    validate_extension_manifest(&package.manifest)
}

pub fn validate_extension_manifest(manifest: &ExtensionManifest) -> ManifestValidationReport {
    let mut issues = Vec::new();

    if manifest.permissions.is_empty() {
        issues.push(ManifestValidationIssue {
            severity: ManifestValidationSeverity::Error,
            code: "missing_permissions".to_owned(),
            message: "extension manifest must declare explicit permissions".to_owned(),
        });
    }

    if manifest.channel_bindings.is_empty() {
        issues.push(ManifestValidationIssue {
            severity: ManifestValidationSeverity::Error,
            code: "missing_channel_bindings".to_owned(),
            message: "extension manifest must declare at least one channel binding".to_owned(),
        });
    }

    if manifest.capabilities.is_empty() {
        issues.push(ManifestValidationIssue {
            severity: ManifestValidationSeverity::Error,
            code: "missing_capabilities".to_owned(),
            message: "extension manifest must declare at least one capability".to_owned(),
        });
    }

    if manifest.supported_modalities.is_empty() {
        issues.push(ManifestValidationIssue {
            severity: ManifestValidationSeverity::Error,
            code: "missing_supported_modalities".to_owned(),
            message: "extension manifest must declare at least one supported modality".to_owned(),
        });
    }

    if manifest
        .runtime_compat_version
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .is_none()
    {
        issues.push(ManifestValidationIssue {
            severity: ManifestValidationSeverity::Error,
            code: "missing_runtime_compat_version".to_owned(),
            message: "extension manifest must declare a runtime compatibility version".to_owned(),
        });
    }

    if manifest
        .config_schema_version
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .is_none()
    {
        issues.push(ManifestValidationIssue {
            severity: ManifestValidationSeverity::Error,
            code: "missing_config_schema_version".to_owned(),
            message: "extension manifest must declare a config schema version".to_owned(),
        });
    }

    if matches!(
        manifest.runtime,
        ExtensionRuntime::Connector | ExtensionRuntime::NativeDynamic
    ) && manifest.entrypoint.is_none()
    {
        issues.push(ManifestValidationIssue {
            severity: ManifestValidationSeverity::Error,
            code: "missing_entrypoint".to_owned(),
            message: "runtime-backed extension manifest must declare an entrypoint".to_owned(),
        });
    }

    if manifest.runtime == ExtensionRuntime::Connector && manifest.health.is_none() {
        issues.push(ManifestValidationIssue {
            severity: ManifestValidationSeverity::Warning,
            code: "missing_health_contract".to_owned(),
            message: "connector extensions should declare an explicit health contract".to_owned(),
        });
    }

    let valid = !issues
        .iter()
        .any(|issue| issue.severity == ManifestValidationSeverity::Error);
    ManifestValidationReport { valid, issues }
}

fn discover_in_path(
    path: &Path,
    policy: &ExtensionDiscoveryPolicy,
    packages: &mut Vec<DiscoveredExtensionPackage>,
) -> Result<(), ExtensionHostError> {
    if !path.exists() {
        return Ok(());
    }

    if path.is_file() {
        if path.file_name().and_then(|name| name.to_str()) == Some("sdkwork-extension.toml") {
            let manifest = parse_manifest(path)?;
            if policy.allows_runtime(&manifest.runtime) {
                let root_dir = path.parent().unwrap_or(path).to_path_buf();
                packages.push(DiscoveredExtensionPackage {
                    root_dir,
                    manifest_path: path.to_path_buf(),
                    manifest,
                });
            }
        }
        return Ok(());
    }

    let entries = fs::read_dir(path).map_err(|error| ExtensionHostError::ManifestReadFailed {
        path: path.display().to_string(),
        message: error.to_string(),
    })?;

    for entry in entries {
        let entry = entry.map_err(|error| ExtensionHostError::ManifestReadFailed {
            path: path.display().to_string(),
            message: error.to_string(),
        })?;
        discover_in_path(&entry.path(), policy, packages)?;
    }

    Ok(())
}

fn parse_manifest(path: &Path) -> Result<ExtensionManifest, ExtensionHostError> {
    let manifest_text =
        fs::read_to_string(path).map_err(|error| ExtensionHostError::ManifestReadFailed {
            path: path.display().to_string(),
            message: error.to_string(),
        })?;

    toml::from_str(&manifest_text).map_err(|error| ExtensionHostError::ManifestParseFailed {
        path: path.display().to_string(),
        message: error.to_string(),
    })
}
