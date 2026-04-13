use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use sdkwork_api_app_identity::{gateway_auth_subject_from_request_context, GatewayRequestContext};
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
                    "{\"email\":\"jobs-portal@example.com\",\"password\":\"PortalPass123!\",\"display_name\":\"Jobs Portal User\"}",
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

fn workspace_request_context(workspace: &Value) -> GatewayRequestContext {
    GatewayRequestContext {
        tenant_id: workspace["tenant"]["id"].as_str().unwrap().to_owned(),
        project_id: workspace["project"]["id"].as_str().unwrap().to_owned(),
        environment: "portal".to_owned(),
        api_key_hash: "portal_workspace_scope".to_owned(),
        api_key_group_id: None,
        canonical_tenant_id: None,
        canonical_organization_id: None,
        canonical_user_id: None,
        canonical_api_key_id: None,
    }
}

async fn create_async_job_tables(pool: &SqlitePool) {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_async_jobs (
            job_id TEXT PRIMARY KEY NOT NULL,
            tenant_id INTEGER NOT NULL,
            organization_id INTEGER NOT NULL,
            user_id INTEGER NOT NULL,
            account_id INTEGER,
            request_id INTEGER,
            provider_id TEXT,
            model_code TEXT,
            capability_code TEXT NOT NULL,
            modality TEXT NOT NULL,
            operation_kind TEXT NOT NULL,
            status TEXT NOT NULL,
            external_job_id TEXT,
            idempotency_key TEXT,
            callback_url TEXT,
            input_summary TEXT,
            progress_percent INTEGER,
            error_code TEXT,
            error_message TEXT,
            created_at_ms INTEGER NOT NULL,
            updated_at_ms INTEGER NOT NULL,
            started_at_ms INTEGER,
            completed_at_ms INTEGER
        )",
    )
    .execute(pool)
    .await
    .unwrap();

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_async_job_attempts (
            attempt_id INTEGER PRIMARY KEY NOT NULL,
            job_id TEXT NOT NULL,
            attempt_number INTEGER NOT NULL,
            status TEXT NOT NULL,
            runtime_kind TEXT NOT NULL,
            endpoint TEXT,
            external_job_id TEXT,
            claimed_at_ms INTEGER,
            finished_at_ms INTEGER,
            error_message TEXT,
            created_at_ms INTEGER NOT NULL,
            updated_at_ms INTEGER NOT NULL
        )",
    )
    .execute(pool)
    .await
    .unwrap();

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_async_job_assets (
            asset_id TEXT PRIMARY KEY NOT NULL,
            job_id TEXT NOT NULL,
            asset_kind TEXT NOT NULL,
            storage_key TEXT NOT NULL,
            download_url TEXT,
            mime_type TEXT,
            size_bytes INTEGER,
            checksum_sha256 TEXT,
            created_at_ms INTEGER NOT NULL
        )",
    )
    .execute(pool)
    .await
    .unwrap();
}

