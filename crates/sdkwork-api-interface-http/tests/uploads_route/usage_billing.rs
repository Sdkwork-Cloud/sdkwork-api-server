use super::*;

#[tokio::test]
async fn stateful_uploads_create_usage_uses_created_upload_id_for_billing() {
    let tenant_id = "tenant-uploads-create";
    let project_id = "project-uploads-create";
    let upstream_state = UpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route("/v1/uploads", post(upstream_uploads_handler))
        .with_state(upstream_state.clone());

    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });

    let LocalUploadTestContext {
        admin_app,
        admin_token,
        api_key,
        gateway_app,
    } = local_upload_test_context(tenant_id, project_id).await;

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

    let provider_upload = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"id\":\"provider-uploads-route\",\"channel_id\":\"openai\",\"adapter_kind\":\"openai\",\"base_url\":\"http://{address}\",\"display_name\":\"Uploads Route Provider\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(provider_upload.status(), StatusCode::CREATED);

    let provider_created = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"id\":\"provider-uploads-created-id\",\"channel_id\":\"openai\",\"adapter_kind\":\"openai\",\"base_url\":\"http://127.0.0.1:1\",\"display_name\":\"Uploads Created Id Provider\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(provider_created.status(), StatusCode::CREATED);

    let upload_credential = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/credentials")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"tenant_id\":\"{tenant_id}\",\"provider_id\":\"provider-uploads-route\",\"key_reference\":\"cred-uploads-route\",\"secret_value\":\"sk-uploads-route\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(upload_credential.status(), StatusCode::CREATED);

    let created_credential = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/credentials")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"tenant_id\":\"{tenant_id}\",\"provider_id\":\"provider-uploads-created-id\",\"key_reference\":\"cred-uploads-created-id\",\"secret_value\":\"sk-uploads-created-id\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(created_credential.status(), StatusCode::CREATED);

    let upload_policy = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/routing/policies")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "policy_id": "route-uploads-by-purpose",
                        "capability": "uploads",
                        "model_pattern": "batch",
                        "enabled": true,
                        "priority": 200,
                        "ordered_provider_ids": ["provider-uploads-route"]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(upload_policy.status(), StatusCode::CREATED);

    let created_policy = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/routing/policies")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "policy_id": "route-uploads-by-created-id",
                        "capability": "uploads",
                        "model_pattern": "upload_upstream",
                        "enabled": true,
                        "priority": 100,
                        "ordered_provider_ids": ["provider-uploads-created-id"]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(created_policy.status(), StatusCode::CREATED);

    let response = gateway_app
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
    assert_eq!(response.status(), StatusCode::OK);
    let response_json = read_json(response).await;
    assert_eq!(response_json["id"], "upload_upstream");
    assert_eq!(
        upstream_state.authorization_header().as_deref(),
        Some("Bearer sk-uploads-route")
    );
    support::assert_single_usage_record_and_decision_log(
        admin_app,
        &admin_token,
        "upload_upstream",
        "provider-uploads-route",
        "batch",
    )
    .await;
}

#[tokio::test]
async fn stateful_upload_part_create_usage_uses_upload_route_key_for_provider_selection() {
    let tenant_id = "tenant-upload-part-create";
    let project_id = "project-upload-part-create";
    let upstream_state = UpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route(
            "/v1/uploads/upload_1/parts",
            post(upstream_upload_parts_handler),
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
    } = local_upload_test_context(tenant_id, project_id).await;

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

    let provider_upload = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"id\":\"provider-upload-create\",\"channel_id\":\"openai\",\"adapter_kind\":\"openai\",\"base_url\":\"http://{address}\",\"display_name\":\"Upload Provider\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(provider_upload.status(), StatusCode::CREATED);

    let provider_part = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"id\":\"provider-upload-part-create\",\"channel_id\":\"openai\",\"adapter_kind\":\"openai\",\"base_url\":\"http://127.0.0.1:1\",\"display_name\":\"Upload Part Provider\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(provider_part.status(), StatusCode::CREATED);

    let upload_credential = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/credentials")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"tenant_id\":\"{tenant_id}\",\"provider_id\":\"provider-upload-create\",\"key_reference\":\"cred-upload-create\",\"secret_value\":\"sk-upload-create\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(upload_credential.status(), StatusCode::CREATED);

    let part_credential = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/credentials")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"tenant_id\":\"{tenant_id}\",\"provider_id\":\"provider-upload-part-create\",\"key_reference\":\"cred-upload-part-create\",\"secret_value\":\"sk-upload-part-create\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(part_credential.status(), StatusCode::CREATED);

    let upload_policy = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/routing/policies")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "policy_id": "route-upload-part-create-by-upload",
                        "capability": "uploads",
                        "model_pattern": "upload_1",
                        "enabled": true,
                        "priority": 200,
                        "ordered_provider_ids": ["provider-upload-create"]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(upload_policy.status(), StatusCode::CREATED);

    let part_policy = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/routing/policies")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "policy_id": "route-upload-part-create-by-part",
                        "capability": "uploads",
                        "model_pattern": "part_upstream",
                        "enabled": true,
                        "priority": 100,
                        "ordered_provider_ids": ["provider-upload-part-create"]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(part_policy.status(), StatusCode::CREATED);

    let response = gateway_app
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
    assert_eq!(response.status(), StatusCode::OK);
    let response_json = read_json(response).await;
    assert_eq!(response_json["id"], "part_upstream");
    assert_eq!(
        upstream_state.authorization_header().as_deref(),
        Some("Bearer sk-upload-create")
    );
    assert!(upstream_state
        .content_type_header()
        .as_deref()
        .unwrap_or_default()
        .starts_with("multipart/form-data"));

    let usage = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/usage/records")
                .header("authorization", format!("Bearer {admin_token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(usage.status(), StatusCode::OK);
    let usage_json = read_json(usage).await;
    assert_eq!(usage_json.as_array().unwrap().len(), 1);
    assert_eq!(usage_json[0]["model"], "part_upstream");
    assert_eq!(usage_json[0]["provider"], "provider-upload-create");

    let logs = admin_app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/routing/decision-logs")
                .header("authorization", format!("Bearer {admin_token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(logs.status(), StatusCode::OK);
    let logs_json = read_json(logs).await;
    assert_eq!(logs_json.as_array().unwrap().len(), 1);
    assert_eq!(logs_json[0]["route_key"], "upload_1");
    assert_eq!(
        logs_json[0]["selected_provider_id"],
        "provider-upload-create"
    );
}
