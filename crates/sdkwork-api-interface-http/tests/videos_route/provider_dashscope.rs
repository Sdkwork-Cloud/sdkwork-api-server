use super::*;
use axum::extract::{Path, State};

#[derive(Clone, Default)]
struct DashScopeVideoUpstreamCaptureState {
    authorizations: Arc<Mutex<Vec<String>>>,
    synthesis_body: Arc<Mutex<Option<Value>>>,
    async_header: Arc<Mutex<Option<String>>>,
    task_ids: Arc<Mutex<Vec<String>>>,
}

#[derive(Clone, Default)]
struct GenericDashScopeVideoUpstreamCaptureState {
    hits: Arc<Mutex<usize>>,
}

#[serial(extension_env)]
#[tokio::test]
async fn stateless_video_kling_routes_relay_to_official_dashscope_paths() {
    let upstream_state = DashScopeVideoUpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route(
            "/api/v1/services/aigc/video-generation/video-synthesis",
            post(upstream_kling_video_synthesis_handler),
        )
        .route(
            "/api/v1/tasks/{task_id}",
            get(upstream_kling_video_task_handler),
        )
        .with_state(upstream_state.clone());
    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });

    let app = sdkwork_api_interface_http::gateway_router_with_stateless_config(
        sdkwork_api_interface_http::StatelessGatewayConfig::default().with_upstream(
            sdkwork_api_interface_http::StatelessGatewayUpstream::new_with_protocol_kind_and_identity(
                "sdkwork.provider.kling",
                "custom",
                "kling",
                format!("http://{address}"),
                "sk-stateless-kling-video",
            ),
        ),
    );

    let create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/services/aigc/video-generation/video-synthesis")
                .header("content-type", "application/json")
                .header("X-DashScope-Async", "enable")
                .body(Body::from(
                    "{\"model\":\"kling/kling-v3-video-generation\",\"input\":{\"prompt\":\"一只小猫在月光下奔跑\"},\"parameters\":{\"duration\":5,\"aspect_ratio\":\"16:9\"}}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_response.status(), StatusCode::OK);
    let create_json = read_json(create_response).await;
    assert_eq!(create_json["output"]["task_id"], "task_kling_video_1");

    let task_response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/tasks/task_kling_video_1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(task_response.status(), StatusCode::OK);
    let task_json = read_json(task_response).await;
    assert_eq!(task_json["output"]["task_id"], "task_kling_video_1");
    assert_eq!(
        task_json["output"]["video_url"],
        "https://cdn.example.com/task_kling_video_1.mp4"
    );

    assert_eq!(
        upstream_state.authorizations.lock().unwrap().clone(),
        vec![
            "Bearer sk-stateless-kling-video".to_owned(),
            "Bearer sk-stateless-kling-video".to_owned(),
        ]
    );
    assert_eq!(
        upstream_state.async_header.lock().unwrap().as_deref(),
        Some("enable")
    );
    assert_eq!(
        upstream_state.synthesis_body.lock().unwrap().clone(),
        Some(serde_json::json!({
            "model":"kling/kling-v3-video-generation",
            "input":{"prompt":"一只小猫在月光下奔跑"},
            "parameters":{"duration":5,"aspect_ratio":"16:9"}
        }))
    );
    assert_eq!(
        upstream_state.task_ids.lock().unwrap().clone(),
        vec!["task_kling_video_1".to_owned()]
    );
}

#[serial(extension_env)]
#[tokio::test]
async fn stateful_video_aliyun_routes_use_dashscope_task_ownership_resolution() {
    let tenant_id = "tenant-video-aliyun-stateful";
    let project_id = "project-video-aliyun-stateful";

    let generic_state = GenericDashScopeVideoUpstreamCaptureState::default();
    let generic_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let generic_address = generic_listener.local_addr().unwrap();
    let generic_upstream = Router::new()
        .route(
            "/api/v1/services/aigc/video-generation/video-synthesis",
            post(upstream_generic_dashscope_video_synthesis_handler),
        )
        .route(
            "/api/v1/tasks/{task_id}",
            get(upstream_generic_dashscope_video_task_handler),
        )
        .with_state(generic_state.clone());
    tokio::spawn(async move {
        axum::serve(generic_listener, generic_upstream)
            .await
            .unwrap();
    });

    let kling_state = DashScopeVideoUpstreamCaptureState::default();
    let kling_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let kling_address = kling_listener.local_addr().unwrap();
    let kling_upstream = Router::new()
        .route(
            "/api/v1/services/aigc/video-generation/video-synthesis",
            post(upstream_kling_video_synthesis_handler),
        )
        .route(
            "/api/v1/tasks/{task_id}",
            get(upstream_kling_video_task_handler),
        )
        .with_state(kling_state.clone());
    tokio::spawn(async move {
        axum::serve(kling_listener, kling_upstream).await.unwrap();
    });

    let aliyun_state = DashScopeVideoUpstreamCaptureState::default();
    let aliyun_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let aliyun_address = aliyun_listener.local_addr().unwrap();
    let aliyun_upstream = Router::new()
        .route(
            "/api/v1/services/aigc/video-generation/video-synthesis",
            post(upstream_aliyun_video_synthesis_handler),
        )
        .route(
            "/api/v1/tasks/{task_id}",
            get(upstream_aliyun_video_task_handler),
        )
        .with_state(aliyun_state.clone());
    tokio::spawn(async move {
        axum::serve(aliyun_listener, aliyun_upstream).await.unwrap();
    });

    let pool = memory_pool().await;
    let api_key = support::issue_gateway_api_key_in_byok_group(&pool, tenant_id, project_id).await;
    support::seed_primary_commercial_credit_account(&pool, tenant_id, project_id, &api_key).await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(&pool, admin_app.clone()).await;
    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool);

    create_channel_with_name(&admin_app, &admin_token, "openai", "OpenAI").await;
    create_channel_with_name(&admin_app, &admin_token, "kling", "Kling").await;
    create_channel_with_name(&admin_app, &admin_token, "aliyun", "Aliyun").await;
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
            "{{\"id\":\"provider-z-kling\",\"channel_id\":\"kling\",\"extension_id\":\"sdkwork.provider.kling\",\"adapter_kind\":\"kling\",\"protocol_kind\":\"custom\",\"base_url\":\"http://{kling_address}\",\"display_name\":\"Kling Provider\"}}"
        ),
    )
    .await;
    create_provider_with_payload(
        &admin_app,
        &admin_token,
        &format!(
            "{{\"id\":\"provider-z-aliyun\",\"channel_id\":\"aliyun\",\"extension_id\":\"sdkwork.provider.aliyun\",\"adapter_kind\":\"native-dynamic\",\"protocol_kind\":\"custom\",\"base_url\":\"http://{aliyun_address}\",\"display_name\":\"Aliyun Provider\"}}"
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
        "provider-z-kling",
        "cred-kling",
        "sk-kling-upstream",
    )
    .await;
    create_credential(
        &admin_app,
        &admin_token,
        tenant_id,
        "provider-z-aliyun",
        "cred-aliyun",
        "sk-aliyun-upstream",
    )
    .await;
    create_model_binding(
        &admin_app,
        &admin_token,
        "wanx2.1-t2v-turbo",
        "provider-z-aliyun",
    )
    .await;
    create_model_binding(
        &admin_app,
        &admin_token,
        "kling/kling-v3-video-generation",
        "provider-z-kling",
    )
    .await;

    let create_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/services/aigc/video-generation/video-synthesis")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .header("X-DashScope-Async", "enable")
                .body(Body::from(
                    "{\"model\":\"wanx2.1-t2v-turbo\",\"input\":{\"prompt\":\"一头鲸鱼穿过云海\"},\"parameters\":{\"duration\":5,\"size\":\"1280*720\"}}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_response.status(), StatusCode::OK);
    let create_json = read_json(create_response).await;
    assert_eq!(create_json["output"]["task_id"], "task_aliyun_video_1");

    let task_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/tasks/task_aliyun_video_1")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(task_response.status(), StatusCode::OK);
    let task_json = read_json(task_response).await;
    assert_eq!(task_json["output"]["task_id"], "task_aliyun_video_1");
    assert_eq!(
        task_json["output"]["video_url"],
        "https://cdn.example.com/task_aliyun_video_1.mp4"
    );

    assert_eq!(*generic_state.hits.lock().unwrap(), 0);
    assert!(kling_state.authorizations.lock().unwrap().is_empty());
    assert_eq!(
        aliyun_state.authorizations.lock().unwrap().clone(),
        vec![
            "Bearer sk-aliyun-upstream".to_owned(),
            "Bearer sk-aliyun-upstream".to_owned(),
        ]
    );
    assert_eq!(
        aliyun_state.async_header.lock().unwrap().as_deref(),
        Some("enable")
    );
    assert_eq!(
        aliyun_state.synthesis_body.lock().unwrap().clone(),
        Some(serde_json::json!({
            "model":"wanx2.1-t2v-turbo",
            "input":{"prompt":"一头鲸鱼穿过云海"},
            "parameters":{"duration":5,"size":"1280*720"}
        }))
    );
    assert_eq!(
        aliyun_state.task_ids.lock().unwrap().clone(),
        vec!["task_aliyun_video_1".to_owned()]
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
        record["model"] == "video.aliyun.synthesis" && record["provider"] == "provider-z-aliyun"
    }));
    assert!(usage_records.iter().any(|record| {
        record["model"] == "provider.aliyun.tasks.get" && record["provider"] == "provider-z-aliyun"
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
        record["capability"] == "videos"
            && record["route_key"] == "video.aliyun.synthesis"
            && record["provider_id"] == "provider-z-aliyun"
            && record["reference_id"] == "task_aliyun_video_1"
    }));
    assert!(billing_records.iter().any(|record| {
        record["capability"] == "videos"
            && record["route_key"] == "provider.aliyun.tasks.get"
            && record["provider_id"] == "provider-z-aliyun"
            && record["reference_id"] == "task_aliyun_video_1"
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
        record["capability"] == "videos"
            && record["route_key"] == "video.aliyun.synthesis"
            && record["selected_provider_id"] == "provider-z-aliyun"
    }));
    assert!(logs.iter().any(|record| {
        record["capability"] == "videos"
            && record["route_key"] == "provider.aliyun.tasks.get"
            && record["selected_provider_id"] == "provider-z-aliyun"
    }));
}

