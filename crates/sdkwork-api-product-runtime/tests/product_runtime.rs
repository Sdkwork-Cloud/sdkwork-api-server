use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use reqwest::Client;
use sdkwork_api_config::StandaloneConfigLoader;
use sdkwork_api_product_runtime::{
    ProductRuntimeRole, ProductSiteDirs, RouterProductRuntime, RouterProductRuntimeOptions,
};

static TEMP_DIR_COUNTER: AtomicU64 = AtomicU64::new(0);

#[tokio::test]
async fn desktop_product_runtime_serves_static_sites_and_all_api_health_routes() {
    let config_root = temp_root("desktop-runtime-config");
    let admin_site_dir = temp_root("desktop-admin-site");
    let portal_site_dir = temp_root("desktop-portal-site");
    fs::write(
        admin_site_dir.join("index.html"),
        "<!doctype html><html><body>admin desktop site</body></html>",
    )
    .unwrap();
    fs::write(
        portal_site_dir.join("index.html"),
        "<!doctype html><html><body>portal desktop site</body></html>",
    )
    .unwrap();

    let (loader, config) = StandaloneConfigLoader::from_local_root_and_pairs(
        &config_root,
        std::iter::empty::<(&str, &str)>(),
    )
    .unwrap();

    let runtime = RouterProductRuntime::start(
        loader,
        config,
        RouterProductRuntimeOptions::desktop(ProductSiteDirs::new(
            &admin_site_dir,
            &portal_site_dir,
        )),
    )
    .await
    .unwrap();

    let base_url = runtime.public_base_url().unwrap().to_owned();
    let snapshot = runtime.snapshot();
    let client = http_client();

    assert_eq!(snapshot.mode, "desktop");
    assert_eq!(
        snapshot.roles,
        vec![
            "web".to_owned(),
            "gateway".to_owned(),
            "admin".to_owned(),
            "portal".to_owned()
        ]
    );
    assert_eq!(snapshot.public_base_url.as_deref(), Some(base_url.as_str()));
    assert!(snapshot
        .public_bind_addr
        .as_deref()
        .unwrap()
        .starts_with("127.0.0.1:"));
    assert!(snapshot
        .gateway_bind_addr
        .as_deref()
        .unwrap()
        .starts_with("127.0.0.1:"));
    assert!(snapshot
        .admin_bind_addr
        .as_deref()
        .unwrap()
        .starts_with("127.0.0.1:"));
    assert!(snapshot
        .portal_bind_addr
        .as_deref()
        .unwrap()
        .starts_with("127.0.0.1:"));

    assert_eq!(
        client
            .get(format!("{base_url}/api/admin/health"))
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap(),
        "ok"
    );
    assert_eq!(
        client
            .get(format!("{base_url}/api/portal/health"))
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap(),
        "ok"
    );
    assert_eq!(
        client
            .get(format!("{base_url}/api/v1/health"))
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap(),
        "ok"
    );
    assert!(client
        .get(format!("{base_url}/admin/"))
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap()
        .contains("admin desktop site"));
    assert!(client
        .get(format!("{base_url}/portal/"))
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap()
        .contains("portal desktop site"));
}

#[tokio::test]
async fn server_product_runtime_rejects_web_role_without_required_api_upstreams() {
    let config_root = temp_root("server-runtime-config");
    let admin_site_dir = temp_root("server-admin-site");
    let portal_site_dir = temp_root("server-portal-site");
    fs::write(admin_site_dir.join("index.html"), "admin").unwrap();
    fs::write(portal_site_dir.join("index.html"), "portal").unwrap();

    let (loader, config) = StandaloneConfigLoader::from_local_root_and_pairs(
        &config_root,
        std::iter::empty::<(&str, &str)>(),
    )
    .unwrap();

    let error = RouterProductRuntime::start(
        loader,
        config,
        RouterProductRuntimeOptions::server(ProductSiteDirs::new(
            &admin_site_dir,
            &portal_site_dir,
        ))
        .with_roles([ProductRuntimeRole::Web]),
    )
    .await
    .err()
    .expect("web-only server runtime without API upstreams should fail");

    assert!(error.to_string().contains("gateway upstream"));
}

fn temp_root(label: &str) -> PathBuf {
    let unique = TEMP_DIR_COUNTER.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!("sdkwork-product-runtime-tests-{label}-{unique}"));
    if root.exists() {
        fs::remove_dir_all(&root).unwrap();
    }
    fs::create_dir_all(&root).unwrap();
    root
}

fn http_client() -> Client {
    Client::builder().build().unwrap()
}
