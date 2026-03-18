use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

#[tokio::test]
async fn openapi_routes_expose_gateway_api_inventory() {
    let app = sdkwork_api_interface_http::gateway_router();

    let openapi = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/openapi.json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(openapi.status(), StatusCode::OK);
    let bytes = to_bytes(openapi.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["openapi"], "3.1.0");
    assert_eq!(json["info"]["title"], "SDKWORK Gateway API");
    assert!(json["paths"]["/health"]["get"].is_object());
    assert!(json["paths"]["/v1/models"]["get"].is_object());
    assert!(json["paths"]["/v1/chat/completions"]["post"].is_object());
    assert_eq!(
        json["paths"]["/v1/chat/completions"]["post"]["security"][0]["bearerAuth"],
        serde_json::json!([])
    );

    let docs = app
        .oneshot(Request::builder().uri("/docs").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(docs.status(), StatusCode::OK);
    let bytes = to_bytes(docs.into_body(), usize::MAX).await.unwrap();
    let html = String::from_utf8(bytes.to_vec()).unwrap();
    assert!(html.contains("SDKWORK Gateway API"));
    assert!(html.contains("/openapi.json"));
}
