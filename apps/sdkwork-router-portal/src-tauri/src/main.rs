#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;
use std::sync::Mutex;

use anyhow::Context;
use sdkwork_api_config::StandaloneConfigLoader;
use sdkwork_api_product_runtime::{
  ProductSiteDirs, RouterProductRuntime, RouterProductRuntimeOptions,
  RouterProductRuntimeSnapshot,
};
use tauri::{
  menu::{MenuBuilder, MenuEvent},
  path::BaseDirectory,
  tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
  AppHandle, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder, WindowEvent,
};

mod api_key_setup;
mod desktop_shell;

const SERVICE_START_HIDDEN_ENV: &str = "SDKWORK_ROUTER_PORTAL_START_HIDDEN";
const SERVICE_MODE_ENV: &str = "SDKWORK_ROUTER_SERVICE_MODE";

struct RuntimeState {
  snapshot: Mutex<RouterProductRuntimeSnapshot>,
}

struct RuntimeHandleState {
  runtime: Mutex<Option<RouterProductRuntime>>,
}

struct TrayState {
  _tray: tauri::tray::TrayIcon,
}

#[derive(Clone, Copy, Debug, Default)]
struct DesktopLaunchConfig {
  start_hidden: bool,
}

impl DesktopLaunchConfig {
  fn from_environment() -> Self {
    let start_hidden = env_flag_is_truthy(SERVICE_START_HIDDEN_ENV)
      || env_flag_is_truthy(SERVICE_MODE_ENV)
      || std::env::args().skip(1).any(|arg| {
        matches!(arg.as_str(), "--service" | "--start-hidden")
      });

    Self { start_hidden }
  }
}

#[tauri::command]
async fn runtime_base_url(state: tauri::State<'_, RuntimeState>) -> Result<String, String> {
  current_runtime_snapshot(&state)?
    .public_base_url
    .ok_or_else(|| "Desktop runtime did not expose a public base URL.".to_string())
}

#[tauri::command]
async fn runtime_desktop_snapshot(
  state: tauri::State<'_, RuntimeState>,
) -> Result<RouterProductRuntimeSnapshot, String> {
  current_runtime_snapshot(&state)
}

#[tauri::command]
async fn restart_product_runtime(
  app: AppHandle,
  runtime_state: tauri::State<'_, RuntimeState>,
  runtime_handle_state: tauri::State<'_, RuntimeHandleState>,
) -> Result<RouterProductRuntimeSnapshot, String> {
  let runtime = start_desktop_runtime(app.clone())
    .await
    .map_err(|error| format!("desktop runtime restart failed: {error}"))?;
  let snapshot = runtime.snapshot();

  {
    let mut runtime_guard = runtime_handle_state
      .runtime
      .lock()
      .map_err(|_| "Desktop runtime handle state is unavailable.".to_string())?;
    *runtime_guard = Some(runtime);
  }

  {
    let mut snapshot_guard = runtime_state
      .snapshot
      .lock()
      .map_err(|_| "Desktop runtime snapshot state is unavailable.".to_string())?;
    *snapshot_guard = snapshot.clone();
  }

  refresh_runtime_windows(&app, &snapshot)
    .map_err(|error| format!("desktop runtime window refresh failed: {error}"))?;

  Ok(snapshot)
}

fn main() {
  let launch_config = DesktopLaunchConfig::from_environment();

  tauri::Builder::default()
    .setup(move |app| {
      let runtime = tauri::async_runtime::block_on(start_desktop_runtime(app.handle().clone()))
        .map_err(box_setup_error)?;
      let snapshot = runtime.snapshot();
      let app_handle = app.handle().clone();
      snapshot
        .public_base_url
        .as_deref()
        .context("desktop runtime did not expose a public base url")
        .map_err(box_setup_error)?;

      app.manage(RuntimeState {
        snapshot: Mutex::new(snapshot),
      });
      app.manage(RuntimeHandleState {
        runtime: Mutex::new(Some(runtime)),
      });

      install_desktop_shell(app, launch_config).map_err(box_setup_error)?;

      if launch_config.start_hidden {
        #[cfg(target_os = "macos")]
        let _ = app_handle.set_dock_visibility(false);
        hide_main_window(&app_handle).map_err(box_setup_error)?;
      } else {
        #[cfg(target_os = "macos")]
        let _ = app_handle.set_dock_visibility(true);
        show_main_window(&app_handle).map_err(box_setup_error)?;
      }

      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
      runtime_base_url,
      runtime_desktop_snapshot,
      restart_product_runtime,
      api_key_setup::install_api_router_client_setup,
      api_key_setup::list_api_key_instances
    ])
    .run(tauri::generate_context!())
    .expect("error while running sdkwork-router-portal tauri application");
}

