use sdkwork_api_app_gateway::{cancel_batch, create_batch, get_batch, list_batches};
use sdkwork_api_contract_openai::batches::CreateBatchRequest;

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
fn local_batch_fallback_requires_upstream_provider() {
    let request = CreateBatchRequest::new("file_local_0000000000000001", "/v1/responses", "24h");
    assert_error_contains(
        create_batch("tenant-1", "project-1", &request),
        "Local batch fallback is not supported",
    );
    assert_error_contains(
        list_batches("tenant-1", "project-1"),
        "Local batch listing fallback is not supported",
    );
}

#[test]
fn local_batch_fallback_requires_persisted_batch_state() {
    assert_error_contains(
        get_batch("tenant-1", "project-1", "batch_local_0000000000000001"),
        "batch not found",
    );
    assert_error_contains(
        cancel_batch("tenant-1", "project-1", "batch_local_0000000000000001"),
        "batch not found",
    );
}
