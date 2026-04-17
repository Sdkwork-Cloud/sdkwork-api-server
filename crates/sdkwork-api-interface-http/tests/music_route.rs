use axum::body::Body;
use axum::extract::{Query, State};
use axum::http::{Request, StatusCode};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde_json::Value;
use serial_test::serial;
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tower::ServiceExt;

mod support;

async fn issue_funded_gateway_api_key(
    pool: &SqlitePool,
    tenant_id: &str,
    project_id: &str,
) -> String {
    let api_key = support::issue_gateway_api_key(pool, tenant_id, project_id).await;
    support::seed_primary_commercial_credit_account(pool, tenant_id, project_id, &api_key).await;
    api_key
}

#[serial(extension_env)]
#[tokio::test]
async fn local_music_routes_follow_truthful_fallback_contract() {
    let app = sdkwork_api_interface_http::gateway_router();
    let track_id = "music_local_0000000000000001";

    let create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/music")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"suno-v4\",\"prompt\":\"Write an uplifting electronic anthem\",\"duration_seconds\":123.0}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_invalid_music_request(
        create_response,
        "Local music fallback is not supported without an upstream provider.",
    )
    .await;

    let list_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/music")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_invalid_music_request(
        list_response,
        "Local music listing fallback is not supported without an upstream provider.",
    )
    .await;

    let retrieve_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/v1/music/{track_id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_music_not_found(retrieve_response, "Requested music was not found.").await;

    let content_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/v1/music/{track_id}/content"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_music_not_found(content_response, "Requested music asset was not found.").await;

    let lyrics_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/music/lyrics")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"prompt\":\"Write uplifting synth-pop lyrics\",\"title\":\"Skyline Pulse\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_invalid_music_request(
        lyrics_response,
        "Local music lyrics fallback is not supported without a lyrics generation backend.",
    )
    .await;

    let delete_response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/v1/music/{track_id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_music_not_found(delete_response, "Requested music was not found.").await;
}

#[serial(extension_env)]
#[tokio::test]
async fn music_content_route_returns_not_found_error_for_unknown_track() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/music/music_missing/content")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let json = read_json(response).await;
    assert_eq!(
        json["error"]["message"],
        "Requested music asset was not found."
    );
    assert_eq!(json["error"]["type"], "invalid_request_error");
    assert_eq!(json["error"]["code"], "not_found");
}

#[serial(extension_env)]
#[tokio::test]
async fn music_retrieve_route_returns_not_found_error_for_unknown_track() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/music/music_missing")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_music_not_found(response, "Requested music was not found.").await;
}

#[serial(extension_env)]
#[tokio::test]
async fn music_delete_route_returns_not_found_error_for_unknown_track() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/v1/music/music_missing")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_music_not_found(response, "Requested music was not found.").await;
}

#[serial(extension_env)]
#[tokio::test]
async fn stateless_music_routes_relay_to_openai_compatible_provider() {
    let upstream_state = UpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route(
            "/v1/music",
            get(upstream_music_list_handler).post(upstream_music_create_handler),
        )
        .route(
            "/v1/music/music_1",
            get(upstream_music_retrieve_handler).delete(upstream_music_delete_handler),
        )
        .route(
            "/v1/music/music_1/content",
            get(upstream_music_content_handler),
        )
        .route("/v1/music/lyrics", post(upstream_music_lyrics_handler))
        .with_state(upstream_state.clone());

    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });

    let app = sdkwork_api_interface_http::gateway_router_with_stateless_config(
        sdkwork_api_interface_http::StatelessGatewayConfig::default().with_upstream(
            sdkwork_api_interface_http::StatelessGatewayUpstream::new(
                "openai",
                format!("http://{address}"),
                "sk-stateless-music",
            ),
        ),
    );

    let create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/music")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"suno-v4\",\"prompt\":\"Write an uplifting electronic anthem\",\"duration_seconds\":123.0}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_response.status(), StatusCode::OK);
    let create_json = read_json(create_response).await;
    assert_eq!(create_json["data"][0]["id"], "music_upstream");

    let list_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/music")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(list_response.status(), StatusCode::OK);
    let list_json = read_json(list_response).await;
    assert_eq!(list_json["data"][0]["id"], "music_1");

    let retrieve_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/music/music_1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(retrieve_response.status(), StatusCode::OK);
    let retrieve_json = read_json(retrieve_response).await;
    assert_eq!(retrieve_json["id"], "music_1");

    let content_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/music/music_1/content")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(content_response.status(), StatusCode::OK);
    assert_eq!(
        read_bytes(content_response).await,
        b"UPSTREAM-MUSIC".to_vec()
    );

    let lyrics_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/music/lyrics")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"prompt\":\"Write uplifting synth-pop lyrics\",\"title\":\"Skyline Pulse\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(lyrics_response.status(), StatusCode::OK);
    let lyrics_json = read_json(lyrics_response).await;
    assert_eq!(lyrics_json["id"], "lyrics_1");
    assert_eq!(lyrics_json["object"], "music.lyrics");

    let delete_response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/v1/music/music_1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(delete_response.status(), StatusCode::OK);
    let delete_json = read_json(delete_response).await;
    assert_eq!(delete_json["deleted"], true);

    assert_eq!(
        upstream_state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-stateless-music")
    );
}

