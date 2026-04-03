use sdkwork_api_domain_billing::{AccountRecord, AccountType};
use sdkwork_api_storage_core::AccountKernelStore;
use sdkwork_api_storage_sqlite::{run_migrations, SqliteAdminStore};

#[tokio::test]
async fn sqlite_store_persists_canonical_account_kernel_records_instead_of_exposing_only_a_seam() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    let account = store
        .insert_account_record(&AccountRecord::new(
            7001,
            1001,
            2002,
            9001,
            AccountType::Primary,
        ))
        .await
        .unwrap();
    assert_eq!(account.account_id, 7001);
    assert_eq!(account.tenant_id, 1001);

    let accounts = store.list_account_records().await.unwrap();
    assert_eq!(accounts, vec![account]);

    let request_facts = store.list_request_meter_facts().await.unwrap();
    assert!(request_facts.is_empty());
}

#[tokio::test]
async fn sqlite_store_finds_account_by_canonical_owner_scope() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    let primary_account = AccountRecord::new(7201, 1001, 2002, 9001, AccountType::Primary);
    let grant_account = AccountRecord::new(7202, 1001, 2002, 9001, AccountType::Grant);
    let other_user_account = AccountRecord::new(7203, 1001, 2002, 9002, AccountType::Primary);

    store.insert_account_record(&primary_account).await.unwrap();
    store.insert_account_record(&grant_account).await.unwrap();
    store
        .insert_account_record(&other_user_account)
        .await
        .unwrap();

    let resolved = store
        .find_account_record_by_owner(1001, 2002, 9001, AccountType::Primary)
        .await
        .unwrap();

    assert_eq!(resolved, Some(primary_account));
}
