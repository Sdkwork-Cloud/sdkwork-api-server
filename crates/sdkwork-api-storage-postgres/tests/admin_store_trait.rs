use sdkwork_api_storage_core::{AdminStore, StorageDialect};
use sdkwork_api_storage_postgres::PostgresAdminStore;

#[tokio::test]
async fn postgres_store_implements_admin_store_trait() {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .connect_lazy("postgres://sdkwork:secret@localhost/sdkwork")
        .unwrap();
    let store = PostgresAdminStore::new(pool);
    let trait_store: &dyn AdminStore = &store;

    assert_eq!(trait_store.dialect(), StorageDialect::Postgres);
}
