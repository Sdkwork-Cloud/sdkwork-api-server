use super::route_support::{
    read_json, upstream_vector_store_file_batch_cancel_handler,
    upstream_vector_store_file_batch_files_handler,
    upstream_vector_store_file_batch_retrieve_handler, upstream_vector_store_file_batches_handler,
    UpstreamCaptureState,
};
use super::*;

#[tokio::test]
async fn stateless_vector_store_file_batches_route_relays_to_openai_compatible_provider() {
    let upstream_state = UpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route(
            "/v1/vector_stores/vs_1/file_batches",
            axum::routing::post(upstream_vector_store_file_batches_handler),
        )
        .route(
            "/v1/vector_stores/vs_1/file_batches/vsfb_1",
            get(upstream_vector_store_file_batch_retrieve_handler),
        )
        .route(
            "/v1/vector_stores/vs_1/file_batches/vsfb_1/cancel",
            axum::routing::post(upstream_vector_store_file_batch_cancel_handler),
        )
        .route(
            "/v1/vector_stores/vs_1/file_batches/vsfb_1/files",
            get(upstream_vector_store_file_batch_files_handler),
        )
        .with_state(upstream_state.clone());

    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });

    let app = sdkwork_api_interface_http::gateway_router_with_stateless_config(
        sdkwork_api_interface_http::StatelessGatewayConfig::default().with_upstream(
            sdkwork_api_interface_http::StatelessGatewayUpstream::new(
                "openai",
                format!("http://{address}"),
                "sk-stateless-openai",
            ),
        ),
    );

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/vector_stores/vs_1/file_batches")
                .header("content-type", "application/json")
                .body(Body::from("{\"file_ids\":[\"file_1\"]}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["id"], "vsfb_1");
    assert_eq!(
        upstream_state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-stateless-openai")
    );

    let retrieve_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/vector_stores/vs_1/file_batches/vsfb_1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(retrieve_response.status(), StatusCode::OK);
    let retrieve_json = read_json(retrieve_response).await;
    assert_eq!(retrieve_json["id"], "vsfb_1");

    let cancel_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/vector_stores/vs_1/file_batches/vsfb_1/cancel")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(cancel_response.status(), StatusCode::OK);
    let cancel_json = read_json(cancel_response).await;
    assert_eq!(cancel_json["status"], "cancelled");

    let list_files_response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/vector_stores/vs_1/file_batches/vsfb_1/files")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(list_files_response.status(), StatusCode::OK);
    let list_files_json = read_json(list_files_response).await;
    assert_eq!(list_files_json["data"][0]["id"], "file_1");
}
