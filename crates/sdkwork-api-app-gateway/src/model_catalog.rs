use super::*;

pub fn service_name() -> &'static str {
    "gateway-service"
}

pub fn list_models(_tenant_id: &str, _project_id: &str) -> Result<ListModelsResponse> {
    Ok(ListModelsResponse::new(vec![ModelObject::new(
        "gpt-4.1", "sdkwork",
    )]))
}

fn ensure_local_model_exists(model_id: &str) -> Result<()> {
    if model_id != "gpt-4.1" {
        bail!("model not found");
    }

    Ok(())
}

fn ensure_local_deletable_model_exists(model_id: &str) -> Result<()> {
    if model_id != "ft:gpt-4.1:sdkwork" {
        bail!("model not found");
    }

    Ok(())
}

pub fn get_model(_tenant_id: &str, _project_id: &str, model_id: &str) -> Result<ModelObject> {
    ensure_local_model_exists(model_id)?;
    Ok(ModelObject::new(model_id, "sdkwork"))
}

pub fn delete_model(
    _tenant_id: &str,
    _project_id: &str,
    model_id: &str,
) -> Result<DeleteModelResponse> {
    ensure_local_deletable_model_exists(model_id)?;
    Ok(DeleteModelResponse::deleted(model_id))
}

pub async fn list_models_from_store(
    store: &dyn AdminStore,
    tenant_id: &str,
    project_id: &str,
) -> Result<ListModelsResponse> {
    let Some(cache_store) = capability_catalog_cache_store() else {
        let models = store.list_models().await?;
        return Ok(ListModelsResponse::new(
            models
                .into_iter()
                .map(|entry| ModelObject::new(entry.external_name, entry.provider_id))
                .collect(),
        ));
    };
    let cache_key = capability_catalog_list_cache_key(tenant_id, project_id);
    let payload = cache_get_or_insert_with(
        cache_store.as_ref(),
        CAPABILITY_CATALOG_CACHE_NAMESPACE,
        &cache_key,
        Some(CAPABILITY_CATALOG_CACHE_TTL_MS),
        &[CacheTag::new(CAPABILITY_CATALOG_CACHE_TAG_ALL_MODELS)],
        || async {
            let models = store.list_models().await?;
            let cached = CachedCapabilityCatalogList {
                models: models
                    .into_iter()
                    .map(|entry| CachedCapabilityCatalogModel {
                        id: entry.external_name,
                        owned_by: entry.provider_id,
                    })
                    .collect(),
            };
            Ok(serde_json::to_vec(&cached)?)
        },
    )
    .await?;
    let cached: CachedCapabilityCatalogList = serde_json::from_slice(&payload)
        .context("failed to decode capability catalog list cache payload")?;
    Ok(cached.into_response())
}

pub async fn get_model_from_store(
    store: &dyn AdminStore,
    tenant_id: &str,
    project_id: &str,
    model_id: &str,
) -> Result<Option<ModelObject>> {
    let Some(cache_store) = capability_catalog_cache_store() else {
        return Ok(store
            .find_model(model_id)
            .await?
            .map(|entry| ModelObject::new(entry.external_name, entry.provider_id)));
    };
    let cache_key = capability_catalog_model_cache_key(tenant_id, project_id, model_id);
    let payload = cache_get_or_insert_with(
        cache_store.as_ref(),
        CAPABILITY_CATALOG_CACHE_NAMESPACE,
        &cache_key,
        Some(CAPABILITY_CATALOG_CACHE_TTL_MS),
        &[
            CacheTag::new(CAPABILITY_CATALOG_CACHE_TAG_ALL_MODELS),
            CacheTag::new(format!("model:{model_id}")),
        ],
        || async {
            let cached =
                store
                    .find_model(model_id)
                    .await?
                    .map(|entry| CachedCapabilityCatalogModel {
                        id: entry.external_name,
                        owned_by: entry.provider_id,
                    });
            Ok(serde_json::to_vec(&cached)?)
        },
    )
    .await?;
    let cached: Option<CachedCapabilityCatalogModel> = serde_json::from_slice(&payload)
        .context("failed to decode capability catalog model cache payload")?;
    Ok(cached.map(CachedCapabilityCatalogModel::into_model_object))
}

pub async fn delete_model_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    model_id: &str,
) -> Result<Option<Value>> {
    let Some(model_entry) = store.find_model(model_id).await? else {
        return Ok(None);
    };

    if let Some(provider) = store.find_provider(&model_entry.provider_id).await? {
        if let Some(api_key) =
            resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
                .await?
        {
            let response = execute_json_provider_request_for_provider(
                store,
                &provider,
                &api_key,
                ProviderRequest::ModelsDelete(model_id),
            )
            .await?;

            if let Some(response) = response {
                let _ = store.delete_model(model_id).await?;
                invalidate_capability_catalog_cache().await;
                return Ok(Some(response));
            }
        }
    }

    if store.delete_model(model_id).await? {
        invalidate_capability_catalog_cache().await;
        return Ok(Some(serde_json::to_value(DeleteModelResponse::deleted(
            model_id,
        ))?));
    }

    Ok(None)
}
