use sdkwork_api_contract_openai::audio::CreateSpeechRequest;

#[test]
fn speech_requires_speech_backend() {
    let request = CreateSpeechRequest {
        model: "gpt-4o-mini-tts".to_owned(),
        voice: "nova".to_owned(),
        input: "Hello".to_owned(),
        instructions: None,
        response_format: Some("wav".to_owned()),
        speed: None,
        stream_format: None,
    };

    let error = sdkwork_api_app_gateway::create_speech_response("tenant-1", "project-1", &request)
        .unwrap_err();
    assert!(error.to_string().contains("not supported"));
}
