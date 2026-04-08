pub fn try_gateway_router() -> anyhow::Result<Router> {
    try_gateway_router_with_stateless_config(StatelessGatewayConfig::default())
}

pub fn gateway_router() -> Router {
    try_gateway_router().expect("http exposure config should load from process env")
}

pub fn try_gateway_router_with_stateless_config(
    config: StatelessGatewayConfig,
) -> anyhow::Result<Router> {
    Ok(gateway_router_with_stateless_config_and_http_exposure(
        config,
        http_exposure_config()?,
    ))
}

pub fn gateway_router_with_stateless_config(config: StatelessGatewayConfig) -> Router {
    try_gateway_router_with_stateless_config(config)
        .expect("http exposure config should load from process env")
}

pub fn gateway_router_with_stateless_config_and_http_exposure(
    config: StatelessGatewayConfig,
    http_exposure: HttpExposureConfig,
) -> Router {
    let service_name: Arc<str> = Arc::from("gateway");
    let metrics = Arc::new(HttpMetricsRegistry::new("gateway"));
    Router::new()
        .merge(gateway_docs_router())
        .route("/metrics", metrics_route(metrics.clone(), &http_exposure))
        .route("/health", get(|| async { "ok" }))
        .route("/v1/messages", post(anthropic_messages_handler))
        .route(
            "/v1/messages/count_tokens",
            post(anthropic_count_tokens_handler),
        )
        .route("/v1beta/models/{*tail}", post(gemini_models_compat_handler))
        .route("/v1/models", get(list_models_handler))
        .route(
            "/v1/models/{model_id}",
            get(model_retrieve_handler).delete(model_delete_handler),
        )
        .route(
            "/v1/chat/completions",
            get(chat_completions_list_handler).post(chat_completions_handler),
        )
        .route(
            "/v1/chat/completions/{completion_id}",
            get(chat_completion_retrieve_handler)
                .post(chat_completion_update_handler)
                .delete(chat_completion_delete_handler),
        )
        .route(
            "/v1/chat/completions/{completion_id}/messages",
            get(chat_completion_messages_list_handler),
        )
        .route("/v1/completions", post(completions_handler))
        .route(
            "/v1/conversations",
            get(conversations_list_handler).post(conversations_handler),
        )
        .route(
            "/v1/conversations/{conversation_id}",
            get(conversation_retrieve_handler)
                .post(conversation_update_handler)
                .delete(conversation_delete_handler),
        )
        .route(
            "/v1/conversations/{conversation_id}/items",
            get(conversation_items_list_handler).post(conversation_items_handler),
        )
        .route(
            "/v1/conversations/{conversation_id}/items/{item_id}",
            get(conversation_item_retrieve_handler).delete(conversation_item_delete_handler),
        )
        .route("/v1/threads", post(threads_handler))
        .route(
            "/v1/threads/{thread_id}",
            get(thread_retrieve_handler)
                .post(thread_update_handler)
                .delete(thread_delete_handler),
        )
        .route(
            "/v1/threads/{thread_id}/messages",
            get(thread_messages_list_handler).post(thread_messages_handler),
        )
        .route(
            "/v1/threads/{thread_id}/messages/{message_id}",
            get(thread_message_retrieve_handler)
                .post(thread_message_update_handler)
                .delete(thread_message_delete_handler),
        )
        .route("/v1/threads/runs", post(thread_and_run_handler))
        .route(
            "/v1/threads/{thread_id}/runs",
            get(thread_runs_list_handler).post(thread_runs_handler),
        )
        .route(
            "/v1/threads/{thread_id}/runs/{run_id}",
            get(thread_run_retrieve_handler).post(thread_run_update_handler),
        )
        .route(
            "/v1/threads/{thread_id}/runs/{run_id}/cancel",
            post(thread_run_cancel_handler),
        )
        .route(
            "/v1/threads/{thread_id}/runs/{run_id}/submit_tool_outputs",
            post(thread_run_submit_tool_outputs_handler),
        )
        .route(
            "/v1/threads/{thread_id}/runs/{run_id}/steps",
            get(thread_run_steps_list_handler),
        )
        .route(
            "/v1/threads/{thread_id}/runs/{run_id}/steps/{step_id}",
            get(thread_run_step_retrieve_handler),
        )
        .route("/v1/responses", post(responses_handler))
        .route(
            "/v1/responses/input_tokens",
            post(response_input_tokens_handler),
        )
        .route("/v1/responses/compact", post(response_compact_handler))
        .route(
            "/v1/responses/{response_id}",
            get(response_retrieve_handler).delete(response_delete_handler),
        )
        .route(
            "/v1/responses/{response_id}/input_items",
            get(response_input_items_list_handler),
        )
        .route(
            "/v1/responses/{response_id}/cancel",
            post(response_cancel_handler),
        )
        .route("/v1/embeddings", post(embeddings_handler))
        .route("/v1/moderations", post(moderations_handler))
        .route("/v1/images/generations", post(image_generations_handler))
        .route("/v1/images/edits", post(image_edits_handler))
        .route("/v1/images/variations", post(image_variations_handler))
        .route("/v1/audio/transcriptions", post(transcriptions_handler))
        .route("/v1/audio/translations", post(translations_handler))
        .route("/v1/audio/speech", post(audio_speech_handler))
        .route("/v1/audio/voices", get(audio_voices_handler))
        .route(
            "/v1/audio/voice_consents",
            post(audio_voice_consents_handler),
        )
        .route(
            "/v1/containers",
            get(containers_list_handler).post(containers_handler),
        )
        .route(
            "/v1/containers/{container_id}",
            get(container_retrieve_handler).delete(container_delete_handler),
        )
        .route(
            "/v1/containers/{container_id}/files",
            get(container_files_list_handler).post(container_files_handler),
        )
        .route(
            "/v1/containers/{container_id}/files/{file_id}",
            get(container_file_retrieve_handler).delete(container_file_delete_handler),
        )
        .route(
            "/v1/containers/{container_id}/files/{file_id}/content",
            get(container_file_content_handler),
        )
        .route("/v1/files", get(files_list_handler).post(files_handler))
        .route(
            "/v1/files/{file_id}",
            get(file_retrieve_handler).delete(file_delete_handler),
        )
        .route("/v1/files/{file_id}/content", get(file_content_handler))
        .route("/v1/videos", get(videos_list_handler).post(videos_handler))
        .route(
            "/v1/videos/{video_id}",
            get(video_retrieve_handler).delete(video_delete_handler),
        )
        .route("/v1/videos/{video_id}/content", get(video_content_handler))
        .route("/v1/videos/{video_id}/remix", post(video_remix_handler))
        .route(
            "/v1/videos/characters",
            post(video_character_create_handler),
        )
        .route(
            "/v1/videos/characters/{character_id}",
            get(video_character_retrieve_canonical_handler),
        )
        .route("/v1/videos/edits", post(video_edits_handler))
        .route("/v1/videos/extensions", post(video_extensions_handler))
        .route(
            "/v1/videos/{video_id}/characters",
            get(video_characters_list_handler),
        )
        .route(
            "/v1/videos/{video_id}/characters/{character_id}",
            get(video_character_retrieve_handler).post(video_character_update_handler),
        )
        .route("/v1/videos/{video_id}/extend", post(video_extend_handler))
        .route("/v1/music", get(music_list_handler).post(music_handler))
        .route(
            "/v1/music/{music_id}",
            get(music_retrieve_handler).delete(music_delete_handler),
        )
        .route("/v1/music/{music_id}/content", get(music_content_handler))
        .route("/v1/music/lyrics", post(music_lyrics_handler))
        .route("/v1/uploads", post(uploads_handler))
        .route("/v1/uploads/{upload_id}/parts", post(upload_parts_handler))
        .route(
            "/v1/uploads/{upload_id}/complete",
            post(upload_complete_handler),
        )
        .route(
            "/v1/uploads/{upload_id}/cancel",
            post(upload_cancel_handler),
        )
        .route(
            "/v1/fine_tuning/jobs",
            get(fine_tuning_jobs_list_handler).post(fine_tuning_jobs_handler),
        )
        .route(
            "/v1/fine_tuning/jobs/{fine_tuning_job_id}",
            get(fine_tuning_job_retrieve_handler),
        )
        .route(
            "/v1/fine_tuning/jobs/{fine_tuning_job_id}/cancel",
            post(fine_tuning_job_cancel_handler),
        )
        .route(
            "/v1/fine_tuning/jobs/{fine_tuning_job_id}/events",
            get(fine_tuning_job_events_handler),
        )
        .route(
            "/v1/fine_tuning/jobs/{fine_tuning_job_id}/checkpoints",
            get(fine_tuning_job_checkpoints_handler),
        )
        .route(
            "/v1/fine_tuning/jobs/{fine_tuning_job_id}/pause",
            post(fine_tuning_job_pause_handler),
        )
        .route(
            "/v1/fine_tuning/jobs/{fine_tuning_job_id}/resume",
            post(fine_tuning_job_resume_handler),
        )
        .route(
            "/v1/fine_tuning/checkpoints/{fine_tuned_model_checkpoint}/permissions",
            get(fine_tuning_checkpoint_permissions_list_handler)
                .post(fine_tuning_checkpoint_permissions_handler),
        )
        .route(
            "/v1/fine_tuning/checkpoints/{fine_tuned_model_checkpoint}/permissions/{permission_id}",
            axum::routing::delete(fine_tuning_checkpoint_permission_delete_handler),
        )
        .route(
            "/v1/assistants",
            get(assistants_list_handler).post(assistants_handler),
        )
        .route(
            "/v1/assistants/{assistant_id}",
            get(assistant_retrieve_handler)
                .post(assistant_update_handler)
                .delete(assistant_delete_handler),
        )
        .route(
            "/v1/webhooks",
            get(webhooks_list_handler).post(webhooks_handler),
        )
        .route(
            "/v1/webhooks/{webhook_id}",
            get(webhook_retrieve_handler)
                .post(webhook_update_handler)
                .delete(webhook_delete_handler),
        )
        .route("/v1/realtime/sessions", post(realtime_sessions_handler))
        .route("/v1/evals", get(evals_list_handler).post(evals_handler))
        .route(
            "/v1/evals/{eval_id}",
            get(eval_retrieve_handler)
                .post(eval_update_handler)
                .delete(eval_delete_handler),
        )
        .route(
            "/v1/evals/{eval_id}/runs",
            get(eval_runs_list_handler).post(eval_runs_handler),
        )
        .route(
            "/v1/evals/{eval_id}/runs/{run_id}",
            get(eval_run_retrieve_handler).delete(eval_run_delete_handler),
        )
        .route(
            "/v1/evals/{eval_id}/runs/{run_id}/cancel",
            post(eval_run_cancel_handler),
        )
        .route(
            "/v1/evals/{eval_id}/runs/{run_id}/output_items",
            get(eval_run_output_items_list_handler),
        )
        .route(
            "/v1/evals/{eval_id}/runs/{run_id}/output_items/{output_item_id}",
            get(eval_run_output_item_retrieve_handler),
        )
        .route(
            "/v1/batches",
            get(batches_list_handler).post(batches_handler),
        )
        .route("/v1/batches/{batch_id}", get(batch_retrieve_handler))
        .route("/v1/batches/{batch_id}/cancel", post(batch_cancel_handler))
        .route(
            "/v1/vector_stores",
            get(vector_stores_list_handler).post(vector_stores_handler),
        )
        .route(
            "/v1/vector_stores/{vector_store_id}",
            get(vector_store_retrieve_handler)
                .post(vector_store_update_handler)
                .delete(vector_store_delete_handler),
        )
        .route(
            "/v1/vector_stores/{vector_store_id}/search",
            post(vector_store_search_handler),
        )
        .route(
            "/v1/vector_stores/{vector_store_id}/files",
            get(vector_store_files_list_handler).post(vector_store_files_handler),
        )
        .route(
            "/v1/vector_stores/{vector_store_id}/files/{file_id}",
            get(vector_store_file_retrieve_handler).delete(vector_store_file_delete_handler),
        )
        .route(
            "/v1/vector_stores/{vector_store_id}/file_batches",
            post(vector_store_file_batches_handler),
        )
        .route(
            "/v1/vector_stores/{vector_store_id}/file_batches/{batch_id}",
            get(vector_store_file_batch_retrieve_handler),
        )
        .route(
            "/v1/vector_stores/{vector_store_id}/file_batches/{batch_id}/cancel",
            post(vector_store_file_batch_cancel_handler),
        )
        .route(
            "/v1/vector_stores/{vector_store_id}/file_batches/{batch_id}/files",
            get(vector_store_file_batch_files_handler),
        )
        .layer(axum::middleware::from_fn(apply_request_routing_region))
        .layer(axum::middleware::from_fn_with_state(
            metrics,
            observe_http_metrics,
        ))
        .layer(browser_cors_layer(&http_exposure))
        .layer(axum::middleware::from_fn_with_state(
            service_name,
            observe_http_tracing,
        ))
        .with_state(config.into_context())
}

