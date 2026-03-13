use anyhow::Result;
use async_trait::async_trait;
use sdkwork_api_contract_openai::assistants::{CreateAssistantRequest, UpdateAssistantRequest};
use sdkwork_api_contract_openai::audio::{
    CreateSpeechRequest, CreateTranscriptionRequest, CreateTranslationRequest,
};
use sdkwork_api_contract_openai::batches::CreateBatchRequest;
use sdkwork_api_contract_openai::chat_completions::CreateChatCompletionRequest;
use sdkwork_api_contract_openai::completions::CreateCompletionRequest;
use sdkwork_api_contract_openai::embeddings::CreateEmbeddingRequest;
use sdkwork_api_contract_openai::evals::CreateEvalRequest;
use sdkwork_api_contract_openai::files::CreateFileRequest;
use sdkwork_api_contract_openai::fine_tuning::CreateFineTuningJobRequest;
use sdkwork_api_contract_openai::images::CreateImageRequest;
use sdkwork_api_contract_openai::moderations::CreateModerationRequest;
use sdkwork_api_contract_openai::realtime::CreateRealtimeSessionRequest;
use sdkwork_api_contract_openai::responses::CreateResponseRequest;
use sdkwork_api_contract_openai::uploads::{
    AddUploadPartRequest, CompleteUploadRequest, CreateUploadRequest,
};
use sdkwork_api_contract_openai::vector_stores::{
    CreateVectorStoreFileBatchRequest, CreateVectorStoreFileRequest, CreateVectorStoreRequest,
    SearchVectorStoreRequest, UpdateVectorStoreRequest,
};
use sdkwork_api_contract_openai::videos::{CreateVideoRequest, RemixVideoRequest};
use sdkwork_api_contract_openai::webhooks::{CreateWebhookRequest, UpdateWebhookRequest};
use sdkwork_api_domain_catalog::ModelCatalogEntry;
use sdkwork_api_provider_core::{
    ProviderAdapter, ProviderExecutionAdapter, ProviderOutput, ProviderRequest,
};
use sdkwork_api_provider_openai::OpenAiProviderAdapter;
use serde_json::Value;

pub fn adapter_id() -> &'static str {
    "openrouter"
}

pub fn map_model_object(model: &str) -> ModelCatalogEntry {
    ModelCatalogEntry::new(model, "provider-openrouter-main")
}

#[derive(Debug, Clone)]
pub struct OpenRouterProviderAdapter {
    delegate: OpenAiProviderAdapter,
}

