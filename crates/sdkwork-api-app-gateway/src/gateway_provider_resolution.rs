use super::*;

#[derive(Clone)]
pub(crate) struct ProviderExecutionTarget {
    pub(crate) provider_id: String,
    pub(crate) runtime_key: String,
    pub(crate) base_url: String,
    pub(crate) runtime: ExtensionRuntime,
    pub(crate) local_fallback: bool,
}

impl ProviderExecutionTarget {
    fn local() -> Self {
        Self {
            provider_id: LOCAL_PROVIDER_ID.to_owned(),
            runtime_key: String::new(),
            base_url: String::new(),
            runtime: ExtensionRuntime::Builtin,
            local_fallback: true,
        }
    }

    fn upstream(
        provider_id: String,
        runtime_key: String,
        base_url: String,
        runtime: ExtensionRuntime,
    ) -> Self {
        Self {
            provider_id,
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
    pub(crate) runtime_key: String,
    pub(crate) base_url: String,
    pub(crate) api_key: String,
    pub(crate) runtime: ExtensionRuntime,
    pub(crate) local_fallback: bool,
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
                ensure_connector_runtime_started(&load_plan, &resolved_base_url)
                    .map_err(anyhow::Error::new)?;
            }

            Ok(ProviderExecutionTarget::upstream(
                provider.id.clone(),
                load_plan.extension_id,
                resolved_base_url,
                load_plan.runtime,
            ))
        }
        Err(sdkwork_api_extension_host::ExtensionHostError::InstanceNotFound { .. }) => {
            Ok(ProviderExecutionTarget::upstream(
                provider.id.clone(),
                provider_runtime_key(provider).to_owned(),
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
        runtime_key: target.runtime_key,
        base_url: target.base_url,
        api_key,
        runtime: target.runtime,
        local_fallback: target.local_fallback,
    })
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
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    let descriptor = provider_execution_descriptor_for_provider(store, &provider, api_key).await?;
    if descriptor.local_fallback {
        return Ok(None);
    }

    Ok(Some((
        descriptor.runtime_key,
        descriptor.base_url,
        descriptor.api_key,
    )))
}
