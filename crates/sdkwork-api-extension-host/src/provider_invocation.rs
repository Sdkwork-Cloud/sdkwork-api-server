use super::*;

fn serialize_json_body<T: Serialize>(
    body: &T,
    operation: &str,
) -> Result<Value, ExtensionHostError> {
    serde_json::to_value(body).map_err(|error| {
        ExtensionHostError::NativeDynamicInvocationSerializeFailed {
            operation: operation.to_owned(),
            message: error.to_string(),
        }
    })
}

pub(crate) fn provider_invocation_from_request(
    request: ProviderRequest<'_>,
    api_key: &str,
    base_url: &str,
) -> Result<ProviderInvocation, ExtensionHostError> {
    let options = ProviderRequestOptions::default();
    provider_invocation_from_request_with_options(request, api_key, base_url, &options)
}

pub(crate) fn provider_invocation_from_request_with_options(
    request: ProviderRequest<'_>,
    api_key: &str,
    base_url: &str,
    options: &ProviderRequestOptions,
) -> Result<ProviderInvocation, ExtensionHostError> {
    let resolved_headers = options.resolved_headers();

    macro_rules! invocation_with_body {
        ($operation:expr, [$($param:expr),*], $body:expr, $expects_stream:expr) => {
            ProviderInvocation::new(
                $operation,
                api_key,
                base_url,
                vec![$($param.to_owned()),*],
                serialize_json_body($body, $operation)?,
                $expects_stream,
            )
            .with_headers(resolved_headers.clone())
        };
    }

    macro_rules! invocation_without_body {
        ($operation:expr, [$($param:expr),*], $expects_stream:expr) => {
            ProviderInvocation::new(
                $operation,
                api_key,
                base_url,
                vec![$($param.to_owned()),*],
                Value::Null,
                $expects_stream,
            )
            .with_headers(resolved_headers.clone())
        };
    }

    Ok(match request {
        ProviderRequest::ModelsList => invocation_without_body!("models.list", [], false),
        ProviderRequest::ModelsRetrieve(model_id) => {
            invocation_without_body!("models.retrieve", [model_id], false)
        }
        ProviderRequest::ChatCompletions(body) => {
            invocation_with_body!("chat.completions.create", [], body, false)
        }
        ProviderRequest::ChatCompletionsStream(body) => {
            invocation_with_body!("chat.completions.create", [], body, true)
        }
        ProviderRequest::ChatCompletionsList => {
            invocation_without_body!("chat.completions.list", [], false)
        }
        ProviderRequest::ChatCompletionsRetrieve(completion_id) => {
            invocation_without_body!("chat.completions.retrieve", [completion_id], false)
        }
        ProviderRequest::ChatCompletionsUpdate(completion_id, body) => {
            invocation_with_body!("chat.completions.update", [completion_id], body, false)
        }
        ProviderRequest::ChatCompletionsDelete(completion_id) => {
            invocation_without_body!("chat.completions.delete", [completion_id], false)
        }
        ProviderRequest::ChatCompletionsMessagesList(completion_id) => {
            invocation_without_body!("chat.completions.messages.list", [completion_id], false)
        }
        ProviderRequest::Completions(body) => {
            invocation_with_body!("completions.create", [], body, false)
        }
        ProviderRequest::ModelsDelete(model_id) => {
            invocation_without_body!("models.delete", [model_id], false)
        }
        ProviderRequest::Threads(body) => invocation_with_body!("threads.create", [], body, false),
        ProviderRequest::ThreadsRetrieve(thread_id) => {
            invocation_without_body!("threads.retrieve", [thread_id], false)
        }
        ProviderRequest::ThreadsUpdate(thread_id, body) => {
            invocation_with_body!("threads.update", [thread_id], body, false)
        }
        ProviderRequest::ThreadsDelete(thread_id) => {
            invocation_without_body!("threads.delete", [thread_id], false)
        }
        ProviderRequest::ThreadMessages(thread_id, body) => {
            invocation_with_body!("threads.messages.create", [thread_id], body, false)
        }
        ProviderRequest::ThreadMessagesList(thread_id) => {
            invocation_without_body!("threads.messages.list", [thread_id], false)
        }
        ProviderRequest::ThreadMessagesRetrieve(thread_id, message_id) => {
            invocation_without_body!("threads.messages.retrieve", [thread_id, message_id], false)
        }
        ProviderRequest::ThreadMessagesUpdate(thread_id, message_id, body) => {
            invocation_with_body!(
                "threads.messages.update",
                [thread_id, message_id],
                body,
                false
            )
        }
        ProviderRequest::ThreadMessagesDelete(thread_id, message_id) => {
            invocation_without_body!("threads.messages.delete", [thread_id, message_id], false)
        }
        ProviderRequest::ThreadRuns(thread_id, body) => {
            invocation_with_body!("threads.runs.create", [thread_id], body, false)
        }
        ProviderRequest::ThreadRunsList(thread_id) => {
            invocation_without_body!("threads.runs.list", [thread_id], false)
        }
        ProviderRequest::ThreadRunsRetrieve(thread_id, run_id) => {
            invocation_without_body!("threads.runs.retrieve", [thread_id, run_id], false)
        }
        ProviderRequest::ThreadRunsUpdate(thread_id, run_id, body) => {
            invocation_with_body!("threads.runs.update", [thread_id, run_id], body, false)
        }
        ProviderRequest::ThreadRunsCancel(thread_id, run_id) => {
            invocation_without_body!("threads.runs.cancel", [thread_id, run_id], false)
        }
        ProviderRequest::ThreadRunsSubmitToolOutputs(thread_id, run_id, body) => {
            invocation_with_body!(
                "threads.runs.submit_tool_outputs",
                [thread_id, run_id],
                body,
                false
            )
        }
        ProviderRequest::ThreadRunStepsList(thread_id, run_id) => {
            invocation_without_body!("threads.runs.steps.list", [thread_id, run_id], false)
        }
        ProviderRequest::ThreadRunStepsRetrieve(thread_id, run_id, step_id) => {
            invocation_without_body!(
                "threads.runs.steps.retrieve",
                [thread_id, run_id, step_id],
                false
            )
        }
        ProviderRequest::ThreadsRuns(body) => {
            invocation_with_body!("threads.runs.create_on_thread", [], body, false)
        }
        ProviderRequest::Conversations(body) => {
            invocation_with_body!("conversations.create", [], body, false)
        }
        ProviderRequest::ConversationsList => {
            invocation_without_body!("conversations.list", [], false)
        }
        ProviderRequest::ConversationsRetrieve(conversation_id) => {
            invocation_without_body!("conversations.retrieve", [conversation_id], false)
        }
        ProviderRequest::ConversationsUpdate(conversation_id, body) => {
            invocation_with_body!("conversations.update", [conversation_id], body, false)
        }
        ProviderRequest::ConversationsDelete(conversation_id) => {
            invocation_without_body!("conversations.delete", [conversation_id], false)
        }
        ProviderRequest::ConversationItems(conversation_id, body) => {
            invocation_with_body!("conversations.items.create", [conversation_id], body, false)
        }
        ProviderRequest::ConversationItemsList(conversation_id) => {
            invocation_without_body!("conversations.items.list", [conversation_id], false)
        }
        ProviderRequest::ConversationItemsRetrieve(conversation_id, item_id) => {
            invocation_without_body!(
                "conversations.items.retrieve",
                [conversation_id, item_id],
                false
            )
        }
        ProviderRequest::ConversationItemsDelete(conversation_id, item_id) => {
            invocation_without_body!(
                "conversations.items.delete",
                [conversation_id, item_id],
                false
            )
        }
        ProviderRequest::Responses(body) => {
            invocation_with_body!("responses.create", [], body, false)
        }
        ProviderRequest::ResponsesStream(body) => {
            invocation_with_body!("responses.create", [], body, true)
        }
        ProviderRequest::ResponsesInputTokens(body) => {
            invocation_with_body!("responses.input_tokens.count", [], body, false)
        }
        ProviderRequest::ResponsesRetrieve(response_id) => {
            invocation_without_body!("responses.retrieve", [response_id], false)
        }
        ProviderRequest::ResponsesDelete(response_id) => {
            invocation_without_body!("responses.delete", [response_id], false)
        }
        ProviderRequest::ResponsesInputItemsList(response_id) => {
            invocation_without_body!("responses.input_items.list", [response_id], false)
        }
        ProviderRequest::ResponsesCancel(response_id) => {
            invocation_without_body!("responses.cancel", [response_id], false)
        }
        ProviderRequest::ResponsesCompact(body) => {
            invocation_with_body!("responses.compact", [], body, false)
        }
        ProviderRequest::Embeddings(body) => {
            invocation_with_body!("embeddings.create", [], body, false)
        }
        ProviderRequest::Moderations(body) => {
            invocation_with_body!("moderations.create", [], body, false)
        }
        ProviderRequest::Music(body) => {
            invocation_with_body!("music.create", [], body, false)
        }
        ProviderRequest::MusicList => invocation_without_body!("music.list", [], false),
        ProviderRequest::MusicRetrieve(music_id) => {
            invocation_without_body!("music.retrieve", [music_id], false)
        }
        ProviderRequest::MusicDelete(music_id) => {
            invocation_without_body!("music.delete", [music_id], false)
        }
        ProviderRequest::MusicContent(music_id) => {
            invocation_without_body!("music.content", [music_id], true)
        }
        ProviderRequest::MusicLyrics(body) => {
            invocation_with_body!("music.lyrics.create", [], body, false)
        }
        ProviderRequest::ImagesGenerations(body) => {
            invocation_with_body!("images.generate", [], body, false)
        }
        ProviderRequest::ImagesEdits(body) => {
            invocation_with_body!("images.edit", [], body, false)
        }
        ProviderRequest::ImagesVariations(body) => {
            invocation_with_body!("images.variation", [], body, false)
        }
        ProviderRequest::AudioTranscriptions(body) => {
            invocation_with_body!("audio.transcriptions.create", [], body, false)
        }
        ProviderRequest::AudioTranslations(body) => {
            invocation_with_body!("audio.translations.create", [], body, false)
        }
        ProviderRequest::AudioSpeech(body) => {
            invocation_with_body!("audio.speech.create", [], body, true)
        }
        ProviderRequest::AudioVoicesList => {
            invocation_without_body!("audio.voices.list", [], false)
        }
        ProviderRequest::AudioVoiceConsents(body) => {
            invocation_with_body!("audio.voice_consents.create", [], body, false)
        }
        ProviderRequest::Containers(body) => {
            invocation_with_body!("containers.create", [], body, false)
        }
        ProviderRequest::ContainersList => invocation_without_body!("containers.list", [], false),
        ProviderRequest::ContainersRetrieve(container_id) => {
            invocation_without_body!("containers.retrieve", [container_id], false)
        }
        ProviderRequest::ContainersDelete(container_id) => {
            invocation_without_body!("containers.delete", [container_id], false)
        }
        ProviderRequest::ContainerFiles(container_id, body) => {
            invocation_with_body!("containers.files.create", [container_id], body, false)
        }
        ProviderRequest::ContainerFilesList(container_id) => {
            invocation_without_body!("containers.files.list", [container_id], false)
        }
        ProviderRequest::ContainerFilesRetrieve(container_id, file_id) => {
            invocation_without_body!("containers.files.retrieve", [container_id, file_id], false)
        }
        ProviderRequest::ContainerFilesDelete(container_id, file_id) => {
            invocation_without_body!("containers.files.delete", [container_id, file_id], false)
        }
        ProviderRequest::ContainerFilesContent(container_id, file_id) => {
            invocation_without_body!(
                "containers.files.content.retrieve",
                [container_id, file_id],
                true
            )
        }
        ProviderRequest::Files(body) => invocation_with_body!("files.create", [], body, false),
        ProviderRequest::FilesList => invocation_without_body!("files.list", [], false),
        ProviderRequest::FilesRetrieve(file_id) => {
            invocation_without_body!("files.retrieve", [file_id], false)
        }
        ProviderRequest::FilesDelete(file_id) => {
            invocation_without_body!("files.delete", [file_id], false)
        }
        ProviderRequest::FilesContent(file_id) => {
            invocation_without_body!("files.content", [file_id], true)
        }
        ProviderRequest::Uploads(body) => invocation_with_body!("uploads.create", [], body, false),
        ProviderRequest::UploadParts(body) => {
            invocation_with_body!("uploads.parts.create", [], body, false)
        }
        ProviderRequest::UploadComplete(body) => {
            invocation_with_body!("uploads.complete", [&body.upload_id], body, false)
        }
        ProviderRequest::UploadCancel(upload_id) => {
            invocation_without_body!("uploads.cancel", [upload_id], false)
        }
        ProviderRequest::FineTuningJobs(body) => {
            invocation_with_body!("fine_tuning.jobs.create", [], body, false)
        }
        ProviderRequest::FineTuningJobsList => {
            invocation_without_body!("fine_tuning.jobs.list", [], false)
        }
        ProviderRequest::FineTuningJobsRetrieve(job_id) => {
            invocation_without_body!("fine_tuning.jobs.retrieve", [job_id], false)
        }
        ProviderRequest::FineTuningJobsCancel(job_id) => {
            invocation_without_body!("fine_tuning.jobs.cancel", [job_id], false)
        }
        ProviderRequest::FineTuningJobsEvents(job_id) => {
            invocation_without_body!("fine_tuning.jobs.events.list", [job_id], false)
        }
        ProviderRequest::FineTuningJobsCheckpoints(job_id) => {
            invocation_without_body!("fine_tuning.jobs.checkpoints.list", [job_id], false)
        }
        ProviderRequest::FineTuningJobsPause(job_id) => {
            invocation_without_body!("fine_tuning.jobs.pause", [job_id], false)
        }
        ProviderRequest::FineTuningJobsResume(job_id) => {
            invocation_without_body!("fine_tuning.jobs.resume", [job_id], false)
        }
        ProviderRequest::FineTuningCheckpointPermissions(checkpoint_id, body) => {
            invocation_with_body!(
                "fine_tuning.checkpoints.permissions.create",
                [checkpoint_id],
                body,
                false
            )
        }
        ProviderRequest::FineTuningCheckpointPermissionsList(checkpoint_id) => {
            invocation_without_body!(
                "fine_tuning.checkpoints.permissions.list",
                [checkpoint_id],
                false
            )
        }
        ProviderRequest::FineTuningCheckpointPermissionsDelete(checkpoint_id, permission_id) => {
            invocation_without_body!(
                "fine_tuning.checkpoints.permissions.delete",
                [checkpoint_id, permission_id],
                false
            )
        }
        ProviderRequest::Assistants(body) => {
            invocation_with_body!("assistants.create", [], body, false)
        }
        ProviderRequest::AssistantsList => {
            invocation_without_body!("assistants.list", [], false)
        }
        ProviderRequest::AssistantsRetrieve(assistant_id) => {
            invocation_without_body!("assistants.retrieve", [assistant_id], false)
        }
        ProviderRequest::AssistantsUpdate(assistant_id, body) => {
            invocation_with_body!("assistants.update", [assistant_id], body, false)
        }
        ProviderRequest::AssistantsDelete(assistant_id) => {
            invocation_without_body!("assistants.delete", [assistant_id], false)
        }
        ProviderRequest::RealtimeSessions(body) => {
            invocation_with_body!("realtime.sessions.create", [], body, false)
        }
        ProviderRequest::Evals(body) => invocation_with_body!("evals.create", [], body, false),
        ProviderRequest::EvalsList => invocation_without_body!("evals.list", [], false),
        ProviderRequest::EvalsRetrieve(eval_id) => {
            invocation_without_body!("evals.retrieve", [eval_id], false)
        }
        ProviderRequest::EvalsUpdate(eval_id, body) => {
            invocation_with_body!("evals.update", [eval_id], body, false)
        }
        ProviderRequest::EvalsDelete(eval_id) => {
            invocation_without_body!("evals.delete", [eval_id], false)
        }
        ProviderRequest::EvalRunsList(eval_id) => {
            invocation_without_body!("evals.runs.list", [eval_id], false)
        }
        ProviderRequest::EvalRuns(eval_id, body) => {
            invocation_with_body!("evals.runs.create", [eval_id], body, false)
        }
        ProviderRequest::EvalRunsRetrieve(eval_id, run_id) => {
            invocation_without_body!("evals.runs.retrieve", [eval_id, run_id], false)
        }
        ProviderRequest::EvalRunsDelete(eval_id, run_id) => {
            invocation_without_body!("evals.runs.delete", [eval_id, run_id], false)
        }
        ProviderRequest::EvalRunsCancel(eval_id, run_id) => {
            invocation_without_body!("evals.runs.cancel", [eval_id, run_id], false)
        }
        ProviderRequest::EvalRunOutputItemsList(eval_id, run_id) => {
            invocation_without_body!("evals.runs.output_items.list", [eval_id, run_id], false)
        }
        ProviderRequest::EvalRunOutputItemsRetrieve(eval_id, run_id, output_item_id) => {
            invocation_without_body!(
                "evals.runs.output_items.retrieve",
                [eval_id, run_id, output_item_id],
                false
            )
        }
        ProviderRequest::Batches(body) => invocation_with_body!("batches.create", [], body, false),
        ProviderRequest::BatchesList => invocation_without_body!("batches.list", [], false),
        ProviderRequest::BatchesRetrieve(batch_id) => {
            invocation_without_body!("batches.retrieve", [batch_id], false)
        }
        ProviderRequest::BatchesCancel(batch_id) => {
            invocation_without_body!("batches.cancel", [batch_id], false)
        }
        ProviderRequest::VectorStores(body) => {
            invocation_with_body!("vector_stores.create", [], body, false)
        }
        ProviderRequest::VectorStoresList => {
            invocation_without_body!("vector_stores.list", [], false)
        }
        ProviderRequest::VectorStoresRetrieve(vector_store_id) => {
            invocation_without_body!("vector_stores.retrieve", [vector_store_id], false)
        }
        ProviderRequest::VectorStoresUpdate(vector_store_id, body) => {
            invocation_with_body!("vector_stores.update", [vector_store_id], body, false)
        }
        ProviderRequest::VectorStoresDelete(vector_store_id) => {
            invocation_without_body!("vector_stores.delete", [vector_store_id], false)
        }
        ProviderRequest::VectorStoresSearch(vector_store_id, body) => {
            invocation_with_body!("vector_stores.search", [vector_store_id], body, false)
        }
        ProviderRequest::VectorStoreFiles(vector_store_id, body) => {
            invocation_with_body!("vector_stores.files.create", [vector_store_id], body, false)
        }
        ProviderRequest::VectorStoreFilesList(vector_store_id) => {
            invocation_without_body!("vector_stores.files.list", [vector_store_id], false)
        }
        ProviderRequest::VectorStoreFilesRetrieve(vector_store_id, file_id) => {
            invocation_without_body!(
                "vector_stores.files.retrieve",
                [vector_store_id, file_id],
                false
            )
        }
        ProviderRequest::VectorStoreFilesDelete(vector_store_id, file_id) => {
            invocation_without_body!(
                "vector_stores.files.delete",
                [vector_store_id, file_id],
                false
            )
        }
        ProviderRequest::VectorStoreFileBatches(vector_store_id, body) => {
            invocation_with_body!(
                "vector_stores.file_batches.create",
                [vector_store_id],
                body,
                false
            )
        }
        ProviderRequest::VectorStoreFileBatchesRetrieve(vector_store_id, batch_id) => {
            invocation_without_body!(
                "vector_stores.file_batches.retrieve",
                [vector_store_id, batch_id],
                false
            )
        }
        ProviderRequest::VectorStoreFileBatchesCancel(vector_store_id, batch_id) => {
            invocation_without_body!(
                "vector_stores.file_batches.cancel",
                [vector_store_id, batch_id],
                false
            )
        }
        ProviderRequest::VectorStoreFileBatchesListFiles(vector_store_id, batch_id) => {
            invocation_without_body!(
                "vector_stores.file_batches.files.list",
                [vector_store_id, batch_id],
                false
            )
        }
        ProviderRequest::Videos(body) => invocation_with_body!("videos.create", [], body, false),
        ProviderRequest::VideosList => invocation_without_body!("videos.list", [], false),
        ProviderRequest::VideosRetrieve(video_id) => {
            invocation_without_body!("videos.retrieve", [video_id], false)
        }
        ProviderRequest::VideosDelete(video_id) => {
            invocation_without_body!("videos.delete", [video_id], false)
        }
        ProviderRequest::VideosContent(video_id) => {
            invocation_without_body!("videos.content", [video_id], true)
        }
        ProviderRequest::VideosRemix(video_id, body) => {
            invocation_with_body!("videos.remix", [video_id], body, false)
        }
        ProviderRequest::VideoCharactersCreate(body) => {
            invocation_with_body!("videos.characters.create", [], body, false)
        }
        ProviderRequest::VideoCharactersList(video_id) => {
            invocation_without_body!("videos.characters.list", [video_id], false)
        }
        ProviderRequest::VideoCharactersRetrieve(video_id, character_id) => {
            invocation_without_body!(
                "videos.characters.retrieve",
                [video_id, character_id],
                false
            )
        }
        ProviderRequest::VideoCharactersUpdate(video_id, character_id, body) => {
            invocation_with_body!(
                "videos.characters.update",
                [video_id, character_id],
                body,
                false
            )
        }
        ProviderRequest::VideoCharactersCanonicalRetrieve(character_id) => {
            invocation_without_body!("videos.characters.retrieve", [character_id], false)
        }
        ProviderRequest::VideosEdits(body) => {
            invocation_with_body!("videos.edits.create", [], body, false)
        }
        ProviderRequest::VideosExtensions(body) => {
            invocation_with_body!("videos.extensions.create", [], body, false)
        }
        ProviderRequest::VideosExtend(video_id, body) => {
            invocation_with_body!("videos.extend", [video_id], body, false)
        }
        ProviderRequest::Webhooks(body) => {
            invocation_with_body!("webhooks.create", [], body, false)
        }
        ProviderRequest::WebhooksList => invocation_without_body!("webhooks.list", [], false),
        ProviderRequest::WebhooksRetrieve(webhook_id) => {
            invocation_without_body!("webhooks.retrieve", [webhook_id], false)
        }
        ProviderRequest::WebhooksUpdate(webhook_id, body) => {
            invocation_with_body!("webhooks.update", [webhook_id], body, false)
        }
        ProviderRequest::WebhooksDelete(webhook_id) => {
            invocation_without_body!("webhooks.delete", [webhook_id], false)
        }
    })
}
