use super::*;

#[tokio::test]
async fn stateful_conversations_create_usage_uses_created_conversation_id_for_billing() {
    let tenant_id = "tenant-conversations-create";
    let project_id = "project-conversations-create";
    let upstream_state = UpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route("/v1/conversations", post(upstream_conversations_handler))
        .with_state(upstream_state.clone());

    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });

    let LocalConversationsTestContext {
        admin_app,
        admin_token,
        api_key,
        gateway_app,
    } = local_conversations_test_context(tenant_id, project_id).await;

    let create_channel = admin_app
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
    assert_eq!(create_channel.status(), StatusCode::CREATED);

    let provider_conversation = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"id\":\"provider-conversations-route\",\"channel_id\":\"openai\",\"adapter_kind\":\"openai\",\"base_url\":\"http://{address}\",\"display_name\":\"Conversations Route Provider\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(provider_conversation.status(), StatusCode::CREATED);

    let provider_created = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"id\":\"provider-conversations-created-id\",\"channel_id\":\"openai\",\"adapter_kind\":\"openai\",\"base_url\":\"http://127.0.0.1:1\",\"display_name\":\"Conversations Created Id Provider\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(provider_created.status(), StatusCode::CREATED);

    let conversation_credential = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/credentials")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"tenant_id\":\"{tenant_id}\",\"provider_id\":\"provider-conversations-route\",\"key_reference\":\"cred-conversations-route\",\"secret_value\":\"sk-conversations-route\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(conversation_credential.status(), StatusCode::CREATED);

    let created_credential = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/credentials")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"tenant_id\":\"{tenant_id}\",\"provider_id\":\"provider-conversations-created-id\",\"key_reference\":\"cred-conversations-created-id\",\"secret_value\":\"sk-conversations-created-id\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(created_credential.status(), StatusCode::CREATED);

    let conversation_policy = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/routing/policies")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "policy_id": "route-conversations-by-route-key",
                        "capability": "responses",
                        "model_pattern": "conversations",
                        "enabled": true,
                        "priority": 200,
                        "ordered_provider_ids": ["provider-conversations-route"]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(conversation_policy.status(), StatusCode::CREATED);

    let created_policy = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/routing/policies")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "policy_id": "route-conversations-by-created-id",
                        "capability": "responses",
                        "model_pattern": "conv_upstream",
                        "enabled": true,
                        "priority": 100,
                        "ordered_provider_ids": ["provider-conversations-created-id"]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(created_policy.status(), StatusCode::CREATED);

    let response = gateway_app
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
    assert_eq!(response.status(), StatusCode::OK);
    let response_json = read_json(response).await;
    assert_eq!(response_json["id"], "conv_upstream");
    assert_eq!(
        upstream_state.authorization_header().as_deref(),
        Some("Bearer sk-conversations-route")
    );
    support::assert_single_usage_record_and_decision_log(
        admin_app,
        &admin_token,
        "conv_upstream",
        "provider-conversations-route",
        "conversations",
    )
    .await;
}

