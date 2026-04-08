use super::*;

#[serial(extension_env)]
#[tokio::test]
async fn stateful_video_create_usage_uses_created_video_id_for_billing() {
    let tenant_id = "tenant-video-create";
    let project_id = "project-video-create";
    let upstream_state = UpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route("/v1/videos", post(upstream_videos_handler))
        .with_state(upstream_state.clone());

    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });

    let pool = memory_pool().await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
    let api_key = support::issue_gateway_api_key(&pool, tenant_id, project_id).await;
    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool);

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

    let provider_model = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"id\":\"provider-video-route\",\"channel_id\":\"openai\",\"adapter_kind\":\"openai\",\"base_url\":\"http://{address}\",\"display_name\":\"Video Route Provider\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(provider_model.status(), StatusCode::CREATED);

    let provider_video = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"id\":\"provider-video-child\",\"channel_id\":\"openai\",\"adapter_kind\":\"openai\",\"base_url\":\"http://127.0.0.1:1\",\"display_name\":\"Video Child Provider\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(provider_video.status(), StatusCode::CREATED);

    let model_credential = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/credentials")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"tenant_id\":\"{tenant_id}\",\"provider_id\":\"provider-video-route\",\"key_reference\":\"cred-video-route\",\"secret_value\":\"sk-video-route\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(model_credential.status(), StatusCode::CREATED);

    let video_credential = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/credentials")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"tenant_id\":\"{tenant_id}\",\"provider_id\":\"provider-video-child\",\"key_reference\":\"cred-video-child\",\"secret_value\":\"sk-video-child\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(video_credential.status(), StatusCode::CREATED);

    let model_policy = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/routing/policies")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "policy_id": "route-videos-by-model",
                        "capability": "videos",
                        "model_pattern": "sora-1",
                        "enabled": true,
                        "priority": 200,
                        "ordered_provider_ids": ["provider-video-route"]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(model_policy.status(), StatusCode::CREATED);

    let video_policy = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/routing/policies")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "policy_id": "route-videos-by-created-id",
                        "capability": "videos",
                        "model_pattern": "video_upstream",
                        "enabled": true,
                        "priority": 100,
                        "ordered_provider_ids": ["provider-video-child"]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(video_policy.status(), StatusCode::CREATED);

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/videos")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"sora-1\",\"prompt\":\"A short cinematic flyover\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let response_json = read_json(response).await;
    assert_eq!(response_json["data"][0]["id"], "video_upstream");
    assert_eq!(
        upstream_state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-video-route")
    );
    support::assert_single_usage_record_and_decision_log(
        admin_app,
        &admin_token,
        "video_upstream",
        "provider-video-route",
        "sora-1",
    )
    .await;
}

#[serial(extension_env)]
#[tokio::test]
async fn stateful_video_remix_usage_uses_created_video_id_for_billing() {
    let tenant_id = "tenant-video-remix";
    let project_id = "project-video-remix";
    let upstream_state = UpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route("/v1/videos/video_1/remix", post(upstream_video_remix_handler))
        .with_state(upstream_state.clone());

    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });

    let pool = memory_pool().await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
    let api_key = support::issue_gateway_api_key(&pool, tenant_id, project_id).await;
    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool);

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
                    "{{\"id\":\"provider-video-remix-route\",\"channel_id\":\"openai\",\"adapter_kind\":\"openai\",\"base_url\":\"http://{address}\",\"display_name\":\"Video Remix Route Provider\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(provider_route.status(), StatusCode::CREATED);

    let provider_child = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"id\":\"provider-video-remix-child\",\"channel_id\":\"openai\",\"adapter_kind\":\"openai\",\"base_url\":\"http://127.0.0.1:1\",\"display_name\":\"Video Remix Child Provider\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(provider_child.status(), StatusCode::CREATED);

    let route_credential = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/credentials")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"tenant_id\":\"{tenant_id}\",\"provider_id\":\"provider-video-remix-route\",\"key_reference\":\"cred-video-remix-route\",\"secret_value\":\"sk-video-remix-route\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(route_credential.status(), StatusCode::CREATED);

    let child_credential = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/credentials")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"tenant_id\":\"{tenant_id}\",\"provider_id\":\"provider-video-remix-child\",\"key_reference\":\"cred-video-remix-child\",\"secret_value\":\"sk-video-remix-child\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(child_credential.status(), StatusCode::CREATED);

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
                        "policy_id": "route-video-remix-by-video",
                        "capability": "videos",
                        "model_pattern": "video_1",
                        "enabled": true,
                        "priority": 200,
                        "ordered_provider_ids": ["provider-video-remix-route"]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(route_policy.status(), StatusCode::CREATED);

    let child_policy = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/routing/policies")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "policy_id": "route-video-remix-by-created-video",
                        "capability": "videos",
                        "model_pattern": "video_1_remix",
                        "enabled": true,
                        "priority": 100,
                        "ordered_provider_ids": ["provider-video-remix-child"]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(child_policy.status(), StatusCode::CREATED);

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/videos/video_1/remix")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from("{\"prompt\":\"Make it sunset\"}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let response_json = read_json(response).await;
    assert_eq!(response_json["data"][0]["id"], "video_1_remix");
    assert_eq!(
        upstream_state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-video-remix-route")
    );
    support::assert_single_usage_record_and_decision_log(
        admin_app,
        &admin_token,
        "video_1_remix",
        "provider-video-remix-route",
        "video_1",
    )
    .await;
}

