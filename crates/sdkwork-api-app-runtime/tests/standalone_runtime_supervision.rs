use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration as StdDuration;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::{routing::get, Router};
use reqwest::Client;
use sdkwork_api_app_credential::{
    persist_credential_with_secret_and_manager, resolve_provider_secret_with_manager,
    CredentialSecretManager,
};
use sdkwork_api_app_gateway::reload_configured_extension_host;
use sdkwork_api_app_runtime::{
    create_extension_runtime_rollout, create_standalone_config_rollout,
    find_extension_runtime_rollout, find_standalone_config_rollout,
    start_extension_runtime_rollout_supervision, start_standalone_runtime_supervision,
    CreateStandaloneConfigRolloutRequest, StandaloneListenerHost, StandaloneServiceKind,
    StandaloneServiceReloadHandles,
};
use sdkwork_api_config::StandaloneConfigLoader;
use sdkwork_api_domain_catalog::ModelCatalogEntry;
use sdkwork_api_ext_provider_native_mock::FIXTURE_EXTENSION_ID;
use sdkwork_api_extension_core::{
    CompatibilityLevel, ExtensionKind, ExtensionManifest, ExtensionPermission, ExtensionProtocol,
    ExtensionRuntime,
};
use sdkwork_api_extension_host::shutdown_all_native_dynamic_runtimes;
use sdkwork_api_storage_core::{
    AdminStore, ExtensionRuntimeRolloutParticipantRecord, ExtensionRuntimeRolloutRecord,
    Reloadable, ServiceRuntimeNodeRecord, StandaloneConfigRolloutParticipantRecord,
    StandaloneConfigRolloutRecord,
};
use sdkwork_api_storage_sqlite::{run_migrations, SqliteAdminStore};
use serial_test::serial;
use tokio::time::{sleep, Duration};

#[tokio::test]
async fn standalone_listener_host_rebinds_requests_to_new_bind() {
    let initial_bind = available_bind();
    let next_bind = available_bind();
    let host =
        StandaloneListenerHost::bind(initial_bind.clone(), health_router("listener-host-initial"))
            .await
            .unwrap();

    wait_for_health_response(&initial_bind, "listener-host-initial").await;
    host.reload_handle()
        .rebind(next_bind.clone())
        .await
        .unwrap();

    wait_for_health_response(&next_bind, "listener-host-initial").await;
    wait_for_health_unreachable(&initial_bind).await;

    host.shutdown().await.unwrap();
}

#[serial(extension_env)]
#[tokio::test]
async fn standalone_runtime_supervision_reloads_extension_runtime_after_config_file_change() {
    shutdown_all_native_dynamic_runtimes().unwrap();

    let log_guard = NativeDynamicLifecycleLogGuard::new();
    let config_root = temp_root("runtime-config-reload");
    let extension_root = config_root.join("extensions");
    let package_dir = extension_root.join("sdkwork-provider-native-mock");
    fs::create_dir_all(&package_dir).unwrap();

    let library_path = native_dynamic_fixture_library_path();
    let manifest = native_dynamic_manifest(&library_path);
    let manifest_text = toml::to_string(&manifest).unwrap();
    fs::write(package_dir.join("sdkwork-extension.toml"), &manifest_text).unwrap();

    write_runtime_config(&config_root, true, &extension_root);

    let (loader, initial_config) = StandaloneConfigLoader::from_local_root_and_pairs(
        &config_root,
        std::iter::empty::<(&str, &str)>(),
    )
    .unwrap();
    initial_config.apply_to_process_env();

    let first = reload_configured_extension_host().unwrap();
    assert_eq!(first.discovered_package_count, 1);
    assert_eq!(first.loadable_package_count, 1);
    wait_for_lifecycle_log(log_guard.path(), &["init"]).await;

    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store: Arc<dyn AdminStore> = Arc::new(SqliteAdminStore::new(pool));
    let supervision = start_standalone_runtime_supervision(
        StandaloneServiceKind::Gateway,
        loader,
        initial_config,
        StandaloneServiceReloadHandles::gateway(Reloadable::new(store)),
    );

    write_runtime_config(&config_root, false, &extension_root);
    wait_for_lifecycle_log(log_guard.path(), &["init", "shutdown"]).await;

    write_runtime_config(&config_root, true, &extension_root);
    wait_for_lifecycle_log(log_guard.path(), &["init", "shutdown", "init"]).await;

    drop(supervision);
    shutdown_all_native_dynamic_runtimes().unwrap();
    cleanup_dir(&config_root);
}

