use super::*;

enum ProviderRequestRewrite<'a> {
    Borrowed(ProviderRequest<'a>),
    ModelsRetrieve(String),
    ModelsDelete(String),
    ChatCompletions(CreateChatCompletionRequest),
    ChatCompletionsStream(CreateChatCompletionRequest),
    Completions(CreateCompletionRequest),
    ThreadRuns(String, CreateRunRequest),
    ThreadsRuns(CreateThreadAndRunRequest),
    Responses(CreateResponseRequest),
    ResponsesStream(CreateResponseRequest),
    ResponsesInputTokens(CountResponseInputTokensRequest),
    ResponsesCompact(CompactResponseRequest),
    Embeddings(CreateEmbeddingRequest),
    Moderations(CreateModerationRequest),
    Music(CreateMusicRequest),
    ImagesGenerations(CreateImageRequest),
    ImagesEdits(CreateImageEditRequest),
    ImagesVariations(CreateImageVariationRequest),
    AudioTranscriptions(CreateTranscriptionRequest),
    AudioTranslations(CreateTranslationRequest),
    AudioSpeech(CreateSpeechRequest),
    FineTuningJobs(CreateFineTuningJobRequest),
    Assistants(CreateAssistantRequest),
    AssistantsUpdate(String, UpdateAssistantRequest),
    RealtimeSessions(CreateRealtimeSessionRequest),
    Videos(CreateVideoRequest),
}

impl<'a> ProviderRequestRewrite<'a> {
    fn as_request(&self) -> ProviderRequest<'_> {
        match self {
            Self::Borrowed(request) => *request,
            Self::ModelsRetrieve(model_id) => ProviderRequest::ModelsRetrieve(model_id.as_str()),
            Self::ModelsDelete(model_id) => ProviderRequest::ModelsDelete(model_id.as_str()),
            Self::ChatCompletions(request) => ProviderRequest::ChatCompletions(request),
            Self::ChatCompletionsStream(request) => ProviderRequest::ChatCompletionsStream(request),
            Self::Completions(request) => ProviderRequest::Completions(request),
            Self::ThreadRuns(thread_id, request) => {
                ProviderRequest::ThreadRuns(thread_id.as_str(), request)
            }
            Self::ThreadsRuns(request) => ProviderRequest::ThreadsRuns(request),
            Self::Responses(request) => ProviderRequest::Responses(request),
            Self::ResponsesStream(request) => ProviderRequest::ResponsesStream(request),
            Self::ResponsesInputTokens(request) => ProviderRequest::ResponsesInputTokens(request),
            Self::ResponsesCompact(request) => ProviderRequest::ResponsesCompact(request),
            Self::Embeddings(request) => ProviderRequest::Embeddings(request),
            Self::Moderations(request) => ProviderRequest::Moderations(request),
            Self::Music(request) => ProviderRequest::Music(request),
            Self::ImagesGenerations(request) => ProviderRequest::ImagesGenerations(request),
            Self::ImagesEdits(request) => ProviderRequest::ImagesEdits(request),
            Self::ImagesVariations(request) => ProviderRequest::ImagesVariations(request),
            Self::AudioTranscriptions(request) => ProviderRequest::AudioTranscriptions(request),
            Self::AudioTranslations(request) => ProviderRequest::AudioTranslations(request),
            Self::AudioSpeech(request) => ProviderRequest::AudioSpeech(request),
            Self::FineTuningJobs(request) => ProviderRequest::FineTuningJobs(request),
            Self::Assistants(request) => ProviderRequest::Assistants(request),
            Self::AssistantsUpdate(assistant_id, request) => {
                ProviderRequest::AssistantsUpdate(assistant_id.as_str(), request)
            }
            Self::RealtimeSessions(request) => ProviderRequest::RealtimeSessions(request),
            Self::Videos(request) => ProviderRequest::Videos(request),
        }
    }
}

fn provider_request_canonical_model_id<'a>(request: &'a ProviderRequest<'a>) -> Option<&'a str> {
    match request {
        ProviderRequest::ModelsRetrieve(model_id) | ProviderRequest::ModelsDelete(model_id) => {
            Some(model_id)
        }
        ProviderRequest::ChatCompletions(request)
        | ProviderRequest::ChatCompletionsStream(request) => Some(request.model.as_str()),
        ProviderRequest::Completions(request) => Some(request.model.as_str()),
        ProviderRequest::ThreadRuns(_, request) => request.model.as_deref(),
        ProviderRequest::ThreadsRuns(request) => request.model.as_deref(),
        ProviderRequest::Responses(request) | ProviderRequest::ResponsesStream(request) => {
            Some(request.model.as_str())
        }
        ProviderRequest::ResponsesInputTokens(request) => Some(request.model.as_str()),
        ProviderRequest::ResponsesCompact(request) => Some(request.model.as_str()),
        ProviderRequest::Embeddings(request) => Some(request.model.as_str()),
        ProviderRequest::Moderations(request) => Some(request.model.as_str()),
        ProviderRequest::Music(request) => Some(request.model.as_str()),
        ProviderRequest::ImagesGenerations(request) => Some(request.model.as_str()),
        ProviderRequest::ImagesEdits(request) => request.model.as_deref(),
        ProviderRequest::ImagesVariations(request) => request.model.as_deref(),
        ProviderRequest::AudioTranscriptions(request) => Some(request.model.as_str()),
        ProviderRequest::AudioTranslations(request) => Some(request.model.as_str()),
        ProviderRequest::AudioSpeech(request) => Some(request.model.as_str()),
        ProviderRequest::FineTuningJobs(request) => Some(request.model.as_str()),
        ProviderRequest::Assistants(request) => Some(request.model.as_str()),
        ProviderRequest::AssistantsUpdate(_, request) => request.model.as_deref(),
        ProviderRequest::RealtimeSessions(request) => Some(request.model.as_str()),
        ProviderRequest::Videos(request) => Some(request.model.as_str()),
        _ => None,
    }
}

