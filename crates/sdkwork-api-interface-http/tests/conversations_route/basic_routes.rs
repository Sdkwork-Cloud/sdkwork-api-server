use super::*;

#[tokio::test]
async fn conversations_route_returns_ok() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/conversations")
                .header("content-type", "application/json")
                .body(Body::from("{\"metadata\":{\"workspace\":\"default\"}}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn conversations_list_route_returns_ok() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/conversations")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn conversation_retrieve_update_delete_routes_return_ok() {
    let app = sdkwork_api_interface_http::gateway_router();

    let retrieve = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/conversations/conv_1")
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
                .uri("/v1/conversations/conv_1")
                .header("content-type", "application/json")
                .body(Body::from("{\"metadata\":{\"workspace\":\"next\"}}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(update.status(), StatusCode::OK);

    let delete = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/v1/conversations/conv_1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(delete.status(), StatusCode::OK);
}

#[tokio::test]
async fn conversation_retrieve_route_returns_not_found_for_unknown_conversation() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/conversations/conv_missing")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested conversation was not found.").await;
}

#[tokio::test]
async fn conversation_update_route_returns_not_found_for_unknown_conversation() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/conversations/conv_missing")
                .header("content-type", "application/json")
                .body(Body::from("{\"metadata\":{\"workspace\":\"next\"}}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested conversation was not found.").await;
}

#[tokio::test]
async fn conversation_delete_route_returns_not_found_for_unknown_conversation() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/v1/conversations/conv_missing")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested conversation was not found.").await;
}

#[tokio::test]
async fn conversation_item_routes_return_ok() {
    let app = sdkwork_api_interface_http::gateway_router();

    let create = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/conversations/conv_1/items")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"items\":[{\"id\":\"item_1\",\"type\":\"message\",\"role\":\"user\",\"content\":[{\"type\":\"input_text\",\"text\":\"hello\"}]}]}",
                ))
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
                .uri("/v1/conversations/conv_1/items")
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
                .uri("/v1/conversations/conv_1/items/item_1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(retrieve.status(), StatusCode::OK);

    let delete = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/v1/conversations/conv_1/items/item_1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(delete.status(), StatusCode::OK);
}

#[tokio::test]
async fn conversation_items_create_route_returns_not_found_for_unknown_conversation() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/conversations/conv_missing/items")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"items\":[{\"id\":\"item_1\",\"type\":\"message\",\"role\":\"user\",\"content\":[{\"type\":\"input_text\",\"text\":\"hello\"}]}]}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested conversation was not found.").await;
}

#[tokio::test]
async fn conversation_items_list_route_returns_not_found_for_unknown_conversation() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/conversations/conv_missing/items")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested conversation was not found.").await;
}

#[tokio::test]
async fn conversation_item_retrieve_route_returns_not_found_for_unknown_item() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/conversations/conv_1/items/item_missing")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested conversation item was not found.").await;
}

#[tokio::test]
async fn conversation_item_delete_route_returns_not_found_for_unknown_item() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/v1/conversations/conv_1/items/item_missing")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested conversation item was not found.").await;
}
