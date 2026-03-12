use sdkwork_api_contract_openai::responses::ResponseObject;

#[test]
fn serializes_response_object() {
    let json = serde_json::to_value(ResponseObject::empty("resp_1", "gpt-4.1")).unwrap();
    assert_eq!(json["object"], "response");
}
