use sdkwork_api_contract_openai::audio::CreateSpeechRequest;

#[test]
fn serializes_optional_speech_protocol_fields() {
    let request = CreateSpeechRequest {
        model: "gpt-4o-mini-tts".to_owned(),
        voice: "nova".to_owned(),
        input: "Hello".to_owned(),
        instructions: Some("Speak warmly".to_owned()),
        response_format: Some("wav".to_owned()),
        speed: Some(1.0),
        stream_format: Some("sse".to_owned()),
    };

    let json = serde_json::to_value(request).unwrap();
    assert_eq!(json["response_format"], "wav");
    assert_eq!(json["stream_format"], "sse");
    assert_eq!(json["instructions"], "Speak warmly");
}
