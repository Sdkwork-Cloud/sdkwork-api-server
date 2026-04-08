use super::*;

#[serial(extension_env)]
#[tokio::test]
async fn stateless_containers_routes_relay_to_openai_compatible_provider() {
    let upstream_state = UpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route(
            "/v1/containers",
            get(upstream_containers_list_handler).post(upstream_containers_handler),
        )
        .route(
            "/v1/containers/container_1",
            get(upstream_container_retrieve_handler).delete(upstream_container_delete_handler),
        )
        .route(
            "/v1/containers/container_1/files",
            get(upstream_container_files_list_handler).post(upstream_container_files_handler),
        )
        .route(
            "/v1/containers/container_1/files/file_1",
            get(upstream_container_file_retrieve_handler)
                .delete(upstream_container_file_delete_handler),
        )
        .route(
            "/v1/containers/container_1/files/file_1/content",
            get(upstream_container_file_content_handler),
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

    let create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/containers")
                .header("content-type", "application/json")
                .body(Body::from("{\"name\":\"ci-container\"}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_response.status(), StatusCode::OK);
    let create_json = read_json(create_response).await;
    assert_eq!(create_json["id"], "container_upstream");

    let list_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/containers")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(list_response.status(), StatusCode::OK);
    let list_json = read_json(list_response).await;
    assert_eq!(list_json["data"][0]["id"], "container_1");

    let retrieve_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/containers/container_1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(retrieve_response.status(), StatusCode::OK);
    let retrieve_json = read_json(retrieve_response).await;
    assert_eq!(retrieve_json["id"], "container_1");

    let file_create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/containers/container_1/files")
                .header("content-type", "application/json")
                .body(Body::from("{\"file_id\":\"file_1\"}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(file_create_response.status(), StatusCode::OK);
    let file_create_json = read_json(file_create_response).await;
    assert_eq!(file_create_json["id"], "container_file_upstream");

    let file_list_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/containers/container_1/files")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(file_list_response.status(), StatusCode::OK);
    let file_list_json = read_json(file_list_response).await;
    assert_eq!(file_list_json["data"][0]["id"], "file_1");

    let file_retrieve_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/containers/container_1/files/file_1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(file_retrieve_response.status(), StatusCode::OK);
    let file_retrieve_json = read_json(file_retrieve_response).await;
    assert_eq!(file_retrieve_json["id"], "file_1");

    let file_delete_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/v1/containers/container_1/files/file_1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(file_delete_response.status(), StatusCode::OK);
    let file_delete_json = read_json(file_delete_response).await;
    assert_eq!(file_delete_json["deleted"], true);

    let content_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/containers/container_1/files/file_1/content")
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
        Some("application/octet-stream")
    );
    assert_eq!(read_bytes(content_response).await, b"CONTAINER-FILE".to_vec());

    let delete_response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/v1/containers/container_1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(delete_response.status(), StatusCode::OK);
    let delete_json = read_json(delete_response).await;
    assert_eq!(delete_json["deleted"], true);
    assert_eq!(
        upstream_state.authorization_header().as_deref(),
        Some("Bearer sk-stateless-openai")
    );
}
