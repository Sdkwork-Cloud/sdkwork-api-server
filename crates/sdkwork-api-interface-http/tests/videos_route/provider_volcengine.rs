use super::*;
use axum::extract::{Path, State};

#[derive(Clone, Default)]
struct VolcengineVideoUpstreamCaptureState {
    authorizations: Arc<Mutex<Vec<String>>>,
    create_body: Arc<Mutex<Option<Value>>>,
    task_ids: Arc<Mutex<Vec<String>>>,
}

#[derive(Clone, Default)]
struct GenericVolcengineVideoUpstreamCaptureState {
    hits: Arc<Mutex<usize>>,
}

#[serial(extension_env)]
#[tokio::test]
async fn stateless_video_volcengine_routes_relay_to_official_paths() {
    let upstream_state = VolcengineVideoUpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route(
            "/api/v1/contents/generations/tasks",
            post(upstream_volcengine_video_task_create_handler),
        )
        .route(
            "/api/v1/contents/generations/tasks/{id}",
            get(upstream_volcengine_video_task_get_handler),
        )
        .with_state(upstream_state.clone());

    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });

    let app = sdkwork_api_interface_http::gateway_router_with_stateless_config(
        sdkwork_api_interface_http::StatelessGatewayConfig::default().with_upstream(
            sdkwork_api_interface_http::StatelessGatewayUpstream::new_with_protocol_kind_and_identity(
                "sdkwork.provider.volcengine",
                "custom",
                "volcengine",
                format!("http://{address}"),
                "sk-stateless-volcengine-video",
            ),
        ),
    );

    let create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/contents/generations/tasks")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"seedance-1-0-pro-250528\",\"content\":[{\"type\":\"text\",\"text\":\"A cinematic flying whale above the sea\"}],\"duration\":5,\"ratio\":\"16:9\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_response.status(), StatusCode::OK);
    let create_json = read_json(create_response).await;
    assert_eq!(create_json["id"], "cgt_volcengine_1");

    let get_response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/contents/generations/tasks/cgt_volcengine_1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(get_response.status(), StatusCode::OK);
    let get_json = read_json(get_response).await;
    assert_eq!(get_json["id"], "cgt_volcengine_1");
    assert_eq!(get_json["status"], "succeeded");
    assert_eq!(
        get_json["content"]["video_url"],
        "https://cdn.example.com/cgt_volcengine_1.mp4"
    );

    assert_eq!(
        upstream_state.authorizations.lock().unwrap().clone(),
        vec![
            "Bearer sk-stateless-volcengine-video".to_owned(),
            "Bearer sk-stateless-volcengine-video".to_owned(),
        ]
    );
    assert_eq!(
        upstream_state.create_body.lock().unwrap().clone(),
        Some(serde_json::json!({
            "model":"seedance-1-0-pro-250528",
            "content":[{"type":"text","text":"A cinematic flying whale above the sea"}],
            "duration":5,
            "ratio":"16:9"
        }))
    );
    assert_eq!(
        upstream_state.task_ids.lock().unwrap().clone(),
        vec!["cgt_volcengine_1".to_owned()]
    );
}

