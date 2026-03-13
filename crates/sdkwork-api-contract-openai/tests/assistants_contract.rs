use sdkwork_api_contract_openai::assistants::{
    AssistantObject, CreateAssistantRequest, DeleteAssistantResponse, ListAssistantsResponse,
    UpdateAssistantRequest,
};

#[test]
fn serializes_assistant_resource_contracts() {
    let request = CreateAssistantRequest::new("Support", "gpt-4.1");
    let request_json = serde_json::to_value(request).unwrap();
    assert_eq!(request_json["name"], "Support");
    assert_eq!(request_json["model"], "gpt-4.1");

    let update = UpdateAssistantRequest::new("Support v2");
    let update_json = serde_json::to_value(update).unwrap();
    assert_eq!(update_json["name"], "Support v2");

    let assistant = AssistantObject::new("asst_1", "Support", "gpt-4.1");
    let assistant_json = serde_json::to_value(assistant).unwrap();
    assert_eq!(assistant_json["object"], "assistant");

    let list =
        ListAssistantsResponse::new(vec![AssistantObject::new("asst_1", "Support", "gpt-4.1")]);
    let list_json = serde_json::to_value(list).unwrap();
    assert_eq!(list_json["object"], "list");
    assert_eq!(list_json["first_id"], "asst_1");
    assert_eq!(list_json["last_id"], "asst_1");
    assert_eq!(list_json["has_more"], false);

    let deleted = DeleteAssistantResponse::deleted("asst_1");
    let deleted_json = serde_json::to_value(deleted).unwrap();
    assert_eq!(deleted_json["object"], "assistant.deleted");
    assert_eq!(deleted_json["deleted"], true);
}
