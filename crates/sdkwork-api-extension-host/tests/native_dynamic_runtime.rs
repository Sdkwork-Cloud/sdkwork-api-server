use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::Body;
use sdkwork_api_ext_provider_native_mock::FIXTURE_EXTENSION_ID;
use sdkwork_api_extension_host::{
    list_native_dynamic_runtime_statuses, load_native_dynamic_library_manifest,
    load_native_dynamic_provider_adapter, shutdown_all_native_dynamic_runtimes,
};
use sdkwork_api_provider_core::ProviderRequest;
use sdkwork_api_provider_core::ProviderStreamOutput;
use serial_test::serial;
use tokio::time::{sleep, Duration};

#[serial(native_dynamic_lifecycle)]
#[tokio::test]
async fn loading_native_dynamic_runtime_reports_lifecycle_health_status() {
    shutdown_all_native_dynamic_runtimes().expect("pre-test cleanup");
    let log_guard = NativeDynamicLifecycleLogGuard::new();

    let library_path = native_dynamic_fixture_library_path();
    let _adapter = load_native_dynamic_provider_adapter(&library_path, "https://example.com/v1")
        .expect("native dynamic provider adapter");

    let statuses = list_native_dynamic_runtime_statuses().expect("runtime statuses");
    assert_eq!(statuses.len(), 1);
    assert_eq!(statuses[0].extension_id, FIXTURE_EXTENSION_ID);
    assert_eq!(statuses[0].display_name, "Native Mock");
    assert!(statuses[0].running);
    assert!(statuses[0].healthy);
    assert!(statuses[0].supports_health_check);
    assert!(statuses[0].supports_shutdown);
    assert_eq!(statuses[0].message.as_deref(), Some("native mock healthy"));

    let log = std::fs::read_to_string(log_guard.path()).expect("lifecycle log");
    assert_eq!(log.lines().collect::<Vec<_>>(), vec!["init"]);

    shutdown_all_native_dynamic_runtimes().expect("post-test cleanup");
}

#[serial(native_dynamic_lifecycle)]
#[tokio::test]
async fn shutting_down_native_dynamic_runtimes_invokes_plugin_shutdown_hook() {
    shutdown_all_native_dynamic_runtimes().expect("pre-test cleanup");
    let log_guard = NativeDynamicLifecycleLogGuard::new();

    let library_path = native_dynamic_fixture_library_path();
    let adapter = load_native_dynamic_provider_adapter(&library_path, "https://example.com/v1")
        .expect("native dynamic provider adapter");
    drop(adapter);

    shutdown_all_native_dynamic_runtimes().expect("shutdown");

    let log = std::fs::read_to_string(log_guard.path()).expect("lifecycle log");
    assert_eq!(log.lines().collect::<Vec<_>>(), vec!["init", "shutdown"]);
}

#[serial(native_dynamic_lifecycle)]
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
                extra: serde_json::Map::new(),
            },
        ],
        stream: None,
        extra: serde_json::Map::new(),
    };
    let output = adapter
        .execute("sk-native", ProviderRequest::ChatCompletions(&request))
        .await
        .expect("native dynamic output");
    let output = output.into_json().expect("json output");

    assert_eq!(output["id"], "chatcmpl_native_dynamic");
}

#[serial(native_dynamic_lifecycle)]
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
                extra: serde_json::Map::new(),
            },
        ],
        stream: Some(true),
        extra: serde_json::Map::new(),
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

#[serial(native_dynamic_lifecycle)]
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

#[serial(native_dynamic_lifecycle)]
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

#[serial(native_dynamic_lifecycle)]
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

#[serial(native_dynamic_lifecycle)]
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