#[serial(runtime_config_env)]
#[tokio::test]
async fn standalone_runtime_supervision_rebinds_listener_after_config_file_change() {
    let config_root = temp_root("runtime-listener-rebind");
    let initial_bind = available_bind();
    let next_bind = available_bind();

    write_gateway_runtime_config(&config_root, &initial_bind);

    let (loader, initial_config) = StandaloneConfigLoader::from_local_root_and_pairs(
        &config_root,
        std::iter::empty::<(&str, &str)>(),
    )
    .unwrap();
    initial_config.apply_to_process_env();

    let live_store = Reloadable::new(empty_store().await);
    let listener_host =
        StandaloneListenerHost::bind(initial_bind.clone(), health_router("gateway-runtime"))
            .await
            .unwrap();
    let supervision = start_standalone_runtime_supervision(
        StandaloneServiceKind::Gateway,
        loader,
        initial_config,
        StandaloneServiceReloadHandles::gateway(live_store)
            .with_listener(listener_host.reload_handle()),
    );

    wait_for_health_response(&initial_bind, "gateway-runtime").await;
    write_gateway_runtime_config(&config_root, &next_bind);

    wait_for_health_response(&next_bind, "gateway-runtime").await;
    wait_for_health_unreachable(&initial_bind).await;

    drop(supervision);
    listener_host.shutdown().await.unwrap();
    cleanup_dir(&config_root);
}

#[serial(runtime_config_env)]
#[tokio::test]
async fn standalone_runtime_supervision_retries_listener_rebind_after_bind_failure() {
    let config_root = temp_root("runtime-listener-rebind-retry");
    let initial_bind = available_bind();
    let occupied_bind = available_bind();
    let occupied_listener = std::net::TcpListener::bind(&occupied_bind).unwrap();

    write_gateway_runtime_config(&config_root, &initial_bind);

    let (loader, initial_config) = StandaloneConfigLoader::from_local_root_and_pairs(
        &config_root,
        std::iter::empty::<(&str, &str)>(),
    )
    .unwrap();
    initial_config.apply_to_process_env();

    let live_store = Reloadable::new(empty_store().await);
    let listener_host =
        StandaloneListenerHost::bind(initial_bind.clone(), health_router("gateway-runtime"))
            .await
            .unwrap();
    let supervision = start_standalone_runtime_supervision(
        StandaloneServiceKind::Gateway,
        loader,
        initial_config,
        StandaloneServiceReloadHandles::gateway(live_store)
            .with_listener(listener_host.reload_handle()),
    );

    wait_for_health_response(&initial_bind, "gateway-runtime").await;
    write_gateway_runtime_config(&config_root, &occupied_bind);

    sleep(Duration::from_millis(1200)).await;
    wait_for_health_response(&initial_bind, "gateway-runtime").await;

    drop(occupied_listener);

    wait_for_health_response(&occupied_bind, "gateway-runtime").await;
    wait_for_health_unreachable(&initial_bind).await;

    drop(supervision);
    listener_host.shutdown().await.unwrap();
    cleanup_dir(&config_root);
}

#[serial(runtime_config_env)]
#[tokio::test]
async fn standalone_runtime_supervision_reloads_store_and_jwt_after_config_file_change() {
    let config_root = temp_root("runtime-store-jwt-reload");
    let initial_db_path = config_root.join("initial.db");
    let rotated_db_path = config_root.join("rotated.db");
    let initial_db_url = sqlite_url_for_path(&initial_db_path);
    let rotated_db_url = sqlite_url_for_path(&rotated_db_path);

    seed_model_store(&initial_db_url, "gpt-4.1-initial").await;
    seed_model_store(&rotated_db_url, "gpt-4.1-rotated").await;
    write_portal_runtime_config(&config_root, &initial_db_url, "portal-secret-initial");

    let (loader, initial_config) = StandaloneConfigLoader::from_local_root_and_pairs(
        &config_root,
        std::iter::empty::<(&str, &str)>(),
    )
    .unwrap();
    initial_config.apply_to_process_env();

    let initial_store = seed_model_store(&initial_db_url, "gpt-4.1-initial").await;
    let live_store = Reloadable::new(initial_store);
    let live_portal_jwt = Reloadable::new("portal-secret-initial".to_owned());
    let supervision = start_standalone_runtime_supervision(
        StandaloneServiceKind::Portal,
        loader,
        initial_config,
        StandaloneServiceReloadHandles::portal(live_store.clone(), live_portal_jwt.clone()),
    );

    write_portal_runtime_config(&config_root, &rotated_db_url, "portal-secret-rotated");

    wait_for_models(&live_store, &["gpt-4.1-rotated"]).await;
    wait_for_reloadable_string(&live_portal_jwt, "portal-secret-rotated").await;

    drop(supervision);
    cleanup_dir(&config_root);
}