pub fn gateway_router_with_pool(pool: SqlitePool) -> Router {
    gateway_router_with_pool_and_master_key(pool, "local-dev-master-key")
}

pub fn gateway_router_with_store(store: Arc<dyn AdminStore>) -> Router {
    gateway_router_with_store_and_secret_manager(
        store,
        CredentialSecretManager::database_encrypted("local-dev-master-key"),
    )
}

pub fn gateway_router_with_pool_and_master_key(
    pool: SqlitePool,
    credential_master_key: impl Into<String>,
) -> Router {
    gateway_router_with_state(GatewayApiState::with_master_key(
        pool,
        credential_master_key,
    ))
}

pub fn gateway_router_with_pool_and_secret_manager(
    pool: SqlitePool,
    secret_manager: CredentialSecretManager,
) -> Router {
    gateway_router_with_state(GatewayApiState::with_secret_manager(pool, secret_manager))
}

pub fn gateway_router_with_store_and_secret_manager(
    store: Arc<dyn AdminStore>,
    secret_manager: CredentialSecretManager,
) -> Router {
    gateway_router_with_state(GatewayApiState::with_store_and_secret_manager(
        store,
        secret_manager,
    ))
}

pub fn try_gateway_router_with_state(state: GatewayApiState) -> anyhow::Result<Router> {
    Ok(gateway_router_with_state_and_http_exposure(
        state,
        http_exposure_config()?,
    ))
}

