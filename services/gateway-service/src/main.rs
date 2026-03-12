use sdkwork_api_config::StandaloneConfig;
use sdkwork_api_interface_http::gateway_router;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = StandaloneConfig::default();
    let listener = TcpListener::bind(&config.gateway_bind).await?;
    axum::serve(listener, gateway_router()).await?;
    Ok(())
}
