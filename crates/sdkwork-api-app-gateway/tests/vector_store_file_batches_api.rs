use sdkwork_api_app_gateway::{
    cancel_vector_store_file_batch, create_vector_store_file_batch, get_vector_store_file_batch,
    list_vector_store_file_batch_files,
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
fn local_vector_store_file_batch_fallback_requires_persisted_state() {
    assert_error_contains(
        create_vector_store_file_batch("tenant-1", "project-1", "vs_local_1", &["file_local_1"]),
        "Persisted local vector store file batch state is required",
    );
    assert_error_contains(
        get_vector_store_file_batch("tenant-1", "project-1", "vs_local_1", "vsfb_local_1"),
        "vector store file batch not found",
    );
    assert_error_contains(
        cancel_vector_store_file_batch("tenant-1", "project-1", "vs_local_1", "vsfb_local_1"),
        "vector store file batch not found",
    );
    assert_error_contains(
        list_vector_store_file_batch_files("tenant-1", "project-1", "vs_local_1", "vsfb_local_1"),
        "Persisted local vector store file batch state is required",
    );
}