async fn resolve_provider_model_id_for_request(
    store: &dyn AdminStore,
    provider: &ProxyProvider,
    request: &ProviderRequest<'_>,
) -> Result<Option<String>> {
    let Some(canonical_model_id) = provider_request_canonical_model_id(request) else {
        return Ok(None);
    };
    let canonical_model_id = canonical_model_id.trim();
    if canonical_model_id.is_empty() {
        return Ok(None);
    }

    let mut matches = store
        .list_provider_models_for_provider(&provider.id)
        .await?
        .into_iter()
        .filter(|record| record.is_active && record.model_id == canonical_model_id)
        .collect::<Vec<_>>();
    if matches.is_empty() {
        return Ok(None);
    }

    matches.sort_by(|left, right| {
        (
            !left.is_default_route,
            left.channel_id != provider.channel_id,
            left.channel_id.as_str(),
            left.provider_model_id.as_str(),
        )
            .cmp(&(
                !right.is_default_route,
                right.channel_id != provider.channel_id,
                right.channel_id.as_str(),
                right.provider_model_id.as_str(),
            ))
    });

    let provider_model_id = matches
        .into_iter()
        .next()
        .map(|record| record.provider_model_id)
        .unwrap_or_default();
    if provider_model_id.is_empty() || provider_model_id == canonical_model_id {
        return Ok(None);
    }

    Ok(Some(provider_model_id))
}

async fn rewrite_provider_request_for_execution<'a>(
    store: &dyn AdminStore,
    provider: &ProxyProvider,
    request: ProviderRequest<'a>,
) -> Result<ProviderRequestRewrite<'a>> {
    let Some(provider_model_id) =
        resolve_provider_model_id_for_request(store, provider, &request).await?
    else {
        return Ok(ProviderRequestRewrite::Borrowed(request));
    };

    Ok(match request {
        ProviderRequest::ModelsRetrieve(_) => {
            ProviderRequestRewrite::ModelsRetrieve(provider_model_id)
        }
        ProviderRequest::ModelsDelete(_) => ProviderRequestRewrite::ModelsDelete(provider_model_id),
        ProviderRequest::ChatCompletions(body) => {
            let mut body = body.clone();
            body.model = provider_model_id;
            ProviderRequestRewrite::ChatCompletions(body)
        }
        ProviderRequest::ChatCompletionsStream(body) => {
            let mut body = body.clone();
            body.model = provider_model_id;
            ProviderRequestRewrite::ChatCompletionsStream(body)
        }
        ProviderRequest::Completions(body) => {
            let mut body = body.clone();
            body.model = provider_model_id;
            ProviderRequestRewrite::Completions(body)
        }
        ProviderRequest::ThreadRuns(thread_id, body) => {
            let mut body = body.clone();
            body.model = Some(provider_model_id);
            ProviderRequestRewrite::ThreadRuns(thread_id.to_owned(), body)
        }
        ProviderRequest::ThreadsRuns(body) => {
            let mut body = body.clone();
            body.model = Some(provider_model_id);
            ProviderRequestRewrite::ThreadsRuns(body)
        }
        ProviderRequest::Responses(body) => {
            let mut body = body.clone();
            body.model = provider_model_id;
            ProviderRequestRewrite::Responses(body)
        }
        ProviderRequest::ResponsesStream(body) => {
            let mut body = body.clone();
            body.model = provider_model_id;
            ProviderRequestRewrite::ResponsesStream(body)
        }
        ProviderRequest::ResponsesInputTokens(body) => {
            let mut body = body.clone();
            body.model = provider_model_id;
            ProviderRequestRewrite::ResponsesInputTokens(body)
        }
        ProviderRequest::ResponsesCompact(body) => {
            let mut body = body.clone();
            body.model = provider_model_id;
            ProviderRequestRewrite::ResponsesCompact(body)
        }
        ProviderRequest::Embeddings(body) => {
            let mut body = body.clone();
            body.model = provider_model_id;
            ProviderRequestRewrite::Embeddings(body)
        }
        ProviderRequest::Moderations(body) => {
            let mut body = body.clone();
            body.model = provider_model_id;
            ProviderRequestRewrite::Moderations(body)
        }
        ProviderRequest::Music(body) => {
            let mut body = body.clone();
            body.model = provider_model_id;
            ProviderRequestRewrite::Music(body)
        }
        ProviderRequest::ImagesGenerations(body) => {
            let mut body = body.clone();
            body.model = provider_model_id;
            ProviderRequestRewrite::ImagesGenerations(body)
        }
        ProviderRequest::ImagesEdits(body) => {
            let mut body = body.clone();
            body.model = Some(provider_model_id);
            ProviderRequestRewrite::ImagesEdits(body)
        }
        ProviderRequest::ImagesVariations(body) => {
            let mut body = body.clone();
            body.model = Some(provider_model_id);
            ProviderRequestRewrite::ImagesVariations(body)
        }
        ProviderRequest::AudioTranscriptions(body) => {
            let mut body = body.clone();
            body.model = provider_model_id;
            ProviderRequestRewrite::AudioTranscriptions(body)
        }
        ProviderRequest::AudioTranslations(body) => {
            let mut body = body.clone();
            body.model = provider_model_id;
            ProviderRequestRewrite::AudioTranslations(body)
        }
        ProviderRequest::AudioSpeech(body) => {
            let mut body = body.clone();
            body.model = provider_model_id;
            ProviderRequestRewrite::AudioSpeech(body)
        }
        ProviderRequest::FineTuningJobs(body) => {
            let mut body = body.clone();
            body.model = provider_model_id;
            ProviderRequestRewrite::FineTuningJobs(body)
        }
        ProviderRequest::Assistants(body) => {
            let mut body = body.clone();
            body.model = provider_model_id;
            ProviderRequestRewrite::Assistants(body)
        }
        ProviderRequest::AssistantsUpdate(assistant_id, body) => {
            let mut body = body.clone();
            body.model = Some(provider_model_id);
            ProviderRequestRewrite::AssistantsUpdate(assistant_id.to_owned(), body)
        }
        ProviderRequest::RealtimeSessions(body) => {
            let mut body = body.clone();
            body.model = provider_model_id;
            ProviderRequestRewrite::RealtimeSessions(body)
        }
        ProviderRequest::Videos(body) => {
            let mut body = body.clone();
            body.model = provider_model_id;
            ProviderRequestRewrite::Videos(body)
        }
        _ => ProviderRequestRewrite::Borrowed(request),
    })
}

