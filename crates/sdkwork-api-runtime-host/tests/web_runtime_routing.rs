use std::path::PathBuf;

use sdkwork_api_runtime_host::{
    classify_request, resolve_static_asset, RuntimeHostConfig, RuntimeRoute, RuntimeSite, SiteAsset,
};

fn fixture_root(site: &str) -> PathBuf {
    PathBuf::from("/tmp/sdkwork-router-runtime-tests").join(site)
}

#[test]
fn classify_request_redirects_root_to_portal() {
    assert_eq!(
        classify_request("/"),
        RuntimeRoute::Redirect("/portal/".to_owned())
    );
}

#[test]
fn classify_request_rewrites_api_routes_to_service_paths() {
    assert_eq!(
        classify_request("/api/admin/users/operators"),
        RuntimeRoute::Proxy {
            upstream: "admin".to_owned(),
            request_path: "/admin/users/operators".to_owned(),
        }
    );
    assert_eq!(
        classify_request("/api/portal/dashboard"),
        RuntimeRoute::Proxy {
            upstream: "portal".to_owned(),
            request_path: "/portal/dashboard".to_owned(),
        }
    );
    assert_eq!(
        classify_request("/api/v1/models"),
        RuntimeRoute::Proxy {
            upstream: "gateway".to_owned(),
            request_path: "/v1/models".to_owned(),
        }
    );
}

#[test]
fn classify_request_identifies_static_site_mounts() {
    assert_eq!(
        classify_request("/portal/assets/index.js"),
        RuntimeRoute::Static {
            site: RuntimeSite::Portal,
            request_path: "/portal/assets/index.js".to_owned(),
        }
    );
    assert_eq!(
        classify_request("/admin/#overview"),
        RuntimeRoute::Static {
            site: RuntimeSite::Admin,
            request_path: "/admin/#overview".to_owned(),
        }
    );
}

#[test]
fn resolve_static_asset_uses_index_for_hash_and_route_navigation() {
    let portal_root = fixture_root("portal");
    assert_eq!(
        resolve_static_asset(RuntimeSite::Portal, "/portal/", &portal_root).unwrap(),
        SiteAsset {
            site: RuntimeSite::Portal,
            filesystem_path: portal_root.join("index.html"),
            content_type: "text/html; charset=utf-8".to_owned(),
            cache_control: "no-cache".to_owned(),
        }
    );
    assert_eq!(
        resolve_static_asset(RuntimeSite::Portal, "/portal/dashboard", &portal_root).unwrap(),
        SiteAsset {
            site: RuntimeSite::Portal,
            filesystem_path: portal_root.join("index.html"),
            content_type: "text/html; charset=utf-8".to_owned(),
            cache_control: "no-cache".to_owned(),
        }
    );
}

#[test]
fn resolve_static_asset_maps_hashed_assets_and_rejects_traversal() {
    let admin_root = fixture_root("admin");
    assert_eq!(
        resolve_static_asset(
            RuntimeSite::Admin,
            "/admin/assets/index-abc123.js",
            &admin_root
        )
        .unwrap(),
        SiteAsset {
            site: RuntimeSite::Admin,
            filesystem_path: admin_root.join("assets").join("index-abc123.js"),
            content_type: "text/javascript; charset=utf-8".to_owned(),
            cache_control: "public, max-age=31536000, immutable".to_owned(),
        }
    );

    assert!(
        resolve_static_asset(RuntimeSite::Admin, "/admin/../../secret.txt", &admin_root).is_err()
    );
}

#[test]
fn local_defaults_bind_pingora_for_public_access() {
    let config = RuntimeHostConfig::local_defaults("0.0.0.0:3001");

    assert_eq!(config.bind_addr, "0.0.0.0:3001");
    assert_eq!(
        config.admin_site_dir,
        PathBuf::from("apps/sdkwork-router-admin/dist")
    );
    assert_eq!(
        config.portal_site_dir,
        PathBuf::from("apps/sdkwork-router-portal/dist")
    );
}
