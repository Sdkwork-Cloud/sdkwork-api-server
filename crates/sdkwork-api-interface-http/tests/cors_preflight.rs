use axum::body::Body;
use axum::http::Request;
use tower::ServiceExt;

#[tokio::test]
async fn gateway_chat_completions_preflight_includes_cors_headers() {
    let app = sdkwork_api_interface_http::gateway_router();

    let response = app
        .oneshot(
            Request::builder()
                .method("OPTIONS")
                .uri("/v1/chat/completions")
                .header("origin", "http://localhost:5174")
                .header("access-control-request-method", "POST")
                .header(
                    "access-control-request-headers",
                    "content-type,authorization",
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(response.status().is_success());
    assert_eq!(
        response
            .headers()
            .get("access-control-allow-origin")
            .and_then(|value| value.to_str().ok()),
        Some("*")
    );
    assert!(response
        .headers()
        .get("access-control-allow-methods")
        .is_some());
    assert!(response
        .headers()
        .get("access-control-allow-headers")
        .is_some());
}
