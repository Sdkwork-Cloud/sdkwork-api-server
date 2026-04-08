use super::*;

#[serial(extension_env)]
#[tokio::test]
async fn stateful_containers_routes_relay_to_openai_compatible_provider() {
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

    let LocalContainersTestContext {
        admin_app,
        admin_token,
        api_key,
        gateway_app,
    } = local_containers_test_context("tenant-1", "project-1").await;

    let _ = admin_app
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

    let create_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/containers")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from("{\"name\":\"ci-container\"}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_response.status(), StatusCode::OK);
    let create_json = read_json(create_response).await;
    assert_eq!(create_json["id"], "container_upstream");
    support::assert_single_usage_record_and_decision_log(
        admin_app.clone(),
        &admin_token,
        "container_upstream",
        "provider-openai-official",
        "ci-container",
    )
    .await;

    let list_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/containers")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(list_response.status(), StatusCode::OK);
    let list_json = read_json(list_response).await;
    assert_eq!(list_json["data"][0]["id"], "container_1");

    let retrieve_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/containers/container_1")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(retrieve_response.status(), StatusCode::OK);
    let retrieve_json = read_json(retrieve_response).await;
    assert_eq!(retrieve_json["id"], "container_1");

    let file_create_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/containers/container_1/files")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from("{\"file_id\":\"file_1\"}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(file_create_response.status(), StatusCode::OK);
    let file_create_json = read_json(file_create_response).await;
    assert_eq!(file_create_json["id"], "container_file_upstream");

    let file_list_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/containers/container_1/files")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(file_list_response.status(), StatusCode::OK);
    let file_list_json = read_json(file_list_response).await;
    assert_eq!(file_list_json["data"][0]["id"], "file_1");

    let file_retrieve_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/containers/container_1/files/file_1")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(file_retrieve_response.status(), StatusCode::OK);
    let file_retrieve_json = read_json(file_retrieve_response).await;
    assert_eq!(file_retrieve_json["id"], "file_1");

    let file_delete_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/v1/containers/container_1/files/file_1")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(file_delete_response.status(), StatusCode::OK);
    let file_delete_json = read_json(file_delete_response).await;
    assert_eq!(file_delete_json["deleted"], true);

    let content_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/containers/container_1/files/file_1/content")
                .header("authorization", format!("Bearer {api_key}"))
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

    let delete_response = gateway_app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/v1/containers/container_1")
                .header("authorization", format!("Bearer {api_key}"))
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
        Some("Bearer sk-upstream-openai")
    );
}

#[serial(extension_env)]
#[tokio::test]
async fn stateful_container_retrieve_route_returns_not_found_without_usage() {
    let ctx = local_containers_test_context(
        "tenant-container-retrieve-missing",
        "project-container-retrieve-missing",
    )
    .await;

    let response = ctx
        .gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/containers/container_missing")
                .header("authorization", format!("Bearer {}", ctx.api_key))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested container was not found.").await;
    support::assert_no_usage_records(ctx.admin_app, &ctx.admin_token).await;
}

#[serial(extension_env)]
#[tokio::test]
async fn stateful_container_delete_route_returns_not_found_without_usage() {
    let ctx =
        local_containers_test_context("tenant-container-delete-missing", "project-container-delete-missing")
            .await;

    let response = ctx
        .gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/v1/containers/container_missing")
                .header("authorization", format!("Bearer {}", ctx.api_key))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested container was not found.").await;
    support::assert_no_usage_records(ctx.admin_app, &ctx.admin_token).await;
}

#[serial(extension_env)]
#[tokio::test]
async fn stateful_container_file_create_route_returns_not_found_without_usage() {
    let ctx = local_containers_test_context(
        "tenant-container-file-create-missing",
        "project-container-file-create-missing",
    )
    .await;

    let response = ctx
        .gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/containers/container_missing/files")
                .header("authorization", format!("Bearer {}", ctx.api_key))
                .header("content-type", "application/json")
                .body(Body::from("{\"file_id\":\"file_1\"}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested container was not found.").await;
    support::assert_no_usage_records(ctx.admin_app, &ctx.admin_token).await;
}

#[serial(extension_env)]
#[tokio::test]
async fn stateful_container_files_list_route_returns_not_found_without_usage() {
    let ctx = local_containers_test_context(
        "tenant-container-files-list-missing",
        "project-container-files-list-missing",
    )
    .await;

    let response = ctx
        .gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/containers/container_missing/files")
                .header("authorization", format!("Bearer {}", ctx.api_key))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested container was not found.").await;
    support::assert_no_usage_records(ctx.admin_app, &ctx.admin_token).await;
}

#[serial(extension_env)]
#[tokio::test]
async fn stateful_container_file_retrieve_route_returns_not_found_without_usage() {
    let ctx = local_containers_test_context(
        "tenant-container-file-retrieve-missing",
        "project-container-file-retrieve-missing",
    )
    .await;

    let response = ctx
        .gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/containers/container_1/files/file_missing")
                .header("authorization", format!("Bearer {}", ctx.api_key))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested container file was not found.").await;
    support::assert_no_usage_records(ctx.admin_app, &ctx.admin_token).await;
}

#[serial(extension_env)]
#[tokio::test]
async fn stateful_container_file_delete_route_returns_not_found_without_usage() {
    let ctx = local_containers_test_context(
        "tenant-container-file-delete-missing",
        "project-container-file-delete-missing",
    )
    .await;

    let response = ctx
        .gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/v1/containers/container_1/files/file_missing")
                .header("authorization", format!("Bearer {}", ctx.api_key))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested container file was not found.").await;
    support::assert_no_usage_records(ctx.admin_app, &ctx.admin_token).await;
}

#[serial(extension_env)]
#[tokio::test]
async fn stateful_container_file_content_route_returns_not_found_without_usage() {
    let ctx = local_containers_test_context(
        "tenant-container-file-content-missing",
        "project-container-file-content-missing",
    )
    .await;

    let response = ctx
        .gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/containers/container_1/files/file_missing/content")
                .header("authorization", format!("Bearer {}", ctx.api_key))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested container file was not found.").await;
    support::assert_no_usage_records(ctx.admin_app, &ctx.admin_token).await;
}