#[serial(runtime_config_env)]
#[tokio::test]
async fn standalone_runtime_supervision_continues_heartbeat_in_reloaded_database() {
    let config_root = temp_root("runtime-heartbeat-store-reload");
    let initial_db_path = config_root.join("initial.db");
    let rotated_db_path = config_root.join("rotated.db");
    let initial_db_url = sqlite_url_for_path(&initial_db_path);
    let rotated_db_url = sqlite_url_for_path(&rotated_db_path);

    seed_model_store(&initial_db_url, "gpt-4.1-initial").await;
    seed_model_store(&rotated_db_url, "gpt-4.1-rotated").await;
    write_portal_runtime_config(&config_root, &initial_db_url, "portal-secret-initial");

    let (loader, initial_config) = StandaloneConfigLoader::from_local_root_and_pairs(
        &config_root,
        std::iter::empty::<(&str, &str)>(),
    )
    .unwrap();
    initial_config.apply_to_process_env();

    let live_store = Reloadable::new(seed_model_store(&initial_db_url, "gpt-4.1-initial").await);
    let live_portal_jwt = Reloadable::new("portal-secret-initial".to_owned());
    let supervision = start_standalone_runtime_supervision(
        StandaloneServiceKind::Portal,
        loader,
        initial_config,
        StandaloneServiceReloadHandles::portal(live_store.clone(), live_portal_jwt)
            .with_node_id("portal-node-rotating"),
    );

    let initial_store = live_store.snapshot();
    wait_for_service_runtime_node(initial_store.as_ref(), "portal-node-rotating").await;

    write_portal_runtime_config(&config_root, &rotated_db_url, "portal-secret-rotated");

    wait_for_models(&live_store, &["gpt-4.1-rotated"]).await;
    let rotated_store = live_store.snapshot();
    wait_for_service_runtime_node(rotated_store.as_ref(), "portal-node-rotating").await;

    drop(supervision);
    cleanup_dir(&config_root);
}

#[serial(runtime_config_env)]
#[tokio::test]
async fn standalone_runtime_supervision_reloads_secret_manager_after_config_file_change() {
    let config_root = temp_root("runtime-secret-manager-reload");
    let initial_secret_file = config_root.join("secrets-initial.json");
    let rotated_secret_file = config_root.join("secrets-rotated.json");
    write_gateway_secret_manager_runtime_config(
        &config_root,
        &initial_secret_file,
        "initial-master-key",
        &[],
    );

    let (loader, initial_config) = StandaloneConfigLoader::from_local_root_and_pairs(
        &config_root,
        std::iter::empty::<(&str, &str)>(),
    )
    .unwrap();
    initial_config.apply_to_process_env();

    let live_store = Reloadable::new(empty_store().await);
    let live_secret_manager =
        Reloadable::new(CredentialSecretManager::new_with_legacy_master_keys(
            sdkwork_api_secret_core::SecretBackendKind::LocalEncryptedFile,
            "initial-master-key",
            Vec::new(),
            &initial_secret_file,
            "sdkwork-api-server",
        ));
    persist_credential_with_secret_and_manager(
        live_store.snapshot().as_ref(),
        &live_secret_manager.snapshot(),
        "tenant-1",
        "provider-openai-official",
        "cred-openai",
        "sk-upstream-openai",
    )
    .await
    .unwrap();

    let supervision = start_standalone_runtime_supervision(
        StandaloneServiceKind::Gateway,
        loader,
        initial_config,
        StandaloneServiceReloadHandles::gateway(live_store.clone())
            .with_secret_manager(live_secret_manager.clone()),
    );

    write_gateway_secret_manager_runtime_config(
        &config_root,
        &rotated_secret_file,
        "rotated-master-key",
        &["initial-master-key"],
    );

    wait_for_secret_manager_master_key(&live_secret_manager, "rotated-master-key").await;
    let resolved = resolve_provider_secret_with_manager(
        live_store.snapshot().as_ref(),
        &live_secret_manager.snapshot(),
        "tenant-1",
        "provider-openai-official",
    )
    .await
    .unwrap();
    assert_eq!(resolved.as_deref(), Some("sk-upstream-openai"));

    drop(supervision);
    cleanup_dir(&config_root);
}

