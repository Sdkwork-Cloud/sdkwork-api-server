use std::sync::Arc;

use sdkwork_api_config::StandaloneConfig;
use sdkwork_api_interface_portal::portal_router_with_store_and_jwt_secret;
use sdkwork_api_observability::init_tracing;
use sdkwork_api_storage_core::{AdminStore, StorageDialect};
use sdkwork_api_storage_postgres::{run_migrations as run_postgres_migrations, PostgresAdminStore};
use sdkwork_api_storage_sqlite::{run_migrations, SqliteAdminStore};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing("portal-api-service");
    let config = StandaloneConfig::from_env()?;
    let store: Arc<dyn AdminStore> = match config.storage_dialect() {
        Some(StorageDialect::Sqlite) => {
            let pool = run_migrations(&config.database_url).await?;
            Arc::new(SqliteAdminStore::new(pool))
        }
        Some(StorageDialect::Postgres) => {
            let pool = run_postgres_migrations(&config.database_url).await?;
            Arc::new(PostgresAdminStore::new(pool))
        }
        Some(other) => {
            anyhow::bail!(
                "portal-api-service startup does not yet support storage dialect: {}",
                other.as_str()
            )
        }
        None => anyhow::bail!("portal-api-service received unsupported database URL scheme"),
    };

    let listener = TcpListener::bind(&config.portal_bind).await?;
    axum::serve(
        listener,
        portal_router_with_store_and_jwt_secret(store, config.portal_jwt_signing_secret),
    )
    .await?;
    Ok(())
}
