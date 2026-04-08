use super::*;

#[serial(extension_env)]
#[tokio::test]
async fn responses_route_returns_ok() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/responses")
                .header("content-type", "application/json")
                .body(Body::from("{\"model\":\"gpt-4.1\",\"input\":\"hi\"}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[serial(extension_env)]
#[tokio::test]
async fn responses_route_returns_invalid_request_for_missing_model() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/responses")
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
}

#[serial(extension_env)]
#[tokio::test]
async fn responses_stream_route_returns_invalid_request_for_missing_model() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/responses")
                .header("content-type", "application/json")
                .body(Body::from("{\"model\":\"\",\"input\":\"hi\",\"stream\":true}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let json = read_json(response).await;
    assert_eq!(json["error"]["message"], "Response model is required.");
    assert_eq!(json["error"]["type"], "invalid_request_error");
    assert_eq!(json["error"]["code"], "invalid_model");
}

#[serial(extension_env)]
#[tokio::test]
async fn response_retrieve_route_returns_ok() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/responses/resp_1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[serial(extension_env)]
#[tokio::test]
async fn response_retrieve_route_returns_not_found_for_unknown_response() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/responses/resp_missing")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let json = read_json(response).await;
    assert_eq!(json["error"]["message"], "Requested response was not found.");
    assert_eq!(json["error"]["type"], "invalid_request_error");
    assert_eq!(json["error"]["code"], "not_found");
}

#[serial(extension_env)]
#[tokio::test]
async fn response_input_items_route_returns_ok() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/responses/resp_1/input_items")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[serial(extension_env)]
#[tokio::test]
async fn response_input_items_route_returns_not_found_for_unknown_response() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/responses/resp_missing/input_items")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let json = read_json(response).await;
    assert_eq!(json["error"]["message"], "Requested response was not found.");
    assert_eq!(json["error"]["type"], "invalid_request_error");
    assert_eq!(json["error"]["code"], "not_found");
}

#[serial(extension_env)]
#[tokio::test]
async fn response_delete_route_returns_ok() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/v1/responses/resp_1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[serial(extension_env)]
#[tokio::test]
async fn response_delete_route_returns_not_found_for_unknown_response() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/v1/responses/resp_missing")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let json = read_json(response).await;
    assert_eq!(json["error"]["message"], "Requested response was not found.");
    assert_eq!(json["error"]["type"], "invalid_request_error");
    assert_eq!(json["error"]["code"], "not_found");
}

#[serial(extension_env)]
#[tokio::test]
async fn response_input_tokens_route_returns_ok() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/responses/input_tokens")
                .header("content-type", "application/json")
                .body(Body::from("{\"model\":\"gpt-4.1\",\"input\":\"hi\"}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[serial(extension_env)]
#[tokio::test]
async fn response_input_tokens_route_returns_invalid_request_for_missing_model() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/responses/input_tokens")
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
}

#[serial(extension_env)]
#[tokio::test]
async fn response_cancel_route_returns_not_found_for_unknown_response() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/responses/resp_missing/cancel")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let json = read_json(response).await;
    assert_eq!(json["error"]["message"], "Requested response was not found.");
    assert_eq!(json["error"]["type"], "invalid_request_error");
    assert_eq!(json["error"]["code"], "not_found");
}

#[serial(extension_env)]
#[tokio::test]
async fn response_cancel_route_returns_ok() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/responses/resp_1/cancel")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[serial(extension_env)]
#[tokio::test]
async fn response_compact_route_returns_ok() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/responses/compact")
                .header("content-type", "application/json")
                .body(Body::from("{\"model\":\"gpt-4.1\",\"input\":\"hi\"}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[serial(extension_env)]
#[tokio::test]
async fn response_compact_route_returns_invalid_request_for_missing_model() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/responses/compact")
                .header("content-type", "application/json")
                .body(Body::from("{\"model\":\"\",\"input\":\"hi\"}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let json = read_json(response).await;
    assert_eq!(json["error"]["message"], "Response compaction model is required.");
    assert_eq!(json["error"]["type"], "invalid_request_error");
    assert_eq!(json["error"]["code"], "invalid_model");
}

#[serial(extension_env)]
#[tokio::test]
async fn stateless_responses_route_relays_to_openai_compatible_provider() {
    let upstream_state = UpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route("/v1/responses", post(upstream_responses_handler))
        .with_state(upstream_state.clone());

    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });

    let app = sdkwork_api_interface_http::gateway_router_with_stateless_config(
        sdkwork_api_interface_http::StatelessGatewayConfig::default().with_upstream(
            sdkwork_api_interface_http::StatelessGatewayUpstream::from_adapter_kind(
                "openai",
                format!("http://{address}"),
                "sk-stateless-openai",
            ),
        ),
    );

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/responses")
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
        Some("Bearer sk-stateless-openai")
    );
}

#[serial(extension_env)]
#[tokio::test]
async fn stateless_responses_route_relays_stream_to_openai_compatible_provider() {
    let upstream_state = UpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route("/v1/responses", post(upstream_responses_stream_handler))
        .with_state(upstream_state.clone());

    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });

    let app = sdkwork_api_interface_http::gateway_router_with_stateless_config(
        sdkwork_api_interface_http::StatelessGatewayConfig::default().with_upstream(
            sdkwork_api_interface_http::StatelessGatewayUpstream::from_adapter_kind(
                "openai",
                format!("http://{address}"),
                "sk-stateless-openai",
            ),
        ),
    );

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/responses")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"gpt-4.1\",\"input\":\"hi\",\"stream\":true}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response
            .headers()
            .get("content-type")
            .and_then(|value| value.to_str().ok()),
        Some("text/event-stream")
    );

    let body = read_text(response).await;
    assert!(body.contains("resp_upstream_stream"));
    assert!(body.contains("[DONE]"));
    assert_eq!(
        upstream_state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-stateless-openai")
    );
}

#[serial(extension_env)]
#[tokio::test]
async fn stateless_responses_route_returns_openai_error_envelope_on_upstream_failure() {
    let app = sdkwork_api_interface_http::gateway_router_with_stateless_config(
        sdkwork_api_interface_http::StatelessGatewayConfig::default().with_upstream(
            sdkwork_api_interface_http::StatelessGatewayUpstream::from_adapter_kind(
                "openai",
                "http://127.0.0.1:1",
                "sk-stateless-openai",
            ),
        ),
    );

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/responses")
                .header("content-type", "application/json")
                .body(Body::from("{\"model\":\"gpt-4.1\",\"input\":\"relay me\"}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
    let json = read_json(response).await;
    assert_eq!(
        json["error"]["message"],
        "failed to relay upstream response"
    );
    assert_eq!(json["error"]["type"], "server_error");
    assert_eq!(json["error"]["code"], "bad_gateway");
}

#[serial(extension_env)]
#[tokio::test]
async fn stateless_responses_stream_route_returns_openai_error_envelope_on_upstream_failure() {
    let app = sdkwork_api_interface_http::gateway_router_with_stateless_config(
        sdkwork_api_interface_http::StatelessGatewayConfig::default().with_upstream(
            sdkwork_api_interface_http::StatelessGatewayUpstream::from_adapter_kind(
                "openai",
                "http://127.0.0.1:1",
                "sk-stateless-openai",
            ),
        ),
    );

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/responses")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"gpt-4.1\",\"input\":\"relay me\",\"stream\":true}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
    let json = read_json(response).await;
    assert_eq!(
        json["error"]["message"],
        "failed to relay upstream response stream"
    );
    assert_eq!(json["error"]["type"], "server_error");
    assert_eq!(json["error"]["code"], "bad_gateway");
}
