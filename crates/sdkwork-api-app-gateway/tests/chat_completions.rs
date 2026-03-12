use sdkwork_api_app_gateway::create_chat_completion;

#[test]
fn returns_chat_completion_response() {
    let response = create_chat_completion("tenant-1", "project-1", "gpt-4.1").unwrap();
    assert_eq!(response.object, "chat.completion");
}
