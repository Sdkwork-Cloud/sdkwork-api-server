const PORTAL_ROUTE: &str = "/portal/";
const ADMIN_ROUTE: &str = "/admin/";
const GATEWAY_ROUTE: &str = "/api/v1/docs";

pub const PORTAL_WINDOW_LABEL: &str = "main";
pub const ADMIN_WINDOW_LABEL: &str = "router-admin";
pub const GATEWAY_WINDOW_LABEL: &str = "router-gateway";

pub const TRAY_ACTION_SHOW_WINDOW: &str = "show-window";
pub const TRAY_ACTION_HIDE_WINDOW: &str = "hide-window";
pub const TRAY_ACTION_OPEN_PORTAL: &str = "open-portal";
pub const TRAY_ACTION_OPEN_ADMIN: &str = "open-admin";
pub const TRAY_ACTION_OPEN_GATEWAY: &str = "open-gateway";
pub const TRAY_ACTION_RESTART_RUNTIME: &str = "restart-runtime";
pub const TRAY_ACTION_QUIT_APP: &str = "quit-app";

pub fn portal_url(base_url: &str) -> Option<String> {
  runtime_page_url(base_url, PORTAL_ROUTE)
}

pub fn admin_url(base_url: &str) -> Option<String> {
  runtime_page_url(base_url, ADMIN_ROUTE)
}

pub fn gateway_url(base_url: &str) -> Option<String> {
  runtime_page_url(base_url, GATEWAY_ROUTE)
}

fn runtime_page_url(base_url: &str, route: &str) -> Option<String> {
  let normalized_base = base_url.trim().trim_end_matches('/');
  if normalized_base.is_empty() {
    return None;
  }

  let normalized_route = if route.starts_with('/') {
    route
  } else {
    return None;
  };

  Some(format!("{normalized_base}{normalized_route}"))
}

#[cfg(test)]
mod tests {
  use super::{admin_url, gateway_url, portal_url};

  #[test]
  fn portal_admin_and_gateway_urls_append_the_expected_runtime_paths() {
    let base_url = "http://127.0.0.1:48123/";

    assert_eq!(portal_url(base_url).as_deref(), Some("http://127.0.0.1:48123/portal/"));
    assert_eq!(admin_url(base_url).as_deref(), Some("http://127.0.0.1:48123/admin/"));
    assert_eq!(
      gateway_url(base_url).as_deref(),
      Some("http://127.0.0.1:48123/api/v1/docs"),
    );
  }
}
