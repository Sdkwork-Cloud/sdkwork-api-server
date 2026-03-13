use sdkwork_api_app_gateway::{
    create_vector_store, delete_vector_store, get_vector_store, list_vector_stores,
    update_vector_store,
};

#[test]
fn returns_vector_store_object() {
    let response = create_vector_store("tenant-1", "project-1", "kb-main").unwrap();
    assert_eq!(response.object, "vector_store");
    assert_eq!(response.name, "kb-main");
}

#[test]
fn lists_vector_store_objects() {
    let response = list_vector_stores("tenant-1", "project-1").unwrap();
    assert_eq!(response.object, "list");
    assert_eq!(response.data[0].object, "vector_store");
}

#[test]
fn retrieves_vector_store_object() {
    let response = get_vector_store("tenant-1", "project-1", "vs_1").unwrap();
    assert_eq!(response.id, "vs_1");
    assert_eq!(response.object, "vector_store");
}

#[test]
fn updates_vector_store_object() {
    let response = update_vector_store("tenant-1", "project-1", "vs_1", "kb-updated").unwrap();
    assert_eq!(response.id, "vs_1");
    assert_eq!(response.name, "kb-updated");
}

#[test]
fn deletes_vector_store_object() {
    let response = delete_vector_store("tenant-1", "project-1", "vs_1").unwrap();
    assert_eq!(response.id, "vs_1");
    assert!(response.deleted);
}
