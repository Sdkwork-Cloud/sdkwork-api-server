use sdkwork_api_app_gateway::{
    cancel_response, compact_response, count_response_input_tokens, create_response,
    delete_response, get_response, list_response_input_items,
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
fn local_response_fallback_requires_upstream_provider() {
    assert_error_contains(
        create_response("tenant-1", "project-1", "gpt-4.1"),
        "Local response fallback is not supported",
    );
    assert_error_contains(
        compact_response("tenant-1", "project-1", "gpt-4.1"),
        "Local response compaction fallback is not supported",
    );
}

#[test]
fn local_response_fallback_requires_persisted_response_state() {
    assert_error_contains(
        get_response("tenant-1", "project-1", "resp_local_1"),
        "response not found",
    );
    assert_error_contains(
        delete_response("tenant-1", "project-1", "resp_local_1"),
        "response not found",
    );
    assert_error_contains(
        cancel_response("tenant-1", "project-1", "resp_local_1"),
        "response not found",
    );
}

#[test]
fn local_response_input_item_listing_requires_persisted_state() {
    assert_error_contains(
        list_response_input_items("tenant-1", "project-1", "resp_local_1"),
        "Persisted local response input item state is required",
    );
}

#[test]
fn local_response_token_counting_is_not_supported() {
    assert_error_contains(
        count_response_input_tokens("tenant-1", "project-1", "gpt-4.1"),
        "Response input token counting is not supported",
    );
}
