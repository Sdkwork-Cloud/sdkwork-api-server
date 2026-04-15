use sdkwork_api_contract_openai::uploads::{
    AddUploadPartRequest, CompleteUploadRequest, CreateUploadRequest,
};

fn assert_error_contains<T: std::fmt::Debug, E: std::fmt::Display>(
    result: Result<T, E>,
    expected: &str,
) {
    let error = result.expect_err("expected error");
    assert!(
        error.to_string().contains(expected),
        "expected error containing `{expected}`, got `{error}`"
    );
}

#[test]
fn local_upload_fallback_requires_upstream_provider() {
    let request = CreateUploadRequest::new("batch", "input.jsonl", "application/jsonl", 1024);
    assert_error_contains(
        sdkwork_api_app_gateway::create_upload("tenant-1", "project-1", &request),
        "Local upload fallback is not supported",
    );
}

#[test]
fn local_upload_part_fallback_requires_persisted_state() {
    let request = AddUploadPartRequest::new("upload_local_1", b"part-data".to_vec());
    assert_error_contains(
        sdkwork_api_app_gateway::create_upload_part("tenant-1", "project-1", &request),
        "Persisted local upload part state is required",
    );
}

#[test]
fn local_upload_completion_requires_persisted_state() {
    let request = CompleteUploadRequest::new("upload_local_1", vec!["part_local_1", "part_local_2"]);
    assert_error_contains(
        sdkwork_api_app_gateway::complete_upload("tenant-1", "project-1", &request),
        "upload not found",
    );
    assert_error_contains(
        sdkwork_api_app_gateway::cancel_upload("tenant-1", "project-1", "upload_local_1"),
        "upload not found",
    );
}
