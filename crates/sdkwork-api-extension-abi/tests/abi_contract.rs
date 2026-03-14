use sdkwork_api_extension_abi::{
    ProviderInvocation, ProviderInvocationResult, ProviderStreamInvocationResult,
    SDKWORK_EXTENSION_ABI_VERSION,
};

#[test]
fn serializes_provider_invocation_and_result_envelopes() {
    let invocation = ProviderInvocation::new(
        "chat.completions.create",
        "sk-upstream",
        "https://example.com/v1",
        vec!["chatcmpl_1".to_owned()],
        serde_json::json!({"model":"gpt-4.1"}),
        false,
    );

    let encoded_invocation = serde_json::to_value(&invocation).expect("invocation json");
    assert_eq!(encoded_invocation["operation"], "chat.completions.create");
    assert_eq!(encoded_invocation["api_key"], "sk-upstream");

    let result = ProviderInvocationResult::json(serde_json::json!({"id":"chatcmpl_native"}));
    let encoded_result = serde_json::to_value(&result).expect("result json");

    assert_eq!(SDKWORK_EXTENSION_ABI_VERSION, 1);
    assert_eq!(encoded_result["kind"], "json");
    assert_eq!(encoded_result["body"]["id"], "chatcmpl_native");
}

#[test]
fn serializes_provider_stream_result_envelopes() {
    let result = ProviderStreamInvocationResult::streamed("text/event-stream");
    let encoded_result = serde_json::to_value(&result).expect("stream result json");

    assert_eq!(encoded_result["kind"], "streamed");
    assert_eq!(encoded_result["content_type"], "text/event-stream");
}