#[serial(extension_env)]
#[tokio::test]
async fn stateless_music_suno_routes_relay_to_official_suno_paths() {
    let upstream_state = SunoUpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route("/api/v1/generate", post(upstream_suno_generate_handler))
        .route(
            "/api/v1/generate/record-info",
            get(upstream_suno_generate_record_info_handler),
        )
        .route("/api/v1/lyrics", post(upstream_suno_lyrics_create_handler))
        .route(
            "/api/v1/lyrics/record-info",
            get(upstream_suno_lyrics_record_info_handler),
        )
        .with_state(upstream_state.clone());

    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });

    let app = sdkwork_api_interface_http::gateway_router_with_stateless_config(
        sdkwork_api_interface_http::StatelessGatewayConfig::default().with_upstream(
            sdkwork_api_interface_http::StatelessGatewayUpstream::new_with_protocol_kind_and_identity(
                "sdkwork.provider.suno.relay",
                "custom",
                "suno",
                format!("http://{address}"),
                "sk-stateless-suno",
            ),
        ),
    );

    let create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/generate")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"prompt\":\"Write an uplifting electronic anthem\",\"mv\":\"chirp-v4\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_response.status(), StatusCode::OK);
    let create_json = read_json(create_response).await;
    assert_eq!(create_json["taskId"], "task_music_1");

    let generate_record_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/generate/record-info?taskId=task_music_1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(generate_record_response.status(), StatusCode::OK);
    let generate_record_json = read_json(generate_record_response).await;
    assert_eq!(generate_record_json["taskId"], "task_music_1");

    let lyrics_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/lyrics")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"prompt\":\"Write uplifting synth-pop lyrics\",\"title\":\"Skyline Pulse\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(lyrics_response.status(), StatusCode::OK);
    let lyrics_json = read_json(lyrics_response).await;
    assert_eq!(lyrics_json["taskId"], "task_lyrics_1");

    let lyrics_record_response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/lyrics/record-info?taskId=task_lyrics_1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(lyrics_record_response.status(), StatusCode::OK);
    let lyrics_record_json = read_json(lyrics_record_response).await;
    assert_eq!(lyrics_record_json["taskId"], "task_lyrics_1");

    assert_eq!(
        upstream_state.authorizations.lock().unwrap().clone(),
        vec![
            "Bearer sk-stateless-suno".to_owned(),
            "Bearer sk-stateless-suno".to_owned(),
            "Bearer sk-stateless-suno".to_owned(),
            "Bearer sk-stateless-suno".to_owned(),
        ]
    );
    assert_eq!(
        upstream_state.generate_body.lock().unwrap().clone(),
        Some(serde_json::json!({
            "prompt":"Write an uplifting electronic anthem",
            "mv":"chirp-v4"
        }))
    );
    assert_eq!(
        upstream_state
            .generate_record_task_id
            .lock()
            .unwrap()
            .as_deref(),
        Some("task_music_1")
    );
    assert_eq!(
        upstream_state.lyrics_body.lock().unwrap().clone(),
        Some(serde_json::json!({
            "prompt":"Write uplifting synth-pop lyrics",
            "title":"Skyline Pulse"
        }))
    );
    assert_eq!(
        upstream_state
            .lyrics_record_task_id
            .lock()
            .unwrap()
            .as_deref(),
        Some("task_lyrics_1")
    );
}

