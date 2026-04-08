use super::*;

#[tokio::test]
async fn stateful_fine_tuning_job_create_usage_uses_created_job_id_for_billing() {
    let tenant_id = "tenant-fine-tuning-create";
    let project_id = "project-fine-tuning-create";
    let upstream_state = UpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route("/v1/fine_tuning/jobs", post(upstream_fine_tuning_handler))
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

    let provider_job = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"id\":\"provider-fine-tuning-route\",\"channel_id\":\"openai\",\"adapter_kind\":\"openai\",\"base_url\":\"http://{address}\",\"display_name\":\"Fine Tuning Route Provider\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(provider_job.status(), StatusCode::CREATED);

    let provider_created = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"id\":\"provider-fine-tuning-created-id\",\"channel_id\":\"openai\",\"adapter_kind\":\"openai\",\"base_url\":\"http://127.0.0.1:1\",\"display_name\":\"Fine Tuning Created Id Provider\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(provider_created.status(), StatusCode::CREATED);

    let job_credential = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/credentials")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"tenant_id\":\"{tenant_id}\",\"provider_id\":\"provider-fine-tuning-route\",\"key_reference\":\"cred-fine-tuning-route\",\"secret_value\":\"sk-fine-tuning-route\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(job_credential.status(), StatusCode::CREATED);

    let created_credential = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/credentials")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"tenant_id\":\"{tenant_id}\",\"provider_id\":\"provider-fine-tuning-created-id\",\"key_reference\":\"cred-fine-tuning-created-id\",\"secret_value\":\"sk-fine-tuning-created-id\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(created_credential.status(), StatusCode::CREATED);

    let model = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/models")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"external_name\":\"gpt-4.1-mini\",\"provider_id\":\"provider-fine-tuning-route\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(model.status(), StatusCode::CREATED);

    let job_policy = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/routing/policies")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "policy_id": "route-fine-tuning-by-model",
                        "capability": "fine_tuning",
                        "model_pattern": "gpt-4.1-mini",
                        "enabled": true,
                        "priority": 200,
                        "ordered_provider_ids": ["provider-fine-tuning-route"]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(job_policy.status(), StatusCode::CREATED);

    let created_policy = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/routing/policies")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "policy_id": "route-fine-tuning-by-created-id",
                        "capability": "fine_tuning",
                        "model_pattern": "ftjob_upstream",
                        "enabled": true,
                        "priority": 100,
                        "ordered_provider_ids": ["provider-fine-tuning-created-id"]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(created_policy.status(), StatusCode::CREATED);

    let response = gateway_app
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
    let response_json = read_json(response).await;
    assert_eq!(response_json["id"], "ftjob_upstream");
    assert_eq!(
        upstream_state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-fine-tuning-route")
    );
    support::assert_single_usage_record_and_decision_log(
        admin_app,
        &admin_token,
        "ftjob_upstream",
        "provider-fine-tuning-route",
        "gpt-4.1-mini",
    )
    .await;
}

