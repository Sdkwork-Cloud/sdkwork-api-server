use super::*;

#[tokio::test]
async fn stateful_thread_run_usage_uses_thread_route_key_for_provider_selection() {
    let tenant_id = "tenant-run-usage";
    let project_id = "project-run-usage";
    let upstream_state = UpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route(
            "/v1/threads/thread_1/runs/run_1",
            get(upstream_thread_run_retrieve_handler),
        )
        .route(
            "/v1/threads/thread_1/runs/run_1/steps/step_1",
            get(upstream_thread_run_step_retrieve_handler),
        )
        .with_state(upstream_state.clone());

    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });

    let LocalRunsTestContext {
        admin_app,
        admin_token,
        api_key,
        gateway_app,
    } = local_runs_test_context(tenant_id, project_id).await;

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

    let provider_thread = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"id\":\"provider-thread\",\"channel_id\":\"openai\",\"adapter_kind\":\"openai\",\"base_url\":\"http://{address}\",\"display_name\":\"Thread Provider\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(provider_thread.status(), StatusCode::CREATED);

    let provider_child = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"id\":\"provider-child\",\"channel_id\":\"openai\",\"adapter_kind\":\"openai\",\"base_url\":\"http://127.0.0.1:1\",\"display_name\":\"Child Provider\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(provider_child.status(), StatusCode::CREATED);

    let thread_credential = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/credentials")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"tenant_id\":\"{tenant_id}\",\"provider_id\":\"provider-thread\",\"key_reference\":\"cred-thread\",\"secret_value\":\"sk-thread\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(thread_credential.status(), StatusCode::CREATED);

    let child_credential = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/credentials")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"tenant_id\":\"{tenant_id}\",\"provider_id\":\"provider-child\",\"key_reference\":\"cred-child\",\"secret_value\":\"sk-child\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(child_credential.status(), StatusCode::CREATED);

    let thread_policy = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/routing/policies")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "policy_id": "route-runs-by-thread",
                        "capability": "assistants",
                        "model_pattern": "thread_1",
                        "enabled": true,
                        "priority": 200,
                        "ordered_provider_ids": ["provider-thread"]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(thread_policy.status(), StatusCode::CREATED);

    let run_policy = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/routing/policies")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "policy_id": "route-runs-by-run",
                        "capability": "assistants",
                        "model_pattern": "run_1",
                        "enabled": true,
                        "priority": 100,
                        "ordered_provider_ids": ["provider-child"]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(run_policy.status(), StatusCode::CREATED);

    let step_policy = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/routing/policies")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "policy_id": "route-runs-by-step",
                        "capability": "assistants",
                        "model_pattern": "step_1",
                        "enabled": true,
                        "priority": 100,
                        "ordered_provider_ids": ["provider-child"]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(step_policy.status(), StatusCode::CREATED);

    let run_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/threads/thread_1/runs/run_1")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(run_response.status(), StatusCode::OK);
    let run_json = read_json(run_response).await;
    assert_eq!(run_json["id"], "run_1");

    let step_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/threads/thread_1/runs/run_1/steps/step_1")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(step_response.status(), StatusCode::OK);
    let step_json = read_json(step_response).await;
    assert_eq!(step_json["id"], "step_1");
    assert_eq!(
        upstream_state.authorization_header().as_deref(),
        Some("Bearer sk-thread")
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
    let usage_records = usage_json.as_array().unwrap();
    assert_eq!(usage_records.len(), 2);
    assert_eq!(usage_records[0]["model"], "step_1");
    assert_eq!(usage_records[0]["provider"], "provider-thread");
    assert_eq!(usage_records[1]["model"], "run_1");
    assert_eq!(usage_records[1]["provider"], "provider-thread");

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
    let routing_logs = logs_json.as_array().unwrap();
    assert_eq!(routing_logs.len(), 2);
    assert!(routing_logs
        .iter()
        .all(|entry| entry["route_key"] == "thread_1"));
    assert!(routing_logs
        .iter()
        .all(|entry| entry["selected_provider_id"] == "provider-thread"));
}

