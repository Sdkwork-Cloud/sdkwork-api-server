use super::*;
use serial_test::serial;

#[serial(extension_env)]
#[tokio::test]
async fn files_route_returns_invalid_request_without_provider() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/files")
                .header(
                    "content-type",
                    "multipart/form-data; boundary=----sdkwork-boundary",
                )
                .body(Body::from(build_file_multipart_body(
                    "----sdkwork-boundary",
                )))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_invalid_file_request(
        response,
        "Local file fallback is not supported without an upstream provider.",
    )
    .await;
}

#[serial(extension_env)]
#[tokio::test]
async fn files_route_returns_invalid_request_for_blank_purpose() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/files")
                .header(
                    "content-type",
                    "multipart/form-data; boundary=----sdkwork-boundary",
                )
                .body(Body::from(build_file_multipart_body_with_fields(
                    "----sdkwork-boundary",
                    "",
                    "train.jsonl",
                )))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_invalid_file_request(response, "File purpose is required.").await;
}

#[serial(extension_env)]
#[tokio::test]
async fn files_list_route_returns_invalid_request_without_provider() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/files")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_invalid_file_request(
        response,
        "Local file listing fallback is not supported without an upstream provider.",
    )
    .await;
}

#[serial(extension_env)]
#[tokio::test]
async fn file_retrieve_route_returns_not_found_without_local_state() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/files/file_local_1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_file_not_found(response, "Requested file was not found.").await;
}

#[serial(extension_env)]
#[tokio::test]
async fn file_retrieve_route_returns_not_found_for_unknown_file() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/files/file_missing")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_file_not_found(response, "Requested file was not found.").await;
}

#[serial(extension_env)]
#[tokio::test]
async fn file_delete_route_returns_not_found_without_local_state() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/v1/files/file_local_1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_file_not_found(response, "Requested file was not found.").await;
}

#[serial(extension_env)]
#[tokio::test]
async fn file_content_route_returns_not_found_without_local_state() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/files/file_local_1/content")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_file_not_found(response, "Requested file was not found.").await;
}

#[serial(extension_env)]
#[tokio::test]
async fn file_delete_route_returns_not_found_for_unknown_file() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/v1/files/file_missing")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_file_not_found(response, "Requested file was not found.").await;
}

#[serial(extension_env)]
#[tokio::test]
async fn file_content_route_returns_not_found_error_for_unknown_file() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/files/file_missing/content")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_file_not_found(response, "Requested file was not found.").await;
}
