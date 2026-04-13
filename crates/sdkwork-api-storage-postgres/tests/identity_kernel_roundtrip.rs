use sdkwork_api_domain_identity::{
    CanonicalApiKeyRecord, IdentityBindingRecord, IdentityUserRecord,
};
use sdkwork_api_storage_core::IdentityKernelStore;
use sdkwork_api_storage_postgres::{run_migrations, PostgresAdminStore};

#[tokio::test]
async fn postgres_store_round_trips_canonical_identity_kernel_records_when_url_is_provided() {
    let Some(database_url) = std::env::var("SDKWORK_TEST_POSTGRES_URL").ok() else {
        return;
    };

    let pool = run_migrations(&database_url).await.unwrap();
    let store = PostgresAdminStore::new(pool);

    let user = IdentityUserRecord::new(9001, 1001, 2002)
        .with_external_user_ref(Some("portal-user-1".to_owned()))
        .with_display_name(Some("Portal User".to_owned()))
        .with_email(Some("portal@example.com".to_owned()))
        .with_created_at_ms(10)
        .with_updated_at_ms(20);
    let api_key = CanonicalApiKeyRecord::new(778899, 1001, 2002, 9001, "key_hash_live")
        .with_key_prefix("skw_live")
        .with_display_name("Production key")
        .with_created_at_ms(30)
        .with_updated_at_ms(40);
    let binding = IdentityBindingRecord::new(7001, 1001, 2002, 9001, "jwt_subject")
        .with_issuer(Some("plus-auth".to_owned()))
        .with_subject(Some("user-9001".to_owned()))
        .with_platform(Some("web".to_owned()))
        .with_created_at_ms(50)
        .with_updated_at_ms(60);

    store.insert_identity_user_record(&user).await.unwrap();
    store
        .insert_canonical_api_key_record(&api_key)
        .await
        .unwrap();
    store
        .insert_identity_binding_record(&binding)
        .await
        .unwrap();

    assert_eq!(
        store.list_identity_user_records().await.unwrap(),
        vec![user]
    );
    assert_eq!(
        store
            .find_canonical_api_key_record_by_hash("key_hash_live")
            .await
            .unwrap(),
        Some(api_key)
    );
    assert_eq!(
        store
            .find_identity_binding_record("jwt_subject", Some("plus-auth"), Some("user-9001"))
            .await
            .unwrap(),
        Some(binding)
    );
}
