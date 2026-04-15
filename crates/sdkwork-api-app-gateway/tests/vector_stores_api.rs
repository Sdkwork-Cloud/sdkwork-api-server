use sdkwork_api_app_gateway::{
    create_vector_store, delete_vector_store, get_vector_store, list_vector_stores,
    update_vector_store,
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
fn local_vector_store_fallback_requires_upstream_provider() {
    assert_error_contains(
        create_vector_store("tenant-1", "project-1", "kb-main"),
        "Local vector store fallback is not supported",
    );
    assert_error_contains(
        list_vector_stores("tenant-1", "project-1"),
        "Local vector store listing fallback is not supported",
    );
}

#[test]
fn local_vector_store_fallback_requires_persisted_state() {
    assert_error_contains(
        get_vector_store("tenant-1", "project-1", "vs_local_1"),
        "vector store not found",
    );
    assert_error_contains(
        update_vector_store("tenant-1", "project-1", "vs_local_1", "kb-updated"),
        "vector store not found",
    );
    assert_error_contains(
        delete_vector_store("tenant-1", "project-1", "vs_local_1"),
        "vector store not found",
    );
}
