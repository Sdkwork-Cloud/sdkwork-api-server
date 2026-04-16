use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{to_bytes, Body};
use axum::extract::Request;
use axum::http::StatusCode;
use axum::routing::any;
use axum::{response::Response, Router};
use reqwest::Client;
use sdkwork_api_runtime_host::{EmbeddedRuntime, RuntimeHostConfig};
use tokio::net::TcpListener as TokioTcpListener;

fn unique_temp_dir(name: &str) -> PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("sdkwork-runtime-host-{name}-{suffix}"))
}

fn read_response_headers(stream: &mut TcpStream) -> String {
    let mut buffer = [0_u8; 4096];
    let mut response = Vec::new();
    stream
        .set_read_timeout(Some(std::time::Duration::from_secs(2)))
        .unwrap();

    loop {
        let read = stream.read(&mut buffer).unwrap();
        if read == 0 {
            break;
        }
        response.extend_from_slice(&buffer[..read]);
        if response.windows(4).any(|window| window == b"\r\n\r\n") {
            break;
        }
    }

    String::from_utf8(response).unwrap()
}

async fn spawn_echo_upstream(name: &'static str) -> String {
    let listener = TokioTcpListener::bind("127.0.0.1:0").await.unwrap();
    let bind_addr = listener.local_addr().unwrap().to_string();
    let router = Router::new().fallback(any(move |request: Request<Body>| async move {
        let path_and_query = request
            .uri()
            .path_and_query()
            .map(|value| value.as_str().to_owned())
            .unwrap_or_else(|| request.uri().path().to_owned());
        let body = to_bytes(request.into_body(), usize::MAX).await.unwrap();
        Response::builder()
            .status(StatusCode::OK)
            .header("content-type", "text/plain; charset=utf-8")
            .header("x-upstream-name", name)
            .body(Body::from(format!(
                "{name}:{path_and_query}:{}",
                String::from_utf8_lossy(&body)
            )))
            .unwrap()
    }));

    tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });

    bind_addr
}

#[tokio::test]
async fn embedded_runtime_starts_on_loopback() {
    if TcpListener::bind("127.0.0.1:0").is_err() {
        return;
    }

    let runtime = EmbeddedRuntime::start_ephemeral().await.unwrap();
    assert!(runtime.base_url().starts_with("http://127.0.0.1:"));
}

#[tokio::test]
async fn root_redirect_includes_zero_content_length_for_keep_alive_clients() {
    if TcpListener::bind("127.0.0.1:0").is_err() {
        return;
    }

    let admin_dir = unique_temp_dir("admin");
    let portal_dir = unique_temp_dir("portal");
    std::fs::create_dir_all(&admin_dir).unwrap();
    std::fs::create_dir_all(&portal_dir).unwrap();
    std::fs::write(
        admin_dir.join("index.html"),
        "<!doctype html><title>admin</title>",
    )
    .unwrap();
    std::fs::write(
        portal_dir.join("index.html"),
        "<!doctype html><title>portal</title>",
    )
    .unwrap();

    let runtime = EmbeddedRuntime::start(RuntimeHostConfig::new(
        "127.0.0.1:0",
        &admin_dir,
        &portal_dir,
        "127.0.0.1:9",
        "127.0.0.1:9",
        "127.0.0.1:9",
    ))
    .await
    .unwrap();

    let address = runtime.base_url().trim_start_matches("http://");
    let mut stream = TcpStream::connect(address).unwrap();
    stream
        .write_all(b"GET / HTTP/1.1\r\nHost: localhost\r\n\r\n")
        .unwrap();

    let response = read_response_headers(&mut stream);

    assert!(response.starts_with("HTTP/1.1 302"));
    assert!(response.contains("location: /portal/\r\n"));
    assert!(response.contains("content-length: 0\r\n"));
}