fn install_desktop_shell(app: &mut tauri::App, _launch_config: DesktopLaunchConfig) -> anyhow::Result<()> {
  let menu = build_tray_menu(app)?;
  let app_handle = app.handle().clone();

  if let Some(window) = app_handle.get_webview_window(desktop_shell::PORTAL_WINDOW_LABEL) {
    let app_handle = app_handle.clone();
    window.on_window_event(move |event| {
      if let WindowEvent::CloseRequested { api, .. } = event {
        api.prevent_close();
        let _ = hide_window_by_label(&app_handle, desktop_shell::PORTAL_WINDOW_LABEL);
      }
    });
  }

  let tray = TrayIconBuilder::new()
    .menu(&menu)
    .show_menu_on_left_click(false)
    .icon(
      app.default_window_icon()
        .cloned()
        .context("desktop tray icon is unavailable")?,
    )
    .on_tray_icon_event({
      let app_handle = app_handle.clone();
      move |_, event| {
        if let TrayIconEvent::Click { button, button_state, .. } = event {
          if button == MouseButton::Left && button_state == MouseButtonState::Up {
            let _ = toggle_main_window_visibility(&app_handle);
          }
        }
      }
    })
    .on_menu_event({
      let app_handle = app_handle.clone();
      move |_, event| {
        handle_tray_menu_event(&app_handle, event);
      }
    })
    .build(app)
    .context("failed to build portal tray icon")?;

  app.manage(TrayState { _tray: tray });
  Ok(())
}

fn build_tray_menu(app: &tauri::App) -> anyhow::Result<tauri::menu::Menu<tauri::Wry>> {
  let menu = MenuBuilder::new(app)
    .text(desktop_shell::TRAY_ACTION_SHOW_WINDOW, "显示主窗口")
    .text(desktop_shell::TRAY_ACTION_HIDE_WINDOW, "隐藏主窗口")
    .separator()
    .text(desktop_shell::TRAY_ACTION_OPEN_PORTAL, "打开门户")
    .text(desktop_shell::TRAY_ACTION_OPEN_ADMIN, "打开管理台")
    .text(desktop_shell::TRAY_ACTION_OPEN_GATEWAY, "打开网关")
    .separator()
    .text(desktop_shell::TRAY_ACTION_RESTART_RUNTIME, "重启运行时")
    .separator()
    .text(desktop_shell::TRAY_ACTION_QUIT_APP, "退出应用")
    .build()
    .context("failed to build portal tray menu")?;

  Ok(menu)
}

fn handle_tray_menu_event(app_handle: &AppHandle, event: MenuEvent) {
  match event.id().as_ref() {
    desktop_shell::TRAY_ACTION_SHOW_WINDOW => {
      let _ = show_main_window(app_handle);
    }
    desktop_shell::TRAY_ACTION_HIDE_WINDOW => {
      let _ = hide_main_window(app_handle);
    }
    desktop_shell::TRAY_ACTION_OPEN_PORTAL => {
      let _ = show_main_window(app_handle);
    }
    desktop_shell::TRAY_ACTION_OPEN_ADMIN => {
      let _ = open_runtime_window(
        app_handle,
        desktop_shell::ADMIN_WINDOW_LABEL,
        "SDKWork Router Admin",
        desktop_shell::admin_url,
        1440.0,
        960.0,
      );
    }
    desktop_shell::TRAY_ACTION_OPEN_GATEWAY => {
      let _ = open_runtime_window(
        app_handle,
        desktop_shell::GATEWAY_WINDOW_LABEL,
        "SDKWork Router Gateway",
        desktop_shell::gateway_url,
        1440.0,
        960.0,
      );
    }
    desktop_shell::TRAY_ACTION_RESTART_RUNTIME => {
      let _ = restart_runtime_from_shell(app_handle);
    }
    desktop_shell::TRAY_ACTION_QUIT_APP => {
      app_handle.exit(0);
    }
    _ => {}
  }
}

