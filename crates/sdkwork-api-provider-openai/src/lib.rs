use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;
use reqwest::{Client, RequestBuilder};
use sdkwork_api_contract_openai::assistants::{CreateAssistantRequest, UpdateAssistantRequest};
use sdkwork_api_contract_openai::audio::{
    CreateSpeechRequest, CreateTranscriptionRequest, CreateTranslationRequest,
    CreateVoiceConsentRequest,
};
use sdkwork_api_contract_openai::batches::CreateBatchRequest;
use sdkwork_api_contract_openai::chat_completions::{
    CreateChatCompletionRequest, UpdateChatCompletionRequest,
};
use sdkwork_api_contract_openai::completions::CreateCompletionRequest;
use sdkwork_api_contract_openai::containers::{CreateContainerFileRequest, CreateContainerRequest};
use sdkwork_api_contract_openai::conversations::{
    CreateConversationItemsRequest, CreateConversationRequest, UpdateConversationRequest,
};
use sdkwork_api_contract_openai::embeddings::CreateEmbeddingRequest;
use sdkwork_api_contract_openai::evals::{
    CreateEvalRequest, CreateEvalRunRequest, UpdateEvalRequest,
};
use sdkwork_api_contract_openai::files::CreateFileRequest;
use sdkwork_api_contract_openai::fine_tuning::{
    CreateFineTuningCheckpointPermissionsRequest, CreateFineTuningJobRequest,
};
use sdkwork_api_contract_openai::images::{
    CreateImageEditRequest, CreateImageRequest, CreateImageVariationRequest,
};
use sdkwork_api_contract_openai::moderations::CreateModerationRequest;
use sdkwork_api_contract_openai::realtime::CreateRealtimeSessionRequest;
use sdkwork_api_contract_openai::responses::{
    CompactResponseRequest, CountResponseInputTokensRequest, CreateResponseRequest,
};
use sdkwork_api_contract_openai::runs::{
    CreateRunRequest, CreateThreadAndRunRequest, SubmitToolOutputsRunRequest, UpdateRunRequest,
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
use sdkwork_api_contract_openai::videos::{
    CreateVideoCharacterRequest, CreateVideoRequest, EditVideoRequest, ExtendVideoRequest,
    RemixVideoRequest, UpdateVideoCharacterRequest,
};
use sdkwork_api_contract_openai::webhooks::{CreateWebhookRequest, UpdateWebhookRequest};
use sdkwork_api_domain_catalog::ModelCatalogEntry;
use sdkwork_api_provider_core::{
    ProviderAdapter, ProviderExecutionAdapter, ProviderOutput, ProviderRequest,
    ProviderRequestOptions, ProviderStreamOutput,
};
use serde_json::Value;

pub fn map_model_object(model: &str) -> ModelCatalogEntry {
    ModelCatalogEntry::new(model, "provider-openai-official")
}

#[derive(Debug, Clone)]
pub struct OpenAiProviderAdapter {
    base_url: String,
    client: Client,
    request_headers: HashMap<String, String>,
}

impl OpenAiProviderAdapter {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into().trim_end_matches('/').to_owned(),
            client: Client::new(),
            request_headers: HashMap::new(),
        }
    }

    pub fn with_request_headers(mut self, headers: &HashMap<String, String>) -> Self {
        self.request_headers = headers.clone();
        self
    }

    pub async fn chat_completions(
        &self,
        api_key: &str,
        request: &CreateChatCompletionRequest,
    ) -> Result<Value> {
        self.post_json("/v1/chat/completions", api_key, request)
            .await
    }

    pub async fn list_chat_completions(&self, api_key: &str) -> Result<Value> {
        self.get_json("/v1/chat/completions", api_key).await
    }

    pub async fn retrieve_chat_completion(
        &self,
        api_key: &str,
        completion_id: &str,
    ) -> Result<Value> {
        self.get_json(&format!("/v1/chat/completions/{completion_id}"), api_key)
            .await
    }

    pub async fn update_chat_completion(
        &self,
        api_key: &str,
        completion_id: &str,
        request: &UpdateChatCompletionRequest,
    ) -> Result<Value> {
        self.post_json(
            &format!("/v1/chat/completions/{completion_id}"),
            api_key,
            request,
        )
        .await
    }

    pub async fn delete_chat_completion(
        &self,
        api_key: &str,
        completion_id: &str,
    ) -> Result<Value> {
        self.delete_json(&format!("/v1/chat/completions/{completion_id}"), api_key)
            .await
    }

    pub async fn list_chat_completion_messages(
        &self,
        api_key: &str,
        completion_id: &str,
    ) -> Result<Value> {
        self.get_json(
            &format!("/v1/chat/completions/{completion_id}/messages"),
            api_key,
        )
        .await
    }

    pub async fn conversations(
        &self,
        api_key: &str,
        request: &CreateConversationRequest,
    ) -> Result<Value> {
        self.post_json("/v1/conversations", api_key, request).await
    }

    pub async fn list_conversations(&self, api_key: &str) -> Result<Value> {
        self.get_json("/v1/conversations", api_key).await
    }

    pub async fn retrieve_conversation(
        &self,
        api_key: &str,
        conversation_id: &str,
    ) -> Result<Value> {
        self.get_json(&format!("/v1/conversations/{conversation_id}"), api_key)
            .await
    }

    pub async fn update_conversation(
        &self,
        api_key: &str,
        conversation_id: &str,
        request: &UpdateConversationRequest,
    ) -> Result<Value> {
        self.post_json(
            &format!("/v1/conversations/{conversation_id}"),
            api_key,
            request,
        )
        .await
    }

    pub async fn delete_conversation(&self, api_key: &str, conversation_id: &str) -> Result<Value> {
        self.delete_json(&format!("/v1/conversations/{conversation_id}"), api_key)
            .await
    }

    pub async fn create_conversation_items(
        &self,
        api_key: &str,
        conversation_id: &str,
        request: &CreateConversationItemsRequest,
    ) -> Result<Value> {
        self.post_json(
            &format!("/v1/conversations/{conversation_id}/items"),
            api_key,
            request,
        )
        .await
    }

    pub async fn list_conversation_items(
        &self,
        api_key: &str,
        conversation_id: &str,
    ) -> Result<Value> {
        self.get_json(
            &format!("/v1/conversations/{conversation_id}/items"),
            api_key,
        )
        .await
    }

    pub async fn retrieve_conversation_item(
        &self,
        api_key: &str,
        conversation_id: &str,
        item_id: &str,
    ) -> Result<Value> {
        self.get_json(
            &format!("/v1/conversations/{conversation_id}/items/{item_id}"),
            api_key,
        )
        .await
    }

    pub async fn delete_conversation_item(
        &self,
        api_key: &str,
        conversation_id: &str,
        item_id: &str,
    ) -> Result<Value> {
        self.delete_json(
            &format!("/v1/conversations/{conversation_id}/items/{item_id}"),
            api_key,
        )
        .await
    }

    pub async fn chat_completions_stream(
        &self,
        api_key: &str,
        request: &CreateChatCompletionRequest,
    ) -> Result<ProviderStreamOutput> {
        self.post_stream("/v1/chat/completions", api_key, request)
            .await
    }

    pub async fn responses(&self, api_key: &str, request: &CreateResponseRequest) -> Result<Value> {
        self.post_json("/v1/responses", api_key, request).await
    }

    pub async fn responses_stream(
        &self,
        api_key: &str,
        request: &CreateResponseRequest,
    ) -> Result<ProviderStreamOutput> {
        self.post_stream("/v1/responses", api_key, request).await
    }

    pub async fn count_response_input_tokens(
        &self,
        api_key: &str,
        request: &CountResponseInputTokensRequest,
    ) -> Result<Value> {
        self.post_json("/v1/responses/input_tokens", api_key, request)
            .await
    }

    pub async fn retrieve_response(&self, api_key: &str, response_id: &str) -> Result<Value> {
        self.get_json(&format!("/v1/responses/{response_id}"), api_key)
            .await
    }

    pub async fn delete_response(&self, api_key: &str, response_id: &str) -> Result<Value> {
        self.delete_json(&format!("/v1/responses/{response_id}"), api_key)
            .await
    }

    pub async fn list_response_input_items(
        &self,
        api_key: &str,
        response_id: &str,
    ) -> Result<Value> {
        self.get_json(&format!("/v1/responses/{response_id}/input_items"), api_key)
            .await
    }

    pub async fn cancel_response(&self, api_key: &str, response_id: &str) -> Result<Value> {
        let response = self
            .client
            .post(format!(
                "{}/v1/responses/{response_id}/cancel",
                self.base_url
            ))
            .bearer_auth(api_key)
            .send()
            .await?
            .error_for_status()?;

        Ok(response.json::<Value>().await?)
    }

    pub async fn compact_response(
        &self,
        api_key: &str,
        request: &CompactResponseRequest,
    ) -> Result<Value> {
        self.post_json("/v1/responses/compact", api_key, request)
            .await
    }

    pub async fn completions(
        &self,
        api_key: &str,
        request: &CreateCompletionRequest,
    ) -> Result<Value> {
        self.post_json("/v1/completions", api_key, request).await
    }

    pub async fn containers(
        &self,
        api_key: &str,
        request: &CreateContainerRequest,
    ) -> Result<Value> {
        self.post_json("/v1/containers", api_key, request).await
    }

    pub async fn list_containers(&self, api_key: &str) -> Result<Value> {
        self.get_json("/v1/containers", api_key).await
    }

    pub async fn retrieve_container(&self, api_key: &str, container_id: &str) -> Result<Value> {
        self.get_json(&format!("/v1/containers/{container_id}"), api_key)
            .await
    }

    pub async fn delete_container(&self, api_key: &str, container_id: &str) -> Result<Value> {
        self.delete_json(&format!("/v1/containers/{container_id}"), api_key)
            .await
    }

    pub async fn create_container_file(
        &self,
        api_key: &str,
        container_id: &str,
        request: &CreateContainerFileRequest,
    ) -> Result<Value> {
        self.post_json(
            &format!("/v1/containers/{container_id}/files"),
            api_key,
            request,
        )
        .await
    }

    pub async fn list_container_files(&self, api_key: &str, container_id: &str) -> Result<Value> {
        self.get_json(&format!("/v1/containers/{container_id}/files"), api_key)
            .await
    }

    pub async fn retrieve_container_file(
        &self,
        api_key: &str,
        container_id: &str,
        file_id: &str,
    ) -> Result<Value> {
        self.get_json(
            &format!("/v1/containers/{container_id}/files/{file_id}"),
            api_key,
        )
        .await
    }

    pub async fn delete_container_file(
        &self,
        api_key: &str,
        container_id: &str,
        file_id: &str,
    ) -> Result<Value> {
        self.delete_json(
            &format!("/v1/containers/{container_id}/files/{file_id}"),
            api_key,
        )
        .await
    }

    pub async fn container_file_content(
        &self,
        api_key: &str,
        container_id: &str,
        file_id: &str,
    ) -> Result<ProviderStreamOutput> {
        self.get_stream(
            &format!("/v1/containers/{container_id}/files/{file_id}/content"),
            api_key,
        )
        .await
    }

    pub async fn list_models(&self, api_key: &str) -> Result<Value> {
        self.get_json("/v1/models", api_key).await
    }

    pub async fn retrieve_model(&self, api_key: &str, model_id: &str) -> Result<Value> {
        self.get_json(&format!("/v1/models/{model_id}"), api_key)
            .await
    }

    pub async fn delete_model(&self, api_key: &str, model_id: &str) -> Result<Value> {
        self.delete_json(&format!("/v1/models/{model_id}"), api_key)
            .await
    }

    pub async fn threads(&self, api_key: &str, request: &CreateThreadRequest) -> Result<Value> {
        self.post_json("/v1/threads", api_key, request).await
    }

    pub async fn retrieve_thread(&self, api_key: &str, thread_id: &str) -> Result<Value> {
        self.get_json(&format!("/v1/threads/{thread_id}"), api_key)
            .await
    }

    pub async fn update_thread(
        &self,
        api_key: &str,
        thread_id: &str,
        request: &UpdateThreadRequest,
    ) -> Result<Value> {
        self.post_json(&format!("/v1/threads/{thread_id}"), api_key, request)
            .await
    }

    pub async fn delete_thread(&self, api_key: &str, thread_id: &str) -> Result<Value> {
        self.delete_json(&format!("/v1/threads/{thread_id}"), api_key)
            .await
    }

    pub async fn create_thread_message(
        &self,
        api_key: &str,
        thread_id: &str,
        request: &CreateThreadMessageRequest,
    ) -> Result<Value> {
        self.post_json(
            &format!("/v1/threads/{thread_id}/messages"),
            api_key,
            request,
        )
        .await
    }

    pub async fn list_thread_messages(&self, api_key: &str, thread_id: &str) -> Result<Value> {
        self.get_json(&format!("/v1/threads/{thread_id}/messages"), api_key)
            .await
    }

    pub async fn retrieve_thread_message(
        &self,
        api_key: &str,
        thread_id: &str,
        message_id: &str,
    ) -> Result<Value> {
        self.get_json(
            &format!("/v1/threads/{thread_id}/messages/{message_id}"),
            api_key,
        )
        .await
    }

    pub async fn update_thread_message(
        &self,
        api_key: &str,
        thread_id: &str,
        message_id: &str,
        request: &UpdateThreadMessageRequest,
    ) -> Result<Value> {
        self.post_json(
            &format!("/v1/threads/{thread_id}/messages/{message_id}"),
            api_key,
            request,
        )
        .await
    }

    pub async fn delete_thread_message(
        &self,
        api_key: &str,
        thread_id: &str,
        message_id: &str,
    ) -> Result<Value> {
        self.delete_json(
            &format!("/v1/threads/{thread_id}/messages/{message_id}"),
            api_key,
        )
        .await
    }

    pub async fn create_thread_run(
        &self,
        api_key: &str,
        thread_id: &str,
        request: &CreateRunRequest,
    ) -> Result<Value> {
        self.post_json(&format!("/v1/threads/{thread_id}/runs"), api_key, request)
            .await
    }

    pub async fn list_thread_runs(&self, api_key: &str, thread_id: &str) -> Result<Value> {
        self.get_json(&format!("/v1/threads/{thread_id}/runs"), api_key)
            .await
    }

    pub async fn retrieve_thread_run(
        &self,
        api_key: &str,
        thread_id: &str,
        run_id: &str,
    ) -> Result<Value> {
        self.get_json(&format!("/v1/threads/{thread_id}/runs/{run_id}"), api_key)
            .await
    }

    pub async fn update_thread_run(
        &self,
        api_key: &str,
        thread_id: &str,
        run_id: &str,
        request: &UpdateRunRequest,
    ) -> Result<Value> {
        self.post_json(
            &format!("/v1/threads/{thread_id}/runs/{run_id}"),
            api_key,
            request,
        )
        .await
    }

    pub async fn cancel_thread_run(
        &self,
        api_key: &str,
        thread_id: &str,
        run_id: &str,
    ) -> Result<Value> {
        self.post_empty_json(
            &format!("/v1/threads/{thread_id}/runs/{run_id}/cancel"),
            api_key,
        )
        .await
    }

    pub async fn submit_thread_run_tool_outputs(
        &self,
        api_key: &str,
        thread_id: &str,
        run_id: &str,
        request: &SubmitToolOutputsRunRequest,
    ) -> Result<Value> {
        self.post_json(
            &format!("/v1/threads/{thread_id}/runs/{run_id}/submit_tool_outputs"),
            api_key,
            request,
        )
        .await
    }

    pub async fn list_thread_run_steps(
        &self,
        api_key: &str,
        thread_id: &str,
        run_id: &str,
    ) -> Result<Value> {
        self.get_json(
            &format!("/v1/threads/{thread_id}/runs/{run_id}/steps"),
            api_key,
        )
        .await
    }

    pub async fn retrieve_thread_run_step(
        &self,
        api_key: &str,
        thread_id: &str,
        run_id: &str,
        step_id: &str,
    ) -> Result<Value> {
        self.get_json(
            &format!("/v1/threads/{thread_id}/runs/{run_id}/steps/{step_id}"),
            api_key,
        )
        .await
    }

    pub async fn create_thread_and_run(
        &self,
        api_key: &str,
        request: &CreateThreadAndRunRequest,
    ) -> Result<Value> {
        self.post_json("/v1/threads/runs", api_key, request).await
    }

    pub async fn embeddings(
        &self,
        api_key: &str,
        request: &CreateEmbeddingRequest,
    ) -> Result<Value> {
        self.post_json("/v1/embeddings", api_key, request).await
    }

    pub async fn moderations(
        &self,
        api_key: &str,
        request: &CreateModerationRequest,
    ) -> Result<Value> {
        self.post_json("/v1/moderations", api_key, request).await
    }

    pub async fn images_generations(
        &self,
        api_key: &str,
        request: &CreateImageRequest,
    ) -> Result<Value> {
        self.post_json("/v1/images/generations", api_key, request)
            .await
    }

    pub async fn images_edits(
        &self,
        api_key: &str,
        request: &CreateImageEditRequest,
    ) -> Result<Value> {
        let mut form = reqwest::multipart::Form::new()
            .text("prompt", request.prompt.clone())
            .part(
                "image",
                multipart_file_part(
                    request.image.bytes.clone(),
                    &request.image.filename,
                    request.image.content_type.as_deref(),
                ),
            );
        form = add_optional_text_field(form, "model", request.model.as_deref());
        form = add_optional_number_field(form, "n", request.n);
        form = add_optional_text_field(form, "quality", request.quality.as_deref());
        form = add_optional_text_field(form, "response_format", request.response_format.as_deref());
        form = add_optional_text_field(form, "size", request.size.as_deref());
        form = add_optional_text_field(form, "user", request.user.as_deref());
        if let Some(mask) = &request.mask {
            form = form.part(
                "mask",
                multipart_file_part(
                    mask.bytes.clone(),
                    &mask.filename,
                    mask.content_type.as_deref(),
                ),
            );
        }
        self.post_multipart_json("/v1/images/edits", api_key, form)
            .await
    }

    pub async fn images_variations(
        &self,
        api_key: &str,
        request: &CreateImageVariationRequest,
    ) -> Result<Value> {
        let mut form = reqwest::multipart::Form::new().part(
            "image",
            multipart_file_part(
                request.image.bytes.clone(),
                &request.image.filename,
                request.image.content_type.as_deref(),
            ),
        );
        form = add_optional_text_field(form, "model", request.model.as_deref());
        form = add_optional_number_field(form, "n", request.n);
        form = add_optional_text_field(form, "response_format", request.response_format.as_deref());
        form = add_optional_text_field(form, "size", request.size.as_deref());
        form = add_optional_text_field(form, "user", request.user.as_deref());
        self.post_multipart_json("/v1/images/variations", api_key, form)
            .await
    }

    pub async fn audio_transcriptions(
        &self,
        api_key: &str,
        request: &CreateTranscriptionRequest,
    ) -> Result<Value> {
        self.post_json("/v1/audio/transcriptions", api_key, request)
            .await
    }

    pub async fn audio_translations(
        &self,
        api_key: &str,
        request: &CreateTranslationRequest,
    ) -> Result<Value> {
        self.post_json("/v1/audio/translations", api_key, request)
            .await
    }

    pub async fn audio_speech(
        &self,
        api_key: &str,
        request: &CreateSpeechRequest,
    ) -> Result<ProviderStreamOutput> {
        self.post_stream("/v1/audio/speech", api_key, request).await
    }

    pub async fn list_audio_voices(&self, api_key: &str) -> Result<Value> {
        self.get_json("/v1/audio/voices", api_key).await
    }

    pub async fn audio_voice_consents(
        &self,
        api_key: &str,
        request: &CreateVoiceConsentRequest,
    ) -> Result<Value> {
        self.post_json("/v1/audio/voice_consents", api_key, request)
            .await
    }

    pub async fn files(&self, api_key: &str, request: &CreateFileRequest) -> Result<Value> {
        let file = multipart_file_part(
            request.bytes.clone(),
            &request.filename,
            request.content_type.as_deref(),
        );
        let form = reqwest::multipart::Form::new()
            .text("purpose", request.purpose.clone())
            .part("file", file);
        self.post_multipart_json("/v1/files", api_key, form).await
    }

    pub async fn list_files(&self, api_key: &str) -> Result<Value> {
        self.get_json("/v1/files", api_key).await
    }

    pub async fn retrieve_file(&self, api_key: &str, file_id: &str) -> Result<Value> {
        self.get_json(&format!("/v1/files/{file_id}"), api_key)
            .await
    }

    pub async fn delete_file(&self, api_key: &str, file_id: &str) -> Result<Value> {
        self.delete_json(&format!("/v1/files/{file_id}"), api_key)
            .await
    }

    pub async fn file_content(&self, api_key: &str, file_id: &str) -> Result<ProviderStreamOutput> {
        self.get_stream(&format!("/v1/files/{file_id}/content"), api_key)
            .await
    }

    pub async fn uploads(&self, api_key: &str, request: &CreateUploadRequest) -> Result<Value> {
        self.post_json("/v1/uploads", api_key, request).await
    }

    pub async fn upload_parts(
        &self,
        api_key: &str,
        request: &AddUploadPartRequest,
    ) -> Result<Value> {
        let data = multipart_file_part(
            request.data.clone(),
            request.filename.as_deref().unwrap_or("upload-part.bin"),
            request.content_type.as_deref(),
        );
        let form = reqwest::multipart::Form::new().part("data", data);
        self.post_multipart_json(
            &format!("/v1/uploads/{}/parts", request.upload_id),
            api_key,
            form,
        )
        .await
    }

    pub async fn complete_upload(
        &self,
        api_key: &str,
        request: &CompleteUploadRequest,
    ) -> Result<Value> {
        self.post_json(
            &format!("/v1/uploads/{}/complete", request.upload_id),
            api_key,
            request,
        )
        .await
    }

    pub async fn cancel_upload(&self, api_key: &str, upload_id: &str) -> Result<Value> {
        let response = self
            .client
            .post(format!("{}/v1/uploads/{upload_id}/cancel", self.base_url))
            .bearer_auth(api_key)
            .send()
            .await?
            .error_for_status()?;

        Ok(response.json::<Value>().await?)
    }

    pub async fn fine_tuning_jobs(
        &self,
        api_key: &str,
        request: &CreateFineTuningJobRequest,
    ) -> Result<Value> {
        self.post_json("/v1/fine_tuning/jobs", api_key, request)
            .await
    }

    pub async fn list_fine_tuning_jobs(&self, api_key: &str) -> Result<Value> {
        self.get_json("/v1/fine_tuning/jobs", api_key).await
    }

    pub async fn retrieve_fine_tuning_job(&self, api_key: &str, job_id: &str) -> Result<Value> {
        self.get_json(&format!("/v1/fine_tuning/jobs/{job_id}"), api_key)
            .await
    }

    pub async fn cancel_fine_tuning_job(&self, api_key: &str, job_id: &str) -> Result<Value> {
        self.post_empty_json(&format!("/v1/fine_tuning/jobs/{job_id}/cancel"), api_key)
            .await
    }

    pub async fn list_fine_tuning_job_events(&self, api_key: &str, job_id: &str) -> Result<Value> {
        self.get_json(&format!("/v1/fine_tuning/jobs/{job_id}/events"), api_key)
            .await
    }

    pub async fn list_fine_tuning_job_checkpoints(
        &self,
        api_key: &str,
        job_id: &str,
    ) -> Result<Value> {
        self.get_json(
            &format!("/v1/fine_tuning/jobs/{job_id}/checkpoints"),
            api_key,
        )
        .await
    }

    pub async fn pause_fine_tuning_job(&self, api_key: &str, job_id: &str) -> Result<Value> {
        self.post_empty_json(&format!("/v1/fine_tuning/jobs/{job_id}/pause"), api_key)
            .await
    }

    pub async fn resume_fine_tuning_job(&self, api_key: &str, job_id: &str) -> Result<Value> {
        self.post_empty_json(&format!("/v1/fine_tuning/jobs/{job_id}/resume"), api_key)
            .await
    }

    pub async fn create_fine_tuning_checkpoint_permissions(
        &self,
        api_key: &str,
        fine_tuned_model_checkpoint: &str,
        request: &CreateFineTuningCheckpointPermissionsRequest,
    ) -> Result<Value> {
        self.post_json(
            &format!("/v1/fine_tuning/checkpoints/{fine_tuned_model_checkpoint}/permissions"),
            api_key,
            request,
        )
        .await
    }

    pub async fn list_fine_tuning_checkpoint_permissions(
        &self,
        api_key: &str,
        fine_tuned_model_checkpoint: &str,
    ) -> Result<Value> {
        self.get_json(
            &format!("/v1/fine_tuning/checkpoints/{fine_tuned_model_checkpoint}/permissions"),
            api_key,
        )
        .await
    }

    pub async fn delete_fine_tuning_checkpoint_permission(
        &self,
        api_key: &str,
        fine_tuned_model_checkpoint: &str,
        permission_id: &str,
    ) -> Result<Value> {
        self.delete_json(
            &format!(
                "/v1/fine_tuning/checkpoints/{fine_tuned_model_checkpoint}/permissions/{permission_id}"
            ),
            api_key,
        )
        .await
    }

    pub async fn assistants(
        &self,
        api_key: &str,
        request: &CreateAssistantRequest,
    ) -> Result<Value> {
        self.post_json("/v1/assistants", api_key, request).await
    }

    pub async fn list_assistants(&self, api_key: &str) -> Result<Value> {
        self.get_json("/v1/assistants", api_key).await
    }

    pub async fn retrieve_assistant(&self, api_key: &str, assistant_id: &str) -> Result<Value> {
        self.get_json(&format!("/v1/assistants/{assistant_id}"), api_key)
            .await
    }

    pub async fn update_assistant(
        &self,
        api_key: &str,
        assistant_id: &str,
        request: &UpdateAssistantRequest,
    ) -> Result<Value> {
        self.post_json(&format!("/v1/assistants/{assistant_id}"), api_key, request)
            .await
    }

    pub async fn delete_assistant(&self, api_key: &str, assistant_id: &str) -> Result<Value> {
        self.delete_json(&format!("/v1/assistants/{assistant_id}"), api_key)
            .await
    }

    pub async fn realtime_sessions(
        &self,
        api_key: &str,
        request: &CreateRealtimeSessionRequest,
    ) -> Result<Value> {
        self.post_json("/v1/realtime/sessions", api_key, request)
            .await
    }

    pub async fn evals(&self, api_key: &str, request: &CreateEvalRequest) -> Result<Value> {
        self.post_json("/v1/evals", api_key, request).await
    }

    pub async fn list_evals(&self, api_key: &str) -> Result<Value> {
        self.get_json("/v1/evals", api_key).await
    }

    pub async fn retrieve_eval(&self, api_key: &str, eval_id: &str) -> Result<Value> {
        self.get_json(&format!("/v1/evals/{eval_id}"), api_key)
            .await
    }

    pub async fn update_eval(
        &self,
        api_key: &str,
        eval_id: &str,
        request: &UpdateEvalRequest,
    ) -> Result<Value> {
        self.post_json(&format!("/v1/evals/{eval_id}"), api_key, request)
            .await
    }

    pub async fn delete_eval(&self, api_key: &str, eval_id: &str) -> Result<Value> {
        self.delete_json(&format!("/v1/evals/{eval_id}"), api_key)
            .await
    }

    pub async fn list_eval_runs(&self, api_key: &str, eval_id: &str) -> Result<Value> {
        self.get_json(&format!("/v1/evals/{eval_id}/runs"), api_key)
            .await
    }

    pub async fn create_eval_run(
        &self,
        api_key: &str,
        eval_id: &str,
        request: &CreateEvalRunRequest,
    ) -> Result<Value> {
        self.post_json(&format!("/v1/evals/{eval_id}/runs"), api_key, request)
            .await
    }

    pub async fn retrieve_eval_run(
        &self,
        api_key: &str,
        eval_id: &str,
        run_id: &str,
    ) -> Result<Value> {
        self.get_json(&format!("/v1/evals/{eval_id}/runs/{run_id}"), api_key)
            .await
    }

    pub async fn delete_eval_run(
        &self,
        api_key: &str,
        eval_id: &str,
        run_id: &str,
    ) -> Result<Value> {
        self.delete_json(&format!("/v1/evals/{eval_id}/runs/{run_id}"), api_key)
            .await
    }

    pub async fn cancel_eval_run(
        &self,
        api_key: &str,
        eval_id: &str,
        run_id: &str,
    ) -> Result<Value> {
        self.post_empty_json(
            &format!("/v1/evals/{eval_id}/runs/{run_id}/cancel"),
            api_key,
        )
        .await
    }

    pub async fn list_eval_run_output_items(
        &self,
        api_key: &str,
        eval_id: &str,
        run_id: &str,
    ) -> Result<Value> {
        self.get_json(
            &format!("/v1/evals/{eval_id}/runs/{run_id}/output_items"),
            api_key,
        )
        .await
    }

    pub async fn retrieve_eval_run_output_item(
        &self,
        api_key: &str,
        eval_id: &str,
        run_id: &str,
        output_item_id: &str,
    ) -> Result<Value> {
        self.get_json(
            &format!("/v1/evals/{eval_id}/runs/{run_id}/output_items/{output_item_id}"),
            api_key,
        )
        .await
    }

    pub async fn batches(&self, api_key: &str, request: &CreateBatchRequest) -> Result<Value> {
        self.post_json("/v1/batches", api_key, request).await
    }

    pub async fn list_batches(&self, api_key: &str) -> Result<Value> {
        self.get_json("/v1/batches", api_key).await
    }

    pub async fn retrieve_batch(&self, api_key: &str, batch_id: &str) -> Result<Value> {
        self.get_json(&format!("/v1/batches/{batch_id}"), api_key)
            .await
    }

    pub async fn cancel_batch(&self, api_key: &str, batch_id: &str) -> Result<Value> {
        let response = self
            .client
            .post(format!("{}/v1/batches/{batch_id}/cancel", self.base_url))
            .bearer_auth(api_key)
            .send()
            .await?
            .error_for_status()?;

        Ok(response.json::<Value>().await?)
    }

    pub async fn vector_stores(
        &self,
        api_key: &str,
        request: &CreateVectorStoreRequest,
    ) -> Result<Value> {
        self.post_json("/v1/vector_stores", api_key, request).await
    }

    pub async fn list_vector_stores(&self, api_key: &str) -> Result<Value> {
        self.get_json("/v1/vector_stores", api_key).await
    }

    pub async fn retrieve_vector_store(
        &self,
        api_key: &str,
        vector_store_id: &str,
    ) -> Result<Value> {
        self.get_json(&format!("/v1/vector_stores/{vector_store_id}"), api_key)
            .await
    }

    pub async fn update_vector_store(
        &self,
        api_key: &str,
        vector_store_id: &str,
        request: &UpdateVectorStoreRequest,
    ) -> Result<Value> {
        self.post_json(
            &format!("/v1/vector_stores/{vector_store_id}"),
            api_key,
            request,
        )
        .await
    }

    pub async fn delete_vector_store(&self, api_key: &str, vector_store_id: &str) -> Result<Value> {
        self.delete_json(&format!("/v1/vector_stores/{vector_store_id}"), api_key)
            .await
    }

    pub async fn search_vector_store(
        &self,
        api_key: &str,
        vector_store_id: &str,
        request: &SearchVectorStoreRequest,
    ) -> Result<Value> {
        self.post_json(
            &format!("/v1/vector_stores/{vector_store_id}/search"),
            api_key,
            request,
        )
        .await
    }

    pub async fn create_vector_store_file(
        &self,
        api_key: &str,
        vector_store_id: &str,
        request: &CreateVectorStoreFileRequest,
    ) -> Result<Value> {
        self.post_json(
            &format!("/v1/vector_stores/{vector_store_id}/files"),
            api_key,
            request,
        )
        .await
    }

    pub async fn list_vector_store_files(
        &self,
        api_key: &str,
        vector_store_id: &str,
    ) -> Result<Value> {
        self.get_json(
            &format!("/v1/vector_stores/{vector_store_id}/files"),
            api_key,
        )
        .await
    }

    pub async fn retrieve_vector_store_file(
        &self,
        api_key: &str,
        vector_store_id: &str,
        file_id: &str,
    ) -> Result<Value> {
        self.get_json(
            &format!("/v1/vector_stores/{vector_store_id}/files/{file_id}"),
            api_key,
        )
        .await
    }

    pub async fn delete_vector_store_file(
        &self,
        api_key: &str,
        vector_store_id: &str,
        file_id: &str,
    ) -> Result<Value> {
        self.delete_json(
            &format!("/v1/vector_stores/{vector_store_id}/files/{file_id}"),
            api_key,
        )
        .await
    }

    pub async fn create_vector_store_file_batch(
        &self,
        api_key: &str,
        vector_store_id: &str,
        request: &CreateVectorStoreFileBatchRequest,
    ) -> Result<Value> {
        self.post_json(
            &format!("/v1/vector_stores/{vector_store_id}/file_batches"),
            api_key,
            request,
        )
        .await
    }

    pub async fn retrieve_vector_store_file_batch(
        &self,
        api_key: &str,
        vector_store_id: &str,
        batch_id: &str,
    ) -> Result<Value> {
        self.get_json(
            &format!("/v1/vector_stores/{vector_store_id}/file_batches/{batch_id}"),
            api_key,
        )
        .await
    }

    pub async fn cancel_vector_store_file_batch(
        &self,
        api_key: &str,
        vector_store_id: &str,
        batch_id: &str,
    ) -> Result<Value> {
        let response = self
            .client
            .post(format!(
                "{}/v1/vector_stores/{vector_store_id}/file_batches/{batch_id}/cancel",
                self.base_url
            ))
            .bearer_auth(api_key)
            .send()
            .await?
            .error_for_status()?;

        Ok(response.json::<Value>().await?)
    }

    pub async fn list_vector_store_file_batch_files(
        &self,
        api_key: &str,
        vector_store_id: &str,
        batch_id: &str,
    ) -> Result<Value> {
        self.get_json(
            &format!("/v1/vector_stores/{vector_store_id}/file_batches/{batch_id}/files"),
            api_key,
        )
        .await
    }

    pub async fn videos(&self, api_key: &str, request: &CreateVideoRequest) -> Result<Value> {
        self.post_json("/v1/videos", api_key, request).await
    }

    pub async fn list_videos(&self, api_key: &str) -> Result<Value> {
        self.get_json("/v1/videos", api_key).await
    }

    pub async fn retrieve_video(&self, api_key: &str, video_id: &str) -> Result<Value> {
        self.get_json(&format!("/v1/videos/{video_id}"), api_key)
            .await
    }

    pub async fn delete_video(&self, api_key: &str, video_id: &str) -> Result<Value> {
        self.delete_json(&format!("/v1/videos/{video_id}"), api_key)
            .await
    }

    pub async fn video_content(
        &self,
        api_key: &str,
        video_id: &str,
    ) -> Result<ProviderStreamOutput> {
        self.get_stream(&format!("/v1/videos/{video_id}/content"), api_key)
            .await
    }

    pub async fn remix_video(
        &self,
        api_key: &str,
        video_id: &str,
        request: &RemixVideoRequest,
    ) -> Result<Value> {
        self.post_json(&format!("/v1/videos/{video_id}/remix"), api_key, request)
            .await
    }

    pub async fn create_video_character(
        &self,
        api_key: &str,
        request: &CreateVideoCharacterRequest,
    ) -> Result<Value> {
        self.post_json("/v1/videos/characters", api_key, request)
            .await
    }

    pub async fn list_video_characters(&self, api_key: &str, video_id: &str) -> Result<Value> {
        self.get_json(&format!("/v1/videos/{video_id}/characters"), api_key)
            .await
    }

    pub async fn retrieve_video_character(
        &self,
        api_key: &str,
        video_id: &str,
        character_id: &str,
    ) -> Result<Value> {
        self.get_json(
            &format!("/v1/videos/{video_id}/characters/{character_id}"),
            api_key,
        )
        .await
    }

    pub async fn retrieve_video_character_canonical(
        &self,
        api_key: &str,
        character_id: &str,
    ) -> Result<Value> {
        self.get_json(&format!("/v1/videos/characters/{character_id}"), api_key)
            .await
    }

    pub async fn update_video_character(
        &self,
        api_key: &str,
        video_id: &str,
        character_id: &str,
        request: &UpdateVideoCharacterRequest,
    ) -> Result<Value> {
        self.post_json(
            &format!("/v1/videos/{video_id}/characters/{character_id}"),
            api_key,
            request,
        )
        .await
    }

    pub async fn edit_video(&self, api_key: &str, request: &EditVideoRequest) -> Result<Value> {
        self.post_json("/v1/videos/edits", api_key, request).await
    }

    pub async fn extensions_video(
        &self,
        api_key: &str,
        request: &ExtendVideoRequest,
    ) -> Result<Value> {
        self.post_json("/v1/videos/extensions", api_key, request)
            .await
    }

    pub async fn extend_video(
        &self,
        api_key: &str,
        video_id: &str,
        request: &ExtendVideoRequest,
    ) -> Result<Value> {
        self.post_json(&format!("/v1/videos/{video_id}/extend"), api_key, request)
            .await
    }

    pub async fn webhooks(&self, api_key: &str, request: &CreateWebhookRequest) -> Result<Value> {
        self.post_json("/v1/webhooks", api_key, request).await
    }

    pub async fn list_webhooks(&self, api_key: &str) -> Result<Value> {
        self.get_json("/v1/webhooks", api_key).await
    }

    pub async fn retrieve_webhook(&self, api_key: &str, webhook_id: &str) -> Result<Value> {
        self.get_json(&format!("/v1/webhooks/{webhook_id}"), api_key)
            .await
    }

    pub async fn update_webhook(
        &self,
        api_key: &str,
        webhook_id: &str,
        request: &UpdateWebhookRequest,
    ) -> Result<Value> {
        self.post_json(&format!("/v1/webhooks/{webhook_id}"), api_key, request)
            .await
    }

    pub async fn delete_webhook(&self, api_key: &str, webhook_id: &str) -> Result<Value> {
        self.delete_json(&format!("/v1/webhooks/{webhook_id}"), api_key)
            .await
    }

    async fn post_json<T: serde::Serialize>(
        &self,
        path: &str,
        api_key: &str,
        request: &T,
    ) -> Result<Value> {
        let response = self
            .authorized_request(reqwest::Method::POST, path, api_key)
            .json(request)
            .send()
            .await?
            .error_for_status()?;

        Ok(response.json::<Value>().await?)
    }

    async fn post_stream<T: serde::Serialize>(
        &self,
        path: &str,
        api_key: &str,
        request: &T,
    ) -> Result<ProviderStreamOutput> {
        let response = self
            .authorized_request(reqwest::Method::POST, path, api_key)
            .json(request)
            .send()
            .await?
            .error_for_status()?;

        Ok(ProviderStreamOutput::from_reqwest_response(response))
    }

    async fn get_json(&self, path: &str, api_key: &str) -> Result<Value> {
        let response = self
            .authorized_request(reqwest::Method::GET, path, api_key)
            .send()
            .await?
            .error_for_status()?;

        Ok(response.json::<Value>().await?)
    }

    async fn get_stream(&self, path: &str, api_key: &str) -> Result<ProviderStreamOutput> {
        let response = self
            .authorized_request(reqwest::Method::GET, path, api_key)
            .send()
            .await?
            .error_for_status()?;

        Ok(ProviderStreamOutput::from_reqwest_response(response))
    }

    async fn delete_json(&self, path: &str, api_key: &str) -> Result<Value> {
        let response = self
            .authorized_request(reqwest::Method::DELETE, path, api_key)
            .send()
            .await?
            .error_for_status()?;

        Ok(response.json::<Value>().await?)
    }

    async fn post_multipart_json(
        &self,
        path: &str,
        api_key: &str,
        form: reqwest::multipart::Form,
    ) -> Result<Value> {
        let response = self
            .authorized_request(reqwest::Method::POST, path, api_key)
            .multipart(form)
            .send()
            .await?
            .error_for_status()?;

        Ok(response.json::<Value>().await?)
    }

    async fn post_empty_json(&self, path: &str, api_key: &str) -> Result<Value> {
        let response = self
            .authorized_request(reqwest::Method::POST, path, api_key)
            .send()
            .await?
            .error_for_status()?;

        Ok(response.json::<Value>().await?)
    }

    fn authorized_request(
        &self,
        method: reqwest::Method,
        path: &str,
        api_key: &str,
    ) -> RequestBuilder {
        let mut builder = self
            .client
            .request(method, format!("{}{}", self.base_url, path))
            .bearer_auth(api_key);
        for (name, value) in &self.request_headers {
            builder = builder.header(name, value);
        }

        self.apply_openai_compat_headers(path, builder)
    }

    fn apply_openai_compat_headers(&self, path: &str, builder: RequestBuilder) -> RequestBuilder {
        if path.starts_with("/v1/assistants") || path.starts_with("/v1/threads") {
            builder.header("OpenAI-Beta", "assistants=v2")
        } else {
            builder
        }
    }
}

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
            self.clone().with_request_headers(options.headers())
        };
        adapter.execute(api_key, request).await
    }
}

fn multipart_file_part(
    bytes: Vec<u8>,
    filename: &str,
    content_type: Option<&str>,
) -> reqwest::multipart::Part {
    let mut part = reqwest::multipart::Part::bytes(bytes).file_name(filename.to_owned());
    if let Some(content_type) = content_type {
        part = part
            .mime_str(content_type)
            .expect("valid multipart content type");
    }
    part
}

fn add_optional_text_field(
    form: reqwest::multipart::Form,
    name: &str,
    value: Option<&str>,
) -> reqwest::multipart::Form {
    match value {
        Some(value) => form.text(name.to_owned(), value.to_owned()),
        None => form,
    }
}

fn add_optional_number_field(
    form: reqwest::multipart::Form,
    name: &str,
    value: Option<u32>,
) -> reqwest::multipart::Form {
    match value {
        Some(value) => form.text(name.to_owned(), value.to_string()),
        None => form,
    }
}
