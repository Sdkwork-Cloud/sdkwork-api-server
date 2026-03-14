use anyhow::Result;
use async_trait::async_trait;
use sdkwork_api_contract_openai::assistants::{CreateAssistantRequest, UpdateAssistantRequest};
use sdkwork_api_contract_openai::audio::{
    CreateSpeechRequest, CreateTranscriptionRequest, CreateTranslationRequest,
};
use sdkwork_api_contract_openai::batches::CreateBatchRequest;
use sdkwork_api_contract_openai::chat_completions::{
    CreateChatCompletionRequest, UpdateChatCompletionRequest,
};
use sdkwork_api_contract_openai::completions::CreateCompletionRequest;
use sdkwork_api_contract_openai::conversations::{
    CreateConversationItemsRequest, CreateConversationRequest, UpdateConversationRequest,
};
use sdkwork_api_contract_openai::embeddings::CreateEmbeddingRequest;
use sdkwork_api_contract_openai::evals::CreateEvalRequest;
use sdkwork_api_contract_openai::files::CreateFileRequest;
use sdkwork_api_contract_openai::fine_tuning::CreateFineTuningJobRequest;
use sdkwork_api_contract_openai::images::{
    CreateImageEditRequest, CreateImageRequest, CreateImageVariationRequest,
};
use sdkwork_api_contract_openai::moderations::CreateModerationRequest;
use sdkwork_api_contract_openai::realtime::CreateRealtimeSessionRequest;
use sdkwork_api_contract_openai::responses::{
    CompactResponseRequest, CountResponseInputTokensRequest, CreateResponseRequest,
};
use sdkwork_api_contract_openai::threads::{
    CreateThreadMessageRequest, CreateThreadRequest, UpdateThreadMessageRequest,
    UpdateThreadRequest,
};
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
    ProviderStreamOutput,
};
use sdkwork_api_provider_openai::OpenAiProviderAdapter;
use serde_json::Value;

pub fn adapter_id() -> &'static str {
    "ollama"
}

pub fn map_model_object(model: &str) -> ModelCatalogEntry {
    ModelCatalogEntry::new(model, "provider-ollama-local")
}

#[derive(Debug, Clone)]
pub struct OllamaProviderAdapter {
    delegate: OpenAiProviderAdapter,
}

impl OllamaProviderAdapter {
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

    pub async fn list_chat_completions(&self, api_key: &str) -> Result<Value> {
        self.delegate.list_chat_completions(api_key).await
    }

    pub async fn list_models(&self, api_key: &str) -> Result<Value> {
        self.delegate.list_models(api_key).await
    }

    pub async fn retrieve_model(&self, api_key: &str, model_id: &str) -> Result<Value> {
        self.delegate.retrieve_model(api_key, model_id).await
    }

    pub async fn retrieve_chat_completion(
        &self,
        api_key: &str,
        completion_id: &str,
    ) -> Result<Value> {
        self.delegate
            .retrieve_chat_completion(api_key, completion_id)
            .await
    }

    pub async fn update_chat_completion(
        &self,
        api_key: &str,
        completion_id: &str,
        request: &UpdateChatCompletionRequest,
    ) -> Result<Value> {
        self.delegate
            .update_chat_completion(api_key, completion_id, request)
            .await
    }

    pub async fn delete_chat_completion(
        &self,
        api_key: &str,
        completion_id: &str,
    ) -> Result<Value> {
        self.delegate
            .delete_chat_completion(api_key, completion_id)
            .await
    }

    pub async fn list_chat_completion_messages(
        &self,
        api_key: &str,
        completion_id: &str,
    ) -> Result<Value> {
        self.delegate
            .list_chat_completion_messages(api_key, completion_id)
            .await
    }

    pub async fn conversations(
        &self,
        api_key: &str,
        request: &CreateConversationRequest,
    ) -> Result<Value> {
        self.delegate.conversations(api_key, request).await
    }

    pub async fn list_conversations(&self, api_key: &str) -> Result<Value> {
        self.delegate.list_conversations(api_key).await
    }

    pub async fn retrieve_conversation(
        &self,
        api_key: &str,
        conversation_id: &str,
    ) -> Result<Value> {
        self.delegate
            .retrieve_conversation(api_key, conversation_id)
            .await
    }

    pub async fn update_conversation(
        &self,
        api_key: &str,
        conversation_id: &str,
        request: &UpdateConversationRequest,
    ) -> Result<Value> {
        self.delegate
            .update_conversation(api_key, conversation_id, request)
            .await
    }

    pub async fn delete_conversation(&self, api_key: &str, conversation_id: &str) -> Result<Value> {
        self.delegate
            .delete_conversation(api_key, conversation_id)
            .await
    }

    pub async fn create_conversation_items(
        &self,
        api_key: &str,
        conversation_id: &str,
        request: &CreateConversationItemsRequest,
    ) -> Result<Value> {
        self.delegate
            .create_conversation_items(api_key, conversation_id, request)
            .await
    }

    pub async fn list_conversation_items(
        &self,
        api_key: &str,
        conversation_id: &str,
    ) -> Result<Value> {
        self.delegate
            .list_conversation_items(api_key, conversation_id)
            .await
    }