async fn seed_portal_async_job_fixture(pool: &SqlitePool, workspace: &Value) {
    create_async_job_tables(pool).await;

    let subject = gateway_auth_subject_from_request_context(&workspace_request_context(workspace));

    sqlx::query(
        "INSERT INTO ai_async_jobs (
            job_id, tenant_id, organization_id, user_id, account_id, request_id, provider_id,
            model_code, capability_code, modality, operation_kind, status, external_job_id,
            idempotency_key, callback_url, input_summary, progress_percent, error_code,
            error_message, created_at_ms, updated_at_ms, started_at_ms, completed_at_ms
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind("job_image_workspace_1")
    .bind(i64::try_from(subject.tenant_id).unwrap())
    .bind(i64::try_from(subject.organization_id).unwrap())
    .bind(i64::try_from(subject.user_id).unwrap())
    .bind(Option::<i64>::None)
    .bind(9901_i64)
    .bind("provider-openrouter")
    .bind("gpt-image-1")
    .bind("images")
    .bind("image")
    .bind("generation")
    .bind("succeeded")
    .bind("workspace-upstream-job-1")
    .bind("idem-workspace-job-1")
    .bind("https://workspace.example.com/callbacks/images")
    .bind("Generate a hero image")
    .bind(100_i64)
    .bind(Option::<String>::None)
    .bind(Option::<String>::None)
    .bind(1_710_100_000_000_i64)
    .bind(1_710_100_050_000_i64)
    .bind(1_710_100_005_000_i64)
    .bind(1_710_100_050_000_i64)
    .execute(pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO ai_async_jobs (
            job_id, tenant_id, organization_id, user_id, account_id, request_id, provider_id,
            model_code, capability_code, modality, operation_kind, status, external_job_id,
            idempotency_key, callback_url, input_summary, progress_percent, error_code,
            error_message, created_at_ms, updated_at_ms, started_at_ms, completed_at_ms
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind("job_image_foreign_1")
    .bind(i64::try_from(subject.tenant_id + 1).unwrap())
    .bind(i64::try_from(subject.organization_id + 1).unwrap())
    .bind(i64::try_from(subject.user_id + 1).unwrap())
    .bind(Option::<i64>::None)
    .bind(9902_i64)
    .bind("provider-foreign")
    .bind("foreign-image-model")
    .bind("images")
    .bind("image")
    .bind("generation")
    .bind("running")
    .bind("foreign-upstream-job-1")
    .bind("idem-foreign-job-1")
    .bind("https://foreign.example.com/callbacks/images")
    .bind("Do not expose this job")
    .bind(42_i64)
    .bind(Option::<String>::None)
    .bind(Option::<String>::None)
    .bind(1_710_100_060_000_i64)
    .bind(1_710_100_065_000_i64)
    .bind(1_710_100_061_000_i64)
    .bind(Option::<i64>::None)
    .execute(pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO ai_async_job_attempts (
            attempt_id, job_id, attempt_number, status, runtime_kind, endpoint, external_job_id,
            claimed_at_ms, finished_at_ms, error_message, created_at_ms, updated_at_ms
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(7101_i64)
    .bind("job_image_workspace_1")
    .bind(1_i64)
    .bind("succeeded")
    .bind("provider_api")
    .bind("https://provider.example.com/v1/images")
    .bind("workspace-upstream-job-1")
    .bind(1_710_100_005_000_i64)
    .bind(1_710_100_050_000_i64)
    .bind(Option::<String>::None)
    .bind(1_710_100_005_000_i64)
    .bind(1_710_100_050_000_i64)
    .execute(pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO ai_async_job_attempts (
            attempt_id, job_id, attempt_number, status, runtime_kind, endpoint, external_job_id,
            claimed_at_ms, finished_at_ms, error_message, created_at_ms, updated_at_ms
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(7199_i64)
    .bind("job_image_foreign_1")
    .bind(1_i64)
    .bind("running")
    .bind("provider_api")
    .bind("https://provider.example.com/v1/images")
    .bind("foreign-upstream-job-1")
    .bind(1_710_100_061_000_i64)
    .bind(Option::<i64>::None)
    .bind(Option::<String>::None)
    .bind(1_710_100_061_000_i64)
    .bind(1_710_100_065_000_i64)
    .execute(pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO ai_async_job_assets (
            asset_id, job_id, asset_kind, storage_key, download_url, mime_type, size_bytes,
            checksum_sha256, created_at_ms
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind("asset_image_workspace_1")
    .bind("job_image_workspace_1")
    .bind("image")
    .bind("tenant-scope/jobs/job_image_workspace_1/preview.png")
    .bind("https://cdn.example.com/jobs/job_image_workspace_1/preview.png")
    .bind("image/png")
    .bind(4096_i64)
    .bind("sha256-image-workspace")
    .bind(1_710_100_050_000_i64)
    .execute(pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO ai_async_job_assets (
            asset_id, job_id, asset_kind, storage_key, download_url, mime_type, size_bytes,
            checksum_sha256, created_at_ms
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind("asset_image_foreign_1")
    .bind("job_image_foreign_1")
    .bind("image")
    .bind("foreign/jobs/job_image_foreign_1/preview.png")
    .bind("https://cdn.example.com/jobs/job_image_foreign_1/preview.png")
    .bind("image/png")
    .bind(8192_i64)
    .bind("sha256-image-foreign")
    .bind(1_710_100_070_000_i64)
    .execute(pool)
    .await
    .unwrap();
}

#[tokio::test]
async fn portal_async_jobs_routes_only_expose_workspace_job_inventory() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_portal::portal_router_with_pool(pool.clone());
    let token = portal_token(app.clone()).await;
    let workspace = portal_workspace(app.clone(), &token).await;
    seed_portal_async_job_fixture(&pool, &workspace).await;

    let jobs = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/async-jobs")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(jobs.status(), StatusCode::OK);
    let jobs_json = read_json(jobs).await;
    assert_eq!(jobs_json.as_array().unwrap().len(), 1);
    assert_eq!(jobs_json[0]["job_id"], "job_image_workspace_1");
    assert_eq!(jobs_json[0]["capability_code"], "images");
    assert_eq!(jobs_json[0]["status"], "succeeded");

    let attempts = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/async-jobs/job_image_workspace_1/attempts")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(attempts.status(), StatusCode::OK);
    let attempts_json = read_json(attempts).await;
    assert_eq!(attempts_json.as_array().unwrap().len(), 1);
    assert_eq!(attempts_json[0]["attempt_id"], 7101);
    assert_eq!(attempts_json[0]["status"], "succeeded");

    let assets = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/async-jobs/job_image_workspace_1/assets")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(assets.status(), StatusCode::OK);
    let assets_json = read_json(assets).await;
    assert_eq!(assets_json.as_array().unwrap().len(), 1);
    assert_eq!(assets_json[0]["asset_id"], "asset_image_workspace_1");
    assert_eq!(assets_json[0]["asset_kind"], "image");
}
