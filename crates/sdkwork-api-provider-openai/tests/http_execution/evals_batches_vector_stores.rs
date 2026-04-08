use super::support::*;

#[tokio::test]
async fn adapter_posts_evals_to_openai_compatible_upstream() {
    let state = CaptureState::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let app = Router::new()
        .route("/v1/evals", post(capture_eval_request))
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter =
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));
    let request =
        sdkwork_api_contract_openai::evals::CreateEvalRequest::new("qa-benchmark", "file_1");

    let response = adapter.evals("sk-upstream-openai", &request).await.unwrap();

    assert_eq!(response["id"], "eval_upstream");
    assert_eq!(
        state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-upstream-openai")
    );
    assert_eq!(
        state.body.lock().unwrap().as_ref().unwrap()["name"],
        "qa-benchmark"
    );
}

#[tokio::test]
async fn adapter_posts_batches_to_openai_compatible_upstream() {
    let state = CaptureState::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let app = Router::new()
        .route("/v1/batches", post(capture_batch_request))
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter =
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));
    let request = sdkwork_api_contract_openai::batches::CreateBatchRequest::new(
        "file_1",
        "/v1/responses",
        "24h",
    );

    let response = adapter
        .batches("sk-upstream-openai", &request)
        .await
        .unwrap();

    assert_eq!(response["id"], "batch_upstream");
    assert_eq!(
        state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-upstream-openai")
    );
    assert_eq!(
        state.body.lock().unwrap().as_ref().unwrap()["endpoint"],
        "/v1/responses"
    );
}

#[tokio::test]
async fn adapter_lists_batches_from_openai_compatible_upstream() {
    let state = CaptureState::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let app = Router::new()
        .route("/v1/batches", get(capture_batches_list_request))
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter =
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));
    let response = adapter.list_batches("sk-upstream-openai").await.unwrap();

    assert_eq!(response["object"], "list");
    assert_eq!(response["data"][0]["id"], "batch_1");
    assert_eq!(
        state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-upstream-openai")
    );
}

#[tokio::test]
async fn adapter_retrieves_batch_from_openai_compatible_upstream() {
    let state = CaptureState::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let app = Router::new()
        .route("/v1/batches/batch_1", get(capture_batch_retrieve_request))
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter =
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));
    let response = adapter
        .retrieve_batch("sk-upstream-openai", "batch_1")
        .await
        .unwrap();

    assert_eq!(response["id"], "batch_1");
    assert_eq!(
        state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-upstream-openai")
    );
}

#[tokio::test]
async fn adapter_cancels_batch_on_openai_compatible_upstream() {
    let state = CaptureState::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let app = Router::new()
        .route(
            "/v1/batches/batch_1/cancel",
            post(capture_batch_cancel_request),
        )
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter =
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));
    let response = adapter
        .cancel_batch("sk-upstream-openai", "batch_1")
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
async fn adapter_posts_vector_stores_to_openai_compatible_upstream() {
    let state = CaptureState::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let app = Router::new()
        .route("/v1/vector_stores", post(capture_vector_store_request))
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter =
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));
    let request =
        sdkwork_api_contract_openai::vector_stores::CreateVectorStoreRequest::new("kb-main");

    let response = adapter
        .vector_stores("sk-upstream-openai", &request)
        .await
        .unwrap();

    assert_eq!(response["id"], "vs_upstream");
    assert_eq!(
        state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-upstream-openai")
    );
    assert_eq!(
        state.body.lock().unwrap().as_ref().unwrap()["name"],
        "kb-main"
    );
}

#[tokio::test]
async fn adapter_lists_vector_stores_from_openai_compatible_upstream() {
    let state = CaptureState::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let app = Router::new()
        .route("/v1/vector_stores", get(capture_vector_stores_list_request))
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter =
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));
    let response = adapter
        .list_vector_stores("sk-upstream-openai")
        .await
        .unwrap();

    assert_eq!(response["object"], "list");
    assert_eq!(response["data"][0]["id"], "vs_1");
    assert_eq!(
        state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-upstream-openai")
    );
}

#[tokio::test]
async fn adapter_retrieves_vector_store_from_openai_compatible_upstream() {
    let state = CaptureState::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let app = Router::new()
        .route(
            "/v1/vector_stores/vs_1",
            get(capture_vector_store_retrieve_request),
        )
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter =
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));
    let response = adapter
        .retrieve_vector_store("sk-upstream-openai", "vs_1")
        .await
        .unwrap();

    assert_eq!(response["id"], "vs_1");
    assert_eq!(
        state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-upstream-openai")
    );
}

#[tokio::test]
async fn adapter_modifies_vector_store_on_openai_compatible_upstream() {
    let state = CaptureState::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let app = Router::new()
        .route(
            "/v1/vector_stores/vs_1",
            post(capture_vector_store_update_request),
        )
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter =
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));
    let request =
        sdkwork_api_contract_openai::vector_stores::UpdateVectorStoreRequest::new("kb-updated");

    let response = adapter
        .update_vector_store("sk-upstream-openai", "vs_1", &request)
        .await
        .unwrap();

    assert_eq!(response["name"], "kb-updated");
    assert_eq!(
        state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-upstream-openai")
    );
    assert_eq!(
        state.body.lock().unwrap().as_ref().unwrap()["name"],
        "kb-updated"
    );
}

