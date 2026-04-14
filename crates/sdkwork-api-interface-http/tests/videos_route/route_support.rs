use super::*;

pub(super) async fn read_json(response: axum::response::Response) -> Value {
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

pub(super) async fn assert_video_not_found(response: axum::response::Response, message: &str) {
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let json = read_json(response).await;
    assert_eq!(json["error"]["message"], message);
    assert_eq!(json["error"]["type"], "invalid_request_error");
    assert_eq!(json["error"]["code"], "not_found");
}

pub(super) async fn read_bytes(response: axum::response::Response) -> Vec<u8> {
    axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap()
        .to_vec()
}

pub(super) async fn memory_pool() -> SqlitePool {
    sdkwork_api_storage_sqlite::run_migrations("sqlite::memory:")
        .await
        .unwrap()
}

pub(super) struct LocalVideoTestContext {
    pub(super) admin_app: Router,
    pub(super) admin_token: String,
    pub(super) api_key: String,
    pub(super) gateway_app: Router,
}

pub(super) async fn local_video_test_context(
    tenant_id: &str,
    project_id: &str,
) -> LocalVideoTestContext {
    let pool = memory_pool().await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
    let api_key = support::issue_gateway_api_key(&pool, tenant_id, project_id).await;
    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool);

    LocalVideoTestContext {
        admin_app,
        admin_token,
        api_key,
        gateway_app,
    }
}

#[derive(Clone, Default)]
pub(super) struct UpstreamCaptureState {
    pub(super) authorization: Arc<Mutex<Option<String>>>,
}
