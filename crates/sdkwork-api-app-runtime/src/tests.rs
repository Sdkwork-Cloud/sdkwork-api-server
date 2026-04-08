use super::runtime_reload::{merge_applied_service_config, restart_required_changed_fields};
use super::*;
use sdkwork_api_app_credential::persist_credential_with_secret_and_manager;
use sdkwork_api_config::CacheBackendKind;
use sdkwork_api_storage_sqlite::{SqliteAdminStore, run_migrations};
use std::io::Write;

#[tokio::test]
async fn validate_secret_manager_for_store_checks_multiple_credentials() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);
    let manager = CredentialSecretManager::database_encrypted("runtime-test-master-key");

    persist_credential_with_secret_and_manager(
        &store,
        &manager,
        "tenant-1",
        "provider-a",
        "cred-a",
        "secret-a",
    )
    .await
    .unwrap();
    persist_credential_with_secret_and_manager(
        &store,
        &manager,
        "tenant-2",
        "provider-b",
        "cred-b",
        "secret-b",
    )
    .await
    .unwrap();

    validate_secret_manager_for_store(&store, &manager)
        .await
        .unwrap();
}

#[tokio::test]
async fn build_cache_runtime_from_config_returns_memory_cache_runtime() {
    let config = StandaloneConfig::default();

    let stores = build_cache_runtime_from_config(&config).await.unwrap();
    stores
        .cache_store()
        .put("routing", "selection", b"provider-a".to_vec(), None, &[])
        .await
        .unwrap();
    let cached = stores
        .cache_store()
        .get("routing", "selection")
        .await
        .unwrap()
        .expect("cached entry");

    assert_eq!(cached.value(), b"provider-a");
}

#[tokio::test]
async fn build_cache_runtime_from_config_builds_redis_cache_runtime() {
    let mut config = StandaloneConfig::default();
    config.cache_backend = CacheBackendKind::Redis;
    let server = MinimalRedisPingServer::start();
    config.cache_url = Some(server.url_with_db(4));

    build_cache_runtime_from_config(&config).await.unwrap();
}

#[tokio::test]
async fn build_admin_store_from_config_surfaces_supported_dialects_for_mysql() {
    let mut config = StandaloneConfig::default();
    config.database_url = "mysql://router:secret@localhost:3306/router".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!("mysql should remain unsupported until a real driver ships"),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("mysql"));
    assert!(error.contains("supported dialects"));
    assert!(error.contains("sqlite"));
    assert!(error.contains("postgres"));
}

#[tokio::test]
async fn build_admin_store_from_config_surfaces_supported_dialects_for_libsql() {
    let mut config = StandaloneConfig::default();
    config.database_url = "libsql://router.example.com".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!("libsql should remain unsupported until a real driver ships"),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("libsql"));
    assert!(error.contains("supported dialects"));
    assert!(error.contains("sqlite"));
    assert!(error.contains("postgres"));
}

#[test]
fn restart_required_changed_fields_include_cache_backend_for_gateway_runtime() {
    let current = StandaloneConfig::default();
    let next = StandaloneConfig {
        cache_backend: CacheBackendKind::Redis,
        cache_url: Some("redis://127.0.0.1:6379/8".to_owned()),
        ..current.clone()
    };

    let changed = restart_required_changed_fields(StandaloneServiceKind::Gateway, &current, &next);

    assert!(changed.contains(&"cache_backend"));
    assert!(changed.contains(&"cache_url"));
}

#[test]
fn merge_applied_service_config_keeps_gateway_cache_backend_on_restart_required_changes() {
    let current = StandaloneConfig::default();
    let next = StandaloneConfig {
        gateway_bind: "127.0.0.1:19090".to_owned(),
        cache_backend: CacheBackendKind::Redis,
        cache_url: Some("redis://127.0.0.1:6379/9".to_owned()),
        ..current.clone()
    };

    let applied = merge_applied_service_config(StandaloneServiceKind::Gateway, &current, &next);

    assert_eq!(applied.gateway_bind, "127.0.0.1:19090");
    assert_eq!(applied.cache_backend, CacheBackendKind::Memory);
    assert_eq!(applied.cache_url, None);
}