#[tokio::test]
async fn adapter_deletes_vector_store_on_openai_compatible_upstream() {
    let state = CaptureState::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let app = Router::new()
        .route(
            "/v1/vector_stores/vs_1",
            delete(capture_vector_store_delete_request),
        )
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter =
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));
    let response = adapter
        .delete_vector_store("sk-upstream-openai", "vs_1")
        .await
        .unwrap();

    assert_eq!(response["deleted"], true);
    assert_eq!(
        state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-upstream-openai")
    );
}

#[tokio::test]
async fn adapter_posts_vector_store_files_to_openai_compatible_upstream() {
    let state = CaptureState::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let app = Router::new()
        .route(
            "/v1/vector_stores/vs_1/files",
            post(capture_vector_store_file_request),
        )
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter =
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));
    let request =
        sdkwork_api_contract_openai::vector_stores::CreateVectorStoreFileRequest::new("file_1");

    let response = adapter
        .create_vector_store_file("sk-upstream-openai", "vs_1", &request)
        .await
        .unwrap();

    assert_eq!(response["id"], "file_1");
    assert_eq!(
        state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-upstream-openai")
    );
    assert_eq!(
        state.body.lock().unwrap().as_ref().unwrap()["file_id"],
        "file_1"
    );
}

#[tokio::test]
async fn adapter_lists_vector_store_files_from_openai_compatible_upstream() {
    let state = CaptureState::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let app = Router::new()
        .route(
            "/v1/vector_stores/vs_1/files",
            get(capture_vector_store_files_list_request),
        )
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter =
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));
    let response = adapter
        .list_vector_store_files("sk-upstream-openai", "vs_1")
        .await
        .unwrap();

    assert_eq!(response["object"], "list");
    assert_eq!(response["data"][0]["id"], "file_1");
    assert_eq!(
        state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-upstream-openai")
    );
}

#[tokio::test]
async fn adapter_retrieves_vector_store_file_from_openai_compatible_upstream() {
    let state = CaptureState::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let app = Router::new()
        .route(
            "/v1/vector_stores/vs_1/files/file_1",
            get(capture_vector_store_file_retrieve_request),
        )
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter =
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));
    let response = adapter
        .retrieve_vector_store_file("sk-upstream-openai", "vs_1", "file_1")
        .await
        .unwrap();

    assert_eq!(response["id"], "file_1");
    assert_eq!(
        state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-upstream-openai")
    );
}

#[tokio::test]
async fn adapter_deletes_vector_store_file_on_openai_compatible_upstream() {
    let state = CaptureState::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let app = Router::new()
        .route(
            "/v1/vector_stores/vs_1/files/file_1",
            delete(capture_vector_store_file_delete_request),
        )
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter =
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));
    let response = adapter
        .delete_vector_store_file("sk-upstream-openai", "vs_1", "file_1")
        .await
        .unwrap();

    assert_eq!(response["deleted"], true);
    assert_eq!(
        state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-upstream-openai")
    );
}

#[tokio::test]
async fn adapter_posts_vector_store_file_batches_to_openai_compatible_upstream() {
    let state = CaptureState::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let app = Router::new()
        .route(
            "/v1/vector_stores/vs_1/file_batches",
            post(capture_vector_store_file_batch_request),
        )
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter =
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));
    let request =
        sdkwork_api_contract_openai::vector_stores::CreateVectorStoreFileBatchRequest::new(vec![
            "file_1",
        ]);

    let response = adapter
        .create_vector_store_file_batch("sk-upstream-openai", "vs_1", &request)
        .await
        .unwrap();

    assert_eq!(response["id"], "vsfb_1");
    assert_eq!(
        state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-upstream-openai")
    );
    assert_eq!(
        state.body.lock().unwrap().as_ref().unwrap()["file_ids"][0],
        "file_1"
    );
}

#[tokio::test]
async fn adapter_retrieves_vector_store_file_batch_from_openai_compatible_upstream() {
    let state = CaptureState::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let app = Router::new()
        .route(
            "/v1/vector_stores/vs_1/file_batches/vsfb_1",
            get(capture_vector_store_file_batch_retrieve_request),
        )
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter =
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));
    let response = adapter
        .retrieve_vector_store_file_batch("sk-upstream-openai", "vs_1", "vsfb_1")
        .await
        .unwrap();

    assert_eq!(response["id"], "vsfb_1");
    assert_eq!(
        state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-upstream-openai")
    );
}

#[tokio::test]
async fn adapter_cancels_vector_store_file_batch_on_openai_compatible_upstream() {
    let state = CaptureState::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let app = Router::new()
        .route(
            "/v1/vector_stores/vs_1/file_batches/vsfb_1/cancel",
            post(capture_vector_store_file_batch_cancel_request),
        )
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter =
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));
    let response = adapter
        .cancel_vector_store_file_batch("sk-upstream-openai", "vs_1", "vsfb_1")
        .await
        .unwrap();

    assert_eq!(response["status"], "cancelled");
    assert_eq!(
        state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-upstream-openai")
    );
}