#[serial(extension_env)]
#[tokio::test]
async fn stateful_video_volcengine_routes_use_model_selection_and_task_ownership_resolution() {
    let tenant_id = "tenant-video-volcengine-stateful";
    let project_id = "project-video-volcengine-stateful";

    let generic_state = GenericVolcengineVideoUpstreamCaptureState::default();
    let generic_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let generic_address = generic_listener.local_addr().unwrap();
    let generic_upstream = Router::new()
        .route(
            "/api/v1/contents/generations/tasks",
            post(upstream_generic_volcengine_video_task_create_handler),
        )
        .route(
            "/api/v1/contents/generations/tasks/{id}",
            get(upstream_generic_volcengine_video_task_get_handler),
        )
        .with_state(generic_state.clone());
    tokio::spawn(async move {
        axum::serve(generic_listener, generic_upstream)
            .await
            .unwrap();
    });

    let volcengine_state = VolcengineVideoUpstreamCaptureState::default();
    let volcengine_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let volcengine_address = volcengine_listener.local_addr().unwrap();
    let volcengine_upstream = Router::new()
        .route(
            "/api/v1/contents/generations/tasks",
            post(upstream_volcengine_video_task_create_handler),
        )
        .route(
            "/api/v1/contents/generations/tasks/{id}",
            get(upstream_volcengine_video_task_get_handler),
        )
        .with_state(volcengine_state.clone());
    tokio::spawn(async move {
        axum::serve(volcengine_listener, volcengine_upstream)
            .await
            .unwrap();
    });

    let pool = memory_pool().await;
    let api_key = support::issue_gateway_api_key_in_byok_group(&pool, tenant_id, project_id).await;
    support::seed_primary_commercial_credit_account(&pool, tenant_id, project_id, &api_key).await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(&pool, admin_app.clone()).await;
    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool);

    create_channel_with_name(&admin_app, &admin_token, "openai", "OpenAI").await;
    create_channel_with_name(&admin_app, &admin_token, "volcengine", "Volcengine").await;
    create_provider_with_payload(
        &admin_app,
        &admin_token,
        &format!(
            "{{\"id\":\"provider-a-generic\",\"channel_id\":\"openai\",\"adapter_kind\":\"openai\",\"base_url\":\"http://{generic_address}\",\"display_name\":\"Generic Provider\"}}"
        ),
    )
    .await;
    create_provider_with_payload(
        &admin_app,
        &admin_token,
        &format!(
            "{{\"id\":\"provider-z-volcengine\",\"channel_id\":\"volcengine\",\"extension_id\":\"sdkwork.provider.volcengine\",\"adapter_kind\":\"native-dynamic\",\"protocol_kind\":\"custom\",\"base_url\":\"http://{volcengine_address}\",\"display_name\":\"Volcengine Provider\"}}"
        ),
    )
    .await;
    create_credential(
        &admin_app,
        &admin_token,
        tenant_id,
        "provider-a-generic",
        "cred-generic",
        "sk-generic-upstream",
    )
    .await;
    create_credential(
        &admin_app,
        &admin_token,
        tenant_id,
        "provider-z-volcengine",
        "cred-volcengine",
        "sk-volcengine-upstream",
    )
    .await;
    create_model_binding(
        &admin_app,
        &admin_token,
        "seedance-1-0-pro-250528",
        "provider-z-volcengine",
    )
    .await;

    let create_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/contents/generations/tasks")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"seedance-1-0-pro-250528\",\"content\":[{\"type\":\"text\",\"text\":\"A cinematic flying whale above the sea\"}],\"duration\":5,\"ratio\":\"16:9\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_response.status(), StatusCode::OK);
    let create_json = read_json(create_response).await;
    assert_eq!(create_json["id"], "cgt_volcengine_1");

    let get_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/contents/generations/tasks/cgt_volcengine_1")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(get_response.status(), StatusCode::OK);
    let get_json = read_json(get_response).await;
    assert_eq!(get_json["id"], "cgt_volcengine_1");
    assert_eq!(
        get_json["content"]["video_url"],
        "https://cdn.example.com/cgt_volcengine_1.mp4"
    );

    assert_eq!(*generic_state.hits.lock().unwrap(), 0);
    assert_eq!(
        volcengine_state.authorizations.lock().unwrap().clone(),
        vec![
            "Bearer sk-volcengine-upstream".to_owned(),
            "Bearer sk-volcengine-upstream".to_owned(),
        ]
    );
    assert_eq!(
        volcengine_state.create_body.lock().unwrap().clone(),
        Some(serde_json::json!({
            "model":"seedance-1-0-pro-250528",
            "content":[{"type":"text","text":"A cinematic flying whale above the sea"}],
            "duration":5,
            "ratio":"16:9"
        }))
    );
    assert_eq!(
        volcengine_state.task_ids.lock().unwrap().clone(),
        vec!["cgt_volcengine_1".to_owned()]
    );

    let usage = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/usage/records")
                .header("authorization", format!("Bearer {admin_token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(usage.status(), StatusCode::OK);
    let usage_json = read_json(usage).await;
    let usage_records = usage_json.as_array().unwrap();
    assert_eq!(usage_records.len(), 2);
    assert!(usage_records.iter().any(|record| {
        record["model"] == "video.volcengine.tasks.create"
            && record["provider"] == "provider-z-volcengine"
    }));
    assert!(usage_records.iter().any(|record| {
        record["model"] == "video.volcengine.tasks.get"
            && record["provider"] == "provider-z-volcengine"
    }));

    let billing_events = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/billing/events")
                .header("authorization", format!("Bearer {admin_token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(billing_events.status(), StatusCode::OK);
    let billing_json = read_json(billing_events).await;
    let billing_records = billing_json.as_array().unwrap();
    assert_eq!(billing_records.len(), 2);
    assert!(billing_records.iter().any(|record| {
        record["route_key"] == "video.volcengine.tasks.create"
            && record["provider_id"] == "provider-z-volcengine"
            && record["reference_id"] == "cgt_volcengine_1"
    }));
    assert!(billing_records.iter().any(|record| {
        record["route_key"] == "video.volcengine.tasks.get"
            && record["provider_id"] == "provider-z-volcengine"
            && record["reference_id"] == "cgt_volcengine_1"
    }));

    let logs = admin_app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/routing/decision-logs")
                .header("authorization", format!("Bearer {admin_token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(logs.status(), StatusCode::OK);
    let logs_json = read_json(logs).await;
    let logs = logs_json.as_array().unwrap();
    assert_eq!(logs.len(), 2);
    assert!(logs.iter().any(|record| {
        record["route_key"] == "video.volcengine.tasks.create"
            && record["selected_provider_id"] == "provider-z-volcengine"
    }));
    assert!(logs.iter().any(|record| {
        record["route_key"] == "video.volcengine.tasks.get"
            && record["selected_provider_id"] == "provider-z-volcengine"
    }));
}

async fn create_channel_with_name(
    admin_app: &Router,
    admin_token: &str,
    channel_id: &str,
    name: &str,
) {
    let response = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/channels")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"id\":\"{channel_id}\",\"name\":\"{name}\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
}

async fn create_provider_with_payload(admin_app: &Router, admin_token: &str, payload: &str) {
    let response = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(payload.to_owned()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
}

async fn create_credential(
    admin_app: &Router,
    admin_token: &str,
    tenant_id: &str,
    provider_id: &str,
    key_reference: &str,
    secret_value: &str,
) {
    let response = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/credentials")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"tenant_id\":\"{tenant_id}\",\"provider_id\":\"{provider_id}\",\"key_reference\":\"{key_reference}\",\"secret_value\":\"{secret_value}\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
}

async fn create_model_binding(
    admin_app: &Router,
    admin_token: &str,
    external_name: &str,
    provider_id: &str,
) {
    let response = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/models")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"external_name\":\"{external_name}\",\"provider_id\":\"{provider_id}\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
}

async fn upstream_volcengine_video_task_create_handler(
    State(state): State<VolcengineVideoUpstreamCaptureState>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<Value>,
) -> (StatusCode, Json<Value>) {
    if let Some(value) = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
    {
        state.authorizations.lock().unwrap().push(value.to_owned());
    }
    *state.create_body.lock().unwrap() = Some(payload);
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "id":"cgt_volcengine_1",
            "status":"queued"
        })),
    )
}

async fn upstream_volcengine_video_task_get_handler(
    State(state): State<VolcengineVideoUpstreamCaptureState>,
    headers: axum::http::HeaderMap,
    Path(id): Path<String>,
) -> (StatusCode, Json<Value>) {
    if let Some(value) = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
    {
        state.authorizations.lock().unwrap().push(value.to_owned());
    }
    state.task_ids.lock().unwrap().push(id.clone());
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "id":id,
            "status":"succeeded",
            "content":{"video_url":"https://cdn.example.com/cgt_volcengine_1.mp4"}
        })),
    )
}

async fn upstream_generic_volcengine_video_task_create_handler(
    State(state): State<GenericVolcengineVideoUpstreamCaptureState>,
) -> (StatusCode, Json<Value>) {
    *state.hits.lock().unwrap() += 1;
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "id":"generic_should_not_be_used"
        })),
    )
}

async fn upstream_generic_volcengine_video_task_get_handler(
    State(state): State<GenericVolcengineVideoUpstreamCaptureState>,
) -> (StatusCode, Json<Value>) {
    *state.hits.lock().unwrap() += 1;
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "id":"generic_should_not_be_used"
        })),
    )
}
