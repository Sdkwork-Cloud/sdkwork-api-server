use super::*;

const STANDALONE_SUPPORTED_STORAGE_DIALECTS: [StorageDialect; 2] =
    [StorageDialect::Sqlite, StorageDialect::Postgres];

fn supported_storage_dialects_summary() -> String {
    STANDALONE_SUPPORTED_STORAGE_DIALECTS
        .iter()
        .map(|dialect| dialect.as_str())
        .collect::<Vec<_>>()
        .join(", ")
}

fn standalone_cache_driver_registry() -> CacheDriverRegistry {
    CacheDriverRegistry::new()
        .with_factory(MemoryCacheStoreFactory)
        .with_factory(RedisCacheStoreFactory)
}

pub async fn build_admin_store_from_config(
    config: &StandaloneConfig,
) -> Result<Arc<dyn AdminStore>> {
    let (store, _) = build_admin_store_and_commercial_billing_from_config(config).await?;
    Ok(store)
}

pub async fn build_admin_store_and_commercial_billing_from_config(
    config: &StandaloneConfig,
) -> Result<(Arc<dyn AdminStore>, Arc<dyn CommercialBillingAdminKernel>)> {
    let supported_dialects = supported_storage_dialects_summary();
    let Some(dialect) = config.storage_dialect() else {
        anyhow::bail!(
            "standalone runtime supervision received unsupported database URL scheme for {} (supported dialects: {})",
            config.database_url,
            supported_dialects
        );
    };

    match dialect {
        StorageDialect::Sqlite => {
            let pool = run_sqlite_migrations(&config.database_url).await?;
            let store = Arc::new(SqliteAdminStore::new(pool));
            let admin_store: Arc<dyn AdminStore> = store.clone();
            let commercial_billing: Arc<dyn CommercialBillingAdminKernel> = store;
            Ok::<
                (Arc<dyn AdminStore>, Arc<dyn CommercialBillingAdminKernel>),
                anyhow::Error,
            >((admin_store, commercial_billing))
        }
        StorageDialect::Postgres => {
            let pool = run_postgres_migrations(&config.database_url).await?;
            let store = Arc::new(PostgresAdminStore::new(pool));
            let admin_store: Arc<dyn AdminStore> = store.clone();
            let commercial_billing: Arc<dyn CommercialBillingAdminKernel> = store;
            Ok::<
                (Arc<dyn AdminStore>, Arc<dyn CommercialBillingAdminKernel>),
                anyhow::Error,
            >((admin_store, commercial_billing))
        }
        other => anyhow::bail!(
            "standalone runtime supervision does not yet support storage dialect: {} (supported dialects: {})",
            other.as_str(),
            supported_dialects
        ),
    }
    .with_context(|| {
        format!(
            "failed to initialize standalone admin store with database {}",
            config.database_url
        )
    })
}

pub async fn build_cache_runtime_from_config(
    config: &StandaloneConfig,
) -> Result<CacheRuntimeStores> {
    let registry = standalone_cache_driver_registry();
    let Some(driver) = registry.resolve(config.cache_backend) else {
        anyhow::bail!(
            "standalone runtime does not yet support cache backend: {}",
            config.cache_backend.as_str()
        );
    };

    driver
        .build(config.cache_url.as_deref())
        .await
        .with_context(|| {
            format!(
                "failed to initialize standalone cache runtime with driver {}",
                driver.driver_name()
            )
        })
}

pub(crate) fn build_secret_manager_from_config(
    config: &StandaloneConfig,
) -> CredentialSecretManager {
    CredentialSecretManager::new_with_legacy_master_keys(
        config.secret_backend,
        config.credential_master_key.clone(),
        config.credential_legacy_master_keys.clone(),
        config.secret_local_file.clone(),
        config.secret_keyring_service.clone(),
    )
}

pub(crate) async fn validate_secret_manager_for_store(
    store: &dyn AdminStore,
    manager: &CredentialSecretManager,
) -> Result<()> {
    let credentials = store.list_credentials().await?;
    stream::iter(credentials.into_iter().map(|credential| async move {
        let tenant_id = credential.tenant_id.clone();
        let provider_id = credential.provider_id.clone();
        let key_reference = credential.key_reference.clone();

        resolve_credential_secret_with_manager(
            store,
            manager,
            &tenant_id,
            &provider_id,
            &key_reference,
        )
        .await
        .with_context(|| {
            format!(
                "credential validation failed for tenant={} provider={} key_reference={}",
                tenant_id, provider_id, key_reference
            )
        })
    }))
    .buffer_unordered(8)
    .try_for_each(|_| async { Ok(()) })
    .await
}
