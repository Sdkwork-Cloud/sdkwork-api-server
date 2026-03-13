use sdkwork_api_config::StandaloneConfig;
use sdkwork_api_interface_admin::admin_router_with_pool_and_master_key;
use sdkwork_api_storage_sqlite::run_migrations;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = StandaloneConfig::default();
    let pool = run_migrations(&config.database_url).await?;
    let listener = TcpListener::bind(&config.admin_bind).await?;
    axum::serve(
        listener,
        admin_router_with_pool_and_master_key(pool, config.credential_master_key),
    )
    .await?;
    Ok(())
}