#[serial(extension_env)]
#[tokio::test]
async fn stateless_music_minimax_routes_relay_to_official_paths() {
    let upstream_state = MiniMaxUpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route(
            "/v1/music_generation",
            post(upstream_minimax_music_generation_handler),
        )
        .route(
            "/v1/lyrics_generation",
            post(upstream_minimax_lyrics_generation_handler),
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
                "sk-stateless-minimax",
            ),
        ),
    );

    let music_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/music_generation")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"music-01\",\"prompt\":\"Write a mellow piano ballad\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(music_response.status(), StatusCode::OK);
    let music_json = read_json(music_response).await;
    assert_eq!(music_json["trace_id"], "trace-music-minimax-1");

    let lyrics_response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/lyrics_generation")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"prompt\":\"Write soft pop lyrics about spring rain\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(lyrics_response.status(), StatusCode::OK);
    let lyrics_json = read_json(lyrics_response).await;
    assert_eq!(lyrics_json["data"]["song_title"], "Spring Rain");

    assert_eq!(
        upstream_state.authorizations.lock().unwrap().clone(),
        vec![
            "Bearer sk-stateless-minimax".to_owned(),
            "Bearer sk-stateless-minimax".to_owned()
        ]
    );
    assert_eq!(
        upstream_state.music_generation_body.lock().unwrap().clone(),
        Some(serde_json::json!({
            "model":"music-01",
            "prompt":"Write a mellow piano ballad"
        }))
    );
    assert_eq!(
        upstream_state
            .lyrics_generation_body
            .lock()
            .unwrap()
            .clone(),
        Some(serde_json::json!({
            "prompt":"Write soft pop lyrics about spring rain"
        }))
    );
}

#[serial(extension_env)]
#[tokio::test]
async fn stateless_music_google_routes_relay_to_official_google_predict_path() {
    let upstream_state = GoogleMusicUpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route(
            "/v1/projects/test-project/locations/us-central1/publishers/google/models/lyria-002:predict",
            post(upstream_google_music_predict_handler),
        )
        .with_state(upstream_state.clone());

    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });

    let app = sdkwork_api_interface_http::gateway_router_with_stateless_config(
        sdkwork_api_interface_http::StatelessGatewayConfig::default().with_upstream(
            sdkwork_api_interface_http::StatelessGatewayUpstream::new_with_protocol_kind_and_identity(
                "sdkwork.provider.google.music",
                "custom",
                "google",
                format!("http://{address}"),
                "sk-stateless-google-music",
            ),
        ),
    );

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/projects/test-project/locations/us-central1/publishers/google/models/lyria-002:predict")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"instances\":[{\"prompt\":\"Write a hopeful ambient piano cue\"}],\"parameters\":{\"sampleCount\":1}}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["predictions"][0]["mimeType"], "audio/wav");
    assert_eq!(
        upstream_state.authorizations.lock().unwrap().clone(),
        vec!["Bearer sk-stateless-google-music".to_owned()]
    );
    assert_eq!(
        upstream_state.predict_body.lock().unwrap().clone(),
        Some(serde_json::json!({
            "instances":[{"prompt":"Write a hopeful ambient piano cue"}],
            "parameters":{"sampleCount":1}
        }))
    );
}

#[serial(extension_env)]
#[tokio::test]
async fn stateful_music_create_records_music_seconds_in_billing_events() {
    let tenant_id = "tenant-music-billing";
    let project_id = "project-music-billing";
    let request_model = "suno-v4";
    let upstream_state = UpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route("/v1/music", post(upstream_music_create_handler))
        .with_state(upstream_state.clone());

    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });

    let pool = memory_pool().await;
    let api_key = issue_funded_gateway_api_key(&pool, tenant_id, project_id).await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(&pool, admin_app.clone()).await;
    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool);

    create_channel(&admin_app, &admin_token).await;
    create_provider(
        &admin_app,
        &admin_token,
        "provider-music",
        &format!("http://{address}"),
        "Music Provider",
    )
    .await;
    create_credential(
        &admin_app,
        &admin_token,
        tenant_id,
        "provider-music",
        "cred-music",
        "sk-upstream-music",
    )
    .await;
    create_model(&admin_app, &admin_token, request_model, "provider-music").await;

    let create_response = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/music")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"suno-v4\",\"prompt\":\"Write an uplifting electronic anthem\",\"duration_seconds\":123.0}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(create_response.status(), StatusCode::OK);
    let create_json = read_json(create_response).await;
    assert_eq!(create_json["data"][0]["id"], "music_upstream");
    assert_eq!(
        upstream_state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-upstream-music")
    );
    support::assert_single_usage_record_and_decision_log(
        admin_app.clone(),
        &admin_token,
        request_model,
        "provider-music",
        request_model,
    )
    .await;

    let billing_events = admin_app
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
    assert_eq!(billing_json.as_array().unwrap().len(), 1);
    assert_eq!(billing_json[0]["capability"], "music");
    assert_eq!(billing_json[0]["route_key"], request_model);
    assert_eq!(billing_json[0]["usage_model"], request_model);
    assert_eq!(billing_json[0]["provider_id"], "provider-music");
    assert_eq!(billing_json[0]["reference_id"], "music_upstream");
    assert_eq!(billing_json[0]["music_seconds"], 123.0);
}

