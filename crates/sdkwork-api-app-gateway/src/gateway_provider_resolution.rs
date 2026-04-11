use super::*;
use sdkwork_api_domain_catalog::ProviderAccountRecord;
use sdkwork_api_extension_host::ExtensionLoadPlan;
use utoipa::ToSchema;

#[derive(Clone)]
pub(crate) struct ProviderExecutionTarget {
    pub(crate) provider_id: String,
    pub(crate) provider_account_id: Option<String>,
    pub(crate) execution_instance_id: Option<String>,
    pub(crate) runtime_key: String,
    pub(crate) base_url: String,
    pub(crate) runtime: ExtensionRuntime,
    pub(crate) local_fallback: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
pub struct ProviderExecutionView {
    pub binding_kind: String,
    pub runtime: String,
    pub runtime_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub passthrough_protocol: Option<String>,
    pub supports_provider_adapter: bool,
    pub supports_raw_plugin: bool,
    pub fail_closed: bool,
    pub route_readiness: ProviderRouteReadinessView,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
pub struct ProviderRouteReadinessView {
    pub openai: ProviderRouteExecutionView,
    pub anthropic: ProviderRouteExecutionView,
    pub gemini: ProviderRouteExecutionView,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
pub struct ProviderRouteExecutionView {
    pub ready: bool,
    pub mode: ProviderRouteExecutionMode,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ProviderRouteExecutionMode {
    ProviderAdapter,
    StandardPassthrough,
    RawPlugin,
    FailClosed,
    Unavailable,
}

impl ProviderExecutionTarget {
    fn local() -> Self {
        Self {
            provider_id: LOCAL_PROVIDER_ID.to_owned(),
            provider_account_id: None,
            execution_instance_id: None,
            runtime_key: String::new(),
            base_url: String::new(),
            runtime: ExtensionRuntime::Builtin,
            local_fallback: true,
        }
    }

