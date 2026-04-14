use super::*;

pub async fn relay_response_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CreateResponseRequest,
) -> Result<Option<Value>> {
    Ok(relay_response_from_store_with_execution_context(
        store,
        secret_manager,
        tenant_id,
        _project_id,
        request,
    )
    .await?
    .response)
}

pub async fn relay_response_from_store_with_execution_context(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    project_id: &str,
    request: &CreateResponseRequest,
) -> Result<GatewayExecutionResult<Value>> {
    let original_decision = select_gateway_route(
        store,
        tenant_id,
        Some(project_id),
        "responses",
        &request.model,
    )
    .await?;
    let execution_policy = gateway_execution_policy_for_decision(
        store,
        &original_decision,
        &ProviderRequest::Responses(request),
    )
    .await?;
    let Some((decision, provider, descriptor)) = resolve_store_relay_provider_for_decision(
        store,
        secret_manager,
        tenant_id,
        &original_decision,
        execution_policy.failover_enabled,
    )
    .await?
    else {
        return Ok(GatewayExecutionResult::new(None, None));
    };
    let selected_provider_id = decision.selected_provider_id.clone();
    let preflight_failover_from_provider_id = (selected_provider_id
        != original_decision.selected_provider_id)
        .then(|| original_decision.selected_provider_id.clone());
    let options = ProviderRequestOptions::default();

    match execute_json_provider_request_for_descriptor_with_options(
        store,
        &provider,
        &descriptor,
        ProviderRequest::Responses(request),
        &options,
        execution_policy.retry_policy,
    )
    .await
    {
        Ok(Some(response)) => {
            if let Some(from_provider_id) = preflight_failover_from_provider_id.as_deref() {
                record_gateway_execution_failover(
                    "responses",
                    from_provider_id,
                    &provider.id,
                    "success",
                );
                persist_gateway_execution_failover_decision_log(
                    store,
                    tenant_id,
                    project_id,
                    "responses",
                    &request.model,
                    &original_decision,
                    &provider.id,
                )
                .await?;
            }
            let usage_context = gateway_usage_context_for_decision_provider(
                store,
                tenant_id,
                &decision,
                &provider.id,
                current_request_api_key_group_id(),
                decision.fallback_reason.clone(),
            )
            .await?;
            Ok(GatewayExecutionResult::new(
                Some(response),
                Some(usage_context),
            ))
        }
        Ok(None) => Ok(GatewayExecutionResult::new(None, None)),
        Err(mut last_error) => {
            if let Some(from_provider_id) = preflight_failover_from_provider_id.as_deref() {
                record_gateway_execution_failover(
                    "responses",
                    from_provider_id,
                    &provider.id,
                    "failure",
                );
            }
            if !execution_policy.failover_enabled {
                return Err(last_error);
            }
            for candidate_provider_id in decision
                .candidate_ids
                .iter()
                .filter(|provider_id| provider_id.as_str() != selected_provider_id)
            {
                let Some(candidate_provider) = store.find_provider(candidate_provider_id).await?
                else {
                    continue;
                };
                let Some(candidate_descriptor) =
                    provider_execution_descriptor_for_provider_account_context(
                        store,
                        secret_manager,
                        tenant_id,
                        &candidate_provider,
                        decision.requested_region.as_deref(),
                    )
                    .await?
                else {
                    continue;
                };
                match execute_json_provider_request_for_descriptor_with_options(
                    store,
                    &candidate_provider,
                    &candidate_descriptor,
                    ProviderRequest::Responses(request),
                    &options,
                    execution_policy.retry_policy,
                )
                .await
                {
                    Ok(Some(response)) => {
                        record_gateway_execution_failover(
                            "responses",
                            &selected_provider_id,
                            &candidate_provider.id,
                            "success",
                        );
                        persist_gateway_execution_failover_decision_log(
                            store,
                            tenant_id,
                            project_id,
                            "responses",
                            &request.model,
                            &decision,
                            &candidate_provider.id,
                        )
                        .await?;
                        let usage_context = gateway_usage_context_for_decision_provider(
                            store,
                            tenant_id,
                            &decision,
                            &candidate_provider.id,
                            current_request_api_key_group_id(),
                            gateway_execution_failover_fallback_reason(
                                decision.fallback_reason.as_deref(),
                            ),
                        )
                        .await?;
                        return Ok(GatewayExecutionResult::new(
                            Some(response),
                            Some(usage_context),
                        ));
                    }
                    Ok(None) => continue,
                    Err(error) => {
                        record_gateway_execution_failover(
                            "responses",
                            &selected_provider_id,
                            &candidate_provider.id,
                            "failure",
                        );
                        last_error = error;
                    }
                }
            }
            Err(last_error)
        }
    }
}

