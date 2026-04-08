use super::*;

#[serial(extension_env)]
#[tokio::test]
async fn stateless_videos_canonical_routes_relay_to_openai_compatible_provider() {
    let upstream_state = UpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route(
            "/v1/videos/characters",
            post(upstream_video_character_create_handler),
        )
        .route(
            "/v1/videos/characters/char_1",
            get(upstream_video_character_canonical_retrieve_handler),
        )
        .route("/v1/videos/edits", post(upstream_video_edit_handler))
        .route("/v1/videos/extensions", post(upstream_video_extensions_handler))
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

    let create_character_response = app
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
    assert_eq!(create_character_response.status(), StatusCode::OK);
    let create_character_json = read_json(create_character_response).await;
    assert_eq!(create_character_json["id"], "char_upstream");

    let retrieve_character_response = app
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
    assert_eq!(retrieve_character_response.status(), StatusCode::OK);
    let retrieve_character_json = read_json(retrieve_character_response).await;
    assert_eq!(retrieve_character_json["id"], "char_1");

    let edit_response = app
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
    assert_eq!(edit_response.status(), StatusCode::OK);
    let edit_json = read_json(edit_response).await;
    assert_eq!(edit_json["data"][0]["id"], "video_1_edited");

    let extensions_response = app
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
    assert_eq!(extensions_response.status(), StatusCode::OK);
    let extensions_json = read_json(extensions_response).await;
    assert_eq!(extensions_json["data"][0]["id"], "video_1_extended");
    assert_eq!(
        upstream_state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-stateless-openai")
    );
}

#[serial(extension_env)]
#[tokio::test]
async fn stateful_videos_canonical_routes_relay_to_openai_compatible_provider() {
    let upstream_state = UpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route(
            "/v1/videos/characters",
            post(upstream_video_character_create_handler),
        )
        .route(
            "/v1/videos/characters/char_1",
            get(upstream_video_character_canonical_retrieve_handler),
        )
        .route("/v1/videos/edits", post(upstream_video_edit_handler))
        .route("/v1/videos/extensions", post(upstream_video_extensions_handler))
        .with_state(upstream_state.clone());

    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });

    let pool = memory_pool().await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
    let api_key = support::issue_gateway_api_key(&pool, "tenant-1", "project-1").await;
    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool);

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

    let model = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/models")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"external_name\":\"sora-1\",\"provider_id\":\"provider-openai-official\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(model.status(), StatusCode::CREATED);

    let create_character_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/videos/characters")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from("{\"name\":\"Hero\",\"video_id\":\"video_1\"}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_character_response.status(), StatusCode::OK);
    let create_character_json = read_json(create_character_response).await;
    assert_eq!(create_character_json["id"], "char_upstream");

    let retrieve_character_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/videos/characters/char_1")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(retrieve_character_response.status(), StatusCode::OK);
    let retrieve_character_json = read_json(retrieve_character_response).await;
    assert_eq!(retrieve_character_json["id"], "char_1");

    let edit_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/videos/edits")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"prompt\":\"Add dramatic lighting\",\"video_id\":\"video_1\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(edit_response.status(), StatusCode::OK);
    let edit_json = read_json(edit_response).await;
    assert_eq!(edit_json["data"][0]["id"], "video_1_edited");

    let extensions_response = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/videos/extensions")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"prompt\":\"Extend the ending\",\"video_id\":\"video_1\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(extensions_response.status(), StatusCode::OK);
    let extensions_json = read_json(extensions_response).await;
    assert_eq!(extensions_json["data"][0]["id"], "video_1_extended");
    assert_eq!(
        upstream_state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-upstream-openai")
    );
}