#[serial(extension_env)]
#[tokio::test]
async fn stateful_video_extend_usage_uses_created_video_id_for_billing() {
    let tenant_id = "tenant-video-extend";
    let project_id = "project-video-extend";
    let upstream_state = UpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route("/v1/videos/video_1/extend", post(upstream_video_extend_handler))
        .with_state(upstream_state.clone());

    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });

    let pool = memory_pool().await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
    let api_key = support::issue_gateway_api_key(&pool, tenant_id, project_id).await;
    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool);

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
                    "{{\"id\":\"provider-video-extend-route\",\"channel_id\":\"openai\",\"adapter_kind\":\"openai\",\"base_url\":\"http://{address}\",\"display_name\":\"Video Extend Route Provider\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(provider_route.status(), StatusCode::CREATED);

    let provider_child = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"id\":\"provider-video-extend-child\",\"channel_id\":\"openai\",\"adapter_kind\":\"openai\",\"base_url\":\"http://127.0.0.1:1\",\"display_name\":\"Video Extend Child Provider\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(provider_child.status(), StatusCode::CREATED);

    let route_credential = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/credentials")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"tenant_id\":\"{tenant_id}\",\"provider_id\":\"provider-video-extend-route\",\"key_reference\":\"cred-video-extend-route\",\"secret_value\":\"sk-video-extend-route\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(route_credential.status(), StatusCode::CREATED);

    let child_credential = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/credentials")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"tenant_id\":\"{tenant_id}\",\"provider_id\":\"provider-video-extend-child\",\"key_reference\":\"cred-video-extend-child\",\"secret_value\":\"sk-video-extend-child\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(child_credential.status(), StatusCode::CREATED);

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
                        "policy_id": "route-video-extend-by-video",
                        "capability": "videos",
                        "model_pattern": "video_1",
                        "enabled": true,
                        "priority": 200,
                        "ordered_provider_ids": ["provider-video-extend-route"]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(route_policy.status(), StatusCode::CREATED);

    let child_policy = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/routing/policies")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "policy_id": "route-video-extend-by-created-video",
                        "capability": "videos",
                        "model_pattern": "video_1_extended",
                        "enabled": true,
                        "priority": 100,
                        "ordered_provider_ids": ["provider-video-extend-child"]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(child_policy.status(), StatusCode::CREATED);

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/videos/video_1/extend")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from("{\"prompt\":\"Extend the ending\"}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let response_json = read_json(response).await;
    assert_eq!(response_json["data"][0]["id"], "video_1_extended");
    assert_eq!(
        upstream_state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-video-extend-route")
    );
    support::assert_single_usage_record_and_decision_log(
        admin_app,
        &admin_token,
        "video_1_extended",
        "provider-video-extend-route",
        "video_1",
    )
    .await;
}
