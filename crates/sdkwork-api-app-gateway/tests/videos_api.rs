use sdkwork_api_app_gateway::{
    create_video, delete_video, get_video, list_videos, remix_video, video_content,
};

#[test]
fn create_video_requires_backing_asset_store() {
    let error = create_video(
        "tenant-1",
        "project-1",
        "sora-1",
        "A short cinematic flyover",
    )
    .unwrap_err();
    assert!(error.to_string().contains("not supported"));
}

#[test]
fn lists_video_objects() {
    let response = list_videos("tenant-1", "project-1").unwrap();
    assert_eq!(response.object, "list");
    assert!(response.data.is_empty());
}

#[test]
fn retrieving_video_requires_persisted_state() {
    let error = get_video("tenant-1", "project-1", "video_local_0000000000000001").unwrap_err();
    assert!(error.to_string().contains("not found"));
}

#[test]
fn deletes_video_object() {
    let response = delete_video("tenant-1", "project-1", "video_local_0000000000000001").unwrap();
    assert_eq!(response.id, "video_local_0000000000000001");
    assert!(response.deleted);
}

#[test]
fn video_content_requires_persisted_state() {
    let error = video_content("tenant-1", "project-1", "video_local_0000000000000001").unwrap_err();
    assert!(error.to_string().contains("not found"));
}

#[test]
fn remixes_video_requires_backing_asset_store() {
    let error = remix_video(
        "tenant-1",
        "project-1",
        "video_local_0000000000000001",
        "Make it sunset",
    )
    .unwrap_err();
    assert!(error.to_string().contains("not supported"));
}