#[serial(native_dynamic_lifecycle)]
#[tokio::test]
async fn shutting_down_native_dynamic_runtimes_waits_for_in_flight_json_invocation() {
    shutdown_all_native_dynamic_runtimes().expect("pre-test cleanup");
    let lifecycle_log = NativeDynamicLifecycleLogGuard::new();
    let invocation_log = NativeDynamicInvocationLogGuard::new();
    let _delay_guard = NativeDynamicMockDelayGuard::json(250);

    let library_path = native_dynamic_fixture_library_path();
    let adapter = load_native_dynamic_provider_adapter(&library_path, "https://example.com/v1")
        .expect("native dynamic provider adapter");
    let request = sdkwork_api_contract_openai::chat_completions::CreateChatCompletionRequest {
        model: "gpt-4.1".to_owned(),
        messages: vec![
            sdkwork_api_contract_openai::chat_completions::ChatMessageInput {
                role: "user".to_owned(),
                content: serde_json::Value::String("hello".to_owned()),
                extra: serde_json::Map::new(),
            },
        ],
        stream: None,
        extra: serde_json::Map::new(),
    };

    let invocation = std::thread::spawn(move || {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("thread runtime")
            .block_on(async move {
                let output = adapter
                    .execute("sk-native", ProviderRequest::ChatCompletions(&request))
                    .await
                    .expect("native dynamic output");
                output.into_json().expect("json output")
            })
    });

    wait_for_log_line(invocation_log.path(), "execute_json_start").await;

    let shutdown = tokio::task::spawn_blocking(|| {
        shutdown_all_native_dynamic_runtimes().expect("shutdown");
    });

    sleep(Duration::from_millis(50)).await;
    assert!(
        !shutdown.is_finished(),
        "shutdown should wait for the in-flight JSON invocation"
    );

    let output = invocation.join().expect("invocation thread");
    assert_eq!(output["id"], "chatcmpl_native_dynamic");

    shutdown.await.expect("shutdown join");

    assert_eq!(
        read_log_lines(invocation_log.path()),
        vec!["execute_json_start", "execute_json_finish"]
    );
    assert_eq!(
        read_log_lines(lifecycle_log.path()),
        vec!["init", "shutdown"]
    );
}

#[serial(native_dynamic_lifecycle)]
#[tokio::test]
async fn shutting_down_native_dynamic_runtimes_waits_for_in_flight_stream_invocation() {
    shutdown_all_native_dynamic_runtimes().expect("pre-test cleanup");
    let lifecycle_log = NativeDynamicLifecycleLogGuard::new();
    let invocation_log = NativeDynamicInvocationLogGuard::new();
    let _delay_guard = NativeDynamicMockDelayGuard::stream(250);

    let library_path = native_dynamic_fixture_library_path();
    let adapter = load_native_dynamic_provider_adapter(&library_path, "https://example.com/v1")
        .expect("native dynamic provider adapter");
    let request = sdkwork_api_contract_openai::chat_completions::CreateChatCompletionRequest {
        model: "gpt-4.1".to_owned(),
        messages: vec![
            sdkwork_api_contract_openai::chat_completions::ChatMessageInput {
                role: "user".to_owned(),
                content: serde_json::Value::String("hello".to_owned()),
                extra: serde_json::Map::new(),
            },
        ],
        stream: Some(true),
        extra: serde_json::Map::new(),
    };

    let output = adapter
        .execute(
            "sk-native",
            ProviderRequest::ChatCompletionsStream(&request),
        )
        .await
        .expect("native dynamic stream output");

    wait_for_log_line(invocation_log.path(), "execute_stream_start").await;

    let shutdown = tokio::task::spawn_blocking(|| {
        shutdown_all_native_dynamic_runtimes().expect("shutdown");
    });

    sleep(Duration::from_millis(50)).await;
    assert!(
        !shutdown.is_finished(),
        "shutdown should wait for the in-flight stream invocation thread"
    );

    let stream = output.into_stream().expect("stream output");
    let body = read_provider_stream(stream).await;
    assert!(body.contains("chatcmpl_native_dynamic_stream"));
    assert!(body.contains("[DONE]"));

    shutdown.await.expect("shutdown join");

    assert_eq!(
        read_log_lines(invocation_log.path()),
        vec!["execute_stream_start", "execute_stream_finish"]
    );
    assert_eq!(
        read_log_lines(lifecycle_log.path()),
        vec!["init", "shutdown"]
    );
}

