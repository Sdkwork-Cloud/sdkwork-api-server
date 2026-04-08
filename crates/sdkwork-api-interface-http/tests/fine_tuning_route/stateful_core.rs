use super::*;

#[tokio::test]
async fn stateful_fine_tuning_retrieve_route_returns_not_found_without_usage() {
    let ctx = local_fine_tuning_test_context(
        "tenant-fine-tuning-retrieve-missing",
        "project-fine-tuning-retrieve-missing",
    )
    .await;

    let response = ctx
        .gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/fine_tuning/jobs/ftjob_missing")
                .header("authorization", format!("Bearer {}", ctx.api_key))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested fine-tuning job was not found.").await;
    support::assert_no_usage_records(ctx.admin_app, &ctx.admin_token).await;
}

#[tokio::test]
async fn stateful_fine_tuning_cancel_route_returns_not_found_without_usage() {
    let ctx =
        local_fine_tuning_test_context("tenant-fine-tuning-cancel-missing", "project-fine-tuning-cancel-missing")
            .await;

    let response = ctx
        .gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/fine_tuning/jobs/ftjob_missing/cancel")
                .header("authorization", format!("Bearer {}", ctx.api_key))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested fine-tuning job was not found.").await;
    support::assert_no_usage_records(ctx.admin_app, &ctx.admin_token).await;
}

#[tokio::test]
async fn stateful_fine_tuning_events_route_returns_not_found_without_usage() {
    let ctx =
        local_fine_tuning_test_context("tenant-fine-tuning-events-missing", "project-fine-tuning-events-missing")
            .await;

    let response = ctx
        .gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/fine_tuning/jobs/ftjob_missing/events")
                .header("authorization", format!("Bearer {}", ctx.api_key))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested fine-tuning job was not found.").await;
    support::assert_no_usage_records(ctx.admin_app, &ctx.admin_token).await;
}

#[tokio::test]
async fn stateful_fine_tuning_checkpoints_route_returns_not_found_without_usage() {
    let ctx = local_fine_tuning_test_context(
        "tenant-fine-tuning-checkpoints-missing",
        "project-fine-tuning-checkpoints-missing",
    )
    .await;

    let response = ctx
        .gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/fine_tuning/jobs/ftjob_missing/checkpoints")
                .header("authorization", format!("Bearer {}", ctx.api_key))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested fine-tuning job was not found.").await;
    support::assert_no_usage_records(ctx.admin_app, &ctx.admin_token).await;
}

#[tokio::test]
async fn stateful_fine_tuning_pause_route_returns_not_found_without_usage() {
    let ctx =
        local_fine_tuning_test_context("tenant-fine-tuning-pause-missing", "project-fine-tuning-pause-missing")
            .await;

    let response = ctx
        .gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/fine_tuning/jobs/ftjob_missing/pause")
                .header("authorization", format!("Bearer {}", ctx.api_key))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested fine-tuning job was not found.").await;
    support::assert_no_usage_records(ctx.admin_app, &ctx.admin_token).await;
}

#[tokio::test]
async fn stateful_fine_tuning_resume_route_returns_not_found_without_usage() {
    let ctx =
        local_fine_tuning_test_context("tenant-fine-tuning-resume-missing", "project-fine-tuning-resume-missing")
            .await;

    let response = ctx
        .gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/fine_tuning/jobs/ftjob_missing/resume")
                .header("authorization", format!("Bearer {}", ctx.api_key))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested fine-tuning job was not found.").await;
    support::assert_no_usage_records(ctx.admin_app, &ctx.admin_token).await;
}

