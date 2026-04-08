use super::*;

#[tokio::test]
async fn stateless_upload_routes_relay_to_openai_compatible_provider() {
    let upstream_state = UpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route("/v1/uploads", post(upstream_uploads_handler))
        .route(
            "/v1/uploads/upload_1/parts",
            post(upstream_upload_parts_handler),
        )
        .route(
            "/v1/uploads/upload_1/complete",
            post(upstream_upload_complete_handler),
        )
        .route(
            "/v1/uploads/upload_1/cancel",
            post(upstream_upload_cancel_handler),
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

    let upload_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/uploads")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"purpose\":\"batch\",\"filename\":\"input.jsonl\",\"mime_type\":\"application/jsonl\",\"bytes\":1024}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(upload_response.status(), StatusCode::OK);
    let upload_json = read_json(upload_response).await;
    assert_eq!(upload_json["id"], "upload_upstream");

    let part_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/uploads/upload_1/parts")
                .header(
                    "content-type",
                    "multipart/form-data; boundary=----sdkwork-upload-part",
                )
                .body(Body::from(build_upload_part_multipart_body(
                    "----sdkwork-upload-part",
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(part_response.status(), StatusCode::OK);
    let part_json = read_json(part_response).await;
    assert_eq!(part_json["id"], "part_upstream");

    let complete_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/uploads/upload_1/complete")
                .header("content-type", "application/json")
                .body(Body::from("{\"part_ids\":[\"part_1\",\"part_2\"]}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(complete_response.status(), StatusCode::OK);
    let complete_json = read_json(complete_response).await;
    assert_eq!(complete_json["part_ids"][1], "part_2");
    assert_eq!(
        upstream_state.authorization_header().as_deref(),
        Some("Bearer sk-stateless-openai")
    );
    assert!(upstream_state
        .content_type_header()
        .as_deref()
        .unwrap_or_default()
        .starts_with("multipart/form-data"));

    let cancel_response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/uploads/upload_1/cancel")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(cancel_response.status(), StatusCode::OK);
    let cancel_json = read_json(cancel_response).await;
    assert_eq!(cancel_json["status"], "cancelled");
}