#[tokio::test]
async fn adapter_lists_vector_store_file_batch_files_from_openai_compatible_upstream() {
    let state = CaptureState::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let app = Router::new()
        .route(
            "/v1/vector_stores/vs_1/file_batches/vsfb_1/files",
            get(capture_vector_store_file_batch_files_request),
        )
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter =
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));
    let response = adapter
        .list_vector_store_file_batch_files("sk-upstream-openai", "vs_1", "vsfb_1")
        .await
        .unwrap();

    assert_eq!(response["object"], "list");
    assert_eq!(response["data"][0]["id"], "file_1");
    assert_eq!(
        state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-upstream-openai")
    );
}

#[tokio::test]
async fn adapter_searches_vector_store_on_openai_compatible_upstream() {
    let state = CaptureState::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let app = Router::new()
        .route(
            "/v1/vector_stores/vs_1/search",
            post(capture_vector_store_search_request),
        )
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter =
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));
    let request =
        sdkwork_api_contract_openai::vector_stores::SearchVectorStoreRequest::new("reset password");

    let response = adapter
        .search_vector_store("sk-upstream-openai", "vs_1", &request)
        .await
        .unwrap();

    assert_eq!(response["object"], "list");
    assert_eq!(
        state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-upstream-openai")
    );
    assert_eq!(
        state.body.lock().unwrap().as_ref().unwrap()["query"],
        "reset password"
    );
}


async fn capture_eval_request(
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
            "id":"eval_upstream",
            "object":"eval",
            "name":"qa-benchmark",
            "status":"queued"
        })),
    )
}

async fn capture_batch_request(
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
            "id":"batch_upstream",
            "object":"batch",
            "endpoint":"/v1/responses",
            "input_file_id":"file_1",
            "status":"validating"
        })),
    )
}

async fn capture_batches_list_request(
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
                "id":"batch_1",
                "object":"batch",
                "endpoint":"/v1/responses",
                "input_file_id":"file_1",
                "status":"validating"
            }]
        })),
    )
}

async fn capture_batch_retrieve_request(
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
            "id":"batch_1",
            "object":"batch",
            "endpoint":"/v1/responses",
            "input_file_id":"file_1",
            "status":"in_progress"
        })),
    )
}

async fn capture_batch_cancel_request(
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
            "id":"batch_1",
            "object":"batch",
            "endpoint":"/v1/responses",
            "input_file_id":"file_1",
            "status":"cancelled"
        })),
    )
}

async fn capture_vector_store_request(
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
            "id":"vs_upstream",
            "object":"vector_store",
            "name":"kb-main",
            "status":"completed"
        })),
    )
}

async fn capture_vector_stores_list_request(
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
                "id":"vs_1",
                "object":"vector_store",
                "name":"kb-main",
                "status":"completed"
            }]
        })),
    )
}

async fn capture_vector_store_retrieve_request(
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
            "id":"vs_1",
            "object":"vector_store",
            "name":"kb-main",
            "status":"completed"
        })),
    )
}

async fn capture_vector_store_update_request(
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
            "id":"vs_1",
            "object":"vector_store",
            "name":"kb-updated",
            "status":"completed"
        })),
    )
}

async fn capture_vector_store_delete_request(
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
            "id":"vs_1",
            "object":"vector_store.deleted",
            "deleted":true
        })),
    )
}

async fn capture_vector_store_file_request(
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
            "id":"file_1",
            "object":"vector_store.file",
            "status":"completed"
        })),
    )
}

async fn capture_vector_store_files_list_request(
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
                "id":"file_1",
                "object":"vector_store.file",
                "status":"completed"
            }]
        })),
    )
}

async fn capture_vector_store_file_retrieve_request(
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
            "id":"file_1",
            "object":"vector_store.file",
            "status":"completed"
        })),
    )
}

async fn capture_vector_store_file_delete_request(
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
            "id":"file_1",
            "object":"vector_store.file.deleted",
            "deleted":true
        })),
    )
}

async fn capture_vector_store_file_batch_request(
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
            "id":"vsfb_1",
            "object":"vector_store.file_batch",
            "status":"in_progress"
        })),
    )
}

async fn capture_vector_store_file_batch_retrieve_request(
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
            "id":"vsfb_1",
            "object":"vector_store.file_batch",
            "status":"completed"
        })),
    )
}

async fn capture_vector_store_file_batch_cancel_request(
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
            "id":"vsfb_1",
            "object":"vector_store.file_batch",
            "status":"cancelled"
        })),
    )
}

async fn capture_vector_store_file_batch_files_request(
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
                "id":"file_1",
                "object":"vector_store.file",
                "status":"completed"
            }]
        })),
    )
}

async fn capture_vector_store_search_request(
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
            "object":"list",
            "data":[{
                "file_id":"file_1",
                "filename":"kb.txt",
                "score":0.98,
                "attributes":{},
                "content":[{
                    "type":"text",
                    "text":"reset password"
                }]
            }],
            "has_more":false,
            "next_page":null,
            "search_query":"reset password"
        })),
    )
}

