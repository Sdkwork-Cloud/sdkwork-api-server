use sdkwork_api_provider_openai::map_model_object;

#[test]
fn maps_provider_model_to_catalog_entry() {
    let entry = map_model_object("gpt-4.1");
    assert_eq!(entry.external_name, "gpt-4.1");
}
