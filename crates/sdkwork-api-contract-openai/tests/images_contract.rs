use sdkwork_api_contract_openai::images::{
    CreateImageEditRequest, CreateImageVariationRequest, ImageUpload,
};

#[test]
fn image_variation_defaults_to_gpt_image_1() {
    let request =
        CreateImageVariationRequest::new(ImageUpload::new("variation.png", b"PNGDATA".to_vec()));

    assert_eq!(request.model_or_default(), "gpt-image-1");
}

#[test]
fn image_edit_preserves_mask_attachment() {
    let request = CreateImageEditRequest::new(
        "make it sunset",
        ImageUpload::new("source.png", b"PNGDATA".to_vec()),
    )
    .with_mask(ImageUpload::new("mask.png", b"MASKDATA".to_vec()));

    assert_eq!(request.prompt, "make it sunset");
    assert_eq!(request.image.filename, "source.png");
    assert_eq!(
        request.mask.as_ref().map(|mask| mask.filename.as_str()),
        Some("mask.png")
    );
}
