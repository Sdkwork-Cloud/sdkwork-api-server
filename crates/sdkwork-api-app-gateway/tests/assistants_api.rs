use sdkwork_api_app_gateway::{
    create_assistant, delete_assistant, get_assistant, list_assistants, update_assistant,
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
fn local_assistant_fallback_requires_upstream_provider() {
    assert_error_contains(
        create_assistant("tenant-1", "project-1", "Support", "gpt-4.1"),
        "Local assistant fallback is not supported",
    );
    assert_error_contains(
        list_assistants("tenant-1", "project-1"),
        "Local assistant listing fallback is not supported",
    );
}

#[test]
fn local_assistant_fallback_requires_persisted_assistant_state() {
    assert_error_contains(
        get_assistant("tenant-1", "project-1", "asst_local_1"),
        "assistant not found",
    );
    assert_error_contains(
        update_assistant("tenant-1", "project-1", "asst_local_1", "Support v2"),
        "assistant not found",
    );
    assert_error_contains(
        delete_assistant("tenant-1", "project-1", "asst_local_1"),
        "assistant not found",
    );
}

#[test]
fn local_assistant_create_requires_name() {
    assert_error_contains(
        create_assistant("tenant-1", "project-1", "   ", "gpt-4.1"),
        "Assistant name is required",
    );
}

#[test]
fn local_assistant_create_requires_model() {
    assert_error_contains(
        create_assistant("tenant-1", "project-1", "Support", "   "),
        "Assistant model is required",
    );
}
