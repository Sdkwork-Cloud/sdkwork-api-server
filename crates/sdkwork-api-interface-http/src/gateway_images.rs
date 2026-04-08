fn local_image_generation_gateway_result(
    tenant_id: &str,
    project_id: &str,
    model: &str,
) -> anyhow::Result<ImagesResponse> {
    create_image_generation(tenant_id, project_id, model)
}

fn local_image_generation_result(
    tenant_id: &str,
    project_id: &str,
    model: &str,
) -> std::result::Result<ImagesResponse, Response> {
    local_image_generation_gateway_result(tenant_id, project_id, model)
        .map_err(local_response_invalid_model_response)
}

fn local_image_generation_response(tenant_id: &str, project_id: &str, model: &str) -> Response {
    match local_image_generation_result(tenant_id, project_id, model) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_chat_completion_retrieve_result(
    tenant_id: &str,
    project_id: &str,
    completion_id: &str,
) -> std::result::Result<ChatCompletionResponse, Response> {
    get_chat_completion(tenant_id, project_id, completion_id)
        .map_err(local_chat_completion_not_found_response)
}

fn local_chat_completion_retrieve_response(
    tenant_id: &str,
    project_id: &str,
    completion_id: &str,
) -> Response {
    match local_chat_completion_retrieve_result(tenant_id, project_id, completion_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_chat_completion_update_result(
    tenant_id: &str,
    project_id: &str,
    completion_id: &str,
    metadata: Value,
) -> std::result::Result<ChatCompletionResponse, Response> {
    update_chat_completion(tenant_id, project_id, completion_id, metadata)
        .map_err(local_chat_completion_not_found_response)
}

fn local_chat_completion_update_response(
    tenant_id: &str,
    project_id: &str,
    completion_id: &str,
    metadata: Value,
) -> Response {
    match local_chat_completion_update_result(tenant_id, project_id, completion_id, metadata) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_chat_completion_delete_result(
    tenant_id: &str,
    project_id: &str,
    completion_id: &str,
) -> std::result::Result<DeleteChatCompletionResponse, Response> {
    delete_chat_completion(tenant_id, project_id, completion_id)
        .map_err(local_chat_completion_not_found_response)
}

fn local_chat_completion_delete_response(
    tenant_id: &str,
    project_id: &str,
    completion_id: &str,
) -> Response {
    match local_chat_completion_delete_result(tenant_id, project_id, completion_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_chat_completion_stream_body_response() -> Response {
    let body = format!(
        "{}{}",
        SseFrame::data("{\"id\":\"chatcmpl_1\",\"object\":\"chat.completion.chunk\"}"),
        SseFrame::data("[DONE]")
    );
    ([(header::CONTENT_TYPE, "text/event-stream")], body).into_response()
}

fn local_chat_completion_messages_result(
    tenant_id: &str,
    project_id: &str,
    completion_id: &str,
) -> std::result::Result<ListChatCompletionMessagesResponse, Response> {
    list_chat_completion_messages(tenant_id, project_id, completion_id).map_err(|error| {
        local_gateway_error_response(error, "Requested chat completion was not found.")
    })
}

fn local_chat_completion_messages_response(
    tenant_id: &str,
    project_id: &str,
    completion_id: &str,
) -> Response {
    match local_chat_completion_messages_result(tenant_id, project_id, completion_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_conversation_not_found_response(error: anyhow::Error) -> Response {
    local_gateway_error_response(error, "Requested conversation was not found.")
}

fn local_conversation_retrieve_result(
    tenant_id: &str,
    project_id: &str,
    conversation_id: &str,
) -> std::result::Result<ConversationObject, Response> {
    get_conversation(tenant_id, project_id, conversation_id)
        .map_err(local_conversation_not_found_response)
}

fn local_conversation_retrieve_response(
    tenant_id: &str,
    project_id: &str,
    conversation_id: &str,
) -> Response {
    match local_conversation_retrieve_result(tenant_id, project_id, conversation_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_conversation_update_result(
    tenant_id: &str,
    project_id: &str,
    conversation_id: &str,
    metadata: Value,
) -> std::result::Result<ConversationObject, Response> {
    update_conversation(tenant_id, project_id, conversation_id, metadata)
        .map_err(local_conversation_not_found_response)
}

fn local_conversation_update_response(
    tenant_id: &str,
    project_id: &str,
    conversation_id: &str,
    metadata: Value,
) -> Response {
    match local_conversation_update_result(tenant_id, project_id, conversation_id, metadata) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_conversation_delete_result(
    tenant_id: &str,
    project_id: &str,
    conversation_id: &str,
) -> std::result::Result<DeleteConversationResponse, Response> {
    delete_conversation(tenant_id, project_id, conversation_id)
        .map_err(local_conversation_not_found_response)
}

fn local_conversation_delete_response(
    tenant_id: &str,
    project_id: &str,
    conversation_id: &str,
) -> Response {
    match local_conversation_delete_result(tenant_id, project_id, conversation_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_conversation_items_create_result(
    tenant_id: &str,
    project_id: &str,
    conversation_id: &str,
) -> std::result::Result<ListConversationItemsResponse, Response> {
    create_conversation_items(tenant_id, project_id, conversation_id)
        .map_err(local_conversation_not_found_response)
}

fn local_conversation_items_create_response(
    tenant_id: &str,
    project_id: &str,
    conversation_id: &str,
) -> Response {
    match local_conversation_items_create_result(tenant_id, project_id, conversation_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_conversation_items_list_result(
    tenant_id: &str,
    project_id: &str,
    conversation_id: &str,
) -> std::result::Result<ListConversationItemsResponse, Response> {
    list_conversation_items(tenant_id, project_id, conversation_id)
        .map_err(local_conversation_not_found_response)
}

fn local_conversation_items_list_response(
    tenant_id: &str,
    project_id: &str,
    conversation_id: &str,
) -> Response {
    match local_conversation_items_list_result(tenant_id, project_id, conversation_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_conversation_item_not_found_response(error: anyhow::Error) -> Response {
    local_gateway_error_response(error, "Requested conversation item was not found.")
}

fn local_conversation_item_retrieve_result(
    tenant_id: &str,
    project_id: &str,
    conversation_id: &str,
    item_id: &str,
) -> std::result::Result<ConversationItemObject, Response> {
    get_conversation_item(tenant_id, project_id, conversation_id, item_id)
        .map_err(local_conversation_item_not_found_response)
}

fn local_conversation_item_retrieve_response(
    tenant_id: &str,
    project_id: &str,
    conversation_id: &str,
    item_id: &str,
) -> Response {
    match local_conversation_item_retrieve_result(tenant_id, project_id, conversation_id, item_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_conversation_item_delete_result(
    tenant_id: &str,
    project_id: &str,
    conversation_id: &str,
    item_id: &str,
) -> std::result::Result<DeleteConversationItemResponse, Response> {
    delete_conversation_item(tenant_id, project_id, conversation_id, item_id)
        .map_err(local_conversation_item_not_found_response)
}

fn local_conversation_item_delete_response(
    tenant_id: &str,
    project_id: &str,
    conversation_id: &str,
    item_id: &str,
) -> Response {
    match local_conversation_item_delete_result(tenant_id, project_id, conversation_id, item_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_thread_not_found_response(error: anyhow::Error) -> Response {
    local_gateway_error_response(error, "Requested thread was not found.")
}

fn local_thread_retrieve_result(
    tenant_id: &str,
    project_id: &str,
    thread_id: &str,
) -> std::result::Result<ThreadObject, Response> {
    get_thread(tenant_id, project_id, thread_id).map_err(local_thread_not_found_response)
}

fn local_thread_retrieve_response(tenant_id: &str, project_id: &str, thread_id: &str) -> Response {
    match local_thread_retrieve_result(tenant_id, project_id, thread_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_thread_update_result(
    tenant_id: &str,
    project_id: &str,
    thread_id: &str,
) -> std::result::Result<ThreadObject, Response> {
    update_thread(tenant_id, project_id, thread_id).map_err(local_thread_not_found_response)
}

fn local_thread_update_response(tenant_id: &str, project_id: &str, thread_id: &str) -> Response {
    match local_thread_update_result(tenant_id, project_id, thread_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_thread_delete_result(
    tenant_id: &str,
    project_id: &str,
    thread_id: &str,
) -> std::result::Result<DeleteThreadResponse, Response> {
    delete_thread(tenant_id, project_id, thread_id).map_err(local_thread_not_found_response)
}

fn local_thread_delete_response(tenant_id: &str, project_id: &str, thread_id: &str) -> Response {
    match local_thread_delete_result(tenant_id, project_id, thread_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_thread_message_not_found_response(error: anyhow::Error) -> Response {
    if error
        .to_string()
        .to_ascii_lowercase()
        .contains("thread message not found")
    {
        return not_found_openai_response("Requested thread message was not found.");
    }

    local_thread_not_found_response(error)
}

fn local_thread_messages_create_result(
    tenant_id: &str,
    project_id: &str,
    thread_id: &str,
    role: &str,
    text: &str,
) -> std::result::Result<ThreadMessageObject, Response> {
    create_thread_message(tenant_id, project_id, thread_id, role, text)
        .map_err(local_thread_not_found_response)
}

fn local_thread_messages_list_result(
    tenant_id: &str,
    project_id: &str,
    thread_id: &str,
) -> std::result::Result<ListThreadMessagesResponse, Response> {
    list_thread_messages(tenant_id, project_id, thread_id).map_err(local_thread_not_found_response)
}

fn local_thread_messages_list_response(
    tenant_id: &str,
    project_id: &str,
    thread_id: &str,
) -> Response {
    match local_thread_messages_list_result(tenant_id, project_id, thread_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_thread_message_retrieve_result(
    tenant_id: &str,
    project_id: &str,
    thread_id: &str,
    message_id: &str,
) -> std::result::Result<ThreadMessageObject, Response> {
    get_thread_message(tenant_id, project_id, thread_id, message_id)
        .map_err(local_thread_message_not_found_response)
}

fn local_thread_message_retrieve_response(
    tenant_id: &str,
    project_id: &str,
    thread_id: &str,
    message_id: &str,
) -> Response {
    match local_thread_message_retrieve_result(tenant_id, project_id, thread_id, message_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_thread_message_update_result(
    tenant_id: &str,
    project_id: &str,
    thread_id: &str,
    message_id: &str,
) -> std::result::Result<ThreadMessageObject, Response> {
    update_thread_message(tenant_id, project_id, thread_id, message_id)
        .map_err(local_thread_message_not_found_response)
}

fn local_thread_message_update_response(
    tenant_id: &str,
    project_id: &str,
    thread_id: &str,
    message_id: &str,
) -> Response {
    match local_thread_message_update_result(tenant_id, project_id, thread_id, message_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_thread_message_delete_result(
    tenant_id: &str,
    project_id: &str,
    thread_id: &str,
    message_id: &str,
) -> std::result::Result<DeleteThreadMessageResponse, Response> {
    delete_thread_message(tenant_id, project_id, thread_id, message_id)
        .map_err(local_thread_message_not_found_response)
}

fn local_thread_message_delete_response(
    tenant_id: &str,
    project_id: &str,
    thread_id: &str,
    message_id: &str,
) -> Response {
    match local_thread_message_delete_result(tenant_id, project_id, thread_id, message_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_thread_run_not_found_response(error: anyhow::Error) -> Response {
    if error
        .to_string()
        .to_ascii_lowercase()
        .contains("run not found")
    {
        return not_found_openai_response("Requested run was not found.");
    }

    local_thread_not_found_response(error)
}

fn local_thread_run_step_not_found_response(error: anyhow::Error) -> Response {
    if error
        .to_string()
        .to_ascii_lowercase()
        .contains("run step not found")
    {
        return not_found_openai_response("Requested run step was not found.");
    }

    local_thread_run_not_found_response(error)
}

fn local_thread_runs_create_result(
    tenant_id: &str,
    project_id: &str,
    thread_id: &str,
    assistant_id: &str,
    model: Option<&str>,
) -> std::result::Result<RunObject, Response> {
    create_thread_run(tenant_id, project_id, thread_id, assistant_id, model)
        .map_err(local_thread_not_found_response)
}

fn local_thread_runs_list_result(
    tenant_id: &str,
    project_id: &str,
    thread_id: &str,
) -> std::result::Result<ListRunsResponse, Response> {
    list_thread_runs(tenant_id, project_id, thread_id).map_err(local_thread_not_found_response)
}

fn local_thread_runs_list_response(tenant_id: &str, project_id: &str, thread_id: &str) -> Response {
    match local_thread_runs_list_result(tenant_id, project_id, thread_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_thread_run_retrieve_result(
    tenant_id: &str,
    project_id: &str,
    thread_id: &str,
    run_id: &str,
) -> std::result::Result<RunObject, Response> {
    get_thread_run(tenant_id, project_id, thread_id, run_id)
        .map_err(local_thread_run_not_found_response)
}

fn local_thread_run_retrieve_response(
    tenant_id: &str,
    project_id: &str,
    thread_id: &str,
    run_id: &str,
) -> Response {
    match local_thread_run_retrieve_result(tenant_id, project_id, thread_id, run_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_thread_run_update_result(
    tenant_id: &str,
    project_id: &str,
    thread_id: &str,
    run_id: &str,
) -> std::result::Result<RunObject, Response> {
    update_thread_run(tenant_id, project_id, thread_id, run_id)
        .map_err(local_thread_run_not_found_response)
}

fn local_thread_run_update_response(
    tenant_id: &str,
    project_id: &str,
    thread_id: &str,
    run_id: &str,
) -> Response {
    match local_thread_run_update_result(tenant_id, project_id, thread_id, run_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_thread_run_cancel_result(
    tenant_id: &str,
    project_id: &str,
    thread_id: &str,
    run_id: &str,
) -> std::result::Result<RunObject, Response> {
    cancel_thread_run(tenant_id, project_id, thread_id, run_id)
        .map_err(local_thread_run_not_found_response)
}

fn local_thread_run_cancel_response(
    tenant_id: &str,
    project_id: &str,
    thread_id: &str,
    run_id: &str,
) -> Response {
    match local_thread_run_cancel_result(tenant_id, project_id, thread_id, run_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_thread_run_submit_tool_outputs_result(
    tenant_id: &str,
    project_id: &str,
    thread_id: &str,
    run_id: &str,
    tool_outputs: Vec<(&str, &str)>,
) -> std::result::Result<RunObject, Response> {
    submit_thread_run_tool_outputs(tenant_id, project_id, thread_id, run_id, tool_outputs)
        .map_err(local_thread_run_not_found_response)
}

fn local_thread_run_steps_list_result(
    tenant_id: &str,
    project_id: &str,
    thread_id: &str,
    run_id: &str,
) -> std::result::Result<ListRunStepsResponse, Response> {
    list_thread_run_steps(tenant_id, project_id, thread_id, run_id)
        .map_err(local_thread_run_not_found_response)
}

fn local_thread_run_steps_list_response(
    tenant_id: &str,
    project_id: &str,
    thread_id: &str,
    run_id: &str,
) -> Response {
    match local_thread_run_steps_list_result(tenant_id, project_id, thread_id, run_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_thread_run_step_retrieve_result(
    tenant_id: &str,
    project_id: &str,
    thread_id: &str,
    run_id: &str,
    step_id: &str,
) -> std::result::Result<RunStepObject, Response> {
    get_thread_run_step(tenant_id, project_id, thread_id, run_id, step_id)
        .map_err(local_thread_run_step_not_found_response)
}

fn local_thread_run_step_retrieve_response(
    tenant_id: &str,
    project_id: &str,
    thread_id: &str,
    run_id: &str,
    step_id: &str,
) -> Response {
    match local_thread_run_step_retrieve_result(tenant_id, project_id, thread_id, run_id, step_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}


async fn image_generations_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateImageRequest>,
) -> Response {
    match relay_image_generation_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project_with_media_and_reference_id(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "images",
                &request.model,
                50,
                0.05,
                BillingMediaMetrics {
                    image_count: image_count_from_response(&response),
                    ..BillingMediaMetrics::default()
                },
                response_usage_id_or_single_data_item_id(&response),
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream image generation");
        }
    }

    let response = match local_image_generation_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &request.model,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project_with_media_and_reference_id(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "images",
        &request.model,
        50,
        0.05,
        BillingMediaMetrics {
            image_count: u64::try_from(response.data.len()).unwrap_or(u64::MAX),
            ..BillingMediaMetrics::default()
        },
        None,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn image_edits_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    multipart: Multipart,
) -> Response {
    let request = match parse_image_edit_request(multipart).await {
        Ok(request) => request,
        Err(response) => return response,
    };
    let route_model = request.model_or_default().to_owned();

    match relay_image_edit_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project_with_media_and_reference_id(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "images",
                &route_model,
                50,
                0.05,
                BillingMediaMetrics {
                    image_count: image_count_from_response(&response),
                    ..BillingMediaMetrics::default()
                },
                response_usage_id_or_single_data_item_id(&response),
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream image edit");
        }
    }

    let response = create_image_edit(
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .expect("image edit");

    if record_gateway_usage_for_project_with_media_and_reference_id(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "images",
        &route_model,
        50,
        0.05,
        BillingMediaMetrics {
            image_count: u64::try_from(response.data.len()).unwrap_or(u64::MAX),
            ..BillingMediaMetrics::default()
        },
        None,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn image_variations_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    multipart: Multipart,
) -> Response {
    let request = match parse_image_variation_request(multipart).await {
        Ok(request) => request,
        Err(response) => return response,
    };
    let route_model = request.model_or_default().to_owned();

    match relay_image_variation_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project_with_media_and_reference_id(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "images",
                &route_model,
                50,
                0.05,
                BillingMediaMetrics {
                    image_count: image_count_from_response(&response),
                    ..BillingMediaMetrics::default()
                },
                response_usage_id_or_single_data_item_id(&response),
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream image variation");
        }
    }

    let response = create_image_variation(
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .expect("image variation");

    if record_gateway_usage_for_project_with_media_and_reference_id(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "images",
        &route_model,
        50,
        0.05,
        BillingMediaMetrics {
            image_count: u64::try_from(response.data.len()).unwrap_or(u64::MAX),
            ..BillingMediaMetrics::default()
        },
        None,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}


async fn parse_image_edit_request(
    mut multipart: Multipart,
) -> Result<CreateImageEditRequest, Response> {
    let mut model = None;
    let mut prompt = None;
    let mut image = None;
    let mut mask = None;
    let mut n = None;
    let mut quality = None;
    let mut response_format = None;
    let mut size = None;
    let mut user = None;

    while let Some(field) = multipart.next_field().await.map_err(bad_multipart)? {
        match field.name() {
            Some("model") => model = Some(field.text().await.map_err(bad_multipart)?),
            Some("prompt") => prompt = Some(field.text().await.map_err(bad_multipart)?),
            Some("image") => image = Some(parse_image_upload_field(field).await?),
            Some("mask") => mask = Some(parse_image_upload_field(field).await?),
            Some("n") => {
                n = Some(
                    parse_u32_field(field.text().await.map_err(bad_multipart)?).map_err(
                        |message| (axum::http::StatusCode::BAD_REQUEST, message).into_response(),
                    )?,
                )
            }
            Some("quality") => quality = Some(field.text().await.map_err(bad_multipart)?),
            Some("response_format") => {
                response_format = Some(field.text().await.map_err(bad_multipart)?)
            }
            Some("size") => size = Some(field.text().await.map_err(bad_multipart)?),
            Some("user") => user = Some(field.text().await.map_err(bad_multipart)?),
            _ => {}
        }
    }

    let mut request = CreateImageEditRequest::new(
        prompt.ok_or_else(missing_multipart_field)?,
        image.ok_or_else(missing_multipart_field)?,
    );
    if let Some(model) = model {
        request = request.with_model(model);
    }
    if let Some(mask) = mask {
        request = request.with_mask(mask);
    }
    request.n = n;
    request.quality = quality;
    request.response_format = response_format;
    request.size = size;
    request.user = user;

    Ok(request)
}

async fn parse_image_variation_request(
    mut multipart: Multipart,
) -> Result<CreateImageVariationRequest, Response> {
    let mut model = None;
    let mut image = None;
    let mut n = None;
    let mut response_format = None;
    let mut size = None;
    let mut user = None;

    while let Some(field) = multipart.next_field().await.map_err(bad_multipart)? {
        match field.name() {
            Some("model") => model = Some(field.text().await.map_err(bad_multipart)?),
            Some("image") => image = Some(parse_image_upload_field(field).await?),
            Some("n") => {
                n = Some(
                    parse_u32_field(field.text().await.map_err(bad_multipart)?).map_err(
                        |message| (axum::http::StatusCode::BAD_REQUEST, message).into_response(),
                    )?,
                )
            }
            Some("response_format") => {
                response_format = Some(field.text().await.map_err(bad_multipart)?)
            }
            Some("size") => size = Some(field.text().await.map_err(bad_multipart)?),
            Some("user") => user = Some(field.text().await.map_err(bad_multipart)?),
            _ => {}
        }
    }

    let mut request = CreateImageVariationRequest::new(image.ok_or_else(missing_multipart_field)?);
    if let Some(model) = model {
        request = request.with_model(model);
    }
    request.n = n;
    request.response_format = response_format;
    request.size = size;
    request.user = user;

    Ok(request)
}

async fn parse_image_upload_field(
    field: axum::extract::multipart::Field<'_>,
) -> Result<ImageUpload, Response> {
    let filename = field
        .file_name()
        .map(ToOwned::to_owned)
        .ok_or_else(missing_multipart_field)?;
    let content_type = field.content_type().map(ToOwned::to_owned);
    let bytes = field.bytes().await.map_err(bad_multipart)?.to_vec();
    let mut upload = ImageUpload::new(filename, bytes);
    if let Some(content_type) = content_type {
        upload = upload.with_content_type(content_type);
    }
    Ok(upload)
}

fn parse_u32_field(value: String) -> Result<u32, &'static str> {
    value
        .parse::<u32>()
        .map_err(|_| "invalid numeric multipart field")
}

