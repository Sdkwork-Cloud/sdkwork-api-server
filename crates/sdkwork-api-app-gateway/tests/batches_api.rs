use sdkwork_api_app_gateway::create_batch;
use sdkwork_api_contract_openai::batches::CreateBatchRequest;

#[test]
fn returns_batch_object() {
    let request = CreateBatchRequest::new("file_local_0000000000000001", "/v1/responses", "24h");
    let response = create_batch("tenant-1", "project-1", &request).unwrap();
    assert_eq!(response.object, "batch");
    assert_eq!(response.endpoint, "/v1/responses");
    assert_eq!(response.input_file_id, "file_local_0000000000000001");
}

#[test]
fn lists_batch_objects() {
    let response = sdkwork_api_app_gateway::list_batches("tenant-1", "project-1").unwrap();
    assert_eq!(response.object, "list");
    assert!(response.data.is_empty());
}

#[test]
fn retrieving_batch_without_persistence_fails() {
    let error =
        sdkwork_api_app_gateway::get_batch("tenant-1", "project-1", "batch_local_0000000000000001")
            .unwrap_err();
    assert!(error.to_string().contains("batch not found"));
}

#[test]
fn cancelling_batch_without_persistence_fails() {
    let error = sdkwork_api_app_gateway::cancel_batch(
        "tenant-1",
        "project-1",
        "batch_local_0000000000000001",
    )
    .unwrap_err();
    assert!(error.to_string().contains("batch not found"));
}
