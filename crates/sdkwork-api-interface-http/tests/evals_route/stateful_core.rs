use super::*;

#[tokio::test]
async fn stateful_evals_route_relays_to_openai_compatible_provider() {
    let upstream_state = UpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route(
            "/v1/evals",
            get(upstream_evals_list_handler).post(upstream_evals_handler),
        )
        .route(
            "/v1/evals/eval_1",
            get(upstream_eval_retrieve_handler)
                .post(upstream_eval_update_handler)
                .delete(upstream_eval_delete_handler),
        )
        .route(
            "/v1/evals/eval_1/runs",
            get(upstream_eval_runs_list_handler).post(upstream_eval_runs_create_handler),
        )
        .route(
            "/v1/evals/eval_1/runs/run_1",
            get(upstream_eval_run_retrieve_handler),
        )
        .with_state(upstream_state.clone());

    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });

    let pool = memory_pool().await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
    let api_key = support::issue_gateway_api_key(&pool, "tenant-1", "project-1").await;
    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool);

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

    let response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/evals")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"name\":\"qa-benchmark\",\"data_source_config\":{\"type\":\"file\",\"file_id\":\"file_1\"}}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["id"], "eval_upstream");
    assert_eq!(
        upstream_state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-upstream-openai")
    );
    support::assert_single_usage_record_and_decision_log(
        admin_app.clone(),
        &admin_token,
        "eval_upstream",
        "provider-openai-official",
        "qa-benchmark",
    )
    .await;

    let list_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/evals")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(list_response.status(), StatusCode::OK);
    let list_json = read_json(list_response).await;
    assert_eq!(list_json["data"][0]["id"], "eval_1");

    let retrieve_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/evals/eval_1")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(retrieve_response.status(), StatusCode::OK);
    let retrieve_json = read_json(retrieve_response).await;
    assert_eq!(retrieve_json["id"], "eval_1");

    let update_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/evals/eval_1")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from("{\"name\":\"qa-benchmark-updated\"}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(update_response.status(), StatusCode::OK);
    let update_json = read_json(update_response).await;
    assert_eq!(update_json["name"], "qa-benchmark-updated");

    let runs_list_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/evals/eval_1/runs")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(runs_list_response.status(), StatusCode::OK);
    let runs_list_json = read_json(runs_list_response).await;
    assert_eq!(runs_list_json["data"][0]["id"], "run_1");

    let run_create_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/evals/eval_1/runs")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from("{\"name\":\"daily-regression\"}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(run_create_response.status(), StatusCode::OK);
    let run_create_json = read_json(run_create_response).await;
    assert_eq!(run_create_json["id"], "run_upstream");

    let run_retrieve_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/evals/eval_1/runs/run_1")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(run_retrieve_response.status(), StatusCode::OK);
    let run_retrieve_json = read_json(run_retrieve_response).await;
    assert_eq!(run_retrieve_json["id"], "run_1");

    let delete_response = gateway_app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/v1/evals/eval_1")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(delete_response.status(), StatusCode::OK);
    let delete_json = read_json(delete_response).await;
    assert_eq!(delete_json["deleted"], true);
}

#[tokio::test]
async fn stateful_eval_retrieve_route_returns_not_found_without_usage() {
    let ctx = local_evals_test_context(
        "tenant-eval-retrieve-missing",
        "project-eval-retrieve-missing",
    )
    .await;

    let response = ctx
        .gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/evals/eval_missing")
                .header("authorization", format!("Bearer {}", ctx.api_key))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested eval was not found.").await;
    support::assert_no_usage_records(ctx.admin_app, &ctx.admin_token).await;
}

#[tokio::test]
async fn stateful_eval_update_route_returns_not_found_without_usage() {
    let ctx =
        local_evals_test_context("tenant-eval-update-missing", "project-eval-update-missing")
            .await;

    let response = ctx
        .gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/evals/eval_missing")
                .header("authorization", format!("Bearer {}", ctx.api_key))
                .header("content-type", "application/json")
                .body(Body::from("{\"name\":\"qa-benchmark-updated\"}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested eval was not found.").await;
    support::assert_no_usage_records(ctx.admin_app, &ctx.admin_token).await;
}

#[tokio::test]
async fn stateful_eval_delete_route_returns_not_found_without_usage() {
    let ctx =
        local_evals_test_context("tenant-eval-delete-missing", "project-eval-delete-missing")
            .await;

    let response = ctx
        .gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/v1/evals/eval_missing")
                .header("authorization", format!("Bearer {}", ctx.api_key))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested eval was not found.").await;
    support::assert_no_usage_records(ctx.admin_app, &ctx.admin_token).await;
}

#[tokio::test]
async fn stateful_eval_runs_list_route_returns_not_found_without_usage() {
    let ctx = local_evals_test_context(
        "tenant-eval-runs-list-missing",
        "project-eval-runs-list-missing",
    )
    .await;

    let response = ctx
        .gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/evals/eval_missing/runs")
                .header("authorization", format!("Bearer {}", ctx.api_key))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested eval was not found.").await;
    support::assert_no_usage_records(ctx.admin_app, &ctx.admin_token).await;
}

#[tokio::test]
async fn stateful_eval_runs_create_route_returns_not_found_without_usage() {
    let ctx = local_evals_test_context(
        "tenant-eval-runs-create-missing",
        "project-eval-runs-create-missing",
    )
    .await;

    let response = ctx
        .gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/evals/eval_missing/runs")
                .header("authorization", format!("Bearer {}", ctx.api_key))
                .header("content-type", "application/json")
                .body(Body::from("{\"name\":\"daily-regression\"}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested eval was not found.").await;
    support::assert_no_usage_records(ctx.admin_app, &ctx.admin_token).await;
}

#[tokio::test]
async fn stateful_eval_run_retrieve_route_returns_not_found_without_usage() {
    let ctx = local_evals_test_context(
        "tenant-eval-run-retrieve-missing",
        "project-eval-run-retrieve-missing",
    )
    .await;

    let response = ctx
        .gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/evals/eval_1/runs/run_missing")
                .header("authorization", format!("Bearer {}", ctx.api_key))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested eval run was not found.").await;
    support::assert_no_usage_records(ctx.admin_app, &ctx.admin_token).await;
}

#[tokio::test]
async fn stateful_eval_run_delete_route_returns_not_found_without_usage() {
    let ctx = local_evals_test_context(
        "tenant-eval-run-delete-missing",
        "project-eval-run-delete-missing",
    )
    .await;

    let response = ctx
        .gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/v1/evals/eval_1/runs/run_missing")
                .header("authorization", format!("Bearer {}", ctx.api_key))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested eval run was not found.").await;
    support::assert_no_usage_records(ctx.admin_app, &ctx.admin_token).await;
}

#[tokio::test]
async fn stateful_eval_run_cancel_route_returns_not_found_without_usage() {
    let ctx = local_evals_test_context(
        "tenant-eval-run-cancel-missing",
        "project-eval-run-cancel-missing",
    )
    .await;

    let response = ctx
        .gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/evals/eval_1/runs/run_missing/cancel")
                .header("authorization", format!("Bearer {}", ctx.api_key))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested eval run was not found.").await;
    support::assert_no_usage_records(ctx.admin_app, &ctx.admin_token).await;
}

#[tokio::test]
async fn stateful_eval_run_output_items_list_route_returns_not_found_without_usage() {
    let ctx = local_evals_test_context(
        "tenant-eval-run-output-items-missing",
        "project-eval-run-output-items-missing",
    )
    .await;

    let response = ctx
        .gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/evals/eval_1/runs/run_missing/output_items")
                .header("authorization", format!("Bearer {}", ctx.api_key))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested eval run was not found.").await;
    support::assert_no_usage_records(ctx.admin_app, &ctx.admin_token).await;
}

#[tokio::test]
async fn stateful_eval_run_output_item_retrieve_route_returns_not_found_without_usage() {
    let ctx = local_evals_test_context(
        "tenant-eval-run-output-item-missing",
        "project-eval-run-output-item-missing",
    )
    .await;

    let response = ctx
        .gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/evals/eval_1/runs/run_1/output_items/output_item_missing")
                .header("authorization", format!("Bearer {}", ctx.api_key))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested eval run output item was not found.").await;
    support::assert_no_usage_records(ctx.admin_app, &ctx.admin_token).await;
}
