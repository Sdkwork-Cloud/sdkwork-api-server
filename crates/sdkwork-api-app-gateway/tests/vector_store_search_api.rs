use sdkwork_api_app_gateway::search_vector_store;

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
fn local_vector_store_search_requires_persisted_index_state() {
    assert_error_contains(
        search_vector_store("tenant-1", "project-1", "vs_local_1", "reset password"),
        "Persisted local vector store index state is required",
    );
}

#[test]
fn local_vector_store_search_rejects_missing_vector_store() {
    assert_error_contains(
        search_vector_store("tenant-1", "project-1", "vs_missing", "reset password"),
        "vector store not found",
    );
}
