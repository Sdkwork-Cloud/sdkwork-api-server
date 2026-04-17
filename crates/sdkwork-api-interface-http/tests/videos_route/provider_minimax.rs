use super::*;
use axum::extract::Query;
use std::collections::HashMap;

#[derive(Clone, Default)]
struct MiniMaxVideoUpstreamCaptureState {
    authorizations: Arc<Mutex<Vec<String>>>,
    generation_body: Arc<Mutex<Option<Value>>>,
    query_task_ids: Arc<Mutex<Vec<String>>>,
    retrieve_file_ids: Arc<Mutex<Vec<String>>>,
}

#[derive(Clone, Default)]
struct GenericMiniMaxVideoUpstreamCaptureState {
    hits: Arc<Mutex<usize>>,
}

#[serial(extension_env)]
#[tokio::test]
async fn stateless_video_minimax_routes_relay_to_official_paths() {
    let upstream_state = MiniMaxVideoUpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route(
            "/v1/video_generation",
            post(upstream_minimax_video_generation_handler),
        )
        .route(
            "/v1/query/video_generation",
            get(upstream_minimax_video_generation_query_handler),
        )
        .route(
            "/v1/files/retrieve",
            get(upstream_minimax_file_retrieve_handler),
        )
        .with_state(upstream_state.clone());

    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });

    let app = sdkwork_api_interface_http::gateway_router_with_stateless_config(
        sdkwork_api_interface_http::StatelessGatewayConfig::default().with_upstream(
            sdkwork_api_interface_http::StatelessGatewayUpstream::new_with_protocol_kind_and_identity(
                "sdkwork.provider.minimax",
                "custom",
                "minimax",
                format!("http://{address}"),
                "sk-stateless-minimax-video",
            ),
        ),
    );

    let create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/video_generation")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"MiniMax-Hailuo-2.3\",\"prompt\":\"A lighthouse in a storm\",\"duration\":6,\"resolution\":\"1080P\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_response.status(), StatusCode::OK);
    let create_json = read_json(create_response).await;
    assert_eq!(create_json["task_id"], "task_video_minimax_1");

    let query_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/query/video_generation?task_id=task_video_minimax_1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(query_response.status(), StatusCode::OK);
    let query_json = read_json(query_response).await;
    assert_eq!(query_json["file_id"], "file_video_minimax_1");

    let retrieve_response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/files/retrieve?file_id=file_video_minimax_1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(retrieve_response.status(), StatusCode::OK);
    let retrieve_json = read_json(retrieve_response).await;
    assert_eq!(
        retrieve_json["file"]["download_url"],
        "https://cdn.example.com/video.mp4"
    );

    assert_eq!(
        upstream_state.authorizations.lock().unwrap().clone(),
        vec![
            "Bearer sk-stateless-minimax-video".to_owned(),
            "Bearer sk-stateless-minimax-video".to_owned(),
            "Bearer sk-stateless-minimax-video".to_owned(),
        ]
    );
    assert_eq!(
        upstream_state.generation_body.lock().unwrap().clone(),
        Some(serde_json::json!({
            "model":"MiniMax-Hailuo-2.3",
            "prompt":"A lighthouse in a storm",
            "duration":6,
            "resolution":"1080P"
        }))
    );
    assert_eq!(
        upstream_state.query_task_ids.lock().unwrap().clone(),
        vec!["task_video_minimax_1".to_owned()]
    );
    assert_eq!(
        upstream_state.retrieve_file_ids.lock().unwrap().clone(),
        vec!["file_video_minimax_1".to_owned()]
    );
}