#[tokio::test]
async fn stateful_fine_tuning_checkpoint_permissions_create_route_returns_not_found_without_usage()
{
    let ctx =
        local_fine_tuning_test_context("tenant-fine-tuning-perms-create-missing", "project-fine-tuning-perms-create-missing")
            .await;

    let response = ctx
        .gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/fine_tuning/checkpoints/ft:missing:checkpoint/permissions")
                .header("authorization", format!("Bearer {}", ctx.api_key))
                .header("content-type", "application/json")
                .body(Body::from("{\"project_ids\":[\"project-2\"]}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested fine-tuning checkpoint was not found.").await;
    support::assert_no_usage_records(ctx.admin_app, &ctx.admin_token).await;
}

#[tokio::test]
async fn stateful_fine_tuning_checkpoint_permissions_list_route_returns_not_found_without_usage() {
    let ctx =
        local_fine_tuning_test_context("tenant-fine-tuning-perms-list-missing", "project-fine-tuning-perms-list-missing")
            .await;

    let response = ctx
        .gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/fine_tuning/checkpoints/ft:missing:checkpoint/permissions")
                .header("authorization", format!("Bearer {}", ctx.api_key))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested fine-tuning checkpoint was not found.").await;
    support::assert_no_usage_records(ctx.admin_app, &ctx.admin_token).await;
}

#[tokio::test]
async fn stateful_fine_tuning_checkpoint_permission_delete_route_returns_not_found_without_usage() {
    let ctx =
        local_fine_tuning_test_context("tenant-fine-tuning-perm-delete-missing", "project-fine-tuning-perm-delete-missing")
            .await;

    let response = ctx
        .gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(
                    "/v1/fine_tuning/checkpoints/ft:gpt-4.1-mini:checkpoint-1/permissions/perm_missing",
                )
                .header("authorization", format!("Bearer {}", ctx.api_key))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(
        response,
        "Requested fine-tuning checkpoint permission was not found.",
    )
    .await;
    support::assert_no_usage_records(ctx.admin_app, &ctx.admin_token).await;
}

#[tokio::test]
async fn stateful_fine_tuning_route_relays_to_openai_compatible_provider() {
    let upstream_state = UpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route(
            "/v1/fine_tuning/jobs",
            get(upstream_fine_tuning_list_handler).post(upstream_fine_tuning_handler),
        )
        .route(
            "/v1/fine_tuning/jobs/ftjob_1",
            get(upstream_fine_tuning_retrieve_handler),
        )
        .route(
            "/v1/fine_tuning/jobs/ftjob_1/cancel",
            post(upstream_fine_tuning_cancel_handler),
        )
        .route(
            "/v1/fine_tuning/jobs/ftjob_1/events",
            get(upstream_fine_tuning_events_handler),
        )
        .route(
            "/v1/fine_tuning/jobs/ftjob_1/checkpoints",
            get(upstream_fine_tuning_checkpoints_handler),
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

    let model = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/models")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"external_name\":\"gpt-4.1-mini\",\"provider_id\":\"provider-openai-official\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(model.status(), StatusCode::CREATED);

    let response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/fine_tuning/jobs")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"training_file\":\"file_1\",\"model\":\"gpt-4.1-mini\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["id"], "ftjob_upstream");
    assert_eq!(
        upstream_state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-upstream-openai")
    );
    support::assert_single_usage_record_and_decision_log(
        admin_app.clone(),
        &admin_token,
        "ftjob_upstream",
        "provider-openai-official",
        "gpt-4.1-mini",
    )
    .await;

    let list_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/fine_tuning/jobs")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(list_response.status(), StatusCode::OK);
    let list_json = read_json(list_response).await;
    assert_eq!(list_json["object"], "list");

    let retrieve_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/fine_tuning/jobs/ftjob_1")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(retrieve_response.status(), StatusCode::OK);
    let retrieve_json = read_json(retrieve_response).await;
    assert_eq!(retrieve_json["id"], "ftjob_1");

    let cancel_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/fine_tuning/jobs/ftjob_1/cancel")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(cancel_response.status(), StatusCode::OK);
    let cancel_json = read_json(cancel_response).await;
    assert_eq!(cancel_json["status"], "cancelled");

    let events_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/fine_tuning/jobs/ftjob_1/events")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(events_response.status(), StatusCode::OK);
    let events_json = read_json(events_response).await;
    assert_eq!(events_json["data"][0]["id"], "ftevent_1");

    let checkpoints_response = gateway_app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/fine_tuning/jobs/ftjob_1/checkpoints")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(checkpoints_response.status(), StatusCode::OK);
    let checkpoints_json = read_json(checkpoints_response).await;
    assert_eq!(checkpoints_json["data"][0]["id"], "ftckpt_1");
}
