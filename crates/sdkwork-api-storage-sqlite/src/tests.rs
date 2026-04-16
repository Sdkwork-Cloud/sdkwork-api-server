use sdkwork_api_domain_identity::{ApiKeyGroupRecord, GatewayApiKeyRecord};
use sdkwork_api_domain_jobs::{
    AsyncJobAssetRecord, AsyncJobAttemptRecord, AsyncJobAttemptStatus, AsyncJobCallbackRecord,
    AsyncJobCallbackStatus, AsyncJobRecord, AsyncJobStatus,
};
use sdkwork_api_domain_routing::ProviderHealthSnapshot;
use std::path::PathBuf;
use std::{env, fs};

use super::{run_migrations, sqlite_path_from_url, SqliteAdminStore};

#[cfg(windows)]
#[test]
fn parses_windows_drive_file_sqlite_urls_without_a_leading_separator() {
    let path = sqlite_path_from_url("sqlite:///D:/sdkwork/router/sdkwork-api-server.db")
        .expect("expected sqlite file path");

    assert_eq!(
        path,
        PathBuf::from("D:/sdkwork/router/sdkwork-api-server.db")
    );
}

#[test]
fn routing_store_uses_shared_routing_assessment_codecs() {
    let routing_store = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/routing_store.rs"),
    )
    .expect("read routing_store source");
    let sqlite_support = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/sqlite_support.rs"),
    )
    .expect("read sqlite_support source");

    for signature in [
        "fn encode_routing_assessments(",
        "fn decode_routing_assessments(",
    ] {
        assert!(
            !routing_store.contains(signature),
            "routing_store should use shared sqlite_support helper `{signature}`",
        );
        assert!(
            sqlite_support.contains(signature),
            "sqlite_support should define shared helper `{signature}`",
        );
    }
}

#[tokio::test]
async fn run_migrations_configures_file_sqlite_databases_for_multi_process_runtime_use() {
    let unique_id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let database_path = env::temp_dir()
        .join("sdkwork-router-sqlite-tests")
        .join(format!("runtime-mode-{unique_id}.db"));
    let database_url = format!("sqlite://{}", database_path.to_string_lossy().replace('\\', "/"));

    let pool = run_migrations(&database_url).await.unwrap();

    let journal_mode: String = sqlx::query_scalar("PRAGMA journal_mode;")
        .fetch_one(&pool)
        .await
        .unwrap();
    let busy_timeout_ms: i64 = sqlx::query_scalar("PRAGMA busy_timeout;")
        .fetch_one(&pool)
        .await
        .unwrap();

    assert_eq!(journal_mode.to_ascii_lowercase(), "wal");
    assert_eq!(busy_timeout_ms, 5_000);

    drop(pool);
    let _ = fs::remove_file(database_path);
}

#[tokio::test]
async fn api_key_group_records_round_trip_through_sqlite_store() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    let group = ApiKeyGroupRecord::new(
        "group-live",
        "tenant-1",
        "project-1",
        "live",
        "Production Keys",
        "production-keys",
    )
    .with_description("Primary production pool")
    .with_color("#2563eb")
    .with_default_capability_scope("chat,responses")
    .with_created_at_ms(1_700_000_000_000)
    .with_updated_at_ms(1_700_000_000_500);

    store.insert_api_key_group(&group).await.unwrap();

    let found = store
        .find_api_key_group("group-live")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(found, group);

    let groups = store.list_api_key_groups().await.unwrap();
    assert_eq!(groups, vec![group]);
}

