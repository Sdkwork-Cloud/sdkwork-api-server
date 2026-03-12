use sdkwork_api_app_usage::record_usage;

#[test]
fn usage_record_contains_model_and_provider() {
    let usage = record_usage("project-1", "gpt-4.1", "provider-openai-official").unwrap();
    assert_eq!(usage.model, "gpt-4.1");
}