#[tokio::test]
async fn stateful_fine_tuning_checkpoint_permission_usage_uses_checkpoint_route_key_for_provider_selection()
{
    let tenant_id = "tenant-fine-tuning-permission-usage";
    let project_id = "project-fine-tuning-permission-usage";
    let checkpoint_id = "ft:gpt-4.1-mini:checkpoint-1";
    let upstream_state = UpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route(
            "/v1/fine_tuning/checkpoints/ft:gpt-4.1-mini:checkpoint-1/permissions/perm_1",
            axum::routing::delete(upstream_fine_tuning_checkpoint_permission_delete_handler),
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

    let provider_checkpoint = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"id\":\"provider-checkpoint\",\"channel_id\":\"openai\",\"adapter_kind\":\"openai\",\"base_url\":\"http://{address}\",\"display_name\":\"Checkpoint Provider\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(provider_checkpoint.status(), StatusCode::CREATED);

    let provider_permission = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"id\":\"provider-permission\",\"channel_id\":\"openai\",\"adapter_kind\":\"openai\",\"base_url\":\"http://127.0.0.1:1\",\"display_name\":\"Permission Provider\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(provider_permission.status(), StatusCode::CREATED);

    let checkpoint_credential = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/credentials")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"tenant_id\":\"{tenant_id}\",\"provider_id\":\"provider-checkpoint\",\"key_reference\":\"cred-checkpoint\",\"secret_value\":\"sk-checkpoint\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(checkpoint_credential.status(), StatusCode::CREATED);

    let permission_credential = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/credentials")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"tenant_id\":\"{tenant_id}\",\"provider_id\":\"provider-permission\",\"key_reference\":\"cred-permission\",\"secret_value\":\"sk-permission\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(permission_credential.status(), StatusCode::CREATED);

    let checkpoint_policy = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/routing/policies")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "policy_id": "route-fine-tuning-permissions-by-checkpoint",
                        "capability": "fine_tuning",
                        "model_pattern": checkpoint_id,
                        "enabled": true,
                        "priority": 200,
                        "ordered_provider_ids": ["provider-checkpoint"]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(checkpoint_policy.status(), StatusCode::CREATED);

    let permission_policy = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/routing/policies")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "policy_id": "route-fine-tuning-permissions-by-permission",
                        "capability": "fine_tuning",
                        "model_pattern": "perm_1",
                        "enabled": true,
                        "priority": 100,
                        "ordered_provider_ids": ["provider-permission"]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(permission_policy.status(), StatusCode::CREATED);

    let response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/v1/fine_tuning/checkpoints/ft:gpt-4.1-mini:checkpoint-1/permissions/perm_1")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let response_json = read_json(response).await;
    assert_eq!(response_json["deleted"], true);
    assert_eq!(
        upstream_state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-checkpoint")
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
    assert_eq!(usage_json[0]["model"], "perm_1");
    assert_eq!(usage_json[0]["provider"], "provider-checkpoint");

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
    assert_eq!(logs_json[0]["route_key"], checkpoint_id);
    assert_eq!(logs_json[0]["selected_provider_id"], "provider-checkpoint");
}

#[tokio::test]
async fn stateful_fine_tuning_checkpoint_permissions_create_usage_uses_created_permission_id_for_billing()
{
    let tenant_id = "tenant-fine-tuning-permission-create";
    let project_id = "project-fine-tuning-permission-create";
    let checkpoint_id = "ft:gpt-4.1-mini:checkpoint-1";
    let upstream_state = UpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route(
            "/v1/fine_tuning/checkpoints/ft:gpt-4.1-mini:checkpoint-1/permissions",
            post(upstream_fine_tuning_checkpoint_permissions_create_handler),
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

    let provider_checkpoint = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"id\":\"provider-checkpoint-create-route\",\"channel_id\":\"openai\",\"adapter_kind\":\"openai\",\"base_url\":\"http://{address}\",\"display_name\":\"Checkpoint Create Route Provider\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(provider_checkpoint.status(), StatusCode::CREATED);

    let provider_permission = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"id\":\"provider-checkpoint-create-child\",\"channel_id\":\"openai\",\"adapter_kind\":\"openai\",\"base_url\":\"http://127.0.0.1:1\",\"display_name\":\"Checkpoint Create Child Provider\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(provider_permission.status(), StatusCode::CREATED);

    let checkpoint_credential = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/credentials")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"tenant_id\":\"{tenant_id}\",\"provider_id\":\"provider-checkpoint-create-route\",\"key_reference\":\"cred-checkpoint-create-route\",\"secret_value\":\"sk-checkpoint-create-route\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(checkpoint_credential.status(), StatusCode::CREATED);

    let permission_credential = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/credentials")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"tenant_id\":\"{tenant_id}\",\"provider_id\":\"provider-checkpoint-create-child\",\"key_reference\":\"cred-checkpoint-create-child\",\"secret_value\":\"sk-checkpoint-create-child\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(permission_credential.status(), StatusCode::CREATED);

    let checkpoint_policy = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/routing/policies")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "policy_id": "route-fine-tuning-permissions-by-checkpoint",
                        "capability": "fine_tuning",
                        "model_pattern": checkpoint_id,
                        "enabled": true,
                        "priority": 200,
                        "ordered_provider_ids": ["provider-checkpoint-create-route"]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(checkpoint_policy.status(), StatusCode::CREATED);

    let permission_policy = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/routing/policies")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "policy_id": "route-fine-tuning-permissions-by-permission-id",
                        "capability": "fine_tuning",
                        "model_pattern": "perm_1",
                        "enabled": true,
                        "priority": 100,
                        "ordered_provider_ids": ["provider-checkpoint-create-child"]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(permission_policy.status(), StatusCode::CREATED);

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/fine_tuning/checkpoints/ft:gpt-4.1-mini:checkpoint-1/permissions")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from("{\"project_ids\":[\"project-2\"]}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let response_json = read_json(response).await;
    assert_eq!(response_json["data"][0]["id"], "perm_1");
    assert_eq!(
        upstream_state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-checkpoint-create-route")
    );
    support::assert_single_usage_record_and_decision_log(
        admin_app,
        &admin_token,
        "perm_1",
        "provider-checkpoint-create-route",
        checkpoint_id,
    )
    .await;
}
