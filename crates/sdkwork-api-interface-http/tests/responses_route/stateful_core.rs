use super::*;

#[serial(extension_env)]
#[tokio::test]
async fn stateful_responses_route_relays_to_openai_compatible_provider() {
    let upstream_state = UpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route("/v1/responses", post(upstream_responses_handler))
        .route(
            "/v1/responses/input_tokens",
            post(upstream_response_input_tokens_handler),
        )
        .route(
            "/v1/responses/compact",
            post(upstream_response_compact_handler),
        )
        .route(
            "/v1/responses/resp_1",
            get(upstream_response_retrieve_handler).delete(upstream_response_delete_handler),
        )
        .route(
            "/v1/responses/resp_1/input_items",
            get(upstream_response_input_items_handler),
        )
        .route(
            "/v1/responses/resp_1/cancel",
            post(upstream_response_cancel_handler),
        )
        .with_state(upstream_state.clone());

    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });

    let pool = memory_pool().await;
    let api_key = support::issue_gateway_api_key(&pool, "tenant-1", "project-1").await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
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
                    "{\"external_name\":\"gpt-4.1\",\"provider_id\":\"provider-openai-official\"}",
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
                .uri("/v1/responses")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from("{\"model\":\"gpt-4.1\",\"input\":\"relay me\"}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["id"], "resp_upstream");
    assert_eq!(
        upstream_state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-upstream-openai")
    );

    let retrieve_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/responses/resp_1")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(retrieve_response.status(), StatusCode::OK);
    let retrieve_json = read_json(retrieve_response).await;
    assert_eq!(retrieve_json["id"], "resp_1");

    let input_items_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/responses/resp_1/input_items")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(input_items_response.status(), StatusCode::OK);
    let input_items_json = read_json(input_items_response).await;
    assert_eq!(input_items_json["data"][0]["id"], "item_1");

    let delete_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/v1/responses/resp_1")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(delete_response.status(), StatusCode::OK);
    let delete_json = read_json(delete_response).await;
    assert_eq!(delete_json["deleted"], true);

    let input_tokens_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/responses/input_tokens")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from("{\"model\":\"gpt-4.1\",\"input\":\"count me\"}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(input_tokens_response.status(), StatusCode::OK);
    let input_tokens_json = read_json(input_tokens_response).await;
    assert_eq!(input_tokens_json["object"], "response.input_tokens");
    assert_eq!(input_tokens_json["input_tokens"], 21);

    let cancel_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/responses/resp_1/cancel")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(cancel_response.status(), StatusCode::OK);
    let cancel_json = read_json(cancel_response).await;
    assert_eq!(cancel_json["status"], "cancelled");

    let compact_response = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/responses/compact")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"gpt-4.1\",\"input\":\"compact me\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(compact_response.status(), StatusCode::OK);
    let compact_json = read_json(compact_response).await;
    assert_eq!(compact_json["object"], "response.compaction");
}

#[serial(extension_env)]
#[tokio::test]
async fn stateful_responses_route_returns_invalid_request_for_missing_model_without_usage() {
    let pool = memory_pool().await;
    let tenant_id = "tenant-responses-invalid-model";
    let project_id = "project-responses-invalid-model";
    let api_key = support::issue_gateway_api_key(&pool, tenant_id, project_id).await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool);

    let response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/responses")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from("{\"model\":\"\",\"input\":\"hi\"}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let json = read_json(response).await;
    assert_eq!(json["error"]["message"], "Response model is required.");
    assert_eq!(json["error"]["type"], "invalid_request_error");
    assert_eq!(json["error"]["code"], "invalid_model");

    support::assert_no_usage_records(admin_app, &admin_token).await;
}

#[serial(extension_env)]
#[tokio::test]
async fn stateful_responses_stream_route_returns_invalid_request_for_missing_model_without_usage() {
    let pool = memory_pool().await;
    let tenant_id = "tenant-responses-stream-invalid-model";
    let project_id = "project-responses-stream-invalid-model";
    let api_key = support::issue_gateway_api_key(&pool, tenant_id, project_id).await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool);

    let response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/responses")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"\",\"input\":\"hi\",\"stream\":true}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let json = read_json(response).await;
    assert_eq!(json["error"]["message"], "Response model is required.");
    assert_eq!(json["error"]["type"], "invalid_request_error");
    assert_eq!(json["error"]["code"], "invalid_model");

    support::assert_no_usage_records(admin_app, &admin_token).await;
}

#[serial(extension_env)]
#[tokio::test]
async fn stateful_response_input_tokens_route_returns_invalid_request_for_missing_model_without_usage(
) {
    let pool = memory_pool().await;
    let tenant_id = "tenant-response-input-tokens-invalid-model";
    let project_id = "project-response-input-tokens-invalid-model";
    let api_key = support::issue_gateway_api_key(&pool, tenant_id, project_id).await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool);

    let response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/responses/input_tokens")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from("{\"model\":\"\",\"input\":\"hi\"}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let json = read_json(response).await;
    assert_eq!(json["error"]["message"], "Response model is required.");
    assert_eq!(json["error"]["type"], "invalid_request_error");
    assert_eq!(json["error"]["code"], "invalid_model");

    support::assert_no_usage_records(admin_app, &admin_token).await;
}
