use sdkwork_api_config::RuntimeMode;

#[test]
fn defaults_to_server_mode() {
    assert_eq!(RuntimeMode::default(), RuntimeMode::Server);
}
