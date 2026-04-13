use super::*;

pub async fn run_migrations(url: &str) -> Result<PgPool> {
    let pool = PgPoolOptions::new().max_connections(5).connect(url).await?;
    apply_postgres_identity_schema(&pool).await?;
    apply_postgres_marketing_schema(&pool).await?;
    apply_postgres_routing_schema(&pool).await?;
    apply_postgres_billing_schema(&pool).await?;
    apply_postgres_payment_schema(&pool).await?;
    apply_postgres_commerce_jobs_schema(&pool).await?;
    apply_postgres_catalog_gateway_schema(&pool).await?;
    apply_postgres_runtime_schema(&pool).await?;
    apply_postgres_legacy_table_compatibility(&pool).await?;
    seed_postgres_builtin_channels(&pool).await?;
    migrate_postgres_legacy_catalog_records(&pool).await?;
    recreate_postgres_legacy_compatibility_views(&pool).await?;
    Ok(pool)
}
