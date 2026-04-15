use sdkwork_api_contract_openai::files::CreateFileRequest;

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
fn local_file_fallback_requires_upstream_provider() {
    let request = CreateFileRequest::new("fine-tune", "train.jsonl", b"{}".to_vec());
    assert_error_contains(
        sdkwork_api_app_gateway::create_file("tenant-1", "project-1", &request),
        "Local file fallback is not supported",
    );
    assert_error_contains(
        sdkwork_api_app_gateway::list_files("tenant-1", "project-1"),
        "Local file listing fallback is not supported",
    );
}

#[test]
fn local_file_fallback_requires_persisted_file_state() {
    assert_error_contains(
        sdkwork_api_app_gateway::get_file("tenant-1", "project-1", "file_local_1"),
        "file not found",
    );
    assert_error_contains(
        sdkwork_api_app_gateway::delete_file("tenant-1", "project-1", "file_local_1"),
        "file not found",
    );
    assert_error_contains(
        sdkwork_api_app_gateway::file_content("tenant-1", "project-1", "file_local_1"),
        "file not found",
    );
}
