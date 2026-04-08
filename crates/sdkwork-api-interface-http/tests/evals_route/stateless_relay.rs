use super::*;

#[tokio::test]
async fn stateless_evals_route_relays_to_openai_compatible_provider() {
    let upstream_state = UpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route(
            "/v1/evals",
            get(upstream_evals_list_handler).post(upstream_evals_handler),
        )
        .route(
            "/v1/evals/eval_1",
            get(upstream_eval_retrieve_handler)
                .post(upstream_eval_update_handler)
                .delete(upstream_eval_delete_handler),
        )
        .route(
            "/v1/evals/eval_1/runs",
            get(upstream_eval_runs_list_handler).post(upstream_eval_runs_create_handler),
        )
        .route(
            "/v1/evals/eval_1/runs/run_1",
            get(upstream_eval_run_retrieve_handler),
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

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/evals")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"name\":\"qa-benchmark\",\"data_source_config\":{\"type\":\"file\",\"file_id\":\"file_1\"}}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["id"], "eval_upstream");
    assert_eq!(
        upstream_state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-stateless-openai")
    );

    let list_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/evals")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(list_response.status(), StatusCode::OK);
    let list_json = read_json(list_response).await;
    assert_eq!(list_json["data"][0]["id"], "eval_1");

    let retrieve_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/evals/eval_1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(retrieve_response.status(), StatusCode::OK);
    let retrieve_json = read_json(retrieve_response).await;
    assert_eq!(retrieve_json["id"], "eval_1");

    let update_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/evals/eval_1")
                .header("content-type", "application/json")
                .body(Body::from("{\"name\":\"qa-benchmark-updated\"}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(update_response.status(), StatusCode::OK);
    let update_json = read_json(update_response).await;
    assert_eq!(update_json["name"], "qa-benchmark-updated");

    let runs_list_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/evals/eval_1/runs")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(runs_list_response.status(), StatusCode::OK);
    let runs_list_json = read_json(runs_list_response).await;
    assert_eq!(runs_list_json["data"][0]["id"], "run_1");

    let run_create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/evals/eval_1/runs")
                .header("content-type", "application/json")
                .body(Body::from("{\"name\":\"daily-regression\"}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(run_create_response.status(), StatusCode::OK);
    let run_create_json = read_json(run_create_response).await;
    assert_eq!(run_create_json["id"], "run_upstream");

    let run_retrieve_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/evals/eval_1/runs/run_1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(run_retrieve_response.status(), StatusCode::OK);
    let run_retrieve_json = read_json(run_retrieve_response).await;
    assert_eq!(run_retrieve_json["id"], "run_1");

    let delete_response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/v1/evals/eval_1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(delete_response.status(), StatusCode::OK);
    let delete_json = read_json(delete_response).await;
    assert_eq!(delete_json["deleted"], true);
}
