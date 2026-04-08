use super::*;

#[tokio::test]
async fn stateful_upload_parts_route_returns_not_found_without_usage() {
    let ctx = local_upload_test_context(
        "tenant-upload-parts-missing",
        "project-upload-parts-missing",
    )
    .await;

    let response = ctx
        .gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/uploads/upload_missing/parts")
                .header(
                    "content-type",
                    "multipart/form-data; boundary=----sdkwork-upload-part",
                )
                .header("authorization", format!("Bearer {}", ctx.api_key))
                .body(Body::from(build_upload_part_multipart_body(
                    "----sdkwork-upload-part",
                )))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested upload session was not found.").await;
    support::assert_no_usage_records(ctx.admin_app, &ctx.admin_token).await;
}

#[tokio::test]
async fn stateful_upload_complete_route_returns_not_found_without_usage() {
    let ctx = local_upload_test_context(
        "tenant-upload-complete-missing",
        "project-upload-complete-missing",
    )
    .await;

    let response = ctx
        .gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/uploads/upload_missing/complete")
                .header("authorization", format!("Bearer {}", ctx.api_key))
                .header("content-type", "application/json")
                .body(Body::from("{\"part_ids\":[\"part_1\",\"part_2\"]}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested upload session was not found.").await;
    support::assert_no_usage_records(ctx.admin_app, &ctx.admin_token).await;
}

#[tokio::test]
async fn stateful_upload_cancel_route_returns_not_found_without_usage() {
    let ctx = local_upload_test_context(
        "tenant-upload-cancel-missing",
        "project-upload-cancel-missing",
    )
    .await;

    let response = ctx
        .gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/uploads/upload_missing/cancel")
                .header("authorization", format!("Bearer {}", ctx.api_key))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested upload session was not found.").await;
    support::assert_no_usage_records(ctx.admin_app, &ctx.admin_token).await;
}

#[tokio::test]
async fn stateful_upload_routes_relay_to_openai_compatible_provider() {
    let upstream_state = UpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route("/v1/uploads", post(upstream_uploads_handler))
        .route(
            "/v1/uploads/upload_1/parts",
            post(upstream_upload_parts_handler),
        )
        .route(
            "/v1/uploads/upload_1/complete",
            post(upstream_upload_complete_handler),
        )
        .route(
            "/v1/uploads/upload_1/cancel",
            post(upstream_upload_cancel_handler),
        )
        .with_state(upstream_state.clone());

    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });

    let LocalUploadTestContext {
        admin_app,
        admin_token,
        api_key,
        gateway_app,
    } = local_upload_test_context("tenant-1", "project-1").await;

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

    let upload_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/uploads")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"purpose\":\"batch\",\"filename\":\"input.jsonl\",\"mime_type\":\"application/jsonl\",\"bytes\":1024}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(upload_response.status(), StatusCode::OK);
    let upload_json = read_json(upload_response).await;
    assert_eq!(upload_json["id"], "upload_upstream");
    support::assert_single_usage_record_and_decision_log(
        admin_app.clone(),
        &admin_token,
        "upload_upstream",
        "provider-openai-official",
        "batch",
    )
    .await;

    let part_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/uploads/upload_1/parts")
                .header("authorization", format!("Bearer {api_key}"))
                .header(
                    "content-type",
                    "multipart/form-data; boundary=----sdkwork-upload-part",
                )
                .body(Body::from(build_upload_part_multipart_body(
                    "----sdkwork-upload-part",
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(part_response.status(), StatusCode::OK);
    let part_json = read_json(part_response).await;
    assert_eq!(part_json["id"], "part_upstream");

    let complete_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/uploads/upload_1/complete")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from("{\"part_ids\":[\"part_1\",\"part_2\"]}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(complete_response.status(), StatusCode::OK);
    let complete_json = read_json(complete_response).await;
    assert_eq!(complete_json["part_ids"][1], "part_2");
    assert_eq!(
        upstream_state.authorization_header().as_deref(),
        Some("Bearer sk-upstream-openai")
    );
    assert!(upstream_state
        .content_type_header()
        .as_deref()
        .unwrap_or_default()
        .starts_with("multipart/form-data"));

    let cancel_response = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/uploads/upload_1/cancel")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(cancel_response.status(), StatusCode::OK);
    let cancel_json = read_json(cancel_response).await;
    assert_eq!(cancel_json["status"], "cancelled");
}
