use sdkwork_api_app_gateway::{
    create_thread, create_thread_message, delete_thread, delete_thread_message, get_thread,
    get_thread_message, list_thread_messages, update_thread, update_thread_message,
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
fn local_thread_fallback_returns_placeholder_thread_objects() {
    let created =
        create_thread("tenant-1", "project-1").expect("local fallback should create a thread");
    assert_eq!(created.id, "thread_1");
    assert_eq!(created.object, "thread");
    assert_eq!(created.metadata.unwrap()["workspace"], "default");

    let retrieved = get_thread("tenant-1", "project-1", "thread_1")
        .expect("local fallback should retrieve a placeholder thread");
    assert_eq!(retrieved.id, "thread_1");
    assert_eq!(retrieved.object, "thread");

    let updated = update_thread("tenant-1", "project-1", "thread_1")
        .expect("local fallback should update a placeholder thread");
    assert_eq!(updated.id, "thread_1");
    assert_eq!(updated.metadata.unwrap()["workspace"], "next");

    let deleted = delete_thread("tenant-1", "project-1", "thread_1")
        .expect("local fallback should delete a placeholder thread");
    assert_eq!(deleted.id, "thread_1");
    assert!(deleted.deleted);
}

#[test]
fn local_thread_message_fallback_returns_placeholder_message_objects() {
    let created = create_thread_message("tenant-1", "project-1", "thread_1", "user", "hello")
        .expect("local fallback should create a placeholder message");
    assert_eq!(created.id, "msg_1");
    assert_eq!(created.thread_id, "thread_1");
    assert_eq!(created.role, "user");
    assert_eq!(created.content[0].text.value, "hello");

    let listed = list_thread_messages("tenant-1", "project-1", "thread_1")
        .expect("local fallback should list placeholder messages");
    assert_eq!(listed.data.len(), 1);
    assert_eq!(listed.data[0].id, "msg_1");

    let retrieved = get_thread_message("tenant-1", "project-1", "thread_1", "msg_1")
        .expect("local fallback should retrieve a placeholder message");
    assert_eq!(retrieved.id, "msg_1");

    let updated = update_thread_message("tenant-1", "project-1", "thread_1", "msg_1")
        .expect("local fallback should update a placeholder message");
    assert_eq!(updated.id, "msg_1");

    let deleted = delete_thread_message("tenant-1", "project-1", "thread_1", "msg_1")
        .expect("local fallback should delete a placeholder message");
    assert_eq!(deleted.id, "msg_1");
    assert!(deleted.deleted);
}

#[test]
fn local_thread_fallback_returns_not_found_for_missing_ids() {
    assert_error_contains(
        get_thread("tenant-1", "project-1", "thread_missing"),
        "thread not found",
    );
    assert_error_contains(
        update_thread("tenant-1", "project-1", "thread_missing"),
        "thread not found",
    );
    assert_error_contains(
        delete_thread("tenant-1", "project-1", "thread_missing"),
        "thread not found",
    );
    assert_error_contains(
        create_thread_message("tenant-1", "project-1", "thread_missing", "user", "hello"),
        "thread not found",
    );
    assert_error_contains(
        list_thread_messages("tenant-1", "project-1", "thread_missing"),
        "thread not found",
    );
    assert_error_contains(
        get_thread_message("tenant-1", "project-1", "thread_1", "msg_missing"),
        "thread message not found",
    );
    assert_error_contains(
        update_thread_message("tenant-1", "project-1", "thread_1", "msg_missing"),
        "thread message not found",
    );
    assert_error_contains(
        delete_thread_message("tenant-1", "project-1", "thread_1", "msg_missing"),
        "thread message not found",
    );
}