#[tokio::test]
async fn stateful_thread_run_create_usage_uses_thread_route_key_for_provider_selection() {
    let tenant_id = "tenant-run-create";
    let project_id = "project-run-create";
    let upstream_state = UpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route(
            "/v1/threads/thread_1/runs",
            post(upstream_thread_runs_handler),
        )
        .with_state(upstream_state.clone());

    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });

    let LocalRunsTestContext {
        admin_app,
        admin_token,
        api_key,
        gateway_app,
    } = local_runs_test_context(tenant_id, project_id).await;

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

    let provider_thread = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"id\":\"provider-thread-create\",\"channel_id\":\"openai\",\"adapter_kind\":\"openai\",\"base_url\":\"http://{address}\",\"display_name\":\"Thread Provider\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(provider_thread.status(), StatusCode::CREATED);

    let provider_run = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"id\":\"provider-run-create\",\"channel_id\":\"openai\",\"adapter_kind\":\"openai\",\"base_url\":\"http://127.0.0.1:1\",\"display_name\":\"Run Provider\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(provider_run.status(), StatusCode::CREATED);

    let thread_credential = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/credentials")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"tenant_id\":\"{tenant_id}\",\"provider_id\":\"provider-thread-create\",\"key_reference\":\"cred-thread-create\",\"secret_value\":\"sk-thread-create\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(thread_credential.status(), StatusCode::CREATED);

    let run_credential = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/credentials")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"tenant_id\":\"{tenant_id}\",\"provider_id\":\"provider-run-create\",\"key_reference\":\"cred-run-create\",\"secret_value\":\"sk-run-create\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(run_credential.status(), StatusCode::CREATED);

    let thread_policy = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/routing/policies")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "policy_id": "route-thread-run-create-by-thread",
                        "capability": "assistants",
                        "model_pattern": "thread_1",
                        "enabled": true,
                        "priority": 200,
                        "ordered_provider_ids": ["provider-thread-create"]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(thread_policy.status(), StatusCode::CREATED);

    let run_policy = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/routing/policies")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "policy_id": "route-thread-run-create-by-run",
                        "capability": "assistants",
                        "model_pattern": "run_1",
                        "enabled": true,
                        "priority": 100,
                        "ordered_provider_ids": ["provider-run-create"]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(run_policy.status(), StatusCode::CREATED);

    let response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/threads/thread_1/runs")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from("{\"assistant_id\":\"asst_1\"}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let response_json = read_json(response).await;
    assert_eq!(response_json["id"], "run_1");
    assert_eq!(
        upstream_state.authorization_header().as_deref(),
        Some("Bearer sk-thread-create")
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
    assert_eq!(usage_json[0]["model"], "run_1");
    assert_eq!(usage_json[0]["provider"], "provider-thread-create");

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
    assert_eq!(logs_json[0]["route_key"], "thread_1");
    assert_eq!(
        logs_json[0]["selected_provider_id"],
        "provider-thread-create"
    );
}

#[tokio::test]
async fn stateful_thread_and_run_create_usage_uses_created_run_id_for_billing() {
    let tenant_id = "tenant-thread-and-run-create";
    let project_id = "project-thread-and-run-create";
    let upstream_state = UpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route("/v1/threads/runs", post(upstream_thread_and_run_handler))
        .with_state(upstream_state.clone());

    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });

    let LocalRunsTestContext {
        admin_app,
        admin_token,
        api_key,
        gateway_app,
    } = local_runs_test_context(tenant_id, project_id).await;

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
                    "{{\"id\":\"provider-thread-and-run-route\",\"channel_id\":\"openai\",\"adapter_kind\":\"openai\",\"base_url\":\"http://{address}\",\"display_name\":\"Thread And Run Route Provider\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(provider_route.status(), StatusCode::CREATED);

    let provider_run = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"id\":\"provider-thread-and-run-child\",\"channel_id\":\"openai\",\"adapter_kind\":\"openai\",\"base_url\":\"http://127.0.0.1:1\",\"display_name\":\"Thread And Run Child Provider\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(provider_run.status(), StatusCode::CREATED);

    let route_credential = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/credentials")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"tenant_id\":\"{tenant_id}\",\"provider_id\":\"provider-thread-and-run-route\",\"key_reference\":\"cred-thread-and-run-route\",\"secret_value\":\"sk-thread-and-run-route\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(route_credential.status(), StatusCode::CREATED);

    let run_credential = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/credentials")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"tenant_id\":\"{tenant_id}\",\"provider_id\":\"provider-thread-and-run-child\",\"key_reference\":\"cred-thread-and-run-child\",\"secret_value\":\"sk-thread-and-run-child\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(run_credential.status(), StatusCode::CREATED);

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
                        "policy_id": "route-thread-and-run-by-generic-route",
                        "capability": "assistants",
                        "model_pattern": "threads/runs",
                        "enabled": true,
                        "priority": 200,
                        "ordered_provider_ids": ["provider-thread-and-run-route"]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(route_policy.status(), StatusCode::CREATED);

    let run_policy = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/routing/policies")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "policy_id": "route-thread-and-run-by-run-id",
                        "capability": "assistants",
                        "model_pattern": "run_1",
                        "enabled": true,
                        "priority": 100,
                        "ordered_provider_ids": ["provider-thread-and-run-child"]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(run_policy.status(), StatusCode::CREATED);

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/threads/runs")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"assistant_id\":\"asst_1\",\"thread\":{\"metadata\":{\"workspace\":\"default\"}}}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let response_json = read_json(response).await;
    assert_eq!(response_json["id"], "run_1");
    assert_eq!(
        upstream_state.authorization_header().as_deref(),
        Some("Bearer sk-thread-and-run-route")
    );
    support::assert_single_usage_record_and_decision_log(
        admin_app,
        &admin_token,
        "run_1",
        "provider-thread-and-run-route",
        "threads/runs",
    )
    .await;
}