pub async fn relay_response_stream_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CreateResponseRequest,
) -> Result<Option<ProviderStreamOutput>> {
    Ok(relay_response_stream_from_store_with_execution_context(
        store,
        secret_manager,
        tenant_id,
        _project_id,
        request,
    )
    .await?
    .response)
}

pub async fn relay_response_stream_from_store_with_execution_context(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    project_id: &str,
    request: &CreateResponseRequest,
) -> Result<GatewayExecutionResult<ProviderStreamOutput>> {
    let original_decision = select_gateway_route(
        store,
        tenant_id,
        Some(project_id),
        "responses",
        &request.model,
    )
    .await?;
    let execution_policy = gateway_execution_policy_for_decision(
        store,
        &original_decision,
        &ProviderRequest::ResponsesStream(request),
    )
    .await?;
    let Some((decision, provider, descriptor)) = resolve_store_relay_provider_for_decision(
        store,
        secret_manager,
        tenant_id,
        &original_decision,
        execution_policy.failover_enabled,
    )
    .await?
    else {
        return Ok(GatewayExecutionResult::new(None, None));
    };
    let selected_provider_id = decision.selected_provider_id.clone();
    let preflight_failover_from_provider_id = (selected_provider_id
        != original_decision.selected_provider_id)
        .then(|| original_decision.selected_provider_id.clone());
    let options = ProviderRequestOptions::default();

    match execute_stream_provider_request_for_descriptor_with_options(
        store,
        &provider,
        &descriptor,
        ProviderRequest::ResponsesStream(request),
        &options,
        execution_policy.retry_policy,
    )
    .await
    {
        Ok(Some(response)) => {
            if let Some(from_provider_id) = preflight_failover_from_provider_id.as_deref() {
                record_gateway_execution_failover(
                    "responses",
                    from_provider_id,
                    &provider.id,
                    "success",
                );
                persist_gateway_execution_failover_decision_log(
                    store,
                    tenant_id,
                    project_id,
                    "responses",
                    &request.model,
                    &original_decision,
                    &provider.id,
                )
                .await?;
            }
            let usage_context = gateway_usage_context_for_decision_provider(
                store,
                tenant_id,
                &decision,
                &provider.id,
                current_request_api_key_group_id(),
                decision.fallback_reason.clone(),
            )
            .await?;
            Ok(GatewayExecutionResult::new(
                Some(response),
                Some(usage_context),
            ))
        }
        Ok(None) => Ok(GatewayExecutionResult::new(None, None)),
        Err(mut last_error) => {
            if let Some(from_provider_id) = preflight_failover_from_provider_id.as_deref() {
                record_gateway_execution_failover(
                    "responses",
                    from_provider_id,
                    &provider.id,
                    "failure",
                );
            }
            if !execution_policy.failover_enabled {
                return Err(last_error);
            }
            for candidate_provider_id in decision
                .candidate_ids
                .iter()
                .filter(|provider_id| provider_id.as_str() != selected_provider_id)
            {
                let Some(candidate_provider) = store.find_provider(candidate_provider_id).await?
                else {
                    continue;
                };
                let Some(candidate_descriptor) =
                    provider_execution_descriptor_for_provider_account_context(
                        store,
                        secret_manager,
                        tenant_id,
                        &candidate_provider,
                        decision.requested_region.as_deref(),
                    )
                    .await?
                else {
                    continue;
                };
                match execute_stream_provider_request_for_descriptor_with_options(
                    store,
                    &candidate_provider,
                    &candidate_descriptor,
                    ProviderRequest::ResponsesStream(request),
                    &options,
                    execution_policy.retry_policy,
                )
                .await
                {
                    Ok(Some(response)) => {
                        record_gateway_execution_failover(
                            "responses",
                            &selected_provider_id,
                            &candidate_provider.id,
                            "success",
                        );
                        persist_gateway_execution_failover_decision_log(
                            store,
                            tenant_id,
                            project_id,
                            "responses",
                            &request.model,
                            &decision,
                            &candidate_provider.id,
                        )
                        .await?;
                        let usage_context = gateway_usage_context_for_decision_provider(
                            store,
                            tenant_id,
                            &decision,
                            &candidate_provider.id,
                            current_request_api_key_group_id(),
                            gateway_execution_failover_fallback_reason(
                                decision.fallback_reason.as_deref(),
                            ),
                        )
                        .await?;
                        return Ok(GatewayExecutionResult::new(
                            Some(response),
                            Some(usage_context),
                        ));
                    }
                    Ok(None) => continue,
                    Err(error) => {
                        record_gateway_execution_failover(
                            "responses",
                            &selected_provider_id,
                            &candidate_provider.id,
                            "failure",
                        );
                        last_error = error;
                    }
                }
            }
            Err(last_error)
        }
    }
}

