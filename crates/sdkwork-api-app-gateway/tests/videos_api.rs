use sdkwork_api_app_gateway::{
    create_video, delete_video, get_video, list_videos, remix_video, video_content,
};

fn assert_error_contains<T: std::fmt::Debug, E: std::fmt::Display>(
    result: Result<T, E>,
    expected: &str,
) {
    let error = result.expect_err("expected error");
    assert!(
        error.to_string().contains(expected),
        "expected error containing `{expected}`, got `{error}`"
    );
}

#[test]
fn local_video_fallback_requires_backing_asset_store() {
    assert_error_contains(
        create_video(
            "tenant-1",
            "project-1",
            "sora-1",
            "A short cinematic flyover",
        ),
        "Local video fallback is not supported",
    );
    assert_error_contains(
        remix_video(
            "tenant-1",
            "project-1",
            "video_local_0000000000000001",
            "Make it sunset",
        ),
        "Local video fallback is not supported",
    );
}

#[test]
fn local_video_listing_requires_upstream_provider() {
    assert_error_contains(
        list_videos("tenant-1", "project-1"),
        "Local video listing fallback is not supported",
    );
}

#[test]
fn local_video_fallback_requires_persisted_state() {
    assert_error_contains(
        get_video("tenant-1", "project-1", "video_local_0000000000000001"),
        "video not found",
    );
    assert_error_contains(
        delete_video("tenant-1", "project-1", "video_local_0000000000000001"),
        "video not found",
    );
    assert_error_contains(
        video_content("tenant-1", "project-1", "video_local_0000000000000001"),
        "video not found",
    );
}
