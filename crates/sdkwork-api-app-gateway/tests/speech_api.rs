use sdkwork_api_contract_openai::audio::CreateSpeechRequest;

#[test]
fn returns_speech_fallback_payload() {
    let request = CreateSpeechRequest {
        model: "gpt-4o-mini-tts".to_owned(),
        voice: "nova".to_owned(),
        input: "Hello".to_owned(),
        instructions: None,
        response_format: Some("wav".to_owned()),
        speed: None,
        stream_format: None,
    };

    let response =
        sdkwork_api_app_gateway::create_speech_response("tenant-1", "project-1", &request).unwrap();
    assert_eq!(response.format, "wav");
    assert!(!response.audio_base64.is_empty());
}
