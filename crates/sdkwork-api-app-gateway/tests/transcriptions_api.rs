use sdkwork_api_app_gateway::create_transcription;

#[test]
fn transcription_requires_transcription_backend() {
    let error =
        create_transcription("tenant-1", "project-1", "gpt-4o-mini-transcribe").unwrap_err();
    assert!(error.to_string().contains("not supported"));
}