async fn upstream_kling_video_synthesis_handler(
    State(state): State<DashScopeVideoUpstreamCaptureState>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<Value>,
) -> (StatusCode, Json<Value>) {
    state
        .authorizations
        .lock()
        .unwrap()
        .push(header_value(&headers, "authorization"));
    *state.async_header.lock().unwrap() = headers
        .get("X-DashScope-Async")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);
    *state.synthesis_body.lock().unwrap() = Some(payload);
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "output":{"task_id":"task_kling_video_1","task_status":"PENDING"},
            "request_id":"req-kling-video-1"
        })),
    )
}

async fn upstream_aliyun_video_synthesis_handler(
    State(state): State<DashScopeVideoUpstreamCaptureState>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<Value>,
) -> (StatusCode, Json<Value>) {
    state
        .authorizations
        .lock()
        .unwrap()
        .push(header_value(&headers, "authorization"));
    *state.async_header.lock().unwrap() = headers
        .get("X-DashScope-Async")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);
    *state.synthesis_body.lock().unwrap() = Some(payload);
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "output":{"task_id":"task_aliyun_video_1","task_status":"PENDING"},
            "request_id":"req-aliyun-video-1"
        })),
    )
}

async fn upstream_kling_video_task_handler(
    State(state): State<DashScopeVideoUpstreamCaptureState>,
    headers: axum::http::HeaderMap,
    Path(task_id): Path<String>,
) -> (StatusCode, Json<Value>) {
    state
        .authorizations
        .lock()
        .unwrap()
        .push(header_value(&headers, "authorization"));
    state.task_ids.lock().unwrap().push(task_id.clone());
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "output":{
                "task_id":task_id,
                "task_status":"SUCCEEDED",
                "video_url":"https://cdn.example.com/task_kling_video_1.mp4"
            }
        })),
    )
}

