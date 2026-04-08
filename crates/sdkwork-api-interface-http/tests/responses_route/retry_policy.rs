use super::*;

#[serial(extension_env)]
#[tokio::test]
async fn stateful_responses_route_retries_retryable_primary_failure_before_failing_over() {
    let tenant_id = "tenant-responses-retryable-primary";
    let project_id = "project-responses-retryable-primary";
    let primary_state = UpstreamCaptureState::default();

    let primary_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let primary_address = primary_listener.local_addr().unwrap();
    let primary_upstream = Router::new()
        .route(
            "/v1/responses",
            post(upstream_responses_handler_retryable_once_then_success),
        )
        .with_state(primary_state.clone());
    tokio::spawn(async move {
        axum::serve(primary_listener, primary_upstream)
            .await
            .unwrap();
    });

    let (gateway_app, _admin_app, _admin_token, api_key) =
        setup_stateful_responses_route_with_single_provider(
            tenant_id,
            project_id,
            "provider-responses-retryable-primary",
            &format!("http://{primary_address}"),
            "Retryable Responses Primary",
            "cred-responses-retryable-primary",
            "sk-responses-retryable-primary",
            "route-responses-retryable-primary",
        )
        .await;

    let response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/responses")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"gpt-4.1\",\"input\":\"retry transient responses primary failure\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["id"], "resp_retry_recovered");
    assert_eq!(primary_state.request_count.load(Ordering::SeqCst), 2);

    let metrics = gateway_app
        .oneshot(
            Request::builder()
                .uri("/metrics")
                .header("authorization", "Bearer local-dev-metrics-token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(metrics.status(), StatusCode::OK);
    let metrics_body = axum::body::to_bytes(metrics.into_body(), usize::MAX)
        .await
        .unwrap();
    let metrics_text = String::from_utf8(metrics_body.to_vec()).unwrap();
    assert!(metrics_text.contains(
        "sdkwork_upstream_requests_total{service=\"gateway\",capability=\"responses\",provider=\"provider-responses-retryable-primary\",outcome=\"attempt\"} 2"
    ));
    assert!(metrics_text.contains(
        "sdkwork_upstream_requests_total{service=\"gateway\",capability=\"responses\",provider=\"provider-responses-retryable-primary\",outcome=\"success\"} 1"
    ));
    assert!(!metrics_text.contains(
        "sdkwork_upstream_requests_total{service=\"gateway\",capability=\"responses\",provider=\"provider-responses-retryable-primary\",outcome=\"failure\"}"
    ));
    assert!(metrics_text.contains(
        "sdkwork_upstream_retries_total{service=\"gateway\",capability=\"responses\",provider=\"provider-responses-retryable-primary\",outcome=\"scheduled\"} 1"
    ));
    assert!(metrics_text.contains(
        "sdkwork_upstream_retry_reasons_total{service=\"gateway\",capability=\"responses\",provider=\"provider-responses-retryable-primary\",outcome=\"scheduled\",reason=\"status_429\"} 1"
    ));
    assert!(metrics_text.contains(
        "sdkwork_upstream_retry_delay_ms_total{service=\"gateway\",capability=\"responses\",provider=\"provider-responses-retryable-primary\",source=\"backoff\"} 25"
    ));
    assert!(metrics_text.contains(
        "sdkwork_provider_health_status{service=\"gateway\",provider=\"provider-responses-retryable-primary\",runtime=\"builtin\"} 1"
    ));
}

#[serial(extension_env)]
#[tokio::test]
async fn stateful_responses_route_does_not_retry_non_retryable_primary_failure() {
    let tenant_id = "tenant-responses-non-retryable-primary";
    let project_id = "project-responses-non-retryable-primary";
    let primary_state = UpstreamCaptureState::default();

    let primary_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let primary_address = primary_listener.local_addr().unwrap();
    let primary_upstream = Router::new()
        .route(
            "/v1/responses",
            post(upstream_responses_handler_non_retryable_once_then_success),
        )
        .with_state(primary_state.clone());
    tokio::spawn(async move {
        axum::serve(primary_listener, primary_upstream)
            .await
            .unwrap();
    });

    let (gateway_app, _admin_app, _admin_token, api_key) =
        setup_stateful_responses_route_with_single_provider(
            tenant_id,
            project_id,
            "provider-responses-non-retryable-primary",
            &format!("http://{primary_address}"),
            "Non Retryable Responses Primary",
            "cred-responses-non-retryable-primary",
            "sk-responses-non-retryable-primary",
            "route-responses-non-retryable-primary",
        )
        .await;

    let response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/responses")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"gpt-4.1\",\"input\":\"do not retry invalid responses request\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
    assert_eq!(primary_state.request_count.load(Ordering::SeqCst), 1);

    let metrics = gateway_app
        .oneshot(
            Request::builder()
                .uri("/metrics")
                .header("authorization", "Bearer local-dev-metrics-token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(metrics.status(), StatusCode::OK);
    let metrics_body = axum::body::to_bytes(metrics.into_body(), usize::MAX)
        .await
        .unwrap();
    let metrics_text = String::from_utf8(metrics_body.to_vec()).unwrap();
    assert!(metrics_text.contains(
        "sdkwork_upstream_requests_total{service=\"gateway\",capability=\"responses\",provider=\"provider-responses-non-retryable-primary\",outcome=\"attempt\"} 1"
    ));
    assert!(metrics_text.contains(
        "sdkwork_upstream_requests_total{service=\"gateway\",capability=\"responses\",provider=\"provider-responses-non-retryable-primary\",outcome=\"failure\"} 1"
    ));
    assert!(!metrics_text.contains(
        "sdkwork_upstream_retries_total{service=\"gateway\",capability=\"responses\",provider=\"provider-responses-non-retryable-primary\",outcome=\"scheduled\"}"
    ));
}

#[serial(extension_env)]
#[tokio::test]
async fn stateful_responses_stream_route_retries_retryable_primary_failure_before_failing_over() {
    let tenant_id = "tenant-responses-stream-retryable-primary";
    let project_id = "project-responses-stream-retryable-primary";
    let primary_state = UpstreamCaptureState::default();

    let primary_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let primary_address = primary_listener.local_addr().unwrap();
    let primary_upstream = Router::new()
        .route(
            "/v1/responses",
            post(upstream_responses_stream_handler_retryable_once_then_success),
        )
        .with_state(primary_state.clone());
    tokio::spawn(async move {
        axum::serve(primary_listener, primary_upstream)
            .await
            .unwrap();
    });

    let (gateway_app, _admin_app, _admin_token, api_key) =
        setup_stateful_responses_route_with_single_provider(
            tenant_id,
            project_id,
            "provider-responses-stream-retryable-primary",
            &format!("http://{primary_address}"),
            "Retryable Responses Stream Primary",
            "cred-responses-stream-retryable-primary",
            "sk-responses-stream-retryable-primary",
            "route-responses-stream-retryable-primary",
        )
        .await;

    let response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/responses")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"gpt-4.1\",\"input\":\"retry transient responses stream primary failure\",\"stream\":true}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let stream_body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let stream_text = String::from_utf8(stream_body.to_vec()).unwrap();
    assert!(stream_text.contains("resp_stream_retry_recovered"));
    assert_eq!(primary_state.request_count.load(Ordering::SeqCst), 2);

    let metrics = gateway_app
        .oneshot(
            Request::builder()
                .uri("/metrics")
                .header("authorization", "Bearer local-dev-metrics-token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(metrics.status(), StatusCode::OK);
    let metrics_body = axum::body::to_bytes(metrics.into_body(), usize::MAX)
        .await
        .unwrap();
    let metrics_text = String::from_utf8(metrics_body.to_vec()).unwrap();
    assert!(metrics_text.contains(
        "sdkwork_upstream_requests_total{service=\"gateway\",capability=\"responses\",provider=\"provider-responses-stream-retryable-primary\",outcome=\"attempt\"} 2"
    ));
    assert!(metrics_text.contains(
        "sdkwork_upstream_requests_total{service=\"gateway\",capability=\"responses\",provider=\"provider-responses-stream-retryable-primary\",outcome=\"success\"} 1"
    ));
    assert!(!metrics_text.contains(
        "sdkwork_upstream_requests_total{service=\"gateway\",capability=\"responses\",provider=\"provider-responses-stream-retryable-primary\",outcome=\"failure\"}"
    ));
    assert!(metrics_text.contains(
        "sdkwork_upstream_retries_total{service=\"gateway\",capability=\"responses\",provider=\"provider-responses-stream-retryable-primary\",outcome=\"scheduled\"} 1"
    ));
}

#[serial(extension_env)]
#[tokio::test]
async fn stateful_responses_route_honors_retry_after_before_retrying_primary_provider() {
    let tenant_id = "tenant-responses-retry-after-primary";
    let project_id = "project-responses-retry-after-primary";
    let primary_state = UpstreamCaptureState::default();

    let primary_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let primary_address = primary_listener.local_addr().unwrap();
    let primary_upstream = Router::new()
        .route(
            "/v1/responses",
            post(upstream_responses_handler_retry_after_once_then_success),
        )
        .with_state(primary_state.clone());
    tokio::spawn(async move {
        axum::serve(primary_listener, primary_upstream)
            .await
            .unwrap();
    });

    let (gateway_app, _admin_app, _admin_token, api_key) =
        setup_stateful_responses_route_with_single_provider(
            tenant_id,
            project_id,
            "provider-responses-retry-after-primary",
            &format!("http://{primary_address}"),
            "Retry After Responses Primary",
            "cred-responses-retry-after-primary",
            "sk-responses-retry-after-primary",
            "route-responses-retry-after-primary",
        )
        .await;

    let started = Instant::now();
    let response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/responses")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"gpt-4.1\",\"input\":\"retry after please for responses\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let elapsed = started.elapsed();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["id"], "resp_retry_after_recovered");
    assert_eq!(primary_state.request_count.load(Ordering::SeqCst), 2);
    assert!(
        elapsed >= Duration::from_millis(900),
        "expected retry-after delay to be honored, got {:?}",
        elapsed
    );

    let metrics = gateway_app
        .oneshot(
            Request::builder()
                .uri("/metrics")
                .header("authorization", "Bearer local-dev-metrics-token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(metrics.status(), StatusCode::OK);
    let metrics_body = axum::body::to_bytes(metrics.into_body(), usize::MAX)
        .await
        .unwrap();
    let metrics_text = String::from_utf8(metrics_body.to_vec()).unwrap();
    assert!(metrics_text.contains(
        "sdkwork_upstream_retry_reasons_total{service=\"gateway\",capability=\"responses\",provider=\"provider-responses-retry-after-primary\",outcome=\"scheduled\",reason=\"status_429\"} 1"
    ));
    assert!(metrics_text.contains(
        "sdkwork_upstream_retry_delay_ms_total{service=\"gateway\",capability=\"responses\",provider=\"provider-responses-retry-after-primary\",source=\"retry_after_seconds\"} 1000"
    ));
}

#[serial(extension_env)]
#[tokio::test]
async fn stateful_responses_route_honors_http_date_retry_after_before_retrying_primary_provider() {
    let _retry_delay_guard =
        EnvVarGuard::set("SDKWORK_GATEWAY_UPSTREAM_RETRY_MAX_DELAY_MS", "1000");
    let tenant_id = "tenant-responses-http-date-retry-after-primary";
    let project_id = "project-responses-http-date-retry-after-primary";
    let primary_state = UpstreamCaptureState::default();

    let primary_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let primary_address = primary_listener.local_addr().unwrap();
    let primary_upstream = Router::new()
        .route(
            "/v1/responses",
            post(upstream_responses_handler_http_date_retry_after_once_then_success),
        )
        .with_state(primary_state.clone());
    tokio::spawn(async move {
        axum::serve(primary_listener, primary_upstream)
            .await
            .unwrap();
    });

    let (gateway_app, _admin_app, _admin_token, api_key) =
        setup_stateful_responses_route_with_single_provider(
            tenant_id,
            project_id,
            "provider-responses-http-date-retry-after-primary",
            &format!("http://{primary_address}"),
            "HTTP Date Retry After Responses Primary",
            "cred-responses-http-date-retry-after-primary",
            "sk-responses-http-date-retry-after-primary",
            "route-responses-http-date-retry-after-primary",
        )
        .await;

    let started = Instant::now();
    let response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/responses")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"gpt-4.1\",\"input\":\"retry after http date please for responses\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let elapsed = started.elapsed();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["id"], "resp_http_date_retry_after_recovered");
    assert_eq!(primary_state.request_count.load(Ordering::SeqCst), 2);
    assert!(
        elapsed >= Duration::from_millis(900),
        "expected http-date retry-after delay to be honored, got {:?}",
        elapsed
    );

    let metrics = gateway_app
        .oneshot(
            Request::builder()
                .uri("/metrics")
                .header("authorization", "Bearer local-dev-metrics-token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(metrics.status(), StatusCode::OK);
    let metrics_body = axum::body::to_bytes(metrics.into_body(), usize::MAX)
        .await
        .unwrap();
    let metrics_text = String::from_utf8(metrics_body.to_vec()).unwrap();
    assert!(metrics_text.contains(
        "sdkwork_upstream_retry_reasons_total{service=\"gateway\",capability=\"responses\",provider=\"provider-responses-http-date-retry-after-primary\",outcome=\"scheduled\",reason=\"status_429\"} 1"
    ));
    assert!(metrics_text.contains(
        "sdkwork_upstream_retry_delay_ms_total{service=\"gateway\",capability=\"responses\",provider=\"provider-responses-http-date-retry-after-primary\",source=\"retry_after_http_date\"} 1000"
    ));
}
