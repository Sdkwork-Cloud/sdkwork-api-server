use sdkwork_api_domain_identity::GatewayApiKey;

#[test]
fn revoked_key_is_not_active() {
    let mut key = GatewayApiKey::new("tenant-1", "project-1", "live");
    key.revoke();
    assert!(!key.is_active());
}
