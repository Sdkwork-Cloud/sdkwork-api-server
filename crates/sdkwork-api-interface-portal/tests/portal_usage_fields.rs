use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use serde_json::Value;
use sqlx::SqlitePool;
use tower::ServiceExt;

async fn read_json(response: axum::response::Response) -> Value {
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

async fn memory_pool() -> SqlitePool {
    sdkwork_api_storage_sqlite::run_migrations("sqlite::memory:")
        .await
        .unwrap()
}

async fn portal_token(app: axum::Router) -> String {
    let register_response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"email\":\"portal-fields@example.com\",\"password\":\"hunter2!\",\"display_name\":\"Portal User\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(register_response.status(), StatusCode::CREATED);
    read_json(register_response).await["token"]
        .as_str()
        .unwrap()
        .to_owned()
}

async fn portal_workspace(app: axum::Router, token: &str) -> Value {
    let workspace_response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/workspace")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(workspace_response.status(), StatusCode::OK);
    read_json(workspace_response).await
}

#[tokio::test]
async fn portal_usage_records_include_api_key_channel_latency_and_reference_price() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_portal::portal_router_with_pool(pool.clone());
    let token = portal_token(app.clone()).await;
    let workspace = portal_workspace(app.clone(), &token).await;
    let project_id = workspace["project"]["id"].as_str().unwrap().to_owned();

    sqlx::query(
        "INSERT INTO ai_usage_records (
            project_id,
            model,
            provider_id,
            units,
            amount,
            input_tokens,
            output_tokens,
            total_tokens,
            api_key_hash,
            channel_id,
            latency_ms,
            reference_amount,
            created_at_ms
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&project_id)
    .bind("gpt-4.1")
    .bind("provider-openai-official")
    .bind(240_i64)
    .bind(0.42_f64)
    .bind(160_i64)
    .bind(40_i64)
    .bind(200_i64)
    .bind("key-live")
    .bind("openai")
    .bind(850_i64)
    .bind(0.55_f64)
    .bind(1_710_000_001_i64)
    .execute(&pool)
    .await
    .unwrap();

    let usage_records_response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/usage/records")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(usage_records_response.status(), StatusCode::OK);
    let usage_records_json = read_json(usage_records_response).await;
    assert_eq!(usage_records_json.as_array().unwrap().len(), 1);
    assert_eq!(usage_records_json[0]["project_id"], project_id);
    assert_eq!(usage_records_json[0]["api_key_hash"], "key-live");
    assert_eq!(usage_records_json[0]["channel_id"], "openai");
    assert_eq!(usage_records_json[0]["latency_ms"], 850);
    assert_eq!(usage_records_json[0]["reference_amount"], 0.55);
}