#[serial(extension_env)]
#[tokio::test]
async fn cluster_runtime_rollout_workers_complete_shared_rollout() {
    let store = empty_store().await;
    let live_store = Reloadable::new(store.clone());
    let now_ms = unix_timestamp_ms();
    store
        .upsert_service_runtime_node(&ServiceRuntimeNodeRecord::new(
            "gateway-node-a",
            "gateway",
            now_ms,
        ))
        .await
        .unwrap();
    store
        .upsert_service_runtime_node(&ServiceRuntimeNodeRecord::new(
            "admin-node-a",
            "admin",
            now_ms,
        ))
        .await
        .unwrap();

    let rollout = create_extension_runtime_rollout(
        store.as_ref(),
        "admin-user",
        sdkwork_api_app_gateway::ConfiguredExtensionHostReloadScope::All,
        30,
    )
    .await
    .unwrap();

    let gateway_worker = start_extension_runtime_rollout_supervision(
        StandaloneServiceKind::Gateway,
        "gateway-node-a",
        live_store.clone(),
    )
    .unwrap();
    let admin_worker = start_extension_runtime_rollout_supervision(
        StandaloneServiceKind::Admin,
        "admin-node-a",
        live_store,
    )
    .unwrap();

    wait_for_extension_runtime_rollout_status(store.as_ref(), &rollout.rollout_id, "succeeded")
        .await;

    let rollout = find_extension_runtime_rollout(store.as_ref(), &rollout.rollout_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(rollout.participant_count, 2);
    assert_eq!(rollout.participants.len(), 2);
    assert_eq!(rollout.participants[0].status, "succeeded");
    assert_eq!(rollout.participants[1].status, "succeeded");

    gateway_worker.abort();
    admin_worker.abort();
}

#[tokio::test]
async fn cluster_runtime_rollout_times_out_when_participant_stays_pending() {
    let store = empty_store().await;
    let now_ms = unix_timestamp_ms();
    store
        .insert_extension_runtime_rollout(&ExtensionRuntimeRolloutRecord::new(
            "rollout-timeout",
            "all",
            None,
            None,
            None,
            "admin-user",
            now_ms - 5_000,
            now_ms - 1,
        ))
        .await
        .unwrap();
    store
        .insert_extension_runtime_rollout_participant(
            &ExtensionRuntimeRolloutParticipantRecord::new(
                "rollout-timeout",
                "gateway-node-a",
                "gateway",
                "pending",
                now_ms - 5_000,
            ),
        )
        .await
        .unwrap();

    let rollout = find_extension_runtime_rollout(store.as_ref(), "rollout-timeout")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(rollout.status, "timed_out");
    assert_eq!(rollout.participant_count, 1);
}

#[serial(runtime_config_env)]
#[tokio::test]
async fn cluster_standalone_config_rollout_workers_apply_shared_reload() {
    let shared_store = empty_store().await;

    let portal_a_root = temp_root("cluster-config-rollout-portal-a");
    let portal_b_root = temp_root("cluster-config-rollout-portal-b");
    let portal_a_initial_db = sqlite_url_for_path(&portal_a_root.join("initial.db"));
    let portal_a_rotated_db = sqlite_url_for_path(&portal_a_root.join("rotated.db"));
    let portal_b_initial_db = sqlite_url_for_path(&portal_b_root.join("initial.db"));
    let portal_b_rotated_db = sqlite_url_for_path(&portal_b_root.join("rotated.db"));

    seed_model_store(&portal_a_initial_db, "portal-a-initial").await;
    seed_model_store(&portal_a_rotated_db, "portal-a-rotated").await;
    seed_model_store(&portal_b_initial_db, "portal-b-initial").await;
    seed_model_store(&portal_b_rotated_db, "portal-b-rotated").await;
    write_portal_runtime_config(
        &portal_a_root,
        &portal_a_initial_db,
        "portal-secret-a-initial",
    );
    write_portal_runtime_config(
        &portal_b_root,
        &portal_b_initial_db,
        "portal-secret-b-initial",
    );

    let (portal_a_loader, portal_a_initial_config) =
        StandaloneConfigLoader::from_local_root_and_pairs(
            &portal_a_root,
            std::iter::empty::<(&str, &str)>(),
        )
        .unwrap();
    let (portal_b_loader, portal_b_initial_config) =
        StandaloneConfigLoader::from_local_root_and_pairs(
            &portal_b_root,
            std::iter::empty::<(&str, &str)>(),
        )
        .unwrap();

    let portal_a_live_store =
        Reloadable::new(seed_model_store(&portal_a_initial_db, "portal-a-initial").await);
    let portal_b_live_store =
        Reloadable::new(seed_model_store(&portal_b_initial_db, "portal-b-initial").await);
    let portal_a_live_jwt = Reloadable::new("portal-secret-a-initial".to_owned());
    let portal_b_live_jwt = Reloadable::new("portal-secret-b-initial".to_owned());

    let portal_a_supervision = start_standalone_runtime_supervision(
        StandaloneServiceKind::Portal,
        portal_a_loader,
        portal_a_initial_config,
        StandaloneServiceReloadHandles::portal(
            portal_a_live_store.clone(),
            portal_a_live_jwt.clone(),
        )
        .with_coordination_store(shared_store.clone())
        .with_node_id("portal-node-a"),
    );
    let portal_b_supervision = start_standalone_runtime_supervision(
        StandaloneServiceKind::Portal,
        portal_b_loader,
        portal_b_initial_config,
        StandaloneServiceReloadHandles::portal(
            portal_b_live_store.clone(),
            portal_b_live_jwt.clone(),
        )
        .with_coordination_store(shared_store.clone())
        .with_node_id("portal-node-b"),
    );

    wait_for_service_runtime_node(shared_store.as_ref(), "portal-node-a").await;
    wait_for_service_runtime_node(shared_store.as_ref(), "portal-node-b").await;

    write_portal_runtime_config(
        &portal_a_root,
        &portal_a_rotated_db,
        "portal-secret-a-rotated",
    );
    write_portal_runtime_config(
        &portal_b_root,
        &portal_b_rotated_db,
        "portal-secret-b-rotated",
    );

    let rollout = create_standalone_config_rollout(
        shared_store.as_ref(),
        "admin-user",
        CreateStandaloneConfigRolloutRequest::new(Some("portal".to_owned()), 30),
    )
    .await
    .unwrap();

    wait_for_standalone_config_rollout_status(
        shared_store.as_ref(),
        &rollout.rollout_id,
        "succeeded",
    )
    .await;
    wait_for_models(&portal_a_live_store, &["portal-a-rotated"]).await;
    wait_for_models(&portal_b_live_store, &["portal-b-rotated"]).await;
    wait_for_reloadable_string(&portal_a_live_jwt, "portal-secret-a-rotated").await;
    wait_for_reloadable_string(&portal_b_live_jwt, "portal-secret-b-rotated").await;

    let rollout = find_standalone_config_rollout(shared_store.as_ref(), &rollout.rollout_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(rollout.participant_count, 2);
    assert_eq!(rollout.participants.len(), 2);
    assert_eq!(rollout.participants[0].status, "succeeded");
    assert_eq!(rollout.participants[1].status, "succeeded");

    drop(portal_a_supervision);
    drop(portal_b_supervision);
    cleanup_dir(&portal_a_root);
    cleanup_dir(&portal_b_root);
}

#[tokio::test]
async fn cluster_standalone_config_rollout_times_out_when_participant_stays_pending() {
    let store = empty_store().await;
    let now_ms = unix_timestamp_ms();
    store
        .insert_standalone_config_rollout(&StandaloneConfigRolloutRecord::new(
            "config-rollout-timeout",
            Some("portal".to_owned()),
            "admin-user",
            now_ms - 5_000,
            now_ms - 1,
        ))
        .await
        .unwrap();
    store
        .insert_standalone_config_rollout_participant(
            &StandaloneConfigRolloutParticipantRecord::new(
                "config-rollout-timeout",
                "portal-node-a",
                "portal",
                "pending",
                now_ms - 5_000,
            ),
        )
        .await
        .unwrap();

    let rollout = find_standalone_config_rollout(store.as_ref(), "config-rollout-timeout")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(rollout.status, "timed_out");
    assert_eq!(rollout.participant_count, 1);
}

fn write_runtime_config(root: &Path, enable_native_dynamic: bool, extension_root: &Path) {
    fs::write(
        root.join("config.yaml"),
        format!(
            r#"
extension_paths:
  - "{}"
enable_connector_extensions: false
enable_native_dynamic_extensions: {}
require_signed_native_dynamic_extensions: false
runtime_snapshot_interval_secs: 0
extension_hot_reload_interval_secs: 0
"#,
            config_path_value(extension_root),
            enable_native_dynamic,
        ),
    )
    .unwrap();
}

fn write_gateway_runtime_config(root: &Path, gateway_bind: &str) {
    fs::write(
        root.join("config.yaml"),
        format!(
            r#"
gateway_bind: "{gateway_bind}"
enable_connector_extensions: false
enable_native_dynamic_extensions: false
runtime_snapshot_interval_secs: 0
extension_hot_reload_interval_secs: 0
"#,
        ),
    )
    .unwrap();
}

fn write_gateway_secret_manager_runtime_config(
    root: &Path,
    secret_local_file: &Path,
    credential_master_key: &str,
    credential_legacy_master_keys: &[&str],
) {
    let legacy_keys = if credential_legacy_master_keys.is_empty() {
        "credential_legacy_master_keys: []".to_owned()
    } else {
        format!(
            "credential_legacy_master_keys:\n{}",
            credential_legacy_master_keys
                .iter()
                .map(|value| format!("  - \"{value}\""))
                .collect::<Vec<_>>()
                .join("\n")
        )
    };
    fs::write(
        root.join("config.yaml"),
        format!(
            r#"
secret_backend: "local_encrypted_file"
secret_local_file: "{}"
credential_master_key: "{credential_master_key}"
{legacy_keys}
enable_connector_extensions: false
enable_native_dynamic_extensions: false
runtime_snapshot_interval_secs: 0
extension_hot_reload_interval_secs: 0
"#,
            config_path_value(secret_local_file),
        ),
    )
    .unwrap();
}

fn temp_root(suffix: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    path.push(format!("sdkwork-app-runtime-{suffix}-{millis}"));
    fs::create_dir_all(&path).unwrap();
    path
}

fn cleanup_dir(path: &Path) {
    let _ = fs::remove_dir_all(path);
}

fn unix_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("unix time")
        .as_millis() as u64
}

async fn seed_model_store(database_url: &str, model_id: &str) -> Arc<dyn AdminStore> {
    if let Some(path) = sqlite_path_from_url(database_url) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        let _ = fs::File::create(path).unwrap();
    }
    let pool = run_migrations(database_url).await.unwrap();
    let store = SqliteAdminStore::new(pool);
    store
        .insert_model(&ModelCatalogEntry::new(
            model_id,
            "provider-openai-official",
        ))
        .await
        .unwrap();
    Arc::new(store)
}

