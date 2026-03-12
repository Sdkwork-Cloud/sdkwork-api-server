use sdkwork_api_config::StandaloneConfig;
use sdkwork_api_interface_http::gateway_router_with_pool;
use sdkwork_api_storage_sqlite::run_migrations;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = StandaloneConfig::default();
    let pool = run_migrations(&config.database_url).await?;
    let listener = TcpListener::bind(&config.gateway_bind).await?;
    axum::serve(listener, gateway_router_with_pool(pool)).await?;
    Ok(())
}
