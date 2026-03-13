use sdkwork_api_app_gateway::{
    create_chat_completion, delete_chat_completion, get_chat_completion,
    list_chat_completion_messages, list_chat_completions, update_chat_completion,
};

#[test]
fn returns_chat_completion_response() {
    let response = create_chat_completion("tenant-1", "project-1", "gpt-4.1").unwrap();
    assert_eq!(response.object, "chat.completion");
}

#[test]
fn lists_chat_completion_responses() {
    let response = list_chat_completions("tenant-1", "project-1").unwrap();
    assert_eq!(response.object, "list");
    assert_eq!(response.data[0].object, "chat.completion");
}

#[test]
fn retrieves_chat_completion_response() {
    let response = get_chat_completion("tenant-1", "project-1", "chatcmpl_1").unwrap();
    assert_eq!(response.id, "chatcmpl_1");
}

#[test]
fn updates_chat_completion_response() {
    let response = update_chat_completion(
        "tenant-1",
        "project-1",
        "chatcmpl_1",
        serde_json::json!({"tier":"gold"}),
    )
    .unwrap();
    assert_eq!(response.metadata, Some(serde_json::json!({"tier":"gold"})));
}

#[test]
fn deletes_chat_completion_response() {
    let response = delete_chat_completion("tenant-1", "project-1", "chatcmpl_1").unwrap();
    assert_eq!(response.id, "chatcmpl_1");
    assert!(response.deleted);
}

#[test]
fn lists_chat_completion_messages() {
    let response = list_chat_completion_messages("tenant-1", "project-1", "chatcmpl_1").unwrap();
    assert_eq!(response.object, "list");
    assert_eq!(response.data[0].object, "chat.completion.message");
}
