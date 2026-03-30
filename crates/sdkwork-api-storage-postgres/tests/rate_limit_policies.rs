use sdkwork_api_domain_rate_limit::RateLimitPolicy;
use sdkwork_api_storage_postgres::{run_migrations, PostgresAdminStore};

#[tokio::test]
async fn postgres_store_persists_rate_limit_policies_when_url_is_provided() {
    let Some(database_url) = std::env::var("SDKWORK_TEST_POSTGRES_URL").ok() else {
        return;
    };

    let pool = run_migrations(&database_url).await.unwrap();
    let store = PostgresAdminStore::new(pool);

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
