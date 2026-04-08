use super::support::*;

#[tokio::test]
async fn adapter_posts_files_multipart_to_openai_compatible_upstream() {
    let state = CaptureState::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let app = Router::new()
        .route("/v1/files", post(capture_file_request))
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter =
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));
    let request = sdkwork_api_contract_openai::files::CreateFileRequest::new(
        "fine-tune",
        "train.jsonl",
        b"{\"messages\":[]}".to_vec(),
    )
    .with_content_type("application/jsonl");

    let response = adapter.files("sk-upstream-openai", &request).await.unwrap();

    assert_eq!(response["id"], "file_upstream");
    assert_eq!(
        state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-upstream-openai")
    );
    assert!(state
        .content_type
        .lock()
        .unwrap()
        .as_deref()
        .unwrap_or_default()
        .starts_with("multipart/form-data"));
    let raw_body = state.raw_body.lock().unwrap().clone().unwrap();
    let raw_body = String::from_utf8_lossy(&raw_body);
    assert!(raw_body.contains("name=\"purpose\""));
    assert!(raw_body.contains("fine-tune"));
    assert!(raw_body.contains("filename=\"train.jsonl\""));
}

#[tokio::test]
async fn adapter_lists_files_from_openai_compatible_upstream() {
    let state = CaptureState::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let app = Router::new()
        .route("/v1/files", get(capture_files_list_request))
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter =
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));
    let response = adapter.list_files("sk-upstream-openai").await.unwrap();

    assert_eq!(response["object"], "list");
    assert_eq!(response["data"][0]["id"], "file_1");
    assert_eq!(
        state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-upstream-openai")
    );
}

#[tokio::test]
async fn adapter_retrieves_file_from_openai_compatible_upstream() {
    let state = CaptureState::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let app = Router::new()
        .route("/v1/files/file_1", get(capture_file_retrieve_request))
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter =
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));
    let response = adapter
        .retrieve_file("sk-upstream-openai", "file_1")
        .await
        .unwrap();

    assert_eq!(response["id"], "file_1");
    assert_eq!(
        state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-upstream-openai")
    );
}

#[tokio::test]
async fn adapter_deletes_file_from_openai_compatible_upstream() {
    let state = CaptureState::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let app = Router::new()
        .route(
            "/v1/files/file_1",
            axum::routing::delete(capture_file_delete_request),
        )
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter =
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));
    let response = adapter
        .delete_file("sk-upstream-openai", "file_1")
        .await
        .unwrap();

    assert_eq!(response["deleted"], true);
    assert_eq!(
        state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-upstream-openai")
    );
    assert_eq!(state.raw_body.lock().unwrap().as_deref(), Some(&[][..]));
}

#[tokio::test]
async fn adapter_streams_file_content_from_openai_compatible_upstream() {
    let state = CaptureState::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let app = Router::new()
        .route(
            "/v1/files/file_1/content",
            get(capture_file_content_request),
        )
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter =
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));
    let response = adapter
        .file_content("sk-upstream-openai", "file_1")
        .await
        .unwrap();
    let content_type = response.content_type().to_owned();
    let mut stream = response.into_body_stream();
    let mut bytes = Vec::new();
    while let Some(chunk) = stream.next().await {
        bytes.extend_from_slice(&chunk.unwrap());
    }

    assert_eq!(content_type, "application/jsonl");
    assert_eq!(bytes.as_slice(), b"{}");
    assert_eq!(
        state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-upstream-openai")
    );
}

#[tokio::test]
async fn adapter_posts_uploads_create_to_openai_compatible_upstream() {
    let state = CaptureState::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let app = Router::new()
        .route("/v1/uploads", post(capture_upload_create_request))
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter =
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));
    let request = sdkwork_api_contract_openai::uploads::CreateUploadRequest::new(
        "batch",
        "input.jsonl",
        "application/jsonl",
        1024,
    );

    let response = adapter
        .uploads("sk-upstream-openai", &request)
        .await
        .unwrap();

    assert_eq!(response["id"], "upload_upstream");
    assert_eq!(
        state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-upstream-openai")
    );
    assert_eq!(
        state.body.lock().unwrap().as_ref().unwrap()["mime_type"],
        "application/jsonl"
    );
}

#[tokio::test]
async fn adapter_posts_upload_parts_multipart_to_openai_compatible_upstream() {
    let state = CaptureState::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let app = Router::new()
        .route(
            "/v1/uploads/upload_1/parts",
            post(capture_upload_part_request),
        )
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter =
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));
    let request = sdkwork_api_contract_openai::uploads::AddUploadPartRequest::new(
        "upload_1",
        b"part-data".to_vec(),
    )
    .with_filename("part-1.bin")
    .with_content_type("application/octet-stream");

    let response = adapter
        .upload_parts("sk-upstream-openai", &request)
        .await
        .unwrap();

    assert_eq!(response["id"], "part_upstream");
    assert_eq!(
        state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-upstream-openai")
    );
    assert!(state
        .content_type
        .lock()
        .unwrap()
        .as_deref()
        .unwrap_or_default()
        .starts_with("multipart/form-data"));
    let raw_body = state.raw_body.lock().unwrap().clone().unwrap();
    let raw_body = String::from_utf8_lossy(&raw_body);
    assert!(raw_body.contains("name=\"data\""));
    assert!(raw_body.contains("filename=\"part-1.bin\""));
    assert!(raw_body.contains("part-data"));
}