#[tokio::test]
async fn runtime_serves_static_sites_and_proxies_api_routes() {
    if TcpListener::bind("127.0.0.1:0").is_err() {
        return;
    }

    let admin_dir = unique_temp_dir("admin-static");
    let portal_dir = unique_temp_dir("portal-static");
    std::fs::create_dir_all(admin_dir.join("assets")).unwrap();
    std::fs::create_dir_all(portal_dir.join("assets")).unwrap();
    std::fs::write(
        admin_dir.join("index.html"),
        "<!doctype html><title>admin-home</title>",
    )
    .unwrap();
    std::fs::write(
        portal_dir.join("index.html"),
        "<!doctype html><title>portal-home</title>",
    )
    .unwrap();
    std::fs::write(
        admin_dir.join("assets").join("main.js"),
        "console.log('admin');",
    )
    .unwrap();

    let runtime = EmbeddedRuntime::start(
        RuntimeHostConfig::new(
            "127.0.0.1:0",
            &admin_dir,
            &portal_dir,
            spawn_echo_upstream("admin").await,
            spawn_echo_upstream("portal").await,
            spawn_echo_upstream("gateway").await,
        )
        .with_browser_allowed_origins(["https://console.example.com"]),
    )
    .await
    .unwrap();

    sdkwork_api_kernel::ensure_reqwest_rustls_provider();
    let client = Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    let root_response = client.get(runtime.base_url()).send().await.unwrap();
    assert_eq!(root_response.status(), reqwest::StatusCode::FOUND);
    assert_eq!(root_response.headers().get("location").unwrap(), "/portal/");

    let admin_response = client
        .get(format!("{}/admin/", runtime.base_url()))
        .send()
        .await
        .unwrap();
    assert_eq!(admin_response.status(), reqwest::StatusCode::OK);
    assert!(admin_response.text().await.unwrap().contains("admin-home"));

    let asset_response = client
        .get(format!("{}/admin/assets/main.js", runtime.base_url()))
        .send()
        .await
        .unwrap();
    assert_eq!(asset_response.status(), reqwest::StatusCode::OK);
    assert_eq!(
        asset_response.headers().get("cache-control").unwrap(),
        "public, max-age=31536000, immutable"
    );

    let admin_proxy_response = client
        .post(format!(
            "{}/api/admin/sessions?next=dashboard",
            runtime.base_url()
        ))
        .header("origin", "https://console.example.com")
        .body("payload-body")
        .send()
        .await
        .unwrap();
    assert_eq!(admin_proxy_response.status(), reqwest::StatusCode::OK);
    assert_eq!(
        admin_proxy_response
            .headers()
            .get("access-control-allow-origin")
            .unwrap(),
        "https://console.example.com"
    );
    assert_eq!(
        admin_proxy_response
            .headers()
            .get("x-upstream-name")
            .unwrap(),
        "admin"
    );
    assert_eq!(
        admin_proxy_response.text().await.unwrap(),
        "admin:/admin/sessions?next=dashboard:payload-body"
    );

    let gateway_health_response = client
        .get(format!("{}/api/v1/health", runtime.base_url()))
        .header("origin", "https://console.example.com")
        .send()
        .await
        .unwrap();
    assert_eq!(gateway_health_response.status(), reqwest::StatusCode::OK);
    assert_eq!(
        gateway_health_response
            .headers()
            .get("access-control-allow-origin")
            .unwrap(),
        "https://console.example.com"
    );
    assert_eq!(
        gateway_health_response
            .headers()
            .get("x-upstream-name")
            .unwrap(),
        "gateway"
    );
    assert_eq!(
        gateway_health_response.text().await.unwrap(),
        "gateway:/health:"
    );

    let gateway_openapi_response = client
        .get(format!("{}/openapi.json", runtime.base_url()))
        .header("origin", "https://console.example.com")
        .send()
        .await
        .unwrap();
    assert_eq!(gateway_openapi_response.status(), reqwest::StatusCode::OK);
    assert_eq!(
        gateway_openapi_response
            .headers()
            .get("x-upstream-name")
            .unwrap(),
        "gateway"
    );
    assert_eq!(
        gateway_openapi_response.text().await.unwrap(),
        "gateway:/openapi.json:"
    );

    let gateway_docs_response = client
        .get(format!("{}/docs", runtime.base_url()))
        .header("origin", "https://console.example.com")
        .send()
        .await
        .unwrap();
    assert_eq!(gateway_docs_response.status(), reqwest::StatusCode::OK);
    assert_eq!(
        gateway_docs_response.headers().get("x-upstream-name").unwrap(),
        "gateway"
    );
    assert_eq!(gateway_docs_response.text().await.unwrap(), "gateway:/docs:");

    let preflight_response = client
        .request(
            reqwest::Method::OPTIONS,
            format!("{}/api/v1/health", runtime.base_url()),
        )
        .header("origin", "https://console.example.com")
        .send()
        .await
        .unwrap();
    assert_eq!(preflight_response.status(), reqwest::StatusCode::NO_CONTENT);
    assert_eq!(
        preflight_response
            .headers()
            .get("access-control-allow-origin")
            .unwrap(),
        "https://console.example.com"
    );

    let disallowed_origin_response = client
        .get(format!("{}/api/v1/health", runtime.base_url()))
        .header("origin", "https://evil.example.com")
        .send()
        .await
        .unwrap();
    assert_eq!(disallowed_origin_response.status(), reqwest::StatusCode::OK);
    assert!(disallowed_origin_response
        .headers()
        .get("access-control-allow-origin")
        .is_none());
}
