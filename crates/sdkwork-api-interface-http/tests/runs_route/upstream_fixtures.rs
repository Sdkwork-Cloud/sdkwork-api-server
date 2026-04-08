use super::*;

pub(super) async fn upstream_thread_runs_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    state.capture_headers(&headers);
    (StatusCode::OK, Json(thread_run_json("run_1", "queued")))
}

pub(super) async fn upstream_thread_runs_list_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    state.capture_headers(&headers);
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "object":"list",
            "data":[thread_run_json("run_1", "queued")],
            "first_id":"run_1",
            "last_id":"run_1",
            "has_more":false
        })),
    )
}

pub(super) async fn upstream_thread_run_retrieve_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    state.capture_headers(&headers);
    (
        StatusCode::OK,
        Json(thread_run_json("run_1", "in_progress")),
    )
}

pub(super) async fn upstream_thread_run_update_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    state.capture_headers(&headers);
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "id":"run_1",
            "object":"thread.run",
            "thread_id":"thread_1",
            "assistant_id":"asst_1",
            "status":"in_progress",
            "model":"gpt-4.1",
            "metadata":{"priority":"high"}
        })),
    )
}

pub(super) async fn upstream_thread_run_cancel_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    state.capture_headers(&headers);
    (StatusCode::OK, Json(thread_run_json("run_1", "cancelled")))
}

pub(super) async fn upstream_thread_run_submit_tool_outputs_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    state.capture_headers(&headers);
    (StatusCode::OK, Json(thread_run_json("run_1", "queued")))
}

pub(super) async fn upstream_thread_run_steps_list_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    state.capture_headers(&headers);
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "object":"list",
            "data":[thread_run_step_json("step_1")],
            "first_id":"step_1",
            "last_id":"step_1",
            "has_more":false
        })),
    )
}

pub(super) async fn upstream_thread_run_step_retrieve_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    state.capture_headers(&headers);
    (StatusCode::OK, Json(thread_run_step_json("step_1")))
}

pub(super) async fn upstream_thread_and_run_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    state.capture_headers(&headers);
    (StatusCode::OK, Json(thread_run_json("run_1", "queued")))
}

fn thread_run_json(id: &str, status: &str) -> Value {
    serde_json::json!({
        "id":id,
        "object":"thread.run",
        "thread_id":"thread_1",
        "assistant_id":"asst_1",
        "status":status,
        "model":"gpt-4.1",
        "metadata":{"priority":"high"}
    })
}

fn thread_run_step_json(id: &str) -> Value {
    serde_json::json!({
        "id":id,
        "object":"thread.run.step",
        "thread_id":"thread_1",
        "assistant_id":"asst_1",
        "run_id":"run_1",
        "type":"message_creation",
        "status":"completed",
        "step_details":{
            "message_creation":{
                "message_id":"msg_1"
            }
        }
    })
}