    pub async fn retrieve_conversation_item(
        &self,
        api_key: &str,
        conversation_id: &str,
        item_id: &str,
    ) -> Result<Value> {
        self.delegate
            .retrieve_conversation_item(api_key, conversation_id, item_id)
            .await
    }

    pub async fn delete_conversation_item(
        &self,
        api_key: &str,
        conversation_id: &str,
        item_id: &str,
    ) -> Result<Value> {
        self.delegate
            .delete_conversation_item(api_key, conversation_id, item_id)
            .await
    }

    pub async fn chat_completions_stream(
        &self,
        api_key: &str,
        request: &CreateChatCompletionRequest,
    ) -> Result<ProviderStreamOutput> {
        self.delegate
            .chat_completions_stream(api_key, request)
            .await
    }

    pub async fn responses(&self, api_key: &str, request: &CreateResponseRequest) -> Result<Value> {
        self.delegate.responses(api_key, request).await
    }

    pub async fn responses_stream(
        &self,
        api_key: &str,
        request: &CreateResponseRequest,
    ) -> Result<ProviderStreamOutput> {
        self.delegate.responses_stream(api_key, request).await
    }

    pub async fn count_response_input_tokens(
        &self,
        api_key: &str,
        request: &CountResponseInputTokensRequest,
    ) -> Result<Value> {
        self.delegate
            .count_response_input_tokens(api_key, request)
            .await
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

    pub async fn cancel_response(&self, api_key: &str, response_id: &str) -> Result<Value> {
        self.delegate.cancel_response(api_key, response_id).await
    }

    pub async fn compact_response(
        &self,
        api_key: &str,
        request: &CompactResponseRequest,
    ) -> Result<Value> {
        self.delegate.compact_response(api_key, request).await
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

    pub async fn delete_model(&self, api_key: &str, model_id: &str) -> Result<Value> {
        self.delegate.delete_model(api_key, model_id).await
    }

    pub async fn threads(&self, api_key: &str, request: &CreateThreadRequest) -> Result<Value> {
        self.delegate.threads(api_key, request).await
    }

    pub async fn retrieve_thread(&self, api_key: &str, thread_id: &str) -> Result<Value> {
        self.delegate.retrieve_thread(api_key, thread_id).await
    }

    pub async fn update_thread(
        &self,
        api_key: &str,
        thread_id: &str,
        request: &UpdateThreadRequest,
    ) -> Result<Value> {
        self.delegate
            .update_thread(api_key, thread_id, request)
            .await
    }

    pub async fn delete_thread(&self, api_key: &str, thread_id: &str) -> Result<Value> {
        self.delegate.delete_thread(api_key, thread_id).await
    }

    pub async fn create_thread_message(
        &self,
        api_key: &str,
        thread_id: &str,
        request: &CreateThreadMessageRequest,
    ) -> Result<Value> {
        self.delegate
            .create_thread_message(api_key, thread_id, request)
            .await
    }

    pub async fn list_thread_messages(&self, api_key: &str, thread_id: &str) -> Result<Value> {
        self.delegate.list_thread_messages(api_key, thread_id).await
    }

    pub async fn retrieve_thread_message(
        &self,
        api_key: &str,
        thread_id: &str,
        message_id: &str,
    ) -> Result<Value> {
        self.delegate
            .retrieve_thread_message(api_key, thread_id, message_id)
            .await
    }

    pub async fn update_thread_message(
        &self,
        api_key: &str,
        thread_id: &str,
        message_id: &str,
        request: &UpdateThreadMessageRequest,
    ) -> Result<Value> {
        self.delegate
            .update_thread_message(api_key, thread_id, message_id, request)
            .await
    }

    pub async fn delete_thread_message(
        &self,
        api_key: &str,
        thread_id: &str,
        message_id: &str,
    ) -> Result<Value> {
        self.delegate
            .delete_thread_message(api_key, thread_id, message_id)
            .await
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

    pub async fn images_edits(
        &self,
        api_key: &str,
        request: &CreateImageEditRequest,
    ) -> Result<Value> {
        self.delegate.images_edits(api_key, request).await
    }

    pub async fn images_variations(
        &self,
        api_key: &str,
        request: &CreateImageVariationRequest,
    ) -> Result<Value> {
        self.delegate.images_variations(api_key, request).await
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
    ) -> Result<ProviderStreamOutput> {
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

    pub async fn file_content(&self, api_key: &str, file_id: &str) -> Result<ProviderStreamOutput> {
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

    pub async fn video_content(
        &self,
        api_key: &str,
        video_id: &str,
    ) -> Result<ProviderStreamOutput> {
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

impl ProviderAdapter for OllamaProviderAdapter {
    fn id(&self) -> &'static str {
        "ollama"
    }
}

#[async_trait]
impl ProviderExecutionAdapter for OllamaProviderAdapter {
    async fn execute(&self, api_key: &str, request: ProviderRequest<'_>) -> Result<ProviderOutput> {
        self.delegate.execute(api_key, request).await
    }
}
