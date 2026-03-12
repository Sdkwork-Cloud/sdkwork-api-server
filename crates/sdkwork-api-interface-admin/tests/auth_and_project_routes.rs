use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

#[tokio::test]
async fn login_route_exists() {
    let app = sdkwork_api_interface_admin::admin_router();
    let response = app
        .oneshot(
            Request::builder()
                .uri("/admin/auth/login")
                .method("POST")
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_ne!(response.status(), StatusCode::NOT_FOUND);
}
