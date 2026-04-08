use super::support::*;

#[tokio::test]
async fn adapter_posts_fine_tuning_jobs_to_openai_compatible_upstream() {
    let state = CaptureState::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let app = Router::new()
        .route("/v1/fine_tuning/jobs", post(capture_fine_tuning_request))
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter =
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));
    let request = sdkwork_api_contract_openai::fine_tuning::CreateFineTuningJobRequest::new(
        "file_1",
        "gpt-4.1-mini",
    );

    let response = adapter
        .fine_tuning_jobs("sk-upstream-openai", &request)
        .await
        .unwrap();

    assert_eq!(response["id"], "ftjob_upstream");
    assert_eq!(
        state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-upstream-openai")
    );
    assert_eq!(
        state.body.lock().unwrap().as_ref().unwrap()["training_file"],
        "file_1"
    );
}

#[tokio::test]
async fn adapter_lists_fine_tuning_jobs_from_openai_compatible_upstream() {
    let state = CaptureState::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let app = Router::new()
        .route(
            "/v1/fine_tuning/jobs",
            get(capture_fine_tuning_list_request),
        )
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter =
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));
    let response = adapter
        .list_fine_tuning_jobs("sk-upstream-openai")
        .await
        .unwrap();

    assert_eq!(response["object"], "list");
    assert_eq!(response["data"][0]["id"], "ftjob_1");
    assert_eq!(
        state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-upstream-openai")
    );
}

#[tokio::test]
async fn adapter_retrieves_fine_tuning_job_from_openai_compatible_upstream() {
    let state = CaptureState::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let app = Router::new()
        .route(
            "/v1/fine_tuning/jobs/ftjob_1",
            get(capture_fine_tuning_retrieve_request),
        )
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter =
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));
    let response = adapter
        .retrieve_fine_tuning_job("sk-upstream-openai", "ftjob_1")
        .await
        .unwrap();

    assert_eq!(response["id"], "ftjob_1");
    assert_eq!(
        state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-upstream-openai")
    );
}

#[tokio::test]
async fn adapter_cancels_fine_tuning_job_on_openai_compatible_upstream() {
    let state = CaptureState::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let app = Router::new()
        .route(
            "/v1/fine_tuning/jobs/ftjob_1/cancel",
            post(capture_fine_tuning_cancel_request),
        )
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter =
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));
    let response = adapter
        .cancel_fine_tuning_job("sk-upstream-openai", "ftjob_1")
        .await
        .unwrap();

    assert_eq!(response["status"], "cancelled");
    assert_eq!(
        state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-upstream-openai")
    );
    assert_eq!(state.raw_body.lock().unwrap().as_deref(), Some(&[][..]));
}

#[tokio::test]
async fn adapter_posts_assistants_to_openai_compatible_upstream() {
    let state = CaptureState::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let app = Router::new()
        .route("/v1/assistants", post(capture_assistant_request))
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter =
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));
    let request =
        sdkwork_api_contract_openai::assistants::CreateAssistantRequest::new("Support", "gpt-4.1");

    let response = adapter
        .assistants("sk-upstream-openai", &request)
        .await
        .unwrap();

    assert_eq!(response["id"], "asst_upstream");
    assert_eq!(
        state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-upstream-openai")
    );
    assert_eq!(
        state.body.lock().unwrap().as_ref().unwrap()["name"],
        "Support"
    );
}

#[tokio::test]
async fn adapter_posts_webhooks_to_openai_compatible_upstream() {
    let state = CaptureState::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let app = Router::new()
        .route("/v1/webhooks", post(capture_webhook_request))
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter =
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));
    let request = sdkwork_api_contract_openai::webhooks::CreateWebhookRequest::new(
        "https://example.com/webhook",
        vec!["response.completed"],
    );

    let response = adapter
        .webhooks("sk-upstream-openai", &request)
        .await
        .unwrap();

    assert_eq!(response["id"], "wh_upstream");
    assert_eq!(
        state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-upstream-openai")
    );
}

#[tokio::test]
async fn adapter_lists_webhooks_from_openai_compatible_upstream() {
    let state = CaptureState::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let app = Router::new()
        .route("/v1/webhooks", get(capture_webhooks_list_request))
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter =
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));
    let response = adapter.list_webhooks("sk-upstream-openai").await.unwrap();

    assert_eq!(response["object"], "list");
    assert_eq!(response["data"][0]["id"], "wh_1");
}

#[tokio::test]
async fn adapter_retrieves_webhook_from_openai_compatible_upstream() {
    let state = CaptureState::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let app = Router::new()
        .route("/v1/webhooks/wh_1", get(capture_webhook_retrieve_request))
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter =
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));
    let response = adapter
        .retrieve_webhook("sk-upstream-openai", "wh_1")
        .await
        .unwrap();

    assert_eq!(response["id"], "wh_1");
}

