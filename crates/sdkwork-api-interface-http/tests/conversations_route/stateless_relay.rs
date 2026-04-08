use super::*;

#[tokio::test]
async fn stateless_conversations_route_relays_to_openai_compatible_provider() {
    let upstream_state = UpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route(
            "/v1/conversations",
            get(upstream_conversations_list_handler).post(upstream_conversations_handler),
        )
        .route(
            "/v1/conversations/conv_1",
            get(upstream_conversation_retrieve_handler)
                .post(upstream_conversation_update_handler)
                .delete(upstream_conversation_delete_handler),
        )
        .route(
            "/v1/conversations/conv_1/items",
            get(upstream_conversation_items_list_handler).post(upstream_conversation_items_handler),
        )
        .route(
            "/v1/conversations/conv_1/items/item_1",
            get(upstream_conversation_item_retrieve_handler)
                .delete(upstream_conversation_item_delete_handler),
        )
        .with_state(upstream_state.clone());

    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });

    let app = sdkwork_api_interface_http::gateway_router_with_stateless_config(
        sdkwork_api_interface_http::StatelessGatewayConfig::default().with_upstream(
            sdkwork_api_interface_http::StatelessGatewayUpstream::new(
                "openai",
                format!("http://{address}"),
                "sk-stateless-openai",
            ),
        ),
    );

    let create_response = app
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
    assert_eq!(create_response.status(), StatusCode::OK);
    let create_json = read_json(create_response).await;
    assert_eq!(create_json["id"], "conv_upstream");
    assert_eq!(
        upstream_state.authorization_header().as_deref(),
        Some("Bearer sk-stateless-openai")
    );

    let list_response = app
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
    assert_eq!(list_response.status(), StatusCode::OK);
    let list_json = read_json(list_response).await;
    assert_eq!(list_json["data"][0]["id"], "conv_1");

    let retrieve_response = app
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
    assert_eq!(retrieve_response.status(), StatusCode::OK);
    let retrieve_json = read_json(retrieve_response).await;
    assert_eq!(retrieve_json["id"], "conv_1");

    let update_response = app
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
    assert_eq!(update_response.status(), StatusCode::OK);
    let update_json = read_json(update_response).await;
    assert_eq!(update_json["metadata"]["workspace"], "next");

    let delete_response = app
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
    assert_eq!(delete_response.status(), StatusCode::OK);
    let delete_json = read_json(delete_response).await;
    assert_eq!(delete_json["deleted"], true);

    let create_items_response = app
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
    assert_eq!(create_items_response.status(), StatusCode::OK);
    let create_items_json = read_json(create_items_response).await;
    assert_eq!(create_items_json["data"][0]["id"], "item_1");

    let list_items_response = app
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
    assert_eq!(list_items_response.status(), StatusCode::OK);
    let list_items_json = read_json(list_items_response).await;
    assert_eq!(list_items_json["data"][0]["id"], "item_1");

    let retrieve_item_response = app
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
    assert_eq!(retrieve_item_response.status(), StatusCode::OK);
    let retrieve_item_json = read_json(retrieve_item_response).await;
    assert_eq!(retrieve_item_json["id"], "item_1");

    let delete_item_response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/v1/conversations/conv_1/items/item_1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(delete_item_response.status(), StatusCode::OK);
    let delete_item_json = read_json(delete_item_response).await;
    assert_eq!(delete_item_json["deleted"], true);
}
