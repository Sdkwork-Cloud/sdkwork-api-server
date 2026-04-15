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
    let pool = sdkwork_api_storage_sqlite::run_migrations("sqlite::memory:")
        .await
        .unwrap();
    let store = sdkwork_api_storage_sqlite::SqliteAdminStore::new(pool.clone());
    sdkwork_api_app_identity::upsert_admin_user(
        &store,
        Some("admin_local_default"),
        "admin@sdkwork.local",
        "Admin Operator",
        Some("ChangeMe123!"),
        Some(sdkwork_api_domain_identity::AdminUserRole::SuperAdmin),
        true,
    )
    .await
    .unwrap();
    pool
}

async fn login_token(app: axum::Router) -> String {
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"email\":\"admin@sdkwork.local\",\"password\":\"ChangeMe123!\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    read_json(response).await["token"]
        .as_str()
        .unwrap()
        .to_owned()
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

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_async_job_callbacks (
            callback_id INTEGER PRIMARY KEY NOT NULL,
            job_id TEXT NOT NULL,
            event_type TEXT NOT NULL,
            dedupe_key TEXT,
            payload_json TEXT NOT NULL,
            status TEXT NOT NULL,
            received_at_ms INTEGER NOT NULL,
            processed_at_ms INTEGER
        )",
    )
    .execute(pool)
    .await
    .unwrap();
}

async fn seed_async_job_fixture(pool: &SqlitePool) {
    create_async_job_tables(pool).await;

    sqlx::query(
        "INSERT INTO ai_async_jobs (
            job_id, tenant_id, organization_id, user_id, account_id, request_id, provider_id,
            model_code, capability_code, modality, operation_kind, status, external_job_id,
            idempotency_key, callback_url, input_summary, progress_percent, error_code,
            error_message, created_at_ms, updated_at_ms, started_at_ms, completed_at_ms
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind("job_video_render_1")
    .bind(1001_i64)
    .bind(2002_i64)
    .bind(3003_i64)
    .bind(7001_i64)
    .bind(8801_i64)
    .bind("provider-openrouter")
    .bind("google-veo-3")
    .bind("videos")
    .bind("video")
    .bind("generation")
    .bind("running")
    .bind("upstream-job-1")
    .bind("idem-job-1")
    .bind("https://merchant.example.com/callbacks/video")
    .bind("Generate a product teaser")
    .bind(56_i64)
    .bind(Option::<String>::None)
    .bind(Option::<String>::None)
    .bind(1_710_000_000_000_i64)
    .bind(1_710_000_030_000_i64)
    .bind(1_710_000_005_000_i64)
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
    .bind(5101_i64)
    .bind("job_video_render_1")
    .bind(1_i64)
    .bind("running")
    .bind("provider_api")
    .bind("https://provider.example.com/v1/videos")
    .bind("upstream-job-1")
    .bind(1_710_000_005_000_i64)
    .bind(Option::<i64>::None)
    .bind(Option::<String>::None)
    .bind(1_710_000_005_000_i64)
    .bind(1_710_000_020_000_i64)
    .execute(pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO ai_async_job_assets (
            asset_id, job_id, asset_kind, storage_key, download_url, mime_type, size_bytes,
            checksum_sha256, created_at_ms
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind("asset_video_master_1")
    .bind("job_video_render_1")
    .bind("video")
    .bind("tenant-1001/jobs/job_video_render_1/master.mp4")
    .bind("https://cdn.example.com/jobs/job_video_render_1/master.mp4")
    .bind("video/mp4")
    .bind(24_576_i64)
    .bind("sha256-video-master")
    .bind(1_710_000_040_000_i64)
    .execute(pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO ai_async_job_callbacks (
            callback_id, job_id, event_type, dedupe_key, payload_json, status, received_at_ms,
            processed_at_ms
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(6101_i64)
    .bind("job_video_render_1")
    .bind("provider.progress")
    .bind("provider.progress:upstream-job-1:56")
    .bind("{\"progress\":56}")
    .bind("processed")
    .bind(1_710_000_025_000_i64)
    .bind(1_710_000_026_000_i64)
    .execute(pool)
    .await
    .unwrap();
}

#[tokio::test]
async fn admin_async_jobs_routes_expose_job_attempt_asset_and_callback_inventory() {
    let pool = memory_pool().await;
    seed_async_job_fixture(&pool).await;

    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let jobs = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/async-jobs")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(jobs.status(), StatusCode::OK);
    let jobs_json = read_json(jobs).await;
    assert_eq!(jobs_json.as_array().unwrap().len(), 1);
    assert_eq!(jobs_json[0]["job_id"], "job_video_render_1");
    assert_eq!(jobs_json[0]["capability_code"], "videos");
    assert_eq!(jobs_json[0]["status"], "running");
    assert_eq!(jobs_json[0]["progress_percent"], 56);

    let attempts = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/async-jobs/job_video_render_1/attempts")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(attempts.status(), StatusCode::OK);
    let attempts_json = read_json(attempts).await;
    assert_eq!(attempts_json.as_array().unwrap().len(), 1);
    assert_eq!(attempts_json[0]["attempt_id"], 5101);
    assert_eq!(attempts_json[0]["runtime_kind"], "provider_api");
    assert_eq!(attempts_json[0]["status"], "running");

    let assets = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/async-jobs/job_video_render_1/assets")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(assets.status(), StatusCode::OK);
    let assets_json = read_json(assets).await;
    assert_eq!(assets_json.as_array().unwrap().len(), 1);
    assert_eq!(assets_json[0]["asset_id"], "asset_video_master_1");
    assert_eq!(assets_json[0]["asset_kind"], "video");

    let callbacks = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/async-jobs/job_video_render_1/callbacks")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(callbacks.status(), StatusCode::OK);
    let callbacks_json = read_json(callbacks).await;
    assert_eq!(callbacks_json.as_array().unwrap().len(), 1);
    assert_eq!(callbacks_json[0]["callback_id"], 6101);
    assert_eq!(callbacks_json[0]["event_type"], "provider.progress");
    assert_eq!(callbacks_json[0]["status"], "processed");
}
