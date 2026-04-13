use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

#[tokio::test]
async fn health_route_returns_ok_and_metrics_require_bearer_token() {
    let app = sdkwork_api_interface_http::gateway_router();
    let health = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(health.status(), StatusCode::OK);
    let generated_request_id = health
        .headers()
        .get("x-request-id")
        .and_then(|value| value.to_str().ok())
        .unwrap()
        .to_owned();
    assert!(generated_request_id.starts_with("sdkw-"));

    let preserved = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/health")
                .header("x-request-id", "gateway-caller-id")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(preserved.status(), StatusCode::OK);
    assert_eq!(
        preserved
            .headers()
            .get("x-request-id")
            .and_then(|value| value.to_str().ok())
            .unwrap(),
        "gateway-caller-id"
    );

    let unauthorized_metrics = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/metrics")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(unauthorized_metrics.status(), StatusCode::UNAUTHORIZED);

    let metrics = app
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
    let bytes = to_bytes(metrics.into_body(), usize::MAX).await.unwrap();
    let body = String::from_utf8(bytes.to_vec()).unwrap();
    assert!(body.contains("sdkwork_service_info{service=\"gateway\"} 1"));
    assert!(body.contains(
        "sdkwork_http_requests_total{service=\"gateway\",method=\"GET\",route=\"/health\",status=\"200\",tenant=\"none\",model=\"none\",provider=\"none\",billing_mode=\"none\",retry_outcome=\"none\",failover_outcome=\"none\",payment_outcome=\"none\"} 2"
    ));
}
