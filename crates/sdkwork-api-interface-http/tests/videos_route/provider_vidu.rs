use super::*;

#[derive(Clone, Default)]
struct ViduUpstreamCaptureState {
    authorizations: Arc<Mutex<Vec<String>>>,
    text2video_body: Arc<Mutex<Option<Value>>>,
    img2video_body: Arc<Mutex<Option<Value>>>,
    reference2video_body: Arc<Mutex<Option<Value>>>,
    task_creations_ids: Arc<Mutex<Vec<String>>>,
    cancel_ids: Arc<Mutex<Vec<String>>>,
}

#[derive(Clone, Default)]
struct GenericViduUpstreamCaptureState {
    hits: Arc<Mutex<usize>>,
}

#[serial(extension_env)]
#[tokio::test]
async fn stateless_video_vidu_routes_relay_to_official_paths_and_token_auth() {
    let upstream_state = ViduUpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route("/ent/v2/text2video", post(upstream_vidu_text2video_handler))
        .route("/ent/v2/img2video", post(upstream_vidu_img2video_handler))
        .route(
            "/ent/v2/reference2video",
            post(upstream_vidu_reference2video_handler),
        )
        .route(
            "/ent/v2/tasks/task_vidu_1/creations",
            get(upstream_vidu_task_creations_handler),
        )
        .route(
            "/ent/v2/tasks/task_vidu_1/cancel",
            post(upstream_vidu_task_cancel_handler),
        )
        .with_state(upstream_state.clone());

    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });

    let app = sdkwork_api_interface_http::gateway_router_with_stateless_config(
        sdkwork_api_interface_http::StatelessGatewayConfig::default().with_upstream(
            sdkwork_api_interface_http::StatelessGatewayUpstream::new_with_protocol_kind_and_identity(
                "sdkwork.provider.vidu",
                "custom",
                "vidu",
                format!("http://{address}"),
                "sk-stateless-vidu",
            ),
        ),
    );

    let text_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/ent/v2/text2video")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"viduq1\",\"prompt\":\"An astronaut walking through fog\",\"duration\":\"5\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(text_response.status(), StatusCode::OK);
    let text_json = read_json(text_response).await;
    assert_eq!(text_json["task_id"], "task_vidu_1");

    let img_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/ent/v2/img2video")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"viduq1\",\"images\":[\"https://example.com/source.png\"],\"prompt\":\"The astronaut waved\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(img_response.status(), StatusCode::OK);
    let img_json = read_json(img_response).await;
    assert_eq!(img_json["task_id"], "task_vidu_1");

    let reference_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/ent/v2/reference2video")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"viduq1\",\"images\":[\"https://example.com/1.png\",\"https://example.com/2.png\"],\"prompt\":\"Two characters hug\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(reference_response.status(), StatusCode::OK);
    let reference_json = read_json(reference_response).await;
    assert_eq!(reference_json["task_id"], "task_vidu_1");

    let get_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/ent/v2/tasks/task_vidu_1/creations")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(get_response.status(), StatusCode::OK);
    let get_json = read_json(get_response).await;
    assert_eq!(
        get_json["creations"][0]["url"],
        "https://cdn.example.com/vidu.mp4"
    );

    let cancel_response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/ent/v2/tasks/task_vidu_1/cancel")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(cancel_response.status(), StatusCode::OK);
    let cancel_json = read_json(cancel_response).await;
    assert_eq!(cancel_json, serde_json::json!({}));

    assert_eq!(
        upstream_state.authorizations.lock().unwrap().clone(),
        vec![
            "Token sk-stateless-vidu".to_owned(),
            "Token sk-stateless-vidu".to_owned(),
            "Token sk-stateless-vidu".to_owned(),
            "Token sk-stateless-vidu".to_owned(),
            "Token sk-stateless-vidu".to_owned(),
        ]
    );
    assert_eq!(
        upstream_state.text2video_body.lock().unwrap().clone(),
        Some(serde_json::json!({
            "model":"viduq1",
            "prompt":"An astronaut walking through fog",
            "duration":"5"
        }))
    );
    assert_eq!(
        upstream_state.img2video_body.lock().unwrap().clone(),
        Some(serde_json::json!({
            "model":"viduq1",
            "images":["https://example.com/source.png"],
            "prompt":"The astronaut waved"
        }))
    );
    assert_eq!(
        upstream_state.reference2video_body.lock().unwrap().clone(),
        Some(serde_json::json!({
            "model":"viduq1",
            "images":["https://example.com/1.png","https://example.com/2.png"],
            "prompt":"Two characters hug"
        }))
    );
    assert_eq!(
        upstream_state.task_creations_ids.lock().unwrap().clone(),
        vec!["task_vidu_1".to_owned()]
    );
    assert_eq!(
        upstream_state.cancel_ids.lock().unwrap().clone(),
        vec!["task_vidu_1".to_owned()]
    );
}