pub(crate) async fn execute_json_provider_request_for_provider(
    store: &dyn AdminStore,
    provider: &ProxyProvider,
    api_key: &str,
    request: ProviderRequest<'_>,
) -> Result<Option<Value>> {
    let options = ProviderRequestOptions::default();
    let retry_policy = gateway_upstream_retry_policy(&request, None);
    execute_json_provider_request_for_provider_with_options(
        store,
        provider,
        api_key,
        request,
        &options,
        retry_policy,
    )
    .await
}

pub(crate) async fn execute_json_provider_request_for_provider_with_options(
    store: &dyn AdminStore,
    provider: &ProxyProvider,
    api_key: &str,
    request: ProviderRequest<'_>,
    options: &ProviderRequestOptions,
    retry_policy: GatewayUpstreamRetryPolicy,
) -> Result<Option<Value>> {
    let descriptor =
        provider_execution_descriptor_for_provider(store, provider, api_key.to_owned()).await?;
    execute_json_provider_request_for_descriptor_with_options(
        store,
        provider,
        &descriptor,
        request,
        options,
        retry_policy,
    )
    .await
}

pub(crate) async fn execute_stream_provider_request_for_provider(
    store: &dyn AdminStore,
    provider: &ProxyProvider,
    api_key: &str,
    request: ProviderRequest<'_>,
) -> Result<Option<ProviderStreamOutput>> {
    let options = ProviderRequestOptions::default();
    let retry_policy = gateway_upstream_retry_policy(&request, None);
    execute_stream_provider_request_for_provider_with_options(
        store,
        provider,
        api_key,
        request,
        &options,
        retry_policy,
    )
    .await
}

pub(crate) async fn execute_stream_provider_request_for_provider_with_options(
    store: &dyn AdminStore,
    provider: &ProxyProvider,
    api_key: &str,
    request: ProviderRequest<'_>,
    options: &ProviderRequestOptions,
    retry_policy: GatewayUpstreamRetryPolicy,
) -> Result<Option<ProviderStreamOutput>> {
    let descriptor =
        provider_execution_descriptor_for_provider(store, provider, api_key.to_owned()).await?;
    execute_stream_provider_request_for_descriptor_with_options(
        store,
        provider,
        &descriptor,
        request,
        options,
        retry_policy,
    )
    .await
}

pub(crate) async fn execute_json_provider_request_for_descriptor_with_options(
    store: &dyn AdminStore,
    provider: &ProxyProvider,
    descriptor: &ProviderExecutionDescriptor,
    request: ProviderRequest<'_>,
    options: &ProviderRequestOptions,
    retry_policy: GatewayUpstreamRetryPolicy,
) -> Result<Option<Value>> {
    debug_assert!(descriptor.local_fallback || descriptor.provider_id == provider.id);
    if descriptor.local_fallback {
        return Ok(None);
    }

    let host = build_extension_host_from_store(store).await?;
    let adapter = resolve_execution_adapter_for_descriptor(&host, descriptor)?;
    let request = rewrite_provider_request_for_execution(store, provider, request).await?;

    let capability = provider_request_metric_capability(&request.as_request());
    let mut attempt = 1usize;

    loop {
        record_gateway_upstream_outcome(capability, &descriptor.provider_id, "attempt");
        match execute_provider_request_with_execution_context(
            adapter.as_ref(),
            Some(&descriptor.provider_id),
            &descriptor.api_key,
            request.as_request(),
            options,
        )
        .await
        {
            Ok(response) => {
                record_gateway_upstream_outcome(capability, &descriptor.provider_id, "success");
                persist_gateway_execution_health_snapshot(
                    store,
                    descriptor,
                    true,
                    capability,
                    None,
                )
                .await;
                return Ok(response.into_json());
            }
            Err(error) => {
                record_gateway_execution_context_failure_from_error(
                    capability,
                    &descriptor.provider_id,
                    &error,
                );
                let retryable = gateway_upstream_error_is_retryable(&error);
                let can_retry =
                    retry_policy.enabled() && retryable && attempt < retry_policy.max_attempts;
                if can_retry {
                    let retry_reason = gateway_retry_reason_for_error(&error);
                    let retry_delay = gateway_retry_delay_for_error(retry_policy, attempt, &error);
                    record_gateway_upstream_retry_with_detail(
                        capability,
                        &descriptor.provider_id,
                        "scheduled",
                        retry_reason,
                        Some(retry_delay.source),
                        Some(retry_delay.delay.as_millis() as u64),
                    );
                    tokio::time::sleep(retry_delay.delay).await;
                    attempt += 1;
                    continue;
                }

                if retry_policy.enabled() && retryable && attempt > 1 {
                    record_gateway_upstream_retry_with_detail(
                        capability,
                        &descriptor.provider_id,
                        "exhausted",
                        gateway_retry_reason_for_error(&error),
                        None,
                        None,
                    );
                }
                record_gateway_upstream_outcome(capability, &descriptor.provider_id, "failure");
                if gateway_error_impacts_provider_health(&error) {
                    persist_gateway_execution_health_snapshot(
                        store,
                        descriptor,
                        false,
                        capability,
                        Some(&error),
                    )
                    .await;
                }
                return Err(error);
            }
        }
    }
}

