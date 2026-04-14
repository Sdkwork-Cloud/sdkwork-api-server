use sdkwork_api_app_gateway::{create_image_edit, create_image_generation, create_image_variation};
use sdkwork_api_contract_openai::images::{
    CreateImageEditRequest, CreateImageVariationRequest, ImageUpload,
};

#[test]
fn image_generation_requires_image_backend() {
    let error = create_image_generation("tenant-1", "project-1", "gpt-image-1").unwrap_err();
    assert!(error.to_string().contains("not supported"));
}

#[test]
fn image_edit_requires_image_backend() {
    let request = CreateImageEditRequest::new(
        "make it sunset",
        ImageUpload::new("source.png", b"PNGDATA".to_vec()).with_content_type("image/png"),
    )
    .with_model("gpt-image-1")
    .with_mask(ImageUpload::new("mask.png", b"MASKDATA".to_vec()).with_content_type("image/png"));

    let error = create_image_edit("tenant-1", "project-1", &request).unwrap_err();
    assert!(error.to_string().contains("not supported"));
}

#[test]
fn image_variation_requires_image_backend() {
    let request = CreateImageVariationRequest::new(
        ImageUpload::new("source.png", b"PNGDATA".to_vec()).with_content_type("image/png"),
    )
    .with_model("gpt-image-1");

    let error = create_image_variation("tenant-1", "project-1", &request).unwrap_err();
    assert!(error.to_string().contains("not supported"));
}
