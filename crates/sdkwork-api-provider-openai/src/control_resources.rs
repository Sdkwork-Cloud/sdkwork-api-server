use super::*;

impl OpenAiProviderAdapter {
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
            .await?;
        let response = self.require_success(response).await?;

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
            .await?;
        let response = self.require_success(response).await?;

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

    pub async fn music(&self, api_key: &str, request: &CreateMusicRequest) -> Result<Value> {
        self.post_json("/v1/music", api_key, request).await
    }

    pub async fn list_music(&self, api_key: &str) -> Result<Value> {
        self.get_json("/v1/music", api_key).await
    }

    pub async fn retrieve_music(&self, api_key: &str, music_id: &str) -> Result<Value> {
        self.get_json(&format!("/v1/music/{music_id}"), api_key)
            .await
    }

    pub async fn delete_music(&self, api_key: &str, music_id: &str) -> Result<Value> {
        self.delete_json(&format!("/v1/music/{music_id}"), api_key)
            .await
    }

    pub async fn music_content(
        &self,
        api_key: &str,
        music_id: &str,
    ) -> Result<ProviderStreamOutput> {
        self.get_stream(&format!("/v1/music/{music_id}/content"), api_key)
            .await
    }

    pub async fn music_lyrics(
        &self,
        api_key: &str,
        request: &CreateMusicLyricsRequest,
    ) -> Result<Value> {
        self.post_json("/v1/music/lyrics", api_key, request).await
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
}
