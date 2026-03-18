use sdkwork_api_app_usage::record_usage;
use sdkwork_api_app_usage::{
    list_usage_records, persist_usage_record, persist_usage_record_with_tokens,
    record_usage_with_tokens,
};
use sdkwork_api_storage_sqlite::{run_migrations, SqliteAdminStore};

#[test]
fn usage_record_contains_metering_details() {
    let usage = record_usage("project-1", "gpt-4.1", "provider-openai-official", 42, 0.21).unwrap();
    assert_eq!(usage.model, "gpt-4.1");
    assert_eq!(usage.units, 42);
    assert_eq!(usage.amount, 0.21);
    assert!(usage.created_at_ms > 0);
}

#[test]
fn usage_record_contains_token_details_when_available() {
    let usage = record_usage_with_tokens(
        "project-1",
        "gpt-4.1",
        "provider-openai-official",
        42,
        0.21,
        120,
        80,
        200,
    )
    .unwrap();
    assert_eq!(usage.input_tokens, 120);
    assert_eq!(usage.output_tokens, 80);
    assert_eq!(usage.total_tokens, 200);
}

#[tokio::test]
async fn persisted_usage_can_be_listed() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    persist_usage_record(
        &store,
        "project-1",
        "gpt-4.1",
        "provider-openai-official",
        128,
        0.64,
    )
    .await
    .unwrap();

    let records = list_usage_records(&store).await.unwrap();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].provider, "provider-openai-official");
    assert_eq!(records[0].units, 128);
    assert_eq!(records[0].amount, 0.64);
    assert!(records[0].created_at_ms > 0);
}

#[tokio::test]
async fn persisted_usage_with_tokens_can_be_listed() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    persist_usage_record_with_tokens(
        &store,
        "project-1",
        "gpt-4.1",
        "provider-openai-official",
        128,
        0.64,
        240,
        60,
        300,
    )
    .await
    .unwrap();

    let records = list_usage_records(&store).await.unwrap();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].input_tokens, 240);
    assert_eq!(records[0].output_tokens, 60);
    assert_eq!(records[0].total_tokens, 300);
}