    fn upstream(
        provider_id: String,
        provider_account_id: Option<String>,
        execution_instance_id: Option<String>,
        runtime_key: String,
        base_url: String,
        runtime: ExtensionRuntime,
    ) -> Self {
        Self {
            provider_id,
            provider_account_id,
            execution_instance_id,
            runtime_key,
            base_url,
            runtime,
            local_fallback: false,
        }
    }
}

#[derive(Clone)]
pub(crate) struct ProviderExecutionDescriptor {
    pub(crate) provider_id: String,
    pub(crate) provider_account_id: Option<String>,
    pub(crate) execution_instance_id: Option<String>,
    pub(crate) runtime_key: String,
    pub(crate) base_url: String,
    pub(crate) api_key: String,
    pub(crate) runtime: ExtensionRuntime,
    pub(crate) local_fallback: bool,
}

#[derive(Clone)]
struct ProviderAccountExecutionBinding {
    account: ProviderAccountRecord,
    load_plan: ExtensionLoadPlan,
}

pub(crate) async fn build_extension_host_from_store(
    store: &dyn AdminStore,
) -> Result<ExtensionHost> {
    let mut host = configured_extension_host()?;

    let mut installations = store.list_extension_installations().await?;
    installations.sort_by(|left, right| left.installation_id.cmp(&right.installation_id));
    for installation in installations {
        match host.install(installation) {
            Ok(()) => {}
            Err(sdkwork_api_extension_host::ExtensionHostError::ManifestNotFound { .. }) => {}
            Err(error) => return Err(error.into()),
        }
    }

    let mut instances = store.list_extension_instances().await?;
    instances.sort_by(|left, right| left.instance_id.cmp(&right.instance_id));
    for instance in instances {
        match host.mount_instance(instance) {
            Ok(()) => {}
            Err(sdkwork_api_extension_host::ExtensionHostError::InstallationNotFound {
                ..
            }) => {}
            Err(error) => return Err(error.into()),
        }
    }

    Ok(host)
}

pub(crate) async fn provider_execution_target_for_provider(
    store: &dyn AdminStore,
    provider: &ProxyProvider,
) -> Result<ProviderExecutionTarget> {
    let host = build_extension_host_from_store(store).await?;
    provider_execution_target_for_provider_with_host(store, provider, &host).await
}

async fn provider_execution_target_for_provider_with_host(
    store: &dyn AdminStore,
    provider: &ProxyProvider,
    host: &ExtensionHost,
) -> Result<ProviderExecutionTarget> {
    let host = host.clone();

    match host.load_plan(&provider.id) {
        Ok(load_plan) => {
            if !load_plan.enabled {
                return Ok(ProviderExecutionTarget::local());
            }

            let resolved_base_url = load_plan
                .base_url
                .clone()
                .unwrap_or_else(|| provider.base_url.clone());
            if load_plan.runtime == ExtensionRuntime::Connector {
                let connector_load_plan = load_plan.clone();
                let connector_base_url = resolved_base_url.clone();
                tokio::task::spawn_blocking(move || {
                    ensure_connector_runtime_started(&connector_load_plan, &connector_base_url)
                })
                .await
                .map_err(anyhow::Error::from)?
                .map_err(anyhow::Error::new)?;
            }

            Ok(ProviderExecutionTarget::upstream(
                provider.id.clone(),
                None,
                None,
                load_plan.extension_id,
                resolved_base_url,
                load_plan.runtime,
            ))
        }
        Err(sdkwork_api_extension_host::ExtensionHostError::InstanceNotFound { .. }) => {
            if provider_has_persisted_runtime_binding(store, provider).await? {
                return Ok(ProviderExecutionTarget::local());
            }
            Ok(ProviderExecutionTarget::upstream(
                provider.id.clone(),
                None,
                None,
                preferred_provider_runtime_key(&host, provider),
                provider.base_url.clone(),
                ExtensionRuntime::Builtin,
            ))
        }
        Err(error) => Err(error.into()),
    }
}

pub(crate) async fn provider_execution_descriptor_for_provider(
    store: &dyn AdminStore,
    provider: &ProxyProvider,
    api_key: String,
) -> Result<ProviderExecutionDescriptor> {
    let target = provider_execution_target_for_provider(store, provider).await?;
    Ok(ProviderExecutionDescriptor {
        provider_id: target.provider_id,
        provider_account_id: target.provider_account_id,
        execution_instance_id: target.execution_instance_id,
        runtime_key: target.runtime_key,
        base_url: target.base_url,
        api_key,
        runtime: target.runtime,
        local_fallback: target.local_fallback,
    })
}

pub(crate) async fn provider_execution_descriptor_for_provider_account_context(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    provider: &ProxyProvider,
    requested_region: Option<&str>,
) -> Result<Option<ProviderExecutionDescriptor>> {
    let host = build_extension_host_from_store(store).await?;
    if let Some(binding) = select_provider_account_execution_binding(
        store,
        provider,
        Some(tenant_id),
        requested_region,
        &host,
    )
    .await?
    {
        let Some(api_key) = resolve_provider_account_secret(
            store,
            secret_manager,
            tenant_id,
            provider,
            &binding.load_plan,
        )
        .await?
        else {
            return Ok(None);
        };
        let target = provider_execution_target_for_account_binding(provider, &binding)?;
        return Ok(Some(ProviderExecutionDescriptor {
            provider_id: target.provider_id,
            provider_account_id: target.provider_account_id,
            execution_instance_id: target.execution_instance_id,
            runtime_key: target.runtime_key,
            base_url: target.base_url,
            api_key,
            runtime: target.runtime,
            local_fallback: target.local_fallback,
        }));
    }

    let Some(api_key) = resolve_provider_secret_with_fallback_and_manager(
        store,
        secret_manager,
        tenant_id,
        &provider.id,
    )
    .await?
    else {
        return Ok(None);
    };
    let target = provider_execution_target_for_provider_with_host(store, provider, &host).await?;
    Ok(Some(ProviderExecutionDescriptor {
        provider_id: target.provider_id,
        provider_account_id: target.provider_account_id,
        execution_instance_id: target.execution_instance_id,
        runtime_key: target.runtime_key,
        base_url: target.base_url,
        api_key,
        runtime: target.runtime,
        local_fallback: target.local_fallback,
    }))
}

pub async fn inspect_provider_execution_views(
    store: &dyn AdminStore,
    providers: &[ProxyProvider],
) -> Result<HashMap<String, ProviderExecutionView>> {
    let host = build_extension_host_from_store(store).await?;
    let installations = store
        .list_extension_installations()
        .await?
        .into_iter()
        .map(|installation| (installation.installation_id.clone(), installation))
        .collect::<HashMap<_, _>>();
    let instances = store
        .list_extension_instances()
        .await?
        .into_iter()
        .collect::<Vec<_>>();

    let mut views = HashMap::with_capacity(providers.len());
    for provider in providers {
        let exact_instance = instances
            .iter()
            .find(|instance| instance.instance_id == provider.id);
        let matching_installation = exact_instance
            .and_then(|instance| installations.get(&instance.installation_id))
            .or_else(|| {
                instances
                    .iter()
                    .find(|instance| instance.extension_id == provider.extension_id)
                    .and_then(|instance| installations.get(&instance.installation_id))
            })
            .or_else(|| {
                installations
                    .values()
                    .find(|installation| installation.extension_id == provider.extension_id)
            });

        let binding_kind = if exact_instance.is_some() {
            "explicit_instance"
        } else if instances
            .iter()
            .any(|instance| instance.extension_id == provider.extension_id)
            || installations
                .values()
                .any(|installation| installation.extension_id == provider.extension_id)
        {
            "explicit_extension"
        } else {
            "implicit_default"
        };

        let passthrough_protocol =
            standard_passthrough_protocol(provider.protocol_kind()).map(ToOwned::to_owned);

        let view = match host.load_plan(&provider.id) {
            Ok(load_plan) if !load_plan.enabled => {
                let runtime_key = load_plan.extension_id.clone();
                let route_readiness =
                    provider_route_readiness(&host, provider, &runtime_key, false, true);
                ProviderExecutionView {
                    binding_kind: binding_kind.to_owned(),
                    runtime: load_plan.runtime.as_str().to_owned(),
                    runtime_key,
                    passthrough_protocol,
                    supports_provider_adapter: false,
                    supports_raw_plugin: false,
                    fail_closed: true,
                    route_readiness,
                    reason: Some(
                        "explicit runtime binding is disabled, so gateway execution would fail closed instead of silently downgrading"
                            .to_owned(),
                    ),
                }
            }
            Ok(load_plan) => {
                let runtime_key = load_plan.extension_id.clone();
                let supports_provider_adapter = host.can_resolve_provider(&runtime_key);
                let supports_raw_plugin = load_plan.runtime.supports_raw_provider_execution();
                let fail_closed = !supports_provider_adapter && !supports_raw_plugin;
                let route_readiness = provider_route_readiness(
                    &host,
                    provider,
                    &runtime_key,
                    supports_provider_adapter,
                    fail_closed,
                );
                let reason = provider_execution_reason(
                    binding_kind,
                    fail_closed,
                    supports_provider_adapter,
                    supports_raw_plugin,
                    passthrough_protocol.as_deref(),
                );
                ProviderExecutionView {
                    binding_kind: binding_kind.to_owned(),
                    runtime: load_plan.runtime.as_str().to_owned(),
                    runtime_key,
                    passthrough_protocol: passthrough_protocol.clone(),
                    supports_provider_adapter,
                    supports_raw_plugin,
                    fail_closed,
                    route_readiness,
                    reason,
                }
            }
            Err(_) if binding_kind != "implicit_default" => ProviderExecutionView {
                binding_kind: binding_kind.to_owned(),
                runtime: inferred_runtime_kind(matching_installation, &host, &provider.extension_id),
                runtime_key: provider.extension_id.clone(),
                passthrough_protocol,
                supports_provider_adapter: false,
                supports_raw_plugin: false,
                fail_closed: true,
                route_readiness: provider_route_readiness(
                    &host,
                    provider,
                    &provider.extension_id,
                    false,
                    true,
                ),
                reason: Some(
                    "explicit runtime binding is not currently loadable, so gateway execution would fail closed instead of silently downgrading"
                        .to_owned(),
                ),
            },
            Err(_) => {
                let runtime_key = preferred_provider_runtime_key(&host, provider);
                let supports_provider_adapter = host.can_resolve_provider(&runtime_key);
                let route_readiness = provider_route_readiness(
                    &host,
                    provider,
                    &runtime_key,
                    supports_provider_adapter,
                    false,
                );
                let reason = provider_execution_reason(
                    binding_kind,
                    false,
                    supports_provider_adapter,
                    false,
                    passthrough_protocol.as_deref(),
                );
                ProviderExecutionView {
                    binding_kind: binding_kind.to_owned(),
                    runtime: inferred_runtime_kind(None, &host, &runtime_key),
                    runtime_key,
                    passthrough_protocol: passthrough_protocol.clone(),
                    supports_provider_adapter,
                    supports_raw_plugin: false,
                    fail_closed: false,
                    route_readiness,
                    reason,
                }
            }
        };

        views.insert(provider.id.clone(), view);
    }

    Ok(views)
}

async fn provider_has_persisted_runtime_binding(
    store: &dyn AdminStore,
    provider: &ProxyProvider,
) -> Result<bool> {
    if store
        .list_extension_installations()
        .await?
        .into_iter()
        .any(|installation| installation.extension_id == provider.extension_id)
    {
        return Ok(true);
    }

    Ok(store
        .list_extension_instances()
        .await?
        .into_iter()
        .any(|instance| {
            instance.instance_id == provider.id || instance.extension_id == provider.extension_id
        }))
}

pub(crate) async fn resolve_non_model_provider(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    project_id: &str,
    capability: &str,
    route_key: &str,
) -> Result<Option<(String, String, String)>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(project_id), capability, route_key).await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(descriptor) = provider_execution_descriptor_for_provider_account_context(
        store,
        secret_manager,
        tenant_id,
        &provider,
        decision.requested_region.as_deref(),
    )
    .await?
    else {
        return Ok(None);
    };
    if descriptor.local_fallback {
        return Ok(None);
    }

