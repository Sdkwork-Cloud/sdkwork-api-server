use sdkwork_api_app_gateway::create_batch;

#[test]
fn returns_batch_object() {
    let response = create_batch("tenant-1", "project-1", "/v1/responses", "file_1").unwrap();
    assert_eq!(response.object, "batch");
    assert_eq!(response.endpoint, "/v1/responses");
    assert_eq!(response.input_file_id, "file_1");
}
