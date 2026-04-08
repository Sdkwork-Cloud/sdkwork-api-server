use super::*;

pub fn verify_discovered_extension_package_trust(
    package: &DiscoveredExtensionPackage,
    policy: &ExtensionDiscoveryPolicy,
) -> ExtensionTrustReport {
    let requires_signature = policy.requires_signature(&package.manifest.runtime);
    let Some(trust) = package.manifest.trust.as_ref() else {
        return ExtensionTrustReport {
            state: ExtensionTrustState::Unsigned,
            publisher: None,
            signature_present: false,
            signature_verified: false,
            trusted_signer: false,
            load_allowed: !requires_signature,
            issues: vec![ExtensionTrustIssue {
                code: "unsigned_package".to_owned(),
                message: if requires_signature {
                    "extension package must be signed by a trusted publisher for this runtime"
                        .to_owned()
                } else {
                    "extension package does not declare trust metadata".to_owned()
                },
            }],
        };
    };

    let payload = match package_signature_payload(package) {
        Ok(payload) => payload,
        Err(message) => {
            return ExtensionTrustReport {
                state: ExtensionTrustState::VerificationFailed,
                publisher: Some(trust.publisher.clone()),
                signature_present: true,
                signature_verified: false,
                trusted_signer: false,
                load_allowed: false,
                issues: vec![ExtensionTrustIssue {
                    code: "package_payload_unreadable".to_owned(),
                    message: message.to_string(),
                }],
            };
        }
    };

    if let Err(message) = verify_signature_bytes(
        &payload,
        trust.signature.algorithm.clone(),
        &trust.signature.public_key,
        &trust.signature.signature,
    ) {
        return ExtensionTrustReport {
            state: ExtensionTrustState::InvalidSignature,
            publisher: Some(trust.publisher.clone()),
            signature_present: true,
            signature_verified: false,
            trusted_signer: false,
            load_allowed: false,
            issues: vec![ExtensionTrustIssue {
                code: "invalid_signature".to_owned(),
                message,
            }],
        };
    }

    let trusted_signer = policy
        .trusted_signers
        .get(&trust.publisher)
        .map(|expected_public_key| {
            public_keys_match(expected_public_key, &trust.signature.public_key)
        })
        .unwrap_or(false);
    if !trusted_signer {
        return ExtensionTrustReport {
            state: ExtensionTrustState::UntrustedSigner,
            publisher: Some(trust.publisher.clone()),
            signature_present: true,
            signature_verified: true,
            trusted_signer: false,
            load_allowed: false,
            issues: vec![ExtensionTrustIssue {
                code: "untrusted_signer".to_owned(),
                message: format!(
                    "publisher {} is not trusted by the current extension trust policy",
                    trust.publisher
                ),
            }],
        };
    }

    ExtensionTrustReport {
        state: ExtensionTrustState::Verified,
        publisher: Some(trust.publisher.clone()),
        signature_present: true,
        signature_verified: true,
        trusted_signer: true,
        load_allowed: true,
        issues: Vec::new(),
    }
}

#[derive(Serialize)]
struct PackageSignaturePayload {
    manifest: ExtensionManifest,
    files: Vec<PackageFileDigest>,
}

#[derive(Serialize)]
struct PackageFileDigest {
    path: String,
    sha256: String,
}

fn package_signature_payload(
    package: &DiscoveredExtensionPackage,
) -> Result<Vec<u8>, ExtensionHostError> {
    let mut manifest = package.manifest.clone();
    manifest.trust = None;
    let payload = PackageSignaturePayload {
        manifest,
        files: collect_package_file_digests(&package.root_dir)?,
    };
    serde_json::to_vec(&payload).map_err(|error| ExtensionHostError::ManifestReadFailed {
        path: package.manifest_path.display().to_string(),
        message: error.to_string(),
    })
}

fn collect_package_file_digests(root: &Path) -> Result<Vec<PackageFileDigest>, ExtensionHostError> {
    let mut files = Vec::new();
    collect_package_file_digests_in(root, root, &mut files)?;
    files.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(files)
}

fn collect_package_file_digests_in(
    root: &Path,
    current: &Path,
    files: &mut Vec<PackageFileDigest>,
) -> Result<(), ExtensionHostError> {
    let entries =
        fs::read_dir(current).map_err(|error| ExtensionHostError::ManifestReadFailed {
            path: current.display().to_string(),
            message: error.to_string(),
        })?;
    for entry in entries {
        let entry = entry.map_err(|error| ExtensionHostError::ManifestReadFailed {
            path: current.display().to_string(),
            message: error.to_string(),
        })?;
        let path = entry.path();
        if path.is_dir() {
            collect_package_file_digests_in(root, &path, files)?;
            continue;
        }
        if path.file_name().and_then(|name| name.to_str()) == Some("sdkwork-extension.toml") {
            continue;
        }

        let bytes = fs::read(&path).map_err(|error| ExtensionHostError::ManifestReadFailed {
            path: path.display().to_string(),
            message: error.to_string(),
        })?;
        let relative_path = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        files.push(PackageFileDigest {
            path: relative_path,
            sha256: sha256_hex(&bytes),
        });
    }
    Ok(())
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut encoded = String::with_capacity(digest.len() * 2);
    for byte in digest {
        encoded.push_str(&format!("{byte:02x}"));
    }
    encoded
}

fn verify_signature_bytes(
    payload: &[u8],
    algorithm: ExtensionSignatureAlgorithm,
    public_key: &str,
    signature: &str,
) -> Result<(), String> {
    match algorithm {
        ExtensionSignatureAlgorithm::Ed25519 => {
            let public_key_bytes = decode_fixed_base64::<32>(public_key, "public key")?;
            let verifying_key =
                VerifyingKey::from_bytes(&public_key_bytes).map_err(|error| error.to_string())?;
            let signature_bytes = decode_fixed_base64::<64>(signature, "signature")?;
            let signature = Signature::from_bytes(&signature_bytes);
            verifying_key
                .verify(payload, &signature)
                .map_err(|error| error.to_string())
        }
    }
}

fn public_keys_match(expected_public_key: &str, actual_public_key: &str) -> bool {
    match (
        STANDARD.decode(expected_public_key),
        STANDARD.decode(actual_public_key),
    ) {
        (Ok(expected), Ok(actual)) => expected == actual,
        _ => expected_public_key == actual_public_key,
    }
}

fn decode_fixed_base64<const N: usize>(value: &str, label: &str) -> Result<[u8; N], String> {
    let decoded = STANDARD
        .decode(value)
        .map_err(|error| format!("invalid {label} encoding: {error}"))?;
    decoded
        .try_into()
        .map_err(|_| format!("invalid {label} length"))
}
