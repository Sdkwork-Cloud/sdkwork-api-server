use super::*;

#[tokio::test]
async fn stateless_thread_runs_routes_relay_to_openai_compatible_provider() {
    let upstream_state = UpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route(
            "/v1/threads/thread_1/runs",
            get(upstream_thread_runs_list_handler).post(upstream_thread_runs_handler),
        )
        .route(
            "/v1/threads/thread_1/runs/run_1",
            get(upstream_thread_run_retrieve_handler).post(upstream_thread_run_update_handler),
        )
        .route(
            "/v1/threads/thread_1/runs/run_1/cancel",
            post(upstream_thread_run_cancel_handler),
        )
        .route(
            "/v1/threads/thread_1/runs/run_1/submit_tool_outputs",
            post(upstream_thread_run_submit_tool_outputs_handler),
        )
        .route(
            "/v1/threads/thread_1/runs/run_1/steps",
            get(upstream_thread_run_steps_list_handler),
        )
        .route(
            "/v1/threads/thread_1/runs/run_1/steps/step_1",
            get(upstream_thread_run_step_retrieve_handler),
        )
        .route("/v1/threads/runs", post(upstream_thread_and_run_handler))
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
                .uri("/v1/threads/thread_1/runs")
                .header("content-type", "application/json")
                .body(Body::from("{\"assistant_id\":\"asst_1\"}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_response.status(), StatusCode::OK);
    let create_json = read_json(create_response).await;
    assert_eq!(create_json["id"], "run_1");
    assert_eq!(
        upstream_state.authorization_header().as_deref(),
        Some("Bearer sk-stateless-openai")
    );
    assert_eq!(
        upstream_state.beta_header().as_deref(),
        Some("assistants=v2")
    );

    let list_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/threads/thread_1/runs")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(list_response.status(), StatusCode::OK);
    let list_json = read_json(list_response).await;
    assert_eq!(list_json["data"][0]["id"], "run_1");

    let retrieve_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/threads/thread_1/runs/run_1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(retrieve_response.status(), StatusCode::OK);
    let retrieve_json = read_json(retrieve_response).await;
    assert_eq!(retrieve_json["id"], "run_1");

    let update_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/threads/thread_1/runs/run_1")
                .header("content-type", "application/json")
                .body(Body::from("{\"metadata\":{\"priority\":\"high\"}}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(update_response.status(), StatusCode::OK);
    let update_json = read_json(update_response).await;
    assert_eq!(update_json["metadata"]["priority"], "high");

    let cancel_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/threads/thread_1/runs/run_1/cancel")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(cancel_response.status(), StatusCode::OK);
    let cancel_json = read_json(cancel_response).await;
    assert_eq!(cancel_json["status"], "cancelled");

    let submit_tool_outputs_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/threads/thread_1/runs/run_1/submit_tool_outputs")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"tool_outputs\":[{\"tool_call_id\":\"call_1\",\"output\":\"{\\\"ok\\\":true}\"}]}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(submit_tool_outputs_response.status(), StatusCode::OK);
    let submit_tool_outputs_json = read_json(submit_tool_outputs_response).await;
    assert_eq!(submit_tool_outputs_json["id"], "run_1");

    let steps_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/threads/thread_1/runs/run_1/steps")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(steps_response.status(), StatusCode::OK);
    let steps_json = read_json(steps_response).await;
    assert_eq!(steps_json["data"][0]["id"], "step_1");

    let step_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/threads/thread_1/runs/run_1/steps/step_1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(step_response.status(), StatusCode::OK);
    let step_json = read_json(step_response).await;
    assert_eq!(step_json["id"], "step_1");

    let create_and_run_response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/threads/runs")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"assistant_id\":\"asst_1\",\"thread\":{\"metadata\":{\"workspace\":\"default\"}}}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_and_run_response.status(), StatusCode::OK);
    let create_and_run_json = read_json(create_and_run_response).await;
    assert_eq!(create_and_run_json["id"], "run_1");
}
