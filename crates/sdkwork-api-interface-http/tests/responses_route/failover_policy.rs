use super::*;

#[serial(extension_env)]
#[tokio::test]
async fn stateful_responses_route_fails_over_to_backup_provider_and_records_actual_provider() {
    let tenant_id = "tenant-responses-failover-json";
    let project_id = "project-responses-failover-json";
    let primary_state = UpstreamCaptureState::default();
    let backup_state = UpstreamCaptureState::default();

    let primary_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let primary_address = primary_listener.local_addr().unwrap();
    let primary_upstream = Router::new()
        .route("/v1/responses", post(upstream_responses_handler_failure))
        .with_state(primary_state.clone());
    tokio::spawn(async move {
        axum::serve(primary_listener, primary_upstream)
            .await
            .unwrap();
    });

    let backup_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let backup_address = backup_listener.local_addr().unwrap();
    let backup_upstream = Router::new()
        .route("/v1/responses", post(upstream_responses_handler_with_usage))
        .with_state(backup_state.clone());
    tokio::spawn(async move {
        axum::serve(backup_listener, backup_upstream).await.unwrap();
    });

    let pool = memory_pool().await;
    let api_key = support::issue_gateway_api_key(&pool, tenant_id, project_id).await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool);

    create_openai_channel(&admin_app, &admin_token).await;
    create_stateful_openai_provider_for_responses(
        &admin_app,
        &admin_token,
        tenant_id,
        "provider-responses-failover-primary",
        &format!("http://{primary_address}"),
        "Responses Failover Primary",
        "cred-responses-failover-primary",
        "sk-responses-failover-primary",
    )
    .await;
    create_stateful_openai_provider_for_responses(
        &admin_app,
        &admin_token,
        tenant_id,
        "provider-responses-failover-backup",
        &format!("http://{backup_address}"),
        "Responses Failover Backup",
        "cred-responses-failover-backup",
        "sk-responses-failover-backup",
    )
    .await;
    create_responses_routing_policy(
        &admin_app,
        &admin_token,
        "route-responses-failover-json",
        vec![
            "provider-responses-failover-primary",
            "provider-responses-failover-backup",
        ],
    )
    .await;

    let response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/responses")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"gpt-4.1\",\"input\":\"fail over please for responses\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["id"], "resp_upstream");
    assert_eq!(
        primary_state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-responses-failover-primary")
    );
    assert_eq!(
        backup_state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-responses-failover-backup")
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
    assert_eq!(
        usage_json[0]["provider"],
        "provider-responses-failover-backup"
    );
    assert_eq!(usage_json[0]["input_tokens"], 160);
    assert_eq!(usage_json[0]["output_tokens"], 40);
    assert_eq!(usage_json[0]["total_tokens"], 200);

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
        "provider-responses-failover-backup"
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
        "sdkwork_upstream_requests_total{service=\"gateway\",capability=\"responses\",provider=\"provider-responses-failover-primary\",outcome=\"failure\"} 1"
    ));
    assert!(metrics_text.contains(
        "sdkwork_upstream_requests_total{service=\"gateway\",capability=\"responses\",provider=\"provider-responses-failover-backup\",outcome=\"success\"} 1"
    ));
    assert!(metrics_text.contains(
        "sdkwork_gateway_failovers_total{service=\"gateway\",capability=\"responses\",from_provider=\"provider-responses-failover-primary\",to_provider=\"provider-responses-failover-backup\",outcome=\"success\"} 1"
    ));
}

