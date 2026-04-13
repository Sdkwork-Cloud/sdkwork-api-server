use super::*;
use crate::bootstrap_data::bootstrap_repository_data_from_config;
use sdkwork_api_storage_core::{AccountKernelStore, CommercialKernelStore, IdentityKernelStore};

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
    Ok(build_admin_payment_store_handles_from_config(config)
        .await?
        .admin_store)
}

pub async fn build_admin_store_and_commercial_billing_from_config(
    config: &StandaloneConfig,
) -> Result<(Arc<dyn AdminStore>, Arc<dyn CommercialBillingAdminKernel>)> {
    let handles = build_admin_payment_store_handles_from_config(config).await?;
    Ok((handles.admin_store, handles.commercial_billing))
}

pub struct StandaloneAdminPaymentStoreHandles {
    pub admin_store: Arc<dyn AdminStore>,
    pub commercial_billing: Arc<dyn CommercialBillingAdminKernel>,
    pub payment_store: Arc<dyn CommercialKernelStore>,
    pub identity_store: Arc<dyn IdentityKernelStore>,
}

pub async fn build_admin_payment_store_handles_from_config(
    config: &StandaloneConfig,
) -> Result<StandaloneAdminPaymentStoreHandles> {
    let secret_manager = build_secret_manager_from_config(config);
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
            let account_kernel: Arc<dyn AccountKernelStore> = store.clone();
            let commercial_billing: Arc<dyn CommercialBillingAdminKernel> = store.clone();
            let payment_store: Arc<dyn CommercialKernelStore> = store.clone();
            let identity_store: Arc<dyn IdentityKernelStore> = store;
            bootstrap_repository_data_from_config(
                admin_store.as_ref(),
                account_kernel.as_ref(),
                commercial_billing.as_ref(),
                config,
            )
            .await?;
            bootstrap_official_provider_access(admin_store.as_ref(), &secret_manager, config)
                .await?;
            Ok::<StandaloneAdminPaymentStoreHandles, anyhow::Error>(StandaloneAdminPaymentStoreHandles {
                admin_store,
                commercial_billing,
                payment_store,
                identity_store,
            })
        }
        StorageDialect::Postgres => {
            let pool = run_postgres_migrations(&config.database_url).await?;
            let store = Arc::new(PostgresAdminStore::new(pool));
            let admin_store: Arc<dyn AdminStore> = store.clone();
            let account_kernel: Arc<dyn AccountKernelStore> = store.clone();
            let commercial_billing: Arc<dyn CommercialBillingAdminKernel> = store.clone();
            let payment_store: Arc<dyn CommercialKernelStore> = store.clone();
            let identity_store: Arc<dyn IdentityKernelStore> = store;
            bootstrap_repository_data_from_config(
                admin_store.as_ref(),
                account_kernel.as_ref(),
                commercial_billing.as_ref(),
                config,
            )
            .await?;
            bootstrap_official_provider_access(admin_store.as_ref(), &secret_manager, config)
                .await?;
            Ok::<StandaloneAdminPaymentStoreHandles, anyhow::Error>(StandaloneAdminPaymentStoreHandles {
                admin_store,
                commercial_billing,
                payment_store,
                identity_store,
            })
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

#[derive(Clone, Copy)]
struct OfficialBootstrapDefinition<'a> {
    provider_id: &'a str,
    channel_id: &'a str,
    channel_name: &'a str,
    adapter_kind: &'a str,
    extension_id: &'a str,
    display_name: &'a str,
    enabled: bool,
    base_url: &'a str,
    api_key: &'a str,
}

