use sdkwork_api_app_gateway::create_vector_store;

#[test]
fn returns_vector_store_object() {
    let response = create_vector_store("tenant-1", "project-1", "kb-main").unwrap();
    assert_eq!(response.object, "vector_store");
    assert_eq!(response.name, "kb-main");
}