fn show_main_window(app_handle: &AppHandle) -> anyhow::Result<()> {
  let window = app_handle
    .get_webview_window(desktop_shell::PORTAL_WINDOW_LABEL)
    .context("main portal window is unavailable")?;
  show_window(&window)
}

fn hide_main_window(app_handle: &AppHandle) -> anyhow::Result<()> {
  let window = app_handle
    .get_webview_window(desktop_shell::PORTAL_WINDOW_LABEL)
    .context("main portal window is unavailable")?;
  hide_window(&window)
}

fn toggle_main_window_visibility(app_handle: &AppHandle) -> anyhow::Result<()> {
  let window = app_handle
    .get_webview_window(desktop_shell::PORTAL_WINDOW_LABEL)
    .context("main portal window is unavailable")?;
  if window.is_visible().unwrap_or(false) {
    hide_window(&window)
  } else {
    show_window(&window)
  }
}

fn open_runtime_window(
  app_handle: &AppHandle,
  window_label: &'static str,
  title: &str,
  build_path: fn(&str) -> Option<String>,
  width: f64,
  height: f64,
) -> anyhow::Result<()> {
  let base_url = runtime_public_base_url(app_handle)
    .context("desktop runtime did not expose a public base URL")?;
  let url = build_path(&base_url).context("failed to build runtime URL")?;
  let target_url = url.parse().context("failed to parse runtime URL")?;

  if let Some(window) = app_handle.get_webview_window(window_label) {
    window.navigate(target_url)?;
    return show_window(&window);
  }

  let window = WebviewWindowBuilder::new(app_handle, window_label, WebviewUrl::External(target_url))
    .title(title)
    .center()
    .inner_size(width, height)
    .resizable(true)
    .decorations(false)
    .skip_taskbar(false)
    .visible(false)
    .build()
    .context("failed to build runtime window")?;

  install_window_close_hides(&window, window_label);
  show_window(&window)?;
  Ok(())
}

fn install_window_close_hides(window: &WebviewWindow, window_label: &'static str) {
  let app_handle = window.app_handle().clone();
  let window_label = window_label.to_owned();

  window.on_window_event(move |event| {
    if let WindowEvent::CloseRequested { api, .. } = event {
      api.prevent_close();
      let _ = hide_window_by_label(&app_handle, &window_label);
    }
  });
}

fn hide_window_by_label(app_handle: &AppHandle, window_label: &str) -> anyhow::Result<()> {
  let window = app_handle
    .get_webview_window(window_label)
    .with_context(|| format!("{window_label} window is unavailable"))?;
  hide_window(&window)
}

fn show_window(window: &WebviewWindow) -> anyhow::Result<()> {
  let _ = window.set_skip_taskbar(false);
  window.show()?;
  window.unminimize()?;
  window.set_focus()?;
  Ok(())
}

fn hide_window(window: &WebviewWindow) -> anyhow::Result<()> {
  let _ = window.set_skip_taskbar(true);
  window.hide()?;
  Ok(())
}

fn restart_runtime_from_shell(app_handle: &AppHandle) -> anyhow::Result<()> {
  let runtime_state = app_handle.state::<RuntimeState>();
  let runtime_handle_state = app_handle.state::<RuntimeHandleState>();
  let runtime = tauri::async_runtime::block_on(start_desktop_runtime(app_handle.clone()))?;
  let snapshot = runtime.snapshot();

  {
    let mut runtime_guard = runtime_handle_state
      .runtime
      .lock()
      .map_err(|_| anyhow::anyhow!("Desktop runtime handle state is unavailable."))?;
    *runtime_guard = Some(runtime);
  }

  {
    let mut snapshot_guard = runtime_state
      .snapshot
      .lock()
      .map_err(|_| anyhow::anyhow!("Desktop runtime snapshot state is unavailable."))?;
    *snapshot_guard = snapshot.clone();
  }

  refresh_runtime_windows(app_handle, &snapshot)?;
  Ok(())
}