async fn bootstrap_official_provider_access(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    config: &StandaloneConfig,
) -> Result<()> {
    let definitions = [
        OfficialBootstrapDefinition {
            provider_id: "provider-openai-official",
            channel_id: "openai",
            channel_name: "OpenAI",
            adapter_kind: "openai",
            extension_id: "sdkwork.provider.openai",
            display_name: "OpenAI Official",
            enabled: config.official_openai_enabled,
            base_url: &config.official_openai_base_url,
            api_key: &config.official_openai_api_key,
        },
        OfficialBootstrapDefinition {
            provider_id: "provider-anthropic-official",
            channel_id: "anthropic",
            channel_name: "Anthropic",
            adapter_kind: "anthropic",
            extension_id: "sdkwork.provider.anthropic",
            display_name: "Anthropic Official",
            enabled: config.official_anthropic_enabled,
            base_url: &config.official_anthropic_base_url,
            api_key: &config.official_anthropic_api_key,
        },
        OfficialBootstrapDefinition {
            provider_id: "provider-gemini-official",
            channel_id: "gemini",
            channel_name: "Gemini",
            adapter_kind: "gemini",
            extension_id: "sdkwork.provider.gemini",
            display_name: "Gemini Official",
            enabled: config.official_gemini_enabled,
            base_url: &config.official_gemini_base_url,
            api_key: &config.official_gemini_api_key,
        },
    ];

    for definition in definitions {
        ensure_builtin_official_channel(store, definition).await?;
        ensure_builtin_official_provider(store, definition).await?;
        ensure_official_provider_config_seed(store, secret_manager, definition).await?;
    }

    ensure_starter_official_models(store).await?;

    Ok(())
}

async fn ensure_builtin_official_channel(
    store: &dyn AdminStore,
    definition: OfficialBootstrapDefinition<'_>,
) -> Result<()> {
    if store.list_channels().await?.iter().any(|channel| channel.id == definition.channel_id) {
        return Ok(());
    }

    sdkwork_api_app_catalog::persist_channel(store, definition.channel_id, definition.channel_name)
        .await?;
    Ok(())
}

async fn ensure_builtin_official_provider(
    store: &dyn AdminStore,
    definition: OfficialBootstrapDefinition<'_>,
) -> Result<()> {
    if store.find_provider(definition.provider_id).await?.is_some() {
        return Ok(());
    }

    sdkwork_api_app_catalog::persist_provider_with_bindings_and_extension_id(
        store,
        sdkwork_api_app_catalog::PersistProviderWithBindingsRequest {
            id: definition.provider_id,
            channel_id: definition.channel_id,
            adapter_kind: definition.adapter_kind,
            protocol_kind: Some(definition.adapter_kind),
            extension_id: Some(definition.extension_id),
            base_url: definition.base_url,
            display_name: definition.display_name,
            channel_bindings: &[],
        },
    )
    .await?;
    Ok(())
}

async fn ensure_official_provider_config_seed(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    definition: OfficialBootstrapDefinition<'_>,
) -> Result<()> {
    let existing = store
        .find_official_provider_config(definition.provider_id)
        .await?;

    let metadata_differs = existing
        .as_ref()
        .map(|config| config.base_url != definition.base_url || config.enabled != definition.enabled)
        .unwrap_or(true);
    let secret_missing = !definition.api_key.trim().is_empty()
        && !sdkwork_api_app_credential::official_provider_secret_configured(
            store,
            definition.provider_id,
        )
        .await?;

    if metadata_differs || secret_missing {
        let secret_value = if secret_missing {
            definition.api_key
        } else {
            ""
        };
        sdkwork_api_app_credential::persist_official_provider_config_with_secret_and_manager(
            store,
            secret_manager,
            definition.provider_id,
            definition.base_url,
            definition.enabled,
            secret_value,
        )
        .await?;
    }

    Ok(())
}

#[derive(Clone)]
struct StarterModelSeed<'a> {
    external_name: &'a str,
    provider_id: &'a str,
    channel_id: &'a str,
    model_display_name: &'a str,
    capabilities: Vec<sdkwork_api_domain_catalog::ModelCapability>,
    streaming: bool,
    context_window: Option<u64>,
    description: &'a str,
    input_price: f64,
    output_price: f64,
    request_price: f64,
}