impl OpenRouterProviderAdapter {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            delegate: OpenAiProviderAdapter::new(base_url),
        }
    }

    pub async fn chat_completions(
        &self,
        api_key: &str,
        request: &CreateChatCompletionRequest,
    ) -> Result<Value> {
        self.delegate.chat_completions(api_key, request).await
    }

    pub async fn chat_completions_stream(
        &self,
        api_key: &str,
        request: &CreateChatCompletionRequest,
    ) -> Result<reqwest::Response> {
        self.delegate
            .chat_completions_stream(api_key, request)
            .await
    }

    pub async fn responses(&self, api_key: &str, request: &CreateResponseRequest) -> Result<Value> {
        self.delegate.responses(api_key, request).await
    }

    pub async fn retrieve_response(&self, api_key: &str, response_id: &str) -> Result<Value> {
        self.delegate.retrieve_response(api_key, response_id).await
    }

    pub async fn delete_response(&self, api_key: &str, response_id: &str) -> Result<Value> {
        self.delegate.delete_response(api_key, response_id).await
    }

    pub async fn list_response_input_items(
        &self,
        api_key: &str,
        response_id: &str,
    ) -> Result<Value> {
        self.delegate
            .list_response_input_items(api_key, response_id)
            .await
    }

    pub async fn completions(
        &self,
        api_key: &str,
        request: &CreateCompletionRequest,
    ) -> Result<Value> {
        self.delegate.completions(api_key, request).await
    }

    pub async fn embeddings(
        &self,
        api_key: &str,
        request: &CreateEmbeddingRequest,
    ) -> Result<Value> {
        self.delegate.embeddings(api_key, request).await
    }

    pub async fn moderations(
        &self,
        api_key: &str,
        request: &CreateModerationRequest,
    ) -> Result<Value> {
        self.delegate.moderations(api_key, request).await
    }

    pub async fn images_generations(
        &self,
        api_key: &str,
        request: &CreateImageRequest,
    ) -> Result<Value> {
        self.delegate.images_generations(api_key, request).await
    }

    pub async fn audio_transcriptions(
        &self,
        api_key: &str,
        request: &CreateTranscriptionRequest,
    ) -> Result<Value> {
        self.delegate.audio_transcriptions(api_key, request).await
    }

    pub async fn audio_translations(
        &self,
        api_key: &str,
        request: &CreateTranslationRequest,
    ) -> Result<Value> {
        self.delegate.audio_translations(api_key, request).await
    }

    pub async fn audio_speech(
        &self,
        api_key: &str,
        request: &CreateSpeechRequest,
    ) -> Result<reqwest::Response> {
        self.delegate.audio_speech(api_key, request).await
    }

    pub async fn files(&self, api_key: &str, request: &CreateFileRequest) -> Result<Value> {
        self.delegate.files(api_key, request).await
    }

    pub async fn list_files(&self, api_key: &str) -> Result<Value> {
        self.delegate.list_files(api_key).await
    }

    pub async fn retrieve_file(&self, api_key: &str, file_id: &str) -> Result<Value> {
        self.delegate.retrieve_file(api_key, file_id).await
    }

    pub async fn delete_file(&self, api_key: &str, file_id: &str) -> Result<Value> {
        self.delegate.delete_file(api_key, file_id).await
    }

    pub async fn file_content(&self, api_key: &str, file_id: &str) -> Result<reqwest::Response> {
        self.delegate.file_content(api_key, file_id).await
    }

    pub async fn uploads(&self, api_key: &str, request: &CreateUploadRequest) -> Result<Value> {
        self.delegate.uploads(api_key, request).await
    }

    pub async fn upload_parts(
        &self,
        api_key: &str,
        request: &AddUploadPartRequest,
    ) -> Result<Value> {
        self.delegate.upload_parts(api_key, request).await
    }

    pub async fn complete_upload(
        &self,
        api_key: &str,
        request: &CompleteUploadRequest,
    ) -> Result<Value> {
        self.delegate.complete_upload(api_key, request).await
    }

    pub async fn cancel_upload(&self, api_key: &str, upload_id: &str) -> Result<Value> {
        self.delegate.cancel_upload(api_key, upload_id).await
    }

    pub async fn fine_tuning_jobs(
        &self,
        api_key: &str,
        request: &CreateFineTuningJobRequest,
    ) -> Result<Value> {
        self.delegate.fine_tuning_jobs(api_key, request).await
    }

    pub async fn list_fine_tuning_jobs(&self, api_key: &str) -> Result<Value> {
        self.delegate.list_fine_tuning_jobs(api_key).await
    }

    pub async fn retrieve_fine_tuning_job(&self, api_key: &str, job_id: &str) -> Result<Value> {
        self.delegate
            .retrieve_fine_tuning_job(api_key, job_id)
            .await
    }

    pub async fn cancel_fine_tuning_job(&self, api_key: &str, job_id: &str) -> Result<Value> {
        self.delegate.cancel_fine_tuning_job(api_key, job_id).await
    }

    pub async fn assistants(
        &self,
        api_key: &str,
        request: &CreateAssistantRequest,
    ) -> Result<Value> {
        self.delegate.assistants(api_key, request).await
    }

    pub async fn list_assistants(&self, api_key: &str) -> Result<Value> {
        self.delegate.list_assistants(api_key).await
    }

    pub async fn retrieve_assistant(&self, api_key: &str, assistant_id: &str) -> Result<Value> {
        self.delegate
            .retrieve_assistant(api_key, assistant_id)
            .await
    }

    pub async fn update_assistant(
        &self,
        api_key: &str,
        assistant_id: &str,
        request: &UpdateAssistantRequest,
    ) -> Result<Value> {
        self.delegate
            .update_assistant(api_key, assistant_id, request)
            .await
    }

    pub async fn delete_assistant(&self, api_key: &str, assistant_id: &str) -> Result<Value> {
        self.delegate.delete_assistant(api_key, assistant_id).await
    }

    pub async fn webhooks(&self, api_key: &str, request: &CreateWebhookRequest) -> Result<Value> {
        self.delegate.webhooks(api_key, request).await
    }

    pub async fn list_webhooks(&self, api_key: &str) -> Result<Value> {
        self.delegate.list_webhooks(api_key).await
    }

    pub async fn retrieve_webhook(&self, api_key: &str, webhook_id: &str) -> Result<Value> {
        self.delegate.retrieve_webhook(api_key, webhook_id).await
    }

    pub async fn update_webhook(
        &self,
        api_key: &str,
        webhook_id: &str,
        request: &UpdateWebhookRequest,
    ) -> Result<Value> {
        self.delegate
            .update_webhook(api_key, webhook_id, request)
            .await
    }

    pub async fn delete_webhook(&self, api_key: &str, webhook_id: &str) -> Result<Value> {
        self.delegate.delete_webhook(api_key, webhook_id).await
    }

    pub async fn realtime_sessions(
        &self,
        api_key: &str,
        request: &CreateRealtimeSessionRequest,
    ) -> Result<Value> {
        self.delegate.realtime_sessions(api_key, request).await
    }

    pub async fn evals(&self, api_key: &str, request: &CreateEvalRequest) -> Result<Value> {
        self.delegate.evals(api_key, request).await
    }

    pub async fn batches(&self, api_key: &str, request: &CreateBatchRequest) -> Result<Value> {
        self.delegate.batches(api_key, request).await
    }

    pub async fn list_batches(&self, api_key: &str) -> Result<Value> {
        self.delegate.list_batches(api_key).await
    }

    pub async fn retrieve_batch(&self, api_key: &str, batch_id: &str) -> Result<Value> {
        self.delegate.retrieve_batch(api_key, batch_id).await
    }

    pub async fn cancel_batch(&self, api_key: &str, batch_id: &str) -> Result<Value> {
        self.delegate.cancel_batch(api_key, batch_id).await
    }

    pub async fn vector_stores(
        &self,
        api_key: &str,
        request: &CreateVectorStoreRequest,
    ) -> Result<Value> {
        self.delegate.vector_stores(api_key, request).await
    }

    pub async fn list_vector_stores(&self, api_key: &str) -> Result<Value> {
        self.delegate.list_vector_stores(api_key).await
    }

    pub async fn retrieve_vector_store(
        &self,
        api_key: &str,
        vector_store_id: &str,
    ) -> Result<Value> {
        self.delegate
            .retrieve_vector_store(api_key, vector_store_id)
            .await
    }

    pub async fn update_vector_store(
        &self,
        api_key: &str,
        vector_store_id: &str,
        request: &UpdateVectorStoreRequest,
    ) -> Result<Value> {
        self.delegate
            .update_vector_store(api_key, vector_store_id, request)
            .await
    }

    pub async fn delete_vector_store(&self, api_key: &str, vector_store_id: &str) -> Result<Value> {
        self.delegate
            .delete_vector_store(api_key, vector_store_id)
            .await
    }

    pub async fn search_vector_store(
        &self,
        api_key: &str,
        vector_store_id: &str,
        request: &SearchVectorStoreRequest,
    ) -> Result<Value> {
        self.delegate
            .search_vector_store(api_key, vector_store_id, request)
            .await
    }

    pub async fn create_vector_store_file(
        &self,
        api_key: &str,
        vector_store_id: &str,
        request: &CreateVectorStoreFileRequest,
    ) -> Result<Value> {
        self.delegate
            .create_vector_store_file(api_key, vector_store_id, request)
            .await
    }

    pub async fn list_vector_store_files(
        &self,
        api_key: &str,
        vector_store_id: &str,
    ) -> Result<Value> {
        self.delegate
            .list_vector_store_files(api_key, vector_store_id)
            .await
    }

    pub async fn retrieve_vector_store_file(
        &self,
        api_key: &str,
        vector_store_id: &str,
        file_id: &str,
    ) -> Result<Value> {
        self.delegate
            .retrieve_vector_store_file(api_key, vector_store_id, file_id)
            .await
    }

    pub async fn delete_vector_store_file(
        &self,
        api_key: &str,
        vector_store_id: &str,
        file_id: &str,
    ) -> Result<Value> {
        self.delegate
            .delete_vector_store_file(api_key, vector_store_id, file_id)
            .await
    }

    pub async fn create_vector_store_file_batch(
        &self,
        api_key: &str,
        vector_store_id: &str,
        request: &CreateVectorStoreFileBatchRequest,
    ) -> Result<Value> {
        self.delegate
            .create_vector_store_file_batch(api_key, vector_store_id, request)
            .await
    }

    pub async fn retrieve_vector_store_file_batch(
        &self,
        api_key: &str,
        vector_store_id: &str,
        batch_id: &str,
    ) -> Result<Value> {
        self.delegate
            .retrieve_vector_store_file_batch(api_key, vector_store_id, batch_id)
            .await
    }

    pub async fn cancel_vector_store_file_batch(
        &self,
        api_key: &str,
        vector_store_id: &str,
        batch_id: &str,
    ) -> Result<Value> {
        self.delegate
            .cancel_vector_store_file_batch(api_key, vector_store_id, batch_id)
            .await
    }

    pub async fn list_vector_store_file_batch_files(
        &self,
        api_key: &str,
        vector_store_id: &str,
        batch_id: &str,
    ) -> Result<Value> {
        self.delegate
            .list_vector_store_file_batch_files(api_key, vector_store_id, batch_id)
            .await
    }

    pub async fn videos(&self, api_key: &str, request: &CreateVideoRequest) -> Result<Value> {
        self.delegate.videos(api_key, request).await
    }

    pub async fn list_videos(&self, api_key: &str) -> Result<Value> {
        self.delegate.list_videos(api_key).await
    }

    pub async fn retrieve_video(&self, api_key: &str, video_id: &str) -> Result<Value> {
        self.delegate.retrieve_video(api_key, video_id).await
    }

    pub async fn delete_video(&self, api_key: &str, video_id: &str) -> Result<Value> {
        self.delegate.delete_video(api_key, video_id).await
    }

    pub async fn video_content(&self, api_key: &str, video_id: &str) -> Result<reqwest::Response> {
        self.delegate.video_content(api_key, video_id).await
    }

    pub async fn remix_video(
        &self,
        api_key: &str,
        video_id: &str,
        request: &RemixVideoRequest,
    ) -> Result<Value> {
        self.delegate.remix_video(api_key, video_id, request).await
    }
}

