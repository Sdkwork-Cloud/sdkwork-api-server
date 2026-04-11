use super::*;

#[tokio::test]
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
async fn stateful_chat_route_fails_over_before_execution_when_primary_lacks_tenant_credential() {
    let tenant_id = "tenant-chat-preflight-missing-credential-json";
    let project_id = "project-chat-preflight-missing-credential-json";
    let primary_state = UpstreamCaptureState::default();
    let backup_state = UpstreamCaptureState::default();

    let primary_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let primary_address = primary_listener.local_addr().unwrap();
    let primary_upstream = Router::new()
        .route("/v1/chat/completions", post(upstream_chat_handler_with_usage))
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

    create_openai_channel_for_chat_failover(&admin_app, &admin_token).await;
    create_stateful_openai_provider_for_chat_failover(
        &admin_app,
        &admin_token,
        tenant_id,
        "provider-chat-preflight-primary",
        &format!("http://{primary_address}"),
        "Chat Preflight Primary",
        None,
        None,
    )
    .await;
    create_stateful_openai_provider_for_chat_failover(
        &admin_app,
        &admin_token,
        tenant_id,
        "provider-chat-preflight-backup",
        &format!("http://{backup_address}"),
        "Chat Preflight Backup",
        Some("cred-chat-preflight-backup"),
        Some("sk-chat-preflight-backup"),
    )
    .await;
    create_chat_routing_policy_for_failover(
        &admin_app,
        &admin_token,
        "route-chat-preflight-missing-credential-json",
        vec![
            "provider-chat-preflight-primary",
            "provider-chat-preflight-backup",
        ],
        serde_json::json!({}),
    )
    .await;

    let response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/chat/completions")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"gpt-4.1\",\"messages\":[{\"role\":\"user\",\"content\":\"preflight fail over please\"}]}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["id"], "chatcmpl_backup");
    assert_eq!(primary_state.request_count.load(Ordering::SeqCst), 0);
    assert_eq!(backup_state.request_count.load(Ordering::SeqCst), 1);
    assert!(primary_state.authorization.lock().unwrap().is_none());
    assert_eq!(
        backup_state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-chat-preflight-backup")
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
    assert_eq!(usage_json[0]["provider"], "provider-chat-preflight-backup");

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
    assert_eq!(
        logs_json[0]["selected_provider_id"],
        "provider-chat-preflight-backup"
    );
    assert_eq!(
        logs_json[0]["fallback_reason"],
        "gateway_execution_failover"
    );
}

#[tokio::test]
async fn stateful_chat_route_fails_over_before_execution_when_primary_requires_non_openai_standard_without_plugin()
{
    let tenant_id = "tenant-chat-preflight-incompatible-standard-json";
    let project_id = "project-chat-preflight-incompatible-standard-json";
    let primary_state = UpstreamCaptureState::default();
    let backup_state = UpstreamCaptureState::default();

    let primary_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let primary_address = primary_listener.local_addr().unwrap();
    let primary_upstream = Router::new()
        .route("/v1/messages", post(upstream_chat_handler_with_usage))
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

    create_openai_channel_for_chat_failover(&admin_app, &admin_token).await;

    let primary_provider = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"id\":\"provider-chat-preflight-incompatible-primary\",\"channel_id\":\"openai\",\"adapter_kind\":\"anthropic\",\"base_url\":\"http://{primary_address}\",\"display_name\":\"Chat Preflight Incompatible Primary\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(primary_provider.status(), StatusCode::CREATED);

    let primary_credential = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/credentials")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"tenant_id\":\"{tenant_id}\",\"provider_id\":\"provider-chat-preflight-incompatible-primary\",\"key_reference\":\"cred-chat-preflight-incompatible-primary\",\"secret_value\":\"sk-chat-preflight-incompatible-primary\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(primary_credential.status(), StatusCode::CREATED);

    let primary_model = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/models")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"external_name\":\"gpt-4.1\",\"provider_id\":\"provider-chat-preflight-incompatible-primary\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(primary_model.status(), StatusCode::CREATED);

    create_stateful_openai_provider_for_chat_failover(
        &admin_app,
        &admin_token,
        tenant_id,
        "provider-chat-preflight-incompatible-backup",
        &format!("http://{backup_address}"),
        "Chat Preflight Incompatible Backup",
        Some("cred-chat-preflight-incompatible-backup"),
        Some("sk-chat-preflight-incompatible-backup"),
    )
    .await;
    create_chat_routing_policy_for_failover(
        &admin_app,
        &admin_token,
        "route-chat-preflight-incompatible-standard-json",
        vec![
            "provider-chat-preflight-incompatible-primary",
            "provider-chat-preflight-incompatible-backup",
        ],
        serde_json::json!({}),
    )
    .await;

    let response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/chat/completions")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"gpt-4.1\",\"messages\":[{\"role\":\"user\",\"content\":\"use the compatible backup\"}]}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["id"], "chatcmpl_backup");
    assert_eq!(primary_state.request_count.load(Ordering::SeqCst), 0);
    assert_eq!(backup_state.request_count.load(Ordering::SeqCst), 1);
    assert!(primary_state.authorization.lock().unwrap().is_none());
    assert_eq!(
        backup_state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-chat-preflight-incompatible-backup")
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
        "provider-chat-preflight-incompatible-backup"
    );

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
    assert_eq!(
        logs_json[0]["selected_provider_id"],
        "provider-chat-preflight-incompatible-backup"
    );
    assert_eq!(
        logs_json[0]["fallback_reason"],
        "gateway_execution_failover"
    );
}

#[tokio::test]
async fn stateful_chat_route_fails_over_before_execution_when_primary_provider_is_missing() {
    let tenant_id = "tenant-chat-preflight-missing-provider-json";
    let project_id = "project-chat-preflight-missing-provider-json";
    let backup_state = UpstreamCaptureState::default();

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

    create_openai_channel_for_chat_failover(&admin_app, &admin_token).await;
    create_stateful_openai_provider_for_chat_failover(
        &admin_app,
        &admin_token,
        tenant_id,
        "provider-chat-preflight-backup-missing-primary",
        &format!("http://{backup_address}"),
        "Chat Preflight Backup Missing Primary",
        Some("cred-chat-preflight-backup-missing-primary"),
        Some("sk-chat-preflight-backup-missing-primary"),
    )
    .await;
    create_chat_routing_policy_for_failover(
        &admin_app,
        &admin_token,
        "route-chat-preflight-missing-provider-json",
        vec![
            "provider-chat-preflight-primary-missing",
            "provider-chat-preflight-backup-missing-primary",
        ],
        serde_json::json!({}),
    )
    .await;

    let response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/chat/completions")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"gpt-4.1\",\"messages\":[{\"role\":\"user\",\"content\":\"missing provider fail over please\"}]}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["id"], "chatcmpl_backup");
    assert_eq!(backup_state.request_count.load(Ordering::SeqCst), 1);
    assert_eq!(
        backup_state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-chat-preflight-backup-missing-primary")
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
        "provider-chat-preflight-backup-missing-primary"
    );

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
    assert_eq!(
        logs_json[0]["selected_provider_id"],
        "provider-chat-preflight-backup-missing-primary"
    );
    assert_eq!(
        logs_json[0]["fallback_reason"],
        "policy_candidate_unavailable"
    );
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
async fn stateful_chat_stream_route_fails_over_before_execution_when_primary_lacks_tenant_credential()
{
    let tenant_id = "tenant-chat-preflight-missing-credential-stream";
    let project_id = "project-chat-preflight-missing-credential-stream";
    let primary_state = UpstreamCaptureState::default();
    let backup_state = UpstreamCaptureState::default();

    let primary_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let primary_address = primary_listener.local_addr().unwrap();
    let primary_upstream = Router::new()
        .route("/v1/chat/completions", post(upstream_chat_stream_handler_success))
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

    create_openai_channel_for_chat_failover(&admin_app, &admin_token).await;
    create_stateful_openai_provider_for_chat_failover(
        &admin_app,
        &admin_token,
        tenant_id,
        "provider-chat-stream-preflight-primary",
        &format!("http://{primary_address}"),
        "Chat Stream Preflight Primary",
        None,
        None,
    )
    .await;
    create_stateful_openai_provider_for_chat_failover(
        &admin_app,
        &admin_token,
        tenant_id,
        "provider-chat-stream-preflight-backup",
        &format!("http://{backup_address}"),
        "Chat Stream Preflight Backup",
        Some("cred-chat-stream-preflight-backup"),
        Some("sk-chat-stream-preflight-backup"),
    )
    .await;
    create_chat_routing_policy_for_failover(
        &admin_app,
        &admin_token,
        "route-chat-preflight-missing-credential-stream",
        vec![
            "provider-chat-stream-preflight-primary",
            "provider-chat-stream-preflight-backup",
        ],
        serde_json::json!({}),
    )
    .await;

    let response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/chat/completions")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"gpt-4.1\",\"messages\":[{\"role\":\"user\",\"content\":\"preflight stream fail over please\"}],\"stream\":true}",
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
    assert_eq!(primary_state.request_count.load(Ordering::SeqCst), 0);
    assert_eq!(backup_state.request_count.load(Ordering::SeqCst), 1);
    assert!(primary_state.authorization.lock().unwrap().is_none());
    assert_eq!(
        backup_state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-chat-stream-preflight-backup")
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
        "provider-chat-stream-preflight-backup"
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
        "provider-chat-stream-preflight-backup"
    );
    assert_eq!(
        logs_json[0]["fallback_reason"],
        "gateway_execution_failover"
    );
}

#[tokio::test]
async fn stateful_chat_stream_route_fails_over_before_execution_when_primary_requires_non_openai_standard_without_plugin()
{
    let tenant_id = "tenant-chat-stream-preflight-incompatible-standard";
    let project_id = "project-chat-stream-preflight-incompatible-standard";
    let primary_state = UpstreamCaptureState::default();
    let backup_state = UpstreamCaptureState::default();

    let primary_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let primary_address = primary_listener.local_addr().unwrap();
    let primary_upstream = Router::new()
        .route("/v1/messages", post(upstream_chat_stream_handler_success))
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

    create_openai_channel_for_chat_failover(&admin_app, &admin_token).await;

    let primary_provider = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"id\":\"provider-chat-stream-preflight-incompatible-primary\",\"channel_id\":\"openai\",\"adapter_kind\":\"anthropic\",\"base_url\":\"http://{primary_address}\",\"display_name\":\"Chat Stream Preflight Incompatible Primary\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(primary_provider.status(), StatusCode::CREATED);

    let primary_credential = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/credentials")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"tenant_id\":\"{tenant_id}\",\"provider_id\":\"provider-chat-stream-preflight-incompatible-primary\",\"key_reference\":\"cred-chat-stream-preflight-incompatible-primary\",\"secret_value\":\"sk-chat-stream-preflight-incompatible-primary\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(primary_credential.status(), StatusCode::CREATED);

    let primary_model = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/models")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"external_name\":\"gpt-4.1\",\"provider_id\":\"provider-chat-stream-preflight-incompatible-primary\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(primary_model.status(), StatusCode::CREATED);

    create_stateful_openai_provider_for_chat_failover(
        &admin_app,
        &admin_token,
        tenant_id,
        "provider-chat-stream-preflight-incompatible-backup",
        &format!("http://{backup_address}"),
        "Chat Stream Preflight Incompatible Backup",
        Some("cred-chat-stream-preflight-incompatible-backup"),
        Some("sk-chat-stream-preflight-incompatible-backup"),
    )
    .await;
    create_chat_routing_policy_for_failover(
        &admin_app,
        &admin_token,
        "route-chat-stream-preflight-incompatible-standard",
        vec![
            "provider-chat-stream-preflight-incompatible-primary",
            "provider-chat-stream-preflight-incompatible-backup",
        ],
        serde_json::json!({}),
    )
    .await;

    let response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/chat/completions")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"gpt-4.1\",\"messages\":[{\"role\":\"user\",\"content\":\"use the compatible stream backup\"}],\"stream\":true}",
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
    assert_eq!(primary_state.request_count.load(Ordering::SeqCst), 0);
    assert_eq!(backup_state.request_count.load(Ordering::SeqCst), 1);
    assert!(primary_state.authorization.lock().unwrap().is_none());
    assert_eq!(
        backup_state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-chat-stream-preflight-incompatible-backup")
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
        "provider-chat-stream-preflight-incompatible-backup"
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
        "provider-chat-stream-preflight-incompatible-backup"
    );
    assert_eq!(
        logs_json[0]["fallback_reason"],
        "gateway_execution_failover"
    );
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

async fn create_openai_channel_for_chat_failover(admin_app: &Router, admin_token: &str) {
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
}

async fn create_stateful_openai_provider_for_chat_failover(
    admin_app: &Router,
    admin_token: &str,
    tenant_id: &str,
    provider_id: &str,
    base_url: &str,
    display_name: &str,
    credential_ref: Option<&str>,
    secret_value: Option<&str>,
) {
    let provider = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"id\":\"{provider_id}\",\"channel_id\":\"openai\",\"adapter_kind\":\"openai\",\"base_url\":\"{base_url}\",\"display_name\":\"{display_name}\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(provider.status(), StatusCode::CREATED);

    if let (Some(credential_ref), Some(secret_value)) = (credential_ref, secret_value) {
        let credential = admin_app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/admin/credentials")
                    .header("authorization", format!("Bearer {admin_token}"))
                    .header("content-type", "application/json")
                    .body(Body::from(format!(
                        "{{\"tenant_id\":\"{tenant_id}\",\"provider_id\":\"{provider_id}\",\"key_reference\":\"{credential_ref}\",\"secret_value\":\"{secret_value}\"}}"
                    )))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(credential.status(), StatusCode::CREATED);
    }

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

async fn create_chat_routing_policy_for_failover(
    admin_app: &Router,
    admin_token: &str,
    policy_id: &str,
    ordered_provider_ids: Vec<&str>,
    overrides: Value,
) {
    let mut body = serde_json::json!({
        "policy_id": policy_id,
        "capability": "chat_completion",
        "model_pattern": "gpt-4.1",
        "enabled": true,
        "priority": 300,
        "ordered_provider_ids": ordered_provider_ids
    });
    let body_object = body
        .as_object_mut()
        .expect("routing policy body should be an object");
    if let Some(overrides_object) = overrides.as_object() {
        body_object.extend(
            overrides_object
                .iter()
                .map(|(key, value)| (key.clone(), value.clone())),
        );
    }

    let policy = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/routing/policies")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(policy.status(), StatusCode::CREATED);
}

