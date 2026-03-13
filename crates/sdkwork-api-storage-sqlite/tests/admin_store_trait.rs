use sdkwork_api_storage_core::{AdminStore, StorageDialect};
use sdkwork_api_storage_sqlite::{run_migrations, SqliteAdminStore};

#[tokio::test]
async fn sqlite_store_implements_admin_store_trait() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);
    let trait_store: &dyn AdminStore = &store;

    assert_eq!(trait_store.dialect(), StorageDialect::Sqlite);
    assert!(trait_store.list_channels().await.unwrap().is_empty());
}
