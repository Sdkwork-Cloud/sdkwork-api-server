use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

#[tokio::test]
async fn openapi_routes_expose_admin_api_inventory() {
    let app = sdkwork_api_interface_admin::admin_router();

    let openapi = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/admin/openapi.json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(openapi.status(), StatusCode::OK);
    let bytes = to_bytes(openapi.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["openapi"], "3.1.0");
    assert_eq!(json["info"]["title"], "SDKWORK Admin API");
    assert!(json["paths"]["/admin/health"]["get"].is_object());
    assert!(json["paths"]["/admin/auth/login"]["post"].is_object());
    assert!(json["paths"]["/admin/tenants"]["get"].is_object());
    assert_eq!(
        json["paths"]["/admin/tenants"]["get"]["security"][0]["bearerAuth"],
        serde_json::json!([])
    );
    assert!(
        json["paths"]["/admin/auth/login"]["post"]["security"].is_null()
            || json["paths"]["/admin/auth/login"]["post"]["security"]
                .as_array()
                .is_some_and(Vec::is_empty)
    );

    let docs = app
        .oneshot(
            Request::builder()
                .uri("/admin/docs")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(docs.status(), StatusCode::OK);
    let bytes = to_bytes(docs.into_body(), usize::MAX).await.unwrap();
    let html = String::from_utf8(bytes.to_vec()).unwrap();
    assert!(html.contains("SDKWORK Admin API"));
    assert!(html.contains("/admin/openapi.json"));
}