#[serial(extension_env)]
#[tokio::test]
async fn stateful_music_suno_generate_uses_suno_provider_identity() {
    let tenant_id = "tenant-music-suno-stateful";
    let project_id = "project-music-suno-stateful";

    let generic_state = GenericUpstreamCaptureState::default();
    let generic_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let generic_address = generic_listener.local_addr().unwrap();
    let generic_upstream = Router::new()
        .route("/api/v1/generate", post(upstream_generic_generate_handler))
        .with_state(generic_state.clone());
    tokio::spawn(async move {
        axum::serve(generic_listener, generic_upstream)
            .await
            .unwrap();
    });

    let suno_state = SunoUpstreamCaptureState::default();
    let suno_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let suno_address = suno_listener.local_addr().unwrap();
    let suno_upstream = Router::new()
        .route("/api/v1/generate", post(upstream_suno_generate_handler))
        .with_state(suno_state.clone());
    tokio::spawn(async move {
        axum::serve(suno_listener, suno_upstream).await.unwrap();
    });

    let pool = memory_pool().await;
    let api_key = issue_funded_gateway_api_key(&pool, tenant_id, project_id).await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(&pool, admin_app.clone()).await;
    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool);

    create_channel_with_name(&admin_app, &admin_token, "openai", "OpenAI").await;
    create_channel_with_name(&admin_app, &admin_token, "suno", "Suno").await;
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
            "{{\"id\":\"provider-z-suno\",\"channel_id\":\"suno\",\"extension_id\":\"sdkwork.provider.suno.relay\",\"adapter_kind\":\"native-dynamic\",\"protocol_kind\":\"custom\",\"base_url\":\"http://{suno_address}\",\"display_name\":\"Suno Provider\"}}"
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
        "provider-z-suno",
        "cred-suno",
        "sk-suno-upstream",
    )
    .await;

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/generate")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"prompt\":\"Write an uplifting electronic anthem\",\"mv\":\"chirp-v4\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["taskId"], "task_music_1");
    assert_eq!(*generic_state.hits.lock().unwrap(), 0);
    assert_eq!(
        suno_state.authorizations.lock().unwrap().clone(),
        vec!["Bearer sk-suno-upstream".to_owned()]
    );
    assert_eq!(
        suno_state.generate_body.lock().unwrap().clone(),
        Some(serde_json::json!({
            "prompt":"Write an uplifting electronic anthem",
            "mv":"chirp-v4"
        }))
    );
    support::assert_single_decision_log(
        admin_app,
        &admin_token,
        "music.suno.generate",
        "provider-z-suno",
    )
    .await;
}

