use sdkwork_api_contract_openai::uploads::{
    AddUploadPartRequest, CompleteUploadRequest, CreateUploadRequest,
};

#[test]
fn returns_upload_object() {
    let request = CreateUploadRequest::new("batch", "input.jsonl", "application/jsonl", 1024);
    let response =
        sdkwork_api_app_gateway::create_upload("tenant-1", "project-1", &request).unwrap();
    assert_eq!(response.object, "upload");
    assert_eq!(response.filename, "input.jsonl");
    assert_eq!(response.bytes, 1024);
}

#[test]
fn returns_upload_part_object() {
    let request = AddUploadPartRequest::new("upload_1", b"part-data".to_vec());
    let response =
        sdkwork_api_app_gateway::create_upload_part("tenant-1", "project-1", &request).unwrap();
    assert_eq!(response.object, "upload.part");
    assert_eq!(response.upload_id, "upload_1");
}

#[test]
fn completes_upload_object() {
    let request = CompleteUploadRequest::new("upload_1", vec!["part_1", "part_2"]);
    let response =
        sdkwork_api_app_gateway::complete_upload("tenant-1", "project-1", &request).unwrap();
    assert_eq!(response.object, "upload");
    assert_eq!(response.part_ids.len(), 2);
}

#[test]
fn cancels_upload_object() {
    let response =
        sdkwork_api_app_gateway::cancel_upload("tenant-1", "project-1", "upload_1").unwrap();
    assert_eq!(response.object, "upload");
    assert_eq!(response.id, "upload_1");
    assert_eq!(response.status, "cancelled");
}