impl ProviderAdapter for OpenRouterProviderAdapter {
    fn id(&self) -> &'static str {
        "openrouter"
    }
}

#[async_trait]
impl ProviderExecutionAdapter for OpenRouterProviderAdapter {
    async fn execute(&self, api_key: &str, request: ProviderRequest<'_>) -> Result<ProviderOutput> {
        match request {
            ProviderRequest::ChatCompletions(request) => Ok(ProviderOutput::Json(
                self.chat_completions(api_key, request).await?,
            )),
            ProviderRequest::ChatCompletionsStream(request) => Ok(ProviderOutput::Stream(
                self.chat_completions_stream(api_key, request).await?,
            )),
            ProviderRequest::Completions(request) => Ok(ProviderOutput::Json(
                self.completions(api_key, request).await?,
            )),
            ProviderRequest::Responses(request) => Ok(ProviderOutput::Json(
                self.responses(api_key, request).await?,
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
            ProviderRequest::Embeddings(request) => Ok(ProviderOutput::Json(
                self.embeddings(api_key, request).await?,
            )),
            ProviderRequest::Moderations(request) => Ok(ProviderOutput::Json(
                self.moderations(api_key, request).await?,
            )),
            ProviderRequest::ImagesGenerations(request) => Ok(ProviderOutput::Json(
                self.images_generations(api_key, request).await?,
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
            ProviderRequest::RealtimeSessions(request) => Ok(ProviderOutput::Json(
                self.realtime_sessions(api_key, request).await?,
            )),
            ProviderRequest::Evals(request) => {
                Ok(ProviderOutput::Json(self.evals(api_key, request).await?))
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
        }
    }
}