#[serial(extension_env)]
#[tokio::test]
async fn stateful_music_google_predict_uses_google_provider_identity() {
    let tenant_id = "tenant-music-google-stateful";
    let project_id = "project-music-google-stateful";
    let request_model = "lyria-002";

    let generic_state = GenericUpstreamCaptureState::default();
    let generic_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let generic_address = generic_listener.local_addr().unwrap();
    let generic_upstream = Router::new()
        .route(
            "/v1/projects/test-project/locations/us-central1/publishers/google/models/lyria-002:predict",
            post(upstream_generic_google_music_predict_handler),
        )
        .with_state(generic_state.clone());
    tokio::spawn(async move {
        axum::serve(generic_listener, generic_upstream)
            .await
            .unwrap();
    });

    let google_state = GoogleMusicUpstreamCaptureState::default();
    let google_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let google_address = google_listener.local_addr().unwrap();
    let google_upstream = Router::new()
        .route(
            "/v1/projects/test-project/locations/us-central1/publishers/google/models/lyria-002:predict",
            post(upstream_google_music_predict_handler),
        )
        .with_state(google_state.clone());
    tokio::spawn(async move {
        axum::serve(google_listener, google_upstream).await.unwrap();
    });

    let pool = memory_pool().await;
    let api_key = issue_funded_gateway_api_key(&pool, tenant_id, project_id).await;
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
            "{{\"id\":\"provider-z-google-music\",\"channel_id\":\"google\",\"extension_id\":\"sdkwork.provider.google.music\",\"adapter_kind\":\"native-dynamic\",\"protocol_kind\":\"custom\",\"base_url\":\"http://{google_address}\",\"display_name\":\"Google Music Provider\"}}"
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
        "provider-z-google-music",
        "cred-google-music",
        "sk-google-music-upstream",
    )
    .await;
    create_model(
        &admin_app,
        &admin_token,
        request_model,
        "provider-z-google-music",
    )
    .await;

    let response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/projects/test-project/locations/us-central1/publishers/google/models/lyria-002:predict")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"instances\":[{\"prompt\":\"Write a hopeful ambient piano cue\"}],\"parameters\":{\"sampleCount\":1}}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["predictions"][0]["mimeType"], "audio/wav");
    assert_eq!(*generic_state.hits.lock().unwrap(), 0);
    assert_eq!(
        google_state.authorizations.lock().unwrap().clone(),
        vec!["Bearer sk-google-music-upstream".to_owned()]
    );
    assert_eq!(
        google_state.predict_body.lock().unwrap().clone(),
        Some(serde_json::json!({
            "instances":[{"prompt":"Write a hopeful ambient piano cue"}],
            "parameters":{"sampleCount":1}
        }))
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
    assert_eq!(usage_records.len(), 1);
    assert_eq!(usage_records[0]["model"], "music.google.predict");
    assert_eq!(usage_records[0]["provider"], "provider-z-google-music");

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
    assert_eq!(billing_records.len(), 1);
    assert_eq!(billing_records[0]["route_key"], "music.google.predict");
    assert_eq!(billing_records[0]["provider_id"], "provider-z-google-music");

    support::assert_single_decision_log(
        admin_app,
        &admin_token,
        "music.google.predict",
        "provider-z-google-music",
    )
    .await;
}

#[serial(extension_env)]
#[tokio::test]
async fn stateful_music_minimax_routes_use_minimax_provider_identity() {
    let tenant_id = "tenant-music-minimax-stateful";
    let project_id = "project-music-minimax-stateful";

    let generic_state = GenericUpstreamCaptureState::default();
    let generic_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let generic_address = generic_listener.local_addr().unwrap();
    let generic_upstream = Router::new()
        .route(
            "/v1/music_generation",
            post(upstream_generic_music_generation_handler),
        )
        .route(
            "/v1/lyrics_generation",
            post(upstream_generic_lyrics_generation_handler),
        )
        .with_state(generic_state.clone());
    tokio::spawn(async move {
        axum::serve(generic_listener, generic_upstream)
            .await
            .unwrap();
    });

    let minimax_state = MiniMaxUpstreamCaptureState::default();
    let minimax_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let minimax_address = minimax_listener.local_addr().unwrap();
    let minimax_upstream = Router::new()
        .route(
            "/v1/music_generation",
            post(upstream_minimax_music_generation_handler),
        )
        .route(
            "/v1/lyrics_generation",
            post(upstream_minimax_lyrics_generation_handler),
        )
        .with_state(minimax_state.clone());
    tokio::spawn(async move {
        axum::serve(minimax_listener, minimax_upstream)
            .await
            .unwrap();
    });

    let pool = memory_pool().await;
    let api_key = issue_funded_gateway_api_key(&pool, tenant_id, project_id).await;
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

    let music_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/music_generation")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"music-01\",\"prompt\":\"Write a mellow piano ballad\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(music_response.status(), StatusCode::OK);
    let music_json = read_json(music_response).await;
    assert_eq!(music_json["trace_id"], "trace-music-minimax-1");

    let lyrics_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/lyrics_generation")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"prompt\":\"Write soft pop lyrics about spring rain\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(lyrics_response.status(), StatusCode::OK);
    let lyrics_json = read_json(lyrics_response).await;
    assert_eq!(lyrics_json["data"]["song_title"], "Spring Rain");

    assert_eq!(*generic_state.hits.lock().unwrap(), 0);
    assert_eq!(
        minimax_state.authorizations.lock().unwrap().clone(),
        vec![
            "Bearer sk-minimax-upstream".to_owned(),
            "Bearer sk-minimax-upstream".to_owned()
        ]
    );
    assert_eq!(
        minimax_state.music_generation_body.lock().unwrap().clone(),
        Some(serde_json::json!({
            "model":"music-01",
            "prompt":"Write a mellow piano ballad"
        }))
    );
    assert_eq!(
        minimax_state.lyrics_generation_body.lock().unwrap().clone(),
        Some(serde_json::json!({
            "prompt":"Write soft pop lyrics about spring rain"
        }))
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
        record["model"] == "music.minimax.generate" && record["provider"] == "provider-z-minimax"
    }));
    assert!(usage_records.iter().any(|record| {
        record["model"] == "music.minimax.lyrics" && record["provider"] == "provider-z-minimax"
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
        record["route_key"] == "music.minimax.generate"
            && record["provider_id"] == "provider-z-minimax"
            && record["reference_id"] == "trace-music-minimax-1"
            && record["music_seconds"] == 180.0
    }));
    assert!(billing_records.iter().any(|record| {
        record["route_key"] == "music.minimax.lyrics"
            && record["provider_id"] == "provider-z-minimax"
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
        record["route_key"] == "music.minimax.generate"
            && record["selected_provider_id"] == "provider-z-minimax"
    }));
    assert!(logs.iter().any(|record| {
        record["route_key"] == "music.minimax.lyrics"
            && record["selected_provider_id"] == "provider-z-minimax"
    }));
}

