use super::*;

impl ProviderAdapter for OpenAiProviderAdapter {
    fn id(&self) -> &'static str {
        "openai"
    }
}

#[async_trait]
impl ProviderExecutionAdapter for OpenAiProviderAdapter {
    async fn execute(&self, api_key: &str, request: ProviderRequest<'_>) -> Result<ProviderOutput> {
        match request {
            ProviderRequest::ModelsList => {
                Ok(ProviderOutput::Json(self.list_models(api_key).await?))
            }
            ProviderRequest::ModelsRetrieve(model_id) => Ok(ProviderOutput::Json(
                self.retrieve_model(api_key, model_id).await?,
            )),
            ProviderRequest::ChatCompletions(request) => Ok(ProviderOutput::Json(
                self.chat_completions(api_key, request).await?,
            )),
            ProviderRequest::ChatCompletionsStream(request) => Ok(ProviderOutput::Stream(
                self.chat_completions_stream(api_key, request).await?,
            )),
            ProviderRequest::ChatCompletionsList => Ok(ProviderOutput::Json(
                self.list_chat_completions(api_key).await?,
            )),
            ProviderRequest::ChatCompletionsRetrieve(completion_id) => Ok(ProviderOutput::Json(
                self.retrieve_chat_completion(api_key, completion_id)
                    .await?,
            )),
            ProviderRequest::ChatCompletionsUpdate(completion_id, request) => {
                Ok(ProviderOutput::Json(
                    self.update_chat_completion(api_key, completion_id, request)
                        .await?,
                ))
            }
            ProviderRequest::ChatCompletionsDelete(completion_id) => Ok(ProviderOutput::Json(
                self.delete_chat_completion(api_key, completion_id).await?,
            )),
            ProviderRequest::ChatCompletionsMessagesList(completion_id) => {
                Ok(ProviderOutput::Json(
                    self.list_chat_completion_messages(api_key, completion_id)
                        .await?,
                ))
            }
            ProviderRequest::Completions(request) => Ok(ProviderOutput::Json(
                self.completions(api_key, request).await?,
            )),
            ProviderRequest::Containers(request) => Ok(ProviderOutput::Json(
                self.containers(api_key, request).await?,
            )),
            ProviderRequest::ContainersList => {
                Ok(ProviderOutput::Json(self.list_containers(api_key).await?))
            }
            ProviderRequest::ContainersRetrieve(container_id) => Ok(ProviderOutput::Json(
                self.retrieve_container(api_key, container_id).await?,
            )),
            ProviderRequest::ContainersDelete(container_id) => Ok(ProviderOutput::Json(
                self.delete_container(api_key, container_id).await?,
            )),
            ProviderRequest::ContainerFiles(container_id, request) => Ok(ProviderOutput::Json(
                self.create_container_file(api_key, container_id, request)
                    .await?,
            )),
            ProviderRequest::ContainerFilesList(container_id) => Ok(ProviderOutput::Json(
                self.list_container_files(api_key, container_id).await?,
            )),
            ProviderRequest::ContainerFilesRetrieve(container_id, file_id) => {
                Ok(ProviderOutput::Json(
                    self.retrieve_container_file(api_key, container_id, file_id)
                        .await?,
                ))
            }
            ProviderRequest::ContainerFilesDelete(container_id, file_id) => {
                Ok(ProviderOutput::Json(
                    self.delete_container_file(api_key, container_id, file_id)
                        .await?,
                ))
            }
            ProviderRequest::ContainerFilesContent(container_id, file_id) => {
                Ok(ProviderOutput::Stream(
                    self.container_file_content(api_key, container_id, file_id)
                        .await?,
                ))
            }
            ProviderRequest::ModelsDelete(model_id) => Ok(ProviderOutput::Json(
                self.delete_model(api_key, model_id).await?,
            )),
            ProviderRequest::Threads(request) => {
                Ok(ProviderOutput::Json(self.threads(api_key, request).await?))
            }
            ProviderRequest::ThreadsRetrieve(thread_id) => Ok(ProviderOutput::Json(
                self.retrieve_thread(api_key, thread_id).await?,
            )),
            ProviderRequest::ThreadsUpdate(thread_id, request) => Ok(ProviderOutput::Json(
                self.update_thread(api_key, thread_id, request).await?,
            )),
            ProviderRequest::ThreadsDelete(thread_id) => Ok(ProviderOutput::Json(
                self.delete_thread(api_key, thread_id).await?,
            )),
            ProviderRequest::ThreadMessages(thread_id, request) => Ok(ProviderOutput::Json(
                self.create_thread_message(api_key, thread_id, request)
                    .await?,
            )),
            ProviderRequest::ThreadMessagesList(thread_id) => Ok(ProviderOutput::Json(
                self.list_thread_messages(api_key, thread_id).await?,
            )),
            ProviderRequest::ThreadMessagesRetrieve(thread_id, message_id) => {
                Ok(ProviderOutput::Json(
                    self.retrieve_thread_message(api_key, thread_id, message_id)
                        .await?,
                ))
            }
            ProviderRequest::ThreadMessagesUpdate(thread_id, message_id, request) => {
                Ok(ProviderOutput::Json(
                    self.update_thread_message(api_key, thread_id, message_id, request)
                        .await?,
                ))
            }
            ProviderRequest::ThreadMessagesDelete(thread_id, message_id) => {
                Ok(ProviderOutput::Json(
                    self.delete_thread_message(api_key, thread_id, message_id)
                        .await?,
                ))
            }
            ProviderRequest::ThreadRuns(thread_id, request) => Ok(ProviderOutput::Json(
                self.create_thread_run(api_key, thread_id, request).await?,
            )),
            ProviderRequest::ThreadRunsList(thread_id) => Ok(ProviderOutput::Json(
                self.list_thread_runs(api_key, thread_id).await?,
            )),
            ProviderRequest::ThreadRunsRetrieve(thread_id, run_id) => Ok(ProviderOutput::Json(
                self.retrieve_thread_run(api_key, thread_id, run_id).await?,
            )),
            ProviderRequest::ThreadRunsUpdate(thread_id, run_id, request) => {
                Ok(ProviderOutput::Json(
                    self.update_thread_run(api_key, thread_id, run_id, request)
                        .await?,
                ))
            }
            ProviderRequest::ThreadRunsCancel(thread_id, run_id) => Ok(ProviderOutput::Json(
                self.cancel_thread_run(api_key, thread_id, run_id).await?,
            )),
            ProviderRequest::ThreadRunsSubmitToolOutputs(thread_id, run_id, request) => {
                Ok(ProviderOutput::Json(
                    self.submit_thread_run_tool_outputs(api_key, thread_id, run_id, request)
                        .await?,
                ))
            }
            ProviderRequest::ThreadRunStepsList(thread_id, run_id) => Ok(ProviderOutput::Json(
                self.list_thread_run_steps(api_key, thread_id, run_id)
                    .await?,
            )),
            ProviderRequest::ThreadRunStepsRetrieve(thread_id, run_id, step_id) => {
                Ok(ProviderOutput::Json(
                    self.retrieve_thread_run_step(api_key, thread_id, run_id, step_id)
                        .await?,
                ))
            }
            ProviderRequest::ThreadsRuns(request) => Ok(ProviderOutput::Json(
                self.create_thread_and_run(api_key, request).await?,
            )),
            ProviderRequest::Conversations(request) => Ok(ProviderOutput::Json(
                self.conversations(api_key, request).await?,
            )),
            ProviderRequest::ConversationsList => Ok(ProviderOutput::Json(
                self.list_conversations(api_key).await?,
            )),
            ProviderRequest::ConversationsRetrieve(conversation_id) => Ok(ProviderOutput::Json(
                self.retrieve_conversation(api_key, conversation_id).await?,
            )),
            ProviderRequest::ConversationsUpdate(conversation_id, request) => {
                Ok(ProviderOutput::Json(
                    self.update_conversation(api_key, conversation_id, request)
                        .await?,
                ))
            }
            ProviderRequest::ConversationsDelete(conversation_id) => Ok(ProviderOutput::Json(
                self.delete_conversation(api_key, conversation_id).await?,
            )),
            ProviderRequest::ConversationItems(conversation_id, request) => {
                Ok(ProviderOutput::Json(
                    self.create_conversation_items(api_key, conversation_id, request)
                        .await?,
                ))
            }
            ProviderRequest::ConversationItemsList(conversation_id) => Ok(ProviderOutput::Json(
                self.list_conversation_items(api_key, conversation_id)
                    .await?,
            )),
            ProviderRequest::ConversationItemsRetrieve(conversation_id, item_id) => {
                Ok(ProviderOutput::Json(
                    self.retrieve_conversation_item(api_key, conversation_id, item_id)
                        .await?,
                ))
            }
            ProviderRequest::ConversationItemsDelete(conversation_id, item_id) => {
                Ok(ProviderOutput::Json(
                    self.delete_conversation_item(api_key, conversation_id, item_id)
                        .await?,
                ))
            }
            ProviderRequest::Responses(request) => Ok(ProviderOutput::Json(
                self.responses(api_key, request).await?,
            )),
            ProviderRequest::ResponsesStream(request) => Ok(ProviderOutput::Stream(
                self.responses_stream(api_key, request).await?,
            )),
            ProviderRequest::ResponsesInputTokens(request) => Ok(ProviderOutput::Json(
                self.count_response_input_tokens(api_key, request).await?,
            )),
            ProviderRequest::ResponsesRetrieve(response_id) => Ok(ProviderOutput::Json(
                self.retrieve_response(api_key, response_id).await?,
            )),
            ProviderRequest::ResponsesDelete(response_id) => Ok(ProviderOutput::Json(
                self.delete_response(api_key, response_id).await?,
            )),
            ProviderRequest::ResponsesInputItemsList(response_id) => Ok(ProviderOutput::Json(
                self.list_response_input_items(api_key, response_id).await?,
            )),
            ProviderRequest::ResponsesCancel(response_id) => Ok(ProviderOutput::Json(
                self.cancel_response(api_key, response_id).await?,
            )),
            ProviderRequest::ResponsesCompact(request) => Ok(ProviderOutput::Json(
                self.compact_response(api_key, request).await?,
            )),
            ProviderRequest::Embeddings(request) => Ok(ProviderOutput::Json(
                self.embeddings(api_key, request).await?,
            )),
            ProviderRequest::Moderations(request) => Ok(ProviderOutput::Json(
                self.moderations(api_key, request).await?,
            )),
            ProviderRequest::Music(request) => {
                Ok(ProviderOutput::Json(self.music(api_key, request).await?))
            }
            ProviderRequest::MusicList => Ok(ProviderOutput::Json(self.list_music(api_key).await?)),
            ProviderRequest::MusicRetrieve(music_id) => Ok(ProviderOutput::Json(
                self.retrieve_music(api_key, music_id).await?,
            )),
            ProviderRequest::MusicDelete(music_id) => Ok(ProviderOutput::Json(
                self.delete_music(api_key, music_id).await?,
            )),
            ProviderRequest::MusicContent(music_id) => Ok(ProviderOutput::Stream(
                self.music_content(api_key, music_id).await?,
            )),
            ProviderRequest::MusicLyrics(request) => Ok(ProviderOutput::Json(
                self.music_lyrics(api_key, request).await?,
            )),
            ProviderRequest::ImagesGenerations(request) => Ok(ProviderOutput::Json(
                self.images_generations(api_key, request).await?,
            )),
            ProviderRequest::ImagesEdits(request) => Ok(ProviderOutput::Json(
                self.images_edits(api_key, request).await?,
            )),
            ProviderRequest::ImagesVariations(request) => Ok(ProviderOutput::Json(
                self.images_variations(api_key, request).await?,
            )),
            ProviderRequest::AudioTranscriptions(request) => Ok(ProviderOutput::Json(
                self.audio_transcriptions(api_key, request).await?,
            )),
            ProviderRequest::AudioTranslations(request) => Ok(ProviderOutput::Json(
                self.audio_translations(api_key, request).await?,
            )),
            ProviderRequest::AudioSpeech(request) => Ok(ProviderOutput::Stream(
                self.audio_speech(api_key, request).await?,
            )),
            ProviderRequest::AudioVoicesList => {
                Ok(ProviderOutput::Json(self.list_audio_voices(api_key).await?))
            }
            ProviderRequest::AudioVoiceConsents(request) => Ok(ProviderOutput::Json(
                self.audio_voice_consents(api_key, request).await?,
            )),
            ProviderRequest::Files(request) => {
                Ok(ProviderOutput::Json(self.files(api_key, request).await?))
            }
            ProviderRequest::FilesList => Ok(ProviderOutput::Json(self.list_files(api_key).await?)),
            ProviderRequest::FilesRetrieve(file_id) => Ok(ProviderOutput::Json(
                self.retrieve_file(api_key, file_id).await?,
            )),
            ProviderRequest::FilesDelete(file_id) => Ok(ProviderOutput::Json(
                self.delete_file(api_key, file_id).await?,
            )),
            ProviderRequest::FilesContent(file_id) => Ok(ProviderOutput::Stream(
                self.file_content(api_key, file_id).await?,
            )),
            ProviderRequest::Uploads(request) => {
                Ok(ProviderOutput::Json(self.uploads(api_key, request).await?))
            }
            ProviderRequest::UploadParts(request) => Ok(ProviderOutput::Json(
                self.upload_parts(api_key, request).await?,
            )),
            ProviderRequest::UploadComplete(request) => Ok(ProviderOutput::Json(
                self.complete_upload(api_key, request).await?,
            )),
            ProviderRequest::UploadCancel(upload_id) => Ok(ProviderOutput::Json(
                self.cancel_upload(api_key, upload_id).await?,
            )),
            ProviderRequest::FineTuningJobs(request) => Ok(ProviderOutput::Json(
                self.fine_tuning_jobs(api_key, request).await?,
            )),
            ProviderRequest::FineTuningJobsList => Ok(ProviderOutput::Json(
                self.list_fine_tuning_jobs(api_key).await?,
            )),
            ProviderRequest::FineTuningJobsRetrieve(job_id) => Ok(ProviderOutput::Json(
                self.retrieve_fine_tuning_job(api_key, job_id).await?,
            )),
            ProviderRequest::FineTuningJobsCancel(job_id) => Ok(ProviderOutput::Json(
                self.cancel_fine_tuning_job(api_key, job_id).await?,
            )),
            ProviderRequest::FineTuningJobsEvents(job_id) => Ok(ProviderOutput::Json(
                self.list_fine_tuning_job_events(api_key, job_id).await?,
            )),
            ProviderRequest::FineTuningJobsCheckpoints(job_id) => Ok(ProviderOutput::Json(
                self.list_fine_tuning_job_checkpoints(api_key, job_id)
                    .await?,
            )),
            ProviderRequest::FineTuningJobsPause(job_id) => Ok(ProviderOutput::Json(
                self.pause_fine_tuning_job(api_key, job_id).await?,
            )),
            ProviderRequest::FineTuningJobsResume(job_id) => Ok(ProviderOutput::Json(
                self.resume_fine_tuning_job(api_key, job_id).await?,
            )),
            ProviderRequest::FineTuningCheckpointPermissions(
                fine_tuned_model_checkpoint,
                request,
            ) => Ok(ProviderOutput::Json(
                self.create_fine_tuning_checkpoint_permissions(
                    api_key,
                    fine_tuned_model_checkpoint,
                    request,
                )
                .await?,
            )),
            ProviderRequest::FineTuningCheckpointPermissionsList(fine_tuned_model_checkpoint) => {
                Ok(ProviderOutput::Json(
                    self.list_fine_tuning_checkpoint_permissions(
                        api_key,
                        fine_tuned_model_checkpoint,
                    )
                    .await?,
                ))
            }
            ProviderRequest::FineTuningCheckpointPermissionsDelete(
                fine_tuned_model_checkpoint,
                permission_id,
            ) => Ok(ProviderOutput::Json(
                self.delete_fine_tuning_checkpoint_permission(
                    api_key,
                    fine_tuned_model_checkpoint,
                    permission_id,
                )
                .await?,
            )),
            ProviderRequest::Assistants(request) => Ok(ProviderOutput::Json(
                self.assistants(api_key, request).await?,
            )),
            ProviderRequest::AssistantsList => {
                Ok(ProviderOutput::Json(self.list_assistants(api_key).await?))
            }
            ProviderRequest::AssistantsRetrieve(assistant_id) => Ok(ProviderOutput::Json(
                self.retrieve_assistant(api_key, assistant_id).await?,
            )),
            ProviderRequest::AssistantsUpdate(assistant_id, request) => Ok(ProviderOutput::Json(
                self.update_assistant(api_key, assistant_id, request)
                    .await?,
            )),
            ProviderRequest::AssistantsDelete(assistant_id) => Ok(ProviderOutput::Json(
                self.delete_assistant(api_key, assistant_id).await?,
            )),
            ProviderRequest::RealtimeSessions(request) => Ok(ProviderOutput::Json(
                self.realtime_sessions(api_key, request).await?,
            )),
            ProviderRequest::Evals(request) => {
                Ok(ProviderOutput::Json(self.evals(api_key, request).await?))
            }
            ProviderRequest::EvalsList => Ok(ProviderOutput::Json(self.list_evals(api_key).await?)),
            ProviderRequest::EvalsRetrieve(eval_id) => Ok(ProviderOutput::Json(
                self.retrieve_eval(api_key, eval_id).await?,
            )),
            ProviderRequest::EvalsUpdate(eval_id, request) => Ok(ProviderOutput::Json(
                self.update_eval(api_key, eval_id, request).await?,
            )),
            ProviderRequest::EvalsDelete(eval_id) => Ok(ProviderOutput::Json(
                self.delete_eval(api_key, eval_id).await?,
            )),
            ProviderRequest::EvalRunsList(eval_id) => Ok(ProviderOutput::Json(
                self.list_eval_runs(api_key, eval_id).await?,
            )),
            ProviderRequest::EvalRuns(eval_id, request) => Ok(ProviderOutput::Json(
                self.create_eval_run(api_key, eval_id, request).await?,
            )),
            ProviderRequest::EvalRunsRetrieve(eval_id, run_id) => Ok(ProviderOutput::Json(
                self.retrieve_eval_run(api_key, eval_id, run_id).await?,
            )),
            ProviderRequest::EvalRunsDelete(eval_id, run_id) => Ok(ProviderOutput::Json(
                self.delete_eval_run(api_key, eval_id, run_id).await?,
            )),
            ProviderRequest::EvalRunsCancel(eval_id, run_id) => Ok(ProviderOutput::Json(
                self.cancel_eval_run(api_key, eval_id, run_id).await?,
            )),
            ProviderRequest::EvalRunOutputItemsList(eval_id, run_id) => Ok(ProviderOutput::Json(
                self.list_eval_run_output_items(api_key, eval_id, run_id)
                    .await?,
            )),
            ProviderRequest::EvalRunOutputItemsRetrieve(eval_id, run_id, output_item_id) => {
                Ok(ProviderOutput::Json(
                    self.retrieve_eval_run_output_item(api_key, eval_id, run_id, output_item_id)
                        .await?,
                ))
            }
            ProviderRequest::Batches(request) => {
                Ok(ProviderOutput::Json(self.batches(api_key, request).await?))
            }
            ProviderRequest::BatchesList => {
                Ok(ProviderOutput::Json(self.list_batches(api_key).await?))
            }
            ProviderRequest::BatchesRetrieve(batch_id) => Ok(ProviderOutput::Json(
                self.retrieve_batch(api_key, batch_id).await?,
            )),
            ProviderRequest::BatchesCancel(batch_id) => Ok(ProviderOutput::Json(
                self.cancel_batch(api_key, batch_id).await?,
            )),
            ProviderRequest::VectorStores(request) => Ok(ProviderOutput::Json(
                self.vector_stores(api_key, request).await?,
            )),
            ProviderRequest::VectorStoresList => Ok(ProviderOutput::Json(
                self.list_vector_stores(api_key).await?,
            )),
            ProviderRequest::VectorStoresRetrieve(vector_store_id) => Ok(ProviderOutput::Json(
                self.retrieve_vector_store(api_key, vector_store_id).await?,
            )),
            ProviderRequest::VectorStoresUpdate(vector_store_id, request) => {
                Ok(ProviderOutput::Json(
                    self.update_vector_store(api_key, vector_store_id, request)
                        .await?,
                ))
            }
            ProviderRequest::VectorStoresDelete(vector_store_id) => Ok(ProviderOutput::Json(
                self.delete_vector_store(api_key, vector_store_id).await?,
            )),
            ProviderRequest::VectorStoresSearch(vector_store_id, request) => {
                Ok(ProviderOutput::Json(
                    self.search_vector_store(api_key, vector_store_id, request)
                        .await?,
                ))
            }
            ProviderRequest::VectorStoreFiles(vector_store_id, request) => {
                Ok(ProviderOutput::Json(
                    self.create_vector_store_file(api_key, vector_store_id, request)
                        .await?,
                ))
            }
            ProviderRequest::VectorStoreFilesList(vector_store_id) => Ok(ProviderOutput::Json(
                self.list_vector_store_files(api_key, vector_store_id)
                    .await?,
            )),
            ProviderRequest::VectorStoreFilesRetrieve(vector_store_id, file_id) => {
                Ok(ProviderOutput::Json(
                    self.retrieve_vector_store_file(api_key, vector_store_id, file_id)
                        .await?,
                ))
            }
            ProviderRequest::VectorStoreFilesDelete(vector_store_id, file_id) => {
                Ok(ProviderOutput::Json(
                    self.delete_vector_store_file(api_key, vector_store_id, file_id)
                        .await?,
                ))
            }
            ProviderRequest::VectorStoreFileBatches(vector_store_id, request) => {
                Ok(ProviderOutput::Json(
                    self.create_vector_store_file_batch(api_key, vector_store_id, request)
                        .await?,
                ))
            }
            ProviderRequest::VectorStoreFileBatchesRetrieve(vector_store_id, batch_id) => {
                Ok(ProviderOutput::Json(
                    self.retrieve_vector_store_file_batch(api_key, vector_store_id, batch_id)
                        .await?,
                ))
            }
            ProviderRequest::VectorStoreFileBatchesCancel(vector_store_id, batch_id) => {
                Ok(ProviderOutput::Json(
                    self.cancel_vector_store_file_batch(api_key, vector_store_id, batch_id)
                        .await?,
                ))
            }
            ProviderRequest::VectorStoreFileBatchesListFiles(vector_store_id, batch_id) => {
                Ok(ProviderOutput::Json(
                    self.list_vector_store_file_batch_files(api_key, vector_store_id, batch_id)
                        .await?,
                ))
            }
            ProviderRequest::Videos(request) => {
                Ok(ProviderOutput::Json(self.videos(api_key, request).await?))
            }
            ProviderRequest::VideosList => {
                Ok(ProviderOutput::Json(self.list_videos(api_key).await?))
            }
            ProviderRequest::VideosRetrieve(video_id) => Ok(ProviderOutput::Json(
                self.retrieve_video(api_key, video_id).await?,
            )),
            ProviderRequest::VideosDelete(video_id) => Ok(ProviderOutput::Json(
                self.delete_video(api_key, video_id).await?,
            )),
            ProviderRequest::VideosContent(video_id) => Ok(ProviderOutput::Stream(
                self.video_content(api_key, video_id).await?,
            )),
            ProviderRequest::VideosRemix(video_id, request) => Ok(ProviderOutput::Json(
                self.remix_video(api_key, video_id, request).await?,
            )),
            ProviderRequest::VideoCharactersCreate(request) => Ok(ProviderOutput::Json(
                self.create_video_character(api_key, request).await?,
            )),
            ProviderRequest::VideoCharactersList(video_id) => Ok(ProviderOutput::Json(
                self.list_video_characters(api_key, video_id).await?,
            )),
            ProviderRequest::VideoCharactersRetrieve(video_id, character_id) => {
                Ok(ProviderOutput::Json(
                    self.retrieve_video_character(api_key, video_id, character_id)
                        .await?,
                ))
            }
            ProviderRequest::VideoCharactersUpdate(video_id, character_id, request) => {
                Ok(ProviderOutput::Json(
                    self.update_video_character(api_key, video_id, character_id, request)
                        .await?,
                ))
            }
            ProviderRequest::VideoCharactersCanonicalRetrieve(character_id) => {
                Ok(ProviderOutput::Json(
                    self.retrieve_video_character_canonical(api_key, character_id)
                        .await?,
                ))
            }
            ProviderRequest::VideosEdits(request) => Ok(ProviderOutput::Json(
                self.edit_video(api_key, request).await?,
            )),
            ProviderRequest::VideosExtensions(request) => Ok(ProviderOutput::Json(
                self.extensions_video(api_key, request).await?,
            )),
            ProviderRequest::VideosExtend(video_id, request) => Ok(ProviderOutput::Json(
                self.extend_video(api_key, video_id, request).await?,
            )),
            ProviderRequest::Webhooks(request) => {
                Ok(ProviderOutput::Json(self.webhooks(api_key, request).await?))
            }
            ProviderRequest::WebhooksList => {
                Ok(ProviderOutput::Json(self.list_webhooks(api_key).await?))
            }
            ProviderRequest::WebhooksRetrieve(webhook_id) => Ok(ProviderOutput::Json(
                self.retrieve_webhook(api_key, webhook_id).await?,
            )),
            ProviderRequest::WebhooksUpdate(webhook_id, request) => Ok(ProviderOutput::Json(
                self.update_webhook(api_key, webhook_id, request).await?,
            )),
            ProviderRequest::WebhooksDelete(webhook_id) => Ok(ProviderOutput::Json(
                self.delete_webhook(api_key, webhook_id).await?,
            )),
        }
    }

    async fn execute_with_options(
        &self,
        api_key: &str,
        request: ProviderRequest<'_>,
        options: &ProviderRequestOptions,
    ) -> Result<ProviderOutput> {
        let adapter = if options.is_empty() {
            self.clone()
        } else {
            let resolved_headers = options.resolved_headers();
            self.clone().with_request_headers(&resolved_headers)
        };
        adapter.execute(api_key, request).await
    }
}
