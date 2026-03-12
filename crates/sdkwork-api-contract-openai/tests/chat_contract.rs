use sdkwork_api_contract_openai::chat_completions::ChatCompletionChunk;

#[test]
fn serializes_chunk_object() {
    let json = serde_json::to_value(ChatCompletionChunk::empty("chatcmpl-1", "gpt-4.1")).unwrap();
    assert_eq!(json["object"], "chat.completion.chunk");
}