pub(crate) async fn execute_stream_provider_request_for_descriptor_with_options(
    store: &dyn AdminStore,
    provider: &ProxyProvider,
    descriptor: &ProviderExecutionDescriptor,
    request: ProviderRequest<'_>,
    options: &ProviderRequestOptions,
    retry_policy: GatewayUpstreamRetryPolicy,
) -> Result<Option<ProviderStreamOutput>> {
    debug_assert!(descriptor.local_fallback || descriptor.provider_id == provider.id);
    if descriptor.local_fallback {
        return Ok(None);
    }

    let host = build_extension_host_from_store(store).await?;
    let adapter = resolve_execution_adapter_for_descriptor(&host, descriptor)?;
    let request = rewrite_provider_request_for_execution(store, provider, request).await?;

    let capability = provider_request_metric_capability(&request.as_request());
    let mut attempt = 1usize;

    loop {
        record_gateway_upstream_outcome(capability, &descriptor.provider_id, "attempt");
        match execute_provider_request_with_execution_context(
            adapter.as_ref(),
            Some(&descriptor.provider_id),
            &descriptor.api_key,
            request.as_request(),
            options,
        )
        .await
        {
            Ok(response) => {
                record_gateway_upstream_outcome(capability, &descriptor.provider_id, "success");
                persist_gateway_execution_health_snapshot(
                    store, descriptor, true, capability, None,
                )
                .await;
                return Ok(response.into_stream());
            }
            Err(error) => {
                record_gateway_execution_context_failure_from_error(
                    capability,
                    &descriptor.provider_id,
                    &error,
                );
                let retryable = gateway_upstream_error_is_retryable(&error);
                let can_retry =
                    retry_policy.enabled() && retryable && attempt < retry_policy.max_attempts;
                if can_retry {
                    let retry_reason = gateway_retry_reason_for_error(&error);
                    let retry_delay = gateway_retry_delay_for_error(retry_policy, attempt, &error);
                    record_gateway_upstream_retry_with_detail(
                        capability,
                        &descriptor.provider_id,
                        "scheduled",
                        retry_reason,
                        Some(retry_delay.source),
                        Some(retry_delay.delay.as_millis() as u64),
                    );
                    tokio::time::sleep(retry_delay.delay).await;
                    attempt += 1;
                    continue;
                }

                if retry_policy.enabled() && retryable && attempt > 1 {
                    record_gateway_upstream_retry_with_detail(
                        capability,
                        &descriptor.provider_id,
                        "exhausted",
                        gateway_retry_reason_for_error(&error),
                        None,
                        None,
                    );
                }
                record_gateway_upstream_outcome(capability, &descriptor.provider_id, "failure");
                if gateway_error_impacts_provider_health(&error) {
                    persist_gateway_execution_health_snapshot(
                        store,
                        descriptor,
                        false,
                        capability,
                        Some(&error),
                    )
                    .await;
                }
                return Err(error);
            }
        }
    }
}

pub(crate) fn execute_raw_json_provider_operation(
    runtime_key: &str,
    base_url: String,
    api_key: &str,
    operation: &str,
    path_params: Vec<String>,
    body: Value,
    headers: HashMap<String, String>,
) -> Result<Option<Value>> {
    let host = configured_extension_host()?;
    host.execute_raw_provider_json(
        runtime_key,
        base_url,
        api_key,
        operation,
        path_params,
        body,
        headers,
    )
    .map_err(Into::into)
}

pub(crate) async fn execute_raw_stream_provider_operation(
    runtime_key: &str,
    base_url: String,
    api_key: &str,
    operation: &str,
    path_params: Vec<String>,
    body: Value,
    headers: HashMap<String, String>,
) -> Result<Option<ProviderStreamOutput>> {
    let host = configured_extension_host()?;
    host.execute_raw_provider_stream(
        runtime_key,
        base_url,
        api_key,
        operation,
        path_params,
        body,
        headers,
    )
    .await
    .map_err(Into::into)
}

pub(crate) fn provider_execution_descriptor_from_planned_context(
    planned: &PlannedExecutionProviderContext,
) -> ProviderExecutionDescriptor {
    ProviderExecutionDescriptor {
        provider_id: planned.provider.id.clone(),
        provider_account_id: planned.execution.provider_account_id.clone(),
        execution_instance_id: planned.execution.execution_instance_id.clone(),
        runtime_key: planned.execution.runtime_key.clone(),
        base_url: planned.execution.base_url.clone(),
        api_key: planned.api_key.clone(),
        runtime: planned.execution.runtime.clone(),
        local_fallback: planned.execution.local_fallback,
    }
}

fn resolve_execution_adapter_for_descriptor(
    host: &ExtensionHost,
    descriptor: &ProviderExecutionDescriptor,
) -> Result<Box<dyn ProviderExecutionAdapter>> {
    let Some(adapter) = host.resolve_provider(&descriptor.runtime_key, descriptor.base_url.clone())
    else {
        bail!(
            "selected provider {} is not executable on the provider-adapter surface (runtime_key={}, runtime={})",
            descriptor.provider_id,
            descriptor.runtime_key,
            descriptor.runtime.as_str(),
        );
    };

    Ok(adapter)
}

