use sdkwork_api_contract_openai::responses::{
    DeleteResponseResponse, ListResponseInputItemsResponse, ResponseInputItemObject, ResponseObject,
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
