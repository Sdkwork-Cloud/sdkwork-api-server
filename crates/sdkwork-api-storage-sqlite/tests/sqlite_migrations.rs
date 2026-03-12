use sdkwork_api_storage_sqlite::run_migrations;

#[tokio::test]
async fn creates_identity_and_tenant_tables() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let row: (String,) =
        sqlx::query_as("select name from sqlite_master where name = 'identity_users'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(row.0, "identity_users");
}
