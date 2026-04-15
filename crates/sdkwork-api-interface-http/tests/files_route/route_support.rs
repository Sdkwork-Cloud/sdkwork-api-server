use super::*;

pub(super) async fn read_json(response: axum::response::Response) -> Value {
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

pub(super) async fn assert_file_not_found(response: axum::response::Response, message: &str) {
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let json = read_json(response).await;
    assert_eq!(json["error"]["message"], message);
    assert_eq!(json["error"]["type"], "invalid_request_error");
    assert_eq!(json["error"]["code"], "not_found");
}

pub(super) async fn memory_pool() -> SqlitePool {
    sdkwork_api_storage_sqlite::run_migrations("sqlite::memory:")
        .await
        .unwrap()
}

pub(super) struct LocalFilesTestContext {
    pub(super) admin_app: Router,
    pub(super) admin_token: String,
    pub(super) api_key: String,
    pub(super) gateway_app: Router,
}

pub(super) async fn local_files_test_context(
    tenant_id: &str,
    project_id: &str,
) -> LocalFilesTestContext {
    let pool = memory_pool().await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(&pool, admin_app.clone()).await;
    let api_key = support::issue_gateway_api_key(&pool, tenant_id, project_id).await;
    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool);

    LocalFilesTestContext {
        admin_app,
        admin_token,
        api_key,
        gateway_app,
    }
}

#[derive(Clone, Default)]
pub(super) struct UpstreamCaptureState {
    authorization: Arc<Mutex<Option<String>>>,
    content_type: Arc<Mutex<Option<String>>>,
}

impl UpstreamCaptureState {
    pub(super) fn capture_headers(&self, headers: &axum::http::HeaderMap) {
        *self.authorization.lock().unwrap() = headers
            .get("authorization")
            .and_then(|value| value.to_str().ok())
            .map(ToOwned::to_owned);
        *self.content_type.lock().unwrap() = headers
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .map(ToOwned::to_owned);
    }

    pub(super) fn authorization_header(&self) -> Option<String> {
        self.authorization.lock().unwrap().clone()
    }

    pub(super) fn content_type_header(&self) -> Option<String> {
        self.content_type.lock().unwrap().clone()
    }
}

pub(super) fn build_file_multipart_body(boundary: &str) -> Vec<u8> {
    build_file_multipart_body_with_fields(boundary, "fine-tune", "train.jsonl")
}

pub(super) fn build_file_multipart_body_with_fields(
    boundary: &str,
    purpose: &str,
    filename: &str,
) -> Vec<u8> {
    format!(
        "--{boundary}\r\nContent-Disposition: form-data; name=\"purpose\"\r\n\r\n{purpose}\r\n--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"{filename}\"\r\nContent-Type: application/jsonl\r\n\r\n{{}}\r\n--{boundary}--\r\n"
    )
    .into_bytes()
}

pub(super) async fn assert_invalid_file_request(response: axum::response::Response, message: &str) {
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let json = read_json(response).await;
    assert_eq!(json["error"]["message"], message);
    assert_eq!(json["error"]["type"], "invalid_request_error");
    assert_eq!(json["error"]["code"], "invalid_file");
}