    Ok(Some((
        descriptor.runtime_key,
        descriptor.base_url,
        descriptor.api_key,
    )))
}

async fn select_provider_account_execution_binding(
    store: &dyn AdminStore,
    provider: &ProxyProvider,
    tenant_id: Option<&str>,
    requested_region: Option<&str>,
    host: &ExtensionHost,
) -> Result<Option<ProviderAccountExecutionBinding>> {
    let mut bindings = Vec::new();

    for account in store.list_provider_accounts_for_provider(&provider.id).await? {
        if !account.enabled || !provider_account_matches_owner_scope(&account, tenant_id) {
            continue;
        }

        let Ok(load_plan) = host.load_plan(&account.execution_instance_id) else {
            continue;
        };
        if !load_plan.enabled {
            continue;
        }

        bindings.push(ProviderAccountExecutionBinding { account, load_plan });
    }

    bindings.sort_by(|left, right| {
        provider_account_region_preference(&right.account, requested_region)
            .cmp(&provider_account_region_preference(&left.account, requested_region))
            .then_with(|| right.account.priority.cmp(&left.account.priority))
            .then_with(|| right.account.weight.cmp(&left.account.weight))
            .then_with(|| left.account.provider_account_id.cmp(&right.account.provider_account_id))
    });

    Ok(bindings.into_iter().next())
}

fn provider_account_matches_owner_scope(
    account: &ProviderAccountRecord,
    tenant_id: Option<&str>,
) -> bool {
    if account.owner_scope == "tenant" {
        return tenant_id.is_some() && account.owner_tenant_id.as_deref() == tenant_id;
    }

    true
}

fn provider_account_region_preference(
    account: &ProviderAccountRecord,
    requested_region: Option<&str>,
) -> u8 {
    match (
        account.region.as_deref().map(str::trim).filter(|value| !value.is_empty()),
        requested_region.map(str::trim).filter(|value| !value.is_empty()),
    ) {
        (Some(account_region), Some(requested_region))
            if account_region.eq_ignore_ascii_case(requested_region) =>
        {
            1
        }
        _ => 0,
    }
}

fn provider_execution_target_for_account_binding(
    provider: &ProxyProvider,
    binding: &ProviderAccountExecutionBinding,
) -> Result<ProviderExecutionTarget> {
    let resolved_base_url = binding
        .account
        .base_url_override
        .clone()
        .or_else(|| binding.load_plan.base_url.clone())
        .unwrap_or_else(|| provider.base_url.clone());
    ensure_runtime_started_for_load_plan(&binding.load_plan, &resolved_base_url)?;

    Ok(ProviderExecutionTarget::upstream(
        provider.id.clone(),
        Some(binding.account.provider_account_id.clone()),
        Some(binding.account.execution_instance_id.clone()),
        binding.load_plan.extension_id.clone(),
        resolved_base_url,
        binding.load_plan.runtime.clone(),
    ))
}

fn ensure_runtime_started_for_load_plan(
    load_plan: &ExtensionLoadPlan,
    resolved_base_url: &str,
) -> Result<()> {
    if load_plan.runtime == ExtensionRuntime::Connector {
        ensure_connector_runtime_started(load_plan, resolved_base_url)
            .map_err(anyhow::Error::new)?;
    }

    Ok(())
}

async fn resolve_provider_account_secret(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    provider: &ProxyProvider,
    load_plan: &ExtensionLoadPlan,
) -> Result<Option<String>> {
    let credential_ref = load_plan
        .credential_ref
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let Some(credential_ref) = credential_ref else {
        return resolve_provider_secret_with_fallback_and_manager(
            store,
            secret_manager,
            tenant_id,
            &provider.id,
        )
        .await;
    };

    if store
        .find_credential(tenant_id, &provider.id, credential_ref)
        .await?
        .is_some()
    {
        return resolve_credential_secret_with_manager(
            store,
            secret_manager,
            tenant_id,
            &provider.id,
            credential_ref,
        )
        .await
        .map(Some);
    }

    resolve_official_provider_secret_with_manager(store, secret_manager, &provider.id).await
}

fn inferred_runtime_kind(
    installation: Option<&sdkwork_api_extension_core::ExtensionInstallation>,
    host: &ExtensionHost,
    runtime_key: &str,
) -> String {
    installation
        .map(|installation| installation.runtime.as_str().to_owned())
        .or_else(|| {
            host.manifest(runtime_key)
                .map(|manifest| manifest.runtime.as_str().to_owned())
        })
        .unwrap_or_else(|| ExtensionRuntime::Builtin.as_str().to_owned())
}

fn standard_passthrough_protocol(protocol_kind: &str) -> Option<&str> {
    match protocol_kind {
        "openai" | "anthropic" | "gemini" => Some(protocol_kind),
        _ => None,
    }
}

fn provider_route_readiness(
    host: &ExtensionHost,
    provider: &ProxyProvider,
    runtime_key: &str,
    supports_provider_adapter: bool,
    fail_closed: bool,
) -> ProviderRouteReadinessView {
    let anthropic_raw_plugin =
        runtime_supports_raw_operations(host, runtime_key, ANTHROPIC_ROUTE_OPERATIONS);
    let gemini_raw_plugin =
        runtime_supports_raw_operations(host, runtime_key, GEMINI_ROUTE_OPERATIONS);

    ProviderRouteReadinessView {
        openai: openai_route_readiness(supports_provider_adapter, fail_closed),
        anthropic: standard_family_route_readiness(
            provider.protocol_kind(),
            "anthropic",
            supports_provider_adapter,
            anthropic_raw_plugin,
            fail_closed,
        ),
        gemini: standard_family_route_readiness(
            provider.protocol_kind(),
            "gemini",
            supports_provider_adapter,
            gemini_raw_plugin,
            fail_closed,
        ),
    }
}

fn openai_route_readiness(
    supports_provider_adapter: bool,
    fail_closed: bool,
) -> ProviderRouteExecutionView {
    if supports_provider_adapter {
        return ProviderRouteExecutionView {
            ready: true,
            mode: ProviderRouteExecutionMode::ProviderAdapter,
        };
    }

    terminal_route_readiness(fail_closed)
}

fn standard_family_route_readiness(
    protocol_kind: &str,
    family: &str,
    supports_provider_adapter: bool,
    supports_raw_plugin: bool,
    fail_closed: bool,
) -> ProviderRouteExecutionView {
    if protocol_kind == family {
        return ProviderRouteExecutionView {
            ready: true,
            mode: ProviderRouteExecutionMode::StandardPassthrough,
        };
    }

    if supports_raw_plugin {
        return ProviderRouteExecutionView {
            ready: true,
            mode: ProviderRouteExecutionMode::RawPlugin,
        };
    }

    if supports_provider_adapter {
        return ProviderRouteExecutionView {
            ready: true,
            mode: ProviderRouteExecutionMode::ProviderAdapter,
        };
    }

    terminal_route_readiness(fail_closed)
}

fn terminal_route_readiness(fail_closed: bool) -> ProviderRouteExecutionView {
    if fail_closed {
        ProviderRouteExecutionView {
            ready: false,
            mode: ProviderRouteExecutionMode::FailClosed,
        }
    } else {
        ProviderRouteExecutionView {
            ready: false,
            mode: ProviderRouteExecutionMode::Unavailable,
        }
    }
}

fn runtime_supports_raw_operations(
    host: &ExtensionHost,
    runtime_key: &str,
    operations: &[&str],
) -> bool {
    let Some(manifest) = host.manifest(runtime_key) else {
        return false;
    };

    manifest.runtime.supports_raw_provider_execution()
        && operations
            .iter()
            .all(|operation| manifest_supports_operation(manifest, operation))
}

fn manifest_supports_operation(manifest: &ExtensionManifest, operation: &str) -> bool {
    manifest.capabilities.iter().any(|capability| {
        capability.operation == operation
            && capability.compatibility != sdkwork_api_extension_core::CompatibilityLevel::Unsupported
    })
}

fn provider_execution_reason(
    binding_kind: &str,
    fail_closed: bool,
    supports_provider_adapter: bool,
    supports_raw_plugin: bool,
    passthrough_protocol: Option<&str>,
) -> Option<String> {
    if fail_closed {
        return Some(
            "provider has no currently executable adapter/plugin surface and would fail closed on non-passthrough routes"
                .to_owned(),
        );
    }

    if supports_raw_plugin {
        return Some("provider is executable on the raw plugin surface".to_owned());
    }

    if supports_provider_adapter {
        return Some("provider is executable on the provider-adapter surface".to_owned());
    }

    if let Some(protocol) = passthrough_protocol {
        return Some(format!(
            "provider is executable through {protocol} standard-protocol passthrough"
        ));
    }

    if binding_kind == "implicit_default" {
        return Some(
            "no default executable adapter/plugin surface is resolved; provider needs an explicit plugin/runtime binding"
                .to_owned(),
        );
    }

    None
}

const ANTHROPIC_ROUTE_OPERATIONS: &[&str] = &[
    "anthropic.messages.create",
    "anthropic.messages.count_tokens",
];

const GEMINI_ROUTE_OPERATIONS: &[&str] = &[
    "gemini.generate_content",
    "gemini.stream_generate_content",
    "gemini.count_tokens",
];