async fn create_channel(admin_app: &Router, admin_token: &str) {
    create_channel_with_name(admin_app, admin_token, "openai", "OpenAI").await;
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

async fn create_provider(
    admin_app: &Router,
    admin_token: &str,
    provider_id: &str,
    base_url: &str,
    display_name: &str,
) {
    create_provider_with_payload(
        admin_app,
        admin_token,
        &format!(
            "{{\"id\":\"{provider_id}\",\"channel_id\":\"openai\",\"adapter_kind\":\"openai\",\"base_url\":\"{base_url}\",\"display_name\":\"{display_name}\"}}"
        ),
    )
    .await;
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

async fn create_model(
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

#[serial(extension_env)]
#[tokio::test]
async fn stateful_music_retrieve_route_returns_not_found_without_usage() {
    let ctx = local_music_test_context(
        "tenant-music-retrieve-missing",
        "project-music-retrieve-missing",
    )
    .await;

    let response = ctx
        .gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/music/music_missing")
                .header("authorization", format!("Bearer {}", ctx.api_key))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_music_not_found(response, "Requested music was not found.").await;
    support::assert_no_usage_records(ctx.admin_app, &ctx.admin_token).await;
}

#[serial(extension_env)]
#[tokio::test]
async fn stateful_music_delete_route_returns_not_found_without_usage() {
    let ctx = local_music_test_context(
        "tenant-music-delete-missing",
        "project-music-delete-missing",
    )
    .await;

    let response = ctx
        .gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/v1/music/music_missing")
                .header("authorization", format!("Bearer {}", ctx.api_key))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_music_not_found(response, "Requested music was not found.").await;
    support::assert_no_usage_records(ctx.admin_app, &ctx.admin_token).await;
}

#[serial(extension_env)]
#[tokio::test]
async fn stateful_music_content_route_returns_not_found_without_usage() {
    let ctx = local_music_test_context(
        "tenant-music-content-missing",
        "project-music-content-missing",
    )
    .await;

    let response = ctx
        .gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/music/music_missing/content")
                .header("authorization", format!("Bearer {}", ctx.api_key))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_music_not_found(response, "Requested music asset was not found.").await;
    support::assert_no_usage_records(ctx.admin_app, &ctx.admin_token).await;
}

#[serial(extension_env)]
#[tokio::test]
async fn stateful_music_lyrics_route_returns_invalid_request_without_usage() {
    let ctx = local_music_test_context(
        "tenant-music-lyrics-unsupported",
        "project-music-lyrics-unsupported",
    )
    .await;

    let response = ctx
        .gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/music/lyrics")
                .header("authorization", format!("Bearer {}", ctx.api_key))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"prompt\":\"Write uplifting synth-pop lyrics\",\"title\":\"Skyline Pulse\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_invalid_music_request(
        response,
        "Local music lyrics fallback is not supported without a lyrics generation backend.",
    )
    .await;
    support::assert_no_usage_records(ctx.admin_app, &ctx.admin_token).await;
}

async fn read_json(response: axum::response::Response) -> Value {
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

async fn assert_music_not_found(response: axum::response::Response, message: &str) {
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let json = read_json(response).await;
    assert_eq!(json["error"]["message"], message);
    assert_eq!(json["error"]["type"], "invalid_request_error");
    assert_eq!(json["error"]["code"], "not_found");
}

async fn assert_invalid_music_request(response: axum::response::Response, message: &str) {
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let json = read_json(response).await;
    assert_eq!(json["error"]["message"], message);
    assert_eq!(json["error"]["type"], "invalid_request_error");
    assert_eq!(json["error"]["code"], "invalid_music_request");
}

async fn read_bytes(response: axum::response::Response) -> Vec<u8> {
    axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap()
        .to_vec()
}

async fn memory_pool() -> SqlitePool {
    sdkwork_api_storage_sqlite::run_migrations("sqlite::memory:")
        .await
        .unwrap()
}

struct LocalMusicTestContext {
    admin_app: Router,
    admin_token: String,
    api_key: String,
    gateway_app: Router,
}

async fn local_music_test_context(tenant_id: &str, project_id: &str) -> LocalMusicTestContext {
    let pool = memory_pool().await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(&pool, admin_app.clone()).await;
    let api_key = support::issue_gateway_api_key(&pool, tenant_id, project_id).await;
    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool);

    LocalMusicTestContext {
        admin_app,
        admin_token,
        api_key,
        gateway_app,
    }
}

#[derive(Clone, Default)]
struct UpstreamCaptureState {
    authorization: Arc<Mutex<Option<String>>>,
}

#[derive(Clone, Default)]
struct SunoUpstreamCaptureState {
    authorizations: Arc<Mutex<Vec<String>>>,
    generate_body: Arc<Mutex<Option<Value>>>,
    generate_record_task_id: Arc<Mutex<Option<String>>>,
    lyrics_body: Arc<Mutex<Option<Value>>>,
    lyrics_record_task_id: Arc<Mutex<Option<String>>>,
}

#[derive(Clone, Default)]
struct MiniMaxUpstreamCaptureState {
    authorizations: Arc<Mutex<Vec<String>>>,
    music_generation_body: Arc<Mutex<Option<Value>>>,
    lyrics_generation_body: Arc<Mutex<Option<Value>>>,
}

#[derive(Clone, Default)]
struct GoogleMusicUpstreamCaptureState {
    authorizations: Arc<Mutex<Vec<String>>>,
    predict_body: Arc<Mutex<Option<Value>>>,
}

#[derive(Clone, Default)]
struct GenericUpstreamCaptureState {
    hits: Arc<Mutex<usize>>,
}

async fn upstream_music_create_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    *state.authorization.lock().unwrap() = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "object":"list",
            "data":[{
                "id":"music_upstream",
                "object":"music",
                "status":"completed",
                "model":"suno-v4",
                "audio_url":"https://example.com/music.mp3",
                "duration_seconds":123.0
            }]
        })),
    )
}

