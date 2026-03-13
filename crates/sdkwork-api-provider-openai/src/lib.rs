use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client;
use sdkwork_api_contract_openai::assistants::CreateAssistantRequest;
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
use sdkwork_api_domain_catalog::ModelCatalogEntry;
use sdkwork_api_provider_core::{
    ProviderAdapter, ProviderExecutionAdapter, ProviderOutput, ProviderRequest,
};
use serde_json::Value;

pub fn map_model_object(model: &str) -> ModelCatalogEntry {
    ModelCatalogEntry::new(model, "provider-openai-official")
}

#[derive(Debug, Clone)]
pub struct OpenAiProviderAdapter {
    base_url: String,
    client: Client,
}

impl OpenAiProviderAdapter {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into().trim_end_matches('/').to_owned(),
            client: Client::new(),
        }
    }

    pub async fn chat_completions(
        &self,
        api_key: &str,
        request: &CreateChatCompletionRequest,
    ) -> Result<Value> {
        self.post_json("/v1/chat/completions", api_key, request)
            .await
    }

    pub async fn chat_completions_stream(
        &self,
        api_key: &str,
        request: &CreateChatCompletionRequest,
    ) -> Result<reqwest::Response> {
        self.post_stream("/v1/chat/completions", api_key, request)
            .await
    }

    pub async fn responses(&self, api_key: &str, request: &CreateResponseRequest) -> Result<Value> {
        self.post_json("/v1/responses", api_key, request).await
    }

    pub async fn completions(
        &self,
        api_key: &str,
        request: &CreateCompletionRequest,
    ) -> Result<Value> {
        self.post_json("/v1/completions", api_key, request).await
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
    ) -> Result<reqwest::Response> {
        self.post_stream("/v1/audio/speech", api_key, request).await
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

    pub async fn file_content(&self, api_key: &str, file_id: &str) -> Result<reqwest::Response> {
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
        let response = self
            .client
            .post(format!(
                "{}/v1/fine_tuning/jobs/{job_id}/cancel",
                self.base_url
            ))
            .bearer_auth(api_key)
            .send()
            .await?
            .error_for_status()?;

        Ok(response.json::<Value>().await?)
    }

    pub async fn assistants(
        &self,
        api_key: &str,
        request: &CreateAssistantRequest,
    ) -> Result<Value> {
        self.post_json("/v1/assistants", api_key, request).await
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

    async fn post_json<T: serde::Serialize>(
        &self,
        path: &str,
        api_key: &str,
        request: &T,
    ) -> Result<Value> {
        let response = self
            .client
            .post(format!("{}{}", self.base_url, path))
            .bearer_auth(api_key)
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
    ) -> Result<reqwest::Response> {
        let response = self
            .client
            .post(format!("{}{}", self.base_url, path))
            .bearer_auth(api_key)
            .json(request)
            .send()
            .await?
            .error_for_status()?;

        Ok(response)
    }

    async fn get_json(&self, path: &str, api_key: &str) -> Result<Value> {
        let response = self
            .client
            .get(format!("{}{}", self.base_url, path))
            .bearer_auth(api_key)
            .send()
            .await?
            .error_for_status()?;

        Ok(response.json::<Value>().await?)
    }

    async fn get_stream(&self, path: &str, api_key: &str) -> Result<reqwest::Response> {
        let response = self
            .client
            .get(format!("{}{}", self.base_url, path))
            .bearer_auth(api_key)
            .send()
            .await?
            .error_for_status()?;

        Ok(response)
    }

    async fn delete_json(&self, path: &str, api_key: &str) -> Result<Value> {
        let response = self
            .client
            .delete(format!("{}{}", self.base_url, path))
            .bearer_auth(api_key)
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
            .client
            .post(format!("{}{}", self.base_url, path))
            .bearer_auth(api_key)
            .multipart(form)
            .send()
            .await?
            .error_for_status()?;

        Ok(response.json::<Value>().await?)
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
        }
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
