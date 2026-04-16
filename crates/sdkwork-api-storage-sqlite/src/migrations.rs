use super::*;
use std::time::Duration;

pub async fn run_migrations(url: &str) -> Result<SqlitePool> {
    ensure_sqlite_parent_directory(url)?;
    let mut options = sqlx::sqlite::SqliteConnectOptions::from_str(url)?
        .busy_timeout(Duration::from_millis(5_000));
    if sqlite_path_from_url(url).is_some() {
        options = options
            .create_if_missing(true)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
            .synchronous(sqlx::sqlite::SqliteSynchronous::Normal);
    }
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await?;
    apply_sqlite_identity_schema(&pool).await?;
    apply_sqlite_marketing_schema(&pool).await?;
    apply_sqlite_routing_schema(&pool).await?;
    apply_sqlite_billing_schema(&pool).await?;
    apply_sqlite_payment_schema(&pool).await?;
    apply_sqlite_commerce_jobs_schema(&pool).await?;
    apply_sqlite_catalog_gateway_schema(&pool).await?;
    apply_sqlite_runtime_schema(&pool).await?;
    apply_sqlite_catalog_gateway_compatibility(&pool).await?;
    seed_sqlite_builtin_channels(&pool).await?;
    Ok(pool)
}
