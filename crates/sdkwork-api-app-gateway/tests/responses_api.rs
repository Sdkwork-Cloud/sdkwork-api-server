use sdkwork_api_app_gateway::create_response;

#[test]
fn returns_response_object() {
    let response = create_response("tenant-1", "project-1", "gpt-4.1").unwrap();
    assert_eq!(response.object, "response");
}