#[serial(extension_env)]
#[tokio::test]
async fn stateful_responses_route_does_not_fail_over_when_policy_disables_execution_failover() {
    let tenant_id = "tenant-responses-failover-disabled-json";
    let project_id = "project-responses-failover-disabled-json";
    let primary_state = UpstreamCaptureState::default();
    let backup_state = UpstreamCaptureState::default();

    let primary_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let primary_address = primary_listener.local_addr().unwrap();
    let primary_upstream = Router::new()
        .route("/v1/responses", post(upstream_responses_handler_failure))
        .with_state(primary_state.clone());
    tokio::spawn(async move {
        axum::serve(primary_listener, primary_upstream)
            .await
            .unwrap();
    });

    let backup_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let backup_address = backup_listener.local_addr().unwrap();
    let backup_upstream = Router::new()
        .route("/v1/responses", post(upstream_responses_handler_with_usage))
        .with_state(backup_state.clone());
    tokio::spawn(async move {
        axum::serve(backup_listener, backup_upstream).await.unwrap();
    });

    let pool = memory_pool().await;
    let api_key = support::issue_gateway_api_key(&pool, tenant_id, project_id).await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool);

    create_openai_channel(&admin_app, &admin_token).await;
    create_stateful_openai_provider_for_responses(
        &admin_app,
        &admin_token,
        tenant_id,
        "provider-responses-failover-disabled-primary",
        &format!("http://{primary_address}"),
        "Responses Failover Disabled Primary",
        "cred-responses-failover-disabled-primary",
        "sk-responses-failover-disabled-primary",
    )
    .await;
    create_stateful_openai_provider_for_responses(
        &admin_app,
        &admin_token,
        tenant_id,
        "provider-responses-failover-disabled-backup",
        &format!("http://{backup_address}"),
        "Responses Failover Disabled Backup",
        "cred-responses-failover-disabled-backup",
        "sk-responses-failover-disabled-backup",
    )
    .await;
    create_responses_routing_policy_with_overrides(
        &admin_app,
        &admin_token,
        "route-responses-failover-disabled-json",
        vec![
            "provider-responses-failover-disabled-primary",
            "provider-responses-failover-disabled-backup",
        ],
        serde_json::json!({
            "execution_failover_enabled": false,
            "upstream_retry_max_attempts": 1
        }),
    )
    .await;

    let response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/responses")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"gpt-4.1\",\"input\":\"responses failover must stay disabled\"}",
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
        "sdkwork_upstream_requests_total{service=\"gateway\",capability=\"responses\",provider=\"provider-responses-failover-disabled-primary\",outcome=\"failure\"} 1"
    ));
    assert!(
        !metrics_text.contains(
            "provider=\"provider-responses-failover-disabled-backup\",outcome=\"success\""
        )
    );
    assert!(!metrics_text.contains(
        "sdkwork_gateway_failovers_total{service=\"gateway\",capability=\"responses\",from_provider=\"provider-responses-failover-disabled-primary\",to_provider=\"provider-responses-failover-disabled-backup\",outcome=\"success\"}"
    ));
}

#[serial(extension_env)]
#[tokio::test]
async fn stateful_responses_stream_route_fails_over_to_backup_provider_and_records_actual_provider()
{
    let tenant_id = "tenant-responses-failover-stream";
    let project_id = "project-responses-failover-stream";
    let primary_state = UpstreamCaptureState::default();
    let backup_state = UpstreamCaptureState::default();

    let primary_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let primary_address = primary_listener.local_addr().unwrap();
    let primary_upstream = Router::new()
        .route("/v1/responses", post(upstream_responses_handler_failure))
        .with_state(primary_state.clone());
    tokio::spawn(async move {
        axum::serve(primary_listener, primary_upstream)
            .await
            .unwrap();
    });

    let backup_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let backup_address = backup_listener.local_addr().unwrap();
    let backup_upstream = Router::new()
        .route("/v1/responses", post(upstream_responses_stream_handler))
        .with_state(backup_state.clone());
    tokio::spawn(async move {
        axum::serve(backup_listener, backup_upstream).await.unwrap();
    });

    let pool = memory_pool().await;
    let api_key = support::issue_gateway_api_key(&pool, tenant_id, project_id).await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool);

    create_openai_channel(&admin_app, &admin_token).await;
    create_stateful_openai_provider_for_responses(
        &admin_app,
        &admin_token,
        tenant_id,
        "provider-responses-stream-failover-primary",
        &format!("http://{primary_address}"),
        "Responses Stream Failover Primary",
        "cred-responses-stream-failover-primary",
        "sk-responses-stream-failover-primary",
    )
    .await;
    create_stateful_openai_provider_for_responses(
        &admin_app,
        &admin_token,
        tenant_id,
        "provider-responses-stream-failover-backup",
        &format!("http://{backup_address}"),
        "Responses Stream Failover Backup",
        "cred-responses-stream-failover-backup",
        "sk-responses-stream-failover-backup",
    )
    .await;
    create_responses_routing_policy(
        &admin_app,
        &admin_token,
        "route-responses-failover-stream",
        vec![
            "provider-responses-stream-failover-primary",
            "provider-responses-stream-failover-backup",
        ],
    )
    .await;

    let response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/responses")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"gpt-4.1\",\"input\":\"stream fail over please for responses\",\"stream\":true}",
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
    assert!(stream_text.contains("resp_upstream_stream"));
    assert_eq!(
        primary_state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-responses-stream-failover-primary")
    );
    assert_eq!(
        backup_state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-responses-stream-failover-backup")
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
        "provider-responses-stream-failover-backup"
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
        "provider-responses-stream-failover-backup"
    );
    assert_eq!(
        logs_json[0]["fallback_reason"],
        "gateway_execution_failover"
    );
}
