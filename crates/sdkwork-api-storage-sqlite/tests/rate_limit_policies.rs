use sdkwork_api_domain_rate_limit::RateLimitPolicy;
use sdkwork_api_storage_sqlite::{run_migrations, SqliteAdminStore};

#[tokio::test]
async fn sqlite_store_persists_rate_limit_policies() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    let policy = RateLimitPolicy::new("rate-project-1", "project-1", 60, 60)
        .with_burst_requests(120)
        .with_enabled(true);
    store.insert_rate_limit_policy(&policy).await.unwrap();

    let policies = store
        .list_rate_limit_policies_for_project("project-1")
        .await
        .unwrap();
    assert_eq!(policies, vec![policy]);
}
