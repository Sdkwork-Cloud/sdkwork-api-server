use sdkwork_api_app_gateway::search_vector_store;

#[test]
fn searches_vector_store() {
    let response = search_vector_store("tenant-1", "project-1", "vs_1", "reset password").unwrap();
    assert_eq!(response.object, "list");
    assert_eq!(response.data[0].content[0].text, "reset password");
}