#[tokio::test]
async fn adapter_updates_webhook_on_openai_compatible_upstream() {
    let state = CaptureState::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let app = Router::new()
        .route("/v1/webhooks/wh_1", post(capture_webhook_update_request))
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter =
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));
    let request = sdkwork_api_contract_openai::webhooks::UpdateWebhookRequest::new(
        "https://example.com/webhook/v2",
    );

    let response = adapter
        .update_webhook("sk-upstream-openai", "wh_1", &request)
        .await
        .unwrap();

    assert_eq!(response["url"], "https://example.com/webhook/v2");
}

#[tokio::test]
async fn adapter_deletes_webhook_on_openai_compatible_upstream() {
    let state = CaptureState::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let app = Router::new()
        .route("/v1/webhooks/wh_1", delete(capture_webhook_delete_request))
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter =
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));
    let response = adapter
        .delete_webhook("sk-upstream-openai", "wh_1")
        .await
        .unwrap();

    assert_eq!(response["deleted"], true);
}

#[tokio::test]
async fn adapter_lists_assistants_from_openai_compatible_upstream() {
    let state = CaptureState::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let app = Router::new()
        .route("/v1/assistants", get(capture_assistants_list_request))
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter =
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));
    let response = adapter.list_assistants("sk-upstream-openai").await.unwrap();

    assert_eq!(response["object"], "list");
    assert_eq!(response["data"][0]["id"], "asst_1");
    assert_eq!(
        state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-upstream-openai")
    );
}

#[tokio::test]
async fn adapter_retrieves_assistant_from_openai_compatible_upstream() {
    let state = CaptureState::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let app = Router::new()
        .route(
            "/v1/assistants/asst_1",
            get(capture_assistant_retrieve_request),
        )
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter =
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));
    let response = adapter
        .retrieve_assistant("sk-upstream-openai", "asst_1")
        .await
        .unwrap();

    assert_eq!(response["id"], "asst_1");
    assert_eq!(
        state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-upstream-openai")
    );
}

#[tokio::test]
async fn adapter_updates_assistant_on_openai_compatible_upstream() {
    let state = CaptureState::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let app = Router::new()
        .route(
            "/v1/assistants/asst_1",
            post(capture_assistant_update_request),
        )
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter =
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));
    let request =
        sdkwork_api_contract_openai::assistants::UpdateAssistantRequest::new("Support v2");

    let response = adapter
        .update_assistant("sk-upstream-openai", "asst_1", &request)
        .await
        .unwrap();

    assert_eq!(response["id"], "asst_1");
    assert_eq!(response["name"], "Support v2");
    assert_eq!(
        state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-upstream-openai")
    );
    assert_eq!(
        state.body.lock().unwrap().as_ref().unwrap()["name"],
        "Support v2"
    );
}

#[tokio::test]
async fn adapter_deletes_assistant_on_openai_compatible_upstream() {
    let state = CaptureState::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let app = Router::new()
        .route(
            "/v1/assistants/asst_1",
            delete(capture_assistant_delete_request),
        )
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter =
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));
    let response = adapter
        .delete_assistant("sk-upstream-openai", "asst_1")
        .await
        .unwrap();

    assert_eq!(response["deleted"], true);
    assert_eq!(
        state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-upstream-openai")
    );
}

#[tokio::test]
async fn adapter_posts_realtime_sessions_to_openai_compatible_upstream() {
    let state = CaptureState::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let app = Router::new()
        .route(
            "/v1/realtime/sessions",
            post(capture_realtime_session_request),
        )
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter =
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));
    let request = sdkwork_api_contract_openai::realtime::CreateRealtimeSessionRequest::new(
        "gpt-4o-realtime-preview",
    );

    let response = adapter
        .realtime_sessions("sk-upstream-openai", &request)
        .await
        .unwrap();

    assert_eq!(response["id"], "sess_upstream");
    assert_eq!(
        state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-upstream-openai")
    );
    assert_eq!(
        state.body.lock().unwrap().as_ref().unwrap()["model"],
        "gpt-4o-realtime-preview"
    );
}


async fn capture_fine_tuning_request(
    State(state): State<CaptureState>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> (StatusCode, Json<Value>) {
    *state.authorization.lock().unwrap() = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);
    *state.body.lock().unwrap() = Some(body);

    (
        StatusCode::OK,
        Json(json!({
            "id":"ftjob_upstream",
            "object":"fine_tuning.job",
            "model":"gpt-4.1-mini",
            "status":"queued"
        })),
    )
}

async fn capture_fine_tuning_list_request(
    State(state): State<CaptureState>,
    headers: HeaderMap,
) -> (StatusCode, Json<Value>) {
    *state.authorization.lock().unwrap() = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);

    (
        StatusCode::OK,
        Json(json!({
            "object":"list",
            "data":[{
                "id":"ftjob_1",
                "object":"fine_tuning.job",
                "model":"gpt-4.1-mini",
                "status":"queued"
            }]
        })),
    )
}

async fn capture_fine_tuning_retrieve_request(
    State(state): State<CaptureState>,
    headers: HeaderMap,
) -> (StatusCode, Json<Value>) {
    *state.authorization.lock().unwrap() = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);

    (
        StatusCode::OK,
        Json(json!({
            "id":"ftjob_1",
            "object":"fine_tuning.job",
            "model":"gpt-4.1-mini",
            "status":"running"
        })),
    )
}