#[serial(native_dynamic_lifecycle)]
#[tokio::test]
async fn shutting_down_native_dynamic_runtimes_rolls_back_after_drain_timeout() {
    shutdown_all_native_dynamic_runtimes().expect("pre-test cleanup");
    let lifecycle_log = NativeDynamicLifecycleLogGuard::new();
    let invocation_log = NativeDynamicInvocationLogGuard::new();
    let delay_guard = NativeDynamicMockDelayGuard::json(250);
    let timeout_guard = NativeDynamicDrainTimeoutGuard::new(25);

    let library_path = native_dynamic_fixture_library_path();
    let adapter = load_native_dynamic_provider_adapter(&library_path, "https://example.com/v1")
        .expect("native dynamic provider adapter");
    let request = sdkwork_api_contract_openai::chat_completions::CreateChatCompletionRequest {
        model: "gpt-4.1".to_owned(),
        messages: vec![
            sdkwork_api_contract_openai::chat_completions::ChatMessageInput {
                role: "user".to_owned(),
                content: serde_json::Value::String("hello".to_owned()),
                extra: serde_json::Map::new(),
            },
        ],
        stream: None,
        extra: serde_json::Map::new(),
    };

    let first_invocation = std::thread::spawn(move || {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("thread runtime")
            .block_on(async move {
                let output = adapter
                    .execute("sk-native", ProviderRequest::ChatCompletions(&request))
                    .await
                    .expect("first native dynamic output");
                output.into_json().expect("first json output")
            })
    });

    wait_for_log_line(invocation_log.path(), "execute_json_start").await;

    let shutdown = tokio::task::spawn_blocking(shutdown_all_native_dynamic_runtimes);
    let error = shutdown
        .await
        .expect("shutdown join")
        .expect_err("shutdown should time out");
    assert!(
        error.to_string().contains("drain"),
        "unexpected shutdown error: {error}"
    );

    let statuses = list_native_dynamic_runtime_statuses().expect("runtime statuses after timeout");
    assert_eq!(statuses.len(), 1);
    assert!(statuses[0].running);

    let adapter = load_native_dynamic_provider_adapter(&library_path, "https://example.com/v1")
        .expect("native dynamic provider adapter after rollback");
    let request = sdkwork_api_contract_openai::chat_completions::CreateChatCompletionRequest {
        model: "gpt-4.1".to_owned(),
        messages: vec![
            sdkwork_api_contract_openai::chat_completions::ChatMessageInput {
                role: "user".to_owned(),
                content: serde_json::Value::String("hello again".to_owned()),
                extra: serde_json::Map::new(),
            },
        ],
        stream: None,
        extra: serde_json::Map::new(),
    };

    let second_invocation = std::thread::spawn(move || {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("thread runtime")
            .block_on(async move {
                let output = adapter
                    .execute("sk-native", ProviderRequest::ChatCompletions(&request))
                    .await
                    .expect("second native dynamic output");
                output.into_json().expect("second json output")
            })
    });

    wait_for_log_occurrences(invocation_log.path(), "execute_json_start", 2).await;

    let first_output = first_invocation.join().expect("first invocation thread");
    let second_output = second_invocation.join().expect("second invocation thread");
    assert_eq!(first_output["id"], "chatcmpl_native_dynamic");
    assert_eq!(second_output["id"], "chatcmpl_native_dynamic");
    assert_eq!(read_log_lines(lifecycle_log.path()), vec!["init"]);

    drop(timeout_guard);
    drop(delay_guard);

    shutdown_all_native_dynamic_runtimes().expect("post-timeout cleanup shutdown");
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

struct NativeDynamicLifecycleLogGuard {
    path: PathBuf,
    previous: Option<String>,
}

impl NativeDynamicLifecycleLogGuard {
    fn new() -> Self {
        let mut path = std::env::temp_dir();
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("unix time")
            .as_millis();
        path.push(format!(
            "sdkwork-native-dynamic-lifecycle-runtime-{millis}.log"
        ));
        let previous = std::env::var("SDKWORK_NATIVE_MOCK_LIFECYCLE_LOG").ok();
        std::env::set_var("SDKWORK_NATIVE_MOCK_LIFECYCLE_LOG", &path);
        Self { path, previous }
    }

    fn path(&self) -> &PathBuf {
        &self.path
    }
}

impl Drop for NativeDynamicLifecycleLogGuard {
    fn drop(&mut self) {
        match self.previous.as_deref() {
            Some(value) => std::env::set_var("SDKWORK_NATIVE_MOCK_LIFECYCLE_LOG", value),
            None => std::env::remove_var("SDKWORK_NATIVE_MOCK_LIFECYCLE_LOG"),
        }
        let _ = std::fs::remove_file(&self.path);
    }
}

struct NativeDynamicInvocationLogGuard {
    path: PathBuf,
    previous: Option<String>,
}

impl NativeDynamicInvocationLogGuard {
    fn new() -> Self {
        let mut path = std::env::temp_dir();
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("unix time")
            .as_millis();
        path.push(format!(
            "sdkwork-native-dynamic-invocation-runtime-{millis}.log"
        ));
        let previous = std::env::var("SDKWORK_NATIVE_MOCK_INVOCATION_LOG").ok();
        std::env::set_var("SDKWORK_NATIVE_MOCK_INVOCATION_LOG", &path);
        Self { path, previous }
    }

    fn path(&self) -> &PathBuf {
        &self.path
    }
}

impl Drop for NativeDynamicInvocationLogGuard {
    fn drop(&mut self) {
        match self.previous.as_deref() {
            Some(value) => std::env::set_var("SDKWORK_NATIVE_MOCK_INVOCATION_LOG", value),
            None => std::env::remove_var("SDKWORK_NATIVE_MOCK_INVOCATION_LOG"),
        }
        let _ = std::fs::remove_file(&self.path);
    }
}

struct NativeDynamicMockDelayGuard {
    previous_json_delay_ms: Option<String>,
    previous_stream_delay_ms: Option<String>,
}

impl NativeDynamicMockDelayGuard {
    fn json(delay_ms: u64) -> Self {
        let previous_json_delay_ms = std::env::var("SDKWORK_NATIVE_MOCK_JSON_DELAY_MS").ok();
        let previous_stream_delay_ms = std::env::var("SDKWORK_NATIVE_MOCK_STREAM_DELAY_MS").ok();
        std::env::set_var("SDKWORK_NATIVE_MOCK_JSON_DELAY_MS", delay_ms.to_string());
        std::env::remove_var("SDKWORK_NATIVE_MOCK_STREAM_DELAY_MS");
        Self {
            previous_json_delay_ms,
            previous_stream_delay_ms,
        }
    }

    fn stream(delay_ms: u64) -> Self {
        let previous_json_delay_ms = std::env::var("SDKWORK_NATIVE_MOCK_JSON_DELAY_MS").ok();
        let previous_stream_delay_ms = std::env::var("SDKWORK_NATIVE_MOCK_STREAM_DELAY_MS").ok();
        std::env::remove_var("SDKWORK_NATIVE_MOCK_JSON_DELAY_MS");
        std::env::set_var("SDKWORK_NATIVE_MOCK_STREAM_DELAY_MS", delay_ms.to_string());
        Self {
            previous_json_delay_ms,
            previous_stream_delay_ms,
        }
    }
}

impl Drop for NativeDynamicMockDelayGuard {
    fn drop(&mut self) {
        match self.previous_json_delay_ms.as_deref() {
            Some(value) => std::env::set_var("SDKWORK_NATIVE_MOCK_JSON_DELAY_MS", value),
            None => std::env::remove_var("SDKWORK_NATIVE_MOCK_JSON_DELAY_MS"),
        }
        match self.previous_stream_delay_ms.as_deref() {
            Some(value) => std::env::set_var("SDKWORK_NATIVE_MOCK_STREAM_DELAY_MS", value),
            None => std::env::remove_var("SDKWORK_NATIVE_MOCK_STREAM_DELAY_MS"),
        }
    }
}

struct NativeDynamicDrainTimeoutGuard {
    previous_timeout_ms: Option<String>,
}

impl NativeDynamicDrainTimeoutGuard {
    fn new(timeout_ms: u64) -> Self {
        let previous_timeout_ms =
            std::env::var("SDKWORK_NATIVE_DYNAMIC_SHUTDOWN_DRAIN_TIMEOUT_MS").ok();
        std::env::set_var(
            "SDKWORK_NATIVE_DYNAMIC_SHUTDOWN_DRAIN_TIMEOUT_MS",
            timeout_ms.to_string(),
        );
        Self {
            previous_timeout_ms,
        }
    }
}

impl Drop for NativeDynamicDrainTimeoutGuard {
    fn drop(&mut self) {
        match self.previous_timeout_ms.as_deref() {
            Some(value) => {
                std::env::set_var("SDKWORK_NATIVE_DYNAMIC_SHUTDOWN_DRAIN_TIMEOUT_MS", value)
            }
            None => std::env::remove_var("SDKWORK_NATIVE_DYNAMIC_SHUTDOWN_DRAIN_TIMEOUT_MS"),
        }
    }
}

async fn wait_for_log_line(path: &PathBuf, expected: &str) {
    for _ in 0..100 {
        if read_log_lines(path).iter().any(|line| line == expected) {
            return;
        }
        sleep(Duration::from_millis(10)).await;
    }
    panic!(
        "timed out waiting for log line {expected} in {}",
        path.display()
    );
}

async fn wait_for_log_occurrences(path: &PathBuf, expected: &str, count: usize) {
    for _ in 0..100 {
        if read_log_lines(path)
            .iter()
            .filter(|line| line.as_str() == expected)
            .count()
            >= count
        {
            return;
        }
        sleep(Duration::from_millis(10)).await;
    }
    panic!(
        "timed out waiting for {count} occurrences of {expected} in {}",
        path.display()
    );
}

fn read_log_lines(path: &PathBuf) -> Vec<String> {
    std::fs::read_to_string(path)
        .unwrap_or_default()
        .lines()
        .map(ToOwned::to_owned)
        .collect()
}
