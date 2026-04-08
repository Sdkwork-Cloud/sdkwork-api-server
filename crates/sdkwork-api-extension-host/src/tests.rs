use super::provider_invocation_from_request;
use super::provider_invocation_from_request_with_options;
use sdkwork_api_contract_openai::audio::CreateSpeechRequest;
use sdkwork_api_contract_openai::responses::CreateResponseRequest;
use sdkwork_api_contract_openai::uploads::CompleteUploadRequest;
use sdkwork_api_provider_core::{ProviderRequest, ProviderRequestOptions};

#[test]
fn upload_complete_invocation_preserves_upload_id_as_path_param() {
    let request = CompleteUploadRequest::new("upload_123", ["part_1", "part_2"]);

    let invocation = provider_invocation_from_request(
        ProviderRequest::UploadComplete(&request),
        "sk-native",
        "https://example.com/v1",
    )
    .expect("provider invocation");

    assert_eq!(invocation.operation, "uploads.complete");
    assert_eq!(invocation.path_params, vec!["upload_123".to_owned()]);
    assert_eq!(
        invocation.body["part_ids"],
        serde_json::json!(["part_1", "part_2"])
    );
}

#[test]
fn responses_stream_invocation_marks_stream_expectation() {
    let request = CreateResponseRequest {
        model: "gpt-4.1".to_owned(),
        input: serde_json::Value::String("hello".to_owned()),
        stream: Some(true),
    };

    let invocation = provider_invocation_from_request(
        ProviderRequest::ResponsesStream(&request),
        "sk-native",
        "https://example.com/v1",
    )
    .expect("provider invocation");

    assert_eq!(invocation.operation, "responses.create");
    assert!(invocation.expects_stream);
}

#[test]
fn audio_speech_invocation_marks_stream_expectation() {
    let mut request = CreateSpeechRequest::new("gpt-4o-mini-tts", "nova", "hello");
    request.response_format = Some("mp3".to_owned());

    let invocation = provider_invocation_from_request(
        ProviderRequest::AudioSpeech(&request),
        "sk-native",
        "https://example.com/v1",
    )
    .expect("provider invocation");

    assert_eq!(invocation.operation, "audio.speech.create");
    assert!(invocation.expects_stream);
}

#[test]
fn files_content_invocation_marks_stream_expectation() {
    let invocation = provider_invocation_from_request(
        ProviderRequest::FilesContent("file_1"),
        "sk-native",
        "https://example.com/v1",
    )
    .expect("provider invocation");

    assert_eq!(invocation.operation, "files.content");
    assert!(invocation.expects_stream);
}

#[test]
fn videos_content_invocation_marks_stream_expectation() {
    let invocation = provider_invocation_from_request(
        ProviderRequest::VideosContent("video_1"),
        "sk-native",
        "https://example.com/v1",
    )
    .expect("provider invocation");

    assert_eq!(invocation.operation, "videos.content");
    assert!(invocation.expects_stream);
}

#[test]
fn provider_invocation_preserves_compatibility_headers_when_requested() {
    let request = CreateResponseRequest {
        model: "gpt-4.1".to_owned(),
        input: serde_json::Value::String("hello".to_owned()),
        stream: None,
    };
    let options = ProviderRequestOptions::new()
        .with_header("anthropic-version", "2023-06-01")
        .with_header("anthropic-beta", "tools-2024-04-04");

    let invocation = provider_invocation_from_request_with_options(
        ProviderRequest::Responses(&request),
        "sk-native",
        "https://example.com/v1",
        &options,
    )
    .expect("provider invocation");

    assert_eq!(
        invocation
            .headers
            .get("anthropic-version")
            .map(String::as_str),
        Some("2023-06-01")
    );
    assert_eq!(
        invocation.headers.get("anthropic-beta").map(String::as_str),
        Some("tools-2024-04-04")
    );
}

#[test]
fn provider_invocation_includes_standard_execution_context_headers() {
    let request = CreateResponseRequest {
        model: "gpt-4.1".to_owned(),
        input: serde_json::Value::String("hello".to_owned()),
        stream: None,
    };
    let options = ProviderRequestOptions::new()
        .with_idempotency_key("idem-native")
        .with_request_trace_id("trace-native");

    let invocation = provider_invocation_from_request_with_options(
        ProviderRequest::Responses(&request),
        "sk-native",
        "https://example.com/v1",
        &options,
    )
    .expect("provider invocation");

    assert_eq!(
        invocation
            .headers
            .get("idempotency-key")
            .map(String::as_str),
        Some("idem-native")
    );
    assert_eq!(
        invocation.headers.get("x-request-id").map(String::as_str),
        Some("trace-native")
    );
}