async fn capture_fine_tuning_cancel_request(
    State(state): State<CaptureState>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> (StatusCode, Json<Value>) {
    *state.authorization.lock().unwrap() = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);
    *state.raw_body.lock().unwrap() = Some(body.to_vec());

    (
        StatusCode::OK,
        Json(json!({
            "id":"ftjob_1",
            "object":"fine_tuning.job",
            "model":"gpt-4.1-mini",
            "status":"cancelled"
        })),
    )
}


async fn capture_assistant_request(
    State(state): State<CaptureState>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> (StatusCode, Json<Value>) {
    *state.authorization.lock().unwrap() = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);
    *state.body.lock().unwrap() = Some(body);

    (
        StatusCode::OK,
        Json(json!({
            "id":"asst_upstream",
            "object":"assistant",
            "name":"Support",
            "model":"gpt-4.1"
        })),
    )
}

async fn capture_webhook_request(
    State(state): State<CaptureState>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> (StatusCode, Json<Value>) {
    *state.authorization.lock().unwrap() = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);
    *state.body.lock().unwrap() = Some(body);

    (
        StatusCode::OK,
        Json(json!({
            "id":"wh_upstream",
            "object":"webhook_endpoint",
            "url":"https://example.com/webhook",
            "status":"enabled"
        })),
    )
}

async fn capture_webhooks_list_request(
    State(state): State<CaptureState>,
    headers: HeaderMap,
) -> (StatusCode, Json<Value>) {
    *state.authorization.lock().unwrap() = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);

    (
        StatusCode::OK,
        Json(json!({
            "object":"list",
            "data":[{
                "id":"wh_1",
                "object":"webhook_endpoint",
                "url":"https://example.com/webhook",
                "status":"enabled"
            }]
        })),
    )
}

async fn capture_webhook_retrieve_request(
    State(state): State<CaptureState>,
    headers: HeaderMap,
) -> (StatusCode, Json<Value>) {
    *state.authorization.lock().unwrap() = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);

    (
        StatusCode::OK,
        Json(json!({
            "id":"wh_1",
            "object":"webhook_endpoint",
            "url":"https://example.com/webhook",
            "status":"enabled"
        })),
    )
}

async fn capture_webhook_update_request(
    State(state): State<CaptureState>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> (StatusCode, Json<Value>) {
    *state.authorization.lock().unwrap() = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);
    *state.body.lock().unwrap() = Some(body);

    (
        StatusCode::OK,
        Json(json!({
            "id":"wh_1",
            "object":"webhook_endpoint",
            "url":"https://example.com/webhook/v2",
            "status":"enabled"
        })),
    )
}

async fn capture_webhook_delete_request(
    State(state): State<CaptureState>,
    headers: HeaderMap,
) -> (StatusCode, Json<Value>) {
    *state.authorization.lock().unwrap() = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);

    (
        StatusCode::OK,
        Json(json!({
            "id":"wh_1",
            "object":"webhook_endpoint.deleted",
            "deleted":true
        })),
    )
}

async fn capture_assistants_list_request(
    State(state): State<CaptureState>,
    headers: HeaderMap,
) -> (StatusCode, Json<Value>) {
    *state.authorization.lock().unwrap() = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);

    (
        StatusCode::OK,
        Json(json!({
            "object":"list",
            "data":[{
                "id":"asst_1",
                "object":"assistant",
                "name":"Support",
                "model":"gpt-4.1"
            }],
            "first_id":"asst_1",
            "last_id":"asst_1",
            "has_more":false
        })),
    )
}

async fn capture_assistant_retrieve_request(
    State(state): State<CaptureState>,
    headers: HeaderMap,
) -> (StatusCode, Json<Value>) {
    *state.authorization.lock().unwrap() = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);

    (
        StatusCode::OK,
        Json(json!({
            "id":"asst_1",
            "object":"assistant",
            "name":"Support",
            "model":"gpt-4.1"
        })),
    )
}

async fn capture_assistant_update_request(
    State(state): State<CaptureState>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> (StatusCode, Json<Value>) {
    *state.authorization.lock().unwrap() = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);
    *state.body.lock().unwrap() = Some(body);

    (
        StatusCode::OK,
        Json(json!({
            "id":"asst_1",
            "object":"assistant",
            "name":"Support v2",
            "model":"gpt-4.1"
        })),
    )
}

async fn capture_assistant_delete_request(
    State(state): State<CaptureState>,
    headers: HeaderMap,
) -> (StatusCode, Json<Value>) {
    *state.authorization.lock().unwrap() = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);

    (
        StatusCode::OK,
        Json(json!({
            "id":"asst_1",
            "object":"assistant.deleted",
            "deleted":true
        })),
    )
}

async fn capture_realtime_session_request(
    State(state): State<CaptureState>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> (StatusCode, Json<Value>) {
    *state.authorization.lock().unwrap() = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);
    *state.body.lock().unwrap() = Some(body);

    (
        StatusCode::OK,
        Json(json!({
            "id":"sess_upstream",
            "object":"realtime.session",
            "model":"gpt-4o-realtime-preview"
        })),
    )
}