pub async fn relay_count_response_input_tokens_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    project_id: &str,
    request: &CountResponseInputTokensRequest,
) -> Result<Option<Value>> {
    let original_decision = select_gateway_route(
        store,
        tenant_id,
        Some(project_id),
        "responses",
        &request.model,
    )
    .await?;
    let execution_policy = gateway_execution_policy_for_decision(
        store,
        &original_decision,
        &ProviderRequest::ResponsesInputTokens(request),
    )
    .await?;
    let Some((decision, provider, descriptor)) = resolve_store_relay_provider_for_decision(
        store,
        secret_manager,
        tenant_id,
        &original_decision,
        execution_policy.failover_enabled,
    )
    .await?
    else {
        return Ok(None);
    };
    let selected_provider_id = decision.selected_provider_id.clone();
    let preflight_failover_from_provider_id = (selected_provider_id
        != original_decision.selected_provider_id)
        .then(|| original_decision.selected_provider_id.clone());
    let options = ProviderRequestOptions::default();

    match execute_json_provider_request_for_descriptor_with_options(
        store,
        &provider,
        &descriptor,
        ProviderRequest::ResponsesInputTokens(request),
        &options,
        execution_policy.retry_policy,
    )
    .await
    {
        Ok(Some(response)) => {
            if let Some(from_provider_id) = preflight_failover_from_provider_id.as_deref() {
                record_gateway_execution_failover(
                    "responses",
                    from_provider_id,
                    &provider.id,
                    "success",
                );
                persist_gateway_execution_failover_decision_log(
                    store,
                    tenant_id,
                    project_id,
                    "responses",
                    &request.model,
                    &original_decision,
                    &provider.id,
                )
                .await?;
            }
            Ok(Some(response))
        }
        Ok(None) => Ok(None),
        Err(mut last_error) => {
            if let Some(from_provider_id) = preflight_failover_from_provider_id.as_deref() {
                record_gateway_execution_failover(
                    "responses",
                    from_provider_id,
                    &provider.id,
                    "failure",
                );
            }
            if !execution_policy.failover_enabled {
                return Err(last_error);
            }
            for candidate_provider_id in decision
                .candidate_ids
                .iter()
                .filter(|provider_id| provider_id.as_str() != selected_provider_id)
            {
                let Some(candidate_provider) = store.find_provider(candidate_provider_id).await?
                else {
                    continue;
                };
                let Some(candidate_descriptor) =
                    provider_execution_descriptor_for_provider_account_context(
                        store,
                        secret_manager,
                        tenant_id,
                        &candidate_provider,
                        decision.requested_region.as_deref(),
                    )
                    .await?
                else {
                    continue;
                };
                match execute_json_provider_request_for_descriptor_with_options(
                    store,
                    &candidate_provider,
                    &candidate_descriptor,
                    ProviderRequest::ResponsesInputTokens(request),
                    &options,
                    execution_policy.retry_policy,
                )
                .await
                {
                    Ok(Some(response)) => {
                        record_gateway_execution_failover(
                            "responses",
                            &selected_provider_id,
                            &candidate_provider.id,
                            "success",
                        );
                        persist_gateway_execution_failover_decision_log(
                            store,
                            tenant_id,
                            project_id,
                            "responses",
                            &request.model,
                            &decision,
                            &candidate_provider.id,
                        )
                        .await?;
                        return Ok(Some(response));
                    }
                    Ok(None) => continue,
                    Err(error) => {
                        record_gateway_execution_failover(
                            "responses",
                            &selected_provider_id,
                            &candidate_provider.id,
                            "failure",
                        );
                        last_error = error;
                    }
                }
            }
            Err(last_error)
        }
    }
}

