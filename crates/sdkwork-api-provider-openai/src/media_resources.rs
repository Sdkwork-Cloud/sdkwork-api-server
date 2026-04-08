use super::*;

impl OpenAiProviderAdapter {
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
            .await?;
        let response = self.require_success(response).await?;

        Ok(response.json::<Value>().await?)
    }
}
