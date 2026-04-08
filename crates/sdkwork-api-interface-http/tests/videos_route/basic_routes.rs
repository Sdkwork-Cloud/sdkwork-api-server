use super::*;

#[serial(extension_env)]
#[tokio::test]
async fn videos_create_route_returns_ok() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/videos")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"sora-1\",\"prompt\":\"A short cinematic flyover\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[serial(extension_env)]
#[tokio::test]
async fn videos_list_route_returns_ok() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/videos")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[serial(extension_env)]
#[tokio::test]
async fn video_retrieve_route_returns_ok() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/videos/video_1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[serial(extension_env)]
#[tokio::test]
async fn video_delete_route_returns_ok() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/v1/videos/video_1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[serial(extension_env)]
#[tokio::test]
async fn video_content_route_returns_ok() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/videos/video_1/content")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[serial(extension_env)]
#[tokio::test]
async fn video_content_route_returns_not_found_error_for_unknown_video() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/videos/video_missing/content")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let json = read_json(response).await;
    assert_eq!(
        json["error"]["message"],
        "Requested video asset was not found."
    );
    assert_eq!(json["error"]["type"], "invalid_request_error");
    assert_eq!(json["error"]["code"], "not_found");
}

#[serial(extension_env)]
#[tokio::test]
async fn video_retrieve_route_returns_not_found_error_for_unknown_video() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/videos/video_missing")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_video_not_found(response, "Requested video was not found.").await;
}

#[serial(extension_env)]
#[tokio::test]
async fn video_delete_route_returns_not_found_error_for_unknown_video() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/v1/videos/video_missing")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_video_not_found(response, "Requested video was not found.").await;
}

#[serial(extension_env)]
#[tokio::test]
async fn video_remix_route_returns_ok() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/videos/video_1/remix")
                .header("content-type", "application/json")
                .body(Body::from("{\"prompt\":\"Make it sunset\"}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[serial(extension_env)]
#[tokio::test]
async fn video_characters_list_route_returns_ok() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/videos/video_1/characters")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[serial(extension_env)]
#[tokio::test]
async fn video_character_retrieve_route_returns_ok() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/videos/video_1/characters/char_1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[serial(extension_env)]
#[tokio::test]
async fn video_character_update_route_returns_ok() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/videos/video_1/characters/char_1")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"name\":\"Hero\",\"prompt\":\"Add a red jacket\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[serial(extension_env)]
#[tokio::test]
async fn video_extend_route_returns_ok() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/videos/video_1/extend")
                .header("content-type", "application/json")
                .body(Body::from("{\"prompt\":\"Extend the ending\"}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[serial(extension_env)]
#[tokio::test]
async fn video_character_create_canonical_route_returns_ok() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/videos/characters")
                .header("content-type", "application/json")
                .body(Body::from("{\"name\":\"Hero\",\"video_id\":\"video_1\"}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[serial(extension_env)]
#[tokio::test]
async fn video_character_retrieve_canonical_route_returns_ok() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/videos/characters/char_1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[serial(extension_env)]
#[tokio::test]
async fn video_edits_canonical_route_returns_ok() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/videos/edits")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"prompt\":\"Add dramatic lighting\",\"video_id\":\"video_1\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[serial(extension_env)]
#[tokio::test]
async fn video_extensions_canonical_route_returns_ok() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/videos/extensions")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"prompt\":\"Extend the ending\",\"video_id\":\"video_1\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}
