use super::*;
use serial_test::serial;

#[serial(extension_env)]
#[tokio::test]
async fn stateless_files_routes_relay_to_openai_compatible_provider() {
    let upstream_state = UpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route(
            "/v1/files",
            get(upstream_files_list_handler).post(upstream_files_handler),
        )
        .route(
            "/v1/files/file_1",
            get(upstream_file_retrieve_handler).delete(upstream_file_delete_handler),
        )
        .route(
            "/v1/files/file_1/content",
            get(upstream_file_content_handler),
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
                .uri("/v1/files")
                .header(
                    "content-type",
                    "multipart/form-data; boundary=----sdkwork-boundary",
                )
                .body(Body::from(build_file_multipart_body(
                    "----sdkwork-boundary",
                )))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["id"], "file_upstream");
    assert_eq!(
        upstream_state.authorization_header().as_deref(),
        Some("Bearer sk-stateless-openai")
    );
    assert!(upstream_state
        .content_type_header()
        .as_deref()
        .unwrap_or_default()
        .starts_with("multipart/form-data"));

    let list_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/files")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(list_response.status(), StatusCode::OK);
    let list_json = read_json(list_response).await;
    assert_eq!(list_json["object"], "list");

    let retrieve_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/files/file_1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(retrieve_response.status(), StatusCode::OK);
    let retrieve_json = read_json(retrieve_response).await;
    assert_eq!(retrieve_json["id"], "file_1");

    let delete_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/v1/files/file_1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(delete_response.status(), StatusCode::OK);
    let delete_json = read_json(delete_response).await;
    assert_eq!(delete_json["deleted"], true);

    let content_response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/files/file_1/content")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(content_response.status(), StatusCode::OK);
    assert_eq!(
        content_response
            .headers()
            .get("content-type")
            .and_then(|value| value.to_str().ok()),
        Some("application/jsonl")
    );
    let content_bytes = axum::body::to_bytes(content_response.into_body(), usize::MAX)
        .await
        .unwrap();
    assert_eq!(&content_bytes[..], b"{}");
}

#[serial(extension_env)]
#[tokio::test]
async fn stateless_file_content_route_returns_openai_error_envelope_on_upstream_failure() {
    let app = sdkwork_api_interface_http::gateway_router_with_stateless_config(
        sdkwork_api_interface_http::StatelessGatewayConfig::default().with_upstream(
            sdkwork_api_interface_http::StatelessGatewayUpstream::new(
                "openai",
                "http://127.0.0.1:1",
                "sk-stateless-openai",
            ),
        ),
    );

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/files/file_1/content")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
    let json = read_json(response).await;
    assert_eq!(
        json["error"]["message"],
        "failed to relay upstream file content"
    );
    assert_eq!(json["error"]["type"], "server_error");
    assert_eq!(json["error"]["code"], "bad_gateway");
}
