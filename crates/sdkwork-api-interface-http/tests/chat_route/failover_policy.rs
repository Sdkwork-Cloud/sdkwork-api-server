use super::*;

async fn stateful_chat_route_fails_over_to_backup_provider_and_records_actual_provider() {
    let tenant_id = "tenant-chat-failover-json";
    let project_id = "project-chat-failover-json";
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

    let primary_provider = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"id\":\"provider-chat-failover-primary\",\"channel_id\":\"openai\",\"adapter_kind\":\"openai\",\"base_url\":\"http://{primary_address}\",\"display_name\":\"Chat Failover Primary\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(primary_provider.status(), StatusCode::CREATED);

    let backup_provider = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"id\":\"provider-chat-failover-backup\",\"channel_id\":\"openai\",\"adapter_kind\":\"openai\",\"base_url\":\"http://{backup_address}\",\"display_name\":\"Chat Failover Backup\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(backup_provider.status(), StatusCode::CREATED);

    let primary_credential = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/credentials")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"tenant_id\":\"{tenant_id}\",\"provider_id\":\"provider-chat-failover-primary\",\"key_reference\":\"cred-chat-failover-primary\",\"secret_value\":\"sk-chat-failover-primary\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(primary_credential.status(), StatusCode::CREATED);

    let backup_credential = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/credentials")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"tenant_id\":\"{tenant_id}\",\"provider_id\":\"provider-chat-failover-backup\",\"key_reference\":\"cred-chat-failover-backup\",\"secret_value\":\"sk-chat-failover-backup\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(backup_credential.status(), StatusCode::CREATED);

    let primary_model = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/models")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"external_name\":\"gpt-4.1\",\"provider_id\":\"provider-chat-failover-primary\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(primary_model.status(), StatusCode::CREATED);

    let backup_model = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/models")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"external_name\":\"gpt-4.1\",\"provider_id\":\"provider-chat-failover-backup\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(backup_model.status(), StatusCode::CREATED);

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
                        "policy_id": "route-chat-failover-json",
                        "capability": "chat_completion",
                        "model_pattern": "gpt-4.1",
                        "enabled": true,
                        "priority": 300,
                        "ordered_provider_ids": [
                            "provider-chat-failover-primary",
                            "provider-chat-failover-backup"
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
                    "{\"model\":\"gpt-4.1\",\"messages\":[{\"role\":\"user\",\"content\":\"fail over please\"}]}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["id"], "chatcmpl_backup");
    assert_eq!(
        primary_state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-chat-failover-primary")
    );
    assert_eq!(
        backup_state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-chat-failover-backup")
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
    assert_eq!(usage_json[0]["model"], "gpt-4.1");
    assert_eq!(usage_json[0]["provider"], "provider-chat-failover-backup");
    assert_eq!(usage_json[0]["input_tokens"], 42);
    assert_eq!(usage_json[0]["output_tokens"], 18);
    assert_eq!(usage_json[0]["total_tokens"], 60);

    let logs = admin_app
        .clone()
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
    assert_eq!(logs_json[0]["route_key"], "gpt-4.1");
    assert_eq!(
        logs_json[0]["selected_provider_id"],
        "provider-chat-failover-backup"
    );
    assert_eq!(
        logs_json[0]["fallback_reason"],
        "gateway_execution_failover"
    );

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
        "sdkwork_upstream_requests_total{service=\"gateway\",capability=\"chat_completion\",provider=\"provider-chat-failover-primary\",outcome=\"failure\"} 1"
    ));
    assert!(metrics_text.contains(
        "sdkwork_upstream_requests_total{service=\"gateway\",capability=\"chat_completion\",provider=\"provider-chat-failover-backup\",outcome=\"success\"} 1"
    ));
    assert!(metrics_text.contains(
        "sdkwork_gateway_failovers_total{service=\"gateway\",capability=\"chat_completion\",from_provider=\"provider-chat-failover-primary\",to_provider=\"provider-chat-failover-backup\",outcome=\"success\"} 1"
    ));
    assert!(metrics_text.contains(
        "sdkwork_provider_health_status{service=\"gateway\",provider=\"provider-chat-failover-primary\",runtime=\"builtin\"} 0"
    ));
    assert!(metrics_text.contains(
        "sdkwork_provider_health_status{service=\"gateway\",provider=\"provider-chat-failover-backup\",runtime=\"builtin\"} 1"
    ));
}

