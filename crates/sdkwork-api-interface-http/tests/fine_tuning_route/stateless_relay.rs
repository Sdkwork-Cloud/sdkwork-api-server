use super::*;

#[tokio::test]
async fn stateless_fine_tuning_route_relays_to_openai_compatible_provider() {
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

    let app = sdkwork_api_interface_http::gateway_router_with_stateless_config(
        sdkwork_api_interface_http::StatelessGatewayConfig::default().with_upstream(
            sdkwork_api_interface_http::StatelessGatewayUpstream::new(
                "openai",
                format!("http://{address}"),
                "sk-stateless-openai",
            ),
        ),
    );

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/fine_tuning/jobs")
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
        Some("Bearer sk-stateless-openai")
    );

    let list_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/fine_tuning/jobs")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(list_response.status(), StatusCode::OK);
    let list_json = read_json(list_response).await;
    assert_eq!(list_json["data"][0]["id"], "ftjob_1");

    let retrieve_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/fine_tuning/jobs/ftjob_1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(retrieve_response.status(), StatusCode::OK);
    let retrieve_json = read_json(retrieve_response).await;
    assert_eq!(retrieve_json["id"], "ftjob_1");

    let cancel_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/fine_tuning/jobs/ftjob_1/cancel")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(cancel_response.status(), StatusCode::OK);
    let cancel_json = read_json(cancel_response).await;
    assert_eq!(cancel_json["status"], "cancelled");

    let events_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/fine_tuning/jobs/ftjob_1/events")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(events_response.status(), StatusCode::OK);
    let events_json = read_json(events_response).await;
    assert_eq!(events_json["data"][0]["id"], "ftevent_1");

    let checkpoints_response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/fine_tuning/jobs/ftjob_1/checkpoints")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(checkpoints_response.status(), StatusCode::OK);
    let checkpoints_json = read_json(checkpoints_response).await;
    assert_eq!(checkpoints_json["data"][0]["id"], "ftckpt_1");
}

#[tokio::test]
async fn stateless_fine_tuning_extended_routes_relay_to_openai_compatible_provider() {
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

    let app = sdkwork_api_interface_http::gateway_router_with_stateless_config(
        sdkwork_api_interface_http::StatelessGatewayConfig::default().with_upstream(
            sdkwork_api_interface_http::StatelessGatewayUpstream::new(
                "openai",
                format!("http://{address}"),
                "sk-stateless-openai",
            ),
        ),
    );

    let pause_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/fine_tuning/jobs/ftjob_1/pause")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(pause_response.status(), StatusCode::OK);
    let pause_json = read_json(pause_response).await;
    assert_eq!(pause_json["status"], "paused");

    let resume_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/fine_tuning/jobs/ftjob_1/resume")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resume_response.status(), StatusCode::OK);
    let resume_json = read_json(resume_response).await;
    assert_eq!(resume_json["status"], "running");

    let permissions_create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/fine_tuning/checkpoints/ft:gpt-4.1-mini:checkpoint-1/permissions")
                .header("content-type", "application/json")
                .body(Body::from("{\"project_ids\":[\"project-2\"]}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(permissions_create_response.status(), StatusCode::OK);
    let permissions_create_json = read_json(permissions_create_response).await;
    assert_eq!(permissions_create_json["data"][0]["id"], "perm_1");

    let permissions_list_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/fine_tuning/checkpoints/ft:gpt-4.1-mini:checkpoint-1/permissions")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(permissions_list_response.status(), StatusCode::OK);
    let permissions_list_json = read_json(permissions_list_response).await;
    assert_eq!(permissions_list_json["data"][0]["id"], "perm_1");

    let permissions_delete_response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(
                    "/v1/fine_tuning/checkpoints/ft:gpt-4.1-mini:checkpoint-1/permissions/perm_1",
                )
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
        Some("Bearer sk-stateless-openai")
    );
}
