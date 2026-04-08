use super::*;

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
    debug_assert!(descriptor.local_fallback || descriptor.provider_id == provider.id);
    if descriptor.local_fallback {
        return Ok(None);
    }

    let host = build_extension_host_from_store(store).await?;
    let Some(adapter) = host.resolve_provider(&descriptor.runtime_key, descriptor.base_url.clone())
    else {
        return Ok(None);
    };

    let capability = provider_request_metric_capability(&request);
    let mut attempt = 1usize;

    loop {
        record_gateway_upstream_outcome(capability, &descriptor.provider_id, "attempt");
        match execute_provider_request_with_execution_context(
            adapter.as_ref(),
            Some(&descriptor.provider_id),
            &descriptor.api_key,
            request,
            options,
        )
        .await
        {
            Ok(response) => {
                record_gateway_upstream_outcome(capability, &descriptor.provider_id, "success");
                persist_gateway_execution_health_snapshot(
                    store,
                    &descriptor,
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
                        &descriptor,
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
    debug_assert!(descriptor.local_fallback || descriptor.provider_id == provider.id);
    if descriptor.local_fallback {
        return Ok(None);
    }

    let host = build_extension_host_from_store(store).await?;
    let Some(adapter) = host.resolve_provider(&descriptor.runtime_key, descriptor.base_url.clone())
    else {
        return Ok(None);
    };

    let capability = provider_request_metric_capability(&request);
    let mut attempt = 1usize;

    loop {
        record_gateway_upstream_outcome(capability, &descriptor.provider_id, "attempt");
        match execute_provider_request_with_execution_context(
            adapter.as_ref(),
            Some(&descriptor.provider_id),
            &descriptor.api_key,
            request,
            options,
        )
        .await
        {
            Ok(response) => {
                record_gateway_upstream_outcome(capability, &descriptor.provider_id, "success");
                persist_gateway_execution_health_snapshot(
                    store,
                    &descriptor,
                    true,
                    capability,
                    None,
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
                        &descriptor,
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

pub(crate) fn fallback_speech_bytes(format: &str) -> Vec<u8> {
    match format {
        "wav" => vec![
            0x52, 0x49, 0x46, 0x46, 0x24, 0x00, 0x00, 0x00, 0x57, 0x41, 0x56, 0x45, 0x66, 0x6d,
            0x74, 0x20, 0x10, 0x00, 0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x40, 0x1f, 0x00, 0x00,
            0x80, 0x3e, 0x00, 0x00, 0x02, 0x00, 0x10, 0x00, 0x64, 0x61, 0x74, 0x61, 0x00, 0x00,
            0x00, 0x00,
        ],
        "mp3" => vec![0x49, 0x44, 0x33, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x21],
        "opus" => b"OggS\x00\x02OpusHead\x01\x01\x00\x00\x00\x00\x00\x00\x00".to_vec(),
        "aac" => vec![0xFF, 0xF1, 0x50, 0x80, 0x00, 0x1F, 0xFC],
        "flac" => b"fLaC\x00\x00\x00\x22".to_vec(),
        "pcm" => vec![0x00, 0x00],
        _ => Vec::new(),
    }
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