async fn upstream_music_list_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    *state.authorization.lock().unwrap() = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "object":"list",
            "data":[{
                "id":"music_1",
                "object":"music",
                "status":"completed",
                "audio_url":"https://example.com/music.mp3"
            }]
        })),
    )
}

async fn upstream_music_retrieve_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    *state.authorization.lock().unwrap() = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "id":"music_1",
            "object":"music",
            "status":"completed",
            "audio_url":"https://example.com/music.mp3"
        })),
    )
}

async fn upstream_music_delete_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    *state.authorization.lock().unwrap() = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "id":"music_1",
            "object":"music.deleted",
            "deleted":true
        })),
    )
}

async fn upstream_music_content_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (
    [(axum::http::header::HeaderName, &'static str); 1],
    &'static [u8],
) {
    *state.authorization.lock().unwrap() = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);
    (
        [(axum::http::header::CONTENT_TYPE, "audio/mpeg")],
        b"UPSTREAM-MUSIC",
    )
}

async fn upstream_music_lyrics_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    *state.authorization.lock().unwrap() = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "id":"lyrics_1",
            "object":"music.lyrics",
            "status":"completed",
            "title":"Skyline Pulse",
            "text":"We rise with the skyline"
        })),
    )
}

async fn upstream_suno_generate_handler(
    State(state): State<SunoUpstreamCaptureState>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<Value>,
) -> (StatusCode, Json<Value>) {
    if let Some(value) = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
    {
        state.authorizations.lock().unwrap().push(value.to_owned());
    }
    *state.generate_body.lock().unwrap() = Some(payload);
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "taskId":"task_music_1",
            "status":"submitted"
        })),
    )
}