pub fn gateway_router_with_state(state: GatewayApiState) -> Router {
    try_gateway_router_with_state(state).expect("http exposure config should load from process env")
}

pub fn gateway_router_with_state_and_http_exposure(
    state: GatewayApiState,
    http_exposure: HttpExposureConfig,
) -> Router {
    let service_name: Arc<str> = Arc::from("gateway");
    let metrics = Arc::new(HttpMetricsRegistry::new("gateway"));
    Router::new()
        .merge(gateway_docs_router())
        .route("/metrics", metrics_route(metrics.clone(), &http_exposure))
        .route("/health", get(|| async { "ok" }))
        .route("/v1/messages", post(anthropic_messages_with_state_handler))
        .route(
            "/v1/messages/count_tokens",
            post(anthropic_count_tokens_with_state_handler),
        )
        .route(
            "/v1beta/models/{*tail}",
            post(gemini_models_compat_with_state_handler),
        )
        .route("/v1/models", get(list_models_from_store_handler))
        .route(
            "/v1/models/{model_id}",
            get(model_retrieve_from_store_handler).delete(model_delete_from_store_handler),
        )
        .route(
            "/v1/chat/completions",
            get(chat_completions_list_with_state_handler).post(chat_completions_with_state_handler),
        )
        .route(
            "/v1/chat/completions/{completion_id}",
            get(chat_completion_retrieve_with_state_handler)
                .post(chat_completion_update_with_state_handler)
                .delete(chat_completion_delete_with_state_handler),
        )
        .route(
            "/v1/chat/completions/{completion_id}/messages",
            get(chat_completion_messages_list_with_state_handler),
        )
        .route("/v1/completions", post(completions_with_state_handler))
        .route(
            "/v1/conversations",
            get(conversations_list_with_state_handler).post(conversations_with_state_handler),
        )
        .route(
            "/v1/conversations/{conversation_id}",
            get(conversation_retrieve_with_state_handler)
                .post(conversation_update_with_state_handler)
                .delete(conversation_delete_with_state_handler),
        )
        .route(
            "/v1/conversations/{conversation_id}/items",
            get(conversation_items_list_with_state_handler)
                .post(conversation_items_with_state_handler),
        )
        .route(
            "/v1/conversations/{conversation_id}/items/{item_id}",
            get(conversation_item_retrieve_with_state_handler)
                .delete(conversation_item_delete_with_state_handler),
        )
        .route("/v1/threads", post(threads_with_state_handler))
        .route(
            "/v1/threads/{thread_id}",
            get(thread_retrieve_with_state_handler)
                .post(thread_update_with_state_handler)
                .delete(thread_delete_with_state_handler),
        )
        .route(
            "/v1/threads/{thread_id}/messages",
            get(thread_messages_list_with_state_handler).post(thread_messages_with_state_handler),
        )
        .route(
            "/v1/threads/{thread_id}/messages/{message_id}",
            get(thread_message_retrieve_with_state_handler)
                .post(thread_message_update_with_state_handler)
                .delete(thread_message_delete_with_state_handler),
        )
        .route("/v1/threads/runs", post(thread_and_run_with_state_handler))
        .route(
            "/v1/threads/{thread_id}/runs",
            get(thread_runs_list_with_state_handler).post(thread_runs_with_state_handler),
        )
        .route(
            "/v1/threads/{thread_id}/runs/{run_id}",
            get(thread_run_retrieve_with_state_handler).post(thread_run_update_with_state_handler),
        )
        .route(
            "/v1/threads/{thread_id}/runs/{run_id}/cancel",
            post(thread_run_cancel_with_state_handler),
        )
        .route(
            "/v1/threads/{thread_id}/runs/{run_id}/submit_tool_outputs",
            post(thread_run_submit_tool_outputs_with_state_handler),
        )
        .route(
            "/v1/threads/{thread_id}/runs/{run_id}/steps",
            get(thread_run_steps_list_with_state_handler),
        )
        .route(
            "/v1/threads/{thread_id}/runs/{run_id}/steps/{step_id}",
            get(thread_run_step_retrieve_with_state_handler),
        )
        .route("/v1/responses", post(responses_with_state_handler))
        .route(
            "/v1/responses/input_tokens",
            post(response_input_tokens_with_state_handler),
        )
        .route(
            "/v1/responses/compact",
            post(response_compact_with_state_handler),
        )
        .route(
            "/v1/responses/{response_id}",
            get(response_retrieve_with_state_handler).delete(response_delete_with_state_handler),
        )
        .route(
            "/v1/responses/{response_id}/input_items",
            get(response_input_items_list_with_state_handler),
        )
        .route(
            "/v1/responses/{response_id}/cancel",
            post(response_cancel_with_state_handler),
        )
        .route("/v1/embeddings", post(embeddings_with_state_handler))
        .route("/v1/moderations", post(moderations_with_state_handler))
        .route(
            "/v1/images/generations",
            post(image_generations_with_state_handler),
        )
        .route("/v1/images/edits", post(image_edits_with_state_handler))
        .route(
            "/v1/images/variations",
            post(image_variations_with_state_handler),
        )
        .route(
            "/v1/audio/transcriptions",
            post(transcriptions_with_state_handler),
        )
        .route(
            "/v1/audio/translations",
            post(translations_with_state_handler),
        )
        .route("/v1/audio/speech", post(audio_speech_with_state_handler))
        .route("/v1/audio/voices", get(audio_voices_with_state_handler))
        .route(
            "/v1/audio/voice_consents",
            post(audio_voice_consents_with_state_handler),
        )
        .route(
            "/v1/containers",
            get(containers_list_with_state_handler).post(containers_with_state_handler),
        )
        .route(
            "/v1/containers/{container_id}",
            get(container_retrieve_with_state_handler).delete(container_delete_with_state_handler),
        )
        .route(
            "/v1/containers/{container_id}/files",
            get(container_files_list_with_state_handler).post(container_files_with_state_handler),
        )
        .route(
            "/v1/containers/{container_id}/files/{file_id}",
            get(container_file_retrieve_with_state_handler)
                .delete(container_file_delete_with_state_handler),
        )
        .route(
            "/v1/containers/{container_id}/files/{file_id}/content",
            get(container_file_content_with_state_handler),
        )
        .route(
            "/v1/files",
            get(files_list_with_state_handler).post(files_with_state_handler),
        )
        .route(
            "/v1/files/{file_id}",
            get(file_retrieve_with_state_handler).delete(file_delete_with_state_handler),
        )
        .route(
            "/v1/files/{file_id}/content",
            get(file_content_with_state_handler),
        )
        .route(
            "/v1/videos",
            get(videos_list_with_state_handler).post(videos_with_state_handler),
        )
        .route(
            "/v1/videos/{video_id}",
            get(video_retrieve_with_state_handler).delete(video_delete_with_state_handler),
        )
        .route(
            "/v1/videos/{video_id}/content",
            get(video_content_with_state_handler),
        )
        .route(
            "/v1/videos/{video_id}/remix",
            post(video_remix_with_state_handler),
        )
        .route(
            "/v1/videos/characters",
            post(video_character_create_with_state_handler),
        )
        .route(
            "/v1/videos/characters/{character_id}",
            get(video_character_retrieve_canonical_with_state_handler),
        )
        .route("/v1/videos/edits", post(video_edits_with_state_handler))
        .route(
            "/v1/videos/extensions",
            post(video_extensions_with_state_handler),
        )
        .route(
            "/v1/videos/{video_id}/characters",
            get(video_characters_list_with_state_handler),
        )
        .route(
            "/v1/videos/{video_id}/characters/{character_id}",
            get(video_character_retrieve_with_state_handler)
                .post(video_character_update_with_state_handler),
        )
        .route(
            "/v1/videos/{video_id}/extend",
            post(video_extend_with_state_handler),
        )
        .route(
            "/v1/music",
            get(music_list_with_state_handler).post(music_with_state_handler),
        )
        .route(
            "/v1/music/{music_id}",
            get(music_retrieve_with_state_handler).delete(music_delete_with_state_handler),
        )
        .route(
            "/v1/music/{music_id}/content",
            get(music_content_with_state_handler),
        )
        .route("/v1/music/lyrics", post(music_lyrics_with_state_handler))
        .route("/v1/uploads", post(uploads_with_state_handler))
        .route(
            "/v1/uploads/{upload_id}/parts",
            post(upload_parts_with_state_handler),
        )
        .route(
            "/v1/uploads/{upload_id}/complete",
            post(upload_complete_with_state_handler),
        )
        .route(
            "/v1/uploads/{upload_id}/cancel",
            post(upload_cancel_with_state_handler),
        )
        .route(
            "/v1/fine_tuning/jobs",
            get(fine_tuning_jobs_list_with_state_handler).post(fine_tuning_jobs_with_state_handler),
        )
        .route(
            "/v1/fine_tuning/jobs/{fine_tuning_job_id}",
            get(fine_tuning_job_retrieve_with_state_handler),
        )
        .route(
            "/v1/fine_tuning/jobs/{fine_tuning_job_id}/cancel",
            post(fine_tuning_job_cancel_with_state_handler),
        )
        .route(
            "/v1/fine_tuning/jobs/{fine_tuning_job_id}/events",
            get(fine_tuning_job_events_with_state_handler),
        )
        .route(
            "/v1/fine_tuning/jobs/{fine_tuning_job_id}/checkpoints",
            get(fine_tuning_job_checkpoints_with_state_handler),
        )
        .route(
            "/v1/fine_tuning/jobs/{fine_tuning_job_id}/pause",
            post(fine_tuning_job_pause_with_state_handler),
        )
        .route(
            "/v1/fine_tuning/jobs/{fine_tuning_job_id}/resume",
            post(fine_tuning_job_resume_with_state_handler),
        )
        .route(
            "/v1/fine_tuning/checkpoints/{fine_tuned_model_checkpoint}/permissions",
            get(fine_tuning_checkpoint_permissions_list_with_state_handler)
                .post(fine_tuning_checkpoint_permissions_with_state_handler),
        )
        .route(
            "/v1/fine_tuning/checkpoints/{fine_tuned_model_checkpoint}/permissions/{permission_id}",
            axum::routing::delete(fine_tuning_checkpoint_permission_delete_with_state_handler),
        )
        .route(
            "/v1/assistants",
            get(assistants_list_with_state_handler).post(assistants_with_state_handler),
        )
        .route(
            "/v1/assistants/{assistant_id}",
            get(assistant_retrieve_with_state_handler)
                .post(assistant_update_with_state_handler)
                .delete(assistant_delete_with_state_handler),
        )
        .route(
            "/v1/webhooks",
            get(webhooks_list_with_state_handler).post(webhooks_with_state_handler),
        )
        .route(
            "/v1/webhooks/{webhook_id}",
            get(webhook_retrieve_with_state_handler)
                .post(webhook_update_with_state_handler)
                .delete(webhook_delete_with_state_handler),
        )
        .route(
            "/v1/realtime/sessions",
            post(realtime_sessions_with_state_handler),
        )
        .route(
            "/v1/evals",
            get(evals_list_with_state_handler).post(evals_with_state_handler),
        )
        .route(
            "/v1/evals/{eval_id}",
            get(eval_retrieve_with_state_handler)
                .post(eval_update_with_state_handler)
                .delete(eval_delete_with_state_handler),
        )
        .route(
            "/v1/evals/{eval_id}/runs",
            get(eval_runs_list_with_state_handler).post(eval_runs_with_state_handler),
        )
        .route(
            "/v1/evals/{eval_id}/runs/{run_id}",
            get(eval_run_retrieve_with_state_handler).delete(eval_run_delete_with_state_handler),
        )
        .route(
            "/v1/evals/{eval_id}/runs/{run_id}/cancel",
            post(eval_run_cancel_with_state_handler),
        )
        .route(
            "/v1/evals/{eval_id}/runs/{run_id}/output_items",
            get(eval_run_output_items_list_with_state_handler),
        )
        .route(
            "/v1/evals/{eval_id}/runs/{run_id}/output_items/{output_item_id}",
            get(eval_run_output_item_retrieve_with_state_handler),
        )
        .route(
            "/v1/batches",
            get(batches_list_with_state_handler).post(batches_with_state_handler),
        )
        .route(
            "/v1/batches/{batch_id}",
            get(batch_retrieve_with_state_handler),
        )
        .route(
            "/v1/batches/{batch_id}/cancel",
            post(batch_cancel_with_state_handler),
        )
        .route(
            "/v1/vector_stores",
            get(vector_stores_list_with_state_handler).post(vector_stores_with_state_handler),
        )
        .route(
            "/v1/vector_stores/{vector_store_id}",
            get(vector_store_retrieve_with_state_handler)
                .post(vector_store_update_with_state_handler)
                .delete(vector_store_delete_with_state_handler),
        )
        .route(
            "/v1/vector_stores/{vector_store_id}/search",
            post(vector_store_search_with_state_handler),
        )
        .route(
            "/v1/vector_stores/{vector_store_id}/files",
            get(vector_store_files_list_with_state_handler)
                .post(vector_store_files_with_state_handler),
        )
        .route(
            "/v1/vector_stores/{vector_store_id}/files/{file_id}",
            get(vector_store_file_retrieve_with_state_handler)
                .delete(vector_store_file_delete_with_state_handler),
        )
        .route(
            "/v1/vector_stores/{vector_store_id}/file_batches",
            post(vector_store_file_batches_with_state_handler),
        )
        .route(
            "/v1/vector_stores/{vector_store_id}/file_batches/{batch_id}",
            get(vector_store_file_batch_retrieve_with_state_handler),
        )
        .route(
            "/v1/vector_stores/{vector_store_id}/file_batches/{batch_id}/cancel",
            post(vector_store_file_batch_cancel_with_state_handler),
        )
        .route(
            "/v1/vector_stores/{vector_store_id}/file_batches/{batch_id}/files",
            get(vector_store_file_batch_files_with_state_handler),
        )
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            apply_gateway_rate_limit,
        ))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            apply_gateway_request_context,
        ))
        .layer(axum::middleware::from_fn(apply_request_routing_region))
        .layer(axum::middleware::from_fn_with_state(
            metrics,
            observe_http_metrics,
        ))
        .layer(browser_cors_layer(&http_exposure))
        .layer(axum::middleware::from_fn_with_state(
            service_name,
            observe_http_tracing,
        ))
        .with_state(state)
}

