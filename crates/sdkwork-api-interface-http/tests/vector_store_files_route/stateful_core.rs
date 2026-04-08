use super::*;

#[tokio::test]
async fn stateful_vector_store_files_route_relays_to_openai_compatible_provider() {
    let upstream_state = UpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route(
            "/v1/vector_stores/vs_1/files",
            get(upstream_vector_store_files_list_handler).post(upstream_vector_store_files_handler),
        )
        .route(
            "/v1/vector_stores/vs_1/files/file_1",
            get(upstream_vector_store_file_retrieve_handler)
                .delete(upstream_vector_store_file_delete_handler),
        )
        .with_state(upstream_state.clone());

    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });

    let LocalVectorStoreFilesTestContext {
        admin_app,
        admin_token,
        api_key,
        gateway_app,
    } = local_vector_store_files_test_context("tenant-1", "project-1").await;

    let create_channel = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/channels")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from("{\"id\":\"openai\",\"name\":\"OpenAI\"}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_channel.status(), StatusCode::CREATED);

    let provider = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"id\":\"provider-openai-official\",\"channel_id\":\"openai\",\"adapter_kind\":\"openai\",\"base_url\":\"http://{address}\",\"display_name\":\"OpenAI Official\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(provider.status(), StatusCode::CREATED);

    let credential = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/credentials")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"tenant_id\":\"tenant-1\",\"provider_id\":\"provider-openai-official\",\"key_reference\":\"cred-openai\",\"secret_value\":\"sk-upstream-openai\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(credential.status(), StatusCode::CREATED);

    let response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/vector_stores/vs_1/files")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from("{\"file_id\":\"file_1\"}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["id"], "file_1");
    assert_eq!(
        upstream_state.authorization_header().as_deref(),
        Some("Bearer sk-upstream-openai")
    );

    let list_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/vector_stores/vs_1/files")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(list_response.status(), StatusCode::OK);
    let list_json = read_json(list_response).await;
    assert_eq!(list_json["object"], "list");

    let retrieve_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/vector_stores/vs_1/files/file_1")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(retrieve_response.status(), StatusCode::OK);
    let retrieve_json = read_json(retrieve_response).await;
    assert_eq!(retrieve_json["id"], "file_1");

    let delete_response = gateway_app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/v1/vector_stores/vs_1/files/file_1")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(delete_response.status(), StatusCode::OK);
    let delete_json = read_json(delete_response).await;
    assert_eq!(delete_json["deleted"], true);
}

#[tokio::test]
async fn stateful_vector_store_file_retrieve_route_returns_not_found_without_usage() {
    let LocalVectorStoreFilesTestContext {
        admin_app,
        admin_token,
        api_key,
        gateway_app,
    } = local_vector_store_files_test_context(
        "tenant-vector-file-retrieve-missing",
        "project-vector-file-retrieve-missing",
    )
    .await;

    let response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/vector_stores/vs_1/files/file_missing")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_vector_store_file_not_found(response, "Requested vector store file was not found.")
        .await;
    support::assert_no_usage_records(admin_app, &admin_token).await;
}

#[tokio::test]
async fn stateful_vector_store_file_delete_route_returns_not_found_without_usage() {
    let LocalVectorStoreFilesTestContext {
        admin_app,
        admin_token,
        api_key,
        gateway_app,
    } = local_vector_store_files_test_context(
        "tenant-vector-file-delete-missing",
        "project-vector-file-delete-missing",
    )
    .await;

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/v1/vector_stores/vs_1/files/file_missing")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_vector_store_file_not_found(response, "Requested vector store file was not found.")
        .await;
    support::assert_no_usage_records(admin_app, &admin_token).await;
}
