use super::*;
use axum::extract::State;

#[derive(Clone, Default)]
struct GoogleVeoUpstreamCaptureState {
    authorizations: Arc<Mutex<Vec<String>>>,
    predict_body: Arc<Mutex<Option<Value>>>,
    fetch_body: Arc<Mutex<Option<Value>>>,
}

#[derive(Clone, Default)]
struct GenericGoogleVeoUpstreamCaptureState {
    hits: Arc<Mutex<usize>>,
}

#[serial(extension_env)]
#[tokio::test]
async fn stateless_video_google_veo_routes_relay_to_official_google_paths() {
    let upstream_state = GoogleVeoUpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route(
            "/v1/projects/test-project/locations/us-central1/publishers/google/models/veo-3.0-generate-001:predictLongRunning",
            post(upstream_google_veo_predict_handler),
        )
        .route(
            "/v1/projects/test-project/locations/us-central1/publishers/google/models/veo-3.0-generate-001:fetchPredictOperation",
            post(upstream_google_veo_fetch_handler),
        )
        .with_state(upstream_state.clone());
    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });

    let app = sdkwork_api_interface_http::gateway_router_with_stateless_config(
        sdkwork_api_interface_http::StatelessGatewayConfig::default().with_upstream(
            sdkwork_api_interface_http::StatelessGatewayUpstream::new_with_protocol_kind_and_identity(
                "sdkwork.provider.google-veo",
                "custom",
                "google-veo",
                format!("http://{address}"),
                "sk-stateless-google-veo",
            ),
        ),
    );

    let predict_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/projects/test-project/locations/us-central1/publishers/google/models/veo-3.0-generate-001:predictLongRunning")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"instances\":[{\"prompt\":\"A paper boat sailing through neon rain\"}],\"parameters\":{\"sampleCount\":1}}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(predict_response.status(), StatusCode::OK);
    let predict_json = read_json(predict_response).await;
    assert_eq!(
        predict_json["name"],
        "projects/test-project/locations/us-central1/publishers/google/models/veo-3.0-generate-001/operations/op_google_veo_1"
    );

    let fetch_response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/projects/test-project/locations/us-central1/publishers/google/models/veo-3.0-generate-001:fetchPredictOperation")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"operationName\":\"projects/test-project/locations/us-central1/publishers/google/models/veo-3.0-generate-001/operations/op_google_veo_1\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(fetch_response.status(), StatusCode::OK);
    let fetch_json = read_json(fetch_response).await;
    assert_eq!(fetch_json["done"], true);
    assert_eq!(
        fetch_json["response"]["videos"][0]["uri"],
        "gs://bucket/output/video_google_veo_1.mp4"
    );

    assert_eq!(
        upstream_state.authorizations.lock().unwrap().clone(),
        vec![
            "Bearer sk-stateless-google-veo".to_owned(),
            "Bearer sk-stateless-google-veo".to_owned(),
        ]
    );
    assert_eq!(
        upstream_state.predict_body.lock().unwrap().clone(),
        Some(serde_json::json!({
            "instances":[{"prompt":"A paper boat sailing through neon rain"}],
            "parameters":{"sampleCount":1}
        }))
    );
    assert_eq!(
        upstream_state.fetch_body.lock().unwrap().clone(),
        Some(serde_json::json!({
            "operationName":"projects/test-project/locations/us-central1/publishers/google/models/veo-3.0-generate-001/operations/op_google_veo_1"
        }))
    );
}

