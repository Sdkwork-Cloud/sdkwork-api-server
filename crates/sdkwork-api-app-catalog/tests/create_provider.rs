use sdkwork_api_app_catalog::create_provider;

#[test]
fn creates_provider_for_channel() {
    let provider =
        create_provider("provider-openai-official", "openai", "OpenAI Official").unwrap();
    assert_eq!(provider.channel_id, "openai");
}