#[serial(extension_env)]
#[tokio::test]
async fn stateful_video_vidu_routes_use_vidu_provider_identity_and_token_auth() {
    let tenant_id = "tenant-video-vidu-stateful";
    let project_id = "project-video-vidu-stateful";

    let generic_state = GenericViduUpstreamCaptureState::default();
    let generic_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let generic_address = generic_listener.local_addr().unwrap();
    let generic_upstream = Router::new()
        .route(
            "/ent/v2/text2video",
            post(upstream_generic_vidu_text2video_handler),
        )
        .route(
            "/ent/v2/img2video",
            post(upstream_generic_vidu_img2video_handler),
        )
        .route(
            "/ent/v2/reference2video",
            post(upstream_generic_vidu_reference2video_handler),
        )
        .route(
            "/ent/v2/tasks/task_vidu_1/creations",
            get(upstream_generic_vidu_task_creations_handler),
        )
        .route(
            "/ent/v2/tasks/task_vidu_1/cancel",
            post(upstream_generic_vidu_task_cancel_handler),
        )
        .with_state(generic_state.clone());
    tokio::spawn(async move {
        axum::serve(generic_listener, generic_upstream)
            .await
            .unwrap();
    });

    let vidu_state = ViduUpstreamCaptureState::default();
    let vidu_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let vidu_address = vidu_listener.local_addr().unwrap();
    let vidu_upstream = Router::new()
        .route("/ent/v2/text2video", post(upstream_vidu_text2video_handler))
        .route("/ent/v2/img2video", post(upstream_vidu_img2video_handler))
        .route(
            "/ent/v2/reference2video",
            post(upstream_vidu_reference2video_handler),
        )
        .route(
            "/ent/v2/tasks/task_vidu_1/creations",
            get(upstream_vidu_task_creations_handler),
        )
        .route(
            "/ent/v2/tasks/task_vidu_1/cancel",
            post(upstream_vidu_task_cancel_handler),
        )
        .with_state(vidu_state.clone());
    tokio::spawn(async move {
        axum::serve(vidu_listener, vidu_upstream).await.unwrap();
    });

    let pool = memory_pool().await;
    let api_key = support::issue_gateway_api_key_in_byok_group(&pool, tenant_id, project_id).await;
    support::seed_primary_commercial_credit_account(&pool, tenant_id, project_id, &api_key).await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(&pool, admin_app.clone()).await;
    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool);

    create_channel_with_name(&admin_app, &admin_token, "openai", "OpenAI").await;
    create_channel_with_name(&admin_app, &admin_token, "vidu", "Vidu").await;
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
            "{{\"id\":\"provider-z-vidu\",\"channel_id\":\"vidu\",\"extension_id\":\"sdkwork.provider.vidu\",\"adapter_kind\":\"native-dynamic\",\"protocol_kind\":\"custom\",\"base_url\":\"http://{vidu_address}\",\"display_name\":\"Vidu Provider\"}}"
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
        "provider-z-vidu",
        "cred-vidu",
        "sk-vidu-upstream",
    )
    .await;

    for (method, path, body) in [
        (
            "POST",
            "/ent/v2/text2video",
            Some("{\"model\":\"viduq1\",\"prompt\":\"An astronaut walking through fog\",\"duration\":\"5\"}"),
        ),
        (
            "POST",
            "/ent/v2/img2video",
            Some("{\"model\":\"viduq1\",\"images\":[\"https://example.com/source.png\"],\"prompt\":\"The astronaut waved\"}"),
        ),
        (
            "POST",
            "/ent/v2/reference2video",
            Some("{\"model\":\"viduq1\",\"images\":[\"https://example.com/1.png\",\"https://example.com/2.png\"],\"prompt\":\"Two characters hug\"}"),
        ),
        ("GET", "/ent/v2/tasks/task_vidu_1/creations", None),
        ("POST", "/ent/v2/tasks/task_vidu_1/cancel", None),
    ] {
        let mut builder = Request::builder()
            .method(method)
            .uri(path)
            .header("authorization", format!("Bearer {api_key}"));
        if body.is_some() {
            builder = builder.header("content-type", "application/json");
        }
        let response = gateway_app
            .clone()
            .oneshot(
                builder
                    .body(Body::from(body.unwrap_or_default().to_owned()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    assert_eq!(*generic_state.hits.lock().unwrap(), 0);
    assert_eq!(
        vidu_state.authorizations.lock().unwrap().clone(),
        vec![
            "Token sk-vidu-upstream".to_owned(),
            "Token sk-vidu-upstream".to_owned(),
            "Token sk-vidu-upstream".to_owned(),
            "Token sk-vidu-upstream".to_owned(),
            "Token sk-vidu-upstream".to_owned(),
        ]
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
    assert_eq!(usage_records.len(), 5);
    assert!(usage_records.iter().any(|record| {
        record["model"] == "video.vidu.text2video" && record["provider"] == "provider-z-vidu"
    }));
    assert!(usage_records.iter().any(|record| {
        record["model"] == "video.vidu.img2video" && record["provider"] == "provider-z-vidu"
    }));
    assert!(usage_records.iter().any(|record| {
        record["model"] == "video.vidu.reference2video" && record["provider"] == "provider-z-vidu"
    }));
    assert!(usage_records.iter().any(|record| {
        record["model"] == "video.vidu.creations.get" && record["provider"] == "provider-z-vidu"
    }));
    assert!(usage_records.iter().any(|record| {
        record["model"] == "video.vidu.cancel" && record["provider"] == "provider-z-vidu"
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
    assert_eq!(billing_records.len(), 5);
    assert!(billing_records.iter().any(|record| {
        record["route_key"] == "video.vidu.text2video"
            && record["provider_id"] == "provider-z-vidu"
            && record["reference_id"] == "task_vidu_1"
    }));
    assert!(billing_records.iter().any(|record| {
        record["route_key"] == "video.vidu.img2video"
            && record["provider_id"] == "provider-z-vidu"
            && record["reference_id"] == "task_vidu_1"
    }));
    assert!(billing_records.iter().any(|record| {
        record["route_key"] == "video.vidu.reference2video"
            && record["provider_id"] == "provider-z-vidu"
            && record["reference_id"] == "task_vidu_1"
    }));
    assert!(billing_records.iter().any(|record| {
        record["route_key"] == "video.vidu.creations.get"
            && record["provider_id"] == "provider-z-vidu"
            && record["reference_id"] == "task_vidu_1"
    }));
    assert!(billing_records.iter().any(|record| {
        record["route_key"] == "video.vidu.cancel"
            && record["provider_id"] == "provider-z-vidu"
            && record["reference_id"] == "task_vidu_1"
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
    assert_eq!(logs.len(), 5);
    assert!(logs.iter().any(|record| {
        record["route_key"] == "video.vidu.text2video"
            && record["selected_provider_id"] == "provider-z-vidu"
    }));
    assert!(logs.iter().any(|record| {
        record["route_key"] == "video.vidu.img2video"
            && record["selected_provider_id"] == "provider-z-vidu"
    }));
    assert!(logs.iter().any(|record| {
        record["route_key"] == "video.vidu.reference2video"
            && record["selected_provider_id"] == "provider-z-vidu"
    }));
    assert!(logs.iter().any(|record| {
        record["route_key"] == "video.vidu.creations.get"
            && record["selected_provider_id"] == "provider-z-vidu"
    }));
    assert!(logs.iter().any(|record| {
        record["route_key"] == "video.vidu.cancel"
            && record["selected_provider_id"] == "provider-z-vidu"
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

async fn upstream_vidu_text2video_handler(
    State(state): State<ViduUpstreamCaptureState>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<Value>,
) -> (StatusCode, Json<Value>) {
    if let Some(value) = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
    {
        state.authorizations.lock().unwrap().push(value.to_owned());
    }
    *state.text2video_body.lock().unwrap() = Some(payload);
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "task_id":"task_vidu_1",
            "state":"created"
        })),
    )
}

async fn upstream_vidu_img2video_handler(
    State(state): State<ViduUpstreamCaptureState>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<Value>,
) -> (StatusCode, Json<Value>) {
    if let Some(value) = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
    {
        state.authorizations.lock().unwrap().push(value.to_owned());
    }
    *state.img2video_body.lock().unwrap() = Some(payload);
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "task_id":"task_vidu_1",
            "state":"created"
        })),
    )
}

async fn upstream_vidu_reference2video_handler(
    State(state): State<ViduUpstreamCaptureState>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<Value>,
) -> (StatusCode, Json<Value>) {
    if let Some(value) = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
    {
        state.authorizations.lock().unwrap().push(value.to_owned());
    }
    *state.reference2video_body.lock().unwrap() = Some(payload);
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "task_id":"task_vidu_1",
            "state":"created"
        })),
    )
}

async fn upstream_vidu_task_creations_handler(
    State(state): State<ViduUpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    if let Some(value) = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
    {
        state.authorizations.lock().unwrap().push(value.to_owned());
    }
    state
        .task_creations_ids
        .lock()
        .unwrap()
        .push("task_vidu_1".to_owned());
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "id":"task_vidu_1",
            "state":"success",
            "creations":[{
                "id":"creation_vidu_1",
                "url":"https://cdn.example.com/vidu.mp4",
                "cover_url":"https://cdn.example.com/vidu-cover.png"
            }]
        })),
    )
}

