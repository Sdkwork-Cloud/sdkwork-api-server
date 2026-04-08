use super::route_support::{memory_pool, read_json};
use super::*;

#[tokio::test]
async fn stateful_vector_store_file_batch_retrieve_route_returns_not_found_without_usage() {
    let pool = memory_pool().await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
    let api_key = support::issue_gateway_api_key(
        &pool,
        "tenant-vector-batch-retrieve-missing",
        "project-vector-batch-retrieve-missing",
    )
    .await;
    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool);

    let response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/vector_stores/vs_1/file_batches/vsfb_missing")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let json = read_json(response).await;
    assert_eq!(
        json["error"]["message"],
        "Requested vector store file batch was not found."
    );
    assert_eq!(json["error"]["type"], "invalid_request_error");
    assert_eq!(json["error"]["code"], "not_found");

    support::assert_no_usage_records(admin_app, &admin_token).await;
}

#[tokio::test]
async fn stateful_vector_store_file_batch_cancel_route_returns_not_found_without_usage() {
    let pool = memory_pool().await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
    let api_key = support::issue_gateway_api_key(
        &pool,
        "tenant-vector-batch-cancel-missing",
        "project-vector-batch-cancel-missing",
    )
    .await;
    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool);

    let response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/vector_stores/vs_1/file_batches/vsfb_missing/cancel")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let json = read_json(response).await;
    assert_eq!(
        json["error"]["message"],
        "Requested vector store file batch was not found."
    );
    assert_eq!(json["error"]["type"], "invalid_request_error");
    assert_eq!(json["error"]["code"], "not_found");

    support::assert_no_usage_records(admin_app, &admin_token).await;
}

#[tokio::test]
async fn stateful_vector_store_file_batch_files_route_returns_not_found_without_usage() {
    let pool = memory_pool().await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
    let api_key = support::issue_gateway_api_key(
        &pool,
        "tenant-vector-batch-files-missing",
        "project-vector-batch-files-missing",
    )
    .await;
    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool);

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/vector_stores/vs_1/file_batches/vsfb_missing/files")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let json = read_json(response).await;
    assert_eq!(
        json["error"]["message"],
        "Requested vector store file batch was not found."
    );
    assert_eq!(json["error"]["type"], "invalid_request_error");
    assert_eq!(json["error"]["code"], "not_found");

    support::assert_no_usage_records(admin_app, &admin_token).await;
}
