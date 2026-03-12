use sdkwork_api_app_gateway::create_embedding;

#[test]
fn returns_embedding_list() {
    let response = create_embedding("tenant-1", "project-1", "text-embedding-3-large").unwrap();
    assert_eq!(response.object, "list");
}