#[tokio::test]
async fn stateful_chat_route_does_not_fail_over_when_policy_disables_execution_failover() {
    let tenant_id = "tenant-chat-failover-disabled-json";
    let project_id = "project-chat-failover-disabled-json";
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

    let primary_provider = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"id\":\"provider-chat-failover-disabled-primary\",\"channel_id\":\"openai\",\"adapter_kind\":\"openai\",\"base_url\":\"http://{primary_address}\",\"display_name\":\"Chat Failover Disabled Primary\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(primary_provider.status(), StatusCode::CREATED);

    let backup_provider = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"id\":\"provider-chat-failover-disabled-backup\",\"channel_id\":\"openai\",\"adapter_kind\":\"openai\",\"base_url\":\"http://{backup_address}\",\"display_name\":\"Chat Failover Disabled Backup\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(backup_provider.status(), StatusCode::CREATED);

    let primary_credential = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/credentials")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"tenant_id\":\"{tenant_id}\",\"provider_id\":\"provider-chat-failover-disabled-primary\",\"key_reference\":\"cred-chat-failover-disabled-primary\",\"secret_value\":\"sk-chat-failover-disabled-primary\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(primary_credential.status(), StatusCode::CREATED);

    let backup_credential = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/credentials")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"tenant_id\":\"{tenant_id}\",\"provider_id\":\"provider-chat-failover-disabled-backup\",\"key_reference\":\"cred-chat-failover-disabled-backup\",\"secret_value\":\"sk-chat-failover-disabled-backup\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(backup_credential.status(), StatusCode::CREATED);

    let primary_model = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/models")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"external_name\":\"gpt-4.1\",\"provider_id\":\"provider-chat-failover-disabled-primary\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(primary_model.status(), StatusCode::CREATED);

    let backup_model = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/models")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"external_name\":\"gpt-4.1\",\"provider_id\":\"provider-chat-failover-disabled-backup\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(backup_model.status(), StatusCode::CREATED);

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
                        "policy_id": "route-chat-failover-disabled-json",
                        "capability": "chat_completion",
                        "model_pattern": "gpt-4.1",
                        "enabled": true,
                        "priority": 300,
                        "execution_failover_enabled": false,
                        "upstream_retry_max_attempts": 1,
                        "ordered_provider_ids": [
                            "provider-chat-failover-disabled-primary",
                            "provider-chat-failover-disabled-backup"
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
                    "{\"model\":\"gpt-4.1\",\"messages\":[{\"role\":\"user\",\"content\":\"primary must fail without failover\"}]}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
    assert_eq!(primary_state.request_count.load(Ordering::SeqCst), 1);
    assert_eq!(backup_state.request_count.load(Ordering::SeqCst), 0);

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
        "sdkwork_upstream_requests_total{service=\"gateway\",capability=\"chat_completion\",provider=\"provider-chat-failover-disabled-primary\",outcome=\"failure\"} 1"
    ));
    assert!(
        !metrics_text
            .contains("provider=\"provider-chat-failover-disabled-backup\",outcome=\"success\"")
    );
    assert!(!metrics_text.contains(
        "sdkwork_gateway_failovers_total{service=\"gateway\",capability=\"chat_completion\",from_provider=\"provider-chat-failover-disabled-primary\",to_provider=\"provider-chat-failover-disabled-backup\",outcome=\"success\"}"
    ));
}

#[tokio::test]
async fn stateful_chat_stream_route_fails_over_to_backup_provider_and_records_actual_provider() {
    let tenant_id = "tenant-chat-failover-stream";
    let project_id = "project-chat-failover-stream";
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
            post(upstream_chat_stream_handler_success),
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
            "provider-chat-stream-failover-primary",
            primary_address,
            "sk-chat-stream-failover-primary",
        ),
        (
            "provider-chat-stream-failover-backup",
            backup_address,
            "sk-chat-stream-failover-backup",
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
                        "policy_id": "route-chat-failover-stream",
                        "capability": "chat_completion",
                        "model_pattern": "gpt-4.1",
                        "enabled": true,
                        "priority": 300,
                        "ordered_provider_ids": [
                            "provider-chat-stream-failover-primary",
                            "provider-chat-stream-failover-backup"
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
                    "{\"model\":\"gpt-4.1\",\"messages\":[{\"role\":\"user\",\"content\":\"stream fail over please\"}],\"stream\":true}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let stream_body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let stream_text = String::from_utf8(stream_body.to_vec()).unwrap();
    assert!(stream_text.contains("chatcmpl_stream_backup"));
    assert_eq!(
        primary_state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-chat-stream-failover-primary")
    );
    assert_eq!(
        backup_state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-chat-stream-failover-backup")
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
    assert_eq!(
        usage_json[0]["provider"],
        "provider-chat-stream-failover-backup"
    );

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
    assert_eq!(
        logs_json[0]["selected_provider_id"],
        "provider-chat-stream-failover-backup"
    );
    assert_eq!(
        logs_json[0]["fallback_reason"],
        "gateway_execution_failover"
    );
}

