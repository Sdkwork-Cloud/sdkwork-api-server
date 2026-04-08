use super::*;

impl OpenAiProviderAdapter {
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
            .await?;
        let response = self.require_success(response).await?;

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
}
