use super::*;

#[tokio::test]
async fn threads_route_returns_ok() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/threads")
                .header("content-type", "application/json")
                .body(Body::from("{\"metadata\":{\"workspace\":\"default\"}}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn thread_retrieve_update_delete_routes_return_ok() {
    let app = sdkwork_api_interface_http::gateway_router();

    let retrieve = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/threads/thread_1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(retrieve.status(), StatusCode::OK);

    let update = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/threads/thread_1")
                .header("content-type", "application/json")
                .body(Body::from("{\"metadata\":{\"workspace\":\"next\"}}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(update.status(), StatusCode::OK);

    let delete = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/v1/threads/thread_1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(delete.status(), StatusCode::OK);
}

#[tokio::test]
async fn thread_retrieve_route_returns_not_found_for_unknown_thread() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/threads/thread_missing")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested thread was not found.").await;
}

#[tokio::test]
async fn thread_update_route_returns_not_found_for_unknown_thread() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/threads/thread_missing")
                .header("content-type", "application/json")
                .body(Body::from("{\"metadata\":{\"workspace\":\"next\"}}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested thread was not found.").await;
}

#[tokio::test]
async fn thread_delete_route_returns_not_found_for_unknown_thread() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/v1/threads/thread_missing")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested thread was not found.").await;
}

#[tokio::test]
async fn thread_message_routes_return_ok() {
    let app = sdkwork_api_interface_http::gateway_router();

    let create = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/threads/thread_1/messages")
                .header("content-type", "application/json")
                .body(Body::from("{\"role\":\"user\",\"content\":\"hello\"}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create.status(), StatusCode::OK);

    let list = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/threads/thread_1/messages")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(list.status(), StatusCode::OK);

    let retrieve = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/threads/thread_1/messages/msg_1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(retrieve.status(), StatusCode::OK);

    let update = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/threads/thread_1/messages/msg_1")
                .header("content-type", "application/json")
                .body(Body::from("{\"metadata\":{\"pinned\":\"true\"}}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(update.status(), StatusCode::OK);

    let delete = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/v1/threads/thread_1/messages/msg_1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(delete.status(), StatusCode::OK);
}

#[tokio::test]
async fn thread_messages_create_route_returns_not_found_for_unknown_thread() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/threads/thread_missing/messages")
                .header("content-type", "application/json")
                .body(Body::from("{\"role\":\"user\",\"content\":\"hello\"}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested thread was not found.").await;
}

#[tokio::test]
async fn thread_messages_list_route_returns_not_found_for_unknown_thread() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/threads/thread_missing/messages")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested thread was not found.").await;
}

#[tokio::test]
async fn thread_message_retrieve_route_returns_not_found_for_unknown_message() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/threads/thread_1/messages/msg_missing")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested thread message was not found.").await;
}

#[tokio::test]
async fn thread_message_update_route_returns_not_found_for_unknown_message() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/threads/thread_1/messages/msg_missing")
                .header("content-type", "application/json")
                .body(Body::from("{\"metadata\":{\"pinned\":\"true\"}}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested thread message was not found.").await;
}

#[tokio::test]
async fn thread_message_delete_route_returns_not_found_for_unknown_message() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/v1/threads/thread_1/messages/msg_missing")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested thread message was not found.").await;
}
