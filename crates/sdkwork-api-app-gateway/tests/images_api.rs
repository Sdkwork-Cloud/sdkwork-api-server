use sdkwork_api_app_gateway::{create_image_edit, create_image_generation, create_image_variation};
use sdkwork_api_contract_openai::images::{
    CreateImageEditRequest, CreateImageVariationRequest, ImageUpload,
};

#[test]
fn returns_images_response() {
    let response = create_image_generation("tenant-1", "project-1", "gpt-image-1").unwrap();
    assert_eq!(response.data.len(), 1);
    assert_eq!(response.data[0].b64_json, "sdkwork-image");
}

#[test]
fn returns_image_edit_response() {
    let request = CreateImageEditRequest::new(
        "make it sunset",
        ImageUpload::new("source.png", b"PNGDATA".to_vec()).with_content_type("image/png"),
    )
    .with_model("gpt-image-1")
    .with_mask(ImageUpload::new("mask.png", b"MASKDATA".to_vec()).with_content_type("image/png"));

    let response = create_image_edit("tenant-1", "project-1", &request).unwrap();
    assert_eq!(response.data.len(), 1);
    assert_eq!(response.data[0].b64_json, "sdkwork-image");
}

#[test]
fn returns_image_variation_response() {
    let request = CreateImageVariationRequest::new(
        ImageUpload::new("source.png", b"PNGDATA".to_vec()).with_content_type("image/png"),
    )
    .with_model("gpt-image-1");

    let response = create_image_variation("tenant-1", "project-1", &request).unwrap();
    assert_eq!(response.data.len(), 1);
    assert_eq!(response.data[0].b64_json, "sdkwork-image");
}
