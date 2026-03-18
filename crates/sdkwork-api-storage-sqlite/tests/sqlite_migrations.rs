use sdkwork_api_storage_sqlite::run_migrations;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static TEMP_DIR_COUNTER: AtomicU64 = AtomicU64::new(0);

#[tokio::test]
async fn creates_identity_and_tenant_tables() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let row: (String,) =
        sqlx::query_as("select name from sqlite_master where name = 'identity_users'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(row.0, "identity_users");

    let columns: Vec<(String,)> =
        sqlx::query_as("select name from pragma_table_info('identity_users') order by cid")
            .fetch_all(&pool)
            .await
            .unwrap();
    assert!(columns.iter().any(|(name,)| name == "display_name"));
    assert!(columns.iter().any(|(name,)| name == "workspace_tenant_id"));
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