#[serial(extension_env)]
#[tokio::test]
async fn stateful_video_google_veo_fetch_routes_use_operation_ownership_resolution() {
    let tenant_id = "tenant-video-google-veo-stateful";
    let project_id = "project-video-google-veo-stateful";

    let generic_state = GenericGoogleVeoUpstreamCaptureState::default();
    let generic_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let generic_address = generic_listener.local_addr().unwrap();
    let generic_upstream = Router::new()
        .route(
            "/v1/projects/test-project/locations/us-central1/publishers/google/models/veo-3.0-generate-001:predictLongRunning",
            post(upstream_generic_google_veo_predict_handler),
        )
        .route(
            "/v1/projects/test-project/locations/us-central1/publishers/google/models/veo-3.0-generate-001:fetchPredictOperation",
            post(upstream_generic_google_veo_fetch_handler),
        )
        .with_state(generic_state.clone());
    tokio::spawn(async move {
        axum::serve(generic_listener, generic_upstream)
            .await
            .unwrap();
    });

    let veo_state = GoogleVeoUpstreamCaptureState::default();
    let veo_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let veo_address = veo_listener.local_addr().unwrap();
    let veo_upstream = Router::new()
        .route(
            "/v1/projects/test-project/locations/us-central1/publishers/google/models/veo-3.0-generate-001:predictLongRunning",
            post(upstream_google_veo_predict_handler),
        )
        .route(
            "/v1/projects/test-project/locations/us-central1/publishers/google/models/veo-3.0-generate-001:fetchPredictOperation",
            post(upstream_google_veo_fetch_handler),
        )
        .with_state(veo_state.clone());
    tokio::spawn(async move {
        axum::serve(veo_listener, veo_upstream).await.unwrap();
    });

    let pool = memory_pool().await;
    let api_key = support::issue_gateway_api_key_in_byok_group(&pool, tenant_id, project_id).await;
    support::seed_primary_commercial_credit_account(&pool, tenant_id, project_id, &api_key).await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(&pool, admin_app.clone()).await;
    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool);

    create_channel_with_name(&admin_app, &admin_token, "openai", "OpenAI").await;
    create_channel_with_name(&admin_app, &admin_token, "google", "Google").await;
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
            "{{\"id\":\"provider-z-google-veo\",\"channel_id\":\"google\",\"extension_id\":\"sdkwork.provider.google-veo\",\"adapter_kind\":\"native-dynamic\",\"protocol_kind\":\"custom\",\"base_url\":\"http://{veo_address}\",\"display_name\":\"Google Veo Provider\"}}"
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
        "provider-z-google-veo",
        "cred-google-veo",
        "sk-google-veo-upstream",
    )
    .await;
    create_model_binding(
        &admin_app,
        &admin_token,
        "veo-3.0-generate-001",
        "provider-z-google-veo",
    )
    .await;

    let predict_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/projects/test-project/locations/us-central1/publishers/google/models/veo-3.0-generate-001:predictLongRunning")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"instances\":[{\"prompt\":\"A paper boat sailing through neon rain\"}],\"parameters\":{\"sampleCount\":1}}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(predict_response.status(), StatusCode::OK);
    let predict_json = read_json(predict_response).await;
    let operation_name = predict_json["name"].as_str().unwrap().to_owned();

    let fetch_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/projects/test-project/locations/us-central1/publishers/google/models/veo-3.0-generate-001:fetchPredictOperation")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"operationName\":\"{operation_name}\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(fetch_response.status(), StatusCode::OK);
    let fetch_json = read_json(fetch_response).await;
    assert_eq!(fetch_json["done"], true);
    assert_eq!(
        fetch_json["response"]["videos"][0]["uri"],
        "gs://bucket/output/video_google_veo_1.mp4"
    );

    assert_eq!(*generic_state.hits.lock().unwrap(), 0);
    assert_eq!(
        veo_state.authorizations.lock().unwrap().clone(),
        vec![
            "Bearer sk-google-veo-upstream".to_owned(),
            "Bearer sk-google-veo-upstream".to_owned(),
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
    assert_eq!(usage_records.len(), 2);
    assert!(usage_records.iter().any(|record| {
        record["model"] == "video.google-veo.predict_long_running"
            && record["provider"] == "provider-z-google-veo"
    }));
    assert!(usage_records.iter().any(|record| {
        record["model"] == "video.google-veo.fetch_predict_operation"
            && record["provider"] == "provider-z-google-veo"
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
        record["route_key"] == "video.google-veo.predict_long_running"
            && record["provider_id"] == "provider-z-google-veo"
            && record["reference_id"] == operation_name
    }));
    assert!(billing_records.iter().any(|record| {
        record["route_key"] == "video.google-veo.fetch_predict_operation"
            && record["provider_id"] == "provider-z-google-veo"
            && record["reference_id"] == operation_name
    }));
}

async fn upstream_google_veo_predict_handler(
    State(state): State<GoogleVeoUpstreamCaptureState>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<Value>,
) -> (StatusCode, Json<Value>) {
    if let Some(value) = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
    {
        state.authorizations.lock().unwrap().push(value.to_owned());
    }
    *state.predict_body.lock().unwrap() = Some(payload);
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "name":"projects/test-project/locations/us-central1/publishers/google/models/veo-3.0-generate-001/operations/op_google_veo_1",
            "done":false
        })),
    )
}

async fn upstream_google_veo_fetch_handler(
    State(state): State<GoogleVeoUpstreamCaptureState>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<Value>,
) -> (StatusCode, Json<Value>) {
    if let Some(value) = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
    {
        state.authorizations.lock().unwrap().push(value.to_owned());
    }
    *state.fetch_body.lock().unwrap() = Some(payload.clone());
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "name":payload["operationName"],
            "done":true,
            "response":{"videos":[{"uri":"gs://bucket/output/video_google_veo_1.mp4"}]}
        })),
    )
}

async fn upstream_generic_google_veo_predict_handler(
    State(state): State<GenericGoogleVeoUpstreamCaptureState>,
) -> (StatusCode, Json<Value>) {
    *state.hits.lock().unwrap() += 1;
    (
        StatusCode::OK,
        Json(serde_json::json!({"name":"generic-google-veo-operation","done":false})),
    )
}

async fn upstream_generic_google_veo_fetch_handler(
    State(state): State<GenericGoogleVeoUpstreamCaptureState>,
) -> (StatusCode, Json<Value>) {
    *state.hits.lock().unwrap() += 1;
    (
        StatusCode::OK,
        Json(serde_json::json!({"name":"generic-google-veo-operation","done":true})),
    )
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
