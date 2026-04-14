use sdkwork_api_app_gateway::create_embedding;

#[test]
fn embedding_requires_embedding_backend() {
    let error = create_embedding("tenant-1", "project-1", "text-embedding-3-large").unwrap_err();
    assert!(error.to_string().contains("not supported"));
}
