use super::*;

pub async fn run_migrations(url: &str) -> Result<SqlitePool> {
    ensure_sqlite_parent_directory(url)?;
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect(url)
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
