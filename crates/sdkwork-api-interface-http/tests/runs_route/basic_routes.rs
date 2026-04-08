use super::*;

#[tokio::test]
async fn thread_runs_routes_return_ok() {
    let app = sdkwork_api_interface_http::gateway_router();

    let create = app
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
    assert_eq!(create.status(), StatusCode::OK);

    let list = app
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
    assert_eq!(list.status(), StatusCode::OK);

    let retrieve = app
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
    assert_eq!(retrieve.status(), StatusCode::OK);

    let update = app
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
    assert_eq!(update.status(), StatusCode::OK);

    let cancel = app
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
    assert_eq!(cancel.status(), StatusCode::OK);

    let submit_tool_outputs = app
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
    assert_eq!(submit_tool_outputs.status(), StatusCode::OK);

    let steps = app
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
    assert_eq!(steps.status(), StatusCode::OK);

    let step = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/threads/thread_1/runs/run_1/steps/step_1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(step.status(), StatusCode::OK);
}

#[tokio::test]
async fn thread_and_run_route_returns_ok() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
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

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn thread_and_run_route_returns_invalid_request_for_missing_assistant_id() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/threads/runs")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"assistant_id\":\"\",\"thread\":{\"metadata\":{\"workspace\":\"default\"}}}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_invalid_thread_and_run_request(response).await;
}

#[tokio::test]
async fn thread_runs_create_route_returns_not_found_for_unknown_thread() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/threads/thread_missing/runs")
                .header("content-type", "application/json")
                .body(Body::from("{\"assistant_id\":\"asst_1\"}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested thread was not found.").await;
}

#[tokio::test]
async fn thread_runs_list_route_returns_not_found_for_unknown_thread() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/threads/thread_missing/runs")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested thread was not found.").await;
}

#[tokio::test]
async fn thread_run_retrieve_route_returns_not_found_for_unknown_run() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/threads/thread_1/runs/run_missing")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested run was not found.").await;
}

#[tokio::test]
async fn thread_run_update_route_returns_not_found_for_unknown_run() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/threads/thread_1/runs/run_missing")
                .header("content-type", "application/json")
                .body(Body::from("{\"metadata\":{\"priority\":\"high\"}}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested run was not found.").await;
}

#[tokio::test]
async fn thread_run_cancel_route_returns_not_found_for_unknown_run() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/threads/thread_1/runs/run_missing/cancel")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested run was not found.").await;
}

#[tokio::test]
async fn thread_run_submit_tool_outputs_route_returns_not_found_for_unknown_run() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/threads/thread_1/runs/run_missing/submit_tool_outputs")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"tool_outputs\":[{\"tool_call_id\":\"call_1\",\"output\":\"{\\\"ok\\\":true}\"}]}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested run was not found.").await;
}

#[tokio::test]
async fn thread_run_steps_list_route_returns_not_found_for_unknown_run() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/threads/thread_1/runs/run_missing/steps")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested run was not found.").await;
}

#[tokio::test]
async fn thread_run_step_route_returns_not_found_error_for_unknown_step() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/threads/thread_1/runs/run_1/steps/step_missing")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested run step was not found.").await;
}
