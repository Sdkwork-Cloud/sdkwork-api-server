use sdkwork_api_runtime_host::EmbeddedRuntime;

#[tokio::test]
async fn embedded_runtime_starts_on_loopback() {
    let runtime = EmbeddedRuntime::start_ephemeral().await.unwrap();
    assert!(runtime.base_url().starts_with("http://127.0.0.1:"));
}