async fn empty_store() -> Arc<dyn AdminStore> {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    Arc::new(SqliteAdminStore::new(pool))
}

fn health_router(label: &'static str) -> Router {
    Router::new().route("/health", get(move || async move { label }))
}

fn http_client() -> Client {
    Client::builder()
        .timeout(StdDuration::from_millis(200))
        .build()
        .unwrap()
}

fn available_bind() -> String {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let bind = listener.local_addr().unwrap().to_string();
    drop(listener);
    bind
}

async fn wait_for_health_response(bind: &str, expected: &str) {
    let client = http_client();
    let url = format!("http://{bind}/health");
    for _ in 0..240 {
        if let Ok(response) = client.get(&url).send().await {
            if response.status().is_success()
                && response.text().await.unwrap_or_default() == expected
            {
                return;
            }
        }
        sleep(Duration::from_millis(25)).await;
    }

    panic!("listener did not respond with expected health payload: {url}");
}

async fn wait_for_health_unreachable(bind: &str) {
    let client = http_client();
    let url = format!("http://{bind}/health");
    for _ in 0..240 {
        if client.get(&url).send().await.is_err() {
            return;
        }
        sleep(Duration::from_millis(25)).await;
    }

    panic!("listener remained reachable unexpectedly: {url}");
}

async fn wait_for_lifecycle_log(path: &Path, expected: &[&str]) {
    for _ in 0..160 {
        if fs::read_to_string(path)
            .ok()
            .map(|contents| contents.lines().map(str::to_owned).collect::<Vec<_>>())
            .is_some_and(|lines| {
                lines
                    == expected
                        .iter()
                        .map(|line| (*line).to_owned())
                        .collect::<Vec<_>>()
            })
        {
            return;
        }
        sleep(Duration::from_millis(25)).await;
    }

    panic!(
        "lifecycle log did not reach expected state: {}",
        path.display()
    );
}

