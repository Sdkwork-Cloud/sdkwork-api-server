use super::*;

#[tokio::test]
async fn stateless_threads_routes_relay_to_openai_compatible_provider() {
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
                .uri("/v1/threads")
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
        Some("Bearer sk-stateless-openai")
    );

    let retrieve_response = app
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
    assert_eq!(retrieve_response.status(), StatusCode::OK);
    let retrieve_json = read_json(retrieve_response).await;
    assert_eq!(retrieve_json["id"], "thread_1");

    let update_response = app
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
    assert_eq!(update_response.status(), StatusCode::OK);
    let update_json = read_json(update_response).await;
    assert_eq!(update_json["metadata"]["workspace"], "next");

    let delete_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/v1/threads/thread_1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(delete_response.status(), StatusCode::OK);
    let delete_json = read_json(delete_response).await;
    assert_eq!(delete_json["deleted"], true);

    let create_message_response = app
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
    assert_eq!(create_message_response.status(), StatusCode::OK);
    let create_message_json = read_json(create_message_response).await;
    assert_eq!(create_message_json["id"], "msg_1");

    let list_message_response = app
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
    assert_eq!(list_message_response.status(), StatusCode::OK);
    let list_message_json = read_json(list_message_response).await;
    assert_eq!(list_message_json["data"][0]["id"], "msg_1");

    let retrieve_message_response = app
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
    assert_eq!(retrieve_message_response.status(), StatusCode::OK);
    let retrieve_message_json = read_json(retrieve_message_response).await;
    assert_eq!(retrieve_message_json["id"], "msg_1");

    let update_message_response = app
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
    assert_eq!(update_message_response.status(), StatusCode::OK);
    let update_message_json = read_json(update_message_response).await;
    assert_eq!(update_message_json["metadata"]["pinned"], "true");

    let delete_message_response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/v1/threads/thread_1/messages/msg_1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(delete_message_response.status(), StatusCode::OK);
    let delete_message_json = read_json(delete_message_response).await;
    assert_eq!(delete_message_json["deleted"], true);
}
