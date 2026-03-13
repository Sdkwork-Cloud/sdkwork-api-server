use sdkwork_api_app_gateway::{
    create_assistant, delete_assistant, get_assistant, list_assistants, update_assistant,
};

#[test]
fn returns_assistant_object() {
    let response = create_assistant("tenant-1", "project-1", "Support", "gpt-4.1").unwrap();
    assert_eq!(response.object, "assistant");
    assert_eq!(response.model, "gpt-4.1");
}

#[test]
fn lists_assistant_objects() {
    let response = list_assistants("tenant-1", "project-1").unwrap();
    assert_eq!(response.object, "list");
    assert_eq!(response.data[0].object, "assistant");
}

#[test]
fn retrieves_assistant_object() {
    let response = get_assistant("tenant-1", "project-1", "asst_1").unwrap();
    assert_eq!(response.id, "asst_1");
    assert_eq!(response.object, "assistant");
}

#[test]
fn updates_assistant_object() {
    let response = update_assistant("tenant-1", "project-1", "asst_1", "Support v2").unwrap();
    assert_eq!(response.id, "asst_1");
    assert_eq!(response.name, "Support v2");
}

#[test]
fn deletes_assistant_object() {
    let response = delete_assistant("tenant-1", "project-1", "asst_1").unwrap();
    assert_eq!(response.id, "asst_1");
    assert!(response.deleted);
}