async fn ensure_starter_official_models(store: &dyn AdminStore) -> Result<()> {
    let seeds = vec![
        StarterModelSeed {
            external_name: "gpt-4.1",
            provider_id: "provider-openai-official",
            channel_id: "openai",
            model_display_name: "GPT-4.1",
            capabilities: vec![
                sdkwork_api_domain_catalog::ModelCapability::Responses,
                sdkwork_api_domain_catalog::ModelCapability::ChatCompletions,
            ],
            streaming: true,
            context_window: Some(128_000),
            description: "Flagship general-purpose model for production defaults.",
            input_price: 2.0,
            output_price: 8.0,
            request_price: 0.0,
        },
        StarterModelSeed {
            external_name: "gpt-4.1-mini",
            provider_id: "provider-openai-official",
            channel_id: "openai",
            model_display_name: "GPT-4.1 Mini",
            capabilities: vec![
                sdkwork_api_domain_catalog::ModelCapability::Responses,
                sdkwork_api_domain_catalog::ModelCapability::ChatCompletions,
            ],
            streaming: true,
            context_window: Some(128_000),
            description: "Lower-cost OpenAI default for balanced traffic.",
            input_price: 0.4,
            output_price: 1.6,
            request_price: 0.0,
        },
        StarterModelSeed {
            external_name: "text-embedding-3-small",
            provider_id: "provider-openai-official",
            channel_id: "openai",
            model_display_name: "text-embedding-3-small",
            capabilities: vec![sdkwork_api_domain_catalog::ModelCapability::Embeddings],
            streaming: false,
            context_window: Some(8_192),
            description: "Default OpenAI embedding model.",
            input_price: 0.02,
            output_price: 0.0,
            request_price: 0.0,
        },
        StarterModelSeed {
            external_name: "text-embedding-3-large",
            provider_id: "provider-openai-official",
            channel_id: "openai",
            model_display_name: "text-embedding-3-large",
            capabilities: vec![sdkwork_api_domain_catalog::ModelCapability::Embeddings],
            streaming: false,
            context_window: Some(8_192),
            description: "Higher quality OpenAI embedding model.",
            input_price: 0.13,
            output_price: 0.0,
            request_price: 0.0,
        },
        StarterModelSeed {
            external_name: "gpt-3.5-turbo-instruct",
            provider_id: "provider-openai-official",
            channel_id: "openai",
            model_display_name: "GPT-3.5 Turbo Instruct",
            capabilities: vec![sdkwork_api_domain_catalog::ModelCapability::Completions],
            streaming: false,
            context_window: Some(16_384),
            description: "Compatibility completion model for legacy instruct workloads.",
            input_price: 1.5,
            output_price: 2.0,
            request_price: 0.0,
        },
        StarterModelSeed {
            external_name: "gpt-image-1",
            provider_id: "provider-openai-official",
            channel_id: "openai",
            model_display_name: "GPT Image 1",
            capabilities: vec![sdkwork_api_domain_catalog::ModelCapability::Responses],
            streaming: false,
            context_window: None,
            description: "Image generation starter model.",
            input_price: 0.0,
            output_price: 0.0,
            request_price: 0.04,
        },
        StarterModelSeed {
            external_name: "gpt-4o-mini-transcribe",
            provider_id: "provider-openai-official",
            channel_id: "openai",
            model_display_name: "GPT-4o Mini Transcribe",
            capabilities: vec![sdkwork_api_domain_catalog::ModelCapability::Responses],
            streaming: false,
            context_window: Some(128_000),
            description: "Speech-to-text starter model.",
            input_price: 1.25,
            output_price: 0.0,
            request_price: 0.0,
        },
        StarterModelSeed {
            external_name: "gpt-4o-mini-tts",
            provider_id: "provider-openai-official",
            channel_id: "openai",
            model_display_name: "GPT-4o Mini TTS",
            capabilities: vec![sdkwork_api_domain_catalog::ModelCapability::Responses],
            streaming: false,
            context_window: Some(128_000),
            description: "Text-to-speech starter model.",
            input_price: 0.6,
            output_price: 0.0,
            request_price: 0.0,
        },
        StarterModelSeed {
            external_name: "gpt-4o-realtime-preview",
            provider_id: "provider-openai-official",
            channel_id: "openai",
            model_display_name: "GPT-4o Realtime Preview",
            capabilities: vec![sdkwork_api_domain_catalog::ModelCapability::Responses],
            streaming: true,
            context_window: Some(128_000),
            description: "Realtime interactive starter model.",
            input_price: 5.0,
            output_price: 20.0,
            request_price: 0.0,
        },
        StarterModelSeed {
            external_name: "claude-3-7-sonnet",
            provider_id: "provider-anthropic-official",
            channel_id: "anthropic",
            model_display_name: "Claude 3.7 Sonnet",
            capabilities: vec![sdkwork_api_domain_catalog::ModelCapability::Responses],
            streaming: true,
            context_window: Some(200_000),
            description: "Anthropic starter reasoning model.",
            input_price: 3.0,
            output_price: 15.0,
            request_price: 0.0,
        },
        StarterModelSeed {
            external_name: "claude-3-5-haiku",
            provider_id: "provider-anthropic-official",
            channel_id: "anthropic",
            model_display_name: "Claude 3.5 Haiku",
            capabilities: vec![sdkwork_api_domain_catalog::ModelCapability::Responses],
            streaming: true,
            context_window: Some(200_000),
            description: "Lower-latency Anthropic starter model.",
            input_price: 0.8,
            output_price: 4.0,
            request_price: 0.0,
        },
        StarterModelSeed {
            external_name: "gemini-2.5-pro",
            provider_id: "provider-gemini-official",
            channel_id: "gemini",
            model_display_name: "Gemini 2.5 Pro",
            capabilities: vec![
                sdkwork_api_domain_catalog::ModelCapability::Responses,
                sdkwork_api_domain_catalog::ModelCapability::ChatCompletions,
            ],
            streaming: true,
            context_window: Some(1_000_000),
            description: "High-context Gemini starter model.",
            input_price: 2.5,
            output_price: 10.0,
            request_price: 0.0,
        },
        StarterModelSeed {
            external_name: "gemini-2.5-flash",
            provider_id: "provider-gemini-official",
            channel_id: "gemini",
            model_display_name: "Gemini 2.5 Flash",
            capabilities: vec![
                sdkwork_api_domain_catalog::ModelCapability::Responses,
                sdkwork_api_domain_catalog::ModelCapability::ChatCompletions,
            ],
            streaming: true,
            context_window: Some(1_000_000),
            description: "Lower-latency Gemini starter model.",
            input_price: 0.35,
            output_price: 1.4,
            request_price: 0.0,
        },
        StarterModelSeed {
            external_name: "text-embedding-004",
            provider_id: "provider-gemini-official",
            channel_id: "gemini",
            model_display_name: "text-embedding-004",
            capabilities: vec![sdkwork_api_domain_catalog::ModelCapability::Embeddings],
            streaming: false,
            context_window: Some(2_048),
            description: "Gemini embedding starter model.",
            input_price: 0.15,
            output_price: 0.0,
            request_price: 0.0,
        },
    ];

    let mut existing_model_keys = store
        .list_models()
        .await?
        .into_iter()
        .map(|model| format!("{}::{}", model.external_name, model.provider_id))
        .collect::<std::collections::HashSet<_>>();
    let mut existing_channel_model_keys = store
        .list_channel_models()
        .await?
        .into_iter()
        .map(|record| format!("{}::{}", record.channel_id, record.model_id))
        .collect::<std::collections::HashSet<_>>();
    let mut existing_model_price_keys = store
        .list_model_prices()
        .await?
        .into_iter()
        .map(|record| {
            format!(
                "{}::{}::{}",
                record.channel_id, record.model_id, record.proxy_provider_id
            )
        })
        .collect::<std::collections::HashSet<_>>();

    for seed in seeds {
        let model_key = format!("{}::{}", seed.external_name, seed.provider_id);
        if existing_model_keys.insert(model_key) {
            sdkwork_api_app_catalog::persist_model_with_metadata(
                store,
                seed.external_name,
                seed.provider_id,
                &seed.capabilities,
                seed.streaming,
                seed.context_window,
            )
            .await?;
        }

        let channel_model_key = format!("{}::{}", seed.channel_id, seed.external_name);
        if existing_channel_model_keys.insert(channel_model_key) {
            sdkwork_api_app_catalog::persist_channel_model_with_metadata(
                store,
                seed.channel_id,
                seed.external_name,
                seed.model_display_name,
                &seed.capabilities,
                seed.streaming,
                seed.context_window,
                Some(seed.description),
            )
            .await?;
        }

        let model_price_key = format!(
            "{}::{}::{}",
            seed.channel_id, seed.external_name, seed.provider_id
        );
        if existing_model_price_keys.insert(model_price_key) {
            sdkwork_api_app_catalog::persist_model_price_with_rates(
                store,
                seed.channel_id,
                seed.external_name,
                seed.provider_id,
                "USD",
                "per_1m_tokens",
                seed.input_price,
                seed.output_price,
                0.0,
                0.0,
                seed.request_price,
                true,
            )
            .await?;
        }
    }

    Ok(())
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
