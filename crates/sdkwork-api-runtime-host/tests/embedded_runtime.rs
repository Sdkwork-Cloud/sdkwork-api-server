use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use sdkwork_api_runtime_host::{EmbeddedRuntime, RuntimeHostConfig};

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
