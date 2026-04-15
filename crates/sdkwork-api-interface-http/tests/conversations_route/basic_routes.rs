use super::*;

#[tokio::test]
async fn conversations_route_returns_invalid_request_without_provider() {
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

    assert_openai_invalid_request(
        response,
        "Local conversation fallback is not supported without an upstream provider.",
        "invalid_conversation_request",
    )
    .await;
}

#[tokio::test]
async fn conversations_list_route_returns_invalid_request_without_provider() {
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

    assert_openai_invalid_request(
        response,
        "Local conversation listing fallback is not supported without an upstream provider.",
        "invalid_conversation_request",
    )
    .await;
}

#[tokio::test]
async fn conversation_retrieve_update_delete_routes_return_not_found_without_local_state() {
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
    assert_openai_not_found(retrieve, "Requested conversation was not found.").await;

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
    assert_openai_not_found(update, "Requested conversation was not found.").await;

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
    assert_openai_not_found(delete, "Requested conversation was not found.").await;
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
async fn conversation_item_routes_surface_local_fallback_contract() {
    let app = sdkwork_api_interface_http::gateway_router();

    let create = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/conversations/conv_local_1/items")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"items\":[{\"id\":\"item_1\",\"type\":\"message\",\"role\":\"user\",\"content\":[{\"type\":\"input_text\",\"text\":\"hello\"}]}]}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_openai_invalid_request(
        create,
        "Persisted local conversation item state is required for local item creation.",
        "invalid_conversation_request",
    )
    .await;

    let list = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/conversations/conv_local_1/items")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_openai_invalid_request(
        list,
        "Persisted local conversation item state is required for local item listing.",
        "invalid_conversation_request",
    )
    .await;

    let retrieve = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/conversations/conv_local_1/items/item_local_1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_openai_not_found(retrieve, "Requested conversation item was not found.").await;

    let delete = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/v1/conversations/conv_local_1/items/item_local_1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_openai_not_found(delete, "Requested conversation item was not found.").await;
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

    assert_openai_not_found(response, "Requested conversation item was not found.").await;
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

    assert_openai_not_found(response, "Requested conversation item was not found.").await;
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
