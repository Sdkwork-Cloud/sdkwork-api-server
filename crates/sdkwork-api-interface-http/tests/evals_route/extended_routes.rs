use super::*;

#[tokio::test]
async fn stateless_eval_run_extended_routes_relay_to_openai_compatible_provider() {
    let upstream_state = UpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route(
            "/v1/evals/eval_1/runs/run_1",
            get(upstream_eval_run_retrieve_handler).delete(upstream_eval_run_delete_handler),
        )
        .route(
            "/v1/evals/eval_1/runs/run_1/cancel",
            post(upstream_eval_run_cancel_handler),
        )
        .route(
            "/v1/evals/eval_1/runs/run_1/output_items",
            get(upstream_eval_run_output_items_list_handler),
        )
        .route(
            "/v1/evals/eval_1/runs/run_1/output_items/output_item_1",
            get(upstream_eval_run_output_item_retrieve_handler),
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

    let delete_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/v1/evals/eval_1/runs/run_1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(delete_response.status(), StatusCode::OK);
    let delete_json = read_json(delete_response).await;
    assert_eq!(delete_json["deleted"], true);

    let cancel_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/evals/eval_1/runs/run_1/cancel")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(cancel_response.status(), StatusCode::OK);
    let cancel_json = read_json(cancel_response).await;
    assert_eq!(cancel_json["status"], "cancelled");

    let output_items_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/evals/eval_1/runs/run_1/output_items")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(output_items_response.status(), StatusCode::OK);
    let output_items_json = read_json(output_items_response).await;
    assert_eq!(output_items_json["data"][0]["id"], "output_item_1");

    let output_item_response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/evals/eval_1/runs/run_1/output_items/output_item_1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(output_item_response.status(), StatusCode::OK);
    let output_item_json = read_json(output_item_response).await;
    assert_eq!(output_item_json["id"], "output_item_1");
    assert_eq!(
        upstream_state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-stateless-openai")
    );
}

#[tokio::test]
async fn stateless_eval_run_route_returns_openai_error_envelope_on_upstream_failure() {
    let app = sdkwork_api_interface_http::gateway_router_with_stateless_config(
        sdkwork_api_interface_http::StatelessGatewayConfig::default().with_upstream(
            sdkwork_api_interface_http::StatelessGatewayUpstream::new(
                "openai",
                "http://127.0.0.1:1",
                "sk-stateless-openai",
            ),
        ),
    );

    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/v1/evals/eval_1/runs/run_1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
    let json = read_json(response).await;
    assert_eq!(
        json["error"]["message"],
        "failed to relay upstream eval run delete"
    );
    assert_eq!(json["error"]["type"], "server_error");
    assert_eq!(json["error"]["code"], "bad_gateway");
}

#[tokio::test]
async fn stateful_eval_run_extended_routes_relay_to_openai_compatible_provider() {
    let upstream_state = UpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route(
            "/v1/evals/eval_1/runs/run_1",
            get(upstream_eval_run_retrieve_handler).delete(upstream_eval_run_delete_handler),
        )
        .route(
            "/v1/evals/eval_1/runs/run_1/cancel",
            post(upstream_eval_run_cancel_handler),
        )
        .route(
            "/v1/evals/eval_1/runs/run_1/output_items",
            get(upstream_eval_run_output_items_list_handler),
        )
        .route(
            "/v1/evals/eval_1/runs/run_1/output_items/output_item_1",
            get(upstream_eval_run_output_item_retrieve_handler),
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

    let delete_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/v1/evals/eval_1/runs/run_1")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(delete_response.status(), StatusCode::OK);
    let delete_json = read_json(delete_response).await;
    assert_eq!(delete_json["deleted"], true);

    let cancel_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/evals/eval_1/runs/run_1/cancel")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(cancel_response.status(), StatusCode::OK);
    let cancel_json = read_json(cancel_response).await;
    assert_eq!(cancel_json["status"], "cancelled");

    let output_items_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/evals/eval_1/runs/run_1/output_items")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(output_items_response.status(), StatusCode::OK);
    let output_items_json = read_json(output_items_response).await;
    assert_eq!(output_items_json["data"][0]["id"], "output_item_1");

    let output_item_response = gateway_app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/evals/eval_1/runs/run_1/output_items/output_item_1")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(output_item_response.status(), StatusCode::OK);
    let output_item_json = read_json(output_item_response).await;
    assert_eq!(output_item_json["id"], "output_item_1");
    assert_eq!(
        upstream_state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-upstream-openai")
    );
}
