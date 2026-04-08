use super::*;

#[tokio::test]
async fn stateful_thread_runs_create_route_returns_not_found_without_usage() {
    let LocalRunsTestContext {
        admin_app,
        admin_token,
        api_key,
        gateway_app,
    } = local_runs_test_context(
        "tenant-run-create-missing",
        "project-run-create-missing",
    )
    .await;

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/threads/thread_missing/runs")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from("{\"assistant_id\":\"asst_1\"}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested thread was not found.").await;
    support::assert_no_usage_records(admin_app, &admin_token).await;
}

#[tokio::test]
async fn stateful_thread_and_run_route_returns_invalid_request_without_usage() {
    let LocalRunsTestContext {
        admin_app,
        admin_token,
        api_key,
        gateway_app,
    } = local_runs_test_context(
        "tenant-thread-and-run-invalid",
        "project-thread-and-run-invalid",
    )
    .await;

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/threads/runs")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"assistant_id\":\"\",\"thread\":{\"metadata\":{\"workspace\":\"default\"}}}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_invalid_thread_and_run_request(response).await;
    support::assert_no_usage_records(admin_app, &admin_token).await;
}

#[tokio::test]
async fn stateful_thread_runs_list_route_returns_not_found_without_usage() {
    let LocalRunsTestContext {
        admin_app,
        admin_token,
        api_key,
        gateway_app,
    } = local_runs_test_context("tenant-run-list-missing", "project-run-list-missing").await;

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/threads/thread_missing/runs")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested thread was not found.").await;
    support::assert_no_usage_records(admin_app, &admin_token).await;
}

#[tokio::test]
async fn stateful_thread_run_retrieve_route_returns_not_found_without_usage() {
    let LocalRunsTestContext {
        admin_app,
        admin_token,
        api_key,
        gateway_app,
    } = local_runs_test_context(
        "tenant-run-retrieve-missing",
        "project-run-retrieve-missing",
    )
    .await;

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/threads/thread_1/runs/run_missing")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested run was not found.").await;
    support::assert_no_usage_records(admin_app, &admin_token).await;
}

#[tokio::test]
async fn stateful_thread_run_update_route_returns_not_found_without_usage() {
    let LocalRunsTestContext {
        admin_app,
        admin_token,
        api_key,
        gateway_app,
    } = local_runs_test_context("tenant-run-update-missing", "project-run-update-missing").await;

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/threads/thread_1/runs/run_missing")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from("{\"metadata\":{\"priority\":\"high\"}}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested run was not found.").await;
    support::assert_no_usage_records(admin_app, &admin_token).await;
}

#[tokio::test]
async fn stateful_thread_run_cancel_route_returns_not_found_without_usage() {
    let LocalRunsTestContext {
        admin_app,
        admin_token,
        api_key,
        gateway_app,
    } = local_runs_test_context("tenant-run-cancel-missing", "project-run-cancel-missing").await;

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/threads/thread_1/runs/run_missing/cancel")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested run was not found.").await;
    support::assert_no_usage_records(admin_app, &admin_token).await;
}

#[tokio::test]
async fn stateful_thread_run_submit_tool_outputs_route_returns_not_found_without_usage() {
    let LocalRunsTestContext {
        admin_app,
        admin_token,
        api_key,
        gateway_app,
    } = local_runs_test_context("tenant-run-submit-missing", "project-run-submit-missing").await;

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/threads/thread_1/runs/run_missing/submit_tool_outputs")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"tool_outputs\":[{\"tool_call_id\":\"call_1\",\"output\":\"{\\\"ok\\\":true}\"}]}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested run was not found.").await;
    support::assert_no_usage_records(admin_app, &admin_token).await;
}

#[tokio::test]
async fn stateful_thread_run_steps_list_route_returns_not_found_without_usage() {
    let LocalRunsTestContext {
        admin_app,
        admin_token,
        api_key,
        gateway_app,
    } = local_runs_test_context(
        "tenant-run-steps-list-missing",
        "project-run-steps-list-missing",
    )
    .await;

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/threads/thread_1/runs/run_missing/steps")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested run was not found.").await;
    support::assert_no_usage_records(admin_app, &admin_token).await;
}

