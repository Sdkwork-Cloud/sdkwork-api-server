use super::*;

#[tokio::test]
async fn fine_tuning_route_returns_invalid_request_without_provider() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/fine_tuning/jobs")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"training_file\":\"file_local_1\",\"model\":\"gpt-4.1-mini\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_invalid_request(
        response,
        "Local fine-tuning job fallback is not supported without an upstream provider.",
        "invalid_fine_tuning_request",
    )
    .await;
}

#[tokio::test]
async fn fine_tuning_list_route_returns_invalid_request_without_provider() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/fine_tuning/jobs")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_invalid_request(
        response,
        "Local fine-tuning job listing fallback is not supported without an upstream provider.",
        "invalid_fine_tuning_request",
    )
    .await;
}

#[tokio::test]
async fn fine_tuning_retrieve_route_returns_not_found_without_local_state() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/fine_tuning/jobs/ftjob_local_1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested fine tuning job was not found.").await;
}

#[tokio::test]
async fn fine_tuning_cancel_route_returns_not_found_without_local_state() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/fine_tuning/jobs/ftjob_local_1/cancel")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested fine tuning job was not found.").await;
}

#[tokio::test]
async fn fine_tuning_events_route_returns_invalid_request_without_local_event_state() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/fine_tuning/jobs/ftjob_local_1/events")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_invalid_request(
        response,
        "Persisted local fine tuning job event state is required for local event listing.",
        "invalid_fine_tuning_request",
    )
    .await;
}

#[tokio::test]
async fn fine_tuning_checkpoints_route_returns_invalid_request_without_local_checkpoint_state() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/fine_tuning/jobs/ftjob_local_1/checkpoints")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_invalid_request(
        response,
        "Persisted local fine tuning checkpoint state is required for local checkpoint listing.",
        "invalid_fine_tuning_request",
    )
    .await;
}

#[tokio::test]
async fn fine_tuning_pause_route_returns_not_found_without_local_state() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/fine_tuning/jobs/ftjob_local_1/pause")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested fine tuning job was not found.").await;
}

#[tokio::test]
async fn fine_tuning_resume_route_returns_not_found_without_local_state() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/fine_tuning/jobs/ftjob_local_1/resume")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested fine tuning job was not found.").await;
}

#[tokio::test]
async fn fine_tuning_checkpoint_permissions_create_route_returns_invalid_request_without_local_permission_state(
) {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/fine_tuning/checkpoints/ftckpt_local_1/permissions")
                .header("content-type", "application/json")
                .body(Body::from("{\"project_ids\":[\"project-2\"]}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_invalid_request(
        response,
        "Persisted local fine tuning checkpoint permission state is required for local permission creation.",
        "invalid_fine_tuning_request",
    )
    .await;
}

#[tokio::test]
async fn fine_tuning_checkpoint_permissions_list_route_returns_invalid_request_without_local_permission_state(
) {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/fine_tuning/checkpoints/ftckpt_local_1/permissions")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_invalid_request(
        response,
        "Persisted local fine tuning checkpoint permission state is required for local permission listing.",
        "invalid_fine_tuning_request",
    )
    .await;
}

#[tokio::test]
async fn fine_tuning_checkpoint_permission_delete_route_returns_not_found_without_local_state() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/v1/fine_tuning/checkpoints/ftckpt_local_1/permissions/perm_local_1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(
        response,
        "Requested fine tuning checkpoint permission was not found.",
    )
    .await;
}

#[tokio::test]
async fn fine_tuning_retrieve_route_returns_not_found_for_unknown_job() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/fine_tuning/jobs/ftjob_missing")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested fine tuning job was not found.").await;
}

#[tokio::test]
async fn fine_tuning_cancel_route_returns_not_found_for_unknown_job() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/fine_tuning/jobs/ftjob_missing/cancel")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested fine tuning job was not found.").await;
}

#[tokio::test]
async fn fine_tuning_events_route_returns_not_found_for_unknown_job() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/fine_tuning/jobs/ftjob_missing/events")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested fine tuning job was not found.").await;
}

#[tokio::test]
async fn fine_tuning_checkpoints_route_returns_not_found_for_unknown_job() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/fine_tuning/jobs/ftjob_missing/checkpoints")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested fine tuning job was not found.").await;
}

#[tokio::test]
async fn fine_tuning_pause_route_returns_not_found_for_unknown_job() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/fine_tuning/jobs/ftjob_missing/pause")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested fine tuning job was not found.").await;
}

#[tokio::test]
async fn fine_tuning_resume_route_returns_not_found_for_unknown_job() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/fine_tuning/jobs/ftjob_missing/resume")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested fine tuning job was not found.").await;
}

#[tokio::test]
async fn fine_tuning_checkpoint_permissions_create_route_returns_not_found_for_unknown_checkpoint()
{
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/fine_tuning/checkpoints/ft:missing:checkpoint/permissions")
                .header("content-type", "application/json")
                .body(Body::from("{\"project_ids\":[\"project-2\"]}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested fine tuning checkpoint was not found.").await;
}

#[tokio::test]
async fn fine_tuning_checkpoint_permissions_list_route_returns_not_found_for_unknown_checkpoint() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/fine_tuning/checkpoints/ft:missing:checkpoint/permissions")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(response, "Requested fine tuning checkpoint was not found.").await;
}

#[tokio::test]
async fn fine_tuning_checkpoint_permission_delete_route_returns_not_found_for_unknown_permission() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(
                    "/v1/fine_tuning/checkpoints/ft:gpt-4.1-mini:checkpoint-1/permissions/perm_missing",
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_openai_not_found(
        response,
        "Requested fine tuning checkpoint permission was not found.",
    )
    .await;
}
