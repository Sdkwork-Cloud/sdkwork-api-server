use sdkwork_api_config::RuntimeMode;
use sdkwork_api_config::StandaloneConfig;

#[test]
fn defaults_to_server_mode() {
    assert_eq!(RuntimeMode::default(), RuntimeMode::Server);
}

#[test]
fn standalone_defaults_are_local_friendly() {
    let config = StandaloneConfig::default();
    assert_eq!(config.gateway_bind, "127.0.0.1:8080");
    assert_eq!(config.admin_bind, "127.0.0.1:8081");
    assert_eq!(config.database_url, "sqlite://sdkwork-api-server.db");
}
