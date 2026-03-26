use sdkwork_api_storage_sqlite::run_migrations;
use sqlx::SqlitePool;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static TEMP_DIR_COUNTER: AtomicU64 = AtomicU64::new(0);

#[tokio::test]
async fn creates_canonical_ai_tables_with_only_ai_prefixed_physical_tables() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let tables: Vec<(String,)> = sqlx::query_as(
        "select name
         from sqlite_master
         where type = 'table' and name not like 'sqlite_%'
         order by name",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert!(!tables.is_empty());
    assert!(tables.iter().all(|(name,)| name.starts_with("ai_")));

    for table_name in [
        "ai_portal_users",
        "ai_admin_users",
        "ai_tenants",
        "ai_projects",
        "ai_coupon_campaigns",
        "ai_channel",
        "ai_proxy_provider",
        "ai_proxy_provider_channel",
        "ai_router_credential_records",
        "ai_model",
        "ai_model_price",
        "ai_app_api_keys",
    ] {
        let row: (String,) =
            sqlx::query_as("select name from sqlite_master where type = 'table' and name = ?")
                .bind(table_name)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(row.0, table_name);
    }

    for legacy_name in legacy_compatibility_names() {
        let row: (String, String) =
            sqlx::query_as("select name, type from sqlite_master where name = ?")
                .bind(legacy_name)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(row.0, legacy_name);
        assert_eq!(row.1, "view");
    }
}