pub async fn relay_count_response_input_tokens_from_planned_execution_context(
    store: &dyn AdminStore,
    tenant_id: &str,
    project_id: &str,
    request: &CountResponseInputTokensRequest,
    planned: &PlannedExecutionProviderContext,
) -> Result<Option<Value>> {
    persist_planned_execution_decision_log(
        store,
        tenant_id,
        project_id,
        "responses",
        &request.model,
        &planned.decision,
    )
    .await?;

    let execution_policy = gateway_execution_policy_for_decision(
        store,
        &planned.decision,
        &ProviderRequest::ResponsesInputTokens(request),
    )
    .await?;
    let descriptor = provider_execution_descriptor_from_planned_context(planned);

    execute_json_provider_request_for_descriptor_with_options(
        store,
        &planned.provider,
        &descriptor,
        ProviderRequest::ResponsesInputTokens(request),
        &ProviderRequestOptions::default(),
        execution_policy.retry_policy,
    )
    .await
}

pub async fn relay_get_response_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    response_id: &str,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "responses",
        response_id,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::ResponsesRetrieve(response_id),
    )
    .await
}

pub async fn relay_cancel_response_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    response_id: &str,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "responses",
        response_id,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::ResponsesCancel(response_id),
    )
    .await
}

pub async fn relay_delete_response_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    response_id: &str,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "responses",
        response_id,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::ResponsesDelete(response_id),
    )
    .await
}

pub async fn relay_compact_response_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CompactResponseRequest,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "responses",
        &request.model,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::ResponsesCompact(request),
    )
    .await
}

pub async fn relay_list_response_input_items_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    response_id: &str,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "responses",
        response_id,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::ResponsesInputItemsList(response_id),
    )
    .await
}

fn ensure_local_response_model_present(model: &str) -> Result<()> {
    if model.trim().is_empty() {
        bail!("Response model is required.");
    }

    Ok(())
}

pub fn create_response(_tenant_id: &str, _project_id: &str, model: &str) -> Result<ResponseObject> {
    ensure_local_response_model_present(model)?;
    let _ = model;
    bail!("Local response fallback is not supported without an upstream provider.")
}

pub fn count_response_input_tokens(
    _tenant_id: &str,
    _project_id: &str,
    model: &str,
) -> Result<ResponseInputTokensObject> {
    ensure_local_response_model_present(model)?;
    bail!("Response input token counting is not supported in local fallback.")
}

fn ensure_local_response_exists(response_id: &str) -> Result<()> {
    if !local_object_id_matches(response_id, "resp") {
        bail!("response not found");
    }

    Ok(())
}

pub fn get_response(
    _tenant_id: &str,
    _project_id: &str,
    response_id: &str,
) -> Result<ResponseObject> {
    ensure_local_response_exists(response_id)?;
    bail!("response not found")
}

pub fn list_response_input_items(
    _tenant_id: &str,
    _project_id: &str,
    response_id: &str,
) -> Result<ListResponseInputItemsResponse> {
    ensure_local_response_exists(response_id)?;
    bail!("Persisted local response input item state is required for local input item listing.")
}

pub fn delete_response(
    _tenant_id: &str,
    _project_id: &str,
    response_id: &str,
) -> Result<DeleteResponseResponse> {
    ensure_local_response_exists(response_id)?;
    bail!("response not found")
}

pub fn cancel_response(
    _tenant_id: &str,
    _project_id: &str,
    response_id: &str,
) -> Result<ResponseObject> {
    ensure_local_response_exists(response_id)?;
    bail!("response not found")
}

pub fn compact_response(
    _tenant_id: &str,
    _project_id: &str,
    model: &str,
) -> Result<ResponseCompactionObject> {
    if model.trim().is_empty() {
        bail!("Response compaction model is required.");
    }
    let _ = model;
    bail!("Local response compaction fallback is not supported without an upstream provider.")
}
