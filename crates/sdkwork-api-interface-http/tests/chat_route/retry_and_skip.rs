use super::*;

#[tokio::test]
async fn stateful_chat_route_skips_recently_failed_primary_provider_on_following_request() {
    let tenant_id = "tenant-chat-circuit-breaker";
    let project_id = "project-chat-circuit-breaker";
    let primary_state = UpstreamCaptureState::default();
    let backup_state = UpstreamCaptureState::default();

    let primary_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let primary_address = primary_listener.local_addr().unwrap();
    let primary_upstream = Router::new()
        .route("/v1/chat/completions", post(upstream_chat_handler_failure))
        .with_state(primary_state.clone());
    tokio::spawn(async move {
        axum::serve(primary_listener, primary_upstream)
            .await
            .unwrap();
    });

    let backup_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let backup_address = backup_listener.local_addr().unwrap();
    let backup_upstream = Router::new()
        .route(
            "/v1/chat/completions",
            post(upstream_chat_handler_backup_with_usage),
        )
        .with_state(backup_state.clone());
    tokio::spawn(async move {
        axum::serve(backup_listener, backup_upstream).await.unwrap();
    });

    let pool = memory_pool().await;
    let api_key = support::issue_gateway_api_key(&pool, tenant_id, project_id).await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
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

    for (provider_id, address, secret_value) in [
        (
            "provider-chat-circuit-breaker-primary",
            primary_address,
            "sk-chat-circuit-breaker-primary",
        ),
        (
            "provider-chat-circuit-breaker-backup",
            backup_address,
            "sk-chat-circuit-breaker-backup",
        ),
    ] {
        let provider = admin_app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/admin/providers")
                    .header("authorization", format!("Bearer {admin_token}"))
                    .header("content-type", "application/json")
                    .body(Body::from(format!(
                        "{{\"id\":\"{provider_id}\",\"channel_id\":\"openai\",\"adapter_kind\":\"openai\",\"base_url\":\"http://{address}\",\"display_name\":\"{provider_id}\"}}"
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
                    .body(Body::from(format!(
                        "{{\"tenant_id\":\"{tenant_id}\",\"provider_id\":\"{provider_id}\",\"key_reference\":\"cred-{provider_id}\",\"secret_value\":\"{secret_value}\"}}"
                    )))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(credential.status(), StatusCode::CREATED);

        let model = admin_app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/admin/models")
                    .header("authorization", format!("Bearer {admin_token}"))
                    .header("content-type", "application/json")
                    .body(Body::from(format!(
                        "{{\"external_name\":\"gpt-4.1\",\"provider_id\":\"{provider_id}\"}}"
                    )))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(model.status(), StatusCode::CREATED);
    }

    let policy = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/routing/policies")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "policy_id": "route-chat-circuit-breaker",
                        "capability": "chat_completion",
                        "model_pattern": "gpt-4.1",
                        "enabled": true,
                        "priority": 300,
                        "ordered_provider_ids": [
                            "provider-chat-circuit-breaker-primary",
                            "provider-chat-circuit-breaker-backup"
                        ]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(policy.status(), StatusCode::CREATED);

    let first_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/chat/completions")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"gpt-4.1\",\"messages\":[{\"role\":\"user\",\"content\":\"first request should fail over\"}]}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(first_response.status(), StatusCode::OK);
    let first_json = read_json(first_response).await;
    assert_eq!(first_json["id"], "chatcmpl_backup");

    let second_response = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/chat/completions")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"gpt-4.1\",\"messages\":[{\"role\":\"user\",\"content\":\"second request should bypass failed primary\"}]}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(second_response.status(), StatusCode::OK);
    let second_json = read_json(second_response).await;
    assert_eq!(second_json["id"], "chatcmpl_backup");

    assert_eq!(primary_state.request_count.load(Ordering::SeqCst), 2);
    assert_eq!(backup_state.request_count.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn stateful_chat_route_retries_retryable_primary_failure_before_failing_over() {
    let tenant_id = "tenant-chat-retryable-primary";
    let project_id = "project-chat-retryable-primary";
    let primary_state = UpstreamCaptureState::default();

    let primary_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let primary_address = primary_listener.local_addr().unwrap();
    let primary_upstream = Router::new()
        .route(
            "/v1/chat/completions",
            post(upstream_chat_handler_retryable_once_then_success),
        )
        .with_state(primary_state.clone());
    tokio::spawn(async move {
        axum::serve(primary_listener, primary_upstream)
            .await
            .unwrap();
    });

    let pool = memory_pool().await;
    let api_key = support::issue_gateway_api_key(&pool, tenant_id, project_id).await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
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

    let provider = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"id\":\"provider-chat-retryable-primary\",\"channel_id\":\"openai\",\"adapter_kind\":\"openai\",\"base_url\":\"http://{primary_address}\",\"display_name\":\"Retryable Primary\"}}"
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
                .body(Body::from(format!(
                    "{{\"tenant_id\":\"{tenant_id}\",\"provider_id\":\"provider-chat-retryable-primary\",\"key_reference\":\"cred-chat-retryable-primary\",\"secret_value\":\"sk-chat-retryable-primary\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(credential.status(), StatusCode::CREATED);

    let model = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/models")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"external_name\":\"gpt-4.1\",\"provider_id\":\"provider-chat-retryable-primary\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(model.status(), StatusCode::CREATED);

    let policy = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/routing/policies")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "policy_id": "route-chat-retryable-primary",
                        "capability": "chat_completion",
                        "model_pattern": "gpt-4.1",
                        "enabled": true,
                        "priority": 300,
                        "ordered_provider_ids": [
                            "provider-chat-retryable-primary"
                        ]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(policy.status(), StatusCode::CREATED);

    let response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/chat/completions")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"gpt-4.1\",\"messages\":[{\"role\":\"user\",\"content\":\"retry transient primary failure\"}]}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["id"], "chatcmpl_retry_recovered");
    assert_eq!(primary_state.request_count.load(Ordering::SeqCst), 2);

    let metrics = gateway_app
        .oneshot(
            Request::builder()
                .uri("/metrics")
                .header("authorization", "Bearer local-dev-metrics-token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(metrics.status(), StatusCode::OK);
    let metrics_body = axum::body::to_bytes(metrics.into_body(), usize::MAX)
        .await
        .unwrap();
    let metrics_text = String::from_utf8(metrics_body.to_vec()).unwrap();
    assert!(metrics_text.contains(
        "sdkwork_upstream_requests_total{service=\"gateway\",capability=\"chat_completion\",provider=\"provider-chat-retryable-primary\",outcome=\"attempt\"} 2"
    ));
    assert!(metrics_text.contains(
        "sdkwork_upstream_requests_total{service=\"gateway\",capability=\"chat_completion\",provider=\"provider-chat-retryable-primary\",outcome=\"success\"} 1"
    ));
    assert!(!metrics_text.contains(
        "sdkwork_upstream_requests_total{service=\"gateway\",capability=\"chat_completion\",provider=\"provider-chat-retryable-primary\",outcome=\"failure\"}"
    ));
    assert!(metrics_text.contains(
        "sdkwork_upstream_retries_total{service=\"gateway\",capability=\"chat_completion\",provider=\"provider-chat-retryable-primary\",outcome=\"scheduled\"} 1"
    ));
    assert!(metrics_text.contains(
        "sdkwork_provider_health_status{service=\"gateway\",provider=\"provider-chat-retryable-primary\",runtime=\"builtin\"} 1"
    ));
}

#[tokio::test]
async fn stateful_chat_route_does_not_retry_when_policy_limits_retry_attempts_to_one() {
    let tenant_id = "tenant-chat-retry-max-attempts-one";
    let project_id = "project-chat-retry-max-attempts-one";
    let primary_state = UpstreamCaptureState::default();

    let primary_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let primary_address = primary_listener.local_addr().unwrap();
    let primary_upstream = Router::new()
        .route(
            "/v1/chat/completions",
            post(upstream_chat_handler_retryable_once_then_success),
        )
        .with_state(primary_state.clone());
    tokio::spawn(async move {
        axum::serve(primary_listener, primary_upstream)
            .await
            .unwrap();
    });

    let pool = memory_pool().await;
    let api_key = support::issue_gateway_api_key(&pool, tenant_id, project_id).await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
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

    let provider = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"id\":\"provider-chat-retry-max-attempts-one\",\"channel_id\":\"openai\",\"adapter_kind\":\"openai\",\"base_url\":\"http://{primary_address}\",\"display_name\":\"Retry Max Attempts One\"}}"
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
                .body(Body::from(format!(
                    "{{\"tenant_id\":\"{tenant_id}\",\"provider_id\":\"provider-chat-retry-max-attempts-one\",\"key_reference\":\"cred-chat-retry-max-attempts-one\",\"secret_value\":\"sk-chat-retry-max-attempts-one\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(credential.status(), StatusCode::CREATED);

    let model = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/models")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"external_name\":\"gpt-4.1\",\"provider_id\":\"provider-chat-retry-max-attempts-one\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(model.status(), StatusCode::CREATED);

    let policy = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/routing/policies")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "policy_id": "route-chat-retry-max-attempts-one",
                        "capability": "chat_completion",
                        "model_pattern": "gpt-4.1",
                        "enabled": true,
                        "priority": 300,
                        "upstream_retry_max_attempts": 1,
                        "ordered_provider_ids": [
                            "provider-chat-retry-max-attempts-one"
                        ]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(policy.status(), StatusCode::CREATED);

    let response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/chat/completions")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"gpt-4.1\",\"messages\":[{\"role\":\"user\",\"content\":\"retry once must stay disabled by policy\"}]}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
    assert_eq!(primary_state.request_count.load(Ordering::SeqCst), 1);

    let metrics = gateway_app
        .oneshot(
            Request::builder()
                .uri("/metrics")
                .header("authorization", "Bearer local-dev-metrics-token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(metrics.status(), StatusCode::OK);
    let metrics_body = axum::body::to_bytes(metrics.into_body(), usize::MAX)
        .await
        .unwrap();
    let metrics_text = String::from_utf8(metrics_body.to_vec()).unwrap();
    assert!(metrics_text.contains(
        "sdkwork_upstream_requests_total{service=\"gateway\",capability=\"chat_completion\",provider=\"provider-chat-retry-max-attempts-one\",outcome=\"attempt\"} 1"
    ));
    assert!(metrics_text.contains(
        "sdkwork_upstream_requests_total{service=\"gateway\",capability=\"chat_completion\",provider=\"provider-chat-retry-max-attempts-one\",outcome=\"failure\"} 1"
    ));
    assert!(!metrics_text.contains(
        "sdkwork_upstream_retries_total{service=\"gateway\",capability=\"chat_completion\",provider=\"provider-chat-retry-max-attempts-one\",outcome=\"scheduled\"}"
    ));
}

#[tokio::test]
async fn stateful_chat_route_does_not_retry_non_retryable_primary_failure() {
    let tenant_id = "tenant-chat-non-retryable-primary";
    let project_id = "project-chat-non-retryable-primary";
    let primary_state = UpstreamCaptureState::default();

    let primary_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let primary_address = primary_listener.local_addr().unwrap();
    let primary_upstream = Router::new()
        .route(
            "/v1/chat/completions",
            post(upstream_chat_handler_non_retryable_once_then_success),
        )
        .with_state(primary_state.clone());
    tokio::spawn(async move {
        axum::serve(primary_listener, primary_upstream)
            .await
            .unwrap();
    });

    let pool = memory_pool().await;
    let api_key = support::issue_gateway_api_key(&pool, tenant_id, project_id).await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
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

    let provider = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"id\":\"provider-chat-non-retryable-primary\",\"channel_id\":\"openai\",\"adapter_kind\":\"openai\",\"base_url\":\"http://{primary_address}\",\"display_name\":\"Non Retryable Primary\"}}"
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
                .body(Body::from(format!(
                    "{{\"tenant_id\":\"{tenant_id}\",\"provider_id\":\"provider-chat-non-retryable-primary\",\"key_reference\":\"cred-chat-non-retryable-primary\",\"secret_value\":\"sk-chat-non-retryable-primary\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(credential.status(), StatusCode::CREATED);

    let model = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/models")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"external_name\":\"gpt-4.1\",\"provider_id\":\"provider-chat-non-retryable-primary\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(model.status(), StatusCode::CREATED);

    let policy = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/routing/policies")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "policy_id": "route-chat-non-retryable-primary",
                        "capability": "chat_completion",
                        "model_pattern": "gpt-4.1",
                        "enabled": true,
                        "priority": 300,
                        "ordered_provider_ids": [
                            "provider-chat-non-retryable-primary"
                        ]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(policy.status(), StatusCode::CREATED);

    let response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/chat/completions")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"gpt-4.1\",\"messages\":[{\"role\":\"user\",\"content\":\"do not retry invalid request\"}]}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
    assert_eq!(primary_state.request_count.load(Ordering::SeqCst), 1);

    let metrics = gateway_app
        .oneshot(
            Request::builder()
                .uri("/metrics")
                .header("authorization", "Bearer local-dev-metrics-token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(metrics.status(), StatusCode::OK);
    let metrics_body = axum::body::to_bytes(metrics.into_body(), usize::MAX)
        .await
        .unwrap();
    let metrics_text = String::from_utf8(metrics_body.to_vec()).unwrap();
    assert!(metrics_text.contains(
        "sdkwork_upstream_requests_total{service=\"gateway\",capability=\"chat_completion\",provider=\"provider-chat-non-retryable-primary\",outcome=\"attempt\"} 1"
    ));
    assert!(metrics_text.contains(
        "sdkwork_upstream_requests_total{service=\"gateway\",capability=\"chat_completion\",provider=\"provider-chat-non-retryable-primary\",outcome=\"failure\"} 1"
    ));
    assert!(!metrics_text.contains(
        "sdkwork_upstream_retries_total{service=\"gateway\",capability=\"chat_completion\",provider=\"provider-chat-non-retryable-primary\",outcome=\"scheduled\"}"
    ));
}