fn refresh_runtime_windows(
  app_handle: &AppHandle,
  snapshot: &RouterProductRuntimeSnapshot,
) -> anyhow::Result<()> {
  let base_url = snapshot
    .public_base_url
    .as_deref()
    .context("desktop runtime did not expose a public base URL")?;

  refresh_window(
    app_handle,
    desktop_shell::PORTAL_WINDOW_LABEL,
    desktop_shell::portal_url(base_url),
  )?;
  refresh_window(
    app_handle,
    desktop_shell::ADMIN_WINDOW_LABEL,
    desktop_shell::admin_url(base_url),
  )?;
  refresh_window(
    app_handle,
    desktop_shell::GATEWAY_WINDOW_LABEL,
    desktop_shell::gateway_url(base_url),
  )?;

  Ok(())
}

fn refresh_window(
  app_handle: &AppHandle,
  window_label: &'static str,
  target_url: Option<String>,
) -> anyhow::Result<()> {
  let Some(target_url) = target_url else {
    return Ok(());
  };
  let Some(window) = app_handle.get_webview_window(window_label) else {
    return Ok(());
  };

  let target_url = target_url.parse().context("failed to parse runtime URL")?;
  window.navigate(target_url)?;
  Ok(())
}

fn box_setup_error(error: anyhow::Error) -> Box<dyn std::error::Error> {
  Box::new(std::io::Error::new(
    std::io::ErrorKind::Other,
    error.to_string(),
  ))
}

fn current_runtime_snapshot(
  state: &tauri::State<'_, RuntimeState>,
) -> Result<RouterProductRuntimeSnapshot, String> {
  state
    .snapshot
    .lock()
    .map_err(|_| "Desktop runtime snapshot state is unavailable.".to_string())
    .map(|snapshot| snapshot.clone())
}

fn runtime_public_base_url(app_handle: &AppHandle) -> Option<String> {
  let state = app_handle.state::<RuntimeState>();
  current_runtime_snapshot(&state)
    .ok()
    .and_then(|snapshot| snapshot.public_base_url)
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
    resolve_resource_or_fallback(
      app,
      "embedded-sites/admin",
      workspace_dirs.admin_site_dir,
    )?,
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
    .expect("portal src-tauri must live inside the portal app")
    .parent()
    .expect("portal app must live inside the apps directory");
  ProductSiteDirs::new(
    apps_root.join("sdkwork-router-admin").join("dist"),
    apps_root.join("sdkwork-router-portal").join("dist"),
  )
}

fn env_flag_is_truthy(name: &str) -> bool {
  matches!(
    std::env::var(name).ok().as_deref().map(str::trim).unwrap_or_default(),
    "1" | "true" | "TRUE" | "yes" | "YES" | "on" | "ON"
  )
}

#[cfg(test)]
mod tests {
  use anyhow::anyhow;
  use sdkwork_api_product_runtime::RouterProductRuntimeSnapshot;

  use super::box_setup_error;

  #[test]
  fn box_setup_error_preserves_context_message() {
    let error = box_setup_error(anyhow!("desktop runtime boot failed"));
    assert_eq!(error.to_string(), "desktop runtime boot failed");
  }

  #[test]
  fn runtime_snapshot_remains_cloneable_for_runtime_control_updates() {
    let snapshot = RouterProductRuntimeSnapshot {
      mode: "desktop".to_owned(),
      roles: vec![
        "web".to_owned(),
        "gateway".to_owned(),
        "admin".to_owned(),
        "portal".to_owned(),
      ],
      public_base_url: Some("http://127.0.0.1:48123".to_owned()),
      public_bind_addr: Some("127.0.0.1:48123".to_owned()),
      gateway_bind_addr: Some("127.0.0.1:8080".to_owned()),
      admin_bind_addr: Some("127.0.0.1:8081".to_owned()),
      portal_bind_addr: Some("127.0.0.1:8082".to_owned()),
    };

    let next = snapshot.clone();
    assert_eq!(next.public_base_url.as_deref(), Some("http://127.0.0.1:48123"));
    assert_eq!(next.gateway_bind_addr.as_deref(), Some("127.0.0.1:8080"));
  }
}