#[tokio::test]
async fn stateful_conversation_item_usage_uses_conversation_route_key_for_provider_selection() {
    let tenant_id = "tenant-conversation-usage";
    let project_id = "project-conversation-usage";
    let upstream_state = UpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route(
            "/v1/conversations/conv_1/items/item_1",
            get(upstream_conversation_item_retrieve_handler),
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
    } = local_conversations_test_context(tenant_id, project_id).await;

    let create_channel = admin_app
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
    assert_eq!(create_channel.status(), StatusCode::CREATED);

    let provider_conversation = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"id\":\"provider-conversation\",\"channel_id\":\"openai\",\"adapter_kind\":\"openai\",\"base_url\":\"http://{address}\",\"display_name\":\"Conversation Provider\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(provider_conversation.status(), StatusCode::CREATED);

    let provider_item = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"id\":\"provider-item\",\"channel_id\":\"openai\",\"adapter_kind\":\"openai\",\"base_url\":\"http://127.0.0.1:1\",\"display_name\":\"Item Provider\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(provider_item.status(), StatusCode::CREATED);

    let conversation_credential = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/credentials")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"tenant_id\":\"{tenant_id}\",\"provider_id\":\"provider-conversation\",\"key_reference\":\"cred-conversation\",\"secret_value\":\"sk-conversation\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(conversation_credential.status(), StatusCode::CREATED);

    let item_credential = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/credentials")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"tenant_id\":\"{tenant_id}\",\"provider_id\":\"provider-item\",\"key_reference\":\"cred-item\",\"secret_value\":\"sk-item\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(item_credential.status(), StatusCode::CREATED);

    let conversation_policy = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/routing/policies")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "policy_id": "route-conversations-by-conversation",
                        "capability": "responses",
                        "model_pattern": "conv_1",
                        "enabled": true,
                        "priority": 200,
                        "ordered_provider_ids": ["provider-conversation"]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(conversation_policy.status(), StatusCode::CREATED);

    let item_policy = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/routing/policies")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "policy_id": "route-conversations-by-item",
                        "capability": "responses",
                        "model_pattern": "item_1",
                        "enabled": true,
                        "priority": 100,
                        "ordered_provider_ids": ["provider-item"]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(item_policy.status(), StatusCode::CREATED);

    let response = gateway_app
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
    assert_eq!(response.status(), StatusCode::OK);
    let response_json = read_json(response).await;
    assert_eq!(response_json["id"], "item_1");
    assert_eq!(
        upstream_state.authorization_header().as_deref(),
        Some("Bearer sk-conversation")
    );

    let usage = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/usage/records")
                .header("authorization", format!("Bearer {admin_token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(usage.status(), StatusCode::OK);
    let usage_json = read_json(usage).await;
    assert_eq!(usage_json.as_array().unwrap().len(), 1);
    assert_eq!(usage_json[0]["model"], "item_1");
    assert_eq!(usage_json[0]["provider"], "provider-conversation");

    let logs = admin_app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/routing/decision-logs")
                .header("authorization", format!("Bearer {admin_token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(logs.status(), StatusCode::OK);
    let logs_json = read_json(logs).await;
    assert_eq!(logs_json.as_array().unwrap().len(), 1);
    assert_eq!(logs_json[0]["route_key"], "conv_1");
    assert_eq!(logs_json[0]["selected_provider_id"], "provider-conversation");
}

#[tokio::test]
async fn stateful_conversation_items_create_usage_uses_created_item_id_for_billing() {
    let tenant_id = "tenant-conversation-items-create";
    let project_id = "project-conversation-items-create";
    let conversation_id = "conv_1";
    let upstream_state = UpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route(
            "/v1/conversations/conv_1/items",
            post(upstream_conversation_items_handler),
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
    } = local_conversations_test_context(tenant_id, project_id).await;

    let create_channel = admin_app
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
    assert_eq!(create_channel.status(), StatusCode::CREATED);

    let provider_conversation = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"id\":\"provider-conversation-items-route\",\"channel_id\":\"openai\",\"adapter_kind\":\"openai\",\"base_url\":\"http://{address}\",\"display_name\":\"Conversation Items Route Provider\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(provider_conversation.status(), StatusCode::CREATED);

    let provider_item = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"id\":\"provider-conversation-items-child\",\"channel_id\":\"openai\",\"adapter_kind\":\"openai\",\"base_url\":\"http://127.0.0.1:1\",\"display_name\":\"Conversation Items Child Provider\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(provider_item.status(), StatusCode::CREATED);

    let conversation_credential = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/credentials")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"tenant_id\":\"{tenant_id}\",\"provider_id\":\"provider-conversation-items-route\",\"key_reference\":\"cred-conversation-items-route\",\"secret_value\":\"sk-conversation-items-route\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(conversation_credential.status(), StatusCode::CREATED);

    let item_credential = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/credentials")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"tenant_id\":\"{tenant_id}\",\"provider_id\":\"provider-conversation-items-child\",\"key_reference\":\"cred-conversation-items-child\",\"secret_value\":\"sk-conversation-items-child\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(item_credential.status(), StatusCode::CREATED);

    let conversation_policy = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/routing/policies")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "policy_id": "route-conversation-items-by-conversation",
                        "capability": "responses",
                        "model_pattern": conversation_id,
                        "enabled": true,
                        "priority": 200,
                        "ordered_provider_ids": ["provider-conversation-items-route"]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(conversation_policy.status(), StatusCode::CREATED);

    let item_policy = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/routing/policies")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "policy_id": "route-conversation-items-by-child",
                        "capability": "responses",
                        "model_pattern": "item_1",
                        "enabled": true,
                        "priority": 100,
                        "ordered_provider_ids": ["provider-conversation-items-child"]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(item_policy.status(), StatusCode::CREATED);

    let response = gateway_app
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
    assert_eq!(response.status(), StatusCode::OK);
    let response_json = read_json(response).await;
    assert_eq!(response_json["data"][0]["id"], "item_1");
    assert_eq!(
        upstream_state.authorization_header().as_deref(),
        Some("Bearer sk-conversation-items-route")
    );
    support::assert_single_usage_record_and_decision_log(
        admin_app,
        &admin_token,
        "item_1",
        "provider-conversation-items-route",
        conversation_id,
    )
    .await;
}
