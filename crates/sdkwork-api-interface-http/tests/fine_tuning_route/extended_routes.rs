use super::*;

#[tokio::test]
async fn stateful_fine_tuning_extended_routes_relay_to_openai_compatible_provider() {
    let upstream_state = UpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route(
            "/v1/fine_tuning/jobs/ftjob_1/pause",
            post(upstream_fine_tuning_pause_handler),
        )
        .route(
            "/v1/fine_tuning/jobs/ftjob_1/resume",
            post(upstream_fine_tuning_resume_handler),
        )
        .route(
            "/v1/fine_tuning/checkpoints/ft:gpt-4.1-mini:checkpoint-1/permissions",
            get(upstream_fine_tuning_checkpoint_permissions_list_handler)
                .post(upstream_fine_tuning_checkpoint_permissions_create_handler),
        )
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

    let pause_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/fine_tuning/jobs/ftjob_1/pause")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(pause_response.status(), StatusCode::OK);
    let pause_json = read_json(pause_response).await;
    assert_eq!(pause_json["status"], "paused");

    let resume_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/fine_tuning/jobs/ftjob_1/resume")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resume_response.status(), StatusCode::OK);
    let resume_json = read_json(resume_response).await;
    assert_eq!(resume_json["status"], "running");

    let permissions_create_response = gateway_app
        .clone()
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
    assert_eq!(permissions_create_response.status(), StatusCode::OK);
    let permissions_create_json = read_json(permissions_create_response).await;
    assert_eq!(permissions_create_json["data"][0]["id"], "perm_1");

    let permissions_list_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/fine_tuning/checkpoints/ft:gpt-4.1-mini:checkpoint-1/permissions")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(permissions_list_response.status(), StatusCode::OK);
    let permissions_list_json = read_json(permissions_list_response).await;
    assert_eq!(permissions_list_json["data"][0]["id"], "perm_1");

    let permissions_delete_response = gateway_app
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
    assert_eq!(permissions_delete_response.status(), StatusCode::OK);
    let permissions_delete_json = read_json(permissions_delete_response).await;
    assert_eq!(permissions_delete_json["deleted"], true);
    assert_eq!(
        upstream_state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-upstream-openai")
    );
}
