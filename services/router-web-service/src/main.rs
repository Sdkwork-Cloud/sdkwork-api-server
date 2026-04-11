use std::{env, path::PathBuf};

use sdkwork_api_observability::init_tracing;
use sdkwork_api_runtime_host::{serve_public_web, RuntimeHostConfig};

fn env_or(name: &str, default: &str) -> String {
    env::var(name).unwrap_or_else(|_| default.to_owned())
}

fn env_optional(name: &str) -> Option<String> {
    env::var(name).ok().and_then(|value| {
        let trimmed = value.trim().to_owned();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    })
}

fn env_path_or(name: &str, default: &str) -> PathBuf {
    env::var(name)
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(default))
}

fn main() -> anyhow::Result<()> {
    init_tracing("router-web-service");

    let mut config = RuntimeHostConfig::new(
        env_or("SDKWORK_WEB_BIND", "0.0.0.0:9983"),
        env_path_or("SDKWORK_ADMIN_SITE_DIR", "apps/sdkwork-router-admin/dist"),
        env_path_or("SDKWORK_PORTAL_SITE_DIR", "apps/sdkwork-router-portal/dist"),
        env_or("SDKWORK_ADMIN_PROXY_TARGET", "127.0.0.1:9981"),
        env_or("SDKWORK_PORTAL_PROXY_TARGET", "127.0.0.1:9982"),
        env_or("SDKWORK_GATEWAY_PROXY_TARGET", "127.0.0.1:9980"),
    );
    config.admin_site_proxy_upstream = env_optional("SDKWORK_ADMIN_SITE_PROXY_TARGET");
    config.portal_site_proxy_upstream = env_optional("SDKWORK_PORTAL_SITE_PROXY_TARGET");

    serve_public_web(config)
}
