use sdkwork_api_app_credential::save_credential;

#[test]
fn saves_upstream_credential_binding() {
    let credential =
        save_credential("tenant-1", "provider-openai-official", "cred-openai").unwrap();
    assert_eq!(credential.provider_id, "provider-openai-official");
}
