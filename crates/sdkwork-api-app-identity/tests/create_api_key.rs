use sdkwork_api_app_identity::CreateGatewayApiKey;

#[test]
fn generated_key_has_sdkwork_prefix() {
    let created = CreateGatewayApiKey::execute("tenant-1", "project-1", "live").unwrap();
    assert!(created.plaintext.starts_with("skw_live_"));
}
