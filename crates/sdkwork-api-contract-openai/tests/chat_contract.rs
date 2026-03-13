use sdkwork_api_contract_openai::chat_completions::{
    ChatCompletionChunk, ChatCompletionMessageObject, ChatCompletionResponse,
    DeleteChatCompletionResponse, ListChatCompletionMessagesResponse, ListChatCompletionsResponse,
    UpdateChatCompletionRequest,
};

#[test]
fn serializes_chunk_object() {
    let json = serde_json::to_value(ChatCompletionChunk::empty("chatcmpl-1", "gpt-4.1")).unwrap();
    assert_eq!(json["object"], "chat.completion.chunk");
}

#[test]
fn serializes_chat_completion_extension_contracts() {
    let update = UpdateChatCompletionRequest::new(serde_json::json!({"tier":"gold"}));
    let update_json = serde_json::to_value(update).unwrap();
    assert_eq!(update_json["metadata"]["tier"], "gold");

    let completion = ChatCompletionResponse::with_metadata(
        "chatcmpl-1",
        "gpt-4.1",
        serde_json::json!({"tier":"gold"}),
    );
    let completion_json = serde_json::to_value(completion).unwrap();
    assert_eq!(completion_json["metadata"]["tier"], "gold");

    let list = ListChatCompletionsResponse::new(vec![ChatCompletionResponse::empty(
        "chatcmpl-1",
        "gpt-4.1",
    )]);
    let list_json = serde_json::to_value(list).unwrap();
    assert_eq!(list_json["object"], "list");

    let deleted = DeleteChatCompletionResponse::deleted("chatcmpl-1");
    let deleted_json = serde_json::to_value(deleted).unwrap();
    assert_eq!(deleted_json["deleted"], true);
    assert_eq!(deleted_json["object"], "chat.completion.deleted");

    let messages =
        ListChatCompletionMessagesResponse::new(vec![ChatCompletionMessageObject::assistant(
            "msg_1", "hello",
        )]);
    let messages_json = serde_json::to_value(messages).unwrap();
    assert_eq!(messages_json["object"], "list");
    assert_eq!(
        messages_json["data"][0]["object"],
        "chat.completion.message"
    );
}
