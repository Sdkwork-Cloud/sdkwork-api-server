use sdkwork_api_app_gateway::create_moderation;

#[test]
fn moderation_requires_moderation_backend() {
    let error = create_moderation("tenant-1", "project-1", "omni-moderation-latest").unwrap_err();
    assert!(error.to_string().contains("not supported"));
}