#[tokio::test]
async fn stateful_thread_run_step_route_returns_not_found_without_usage() {
    let LocalRunsTestContext {
        admin_app,
        admin_token,
        api_key,
        gateway_app,
    } = local_runs_test_context("tenant-run-step-missing", "project-run-step-missing").await;

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/threads/thread_1/runs/run_1/steps/step_missing")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested run step was not found.").await;
    support::assert_no_usage_records(admin_app, &admin_token).await;
}

#[tokio::test]
async fn stateful_thread_runs_routes_relay_to_openai_compatible_provider() {
    let upstream_state = UpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route(
            "/v1/threads/thread_1/runs",
            get(upstream_thread_runs_list_handler).post(upstream_thread_runs_handler),
        )
        .route(
            "/v1/threads/thread_1/runs/run_1",
            get(upstream_thread_run_retrieve_handler).post(upstream_thread_run_update_handler),
        )
        .route(
            "/v1/threads/thread_1/runs/run_1/cancel",
            post(upstream_thread_run_cancel_handler),
        )
        .route(
            "/v1/threads/thread_1/runs/run_1/submit_tool_outputs",
            post(upstream_thread_run_submit_tool_outputs_handler),
        )
        .route(
            "/v1/threads/thread_1/runs/run_1/steps",
            get(upstream_thread_run_steps_list_handler),
        )
        .route(
            "/v1/threads/thread_1/runs/run_1/steps/step_1",
            get(upstream_thread_run_step_retrieve_handler),
        )
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
    } = local_runs_test_context("tenant-1", "project-1").await;

    let _ = admin_app
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

    let create_response = gateway_app
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
    assert_eq!(create_response.status(), StatusCode::OK);
    let create_json = read_json(create_response).await;
    assert_eq!(create_json["id"], "run_1");
    assert_eq!(
        upstream_state.authorization_header().as_deref(),
        Some("Bearer sk-upstream-openai")
    );
    assert_eq!(
        upstream_state.beta_header().as_deref(),
        Some("assistants=v2")
    );

    let list_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/threads/thread_1/runs")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(list_response.status(), StatusCode::OK);
    let list_json = read_json(list_response).await;
    assert_eq!(list_json["data"][0]["id"], "run_1");

    let retrieve_response = gateway_app
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
    assert_eq!(retrieve_response.status(), StatusCode::OK);
    let retrieve_json = read_json(retrieve_response).await;
    assert_eq!(retrieve_json["id"], "run_1");

    let update_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/threads/thread_1/runs/run_1")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from("{\"metadata\":{\"priority\":\"high\"}}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(update_response.status(), StatusCode::OK);
    let update_json = read_json(update_response).await;
    assert_eq!(update_json["metadata"]["priority"], "high");

    let cancel_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/threads/thread_1/runs/run_1/cancel")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(cancel_response.status(), StatusCode::OK);
    let cancel_json = read_json(cancel_response).await;
    assert_eq!(cancel_json["status"], "cancelled");

    let submit_tool_outputs_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/threads/thread_1/runs/run_1/submit_tool_outputs")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"tool_outputs\":[{\"tool_call_id\":\"call_1\",\"output\":\"{\\\"ok\\\":true}\"}]}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(submit_tool_outputs_response.status(), StatusCode::OK);
    let submit_tool_outputs_json = read_json(submit_tool_outputs_response).await;
    assert_eq!(submit_tool_outputs_json["id"], "run_1");

    let steps_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/threads/thread_1/runs/run_1/steps")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(steps_response.status(), StatusCode::OK);
    let steps_json = read_json(steps_response).await;
    assert_eq!(steps_json["data"][0]["id"], "step_1");

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

    let create_and_run_response = gateway_app
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
    assert_eq!(create_and_run_response.status(), StatusCode::OK);
    let create_and_run_json = read_json(create_and_run_response).await;
    assert_eq!(create_and_run_json["id"], "run_1");
}