async fn wait_for_models(live_store: &Reloadable<Arc<dyn AdminStore>>, expected: &[&str]) {
    for _ in 0..200 {
        let current = live_store
            .snapshot()
            .list_models()
            .await
            .unwrap()
            .into_iter()
            .map(|entry| entry.external_name)
            .collect::<Vec<_>>();
        if current
            == expected
                .iter()
                .map(|value| (*value).to_owned())
                .collect::<Vec<_>>()
        {
            return;
        }
        sleep(Duration::from_millis(25)).await;
    }

    panic!("live store did not reach expected models");
}

async fn wait_for_reloadable_string(live_value: &Reloadable<String>, expected: &str) {
    for _ in 0..200 {
        if live_value.snapshot() == expected {
            return;
        }
        sleep(Duration::from_millis(25)).await;
    }

    panic!("reloadable string did not reach expected value");
}

async fn wait_for_secret_manager_master_key(
    live_secret_manager: &Reloadable<CredentialSecretManager>,
    expected_master_key: &str,
) {
    for _ in 0..200 {
        if live_secret_manager.snapshot().current_master_key_id()
            == sdkwork_api_secret_core::master_key_id(expected_master_key)
        {
            return;
        }
        sleep(Duration::from_millis(25)).await;
    }

    panic!("live secret manager did not reach expected master key");
}

