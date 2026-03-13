use std::sync::Arc;

use sdkwork_api_app_credential::CredentialSecretManager;
use sdkwork_api_config::StandaloneConfig;
use sdkwork_api_interface_admin::admin_router_with_store_and_secret_manager;
use sdkwork_api_storage_core::{AdminStore, StorageDialect};
use sdkwork_api_storage_postgres::{run_migrations as run_postgres_migrations, PostgresAdminStore};
use sdkwork_api_storage_sqlite::{run_migrations, SqliteAdminStore};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
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
                "admin-api-service startup does not yet support storage dialect: {}",
                other.as_str()
            )
        }
        None => anyhow::bail!("admin-api-service received unsupported database URL scheme"),
    };
    let secret_manager = CredentialSecretManager::new(
        config.secret_backend,
        config.credential_master_key.clone(),
        config.secret_local_file.clone(),
        config.secret_keyring_service.clone(),
    );
    let listener = TcpListener::bind(&config.admin_bind).await?;
    axum::serve(
        listener,
        admin_router_with_store_and_secret_manager(store, secret_manager),
    )
    .await?;
    Ok(())
}