async fn upstream_aliyun_video_task_handler(
    State(state): State<DashScopeVideoUpstreamCaptureState>,
    headers: axum::http::HeaderMap,
    Path(task_id): Path<String>,
) -> (StatusCode, Json<Value>) {
    state
        .authorizations
        .lock()
        .unwrap()
        .push(header_value(&headers, "authorization"));
    state.task_ids.lock().unwrap().push(task_id.clone());
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "output":{
                "task_id":task_id,
                "task_status":"SUCCEEDED",
                "video_url":"https://cdn.example.com/task_aliyun_video_1.mp4"
            }
        })),
    )
}

async fn upstream_generic_dashscope_video_synthesis_handler(
    State(state): State<GenericDashScopeVideoUpstreamCaptureState>,
) -> (StatusCode, Json<Value>) {
    *state.hits.lock().unwrap() += 1;
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "output":{"task_id":"task_generic_video_1","task_status":"PENDING"}
        })),
    )
}

async fn upstream_generic_dashscope_video_task_handler(
    State(state): State<GenericDashScopeVideoUpstreamCaptureState>,
) -> (StatusCode, Json<Value>) {
    *state.hits.lock().unwrap() += 1;
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "output":{"task_id":"task_generic_video_1","task_status":"SUCCEEDED"}
        })),
    )
}

fn header_value(headers: &axum::http::HeaderMap, name: &str) -> String {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_owned()
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
