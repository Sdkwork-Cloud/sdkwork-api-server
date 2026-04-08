use super::*;

#[tokio::test]
async fn uploads_route_returns_ok() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/uploads")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"purpose\":\"batch\",\"filename\":\"input.jsonl\",\"mime_type\":\"application/jsonl\",\"bytes\":1024}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn upload_parts_route_returns_ok() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/uploads/upload_1/parts")
                .header(
                    "content-type",
                    "multipart/form-data; boundary=----sdkwork-upload-part",
                )
                .body(Body::from(build_upload_part_multipart_body(
                    "----sdkwork-upload-part",
                )))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn upload_complete_route_returns_ok() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/uploads/upload_1/complete")
                .header("content-type", "application/json")
                .body(Body::from("{\"part_ids\":[\"part_1\",\"part_2\"]}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn upload_cancel_route_returns_ok() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/uploads/upload_1/cancel")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn upload_parts_route_returns_not_found_for_unknown_upload() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/uploads/upload_missing/parts")
                .header(
                    "content-type",
                    "multipart/form-data; boundary=----sdkwork-upload-part",
                )
                .body(Body::from(build_upload_part_multipart_body(
                    "----sdkwork-upload-part",
                )))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested upload session was not found.").await;
}

#[tokio::test]
async fn upload_complete_route_returns_not_found_for_unknown_upload() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/uploads/upload_missing/complete")
                .header("content-type", "application/json")
                .body(Body::from("{\"part_ids\":[\"part_1\",\"part_2\"]}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested upload session was not found.").await;
}

#[tokio::test]
async fn upload_cancel_route_returns_not_found_for_unknown_upload() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/uploads/upload_missing/cancel")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested upload session was not found.").await;
}
