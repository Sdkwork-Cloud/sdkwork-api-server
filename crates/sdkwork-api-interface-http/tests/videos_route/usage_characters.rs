use super::*;

#[serial(extension_env)]
#[tokio::test]
async fn stateful_video_character_usage_uses_video_route_key_for_provider_selection() {
    let tenant_id = "tenant-video-character-usage";
    let project_id = "project-video-character-usage";
    let upstream_state = UpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route(
            "/v1/videos/video_1/characters/char_1",
            get(upstream_video_character_retrieve_handler),
        )
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

    let provider_video = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"id\":\"provider-video\",\"channel_id\":\"openai\",\"adapter_kind\":\"openai\",\"base_url\":\"http://{address}\",\"display_name\":\"Video Provider\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(provider_video.status(), StatusCode::CREATED);

    let provider_character = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"id\":\"provider-character\",\"channel_id\":\"openai\",\"adapter_kind\":\"openai\",\"base_url\":\"http://127.0.0.1:1\",\"display_name\":\"Character Provider\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(provider_character.status(), StatusCode::CREATED);

    let video_credential = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/credentials")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"tenant_id\":\"{tenant_id}\",\"provider_id\":\"provider-video\",\"key_reference\":\"cred-video\",\"secret_value\":\"sk-video\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(video_credential.status(), StatusCode::CREATED);

    let character_credential = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/credentials")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"tenant_id\":\"{tenant_id}\",\"provider_id\":\"provider-character\",\"key_reference\":\"cred-character\",\"secret_value\":\"sk-character\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(character_credential.status(), StatusCode::CREATED);

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
                        "policy_id": "route-video-characters-by-video",
                        "capability": "videos",
                        "model_pattern": "video_1",
                        "enabled": true,
                        "priority": 200,
                        "ordered_provider_ids": ["provider-video"]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(video_policy.status(), StatusCode::CREATED);

    let character_policy = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/routing/policies")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "policy_id": "route-video-characters-by-character",
                        "capability": "videos",
                        "model_pattern": "char_1",
                        "enabled": true,
                        "priority": 100,
                        "ordered_provider_ids": ["provider-character"]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(character_policy.status(), StatusCode::CREATED);

    let response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/videos/video_1/characters/char_1")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let response_json = read_json(response).await;
    assert_eq!(response_json["id"], "char_1");
    assert_eq!(
        upstream_state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-video")
    );

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
    assert_eq!(usage_json[0]["model"], "char_1");
    assert_eq!(usage_json[0]["provider"], "provider-video");

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
    assert_eq!(logs_json[0]["route_key"], "video_1");
    assert_eq!(logs_json[0]["selected_provider_id"], "provider-video");
}

#[serial(extension_env)]
#[tokio::test]
async fn stateful_video_character_create_usage_uses_video_route_key_for_provider_selection() {
    let tenant_id = "tenant-video-character-create";
    let project_id = "project-video-character-create";
    let upstream_state = UpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route(
            "/v1/videos/characters",
            post(upstream_video_character_create_handler),
        )
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

    let provider_video = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"id\":\"provider-video-create\",\"channel_id\":\"openai\",\"adapter_kind\":\"openai\",\"base_url\":\"http://{address}\",\"display_name\":\"Video Provider\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(provider_video.status(), StatusCode::CREATED);

    let provider_character = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"id\":\"provider-character-create\",\"channel_id\":\"openai\",\"adapter_kind\":\"openai\",\"base_url\":\"http://127.0.0.1:1\",\"display_name\":\"Character Provider\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(provider_character.status(), StatusCode::CREATED);

    let video_credential = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/credentials")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"tenant_id\":\"{tenant_id}\",\"provider_id\":\"provider-video-create\",\"key_reference\":\"cred-video-create\",\"secret_value\":\"sk-video-create\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(video_credential.status(), StatusCode::CREATED);

    let character_credential = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/credentials")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"tenant_id\":\"{tenant_id}\",\"provider_id\":\"provider-character-create\",\"key_reference\":\"cred-character-create\",\"secret_value\":\"sk-character-create\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(character_credential.status(), StatusCode::CREATED);

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
                        "policy_id": "route-video-character-create-by-video",
                        "capability": "videos",
                        "model_pattern": "video_1",
                        "enabled": true,
                        "priority": 200,
                        "ordered_provider_ids": ["provider-video-create"]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(video_policy.status(), StatusCode::CREATED);

    let character_policy = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/routing/policies")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "policy_id": "route-video-character-create-by-character",
                        "capability": "videos",
                        "model_pattern": "char_upstream",
                        "enabled": true,
                        "priority": 100,
                        "ordered_provider_ids": ["provider-character-create"]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(character_policy.status(), StatusCode::CREATED);

    let response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/videos/characters")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from("{\"name\":\"Hero\",\"video_id\":\"video_1\"}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let response_json = read_json(response).await;
    assert_eq!(response_json["id"], "char_upstream");
    assert_eq!(
        upstream_state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-video-create")
    );

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
    assert_eq!(usage_json[0]["model"], "char_upstream");
    assert_eq!(usage_json[0]["provider"], "provider-video-create");

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
    assert_eq!(logs_json[0]["route_key"], "video_1");
    assert_eq!(
        logs_json[0]["selected_provider_id"],
        "provider-video-create"
    );
}