async fn wait_for_extension_runtime_rollout_status(
    store: &dyn AdminStore,
    rollout_id: &str,
    expected_status: &str,
) {
    for _ in 0..200 {
        let Some(rollout) = find_extension_runtime_rollout(store, rollout_id)
            .await
            .unwrap()
        else {
            sleep(Duration::from_millis(25)).await;
            continue;
        };

        if rollout.status == expected_status {
            return;
        }
        sleep(Duration::from_millis(25)).await;
    }

    panic!("extension runtime rollout did not reach expected status");
}

async fn wait_for_standalone_config_rollout_status(
    store: &dyn AdminStore,
    rollout_id: &str,
    expected_status: &str,
) {
    for _ in 0..200 {
        let Some(rollout) = find_standalone_config_rollout(store, rollout_id)
            .await
            .unwrap()
        else {
            sleep(Duration::from_millis(25)).await;
            continue;
        };

        if rollout.status == expected_status {
            return;
        }
        sleep(Duration::from_millis(25)).await;
    }

    panic!("standalone config rollout did not reach expected status");
}

async fn wait_for_service_runtime_node(store: &dyn AdminStore, node_id: &str) {
    for _ in 0..200 {
        if store
            .list_service_runtime_nodes()
            .await
            .unwrap()
            .into_iter()
            .any(|node| node.node_id == node_id)
        {
            return;
        }

        sleep(Duration::from_millis(25)).await;
    }

    panic!("service runtime node did not heartbeat into the shared store: {node_id}");
}

struct NativeDynamicLifecycleLogGuard {
    path: PathBuf,
    previous: Option<String>,
}

