use sdkwork_api_domain_billing::QuotaPolicy;
use sdkwork_api_domain_catalog::{
    Channel, ModelCatalogEntry, ProviderChannelBinding, ProxyProvider,
};
use sdkwork_api_domain_credential::UpstreamCredential;
use sdkwork_api_domain_routing::{
    ProviderHealthSnapshot, RoutingCandidateAssessment, RoutingDecisionLog, RoutingDecisionSource,
    RoutingPolicy, RoutingStrategy,
};
use sdkwork_api_domain_usage::UsageRecord;
use sdkwork_api_secret_core::encrypt;
use sdkwork_api_storage_postgres::{run_migrations, PostgresAdminStore};

#[tokio::test]
async fn postgres_store_persists_catalog_and_credentials_when_url_is_provided() {
    let Some(database_url) = std::env::var("SDKWORK_TEST_POSTGRES_URL").ok() else {
        return;
    };

    let pool = run_migrations(&database_url).await.unwrap();
    let store = PostgresAdminStore::new(pool.clone());

    let channel = store
        .insert_channel(&Channel::new("openai", "OpenAI"))
        .await
        .unwrap();
    assert_eq!(channel.id, "openai");

    let provider = store
        .insert_provider(&ProxyProvider::new(
            "provider-openai-official",
            "openai",
            "openai",
            "https://api.openai.com",
            "OpenAI Official",
        ))
        .await
        .unwrap();
    assert_eq!(provider.adapter_kind, "openai");

    let model = store
        .insert_model(&ModelCatalogEntry::new(
            "gpt-4.1",
            "provider-openai-official",
        ))
        .await
        .unwrap();
    assert_eq!(model.external_name, "gpt-4.1");

    let credential = UpstreamCredential::new("tenant-1", "provider-openai-official", "cred-openai");
    let envelope = encrypt("local-dev-master-key", "sk-upstream-openai").unwrap();
    store
        .insert_encrypted_credential(&credential, &envelope)
        .await
        .unwrap();

    let stored = store
        .find_credential_envelope("tenant-1", "provider-openai-official", "cred-openai")
        .await
        .unwrap()
        .expect("credential envelope");
    assert_eq!(stored, envelope);

    let models = store.list_models().await.unwrap();
    assert_eq!(models.len(), 1);
    assert_eq!(models[0].provider_id, "provider-openai-official");

    let index_names: Vec<(String,)> = sqlx::query_as(
        "select indexname
         from pg_indexes
         where tablename = 'ai_model_price'
         order by indexname",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    let index_names = index_names
        .into_iter()
        .map(|(name,)| name)
        .collect::<std::collections::HashSet<_>>();
    assert!(index_names.contains("idx_ai_model_price_model_active"));
}

#[tokio::test]
async fn postgres_store_persists_routing_policies_when_url_is_provided() {
    let Some(database_url) = std::env::var("SDKWORK_TEST_POSTGRES_URL").ok() else {
        return;
    };

    let pool = run_migrations(&database_url).await.unwrap();
    let store = PostgresAdminStore::new(pool);

    let policy = RoutingPolicy::new("policy-gpt-4-1", "chat_completion", "gpt-4.1")
        .with_priority(100)
        .with_strategy(RoutingStrategy::WeightedRandom)
        .with_ordered_provider_ids(vec![
            "provider-openrouter".to_owned(),
            "provider-openai-official".to_owned(),
        ])
        .with_default_provider_id("provider-openai-official");

    store.insert_routing_policy(&policy).await.unwrap();

    let stored = store
        .list_routing_policies()
        .await
        .unwrap()
        .into_iter()
        .find(|entry| entry.policy_id == "policy-gpt-4-1")
        .expect("routing policy");
    assert_eq!(
        stored.ordered_provider_ids,
        vec![
            "provider-openrouter".to_owned(),
            "provider-openai-official".to_owned(),
        ]
    );
    assert_eq!(
        stored.default_provider_id.as_deref(),
        Some("provider-openai-official")
    );
    assert_eq!(stored.strategy, RoutingStrategy::WeightedRandom);
}

#[tokio::test]
async fn postgres_store_round_trips_slo_policy_fields_when_url_is_provided() {
    let Some(database_url) = std::env::var("SDKWORK_TEST_POSTGRES_URL").ok() else {
        return;
    };

    let pool = run_migrations(&database_url).await.unwrap();
    let store = PostgresAdminStore::new(pool);

    let policy = RoutingPolicy::new("policy-slo", "chat_completion", "gpt-4.1")
        .with_strategy(RoutingStrategy::SloAware)
        .with_max_cost(0.35)
        .with_max_latency_ms(300)
        .with_require_healthy(true)
        .with_ordered_provider_ids(vec!["provider-openrouter".to_owned()]);

    store.insert_routing_policy(&policy).await.unwrap();

    let stored = store
        .list_routing_policies()
        .await
        .unwrap()
        .into_iter()
        .find(|entry| entry.policy_id == "policy-slo")
        .expect("routing policy");
    assert_eq!(stored.strategy, RoutingStrategy::SloAware);
    assert_eq!(stored.max_cost, Some(0.35));
    assert_eq!(stored.max_latency_ms, Some(300));
    assert!(stored.require_healthy);
}

#[tokio::test]
async fn postgres_store_persists_routing_decision_logs_when_url_is_provided() {
    let Some(database_url) = std::env::var("SDKWORK_TEST_POSTGRES_URL").ok() else {
        return;
    };

    let pool = run_migrations(&database_url).await.unwrap();
    let store = PostgresAdminStore::new(pool);

    let log = RoutingDecisionLog::new(
        "decision-postgres",
        RoutingDecisionSource::Gateway,
        "chat_completion",
        "gpt-4.1",
        "provider-openai-official",
        "slo_aware",
        1234,
    )
    .with_tenant_id("tenant-1")
    .with_project_id("project-1")
    .with_selection_reason(
        "selected provider-openai-official as the top-ranked SLO-compliant candidate",
    )
    .with_slo_state(true, false)
    .with_assessments(vec![RoutingCandidateAssessment::new(
        "provider-openai-official",
    )
    .with_slo_eligible(true)]);

    store.insert_routing_decision_log(&log).await.unwrap();

    let logs = store.list_routing_decision_logs().await.unwrap();
    assert!(logs.iter().any(|entry| entry == &log));
}

#[tokio::test]
async fn postgres_store_round_trips_requested_region_in_routing_decision_logs_when_url_is_provided()
{
    let Some(database_url) = std::env::var("SDKWORK_TEST_POSTGRES_URL").ok() else {
        return;
    };

    let pool = run_migrations(&database_url).await.unwrap();
    let store = PostgresAdminStore::new(pool);

    let log = RoutingDecisionLog::new(
        "decision-postgres-region",
        RoutingDecisionSource::AdminSimulation,
        "chat_completion",
        "gpt-4.1",
        "provider-us-east",
        "geo_affinity",
        4321,
    )
    .with_requested_region("us-east")
    .with_assessments(vec![RoutingCandidateAssessment::new("provider-us-east")
        .with_region("us-east")
        .with_region_match(true)]);

    store.insert_routing_decision_log(&log).await.unwrap();

    let logs = store.list_routing_decision_logs().await.unwrap();
    assert!(logs.iter().any(|entry| entry == &log));
}

#[tokio::test]
async fn postgres_store_persists_provider_health_snapshots_when_url_is_provided() {
    let Some(database_url) = std::env::var("SDKWORK_TEST_POSTGRES_URL").ok() else {
        return;
    };

    let pool = run_migrations(&database_url).await.unwrap();
    let store = PostgresAdminStore::new(pool);

    let snapshot = ProviderHealthSnapshot::new(
        "provider-openai-official",
        "sdkwork.provider.openai.official",
        "builtin",
        1234,
    )
    .with_running(true)
    .with_healthy(true)
    .with_message("healthy");

    store
        .insert_provider_health_snapshot(&snapshot)
        .await
        .unwrap();

    let snapshots = store.list_provider_health_snapshots().await.unwrap();
    assert!(snapshots.iter().any(|entry| entry == &snapshot));
}

#[tokio::test]
async fn postgres_store_persists_quota_policies_when_url_is_provided() {
    let Some(database_url) = std::env::var("SDKWORK_TEST_POSTGRES_URL").ok() else {
        return;
    };

    let pool = run_migrations(&database_url).await.unwrap();
    let store = PostgresAdminStore::new(pool);

    let policy = QuotaPolicy::new("quota-project-1", "project-1", 1_000).with_enabled(true);

    store.insert_quota_policy(&policy).await.unwrap();

    let policies = store.list_quota_policies().await.unwrap();
    assert!(policies.iter().any(|entry| entry == &policy));
}

#[tokio::test]
async fn postgres_store_finds_latest_project_routing_log_and_usage_record_when_url_is_provided() {
    let Some(database_url) = std::env::var("SDKWORK_TEST_POSTGRES_URL").ok() else {
        return;
    };

    let pool = run_migrations(&database_url).await.unwrap();
    let store = PostgresAdminStore::new(pool);

    store
        .insert_routing_decision_log(
            &RoutingDecisionLog::new(
                "decision-old",
                RoutingDecisionSource::Gateway,
                "chat_completion",
                "gpt-4.1",
                "provider-a",
                "deterministic_priority",
                100,
            )
            .with_project_id("project-1"),
        )
        .await
        .unwrap();
    store
        .insert_routing_decision_log(
            &RoutingDecisionLog::new(
                "decision-new",
                RoutingDecisionSource::Gateway,
                "chat_completion",
                "gpt-4.1",
                "provider-b",
                "deterministic_priority",
                200,
            )
            .with_project_id("project-1"),
        )
        .await
        .unwrap();
    store
        .insert_usage_record(&UsageRecord {
            project_id: "project-1".to_owned(),
            model: "gpt-4.1".to_owned(),
            provider: "provider-a".to_owned(),
            units: 1,
            amount: 0.01,
            input_tokens: 1,
            output_tokens: 2,
            total_tokens: 3,
            created_at_ms: 100,
            api_key_hash: None,
            channel_id: None,
            latency_ms: None,
            reference_amount: None,
        })
        .await
        .unwrap();
    store
        .insert_usage_record(&UsageRecord {
            project_id: "project-1".to_owned(),
            model: "gpt-4.1-mini".to_owned(),
            provider: "provider-b".to_owned(),
            units: 2,
            amount: 0.02,
            input_tokens: 4,
            output_tokens: 5,
            total_tokens: 9,
            created_at_ms: 200,
            api_key_hash: None,
            channel_id: None,
            latency_ms: None,
            reference_amount: None,
        })
        .await
        .unwrap();

    let latest_log = store
        .find_latest_routing_decision_log_for_project("project-1")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(latest_log.decision_id, "decision-new");

    let latest_usage = store
        .find_latest_usage_record_for_project("project-1")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(latest_usage.model, "gpt-4.1-mini");
}

#[tokio::test]
async fn postgres_store_finds_any_model_without_full_scan_when_url_is_provided() {
    let Some(database_url) = std::env::var("SDKWORK_TEST_POSTGRES_URL").ok() else {
        return;
    };

    let pool = run_migrations(&database_url).await.unwrap();
    let store = PostgresAdminStore::new(pool);

    store
        .insert_channel(&Channel::new("openai", "OpenAI"))
        .await
        .unwrap();
    store
        .insert_provider(&ProxyProvider::new(
            "provider-openai-official",
            "openai",
            "openai",
            "https://api.openai.com",
            "OpenAI Official",
        ))
        .await
        .unwrap();
    store
        .insert_model(&ModelCatalogEntry::new(
            "z-model",
            "provider-openai-official",
        ))
        .await
        .unwrap();
    store
        .insert_model(&ModelCatalogEntry::new(
            "a-model",
            "provider-openai-official",
        ))
        .await
        .unwrap();

    let model = store.find_any_model().await.unwrap().unwrap();
    assert_eq!(model.external_name, "a-model");
}

#[tokio::test]
async fn postgres_store_lists_providers_for_model_without_full_scan_when_url_is_provided() {
    let Some(database_url) = std::env::var("SDKWORK_TEST_POSTGRES_URL").ok() else {
        return;
    };

    let pool = run_migrations(&database_url).await.unwrap();
    let store = PostgresAdminStore::new(pool);

    store
        .insert_channel(&Channel::new("openai", "OpenAI"))
        .await
        .unwrap();
    store
        .insert_provider(&ProxyProvider::new(
            "provider-openai-official",
            "openai",
            "openai",
            "https://api.openai.com",
            "OpenAI Official",
        ))
        .await
        .unwrap();
    store
        .insert_model(&ModelCatalogEntry::new(
            "a-model",
            "provider-openai-official",
        ))
        .await
        .unwrap();

    let providers = store.list_providers_for_model("a-model").await.unwrap();
    assert_eq!(providers.len(), 1);
    assert_eq!(providers[0].id, "provider-openai-official");
}

#[tokio::test]
async fn postgres_store_lists_provider_bindings_for_model_without_drop_when_url_is_provided() {
    let Some(database_url) = std::env::var("SDKWORK_TEST_POSTGRES_URL").ok() else {
        return;
    };

    let pool = run_migrations(&database_url).await.unwrap();
    let store = PostgresAdminStore::new(pool);

    let provider = ProxyProvider::new(
        "provider-openrouter-main",
        "openrouter",
        "openrouter",
        "https://openrouter.ai/api/v1",
        "OpenRouter Main",
    )
    .with_channel_binding(ProviderChannelBinding::new(
        "provider-openrouter-main",
        "openai",
    ));

    store.insert_provider(&provider).await.unwrap();
    store
        .insert_model(&ModelCatalogEntry::new(
            "gpt-4.1",
            "provider-openrouter-main",
        ))
        .await
        .unwrap();

    let providers = store.list_providers_for_model("gpt-4.1").await.unwrap();
    assert_eq!(providers.len(), 1);
    assert_eq!(providers[0].channel_bindings.len(), 2);
    assert_eq!(providers[0].channel_bindings[1].channel_id, "openai");
}