async fn upstream_vidu_task_cancel_handler(
    State(state): State<ViduUpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    if let Some(value) = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
    {
        state.authorizations.lock().unwrap().push(value.to_owned());
    }
    state
        .cancel_ids
        .lock()
        .unwrap()
        .push("task_vidu_1".to_owned());
    (StatusCode::OK, Json(serde_json::json!({})))
}

async fn upstream_generic_vidu_text2video_handler(
    State(state): State<GenericViduUpstreamCaptureState>,
) -> (StatusCode, Json<Value>) {
    *state.hits.lock().unwrap() += 1;
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "task_id":"generic_should_not_be_used"
        })),
    )
}

async fn upstream_generic_vidu_img2video_handler(
    State(state): State<GenericViduUpstreamCaptureState>,
) -> (StatusCode, Json<Value>) {
    *state.hits.lock().unwrap() += 1;
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "task_id":"generic_should_not_be_used"
        })),
    )
}

async fn upstream_generic_vidu_reference2video_handler(
    State(state): State<GenericViduUpstreamCaptureState>,
) -> (StatusCode, Json<Value>) {
    *state.hits.lock().unwrap() += 1;
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "task_id":"generic_should_not_be_used"
        })),
    )
}

async fn upstream_generic_vidu_task_creations_handler(
    State(state): State<GenericViduUpstreamCaptureState>,
) -> (StatusCode, Json<Value>) {
    *state.hits.lock().unwrap() += 1;
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "id":"generic_should_not_be_used"
        })),
    )
}

async fn upstream_generic_vidu_task_cancel_handler(
    State(state): State<GenericViduUpstreamCaptureState>,
) -> (StatusCode, Json<Value>) {
    *state.hits.lock().unwrap() += 1;
    (StatusCode::OK, Json(serde_json::json!({})))
}
