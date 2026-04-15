use super::*;

#[serial(extension_env)]
#[tokio::test]
async fn containers_create_route_returns_invalid_request_without_upstream_provider() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/containers")
                .header("content-type", "application/json")
                .body(Body::from("{\"name\":\"ci-container\"}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_invalid_request(
        response,
        "Local container fallback is not supported without an upstream provider.",
        "invalid_container_request",
    )
    .await;
}

#[serial(extension_env)]
#[tokio::test]
async fn containers_list_route_returns_invalid_request_without_upstream_provider() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/containers")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_invalid_request(
        response,
        "Local container listing fallback is not supported without an upstream provider.",
        "invalid_container_request",
    )
    .await;
}

#[serial(extension_env)]
#[tokio::test]
async fn container_retrieve_route_returns_not_found_without_local_state() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/containers/container_1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested container was not found.").await;
}

#[serial(extension_env)]
#[tokio::test]
async fn container_retrieve_route_returns_not_found_for_unknown_container() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/containers/container_missing")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested container was not found.").await;
}

#[serial(extension_env)]
#[tokio::test]
async fn container_delete_route_returns_not_found_without_local_state() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/v1/containers/container_1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested container was not found.").await;
}

#[serial(extension_env)]
#[tokio::test]
async fn container_delete_route_returns_not_found_for_unknown_container() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/v1/containers/container_missing")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested container was not found.").await;
}

#[serial(extension_env)]
#[tokio::test]
async fn container_file_create_route_returns_not_found_without_local_state() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/containers/container_1/files")
                .header("content-type", "application/json")
                .body(Body::from("{\"file_id\":\"file_1\"}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested container file was not found.").await;
}

#[serial(extension_env)]
#[tokio::test]
async fn container_file_create_route_returns_not_found_for_unknown_container() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/containers/container_missing/files")
                .header("content-type", "application/json")
                .body(Body::from("{\"file_id\":\"file_1\"}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested container file was not found.").await;
}

#[serial(extension_env)]
#[tokio::test]
async fn container_files_list_route_returns_not_found_without_local_state() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/containers/container_1/files")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested container was not found.").await;
}

#[serial(extension_env)]
#[tokio::test]
async fn container_files_list_route_returns_not_found_for_unknown_container() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/containers/container_missing/files")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested container was not found.").await;
}

#[serial(extension_env)]
#[tokio::test]
async fn container_file_retrieve_route_returns_not_found_without_local_state() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/containers/container_1/files/file_1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested container file was not found.").await;
}

#[serial(extension_env)]
#[tokio::test]
async fn container_file_retrieve_route_returns_not_found_for_unknown_file() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/containers/container_1/files/file_missing")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested container file was not found.").await;
}

#[serial(extension_env)]
#[tokio::test]
async fn container_file_delete_route_returns_not_found_without_local_state() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/v1/containers/container_1/files/file_1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested container file was not found.").await;
}

#[serial(extension_env)]
#[tokio::test]
async fn container_file_delete_route_returns_not_found_for_unknown_file() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/v1/containers/container_1/files/file_missing")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested container file was not found.").await;
}

#[serial(extension_env)]
#[tokio::test]
async fn container_file_content_route_returns_not_found_without_local_state() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/containers/container_1/files/file_1/content")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested container file was not found.").await;
}

#[serial(extension_env)]
#[tokio::test]
async fn container_file_content_route_returns_not_found_error_for_unknown_file() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/containers/container_1/files/file_missing/content")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested container file was not found.").await;
}