async fn upstream_suno_generate_record_info_handler(
    State(state): State<SunoUpstreamCaptureState>,
    headers: axum::http::HeaderMap,
    Query(query): Query<HashMap<String, String>>,
) -> (StatusCode, Json<Value>) {
    if let Some(value) = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
    {
        state.authorizations.lock().unwrap().push(value.to_owned());
    }
    *state.generate_record_task_id.lock().unwrap() = query.get("taskId").cloned();
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "taskId": query.get("taskId").cloned().unwrap_or_default(),
            "status":"completed"
        })),
    )
}

async fn upstream_suno_lyrics_create_handler(
    State(state): State<SunoUpstreamCaptureState>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<Value>,
) -> (StatusCode, Json<Value>) {
    if let Some(value) = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
    {
        state.authorizations.lock().unwrap().push(value.to_owned());
    }
    *state.lyrics_body.lock().unwrap() = Some(payload);
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "taskId":"task_lyrics_1",
            "status":"submitted"
        })),
    )
}

async fn upstream_suno_lyrics_record_info_handler(
    State(state): State<SunoUpstreamCaptureState>,
    headers: axum::http::HeaderMap,
    Query(query): Query<HashMap<String, String>>,
) -> (StatusCode, Json<Value>) {
    if let Some(value) = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
    {
        state.authorizations.lock().unwrap().push(value.to_owned());
    }
    *state.lyrics_record_task_id.lock().unwrap() = query.get("taskId").cloned();
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "taskId": query.get("taskId").cloned().unwrap_or_default(),
            "status":"completed"
        })),
    )
}

async fn upstream_minimax_music_generation_handler(
    State(state): State<MiniMaxUpstreamCaptureState>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<Value>,
) -> (StatusCode, Json<Value>) {
    if let Some(value) = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
    {
        state.authorizations.lock().unwrap().push(value.to_owned());
    }
    *state.music_generation_body.lock().unwrap() = Some(payload);
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "base_resp":{"status_code":0,"status_msg":"success"},
            "data":{
                "audio":"abc123",
                "instrumental":"def456",
                "cover":"https://cdn.example.com/minimax-cover.png",
                "song_title":"Mellow Breeze"
            },
            "extra_info":{
                "audio_sample_rate":44100,
                "music_duration":180000
            },
            "trace_id":"trace-music-minimax-1"
        })),
    )
}

async fn upstream_minimax_lyrics_generation_handler(
    State(state): State<MiniMaxUpstreamCaptureState>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<Value>,
) -> (StatusCode, Json<Value>) {
    if let Some(value) = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
    {
        state.authorizations.lock().unwrap().push(value.to_owned());
    }
    *state.lyrics_generation_body.lock().unwrap() = Some(payload);
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "base_resp":{"status_code":0,"status_msg":"success"},
            "data":{
                "song_title":"Spring Rain",
                "lyrics":"Spring rain taps the window tonight"
            }
        })),
    )
}

async fn upstream_google_music_predict_handler(
    State(state): State<GoogleMusicUpstreamCaptureState>,
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
            "predictions":[{
                "audioContent":"UklGRg==",
                "mimeType":"audio/wav"
            }],
            "metadata":{"model":"lyria-002"}
        })),
    )
}

async fn upstream_generic_generate_handler(
    State(state): State<GenericUpstreamCaptureState>,
) -> (StatusCode, Json<Value>) {
    *state.hits.lock().unwrap() += 1;
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "taskId":"generic_should_not_be_used"
        })),
    )
}

async fn upstream_generic_music_generation_handler(
    State(state): State<GenericUpstreamCaptureState>,
) -> (StatusCode, Json<Value>) {
    *state.hits.lock().unwrap() += 1;
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "trace_id":"generic_should_not_be_used"
        })),
    )
}

async fn upstream_generic_google_music_predict_handler(
    State(state): State<GenericUpstreamCaptureState>,
) -> (StatusCode, Json<Value>) {
    *state.hits.lock().unwrap() += 1;
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "predictions":[{
                "audioContent":"Z2VuZXJpYw==",
                "mimeType":"audio/wav"
            }]
        })),
    )
}

async fn upstream_generic_lyrics_generation_handler(
    State(state): State<GenericUpstreamCaptureState>,
) -> (StatusCode, Json<Value>) {
    *state.hits.lock().unwrap() += 1;
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "data":{"song_title":"generic_should_not_be_used"}
        })),
    )
}
