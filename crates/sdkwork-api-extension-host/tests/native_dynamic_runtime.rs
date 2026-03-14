use std::path::PathBuf;

use axum::body::Body;
use sdkwork_api_ext_provider_native_mock::FIXTURE_EXTENSION_ID;
use sdkwork_api_extension_host::{
    load_native_dynamic_library_manifest, load_native_dynamic_provider_adapter,
};
use sdkwork_api_provider_core::ProviderRequest;
use sdkwork_api_provider_core::ProviderStreamOutput;

#[tokio::test]
async fn loads_native_dynamic_manifest_and_executes_provider_request() {
    let library_path = native_dynamic_fixture_library_path();
    let manifest =
        load_native_dynamic_library_manifest(&library_path).expect("native dynamic manifest");
    assert_eq!(manifest.id, FIXTURE_EXTENSION_ID);

    let adapter = load_native_dynamic_provider_adapter(&library_path, "https://example.com/v1")
        .expect("native dynamic provider adapter");
    let request = sdkwork_api_contract_openai::chat_completions::CreateChatCompletionRequest {
        model: "gpt-4.1".to_owned(),
        messages: vec![
            sdkwork_api_contract_openai::chat_completions::ChatMessageInput {
                role: "user".to_owned(),
                content: serde_json::Value::String("hello".to_owned()),
            },
        ],
        stream: None,
    };
    let output = adapter
        .execute("sk-native", ProviderRequest::ChatCompletions(&request))
        .await
        .expect("native dynamic output");
    let output = output.into_json().expect("json output");

    assert_eq!(output["id"], "chatcmpl_native_dynamic");
}

#[tokio::test]
async fn executes_native_dynamic_chat_stream_request() {
    let library_path = native_dynamic_fixture_library_path();
    let adapter = load_native_dynamic_provider_adapter(&library_path, "https://example.com/v1")
        .expect("native dynamic provider adapter");
    let request = sdkwork_api_contract_openai::chat_completions::CreateChatCompletionRequest {
        model: "gpt-4.1".to_owned(),
        messages: vec![
            sdkwork_api_contract_openai::chat_completions::ChatMessageInput {
                role: "user".to_owned(),
                content: serde_json::Value::String("hello".to_owned()),
            },
        ],
        stream: Some(true),
    };

    let output = adapter
        .execute(
            "sk-native",
            ProviderRequest::ChatCompletionsStream(&request),
        )
        .await
        .expect("native dynamic stream output");
    let stream = output.into_stream().expect("stream output");
    assert_eq!(stream.content_type(), "text/event-stream");

    let body = read_provider_stream(stream).await;
    assert!(body.contains("chatcmpl_native_dynamic_stream"));
    assert!(body.contains("[DONE]"));
}

#[tokio::test]
async fn executes_native_dynamic_responses_stream_request() {
    let library_path = native_dynamic_fixture_library_path();
    let adapter = load_native_dynamic_provider_adapter(&library_path, "https://example.com/v1")
        .expect("native dynamic provider adapter");
    let request = sdkwork_api_contract_openai::responses::CreateResponseRequest {
        model: "gpt-4.1".to_owned(),
        input: serde_json::Value::String("hello".to_owned()),
        stream: Some(true),
    };

    let output = adapter
        .execute("sk-native", ProviderRequest::ResponsesStream(&request))
        .await
        .expect("native dynamic response stream output");
    let stream = output.into_stream().expect("stream output");
    assert_eq!(stream.content_type(), "text/event-stream");

    let body = read_provider_stream(stream).await;
    assert!(body.contains("resp_native_dynamic_stream"));
    assert!(body.contains("[DONE]"));
}

#[tokio::test]
async fn executes_native_dynamic_audio_speech_stream_request() {
    let library_path = native_dynamic_fixture_library_path();
    let adapter = load_native_dynamic_provider_adapter(&library_path, "https://example.com/v1")
        .expect("native dynamic provider adapter");
    let mut request = sdkwork_api_contract_openai::audio::CreateSpeechRequest::new(
        "gpt-4o-mini-tts",
        "nova",
        "hello",
    );
    request.response_format = Some("mp3".to_owned());

    let output = adapter
        .execute("sk-native", ProviderRequest::AudioSpeech(&request))
        .await
        .expect("native dynamic audio speech output");
    let stream = output.into_stream().expect("stream output");
    assert_eq!(stream.content_type(), "audio/mpeg");

    let bytes = read_provider_stream_bytes(stream).await;
    assert_eq!(bytes, b"NATIVE-AUDIO");
}

#[tokio::test]
async fn executes_native_dynamic_file_content_stream_request() {
    let library_path = native_dynamic_fixture_library_path();
    let adapter = load_native_dynamic_provider_adapter(&library_path, "https://example.com/v1")
        .expect("native dynamic provider adapter");

    let output = adapter
        .execute("sk-native", ProviderRequest::FilesContent("file_1"))
        .await
        .expect("native dynamic file content output");
    let stream = output.into_stream().expect("stream output");
    assert_eq!(stream.content_type(), "application/jsonl");

    let bytes = read_provider_stream_bytes(stream).await;
    assert_eq!(bytes, b"{\"source\":\"native_dynamic\"}\n");
}

#[tokio::test]
async fn executes_native_dynamic_video_content_stream_request() {
    let library_path = native_dynamic_fixture_library_path();
    let adapter = load_native_dynamic_provider_adapter(&library_path, "https://example.com/v1")
        .expect("native dynamic provider adapter");

    let output = adapter
        .execute("sk-native", ProviderRequest::VideosContent("video_1"))
        .await
        .expect("native dynamic video content output");
    let stream = output.into_stream().expect("stream output");
    assert_eq!(stream.content_type(), "video/mp4");

    let bytes = read_provider_stream_bytes(stream).await;
    assert_eq!(bytes, b"NATIVE-VIDEO");
}

async fn read_provider_stream(stream: ProviderStreamOutput) -> String {
    let bytes = axum::body::to_bytes(Body::from_stream(stream.into_body_stream()), usize::MAX)
        .await
        .expect("stream body");
    String::from_utf8(bytes.to_vec()).expect("utf8 stream body")
}

async fn read_provider_stream_bytes(stream: ProviderStreamOutput) -> Vec<u8> {
    axum::body::to_bytes(Body::from_stream(stream.into_body_stream()), usize::MAX)
        .await
        .expect("stream body")
        .to_vec()
}

fn native_dynamic_fixture_library_path() -> PathBuf {
    let current_exe = std::env::current_exe().expect("current exe");
    let directory = current_exe.parent().expect("exe dir");
    let prefix = if cfg!(windows) {
        "sdkwork_api_ext_provider_native_mock"
    } else {
        "libsdkwork_api_ext_provider_native_mock"
    };
    let extension = if cfg!(windows) {
        "dll"
    } else if cfg!(target_os = "macos") {
        "dylib"
    } else {
        "so"
    };

    std::fs::read_dir(directory)
        .expect("deps dir")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .find(|path| {
            path.extension().and_then(|value| value.to_str()) == Some(extension)
                && path
                    .file_stem()
                    .and_then(|value| value.to_str())
                    .is_some_and(|stem| stem.starts_with(prefix))
        })
        .expect("native dynamic fixture library")
}
