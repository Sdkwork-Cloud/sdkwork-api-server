use super::*;

pub async fn relay_chat_completion_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CreateChatCompletionRequest,
) -> Result<Option<Value>> {
    let options = ProviderRequestOptions::default();
    Ok(relay_chat_completion_from_store_with_execution_context(
        store,
        secret_manager,
        tenant_id,
        _project_id,
        request,
        &options,
    )
    .await?
    .response)
}

pub async fn relay_chat_completion_from_store_with_execution_context(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    project_id: &str,
    request: &CreateChatCompletionRequest,
    options: &ProviderRequestOptions,
) -> Result<GatewayExecutionResult<Value>> {
    let original_decision = select_gateway_route(
        store,
        tenant_id,
        Some(project_id),
        "chat_completion",
        &request.model,
    )
    .await?;
    let execution_policy = gateway_execution_policy_for_decision(
        store,
        &original_decision,
        &ProviderRequest::ChatCompletions(request),
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
    match execute_json_provider_request_for_descriptor_with_options(
        store,
        &provider,
        &descriptor,
        ProviderRequest::ChatCompletions(request),
        options,
        execution_policy.retry_policy,
    )
    .await
    {
        Ok(Some(response)) => {
            if let Some(from_provider_id) = preflight_failover_from_provider_id.as_deref() {
                record_gateway_execution_failover(
                    "chat_completion",
                    from_provider_id,
                    &provider.id,
                    "success",
                );
                persist_gateway_execution_failover_decision_log(
                    store,
                    tenant_id,
                    project_id,
                    "chat_completion",
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
                    "chat_completion",
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
                    ProviderRequest::ChatCompletions(request),
                    options,
                    execution_policy.retry_policy,
                )
                .await
                {
                    Ok(Some(response)) => {
                        record_gateway_execution_failover(
                            "chat_completion",
                            &selected_provider_id,
                            &candidate_provider.id,
                            "success",
                        );
                        persist_gateway_execution_failover_decision_log(
                            store,
                            tenant_id,
                            project_id,
                            "chat_completion",
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
                            "chat_completion",
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

pub async fn relay_chat_completion_from_store_with_options(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CreateChatCompletionRequest,
    options: &ProviderRequestOptions,
) -> Result<Option<Value>> {
    Ok(relay_chat_completion_from_store_with_execution_context(
        store,
        secret_manager,
        tenant_id,
        _project_id,
        request,
        options,
    )
    .await?
    .response)
}

pub async fn relay_chat_completion_from_planned_execution_context_with_options(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    project_id: &str,
    request: &CreateChatCompletionRequest,
    options: &ProviderRequestOptions,
    planned: &PlannedExecutionProviderContext,
) -> Result<GatewayExecutionResult<Value>> {
    persist_planned_execution_decision_log(
        store,
        tenant_id,
        project_id,
        "chat_completion",
        &request.model,
        &planned.decision,
    )
    .await?;

    let execution_policy = gateway_execution_policy_for_decision(
        store,
        &planned.decision,
        &ProviderRequest::ChatCompletions(request),
    )
    .await?;
    let selected_provider_id = planned.provider.id.clone();
    let descriptor = provider_execution_descriptor_from_planned_context(planned);
    match execute_json_provider_request_for_descriptor_with_options(
        store,
        &planned.provider,
        &descriptor,
        ProviderRequest::ChatCompletions(request),
        options,
        execution_policy.retry_policy,
    )
    .await
    {
        Ok(Some(response)) => Ok(GatewayExecutionResult::new(
            Some(response),
            Some(planned.usage_context.clone()),
        )),
        Ok(None) => Ok(GatewayExecutionResult::new(None, None)),
        Err(mut last_error) => {
            if !execution_policy.failover_enabled {
                return Err(last_error);
            }
            for candidate_provider_id in planned
                .decision
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
                        planned.decision.requested_region.as_deref(),
                    )
                    .await?
                else {
                    continue;
                };
                match execute_json_provider_request_for_descriptor_with_options(
                    store,
                    &candidate_provider,
                    &candidate_descriptor,
                    ProviderRequest::ChatCompletions(request),
                    options,
                    execution_policy.retry_policy,
                )
                .await
                {
                    Ok(Some(response)) => {
                        record_gateway_execution_failover(
                            "chat_completion",
                            &selected_provider_id,
                            &candidate_provider.id,
                            "success",
                        );
                        persist_gateway_execution_failover_decision_log(
                            store,
                            tenant_id,
                            project_id,
                            "chat_completion",
                            &request.model,
                            &planned.decision,
                            &candidate_provider.id,
                        )
                        .await?;
                        let usage_context = gateway_usage_context_for_decision_provider(
                            store,
                            tenant_id,
                            &planned.decision,
                            &candidate_provider.id,
                            current_request_api_key_group_id(),
                            gateway_execution_failover_fallback_reason(
                                planned.decision.fallback_reason.as_deref(),
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
                            "chat_completion",
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

pub async fn relay_list_chat_completions_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "chat_completion",
        "chat_completions",
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
        ProviderRequest::ChatCompletionsList,
    )
    .await
}

pub async fn relay_get_chat_completion_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    completion_id: &str,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "chat_completion",
        completion_id,
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
        ProviderRequest::ChatCompletionsRetrieve(completion_id),
    )
    .await
}

pub async fn relay_update_chat_completion_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    completion_id: &str,
    request: &UpdateChatCompletionRequest,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "chat_completion",
        completion_id,
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
        ProviderRequest::ChatCompletionsUpdate(completion_id, request),
    )
    .await
}

pub async fn relay_delete_chat_completion_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    completion_id: &str,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "chat_completion",
        completion_id,
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
        ProviderRequest::ChatCompletionsDelete(completion_id),
    )
    .await
}

pub async fn relay_list_chat_completion_messages_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    completion_id: &str,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "chat_completion",
        completion_id,
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
        ProviderRequest::ChatCompletionsMessagesList(completion_id),
    )
    .await
}

pub async fn relay_chat_completion_stream_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CreateChatCompletionRequest,
) -> Result<Option<ProviderStreamOutput>> {
    let options = ProviderRequestOptions::default();
    Ok(
        relay_chat_completion_stream_from_store_with_execution_context(
            store,
            secret_manager,
            tenant_id,
            _project_id,
            request,
            &options,
        )
        .await?
        .response,
    )
}

pub async fn relay_chat_completion_stream_from_store_with_execution_context(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    project_id: &str,
    request: &CreateChatCompletionRequest,
    options: &ProviderRequestOptions,
) -> Result<GatewayExecutionResult<ProviderStreamOutput>> {
    let original_decision = select_gateway_route(
        store,
        tenant_id,
        Some(project_id),
        "chat_completion",
        &request.model,
    )
    .await?;
    let execution_policy = gateway_execution_policy_for_decision(
        store,
        &original_decision,
        &ProviderRequest::ChatCompletionsStream(request),
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
    match execute_stream_provider_request_for_descriptor_with_options(
        store,
        &provider,
        &descriptor,
        ProviderRequest::ChatCompletionsStream(request),
        options,
        execution_policy.retry_policy,
    )
    .await
    {
        Ok(Some(response)) => {
            if let Some(from_provider_id) = preflight_failover_from_provider_id.as_deref() {
                record_gateway_execution_failover(
                    "chat_completion",
                    from_provider_id,
                    &provider.id,
                    "success",
                );
                persist_gateway_execution_failover_decision_log(
                    store,
                    tenant_id,
                    project_id,
                    "chat_completion",
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
                    "chat_completion",
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
                    ProviderRequest::ChatCompletionsStream(request),
                    options,
                    execution_policy.retry_policy,
                )
                .await
                {
                    Ok(Some(response)) => {
                        record_gateway_execution_failover(
                            "chat_completion",
                            &selected_provider_id,
                            &candidate_provider.id,
                            "success",
                        );
                        persist_gateway_execution_failover_decision_log(
                            store,
                            tenant_id,
                            project_id,
                            "chat_completion",
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
                            "chat_completion",
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

pub async fn relay_chat_completion_stream_from_store_with_options(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CreateChatCompletionRequest,
    options: &ProviderRequestOptions,
) -> Result<Option<ProviderStreamOutput>> {
    Ok(
        relay_chat_completion_stream_from_store_with_execution_context(
            store,
            secret_manager,
            tenant_id,
            _project_id,
            request,
            options,
        )
        .await?
        .response,
    )
}

pub async fn relay_chat_completion_stream_from_planned_execution_context_with_options(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    project_id: &str,
    request: &CreateChatCompletionRequest,
    options: &ProviderRequestOptions,
    planned: &PlannedExecutionProviderContext,
) -> Result<GatewayExecutionResult<ProviderStreamOutput>> {
    persist_planned_execution_decision_log(
        store,
        tenant_id,
        project_id,
        "chat_completion",
        &request.model,
        &planned.decision,
    )
    .await?;

    let execution_policy = gateway_execution_policy_for_decision(
        store,
        &planned.decision,
        &ProviderRequest::ChatCompletionsStream(request),
    )
    .await?;
    let selected_provider_id = planned.provider.id.clone();
    let descriptor = provider_execution_descriptor_from_planned_context(planned);
    match execute_stream_provider_request_for_descriptor_with_options(
        store,
        &planned.provider,
        &descriptor,
        ProviderRequest::ChatCompletionsStream(request),
        options,
        execution_policy.retry_policy,
    )
    .await
    {
        Ok(Some(response)) => Ok(GatewayExecutionResult::new(
            Some(response),
            Some(planned.usage_context.clone()),
        )),
        Ok(None) => Ok(GatewayExecutionResult::new(None, None)),
        Err(mut last_error) => {
            if !execution_policy.failover_enabled {
                return Err(last_error);
            }
            for candidate_provider_id in planned
                .decision
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
                        planned.decision.requested_region.as_deref(),
                    )
                    .await?
                else {
                    continue;
                };
                match execute_stream_provider_request_for_descriptor_with_options(
                    store,
                    &candidate_provider,
                    &candidate_descriptor,
                    ProviderRequest::ChatCompletionsStream(request),
                    options,
                    execution_policy.retry_policy,
                )
                .await
                {
                    Ok(Some(response)) => {
                        record_gateway_execution_failover(
                            "chat_completion",
                            &selected_provider_id,
                            &candidate_provider.id,
                            "success",
                        );
                        persist_gateway_execution_failover_decision_log(
                            store,
                            tenant_id,
                            project_id,
                            "chat_completion",
                            &request.model,
                            &planned.decision,
                            &candidate_provider.id,
                        )
                        .await?;
                        let usage_context = gateway_usage_context_for_decision_provider(
                            store,
                            tenant_id,
                            &planned.decision,
                            &candidate_provider.id,
                            current_request_api_key_group_id(),
                            gateway_execution_failover_fallback_reason(
                                planned.decision.fallback_reason.as_deref(),
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
                            "chat_completion",
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

pub fn create_chat_completion(
    _tenant_id: &str,
    _project_id: &str,
    model: &str,
) -> Result<ChatCompletionResponse> {
    if model.trim().is_empty() {
        bail!("Chat completion model is required.");
    }

    let _ = model;
    bail!("Local chat completion fallback is not supported without an upstream provider.")
}

pub fn list_chat_completions(
    _tenant_id: &str,
    _project_id: &str,
) -> Result<ListChatCompletionsResponse> {
    bail!("Local chat completion listing fallback is not supported without an upstream provider.")
}

pub fn get_chat_completion(
    _tenant_id: &str,
    _project_id: &str,
    completion_id: &str,
) -> Result<ChatCompletionResponse> {
    ensure_local_chat_completion_exists(completion_id)?;
    bail!("chat completion not found")
}

pub fn update_chat_completion(
    _tenant_id: &str,
    _project_id: &str,
    completion_id: &str,
    metadata: Value,
) -> Result<ChatCompletionResponse> {
    let _ = metadata;
    ensure_local_chat_completion_exists(completion_id)?;
    bail!("chat completion not found")
}

pub fn delete_chat_completion(
    _tenant_id: &str,
    _project_id: &str,
    completion_id: &str,
) -> Result<DeleteChatCompletionResponse> {
    ensure_local_chat_completion_exists(completion_id)?;
    bail!("chat completion not found")
}

fn ensure_local_chat_completion_exists(completion_id: &str) -> Result<()> {
    if !local_object_id_matches(completion_id, "chatcmpl") {
        bail!("chat completion not found");
    }

    Ok(())
}

pub fn list_chat_completion_messages(
    _tenant_id: &str,
    _project_id: &str,
    completion_id: &str,
) -> Result<ListChatCompletionMessagesResponse> {
    ensure_local_chat_completion_exists(completion_id)?;
    bail!("Persisted local chat completion message state is required for local message listing.")
}
