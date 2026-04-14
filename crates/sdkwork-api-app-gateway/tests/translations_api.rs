use sdkwork_api_app_gateway::create_translation;

#[test]
fn translation_requires_translation_backend() {
    let error = create_translation("tenant-1", "project-1", "gpt-4o-mini-transcribe").unwrap_err();
    assert!(error.to_string().contains("not supported"));
}