#[tokio::test]
async fn creates_parent_directories_for_file_backed_sqlite_urls() {
    let root = temp_sqlite_root("auto-parent-dirs");
    let database_path = root.join("nested").join("sdkwork-api-server.db");
    let database_url = sqlite_url_for(&database_path);

    assert!(!database_path.parent().unwrap().exists());

    let pool = run_migrations(&database_url).await.unwrap();
    pool.close().await;

    assert!(database_path.parent().unwrap().is_dir());
    assert!(database_path.is_file());

    fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn migrates_legacy_tables_into_canonical_ai_tables_and_replaces_old_names_with_views() {
    let root = temp_sqlite_root("legacy-to-canonical");
    let database_path = root.join("legacy").join("sdkwork-api-server.db");
    let database_url = sqlite_url_for(&database_path);

    seed_legacy_schema(&database_url).await;

    let pool = run_migrations(&database_url).await.unwrap();

    for legacy_name in legacy_compatibility_names() {
        let row: (String,) = sqlx::query_as("select type from sqlite_master where name = ?")
            .bind(legacy_name)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(
            row.0, "view",
            "{legacy_name} should be a compatibility view"
        );
    }

    let portal_users: Vec<(String, String)> = sqlx::query_as(
        "select id, workspace_tenant_id from ai_portal_users where id = 'portal-user-1'",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(
        portal_users,
        vec![("portal-user-1".to_owned(), "tenant-1".to_owned())]
    );

    let channels: Vec<(String, String)> = sqlx::query_as(
        "select channel_id, channel_name from ai_channel where channel_id = 'legacy-openai'",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(
        channels,
        vec![("legacy-openai".to_owned(), "Legacy OpenAI".to_owned())]
    );

    let providers: Vec<(String, String)> = sqlx::query_as(
        "select proxy_provider_id, primary_channel_id
         from ai_proxy_provider
         where proxy_provider_id = 'provider-legacy-openai'",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(
        providers,
        vec![(
            "provider-legacy-openai".to_owned(),
            "legacy-openai".to_owned(),
        )]
    );

    let channel_bindings: Vec<(String, String, i64)> = sqlx::query_as(
        "select proxy_provider_id, channel_id, is_primary
         from ai_proxy_provider_channel
         where proxy_provider_id = 'provider-legacy-openai'",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(
        channel_bindings,
        vec![(
            "provider-legacy-openai".to_owned(),
            "legacy-openai".to_owned(),
            1,
        )]
    );

    let credentials: Vec<(String, String, String)> = sqlx::query_as(
        "select tenant_id, proxy_provider_id, key_reference
         from ai_router_credential_records
         where tenant_id = 'tenant-1'",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(
        credentials,
        vec![(
            "tenant-1".to_owned(),
            "provider-legacy-openai".to_owned(),
            "legacy-key".to_owned(),
        )]
    );

    let models: Vec<(String, String, String)> = sqlx::query_as(
        "select channel_id, model_id, model_display_name
         from ai_model
         where model_id = 'gpt-legacy'",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(
        models,
        vec![(
            "legacy-openai".to_owned(),
            "gpt-legacy".to_owned(),
            "gpt-legacy".to_owned(),
        )]
    );

    let prices: Vec<(String, String, String, String)> = sqlx::query_as(
        "select channel_id, model_id, proxy_provider_id, currency_code
         from ai_model_price
         where model_id = 'gpt-legacy'",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(
        prices,
        vec![(
            "legacy-openai".to_owned(),
            "gpt-legacy".to_owned(),
            "provider-legacy-openai".to_owned(),
            "USD".to_owned(),
        )]
    );

    let app_keys: Vec<(String, Option<String>, String)> = sqlx::query_as(
        "select hashed_key, raw_key, tenant_id
         from ai_app_api_keys
         where hashed_key = 'hashed-legacy-key'",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(
        app_keys,
        vec![("hashed-legacy-key".to_owned(), None, "tenant-1".to_owned(),)]
    );

    let legacy_channel_rows: Vec<(String, String)> =
        sqlx::query_as("select id, name from catalog_channels where id = 'legacy-openai'")
            .fetch_all(&pool)
            .await
            .unwrap();
    assert_eq!(
        legacy_channel_rows,
        vec![("legacy-openai".to_owned(), "Legacy OpenAI".to_owned())]
    );

    pool.close().await;
    fs::remove_dir_all(root).unwrap();
}

fn temp_sqlite_root(label: &str) -> PathBuf {
    let unique = TEMP_DIR_COUNTER.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!("sdkwork-sqlite-tests-{label}-{unique}"));
    if root.exists() {
        fs::remove_dir_all(&root).unwrap();
    }
    root
}

fn sqlite_url_for(path: &PathBuf) -> String {
    let normalized = path.to_string_lossy().replace('\\', "/");
    if normalized.starts_with('/') {
        format!("sqlite://{normalized}")
    } else {
        format!("sqlite:///{normalized}")
    }
}

async fn seed_legacy_schema(database_url: &str) {
    let path = database_url
        .trim_start_matches("sqlite:///")
        .trim_start_matches("sqlite://");
    let database_path = PathBuf::from(path);
    if let Some(parent) = database_path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    if !database_path.exists() {
        let _ = fs::File::create(&database_path).unwrap();
    }
    let pool = SqlitePool::connect(database_url).await.unwrap();

    sqlx::query(
        "create table identity_users (
            id text primary key not null,
            email text not null,
            display_name text not null default '',
            password_salt text not null default '',
            password_hash text not null default '',
            workspace_tenant_id text not null default '',
            workspace_project_id text not null default '',
            active integer not null default 1,
            created_at_ms integer not null default 0
        )",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "insert into identity_users (
            id, email, display_name, password_salt, password_hash,
            workspace_tenant_id, workspace_project_id, active, created_at_ms
        ) values ('portal-user-1', 'portal@example.com', 'Portal User', 'salt', 'hash', 'tenant-1', 'project-1', 1, 1000)",
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query("create table catalog_channels (id text primary key not null, name text not null)")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query(
        "insert into catalog_channels (id, name) values ('legacy-openai', 'Legacy OpenAI')",
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "create table catalog_proxy_providers (
            id text primary key not null,
            channel_id text not null,
            extension_id text not null default '',
            adapter_kind text not null default 'openai',
            base_url text not null default 'http://localhost',
            display_name text not null
        )",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "insert into catalog_proxy_providers (
            id, channel_id, extension_id, adapter_kind, base_url, display_name
        ) values (
            'provider-legacy-openai',
            'legacy-openai',
            'sdkwork.provider.openai.legacy',
            'openai',
            'https://legacy.example.com',
            'Legacy OpenAI Provider'
        )",
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "create table catalog_provider_channel_bindings (
            provider_id text not null,
            channel_id text not null,
            is_primary integer not null default 0,
            primary key (provider_id, channel_id)
        )",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "insert into catalog_provider_channel_bindings (provider_id, channel_id, is_primary)
         values ('provider-legacy-openai', 'legacy-openai', 1)",
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "create table credential_records (
            tenant_id text not null,
            provider_id text not null,
            key_reference text not null,
            secret_backend text not null default 'database_encrypted',
            secret_local_file text,
            secret_keyring_service text,
            secret_master_key_id text,
            secret_ciphertext text,
            secret_key_version integer,
            primary key (tenant_id, provider_id, key_reference)
        )",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "insert into credential_records (
            tenant_id, provider_id, key_reference, secret_backend, secret_master_key_id, secret_ciphertext, secret_key_version
        ) values (
            'tenant-1',
            'provider-legacy-openai',
            'legacy-key',
            'database_encrypted',
            'master-key-1',
            'ciphertext-blob',
            7
        )",
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "create table catalog_models (
            external_name text not null,
            provider_id text not null,
            capabilities text not null default '[]',
            streaming integer not null default 0,
            context_window integer,
            primary key (external_name, provider_id)
        )",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "insert into catalog_models (
            external_name, provider_id, capabilities, streaming, context_window
        ) values (
            'gpt-legacy',
            'provider-legacy-openai',
            '[\"responses\"]',
            1,
            64000
        )",
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "create table identity_gateway_api_keys (
            hashed_key text primary key not null,
            tenant_id text not null,
            project_id text not null,
            environment text not null,
            label text not null default '',
            notes text,
            created_at_ms integer not null default 0,
            last_used_at_ms integer,
            expires_at_ms integer,
            active integer not null
        )",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "insert into identity_gateway_api_keys (
            hashed_key, tenant_id, project_id, environment, label, notes, created_at_ms, last_used_at_ms, expires_at_ms, active
        ) values (
            'hashed-legacy-key',
            'tenant-1',
            'project-1',
            'prod',
            'Legacy Key',
            'legacy migration',
            1000,
            2000,
            3000,
            1
        )",
    )
    .execute(&pool)
    .await
    .unwrap();

    pool.close().await;
}

fn legacy_compatibility_names() -> [&'static str; 26] {
    [
        "identity_users",
        "admin_users",
        "tenant_records",
        "tenant_projects",
        "coupon_campaigns",
        "catalog_channels",
        "catalog_proxy_providers",
        "catalog_provider_channel_bindings",
        "credential_records",
        "catalog_models",
        "routing_policies",
        "routing_policy_providers",
        "project_routing_preferences",
        "routing_decision_logs",
        "routing_provider_health",
        "usage_records",
        "billing_ledger_entries",
        "billing_quota_policies",
        "identity_gateway_api_keys",
        "extension_installations",
        "extension_instances",
        "service_runtime_nodes",
        "extension_runtime_rollouts",
        "extension_runtime_rollout_participants",
        "standalone_config_rollouts",
        "standalone_config_rollout_participants",
    ]
}
