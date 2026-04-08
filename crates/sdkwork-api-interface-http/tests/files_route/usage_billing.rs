use super::*;
use serial_test::serial;

#[serial(extension_env)]
#[tokio::test]
async fn stateful_files_create_usage_uses_created_file_id_for_billing() {
    let tenant_id = "tenant-files-create";
    let project_id = "project-files-create";
    let upstream_state = UpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route(
            "/v1/files",
            get(upstream_files_list_handler).post(upstream_files_handler),
        )
        .with_state(upstream_state.clone());

    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });

    let LocalFilesTestContext {
        admin_app,
        admin_token,
        api_key,
        gateway_app,
    } = local_files_test_context(tenant_id, project_id).await;

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

    let provider_route = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"id\":\"provider-files-route\",\"channel_id\":\"openai\",\"adapter_kind\":\"openai\",\"base_url\":\"http://{address}\",\"display_name\":\"Files Route Provider\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(provider_route.status(), StatusCode::CREATED);

    let provider_file = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"id\":\"provider-files-created-id\",\"channel_id\":\"openai\",\"adapter_kind\":\"openai\",\"base_url\":\"http://127.0.0.1:1\",\"display_name\":\"Files Created Id Provider\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(provider_file.status(), StatusCode::CREATED);

    let route_credential = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/credentials")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"tenant_id\":\"{tenant_id}\",\"provider_id\":\"provider-files-route\",\"key_reference\":\"cred-files-route\",\"secret_value\":\"sk-files-route\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(route_credential.status(), StatusCode::CREATED);

    let file_credential = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/credentials")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"tenant_id\":\"{tenant_id}\",\"provider_id\":\"provider-files-created-id\",\"key_reference\":\"cred-files-created-id\",\"secret_value\":\"sk-files-created-id\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(file_credential.status(), StatusCode::CREATED);

    let route_policy = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/routing/policies")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "policy_id": "route-files-by-purpose",
                        "capability": "files",
                        "model_pattern": "fine-tune",
                        "enabled": true,
                        "priority": 200,
                        "ordered_provider_ids": ["provider-files-route"]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(route_policy.status(), StatusCode::CREATED);

    let file_policy = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/routing/policies")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "policy_id": "route-files-by-created-id",
                        "capability": "files",
                        "model_pattern": "file_upstream",
                        "enabled": true,
                        "priority": 100,
                        "ordered_provider_ids": ["provider-files-created-id"]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(file_policy.status(), StatusCode::CREATED);

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/files")
                .header("authorization", format!("Bearer {api_key}"))
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
    let response_json = read_json(response).await;
    assert_eq!(response_json["id"], "file_upstream");
    assert_eq!(
        upstream_state.authorization_header().as_deref(),
        Some("Bearer sk-files-route")
    );
    assert!(upstream_state
        .content_type_header()
        .as_deref()
        .unwrap_or_default()
        .starts_with("multipart/form-data"));
    support::assert_single_usage_record_and_decision_log(
        admin_app,
        &admin_token,
        "file_upstream",
        "provider-files-route",
        "fine-tune",
    )
    .await;
}
