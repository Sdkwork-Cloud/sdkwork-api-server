use super::*;

#[derive(Clone)]
struct CachedConfiguredExtensionHost {
    key: ConfiguredExtensionHostCacheKey,
    host: ExtensionHost,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConfiguredExtensionHostReloadReport {
    pub discovered_package_count: usize,
    pub loadable_package_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfiguredExtensionHostReloadScope {
    All,
    Extension { extension_id: String },
    Instance { instance_id: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ConfiguredExtensionHostCacheKey {
    search_paths: Vec<PathBuf>,
    enable_connector_extensions: bool,
    enable_native_dynamic_extensions: bool,
    require_signed_connector_extensions: bool,
    require_signed_native_dynamic_extensions: bool,
    trusted_signers: Vec<(String, String)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ConfiguredExtensionHostWatchState {
    key: ConfiguredExtensionHostCacheKey,
    fingerprint: Vec<String>,
}

impl From<&ExtensionDiscoveryPolicy> for ConfiguredExtensionHostCacheKey {
    fn from(policy: &ExtensionDiscoveryPolicy) -> Self {
        let mut trusted_signers = policy
            .trusted_signers
            .iter()
            .map(|(publisher, public_key)| (publisher.clone(), public_key.clone()))
            .collect::<Vec<_>>();
        trusted_signers.sort_unstable();

        Self {
            search_paths: policy.search_paths.clone(),
            enable_connector_extensions: policy.enable_connector_extensions,
            enable_native_dynamic_extensions: policy.enable_native_dynamic_extensions,
            require_signed_connector_extensions: policy.require_signed_connector_extensions,
            require_signed_native_dynamic_extensions: policy
                .require_signed_native_dynamic_extensions,
            trusted_signers,
        }
    }
}

static CONFIGURED_EXTENSION_HOST_CACHE: OnceLock<Mutex<Option<CachedConfiguredExtensionHost>>> =
    OnceLock::new();

struct BuiltConfiguredExtensionHost {
    host: ExtensionHost,
    discovered_package_count: usize,
    loadable_package_count: usize,
}

pub fn builtin_extension_host() -> ExtensionHost {
    let mut host = ExtensionHost::new();
    register_builtin_openai_provider(&mut host, "sdkwork.provider.openai.official", "openai");
    for (extension_id, adapter_kind) in [
        ("sdkwork.provider.xai", "xai"),
        ("sdkwork.provider.deepseek", "deepseek"),
        ("sdkwork.provider.qwen", "qwen"),
        ("sdkwork.provider.doubao", "doubao"),
        ("sdkwork.provider.hunyuan", "hunyuan"),
        ("sdkwork.provider.moonshot", "moonshot"),
        ("sdkwork.provider.zhipu", "zhipu"),
        ("sdkwork.provider.mistral", "mistral"),
        ("sdkwork.provider.cohere", "cohere"),
        ("sdkwork.provider.siliconflow", "siliconflow"),
    ] {
        register_builtin_openai_compatible_provider(&mut host, extension_id, adapter_kind);
    }
    register_builtin_openrouter_provider(&mut host);
    register_builtin_ollama_provider(&mut host);
    host
}

fn register_builtin_openai_provider(
    host: &mut ExtensionHost,
    extension_id: &str,
    adapter_kind: &str,
) {
    host.register_builtin_provider(BuiltinProviderExtensionFactory::new(
        openai_protocol_manifest(extension_id),
        adapter_kind,
        |base_url| Box::new(OpenAiProviderAdapter::new(base_url)),
    ));
}

fn register_builtin_openai_compatible_provider(
    host: &mut ExtensionHost,
    extension_id: &str,
    adapter_kind: &str,
) {
    host.register_builtin_provider(BuiltinProviderExtensionFactory::new(
        openai_protocol_manifest(extension_id),
        adapter_kind,
        |base_url| Box::new(OpenAiProviderAdapter::new(base_url)),
    ));
}

fn register_builtin_openrouter_provider(host: &mut ExtensionHost) {
    host.register_builtin_provider(BuiltinProviderExtensionFactory::new(
        rich_builtin_manifest("sdkwork.provider.openrouter", ExtensionProtocol::OpenAi),
        "openrouter",
        |base_url| Box::new(OpenRouterProviderAdapter::new(base_url)),
    ));
}

fn register_builtin_ollama_provider(host: &mut ExtensionHost) {
    host.register_builtin_provider(BuiltinProviderExtensionFactory::new(
        rich_builtin_manifest("sdkwork.provider.ollama", ExtensionProtocol::Custom),
        "ollama",
        |base_url| Box::new(OllamaProviderAdapter::new(base_url)),
    ));
}

fn openai_protocol_manifest(extension_id: &str) -> ExtensionManifest {
    rich_builtin_manifest(extension_id, ExtensionProtocol::OpenAi)
}

fn rich_builtin_manifest(extension_id: &str, protocol: ExtensionProtocol) -> ExtensionManifest {
    ExtensionManifest::new(
        extension_id,
        ExtensionKind::Provider,
        "0.1.0",
        ExtensionRuntime::Builtin,
    )
    .with_protocol(protocol)
    .with_supported_modality(ExtensionModality::Image)
    .with_supported_modality(ExtensionModality::Audio)
    .with_supported_modality(ExtensionModality::Video)
    .with_supported_modality(ExtensionModality::File)
    .with_supported_modality(ExtensionModality::Embedding)
}

pub(crate) fn provider_runtime_key(provider: &ProxyProvider) -> &str {
    &provider.extension_id
}

pub(crate) fn preferred_provider_runtime_key(
    host: &ExtensionHost,
    provider: &ProxyProvider,
) -> String {
    for runtime_key in [
        provider.extension_id.as_str(),
        provider.protocol_kind(),
        provider.adapter_kind.as_str(),
    ] {
        if !runtime_key.trim().is_empty() && host.can_resolve_provider(runtime_key) {
            return runtime_key.to_owned();
        }
    }

    provider_runtime_key(provider).to_owned()
}

pub(crate) fn configured_extension_host() -> Result<ExtensionHost> {
    let policy = configured_extension_discovery_policy();
    let cache_key = ConfiguredExtensionHostCacheKey::from(&policy);
    let cache = CONFIGURED_EXTENSION_HOST_CACHE.get_or_init(|| Mutex::new(None));
    let mut cache_guard = match cache.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };

    if let Some(cached) = cache_guard.as_ref() {
        if cached.key == cache_key {
            return Ok(cached.host.clone());
        }
    }

    let built = build_configured_extension_host(&policy)?;
    *cache_guard = Some(CachedConfiguredExtensionHost {
        key: cache_key,
        host: built.host.clone(),
    });
    Ok(built.host)
}

pub fn reload_configured_extension_host() -> Result<ConfiguredExtensionHostReloadReport> {
    reload_extension_host_with_scope(&ConfiguredExtensionHostReloadScope::All)
}

pub fn reload_extension_host_with_scope(
    scope: &ConfiguredExtensionHostReloadScope,
) -> Result<ConfiguredExtensionHostReloadReport> {
    let policy = configured_extension_discovery_policy();
    reload_extension_host_with_policy_and_scope(&policy, scope)
}

pub fn reload_extension_host_with_policy(
    policy: &ExtensionDiscoveryPolicy,
) -> Result<ConfiguredExtensionHostReloadReport> {
    reload_extension_host_with_policy_and_scope(policy, &ConfiguredExtensionHostReloadScope::All)
}

fn reload_extension_host_with_policy_and_scope(
    policy: &ExtensionDiscoveryPolicy,
    scope: &ConfiguredExtensionHostReloadScope,
) -> Result<ConfiguredExtensionHostReloadReport> {
    let discovered_packages = discover_extension_packages(policy)?;
    let cache_key = ConfiguredExtensionHostCacheKey::from(policy);

    apply_configured_extension_host_reload_scope(scope)?;
    let built = build_configured_extension_host_from_packages(discovered_packages, policy);
    let cache = CONFIGURED_EXTENSION_HOST_CACHE.get_or_init(|| Mutex::new(None));
    let mut cache_guard = match cache.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    *cache_guard = Some(CachedConfiguredExtensionHost {
        key: cache_key,
        host: built.host.clone(),
    });

    Ok(ConfiguredExtensionHostReloadReport {
        discovered_package_count: built.discovered_package_count,
        loadable_package_count: built.loadable_package_count,
    })
}

fn apply_configured_extension_host_reload_scope(
    scope: &ConfiguredExtensionHostReloadScope,
) -> Result<()> {
    match scope {
        ConfiguredExtensionHostReloadScope::All => {
            shutdown_all_connector_runtimes()?;
            shutdown_all_native_dynamic_runtimes()?;
        }
        ConfiguredExtensionHostReloadScope::Extension { extension_id } => {
            shutdown_connector_runtimes_for_extension(extension_id)?;
            shutdown_native_dynamic_runtimes_for_extension(extension_id)?;
        }
        ConfiguredExtensionHostReloadScope::Instance { instance_id } => {
            shutdown_connector_runtime(instance_id)?;
        }
    }

    Ok(())
}

pub fn start_configured_extension_hot_reload_supervision(
    interval_secs: u64,
) -> Option<JoinHandle<()>> {
    if interval_secs == 0 {
        return None;
    }

    let initial_state = match configured_extension_host_watch_state() {
        Ok(state) => Some(state),
        Err(error) => {
            eprintln!("extension hot reload watch startup state capture failed: {error}");
            None
        }
    };

    Some(tokio::spawn(async move {
        let mut previous_state = initial_state;

        let mut interval = tokio::time::interval(Duration::from_secs(interval_secs));
        interval.set_missed_tick_behavior(MissedTickBehavior::Delay);
        interval.tick().await;

        loop {
            interval.tick().await;

            let next_state = match configured_extension_host_watch_state() {
                Ok(state) => state,
                Err(error) => {
                    eprintln!("extension hot reload watch state capture failed: {error}");
                    continue;
                }
            };

            if previous_state.as_ref() == Some(&next_state) {
                continue;
            }

            match reload_configured_extension_host() {
                Ok(report) => {
                    eprintln!(
                        "extension hot reload applied: discovered_package_count={} loadable_package_count={}",
                        report.discovered_package_count, report.loadable_package_count
                    );
                    previous_state = Some(next_state);
                }
                Err(error) => {
                    eprintln!("extension hot reload failed: {error}");
                }
            }
        }
    }))
}

fn build_configured_extension_host(
    policy: &ExtensionDiscoveryPolicy,
) -> Result<BuiltConfiguredExtensionHost> {
    let packages = discover_extension_packages(policy)?;
    Ok(build_configured_extension_host_from_packages(
        packages, policy,
    ))
}

fn build_configured_extension_host_from_packages(
    packages: Vec<DiscoveredExtensionPackage>,
    policy: &ExtensionDiscoveryPolicy,
) -> BuiltConfiguredExtensionHost {
    let mut host = builtin_extension_host();
    let discovered_package_count = packages.len();
    let mut loadable_package_count = 0;

    for package in packages {
        let trust = verify_discovered_extension_package_trust(&package, policy);
        if !trust.load_allowed {
            continue;
        }
        if register_discovered_extension(&mut host, package) {
            loadable_package_count += 1;
        }
    }

    BuiltConfiguredExtensionHost {
        host,
        discovered_package_count,
        loadable_package_count,
    }
}

fn configured_extension_host_watch_state() -> Result<ConfiguredExtensionHostWatchState> {
    let policy = configured_extension_discovery_policy();
    Ok(ConfiguredExtensionHostWatchState {
        key: ConfiguredExtensionHostCacheKey::from(&policy),
        fingerprint: extension_tree_fingerprint(&policy.search_paths)?,
    })
}

fn extension_tree_fingerprint(search_paths: &[PathBuf]) -> Result<Vec<String>> {
    let mut fingerprint = Vec::new();
    for path in search_paths {
        collect_extension_tree_fingerprint(path, &mut fingerprint)?;
    }
    fingerprint.sort();
    Ok(fingerprint)
}

fn collect_extension_tree_fingerprint(path: &Path, fingerprint: &mut Vec<String>) -> Result<()> {
    match fs::metadata(path) {
        Ok(metadata) => {
            fingerprint.push(fingerprint_entry(path, &metadata));
            if metadata.is_dir() {
                let mut children = fs::read_dir(path)
                    .with_context(|| {
                        format!("failed to read extension directory {}", path.display())
                    })?
                    .collect::<std::result::Result<Vec<_>, _>>()
                    .with_context(|| {
                        format!("failed to enumerate extension directory {}", path.display())
                    })?;
                children.sort_by_key(|entry| entry.path());
                for child in children {
                    collect_extension_tree_fingerprint(&child.path(), fingerprint)?;
                }
            }
            Ok(())
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            fingerprint.push(format!("missing|{}", path.display()));
            Ok(())
        }
        Err(error) => {
            Err(error).with_context(|| format!("failed to stat extension path {}", path.display()))
        }
    }
}

fn fingerprint_entry(path: &Path, metadata: &fs::Metadata) -> String {
    let kind = if metadata.is_dir() { "dir" } else { "file" };
    format!(
        "{kind}|{}|{}|{}",
        path.display(),
        metadata.len(),
        metadata_modified_ms(metadata),
    )
}

fn metadata_modified_ms(metadata: &fs::Metadata) -> u64 {
    metadata
        .modified()
        .ok()
        .and_then(|modified| modified.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}

pub(crate) fn configured_extension_discovery_policy() -> ExtensionDiscoveryPolicy {
    let search_paths = std::env::var_os("SDKWORK_EXTENSION_PATHS")
        .map(|value| std::env::split_paths(&value).collect::<Vec<_>>())
        .unwrap_or_default();

    let mut policy = ExtensionDiscoveryPolicy::new(search_paths)
        .with_connector_extensions(env_flag(
            "SDKWORK_EXTENSION_ENABLE_CONNECTOR_EXTENSIONS",
            true,
        ))
        .with_native_dynamic_extensions(env_flag(
            "SDKWORK_EXTENSION_ENABLE_NATIVE_DYNAMIC_EXTENSIONS",
            false,
        ))
        .with_required_signatures_for_connector_extensions(env_flag(
            "SDKWORK_EXTENSION_REQUIRE_SIGNATURE_FOR_CONNECTOR_EXTENSIONS",
            false,
        ))
        .with_required_signatures_for_native_dynamic_extensions(env_flag(
            "SDKWORK_EXTENSION_REQUIRE_SIGNATURE_FOR_NATIVE_DYNAMIC_EXTENSIONS",
            true,
        ));
    for (publisher, public_key) in env_trusted_signers("SDKWORK_EXTENSION_TRUSTED_SIGNERS") {
        policy = policy.with_trusted_signer(publisher, public_key);
    }
    policy
}

fn env_flag(key: &str, default: bool) -> bool {
    std::env::var(key)
        .ok()
        .and_then(|value| value.parse::<bool>().ok())
        .unwrap_or(default)
}

fn env_trusted_signers(key: &str) -> Vec<(String, String)> {
    std::env::var(key)
        .ok()
        .map(|value| parse_trusted_signers(&value))
        .unwrap_or_default()
}

fn parse_trusted_signers(value: &str) -> Vec<(String, String)> {
    value
        .split(';')
        .filter_map(|entry| {
            let entry = entry.trim();
            if entry.is_empty() {
                return None;
            }
            let (publisher, public_key) = entry.split_once('=')?;
            let publisher = publisher.trim();
            let public_key = public_key.trim();
            if publisher.is_empty() || public_key.is_empty() {
                return None;
            }
            Some((publisher.to_owned(), public_key.to_owned()))
        })
        .collect()
}

fn register_discovered_extension(
    host: &mut ExtensionHost,
    package: DiscoveredExtensionPackage,
) -> bool {
    if host.manifest(&package.manifest.id).is_some() {
        return false;
    }

    if package.manifest.runtime.supports_raw_provider_execution() {
        return host
            .register_discovered_native_dynamic_provider(package)
            .is_ok();
    }

    match (package.manifest.kind.clone(), package.manifest.protocol) {
        (ExtensionKind::Provider, Some(ExtensionProtocol::OpenAi)) => {
            host.register_discovered_provider(package, "openai", |base_url| {
                Box::new(OpenAiProviderAdapter::new(base_url))
            });
        }
        (ExtensionKind::Provider, Some(ExtensionProtocol::OpenRouter)) => {
            host.register_discovered_provider(package, "openrouter", |base_url| {
                Box::new(OpenRouterProviderAdapter::new(base_url))
            });
        }
        (ExtensionKind::Provider, Some(ExtensionProtocol::Ollama)) => {
            host.register_discovered_provider(package, "ollama", |base_url| {
                Box::new(OllamaProviderAdapter::new(base_url))
            });
        }
        _ => host.register_discovered_manifest(package),
    }

    true
}
