use sdkwork_api_app_gateway::{
    create_vector_store_file, delete_vector_store_file, get_vector_store_file,
    list_vector_store_files,
};

#[test]
fn returns_vector_store_file_object() {
    let response = create_vector_store_file("tenant-1", "project-1", "vs_1", "file_1").unwrap();
    assert_eq!(response.id, "file_1");
    assert_eq!(response.object, "vector_store.file");
}

#[test]
fn lists_vector_store_file_objects() {
    let response = list_vector_store_files("tenant-1", "project-1", "vs_1").unwrap();
    assert_eq!(response.object, "list");
    assert_eq!(response.data[0].object, "vector_store.file");
}

#[test]
fn retrieves_vector_store_file_object() {
    let response = get_vector_store_file("tenant-1", "project-1", "vs_1", "file_1").unwrap();
    assert_eq!(response.id, "file_1");
    assert_eq!(response.object, "vector_store.file");
}

#[test]
fn deletes_vector_store_file_object() {
    let response = delete_vector_store_file("tenant-1", "project-1", "vs_1", "file_1").unwrap();
    assert_eq!(response.id, "file_1");
    assert!(response.deleted);
}