impl NativeDynamicLifecycleLogGuard {
    fn new() -> Self {
        let mut path = std::env::temp_dir();
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("unix time")
            .as_millis();
        path.push(format!(
            "sdkwork-app-runtime-native-dynamic-lifecycle-{millis}.log"
        ));

        let previous = std::env::var("SDKWORK_NATIVE_MOCK_LIFECYCLE_LOG").ok();
        std::env::set_var("SDKWORK_NATIVE_MOCK_LIFECYCLE_LOG", &path);

        Self { path, previous }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for NativeDynamicLifecycleLogGuard {
    fn drop(&mut self) {
        match self.previous.as_deref() {
            Some(value) => std::env::set_var("SDKWORK_NATIVE_MOCK_LIFECYCLE_LOG", value),
            None => std::env::remove_var("SDKWORK_NATIVE_MOCK_LIFECYCLE_LOG"),
        }
        let _ = std::fs::remove_file(&self.path);
    }
}

fn native_dynamic_fixture_library_path() -> PathBuf {
    let current_exe = std::env::current_exe().expect("current exe");
    let directory = current_exe.parent().expect("exe dir");
    let prefix = if cfg!(windows) {
        "sdkwork_api_ext_provider_native_mock"
    } else {
        "libsdkwork_api_ext_provider_native_mock"
    };
    let extension = if cfg!(windows) {
        "dll"
    } else if cfg!(target_os = "macos") {
        "dylib"
    } else {
        "so"
    };

    std::fs::read_dir(directory)
        .expect("deps dir")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .find(|path| {
            path.extension().and_then(|value| value.to_str()) == Some(extension)
                && path
                    .file_stem()
                    .and_then(|value| value.to_str())
                    .is_some_and(|stem| stem.starts_with(prefix))
        })
        .expect("native dynamic fixture library")
}

fn write_portal_runtime_config(root: &Path, database_url: &str, jwt_secret: &str) {
    fs::write(
        root.join("config.yaml"),
        format!(
            r#"
database_url: "{database_url}"
portal_jwt_signing_secret: "{jwt_secret}"
runtime_snapshot_interval_secs: 0
extension_hot_reload_interval_secs: 0
"#,
        ),
    )
    .unwrap();
}

fn sqlite_url_for_path(path: &Path) -> String {
    let normalized = path.to_string_lossy().replace('\\', "/");
    if normalized.starts_with('/') {
        format!("sqlite://{normalized}")
    } else {
        format!("sqlite:///{normalized}")
    }
}

fn sqlite_path_from_url(url: &str) -> Option<PathBuf> {
    let raw_path = url.strip_prefix("sqlite://")?;
    let normalized_path = raw_path
        .strip_prefix('/')
        .filter(|candidate| has_windows_drive_prefix(candidate))
        .unwrap_or(raw_path);

    Some(PathBuf::from(normalized_path))
}

fn config_path_value(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn has_windows_drive_prefix(path: &str) -> bool {
    let bytes = path.as_bytes();
    bytes.len() >= 3
        && bytes[0].is_ascii_alphabetic()
        && bytes[1] == b':'
        && (bytes[2] == b'/' || bytes[2] == b'\\')
}

fn native_dynamic_manifest(library_path: &Path) -> ExtensionManifest {
    ExtensionManifest::new(
        FIXTURE_EXTENSION_ID,
        ExtensionKind::Provider,
        "0.1.0",
        ExtensionRuntime::NativeDynamic,
    )
    .with_display_name("Native Mock")
    .with_protocol(ExtensionProtocol::OpenAi)
    .with_entrypoint(library_path.to_string_lossy())
    .with_channel_binding("sdkwork.channel.openai")
    .with_permission(ExtensionPermission::NetworkOutbound)
    .with_capability(sdkwork_api_extension_core::CapabilityDescriptor::new(
        "chat.completions.create",
        CompatibilityLevel::Native,
    ))
    .with_capability(sdkwork_api_extension_core::CapabilityDescriptor::new(
        "chat.completions.stream",
        CompatibilityLevel::Native,
    ))
    .with_capability(sdkwork_api_extension_core::CapabilityDescriptor::new(
        "responses.create",
        CompatibilityLevel::Native,
    ))
    .with_capability(sdkwork_api_extension_core::CapabilityDescriptor::new(
        "responses.stream",
        CompatibilityLevel::Native,
    ))
    .with_capability(sdkwork_api_extension_core::CapabilityDescriptor::new(
        "audio.speech.create",
        CompatibilityLevel::Native,
    ))
    .with_capability(sdkwork_api_extension_core::CapabilityDescriptor::new(
        "files.content",
        CompatibilityLevel::Native,
    ))
    .with_capability(sdkwork_api_extension_core::CapabilityDescriptor::new(
        "videos.content",
        CompatibilityLevel::Native,
    ))
}
