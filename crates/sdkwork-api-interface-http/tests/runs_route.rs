use axum::body::Body;
use axum::extract::State;
use axum::http::{Request, StatusCode};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde_json::Value;
use sqlx::SqlitePool;
use std::sync::{Arc, Mutex};
use tower::ServiceExt;

mod support;

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
                .body(Body::from("{\"tool_outputs\":[{\"tool_call_id\":\"call_1\",\"output\":\"{\\\"ok\\\":true}\"}]}"))
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

async fn read_json(response: axum::response::Response) -> Value {
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

async fn memory_pool() -> SqlitePool {
    sdkwork_api_storage_sqlite::run_migrations("sqlite::memory:")
        .await
        .unwrap()
}

#[derive(Clone, Default)]
struct UpstreamCaptureState {
    authorization: Arc<Mutex<Option<String>>>,
    beta: Arc<Mutex<Option<String>>>,
}

#[tokio::test]
async fn stateful_thread_runs_routes_relay_to_openai_compatible_provider() {
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

    let create_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/threads/thread_1/runs")
                .header("authorization", format!("Bearer {api_key}"))
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
        upstream_state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-upstream-openai")
    );
    assert_eq!(
        upstream_state.beta.lock().unwrap().as_deref(),
        Some("assistants=v2")
    );

    let list_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/threads/thread_1/runs")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(list_response.status(), StatusCode::OK);
    let list_json = read_json(list_response).await;
    assert_eq!(list_json["data"][0]["id"], "run_1");

    let retrieve_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/threads/thread_1/runs/run_1")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(retrieve_response.status(), StatusCode::OK);
    let retrieve_json = read_json(retrieve_response).await;
    assert_eq!(retrieve_json["id"], "run_1");

    let update_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/threads/thread_1/runs/run_1")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from("{\"metadata\":{\"priority\":\"high\"}}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(update_response.status(), StatusCode::OK);
    let update_json = read_json(update_response).await;
    assert_eq!(update_json["metadata"]["priority"], "high");

    let cancel_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/threads/thread_1/runs/run_1/cancel")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(cancel_response.status(), StatusCode::OK);
    let cancel_json = read_json(cancel_response).await;
    assert_eq!(cancel_json["status"], "cancelled");

    let submit_tool_outputs_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/threads/thread_1/runs/run_1/submit_tool_outputs")
                .header("authorization", format!("Bearer {api_key}"))
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

    let steps_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/threads/thread_1/runs/run_1/steps")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(steps_response.status(), StatusCode::OK);
    let steps_json = read_json(steps_response).await;
    assert_eq!(steps_json["data"][0]["id"], "step_1");

    let step_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/threads/thread_1/runs/run_1/steps/step_1")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(step_response.status(), StatusCode::OK);
    let step_json = read_json(step_response).await;
    assert_eq!(step_json["id"], "step_1");

    let create_and_run_response = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/threads/runs")
                .header("authorization", format!("Bearer {api_key}"))
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

async fn upstream_thread_runs_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    capture_headers(&state, &headers);
    (StatusCode::OK, Json(thread_run_json("run_1", "queued")))
}

async fn upstream_thread_runs_list_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    capture_headers(&state, &headers);
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

async fn upstream_thread_run_retrieve_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    capture_headers(&state, &headers);
    (
        StatusCode::OK,
        Json(thread_run_json("run_1", "in_progress")),
    )
}

async fn upstream_thread_run_update_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    capture_headers(&state, &headers);
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

async fn upstream_thread_run_cancel_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    capture_headers(&state, &headers);
    (StatusCode::OK, Json(thread_run_json("run_1", "cancelled")))
}

async fn upstream_thread_run_submit_tool_outputs_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    capture_headers(&state, &headers);
    (StatusCode::OK, Json(thread_run_json("run_1", "queued")))
}

async fn upstream_thread_run_steps_list_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    capture_headers(&state, &headers);
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

async fn upstream_thread_run_step_retrieve_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    capture_headers(&state, &headers);
    (StatusCode::OK, Json(thread_run_step_json("step_1")))
}

async fn upstream_thread_and_run_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    capture_headers(&state, &headers);
    (StatusCode::OK, Json(thread_run_json("run_1", "queued")))
}

fn capture_headers(state: &UpstreamCaptureState, headers: &axum::http::HeaderMap) {
    *state.authorization.lock().unwrap() = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);
    *state.beta.lock().unwrap() = headers
        .get("openai-beta")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);
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
