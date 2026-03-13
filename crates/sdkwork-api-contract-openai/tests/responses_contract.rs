use sdkwork_api_contract_openai::responses::{
    CompactResponseRequest, CountResponseInputTokensRequest, DeleteResponseResponse,
    ListResponseInputItemsResponse, ResponseCompactionObject, ResponseInputItemObject,
    ResponseInputTokensObject, ResponseObject,
};

#[test]
fn serializes_response_object() {
    let json = serde_json::to_value(ResponseObject::empty("resp_1", "gpt-4.1")).unwrap();
    assert_eq!(json["object"], "response");
}

#[test]
fn serializes_response_extension_contracts() {
    let list =
        ListResponseInputItemsResponse::new(vec![ResponseInputItemObject::message("item_1")]);
    let list_json = serde_json::to_value(list).unwrap();
    assert_eq!(list_json["object"], "list");
    assert_eq!(list_json["data"][0]["id"], "item_1");

    let deleted = DeleteResponseResponse::deleted("resp_1");
    let deleted_json = serde_json::to_value(deleted).unwrap();
    assert_eq!(deleted_json["object"], "response.deleted");
    assert_eq!(deleted_json["deleted"], true);
}

#[test]
fn serializes_remaining_response_official_contracts() {
    let input_tokens_request =
        CountResponseInputTokensRequest::new("gpt-4.1", serde_json::json!("hello"));
    let input_tokens_request_json = serde_json::to_value(input_tokens_request).unwrap();
    assert_eq!(input_tokens_request_json["model"], "gpt-4.1");
    assert_eq!(input_tokens_request_json["input"], "hello");

    let input_tokens = ResponseInputTokensObject::new(42);
    let input_tokens_json = serde_json::to_value(input_tokens).unwrap();
    assert_eq!(input_tokens_json["object"], "response.input_tokens");
    assert_eq!(input_tokens_json["input_tokens"], 42);

    let cancelled = ResponseObject::cancelled("resp_1", "gpt-4.1");
    let cancelled_json = serde_json::to_value(cancelled).unwrap();
    assert_eq!(cancelled_json["status"], "cancelled");

    let compact_request = CompactResponseRequest::new("gpt-4.1", serde_json::json!("hello"));
    let compact_request_json = serde_json::to_value(compact_request).unwrap();
    assert_eq!(compact_request_json["model"], "gpt-4.1");
    assert_eq!(compact_request_json["input"], "hello");

    let compaction = ResponseCompactionObject::new("resp_cmp_1", "gpt-4.1");
    let compaction_json = serde_json::to_value(compaction).unwrap();
    assert_eq!(compaction_json["object"], "response.compaction");
}
