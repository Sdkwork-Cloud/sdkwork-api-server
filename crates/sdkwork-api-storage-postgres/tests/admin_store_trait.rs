use sdkwork_api_storage_core::{
    AccountKernelCommandBatch, AdminStore, BillingStore, CatalogStore, CredentialStore,
    ExtensionStore, IdentityStore, RoutingStore, StorageDialect, TenantStore, UsageStore,
};
use sdkwork_api_storage_postgres::PostgresAdminStore;

#[tokio::test]
async fn postgres_store_implements_admin_store_trait() {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .connect_lazy("postgres://sdkwork:secret@localhost/sdkwork")
        .unwrap();
    let store = PostgresAdminStore::new(pool);
    let trait_store: &dyn AdminStore = &store;
    let _identity_store: &dyn IdentityStore = &store;
    let _tenant_store: &dyn TenantStore = &store;
    let _catalog_store: &dyn CatalogStore = &store;
    let _credential_store: &dyn CredentialStore = &store;
    let _routing_store: &dyn RoutingStore = &store;
    let _usage_store: &dyn UsageStore = &store;
    let _billing_store: &dyn BillingStore = &store;
    let _extension_store: &dyn ExtensionStore = &store;

    assert_eq!(trait_store.dialect(), StorageDialect::Postgres);
}

#[tokio::test]
async fn postgres_store_exposes_canonical_kernels() {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .connect_lazy("postgres://sdkwork:secret@localhost/sdkwork")
        .unwrap();
    let store = PostgresAdminStore::new(pool);
    let trait_store: &dyn AdminStore = &store;

    assert!(
        trait_store.identity_kernel().is_some(),
        "postgres admin store must expose identity kernel support",
    );
    assert!(
        trait_store.account_kernel().is_some(),
        "postgres admin store must expose account kernel support",
    );
}

#[tokio::test]
async fn postgres_account_kernel_accepts_empty_batch_without_unsupported_error() {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .connect_lazy("postgres://sdkwork:secret@localhost/sdkwork")
        .unwrap();
    let store = PostgresAdminStore::new(pool);
    let trait_store: &dyn AdminStore = &store;
    let account_kernel = trait_store
        .account_kernel()
        .expect("postgres admin store should expose account kernel");

    account_kernel
        .commit_account_kernel_batch(&AccountKernelCommandBatch::default())
        .await
        .expect("empty account kernel batch should be a supported no-op");
}
