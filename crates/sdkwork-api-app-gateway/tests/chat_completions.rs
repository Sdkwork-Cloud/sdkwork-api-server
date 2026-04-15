use sdkwork_api_app_gateway::{
    create_chat_completion, delete_chat_completion, get_chat_completion,
    list_chat_completion_messages, list_chat_completions, update_chat_completion,
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
fn local_chat_completion_fallback_requires_upstream_provider() {
    assert_error_contains(
        create_chat_completion("tenant-1", "project-1", "gpt-4.1"),
        "Local chat completion fallback is not supported",
    );
    assert_error_contains(
        list_chat_completions("tenant-1", "project-1"),
        "Local chat completion listing fallback is not supported",
    );
}

#[test]
fn local_chat_completion_fallback_requires_persisted_state() {
    assert_error_contains(
        get_chat_completion("tenant-1", "project-1", "chatcmpl_local_1"),
        "chat completion not found",
    );
    assert_error_contains(
        update_chat_completion(
            "tenant-1",
            "project-1",
            "chatcmpl_local_1",
            serde_json::json!({"tier":"gold"}),
        ),
        "chat completion not found",
    );
    assert_error_contains(
        delete_chat_completion("tenant-1", "project-1", "chatcmpl_local_1"),
        "chat completion not found",
    );
}

#[test]
fn local_chat_completion_message_listing_requires_persisted_state() {
    assert_error_contains(
        list_chat_completion_messages("tenant-1", "project-1", "chatcmpl_local_1"),
        "Persisted local chat completion message state is required",
    );
}
