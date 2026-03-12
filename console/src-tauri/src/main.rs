use sdkwork_api_runtime_host::EmbeddedRuntime;

#[tauri::command]
async fn runtime_base_url() -> Result<String, String> {
    EmbeddedRuntime::start_ephemeral()
        .await
        .map(|runtime| runtime.base_url().to_owned())
        .map_err(|err| err.to_string())
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![runtime_base_url])
        .run(tauri::generate_context!())
        .expect("failed to run tauri application");
}

