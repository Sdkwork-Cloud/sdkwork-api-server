use super::*;

#[tokio::test]
async fn stateful_conversations_route_relays_to_openai_compatible_provider() {
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

    let LocalConversationsTestContext {
        admin_app,
        admin_token,
        api_key,
        gateway_app,
    } = local_conversations_test_context("tenant-1", "project-1").await;

    let _ = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/channels")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from("{\"id\":\"openai\",\"name\":\"OpenAI\"}"))
                .unwrap(),
        )
        .await
        .unwrap();

    let provider = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"id\":\"provider-openai-official\",\"channel_id\":\"openai\",\"adapter_kind\":\"openai\",\"base_url\":\"http://{address}\",\"display_name\":\"OpenAI Official\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(provider.status(), StatusCode::CREATED);

    let credential = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/credentials")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"tenant_id\":\"tenant-1\",\"provider_id\":\"provider-openai-official\",\"key_reference\":\"cred-openai\",\"secret_value\":\"sk-upstream-openai\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(credential.status(), StatusCode::CREATED);

    let create_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/conversations")
                .header("authorization", format!("Bearer {api_key}"))
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
        Some("Bearer sk-upstream-openai")
    );
    support::assert_single_usage_record_and_decision_log(
        admin_app.clone(),
        &admin_token,
        "conv_upstream",
        "provider-openai-official",
        "conversations",
    )
    .await;

    let list_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/conversations")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(list_response.status(), StatusCode::OK);
    let list_json = read_json(list_response).await;
    assert_eq!(list_json["object"], "list");

    let retrieve_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/conversations/conv_1")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(retrieve_response.status(), StatusCode::OK);
    let retrieve_json = read_json(retrieve_response).await;
    assert_eq!(retrieve_json["id"], "conv_1");

    let update_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/conversations/conv_1")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from("{\"metadata\":{\"workspace\":\"next\"}}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(update_response.status(), StatusCode::OK);
    let update_json = read_json(update_response).await;
    assert_eq!(update_json["metadata"]["workspace"], "next");

    let delete_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/v1/conversations/conv_1")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(delete_response.status(), StatusCode::OK);
    let delete_json = read_json(delete_response).await;
    assert_eq!(delete_json["deleted"], true);

    let create_items_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/conversations/conv_1/items")
                .header("authorization", format!("Bearer {api_key}"))
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

    let list_items_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/conversations/conv_1/items")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(list_items_response.status(), StatusCode::OK);
    let list_items_json = read_json(list_items_response).await;
    assert_eq!(list_items_json["data"][0]["id"], "item_1");

    let retrieve_item_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/conversations/conv_1/items/item_1")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(retrieve_item_response.status(), StatusCode::OK);
    let retrieve_item_json = read_json(retrieve_item_response).await;
    assert_eq!(retrieve_item_json["id"], "item_1");

    let delete_item_response = gateway_app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/v1/conversations/conv_1/items/item_1")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(delete_item_response.status(), StatusCode::OK);
    let delete_item_json = read_json(delete_item_response).await;
    assert_eq!(delete_item_json["deleted"], true);
}

#[tokio::test]
async fn stateful_conversation_retrieve_route_returns_not_found_without_usage() {
    let LocalConversationsTestContext {
        admin_app,
        admin_token,
        api_key,
        gateway_app,
    } = local_conversations_test_context(
        "tenant-conversation-retrieve-missing",
        "project-conversation-retrieve-missing",
    )
    .await;

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/conversations/conv_missing")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested conversation was not found.").await;
    support::assert_no_usage_records(admin_app, &admin_token).await;
}

#[tokio::test]
async fn stateful_conversation_update_route_returns_not_found_without_usage() {
    let LocalConversationsTestContext {
        admin_app,
        admin_token,
        api_key,
        gateway_app,
    } = local_conversations_test_context(
        "tenant-conversation-update-missing",
        "project-conversation-update-missing",
    )
    .await;

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/conversations/conv_missing")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from("{\"metadata\":{\"workspace\":\"next\"}}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested conversation was not found.").await;
    support::assert_no_usage_records(admin_app, &admin_token).await;
}

#[tokio::test]
async fn stateful_conversation_delete_route_returns_not_found_without_usage() {
    let LocalConversationsTestContext {
        admin_app,
        admin_token,
        api_key,
        gateway_app,
    } = local_conversations_test_context(
        "tenant-conversation-delete-missing",
        "project-conversation-delete-missing",
    )
    .await;

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/v1/conversations/conv_missing")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested conversation was not found.").await;
    support::assert_no_usage_records(admin_app, &admin_token).await;
}

#[tokio::test]
async fn stateful_conversation_items_create_route_returns_not_found_without_usage() {
    let LocalConversationsTestContext {
        admin_app,
        admin_token,
        api_key,
        gateway_app,
    } = local_conversations_test_context(
        "tenant-conversation-items-create-missing",
        "project-conversation-items-create-missing",
    )
    .await;

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/conversations/conv_missing/items")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"items\":[{\"id\":\"item_1\",\"type\":\"message\",\"role\":\"user\",\"content\":[{\"type\":\"input_text\",\"text\":\"hello\"}]}]}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested conversation was not found.").await;
    support::assert_no_usage_records(admin_app, &admin_token).await;
}

#[tokio::test]
async fn stateful_conversation_items_list_route_returns_not_found_without_usage() {
    let LocalConversationsTestContext {
        admin_app,
        admin_token,
        api_key,
        gateway_app,
    } = local_conversations_test_context(
        "tenant-conversation-items-list-missing",
        "project-conversation-items-list-missing",
    )
    .await;

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/conversations/conv_missing/items")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested conversation was not found.").await;
    support::assert_no_usage_records(admin_app, &admin_token).await;
}

#[tokio::test]
async fn stateful_conversation_item_retrieve_route_returns_not_found_without_usage() {
    let LocalConversationsTestContext {
        admin_app,
        admin_token,
        api_key,
        gateway_app,
    } = local_conversations_test_context(
        "tenant-conversation-item-retrieve-missing",
        "project-conversation-item-retrieve-missing",
    )
    .await;

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/conversations/conv_1/items/item_missing")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested conversation was not found.").await;
    support::assert_no_usage_records(admin_app, &admin_token).await;
}

#[tokio::test]
async fn stateful_conversation_item_delete_route_returns_not_found_without_usage() {
    let LocalConversationsTestContext {
        admin_app,
        admin_token,
        api_key,
        gateway_app,
    } = local_conversations_test_context(
        "tenant-conversation-item-delete-missing",
        "project-conversation-item-delete-missing",
    )
    .await;

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/v1/conversations/conv_1/items/item_missing")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested conversation was not found.").await;
    support::assert_no_usage_records(admin_app, &admin_token).await;
}