#[tokio::test]
async fn async_job_records_round_trip_through_sqlite_store() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    let job = AsyncJobRecord::new(
        "job_multimodal_1",
        1001,
        2002,
        3003,
        "videos",
        "video",
        "generation",
        1_710_000_000_000,
    )
    .with_account_id(Some(4004))
    .with_request_id(Some(5005))
    .with_provider_id(Some("provider-openrouter".to_owned()))
    .with_model_code(Some("google-veo-3".to_owned()))
    .with_status(AsyncJobStatus::Running)
    .with_external_job_id(Some("upstream-job-1".to_owned()))
    .with_idempotency_key(Some("idem-job-1".to_owned()))
    .with_callback_url(Some("https://merchant.example.com/job-callback".to_owned()))
    .with_input_summary(Some("Render launch trailer".to_owned()))
    .with_progress_percent(Some(42))
    .with_updated_at_ms(1_710_000_010_000)
    .with_started_at_ms(Some(1_710_000_005_000));

    let attempt = AsyncJobAttemptRecord::new(
        7001,
        job.job_id.clone(),
        1,
        "provider_api",
        1_710_000_005_000,
    )
    .with_status(AsyncJobAttemptStatus::Running)
    .with_endpoint(Some("https://provider.example.com/v1/videos".to_owned()))
    .with_external_job_id(Some("upstream-job-1".to_owned()))
    .with_claimed_at_ms(Some(1_710_000_005_000))
    .with_updated_at_ms(1_710_000_008_000);

    let asset = AsyncJobAssetRecord::new(
        "asset_video_master_1",
        job.job_id.clone(),
        "video",
        "tenant-1001/jobs/job_multimodal_1/master.mp4",
        1_710_000_020_000,
    )
    .with_download_url(Some(
        "https://cdn.example.com/jobs/job_multimodal_1/master.mp4".to_owned(),
    ))
    .with_mime_type(Some("video/mp4".to_owned()))
    .with_size_bytes(Some(24_576))
    .with_checksum_sha256(Some("sha256-video-master".to_owned()));

    let callback = AsyncJobCallbackRecord::new(
        8001,
        job.job_id.clone(),
        "provider.progress",
        "{\"progress\":42}",
        1_710_000_009_000,
    )
    .with_dedupe_key(Some("provider.progress:upstream-job-1:42".to_owned()))
    .with_status(AsyncJobCallbackStatus::Processed)
    .with_processed_at_ms(Some(1_710_000_009_500));

    store.insert_async_job(&job).await.unwrap();
    store.insert_async_job_attempt(&attempt).await.unwrap();
    store.insert_async_job_asset(&asset).await.unwrap();
    store.insert_async_job_callback(&callback).await.unwrap();

    assert_eq!(
        store.find_async_job(&job.job_id).await.unwrap().as_ref(),
        Some(&job)
    );
    assert_eq!(store.list_async_jobs().await.unwrap(), vec![job]);
    assert_eq!(
        store
            .list_async_job_attempts("job_multimodal_1")
            .await
            .unwrap(),
        vec![attempt]
    );
    assert_eq!(
        store
            .list_async_job_assets("job_multimodal_1")
            .await
            .unwrap(),
        vec![asset]
    );
    assert_eq!(
        store
            .list_async_job_callbacks("job_multimodal_1")
            .await
            .unwrap(),
        vec![callback]
    );
}

#[tokio::test]
async fn api_key_group_membership_round_trips_and_legacy_keys_remain_ungrouped() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    let group = ApiKeyGroupRecord::new(
        "group-live",
        "tenant-1",
        "project-1",
        "live",
        "Production Keys",
        "production-keys",
    )
    .with_created_at_ms(1_700_000_000_000)
    .with_updated_at_ms(1_700_000_000_000);
    store.insert_api_key_group(&group).await.unwrap();

    let grouped_key =
        GatewayApiKeyRecord::new("tenant-1", "project-1", "live", "hashed-grouped")
            .with_api_key_group_id("group-live")
            .with_label("Grouped key")
            .with_created_at_ms(1_700_000_000_100);
    store.insert_gateway_api_key(&grouped_key).await.unwrap();

    let legacy_key = GatewayApiKeyRecord::new("tenant-1", "project-1", "live", "hashed-legacy")
        .with_label("Legacy key")
        .with_created_at_ms(1_700_000_000_200);
    store.insert_gateway_api_key(&legacy_key).await.unwrap();

    let keys = store.list_gateway_api_keys().await.unwrap();
    assert_eq!(keys.len(), 2);
    assert_eq!(keys[0].hashed_key, "hashed-legacy");
    assert_eq!(keys[0].api_key_group_id, None);
    assert_eq!(keys[1].hashed_key, "hashed-grouped");
    assert_eq!(keys[1].api_key_group_id.as_deref(), Some("group-live"));
}

#[tokio::test]
async fn provider_health_snapshots_replace_existing_record_for_same_provider_runtime_and_instance(
) {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    let original = ProviderHealthSnapshot::new(
        "provider-openai-official",
        "sdkwork.provider.openai.official",
        "builtin",
        1_000,
    )
    .with_running(true)
    .with_healthy(false)
    .with_message("first failure");
    let replacement = ProviderHealthSnapshot::new(
        "provider-openai-official",
        "sdkwork.provider.openai.official",
        "builtin",
        2_000,
    )
    .with_running(true)
    .with_healthy(true)
    .with_message("recovered");

    store
        .insert_provider_health_snapshot(&original)
        .await
        .unwrap();
    store
        .insert_provider_health_snapshot(&replacement)
        .await
        .unwrap();

    let snapshots = store.list_provider_health_snapshots().await.unwrap();
    assert_eq!(snapshots, vec![replacement]);
}
