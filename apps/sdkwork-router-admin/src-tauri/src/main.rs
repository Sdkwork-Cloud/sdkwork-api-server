#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;
use std::sync::Mutex;

use anyhow::Context;
use sdkwork_api_config::StandaloneConfigLoader;
use sdkwork_api_product_runtime::{
    ProductSiteDirs, RouterProductRuntime, RouterProductRuntimeOptions,
};
use tauri::{path::BaseDirectory, AppHandle, Manager};

mod api_key_setup;

#[derive(Clone)]
struct RuntimeState {
    base_url: String,
}

struct RuntimeHandleState {
    _runtime: Mutex<Option<RouterProductRuntime>>,
}

#[tauri::command]
async fn runtime_base_url(state: tauri::State<'_, RuntimeState>) -> Result<String, String> {
    Ok(state.base_url.clone())
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let runtime = tauri::async_runtime::block_on(start_desktop_runtime(app.handle().clone()))
                .map_err(box_setup_error)?;
            let base_url = runtime
                .public_base_url()
                .context("desktop runtime did not expose a public base url")
                .map_err(box_setup_error)?
                .to_owned();
            app.manage(RuntimeState { base_url });
            app.manage(RuntimeHandleState {
                _runtime: Mutex::new(Some(runtime)),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            runtime_base_url,
            api_key_setup::install_api_router_client_setup,
            api_key_setup::list_api_key_instances
        ])
        .run(tauri::generate_context!())
        .expect("failed to run tauri application");
}

fn box_setup_error(error: anyhow::Error) -> Box<dyn std::error::Error> {
    Box::new(std::io::Error::new(
        std::io::ErrorKind::Other,
        error.to_string(),
    ))
}

async fn start_desktop_runtime(app: AppHandle) -> anyhow::Result<RouterProductRuntime> {
    let (loader, config) = StandaloneConfigLoader::from_env()?;
    RouterProductRuntime::start(
        loader,
        config,
        RouterProductRuntimeOptions::desktop(resolve_desktop_site_dirs(&app)?),
    )
    .await
}

fn resolve_desktop_site_dirs(app: &AppHandle) -> anyhow::Result<ProductSiteDirs> {
    let workspace_dirs = workspace_site_dirs();
    Ok(ProductSiteDirs::new(
        resolve_resource_or_fallback(app, "embedded-sites/admin", workspace_dirs.admin_site_dir)?,
        resolve_resource_or_fallback(
            app,
            "embedded-sites/portal",
            workspace_dirs.portal_site_dir,
        )?,
    ))
}

fn resolve_resource_or_fallback(
    app: &AppHandle,
    resource_path: &str,
    fallback: PathBuf,
) -> anyhow::Result<PathBuf> {
    if let Ok(resource_dir) = app.path().resolve(resource_path, BaseDirectory::Resource) {
        if resource_dir.is_dir() {
            return Ok(resource_dir);
        }
    }

    Ok(fallback)
}

fn workspace_site_dirs() -> ProductSiteDirs {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let apps_root = manifest_dir
        .parent()
        .expect("admin src-tauri must live inside the admin app")
        .parent()
        .expect("admin app must live inside the apps directory");
    ProductSiteDirs::new(
        apps_root.join("sdkwork-router-admin").join("dist"),
        apps_root.join("sdkwork-router-portal").join("dist"),
    )
}

#[cfg(test)]
mod tests {
    use anyhow::anyhow;

    use super::box_setup_error;

    #[test]
    fn box_setup_error_preserves_context_message() {
        let error = box_setup_error(anyhow!("desktop runtime boot failed"));
        assert_eq!(error.to_string(), "desktop runtime boot failed");
    }
}
