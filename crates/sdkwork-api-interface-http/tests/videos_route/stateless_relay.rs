use super::*;

#[serial(extension_env)]
#[tokio::test]
async fn stateless_videos_routes_relay_to_openai_compatible_provider() {
    let upstream_state = UpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route(
            "/v1/videos",
            get(upstream_videos_list_handler).post(upstream_videos_handler),
        )
        .route(
            "/v1/videos/video_1",
            get(upstream_video_retrieve_handler).delete(upstream_video_delete_handler),
        )
        .route("/v1/videos/video_1/content", get(upstream_video_content_handler))
        .route("/v1/videos/video_1/remix", post(upstream_video_remix_handler))
        .route(
            "/v1/videos/video_1/characters",
            get(upstream_video_characters_list_handler),
        )
        .route(
            "/v1/videos/video_1/characters/char_1",
            get(upstream_video_character_retrieve_handler)
                .post(upstream_video_character_update_handler),
        )
        .route("/v1/videos/video_1/extend", post(upstream_video_extend_handler))
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
                .uri("/v1/videos")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"sora-1\",\"prompt\":\"A short cinematic flyover\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_response.status(), StatusCode::OK);
    let create_json = read_json(create_response).await;
    assert_eq!(create_json["data"][0]["id"], "video_upstream");

    let list_response = app
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
    assert_eq!(list_response.status(), StatusCode::OK);
    let list_json = read_json(list_response).await;
    assert_eq!(list_json["data"][0]["id"], "video_1");

    let retrieve_response = app
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
    assert_eq!(retrieve_response.status(), StatusCode::OK);
    let retrieve_json = read_json(retrieve_response).await;
    assert_eq!(retrieve_json["id"], "video_1");

    let content_response = app
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
    assert_eq!(content_response.status(), StatusCode::OK);
    assert_eq!(read_bytes(content_response).await, b"UPSTREAM-VIDEO".to_vec());

    let remix_response = app
        .clone()
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
    assert_eq!(remix_response.status(), StatusCode::OK);
    let remix_json = read_json(remix_response).await;
    assert_eq!(remix_json["data"][0]["id"], "video_1_remix");

    let characters_response = app
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
    assert_eq!(characters_response.status(), StatusCode::OK);
    let characters_json = read_json(characters_response).await;
    assert_eq!(characters_json["data"][0]["id"], "char_1");

    let character_response = app
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
    assert_eq!(character_response.status(), StatusCode::OK);
    let character_json = read_json(character_response).await;
    assert_eq!(character_json["id"], "char_1");

    let character_update_response = app
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
    assert_eq!(character_update_response.status(), StatusCode::OK);
    let character_update_json = read_json(character_update_response).await;
    assert_eq!(character_update_json["name"], "Hero");

    let extend_response = app
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
    assert_eq!(extend_response.status(), StatusCode::OK);
    let extend_json = read_json(extend_response).await;
    assert_eq!(extend_json["data"][0]["id"], "video_1_extended");

    let delete_response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/v1/videos/video_1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(delete_response.status(), StatusCode::OK);
    let delete_json = read_json(delete_response).await;
    assert_eq!(delete_json["deleted"], true);
    assert_eq!(
        upstream_state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-stateless-openai")
    );
}
