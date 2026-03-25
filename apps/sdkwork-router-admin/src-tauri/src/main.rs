#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;

use sdkwork_api_runtime_host::{EmbeddedRuntime, RuntimeHostConfig};

mod api_key_setup;

#[derive(Clone)]
struct RuntimeState {
    base_url: String,
}

#[tauri::command]
async fn runtime_base_url(state: tauri::State<'_, RuntimeState>) -> Result<String, String> {
    Ok(state.base_url.clone())
}

fn main() {
    let runtime = tauri::async_runtime::block_on(EmbeddedRuntime::start(runtime_host_config()))
        .expect("failed to start embedded Pingora runtime");
    let state = RuntimeState {
        base_url: runtime.base_url().to_owned(),
    };

    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            runtime_base_url,
            api_key_setup::install_api_router_client_setup,
            api_key_setup::list_api_key_instances
        ])
        .run(tauri::generate_context!())
        .expect("failed to run tauri application");
}

fn runtime_host_config() -> RuntimeHostConfig {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let admin_root = manifest_dir
        .parent()
        .expect("admin src-tauri must live inside the admin app");
    let apps_root = admin_root
        .parent()
        .expect("admin app must live inside the apps directory");

    RuntimeHostConfig::new(
        "127.0.0.1:3001",
        admin_root.join("dist"),
        apps_root.join("sdkwork-router-portal").join("dist"),
        "127.0.0.1:8081",
        "127.0.0.1:8082",
        "127.0.0.1:8080",
    )
}
