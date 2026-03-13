use sdkwork_api_app_gateway::{
    create_response, delete_response, get_response, list_response_input_items,
};

#[test]
fn returns_response_object() {
    let response = create_response("tenant-1", "project-1", "gpt-4.1").unwrap();
    assert_eq!(response.object, "response");
}

#[test]
fn retrieves_response_object() {
    let response = get_response("tenant-1", "project-1", "resp_1").unwrap();
    assert_eq!(response.id, "resp_1");
    assert_eq!(response.object, "response");
}

#[test]
fn lists_response_input_items() {
    let response = list_response_input_items("tenant-1", "project-1", "resp_1").unwrap();
    assert_eq!(response.object, "list");
    assert_eq!(response.data[0].object, "response.input_item");
}

#[test]
fn deletes_response_object() {
    let response = delete_response("tenant-1", "project-1", "resp_1").unwrap();
    assert_eq!(response.id, "resp_1");
    assert!(response.deleted);
}
