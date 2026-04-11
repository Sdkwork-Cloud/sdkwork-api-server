use sdkwork_api_domain_catalog::{Channel, ProviderAccountRecord, ProxyProvider};
use sdkwork_api_storage_core::AdminStore;
use sdkwork_api_storage_sqlite::{run_migrations, SqliteAdminStore};

#[tokio::test]
async fn sqlite_store_round_trips_provider_accounts() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    store
        .insert_channel(&Channel::new("openai", "OpenAI"))
        .await
        .unwrap();
    store
        .insert_provider(&ProxyProvider::new(
            "provider-openai-official",
            "openai",
            "openai",
            "https://api.openai.com/v1",
            "OpenAI Official",
        ))
        .await
        .unwrap();

    let record = ProviderAccountRecord::new(
        "acct-openai-primary",
        "provider-openai-official",
        "OpenAI Primary",
        "api_key",
        "instance-openai-primary",
    )
    .with_region("us-east")
    .with_priority(100)
    .with_weight(10)
    .with_enabled(true)
    .with_base_url_override("https://api.openai.com/v1");

    store.upsert_provider_account(&record).await.unwrap();

    let listed = store.list_provider_accounts().await.unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].provider_account_id, "acct-openai-primary");
    assert_eq!(listed[0].provider_id, "provider-openai-official");
    assert_eq!(listed[0].execution_instance_id, "instance-openai-primary");
    assert_eq!(listed[0].priority, 100);
    assert_eq!(listed[0].weight, 10);
    assert_eq!(listed[0].region.as_deref(), Some("us-east"));

    let found = store
        .find_provider_account("acct-openai-primary")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(found.display_name, "OpenAI Primary");

    let deleted = store
        .delete_provider_account("acct-openai-primary")
        .await
        .unwrap();
    assert!(deleted);
    assert!(store
        .find_provider_account("acct-openai-primary")
        .await
        .unwrap()
        .is_none());
}
