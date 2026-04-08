use super::*;

#[tokio::test]
async fn stateful_threads_routes_relay_to_openai_compatible_provider() {
    let upstream_state = UpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route("/v1/threads", post(upstream_threads_handler))
        .route(
            "/v1/threads/thread_1",
            get(upstream_thread_retrieve_handler)
                .post(upstream_thread_update_handler)
                .delete(upstream_thread_delete_handler),
        )
        .route(
            "/v1/threads/thread_1/messages",
            get(upstream_thread_messages_list_handler).post(upstream_thread_messages_handler),
        )
        .route(
            "/v1/threads/thread_1/messages/msg_1",
            get(upstream_thread_message_retrieve_handler)
                .post(upstream_thread_message_update_handler)
                .delete(upstream_thread_message_delete_handler),
        )
        .with_state(upstream_state.clone());

    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });

    let LocalThreadsTestContext {
        admin_app,
        admin_token,
        api_key,
        gateway_app,
    } = local_threads_test_context("tenant-1", "project-1").await;

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
                .uri("/v1/threads")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from("{\"metadata\":{\"workspace\":\"default\"}}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_response.status(), StatusCode::OK);
    let create_json = read_json(create_response).await;
    assert_eq!(create_json["id"], "thread_1");
    assert_eq!(
        upstream_state.authorization_header().as_deref(),
        Some("Bearer sk-upstream-openai")
    );
    support::assert_single_usage_record_and_decision_log(
        admin_app.clone(),
        &admin_token,
        "thread_1",
        "provider-openai-official",
        "threads",
    )
    .await;

    let retrieve_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/threads/thread_1")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(retrieve_response.status(), StatusCode::OK);
    let retrieve_json = read_json(retrieve_response).await;
    assert_eq!(retrieve_json["id"], "thread_1");

    let update_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/threads/thread_1")
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
                .uri("/v1/threads/thread_1")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(delete_response.status(), StatusCode::OK);
    let delete_json = read_json(delete_response).await;
    assert_eq!(delete_json["deleted"], true);

    let create_message_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/threads/thread_1/messages")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from("{\"role\":\"user\",\"content\":\"hello\"}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_message_response.status(), StatusCode::OK);
    let create_message_json = read_json(create_message_response).await;
    assert_eq!(create_message_json["id"], "msg_1");

    let list_message_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/threads/thread_1/messages")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(list_message_response.status(), StatusCode::OK);
    let list_message_json = read_json(list_message_response).await;
    assert_eq!(list_message_json["data"][0]["id"], "msg_1");

    let retrieve_message_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/threads/thread_1/messages/msg_1")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(retrieve_message_response.status(), StatusCode::OK);
    let retrieve_message_json = read_json(retrieve_message_response).await;
    assert_eq!(retrieve_message_json["id"], "msg_1");

    let update_message_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/threads/thread_1/messages/msg_1")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from("{\"metadata\":{\"pinned\":\"true\"}}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(update_message_response.status(), StatusCode::OK);
    let update_message_json = read_json(update_message_response).await;
    assert_eq!(update_message_json["metadata"]["pinned"], "true");

    let delete_message_response = gateway_app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/v1/threads/thread_1/messages/msg_1")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(delete_message_response.status(), StatusCode::OK);
    let delete_message_json = read_json(delete_message_response).await;
    assert_eq!(delete_message_json["deleted"], true);
}

#[tokio::test]
async fn stateful_thread_retrieve_route_returns_not_found_without_usage() {
    let LocalThreadsTestContext {
        admin_app,
        admin_token,
        api_key,
        gateway_app,
    } = local_threads_test_context(
        "tenant-thread-retrieve-missing",
        "project-thread-retrieve-missing",
    )
    .await;

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/threads/thread_missing")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested thread was not found.").await;
    support::assert_no_usage_records(admin_app, &admin_token).await;
}

#[tokio::test]
async fn stateful_thread_update_route_returns_not_found_without_usage() {
    let LocalThreadsTestContext {
        admin_app,
        admin_token,
        api_key,
        gateway_app,
    } = local_threads_test_context(
        "tenant-thread-update-missing",
        "project-thread-update-missing",
    )
    .await;

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/threads/thread_missing")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from("{\"metadata\":{\"workspace\":\"next\"}}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested thread was not found.").await;
    support::assert_no_usage_records(admin_app, &admin_token).await;
}

#[tokio::test]
async fn stateful_thread_delete_route_returns_not_found_without_usage() {
    let LocalThreadsTestContext {
        admin_app,
        admin_token,
        api_key,
        gateway_app,
    } = local_threads_test_context(
        "tenant-thread-delete-missing",
        "project-thread-delete-missing",
    )
    .await;

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/v1/threads/thread_missing")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested thread was not found.").await;
    support::assert_no_usage_records(admin_app, &admin_token).await;
}

#[tokio::test]
async fn stateful_thread_messages_create_route_returns_not_found_without_usage() {
    let LocalThreadsTestContext {
        admin_app,
        admin_token,
        api_key,
        gateway_app,
    } = local_threads_test_context(
        "tenant-thread-messages-create-missing",
        "project-thread-messages-create-missing",
    )
    .await;

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/threads/thread_missing/messages")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from("{\"role\":\"user\",\"content\":\"hello\"}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested thread was not found.").await;
    support::assert_no_usage_records(admin_app, &admin_token).await;
}

#[tokio::test]
async fn stateful_thread_messages_list_route_returns_not_found_without_usage() {
    let LocalThreadsTestContext {
        admin_app,
        admin_token,
        api_key,
        gateway_app,
    } = local_threads_test_context(
        "tenant-thread-messages-list-missing",
        "project-thread-messages-list-missing",
    )
    .await;

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/threads/thread_missing/messages")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested thread was not found.").await;
    support::assert_no_usage_records(admin_app, &admin_token).await;
}

#[tokio::test]
async fn stateful_thread_message_retrieve_route_returns_not_found_without_usage() {
    let LocalThreadsTestContext {
        admin_app,
        admin_token,
        api_key,
        gateway_app,
    } = local_threads_test_context(
        "tenant-thread-message-retrieve-missing",
        "project-thread-message-retrieve-missing",
    )
    .await;

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/threads/thread_1/messages/msg_missing")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested thread message was not found.").await;
    support::assert_no_usage_records(admin_app, &admin_token).await;
}

#[tokio::test]
async fn stateful_thread_message_update_route_returns_not_found_without_usage() {
    let LocalThreadsTestContext {
        admin_app,
        admin_token,
        api_key,
        gateway_app,
    } = local_threads_test_context(
        "tenant-thread-message-update-missing",
        "project-thread-message-update-missing",
    )
    .await;

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/threads/thread_1/messages/msg_missing")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from("{\"metadata\":{\"pinned\":\"true\"}}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested thread message was not found.").await;
    support::assert_no_usage_records(admin_app, &admin_token).await;
}

#[tokio::test]
async fn stateful_thread_message_delete_route_returns_not_found_without_usage() {
    let LocalThreadsTestContext {
        admin_app,
        admin_token,
        api_key,
        gateway_app,
    } = local_threads_test_context(
        "tenant-thread-message-delete-missing",
        "project-thread-message-delete-missing",
    )
    .await;

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/v1/threads/thread_1/messages/msg_missing")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested thread message was not found.").await;
    support::assert_no_usage_records(admin_app, &admin_token).await;
}
