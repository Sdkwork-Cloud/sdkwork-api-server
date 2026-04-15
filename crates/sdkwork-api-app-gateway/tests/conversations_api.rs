use sdkwork_api_app_gateway::{
    create_conversation, create_conversation_items, delete_conversation, delete_conversation_item,
    get_conversation, get_conversation_item, list_conversation_items, list_conversations,
    update_conversation,
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
fn local_conversation_fallback_requires_upstream_provider() {
    assert_error_contains(
        create_conversation("tenant-1", "project-1"),
        "Local conversation fallback is not supported",
    );
    assert_error_contains(
        list_conversations("tenant-1", "project-1"),
        "Local conversation listing fallback is not supported",
    );
}

#[test]
fn local_conversation_fallback_requires_persisted_conversation_state() {
    assert_error_contains(
        get_conversation("tenant-1", "project-1", "conv_local_1"),
        "conversation not found",
    );
    assert_error_contains(
        update_conversation(
            "tenant-1",
            "project-1",
            "conv_local_1",
            Some(serde_json::json!({"workspace":"next"})),
        ),
        "conversation not found",
    );
    assert_error_contains(
        delete_conversation("tenant-1", "project-1", "conv_local_1"),
        "conversation not found",
    );
}

#[test]
fn local_conversation_update_requires_metadata() {
    assert_error_contains(
        update_conversation("tenant-1", "project-1", "conv_local_1", None),
        "Conversation metadata is required",
    );
}

#[test]
fn local_conversation_item_fallback_requires_persisted_state() {
    assert_error_contains(
        create_conversation_items("tenant-1", "project-1", "conv_local_1"),
        "Persisted local conversation item state is required",
    );
    assert_error_contains(
        list_conversation_items("tenant-1", "project-1", "conv_local_1"),
        "Persisted local conversation item state is required",
    );
    assert_error_contains(
        get_conversation_item("tenant-1", "project-1", "conv_local_1", "item_local_1"),
        "conversation item not found",
    );
    assert_error_contains(
        delete_conversation_item("tenant-1", "project-1", "conv_local_1", "item_local_1"),
        "conversation item not found",
    );
}