struct MinimalRedisPingServer {
    address: String,
    stop: std::sync::Arc<std::sync::atomic::AtomicBool>,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl MinimalRedisPingServer {
    fn start() -> Self {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let address = listener.local_addr().unwrap().to_string();
        let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let thread_stop = stop.clone();
        let thread = std::thread::spawn(move || {
            while !thread_stop.load(std::sync::atomic::Ordering::Relaxed) {
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        stream.set_nonblocking(false).unwrap();
                        loop {
                            match read_minimal_resp_array(&mut stream) {
                                Ok(Some(command)) => match String::from_utf8_lossy(&command[0])
                                    .to_ascii_uppercase()
                                    .as_str()
                                {
                                    "PING" => {
                                        stream.write_all(b"+PONG\r\n").unwrap();
                                        stream.flush().unwrap();
                                    }
                                    "GET" => {
                                        stream.write_all(b"$-1\r\n").unwrap();
                                        stream.flush().unwrap();
                                    }
                                    "AUTH" | "SELECT" => {
                                        stream.write_all(b"+OK\r\n").unwrap();
                                        stream.flush().unwrap();
                                    }
                                    other => panic!("unexpected minimal redis command: {other}"),
                                },
                                Ok(None) => break,
                                Err(error)
                                    if matches!(
                                        error.kind(),
                                        std::io::ErrorKind::UnexpectedEof
                                            | std::io::ErrorKind::ConnectionReset
                                            | std::io::ErrorKind::TimedOut
                                    ) =>
                                {
                                    break;
                                }
                                Err(error) => panic!("minimal redis server read failed: {error}"),
                            }
                        }
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        std::thread::sleep(std::time::Duration::from_millis(10));
                    }
                    Err(error) => panic!("minimal redis accept failed: {error}"),
                }
            }
        });

        Self {
            address,
            stop,
            thread: Some(thread),
        }
    }

    fn url_with_db(&self, db: u32) -> String {
        format!("redis://{}/{db}", self.address)
    }
}

impl Drop for MinimalRedisPingServer {
    fn drop(&mut self) {
        self.stop.store(true, std::sync::atomic::Ordering::Relaxed);
        let _ = std::net::TcpStream::connect(&self.address);
        if let Some(thread) = self.thread.take() {
            thread.join().unwrap();
        }
    }
}

fn read_minimal_resp_array(
    stream: &mut std::net::TcpStream,
) -> std::io::Result<Option<Vec<Vec<u8>>>> {
    let mut marker = [0_u8; 1];
    match std::io::Read::read_exact(stream, &mut marker) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(error) => return Err(error),
    }
    assert_eq!(marker[0], b'*');
    let count = read_minimal_resp_line(stream)?.parse::<usize>().unwrap();
    let mut values = Vec::with_capacity(count);
    for _ in 0..count {
        let mut bulk_marker = [0_u8; 1];
        std::io::Read::read_exact(stream, &mut bulk_marker)?;
        assert_eq!(bulk_marker[0], b'$');
        let length = read_minimal_resp_line(stream)?.parse::<usize>().unwrap();
        let mut value = vec![0_u8; length];
        std::io::Read::read_exact(stream, &mut value)?;
        let mut crlf = [0_u8; 2];
        std::io::Read::read_exact(stream, &mut crlf)?;
        values.push(value);
    }
    Ok(Some(values))
}

fn read_minimal_resp_line(stream: &mut std::net::TcpStream) -> std::io::Result<String> {
    let mut bytes = Vec::new();
    loop {
        let mut byte = [0_u8; 1];
        std::io::Read::read_exact(stream, &mut byte)?;
        if byte[0] == b'\r' {
            let mut newline = [0_u8; 1];
            std::io::Read::read_exact(stream, &mut newline)?;
            assert_eq!(newline[0], b'\n');
            break;
        }
        bytes.push(byte[0]);
    }
    Ok(String::from_utf8(bytes).unwrap())
}