#[allow(clippy::too_many_arguments)]
async fn execute_raw_json_provider_operation_for_descriptor_with_options(
    store: &dyn AdminStore,
    descriptor: &ProviderExecutionDescriptor,
    capability: &str,
    operation: &str,
    path_params: Vec<String>,
    body: Value,
    headers: HashMap<String, String>,
    options: &ProviderRequestOptions,
    retry_policy: GatewayUpstreamRetryPolicy,
) -> Result<Option<Value>> {
    if descriptor.local_fallback || !descriptor.runtime.supports_raw_provider_execution() {
        return Ok(None);
    }

    let operation = operation.to_owned();

    let mut attempt = 1usize;

    loop {
        let runtime_key = descriptor.runtime_key.clone();
        let base_url = descriptor.base_url.clone();
        let api_key = descriptor.api_key.clone();
        let operation = operation.clone();
        let path_params = path_params.clone();
        let body = body.clone();
        let headers = headers.clone();

        match execute_gateway_blocking_operation_with_execution_context(
            Some(&descriptor.provider_id),
            options,
            move || {
                execute_raw_json_provider_operation(
                    &runtime_key,
                    base_url,
                    &api_key,
                    &operation,
                    path_params,
                    body,
                    headers,
                )
            },
        )
        .await
        {
            Ok(response) => {
                let Some(response) = response else {
                    return Ok(None);
                };
                record_gateway_upstream_outcome(capability, &descriptor.provider_id, "attempt");
                record_gateway_upstream_outcome(capability, &descriptor.provider_id, "success");
                persist_gateway_execution_health_snapshot(
                    store, descriptor, true, capability, None,
                )
                .await;
                return Ok(Some(response));
            }
            Err(error) => {
                record_gateway_upstream_outcome(capability, &descriptor.provider_id, "attempt");
                record_gateway_execution_context_failure_from_error(
                    capability,
                    &descriptor.provider_id,
                    &error,
                );
                let retryable = gateway_upstream_error_is_retryable(&error);
                let can_retry =
                    retry_policy.enabled() && retryable && attempt < retry_policy.max_attempts;
                if can_retry {
                    let retry_reason = gateway_retry_reason_for_error(&error);
                    let retry_delay = gateway_retry_delay_for_error(retry_policy, attempt, &error);
                    record_gateway_upstream_retry_with_detail(
                        capability,
                        &descriptor.provider_id,
                        "scheduled",
                        retry_reason,
                        Some(retry_delay.source),
                        Some(retry_delay.delay.as_millis() as u64),
                    );
                    tokio::time::sleep(retry_delay.delay).await;
                    attempt += 1;
                    continue;
                }

                if retry_policy.enabled() && retryable && attempt > 1 {
                    record_gateway_upstream_retry_with_detail(
                        capability,
                        &descriptor.provider_id,
                        "exhausted",
                        gateway_retry_reason_for_error(&error),
                        None,
                        None,
                    );
                }
                record_gateway_upstream_outcome(capability, &descriptor.provider_id, "failure");
                if gateway_error_impacts_provider_health(&error) {
                    persist_gateway_execution_health_snapshot(
                        store,
                        descriptor,
                        false,
                        capability,
                        Some(&error),
                    )
                    .await;
                }
                return Err(error);
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
async fn execute_raw_stream_provider_operation_for_descriptor_with_options(
    store: &dyn AdminStore,
    descriptor: &ProviderExecutionDescriptor,
    capability: &str,
    operation: &str,
    path_params: Vec<String>,
    body: Value,
    headers: HashMap<String, String>,
    options: &ProviderRequestOptions,
    retry_policy: GatewayUpstreamRetryPolicy,
) -> Result<Option<ProviderStreamOutput>> {
    if descriptor.local_fallback || !descriptor.runtime.supports_raw_provider_execution() {
        return Ok(None);
    }

    let operation = operation.to_owned();
    let mut attempt = 1usize;

    loop {
        let runtime_key = descriptor.runtime_key.clone();
        let base_url = descriptor.base_url.clone();
        let api_key = descriptor.api_key.clone();
        let operation = operation.clone();
        let path_params = path_params.clone();
        let body = body.clone();
        let headers = headers.clone();

        match execute_gateway_future_with_execution_context(
            Some(&descriptor.provider_id),
            options,
            async move {
                execute_raw_stream_provider_operation(
                    &runtime_key,
                    base_url,
                    &api_key,
                    &operation,
                    path_params,
                    body,
                    headers,
                )
                .await
            },
        )
        .await
        {
            Ok(response) => {
                let Some(response) = response else {
                    return Ok(None);
                };
                record_gateway_upstream_outcome(capability, &descriptor.provider_id, "attempt");
                record_gateway_upstream_outcome(capability, &descriptor.provider_id, "success");
                persist_gateway_execution_health_snapshot(
                    store, descriptor, true, capability, None,
                )
                .await;
                return Ok(Some(response));
            }
            Err(error) => {
                record_gateway_upstream_outcome(capability, &descriptor.provider_id, "attempt");
                record_gateway_execution_context_failure_from_error(
                    capability,
                    &descriptor.provider_id,
                    &error,
                );
                let retryable = gateway_upstream_error_is_retryable(&error);
                let can_retry =
                    retry_policy.enabled() && retryable && attempt < retry_policy.max_attempts;
                if can_retry {
                    let retry_reason = gateway_retry_reason_for_error(&error);
                    let retry_delay = gateway_retry_delay_for_error(retry_policy, attempt, &error);
                    record_gateway_upstream_retry_with_detail(
                        capability,
                        &descriptor.provider_id,
                        "scheduled",
                        retry_reason,
                        Some(retry_delay.source),
                        Some(retry_delay.delay.as_millis() as u64),
                    );
                    tokio::time::sleep(retry_delay.delay).await;
                    attempt += 1;
                    continue;
                }

                if retry_policy.enabled() && retryable && attempt > 1 {
                    record_gateway_upstream_retry_with_detail(
                        capability,
                        &descriptor.provider_id,
                        "exhausted",
                        gateway_retry_reason_for_error(&error),
                        None,
                        None,
                    );
                }
                record_gateway_upstream_outcome(capability, &descriptor.provider_id, "failure");
                if gateway_error_impacts_provider_health(&error) {
                    persist_gateway_execution_health_snapshot(
                        store,
                        descriptor,
                        false,
                        capability,
                        Some(&error),
                    )
                    .await;
                }
                return Err(error);
            }
        }
    }
}

const GATEWAY_UPSTREAM_METRICS_SERVICE: &str = "gateway";

pub(crate) fn record_gateway_upstream_outcome(capability: &str, provider_id: &str, outcome: &str) {
    HttpMetricsRegistry::new(GATEWAY_UPSTREAM_METRICS_SERVICE).record_upstream_outcome(
        capability,
        provider_id,
        outcome,
    );
}

pub(crate) fn record_gateway_upstream_retry_with_detail(
    capability: &str,
    provider_id: &str,
    outcome: &str,
    reason: &str,
    delay_source: Option<&str>,
    delay_ms: Option<u64>,
) {
    let registry = HttpMetricsRegistry::new(GATEWAY_UPSTREAM_METRICS_SERVICE);
    registry.record_upstream_retry(capability, provider_id, outcome);
    registry.record_upstream_retry_reason(capability, provider_id, outcome, reason);
    if let (Some(delay_source), Some(delay_ms)) = (delay_source, delay_ms) {
        registry.record_upstream_retry_delay(capability, provider_id, delay_source, delay_ms);
    }
}

pub(crate) fn record_gateway_provider_health(
    provider_id: &str,
    runtime: &str,
    healthy: bool,
    observed_at_ms: u64,
) {
    HttpMetricsRegistry::new(GATEWAY_UPSTREAM_METRICS_SERVICE).record_provider_health(
        provider_id,
        runtime,
        healthy,
        observed_at_ms,
    );
}

pub(crate) fn record_gateway_provider_health_persist_failure(provider_id: &str, runtime: &str) {
    HttpMetricsRegistry::new(GATEWAY_UPSTREAM_METRICS_SERVICE)
        .record_provider_health_persist_failure(provider_id, runtime);
}

pub(crate) fn record_gateway_provider_health_recovery_probe(provider_id: &str, outcome: &str) {
    HttpMetricsRegistry::new(GATEWAY_UPSTREAM_METRICS_SERVICE)
        .record_provider_health_recovery_probe(provider_id, outcome);
}

pub(crate) fn record_gateway_execution_context_failure(
    capability: &str,
    provider_id: &str,
    reason: &str,
) {
    HttpMetricsRegistry::new(GATEWAY_UPSTREAM_METRICS_SERVICE)
        .record_gateway_execution_context_failure(capability, provider_id, reason);
}

pub(crate) fn record_gateway_execution_context_failure_from_error(
    capability: &str,
    provider_id: &str,
    error: &anyhow::Error,
) {
    let Some(error) = gateway_execution_context_error(error) else {
        return;
    };
    record_gateway_execution_context_failure(
        capability,
        provider_id,
        gateway_execution_context_metric_reason(error),
    );
}

pub(crate) fn record_gateway_execution_failover(
    capability: &str,
    from_provider_id: &str,
    to_provider_id: &str,
    outcome: &str,
) {
    HttpMetricsRegistry::new(GATEWAY_UPSTREAM_METRICS_SERVICE).record_gateway_failover(
        capability,
        from_provider_id,
        to_provider_id,
        outcome,
    );
}

pub(crate) fn record_gateway_recovery_probe_from_decision(decision: &RoutingDecision) {
    let Some(probe) = decision.provider_health_recovery_probe.as_ref() else {
        return;
    };
    record_gateway_provider_health_recovery_probe(&probe.provider_id, probe.outcome.as_str());
}

pub(crate) fn provider_request_metric_capability(request: &ProviderRequest<'_>) -> &'static str {
    match request {
        ProviderRequest::ModelsList
        | ProviderRequest::ModelsRetrieve(_)
        | ProviderRequest::ModelsDelete(_) => "models",
        ProviderRequest::ChatCompletions(_)
        | ProviderRequest::ChatCompletionsStream(_)
        | ProviderRequest::ChatCompletionsList
        | ProviderRequest::ChatCompletionsRetrieve(_)
        | ProviderRequest::ChatCompletionsUpdate(_, _)
        | ProviderRequest::ChatCompletionsDelete(_)
        | ProviderRequest::ChatCompletionsMessagesList(_) => "chat_completion",
        ProviderRequest::Completions(_) => "completion",
        ProviderRequest::Containers(_)
        | ProviderRequest::ContainersList
        | ProviderRequest::ContainersRetrieve(_)
        | ProviderRequest::ContainersDelete(_)
        | ProviderRequest::ContainerFiles(_, _)
        | ProviderRequest::ContainerFilesList(_)
        | ProviderRequest::ContainerFilesRetrieve(_, _)
        | ProviderRequest::ContainerFilesDelete(_, _)
        | ProviderRequest::ContainerFilesContent(_, _) => "containers",
        ProviderRequest::Threads(_)
        | ProviderRequest::ThreadsRetrieve(_)
        | ProviderRequest::ThreadsUpdate(_, _)
        | ProviderRequest::ThreadsDelete(_)
        | ProviderRequest::ThreadMessages(_, _)
        | ProviderRequest::ThreadMessagesList(_)
        | ProviderRequest::ThreadMessagesRetrieve(_, _)
        | ProviderRequest::ThreadMessagesUpdate(_, _, _)
        | ProviderRequest::ThreadMessagesDelete(_, _)
        | ProviderRequest::ThreadRuns(_, _)
        | ProviderRequest::ThreadRunsList(_)
        | ProviderRequest::ThreadRunsRetrieve(_, _)
        | ProviderRequest::ThreadRunsUpdate(_, _, _)
        | ProviderRequest::ThreadRunsCancel(_, _)
        | ProviderRequest::ThreadRunsSubmitToolOutputs(_, _, _)
        | ProviderRequest::ThreadRunStepsList(_, _)
        | ProviderRequest::ThreadRunStepsRetrieve(_, _, _)
        | ProviderRequest::ThreadsRuns(_) => "threads",
        ProviderRequest::Conversations(_)
        | ProviderRequest::ConversationsList
        | ProviderRequest::ConversationsRetrieve(_)
        | ProviderRequest::ConversationsUpdate(_, _)
        | ProviderRequest::ConversationsDelete(_)
        | ProviderRequest::ConversationItems(_, _)
        | ProviderRequest::ConversationItemsList(_)
        | ProviderRequest::ConversationItemsRetrieve(_, _)
        | ProviderRequest::ConversationItemsDelete(_, _) => "conversations",
        ProviderRequest::Responses(_)
        | ProviderRequest::ResponsesStream(_)
        | ProviderRequest::ResponsesInputTokens(_)
        | ProviderRequest::ResponsesRetrieve(_)
        | ProviderRequest::ResponsesDelete(_)
        | ProviderRequest::ResponsesInputItemsList(_)
        | ProviderRequest::ResponsesCancel(_)
        | ProviderRequest::ResponsesCompact(_) => "responses",
        ProviderRequest::Embeddings(_) => "embeddings",
        ProviderRequest::Moderations(_) => "moderations",
        ProviderRequest::Music(_)
        | ProviderRequest::MusicList
        | ProviderRequest::MusicRetrieve(_)
        | ProviderRequest::MusicDelete(_)
        | ProviderRequest::MusicContent(_)
        | ProviderRequest::MusicLyrics(_) => "music",
        ProviderRequest::ImagesGenerations(_)
        | ProviderRequest::ImagesEdits(_)
        | ProviderRequest::ImagesVariations(_) => "images",
        ProviderRequest::AudioTranscriptions(_)
        | ProviderRequest::AudioTranslations(_)
        | ProviderRequest::AudioSpeech(_)
        | ProviderRequest::AudioVoicesList
        | ProviderRequest::AudioVoiceConsents(_) => "audio",
        ProviderRequest::Files(_)
        | ProviderRequest::FilesList
        | ProviderRequest::FilesRetrieve(_)
        | ProviderRequest::FilesDelete(_)
        | ProviderRequest::FilesContent(_)
        | ProviderRequest::Uploads(_)
        | ProviderRequest::UploadParts(_)
        | ProviderRequest::UploadComplete(_)
        | ProviderRequest::UploadCancel(_) => "files",
        ProviderRequest::FineTuningJobs(_)
        | ProviderRequest::FineTuningJobsList
        | ProviderRequest::FineTuningJobsRetrieve(_)
        | ProviderRequest::FineTuningJobsCancel(_)
        | ProviderRequest::FineTuningJobsEvents(_)
        | ProviderRequest::FineTuningJobsCheckpoints(_)
        | ProviderRequest::FineTuningJobsPause(_)
        | ProviderRequest::FineTuningJobsResume(_)
        | ProviderRequest::FineTuningCheckpointPermissions(_, _)
        | ProviderRequest::FineTuningCheckpointPermissionsList(_)
        | ProviderRequest::FineTuningCheckpointPermissionsDelete(_, _) => "fine_tuning",
        ProviderRequest::Assistants(_)
        | ProviderRequest::AssistantsList
        | ProviderRequest::AssistantsRetrieve(_)
        | ProviderRequest::AssistantsUpdate(_, _)
        | ProviderRequest::AssistantsDelete(_) => "assistants",
        ProviderRequest::RealtimeSessions(_) => "realtime",
        ProviderRequest::Evals(_)
        | ProviderRequest::EvalsList
        | ProviderRequest::EvalsRetrieve(_)
        | ProviderRequest::EvalsUpdate(_, _)
        | ProviderRequest::EvalsDelete(_)
        | ProviderRequest::EvalRunsList(_)
        | ProviderRequest::EvalRuns(_, _)
        | ProviderRequest::EvalRunsRetrieve(_, _)
        | ProviderRequest::EvalRunsDelete(_, _)
        | ProviderRequest::EvalRunsCancel(_, _)
        | ProviderRequest::EvalRunOutputItemsList(_, _)
        | ProviderRequest::EvalRunOutputItemsRetrieve(_, _, _) => "evals",
        ProviderRequest::Batches(_)
        | ProviderRequest::BatchesList
        | ProviderRequest::BatchesRetrieve(_)
        | ProviderRequest::BatchesCancel(_) => "batches",
        ProviderRequest::VectorStores(_)
        | ProviderRequest::VectorStoresList
        | ProviderRequest::VectorStoresRetrieve(_)
        | ProviderRequest::VectorStoresUpdate(_, _)
        | ProviderRequest::VectorStoresDelete(_)
        | ProviderRequest::VectorStoresSearch(_, _)
        | ProviderRequest::VectorStoreFiles(_, _)
        | ProviderRequest::VectorStoreFilesList(_)
        | ProviderRequest::VectorStoreFilesRetrieve(_, _)
        | ProviderRequest::VectorStoreFilesDelete(_, _)
        | ProviderRequest::VectorStoreFileBatches(_, _)
        | ProviderRequest::VectorStoreFileBatchesRetrieve(_, _)
        | ProviderRequest::VectorStoreFileBatchesCancel(_, _)
        | ProviderRequest::VectorStoreFileBatchesListFiles(_, _) => "vector_stores",
        ProviderRequest::Videos(_)
        | ProviderRequest::VideosList
        | ProviderRequest::VideosRetrieve(_)
        | ProviderRequest::VideosDelete(_)
        | ProviderRequest::VideosContent(_)
        | ProviderRequest::VideosRemix(_, _)
        | ProviderRequest::VideoCharactersCreate(_)
        | ProviderRequest::VideoCharactersList(_)
        | ProviderRequest::VideoCharactersRetrieve(_, _)
        | ProviderRequest::VideoCharactersCanonicalRetrieve(_)
        | ProviderRequest::VideoCharactersUpdate(_, _, _)
        | ProviderRequest::VideosEdits(_)
        | ProviderRequest::VideosExtensions(_)
        | ProviderRequest::VideosExtend(_, _) => "videos",
        ProviderRequest::Webhooks(_)
        | ProviderRequest::WebhooksList
        | ProviderRequest::WebhooksRetrieve(_)
        | ProviderRequest::WebhooksUpdate(_, _)
        | ProviderRequest::WebhooksDelete(_) => "webhooks",
    }
}

pub(crate) async fn execute_json_provider_request(
    runtime_key: &str,
    base_url: String,
    api_key: &str,
    request: ProviderRequest<'_>,
) -> Result<Option<Value>> {
    let options = ProviderRequestOptions::default();
    execute_json_provider_request_with_options(runtime_key, base_url, api_key, request, &options)
        .await
}

pub(crate) async fn execute_json_provider_request_with_options(
    runtime_key: &str,
    base_url: String,
    api_key: &str,
    request: ProviderRequest<'_>,
    options: &ProviderRequestOptions,
) -> Result<Option<Value>> {
    let host = configured_extension_host()?;
    let Some(adapter) = host.resolve_provider(runtime_key, base_url) else {
        return Ok(None);
    };

    let response = execute_provider_request_with_execution_context(
        adapter.as_ref(),
        None,
        api_key,
        request,
        options,
    )
    .await?;
    Ok(response.into_json())
}

pub(crate) async fn execute_stream_provider_request_with_options(
    runtime_key: &str,
    base_url: String,
    api_key: &str,
    request: ProviderRequest<'_>,
    options: &ProviderRequestOptions,
) -> Result<Option<ProviderStreamOutput>> {
    let host = configured_extension_host()?;
    let Some(adapter) = host.resolve_provider(runtime_key, base_url) else {
        return Ok(None);
    };

    let response = execute_provider_request_with_execution_context(
        adapter.as_ref(),
        None,
        api_key,
        request,
        options,
    )
    .await?;
    Ok(response.into_stream())
}

pub async fn execute_json_provider_request_with_runtime(
    runtime_key: &str,
    base_url: impl Into<String>,
    api_key: &str,
    request: ProviderRequest<'_>,
) -> Result<Option<Value>> {
    let options = ProviderRequestOptions::default();
    execute_json_provider_request_with_runtime_and_options(
        runtime_key,
        base_url,
        api_key,
        request,
        &options,
    )
    .await
}

pub async fn execute_json_provider_request_with_runtime_and_options(
    runtime_key: &str,
    base_url: impl Into<String>,
    api_key: &str,
    request: ProviderRequest<'_>,
    options: &ProviderRequestOptions,
) -> Result<Option<Value>> {
    execute_json_provider_request_with_options(
        runtime_key,
        base_url.into(),
        api_key,
        request,
        options,
    )
    .await
}

pub async fn execute_stream_provider_request_with_runtime(
    runtime_key: &str,
    base_url: impl Into<String>,
    api_key: &str,
    request: ProviderRequest<'_>,
) -> Result<Option<ProviderStreamOutput>> {
    let options = ProviderRequestOptions::default();
    execute_stream_provider_request_with_runtime_and_options(
        runtime_key,
        base_url,
        api_key,
        request,
        &options,
    )
    .await
}

pub async fn execute_stream_provider_request_with_runtime_and_options(
    runtime_key: &str,
    base_url: impl Into<String>,
    api_key: &str,
    request: ProviderRequest<'_>,
    options: &ProviderRequestOptions,
) -> Result<Option<ProviderStreamOutput>> {
    execute_stream_provider_request_with_options(
        runtime_key,
        base_url.into(),
        api_key,
        request,
        options,
    )
    .await
}

pub fn execute_raw_json_provider_operation_with_runtime(
    runtime_key: &str,
    base_url: impl Into<String>,
    api_key: &str,
    operation: &str,
    path_params: Vec<String>,
    body: Value,
    headers: HashMap<String, String>,
) -> Result<Option<Value>> {
    execute_raw_json_provider_operation(
        runtime_key,
        base_url.into(),
        api_key,
        operation,
        path_params,
        body,
        headers,
    )
}

pub async fn execute_raw_stream_provider_operation_with_runtime(
    runtime_key: &str,
    base_url: impl Into<String>,
    api_key: &str,
    operation: &str,
    path_params: Vec<String>,
    body: Value,
    headers: HashMap<String, String>,
) -> Result<Option<ProviderStreamOutput>> {
    execute_raw_stream_provider_operation(
        runtime_key,
        base_url.into(),
        api_key,
        operation,
        path_params,
        body,
        headers,
    )
    .await
}

#[allow(clippy::too_many_arguments)]
pub async fn execute_raw_json_provider_operation_from_planned_execution_context_with_options(
    store: &dyn AdminStore,
    planned: &PlannedExecutionProviderContext,
    capability: &str,
    operation: &str,
    path_params: Vec<String>,
    body: Value,
    headers: HashMap<String, String>,
    options: &ProviderRequestOptions,
) -> Result<Option<Value>> {
    let descriptor = provider_execution_descriptor_from_planned_context(planned);
    let retry_policy = gateway_upstream_retry_policy_for_decision_and_capability(
        store,
        &planned.decision,
        capability,
    )
    .await?;
    execute_raw_json_provider_operation_for_descriptor_with_options(
        store,
        &descriptor,
        capability,
        operation,
        path_params,
        body,
        headers,
        options,
        retry_policy,
    )
    .await
}

#[allow(clippy::too_many_arguments)]
pub async fn execute_raw_stream_provider_operation_from_planned_execution_context_with_options(
    store: &dyn AdminStore,
    planned: &PlannedExecutionProviderContext,
    capability: &str,
    operation: &str,
    path_params: Vec<String>,
    body: Value,
    headers: HashMap<String, String>,
    options: &ProviderRequestOptions,
) -> Result<Option<ProviderStreamOutput>> {
    let descriptor = provider_execution_descriptor_from_planned_context(planned);
    let retry_policy = gateway_upstream_retry_policy_for_decision_and_capability(
        store,
        &planned.decision,
        capability,
    )
    .await?;
    execute_raw_stream_provider_operation_for_descriptor_with_options(
        store,
        &descriptor,
        capability,
        operation,
        path_params,
        body,
        headers,
        options,
        retry_policy,
    )
    .await
}

pub(crate) fn normalize_local_speech_format(format: &str) -> Result<&'static str> {
    match format.to_ascii_lowercase().as_str() {
        "wav" => Ok("wav"),
        "mp3" => Ok("mp3"),
        "opus" => Ok("opus"),
        "aac" => Ok("aac"),
        "flac" => Ok("flac"),
        "pcm" => Ok("pcm"),
        _ => bail!("unsupported local speech response_format: {format}"),
    }
}