#[tokio::test]
async fn adapter_posts_upload_complete_to_openai_compatible_upstream() {
    let state = CaptureState::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let app = Router::new()
        .route(
            "/v1/uploads/upload_1/complete",
            post(capture_upload_complete_request),
        )
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter =
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));
    let request = sdkwork_api_contract_openai::uploads::CompleteUploadRequest::new(
        "upload_1",
        vec!["part_1", "part_2"],
    );

    let response = adapter
        .complete_upload("sk-upstream-openai", &request)
        .await
        .unwrap();

    assert_eq!(response["id"], "upload_upstream");
    assert_eq!(
        state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-upstream-openai")
    );
    assert_eq!(
        state.body.lock().unwrap().as_ref().unwrap()["part_ids"][1],
        "part_2"
    );
}

#[tokio::test]
async fn adapter_posts_upload_cancel_to_openai_compatible_upstream() {
    let state = CaptureState::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let app = Router::new()
        .route(
            "/v1/uploads/upload_1/cancel",
            post(capture_upload_cancel_request),
        )
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter =
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));

    let response = adapter
        .cancel_upload("sk-upstream-openai", "upload_1")
        .await
        .unwrap();

    assert_eq!(response["id"], "upload_upstream");
    assert_eq!(response["status"], "cancelled");
    assert_eq!(
        state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-upstream-openai")
    );
    assert_eq!(state.raw_body.lock().unwrap().as_deref(), Some(&[][..]));
}

async fn capture_file_request(
    State(state): State<CaptureState>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> (StatusCode, Json<Value>) {
    *state.authorization.lock().unwrap() = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);
    *state.content_type.lock().unwrap() = headers
        .get("content-type")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);
    *state.raw_body.lock().unwrap() = Some(body.to_vec());

    (
        StatusCode::OK,
        Json(json!({
            "id":"file_upstream",
            "object":"file",
            "purpose":"fine-tune",
            "filename":"train.jsonl",
            "bytes":13,
            "status":"processed"
        })),
    )
}

async fn capture_files_list_request(
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
                "object":"file",
                "purpose":"fine-tune",
                "filename":"train.jsonl",
                "bytes":13,
                "status":"processed"
            }]
        })),
    )
}

async fn capture_file_retrieve_request(
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
            "object":"file",
            "purpose":"fine-tune",
            "filename":"train.jsonl",
            "bytes":13,
            "status":"processed"
        })),
    )
}

async fn capture_file_delete_request(
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
            "id":"file_1",
            "object":"file",
            "deleted":true
        })),
    )
}

async fn capture_file_content_request(
    State(state): State<CaptureState>,
    headers: HeaderMap,
) -> (
    [(axum::http::header::HeaderName, &'static str); 1],
    &'static [u8],
) {
    *state.authorization.lock().unwrap() = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);

    (
        [(axum::http::header::CONTENT_TYPE, "application/jsonl")],
        b"{}",
    )
}

async fn capture_upload_create_request(
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
            "id":"upload_upstream",
            "object":"upload",
            "purpose":"batch",
            "filename":"input.jsonl",
            "mime_type":"application/jsonl",
            "bytes":1024,
            "part_ids":[],
            "status":"pending"
        })),
    )
}

async fn capture_upload_part_request(
    State(state): State<CaptureState>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> (StatusCode, Json<Value>) {
    *state.authorization.lock().unwrap() = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);
    *state.content_type.lock().unwrap() = headers
        .get("content-type")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);
    *state.raw_body.lock().unwrap() = Some(body.to_vec());

    (
        StatusCode::OK,
        Json(json!({
            "id":"part_upstream",
            "object":"upload.part",
            "upload_id":"upload_1",
            "status":"completed"
        })),
    )
}

async fn capture_upload_complete_request(
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
            "id":"upload_upstream",
            "object":"upload",
            "purpose":"batch",
            "filename":"input.jsonl",
            "mime_type":"application/jsonl",
            "bytes":1024,
            "part_ids":["part_1","part_2"],
            "status":"completed"
        })),
    )
}

async fn capture_upload_cancel_request(
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
            "id":"upload_upstream",
            "object":"upload",
            "purpose":"batch",
            "filename":"input.jsonl",
            "mime_type":"application/jsonl",
            "bytes":1024,
            "part_ids":["part_1","part_2"],
            "status":"cancelled"
        })),
    )
}
