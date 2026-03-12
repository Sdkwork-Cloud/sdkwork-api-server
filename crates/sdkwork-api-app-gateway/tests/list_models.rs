use sdkwork_api_app_gateway::list_models;

#[test]
fn returns_platform_models() {
    let response = list_models("tenant-1", "project-1").unwrap();
    assert_eq!(response.object, "list");
}