#[serial(extension_env)]
#[tokio::test]
async fn stateful_video_minimax_routes_use_minimax_provider_identity() {
    let tenant_id = "tenant-video-minimax-stateful";
    let project_id = "project-video-minimax-stateful";

    let generic_state = GenericMiniMaxVideoUpstreamCaptureState::default();
    let generic_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let generic_address = generic_listener.local_addr().unwrap();
    let generic_upstream = Router::new()
        .route(
            "/v1/video_generation",
            post(upstream_generic_minimax_video_generation_handler),
        )
        .route(
            "/v1/query/video_generation",
            get(upstream_generic_minimax_video_generation_query_handler),
        )
        .route(
            "/v1/files/retrieve",
            get(upstream_generic_minimax_file_retrieve_handler),
        )
        .with_state(generic_state.clone());
    tokio::spawn(async move {
        axum::serve(generic_listener, generic_upstream)
            .await
            .unwrap();
    });

    let minimax_state = MiniMaxVideoUpstreamCaptureState::default();
    let minimax_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let minimax_address = minimax_listener.local_addr().unwrap();
    let minimax_upstream = Router::new()
        .route(
            "/v1/video_generation",
            post(upstream_minimax_video_generation_handler),
        )
        .route(
            "/v1/query/video_generation",
            get(upstream_minimax_video_generation_query_handler),
        )
        .route(
            "/v1/files/retrieve",
            get(upstream_minimax_file_retrieve_handler),
        )
        .with_state(minimax_state.clone());
    tokio::spawn(async move {
        axum::serve(minimax_listener, minimax_upstream)
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
    create_channel_with_name(&admin_app, &admin_token, "minimax", "MiniMax").await;
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
            "{{\"id\":\"provider-z-minimax\",\"channel_id\":\"minimax\",\"extension_id\":\"sdkwork.provider.minimax\",\"adapter_kind\":\"native-dynamic\",\"protocol_kind\":\"custom\",\"base_url\":\"http://{minimax_address}\",\"display_name\":\"MiniMax Provider\"}}"
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
        "provider-z-minimax",
        "cred-minimax",
        "sk-minimax-upstream",
    )
    .await;

    let create_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/video_generation")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"MiniMax-Hailuo-2.3\",\"prompt\":\"A lighthouse in a storm\",\"duration\":6,\"resolution\":\"1080P\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_response.status(), StatusCode::OK);
    let create_json = read_json(create_response).await;
    assert_eq!(create_json["task_id"], "task_video_minimax_1");

    let query_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/query/video_generation?task_id=task_video_minimax_1")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(query_response.status(), StatusCode::OK);
    let query_json = read_json(query_response).await;
    assert_eq!(query_json["file_id"], "file_video_minimax_1");

    let retrieve_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/files/retrieve?file_id=file_video_minimax_1")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(retrieve_response.status(), StatusCode::OK);
    let retrieve_json = read_json(retrieve_response).await;
    assert_eq!(retrieve_json["file"]["file_id"], "file_video_minimax_1");

    assert_eq!(*generic_state.hits.lock().unwrap(), 0);
    assert_eq!(
        minimax_state.authorizations.lock().unwrap().clone(),
        vec![
            "Bearer sk-minimax-upstream".to_owned(),
            "Bearer sk-minimax-upstream".to_owned(),
            "Bearer sk-minimax-upstream".to_owned(),
        ]
    );
    assert_eq!(
        minimax_state.generation_body.lock().unwrap().clone(),
        Some(serde_json::json!({
            "model":"MiniMax-Hailuo-2.3",
            "prompt":"A lighthouse in a storm",
            "duration":6,
            "resolution":"1080P"
        }))
    );
    assert_eq!(
        minimax_state.query_task_ids.lock().unwrap().clone(),
        vec!["task_video_minimax_1".to_owned()]
    );
    assert_eq!(
        minimax_state.retrieve_file_ids.lock().unwrap().clone(),
        vec!["file_video_minimax_1".to_owned()]
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
    assert_eq!(usage_records.len(), 3);
    assert!(usage_records.iter().any(|record| {
        record["model"] == "video.minimax.generate" && record["provider"] == "provider-z-minimax"
    }));
    assert!(usage_records.iter().any(|record| {
        record["model"] == "video.minimax.query" && record["provider"] == "provider-z-minimax"
    }));
    assert!(usage_records.iter().any(|record| {
        record["model"] == "video.minimax.files.retrieve"
            && record["provider"] == "provider-z-minimax"
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
    assert_eq!(billing_records.len(), 3);
    assert!(billing_records.iter().any(|record| {
        record["route_key"] == "video.minimax.generate"
            && record["provider_id"] == "provider-z-minimax"
            && record["reference_id"] == "task_video_minimax_1"
    }));
    assert!(billing_records.iter().any(|record| {
        record["route_key"] == "video.minimax.query"
            && record["provider_id"] == "provider-z-minimax"
            && record["reference_id"] == "task_video_minimax_1"
    }));
    assert!(billing_records.iter().any(|record| {
        record["route_key"] == "video.minimax.files.retrieve"
            && record["provider_id"] == "provider-z-minimax"
            && record["reference_id"] == "file_video_minimax_1"
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
    assert_eq!(logs.len(), 3);
    assert!(logs.iter().any(|record| {
        record["route_key"] == "video.minimax.generate"
            && record["selected_provider_id"] == "provider-z-minimax"
    }));
    assert!(logs.iter().any(|record| {
        record["route_key"] == "video.minimax.query"
            && record["selected_provider_id"] == "provider-z-minimax"
    }));
    assert!(logs.iter().any(|record| {
        record["route_key"] == "video.minimax.files.retrieve"
            && record["selected_provider_id"] == "provider-z-minimax"
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

async fn upstream_minimax_video_generation_handler(
    State(state): State<MiniMaxVideoUpstreamCaptureState>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<Value>,
) -> (StatusCode, Json<Value>) {
    if let Some(value) = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
    {
        state.authorizations.lock().unwrap().push(value.to_owned());
    }
    *state.generation_body.lock().unwrap() = Some(payload);
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "task_id":"task_video_minimax_1",
            "base_resp":{"status_code":0,"status_msg":"success"}
        })),
    )
}

async fn upstream_minimax_video_generation_query_handler(
    State(state): State<MiniMaxVideoUpstreamCaptureState>,
    headers: axum::http::HeaderMap,
    Query(query): Query<HashMap<String, String>>,
) -> (StatusCode, Json<Value>) {
    if let Some(value) = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
    {
        state.authorizations.lock().unwrap().push(value.to_owned());
    }
    if let Some(task_id) = query.get("task_id") {
        state
            .query_task_ids
            .lock()
            .unwrap()
            .push(task_id.to_owned());
    }
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "task_id": query.get("task_id").cloned().unwrap_or_default(),
            "status":"Success",
            "file_id":"file_video_minimax_1",
            "video_width":1920,
            "video_height":1080,
            "base_resp":{"status_code":0,"status_msg":"success"}
        })),
    )
}

async fn upstream_minimax_file_retrieve_handler(
    State(state): State<MiniMaxVideoUpstreamCaptureState>,
    headers: axum::http::HeaderMap,
    Query(query): Query<HashMap<String, String>>,
) -> (StatusCode, Json<Value>) {
    if let Some(value) = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
    {
        state.authorizations.lock().unwrap().push(value.to_owned());
    }
    if let Some(file_id) = query.get("file_id") {
        state
            .retrieve_file_ids
            .lock()
            .unwrap()
            .push(file_id.to_owned());
    }
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "file":{
                "file_id": query.get("file_id").cloned().unwrap_or_default(),
                "bytes": 1024,
                "created_at": 1700469398,
                "filename":"output_aigc.mp4",
                "purpose":"video_generation",
                "download_url":"https://cdn.example.com/video.mp4"
            },
            "base_resp":{"status_code":0,"status_msg":"success"}
        })),
    )
}

async fn upstream_generic_minimax_video_generation_handler(
    State(state): State<GenericMiniMaxVideoUpstreamCaptureState>,
) -> (StatusCode, Json<Value>) {
    *state.hits.lock().unwrap() += 1;
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "task_id":"generic_should_not_be_used"
        })),
    )
}

async fn upstream_generic_minimax_video_generation_query_handler(
    State(state): State<GenericMiniMaxVideoUpstreamCaptureState>,
) -> (StatusCode, Json<Value>) {
    *state.hits.lock().unwrap() += 1;
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "file_id":"generic_should_not_be_used"
        })),
    )
}

async fn upstream_generic_minimax_file_retrieve_handler(
    State(state): State<GenericMiniMaxVideoUpstreamCaptureState>,
) -> (StatusCode, Json<Value>) {
    *state.hits.lock().unwrap() += 1;
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "file":{"file_id":"generic_should_not_be_used"}
        })),
    )
}
