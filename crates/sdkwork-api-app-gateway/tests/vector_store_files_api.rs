use sdkwork_api_app_gateway::{
    create_vector_store_file, delete_vector_store_file, get_vector_store_file,
    list_vector_store_files,
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
fn local_vector_store_file_fallback_requires_persisted_state() {
    assert_error_contains(
        create_vector_store_file("tenant-1", "project-1", "vs_local_1", "file_local_1"),
        "Persisted local vector store file state is required",
    );
    assert_error_contains(
        list_vector_store_files("tenant-1", "project-1", "vs_local_1"),
        "Persisted local vector store file state is required",
    );
    assert_error_contains(
        get_vector_store_file("tenant-1", "project-1", "vs_local_1", "file_local_1"),
        "vector store file not found",
    );
    assert_error_contains(
        delete_vector_store_file("tenant-1", "project-1", "vs_local_1", "file_local_1"),
        "vector store file not found",
    );
}
