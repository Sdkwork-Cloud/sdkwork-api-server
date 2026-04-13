use sdkwork_api_domain_billing::{
    AccountBenefitLotRecord, AccountBenefitSourceType, AccountBenefitType,
    AccountHoldAllocationRecord, AccountHoldRecord, AccountLedgerAllocationRecord,
    AccountLedgerEntryRecord, AccountLedgerEntryType, AccountRecord, AccountType,
    BillingAccountingMode, BillingEventRecord, PricingPlanRecord, PricingRateRecord, QuotaPolicy,
    RequestSettlementRecord, RequestSettlementStatus,
};
use sdkwork_api_domain_catalog::{
    Channel, ModelCatalogEntry, ProviderChannelBinding, ProxyProvider,
};
use sdkwork_api_domain_credential::UpstreamCredential;
use sdkwork_api_domain_identity::{
    CanonicalApiKeyRecord, IdentityBindingRecord, IdentityUserRecord,
};
use sdkwork_api_domain_routing::{
    ProviderHealthSnapshot, RoutingCandidateAssessment, RoutingDecisionLog, RoutingDecisionSource,
    RoutingPolicy, RoutingStrategy,
};
use sdkwork_api_domain_usage::{
    RequestMeterFactRecord, RequestMeterMetricRecord, RequestStatus, UsageCaptureStatus,
    UsageRecord,
};
use sdkwork_api_secret_core::encrypt;
use sdkwork_api_storage_core::{AccountKernelCommandBatch, AccountKernelStore, IdentityKernelStore};
use sdkwork_api_storage_postgres::{run_migrations, PostgresAdminStore};
use sqlx::PgPool;

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
async fn postgres_store_creates_canonical_account_kernel_tables_when_url_is_provided() {
    let Some(database_url) = std::env::var("SDKWORK_TEST_POSTGRES_URL").ok() else {
        return;
    };

    let pool = run_migrations(&database_url).await.unwrap();

    for table_name in [
        "ai_account",
        "ai_account_benefit_lot",
        "ai_account_hold",
        "ai_account_hold_allocation",
        "ai_account_ledger_entry",
        "ai_account_ledger_allocation",
        "ai_request_meter_fact",
        "ai_request_meter_metric",
        "ai_request_settlement",
        "ai_pricing_plan",
        "ai_pricing_rate",
    ] {
        let row: (String,) = sqlx::query_as(
            "select tablename
             from pg_tables
             where schemaname = 'public' and tablename = $1",
        )
        .bind(table_name)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(row.0, table_name);
    }

    assert_pg_column(&pool, "ai_account", "tenant_id", "bigint", false, None).await;
    assert_pg_column(
        &pool,
        "ai_account",
        "organization_id",
        "bigint",
        false,
        Some("0"),
    )
    .await;
    assert_pg_column(&pool, "ai_account", "user_id", "bigint", false, None).await;
    assert_pg_column(
        &pool,
        "ai_request_meter_fact",
        "organization_id",
        "bigint",
        false,
        Some("0"),
    )
    .await;
    assert_pg_column(
        &pool,
        "ai_request_meter_fact",
        "account_id",
        "bigint",
        false,
        None,
    )
    .await;
    assert_pg_column(
        &pool,
        "ai_request_settlement",
        "organization_id",
        "bigint",
        false,
        Some("0"),
    )
    .await;
    assert_pg_column(
        &pool,
        "ai_account_hold_allocation",
        "organization_id",
        "bigint",
        false,
        Some("0"),
    )
    .await;
    assert_pg_column(
        &pool,
        "ai_account_ledger_allocation",
        "organization_id",
        "bigint",
        false,
        Some("0"),
    )
    .await;

    let index_names: Vec<(String,)> = sqlx::query_as(
        "select indexname
         from pg_indexes
         where schemaname = 'public'
           and tablename in (
             'ai_account',
             'ai_account_benefit_lot',
             'ai_account_hold',
             'ai_account_hold_allocation',
             'ai_account_ledger_allocation',
             'ai_request_meter_fact',
             'ai_request_settlement',
             'ai_pricing_plan'
           )
         order by indexname",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    let index_names = index_names
        .into_iter()
        .map(|(name,)| name)
        .collect::<std::collections::HashSet<_>>();
    for index_name in [
        "idx_ai_account_user_type",
        "idx_ai_account_benefit_lot_account_status_expiry",
        "idx_ai_account_hold_request",
        "idx_ai_account_hold_allocation_hold_lot",
        "idx_ai_account_ledger_allocation_ledger_lot",
        "idx_ai_request_meter_fact_user_created_at",
        "idx_ai_request_meter_fact_api_key_created_at",
        "idx_ai_request_settlement_request",
        "idx_ai_pricing_plan_code_version",
    ] {
        assert!(
            index_names.contains(index_name),
            "missing index {index_name}"
        );
    }
}

#[tokio::test]
async fn postgres_store_creates_canonical_identity_kernel_tables_when_url_is_provided() {
    let Some(database_url) = std::env::var("SDKWORK_TEST_POSTGRES_URL").ok() else {
        return;
    };

    let pool = run_migrations(&database_url).await.unwrap();

    for table_name in ["ai_user", "ai_api_key", "ai_identity_binding"] {
        let row: (String,) = sqlx::query_as(
            "select tablename
             from pg_tables
             where schemaname = 'public' and tablename = $1",
        )
        .bind(table_name)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(row.0, table_name);
    }

    assert_pg_column(
        &pool,
        "ai_user",
        "organization_id",
        "bigint",
        false,
        Some("0"),
    )
    .await;
    assert_pg_column(
        &pool,
        "ai_api_key",
        "organization_id",
        "bigint",
        false,
        Some("0"),
    )
    .await;
    assert_pg_column(
        &pool,
        "ai_identity_binding",
        "organization_id",
        "bigint",
        false,
        Some("0"),
    )
    .await;

    let index_names: Vec<(String,)> = sqlx::query_as(
        "select indexname
         from pg_indexes
         where schemaname = 'public'
           and tablename in ('ai_user', 'ai_api_key', 'ai_identity_binding')
         order by indexname",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    let index_names = index_names
        .into_iter()
        .map(|(name,)| name)
        .collect::<std::collections::HashSet<_>>();
    for index_name in [
        "idx_ai_user_scope",
        "idx_ai_user_email",
        "idx_ai_api_key_hash",
        "idx_ai_api_key_user_status",
        "idx_ai_identity_binding_lookup",
    ] {
        assert!(
            index_names.contains(index_name),
            "missing index {index_name}"
        );
    }
}

#[tokio::test]
async fn postgres_store_round_trips_canonical_identity_kernel_records_when_url_is_provided() {
    let Some(database_url) = std::env::var("SDKWORK_TEST_POSTGRES_URL").ok() else {
        return;
    };

    let pool = run_migrations(&database_url).await.unwrap();
    let store = PostgresAdminStore::new(pool);

    let user = IdentityUserRecord::new(7101, 8101, 9101)
        .with_external_user_ref(Some("ext-user-7101".to_owned()))
        .with_username(Some("identity-user".to_owned()))
        .with_display_name(Some("Identity User".to_owned()))
        .with_email(Some("identity@example.com".to_owned()))
        .with_created_at_ms(1_710_000_000_100)
        .with_updated_at_ms(1_710_000_000_200);
    store.insert_identity_user_record(&user).await.unwrap();

    let api_key = CanonicalApiKeyRecord::new(7201, 8101, 9101, 7101, "hash-identity-7201")
        .with_key_prefix("sk-router")
        .with_display_name("Primary canonical key")
        .with_last_used_at_ms(Some(1_710_000_000_300))
        .with_created_at_ms(1_710_000_000_100)
        .with_updated_at_ms(1_710_000_000_300);
    store.insert_canonical_api_key_record(&api_key).await.unwrap();

    let binding = IdentityBindingRecord::new(7301, 8101, 9101, 7101, "oauth")
        .with_issuer(Some("https://issuer.example.com".to_owned()))
        .with_subject(Some("sub-7101".to_owned()))
        .with_platform(Some("portal".to_owned()))
        .with_owner(Some("workspace".to_owned()))
        .with_external_ref(Some("binding-ext-7301".to_owned()))
        .with_created_at_ms(1_710_000_000_400)
        .with_updated_at_ms(1_710_000_000_500);
    store.insert_identity_binding_record(&binding).await.unwrap();

    let users = store.list_identity_user_records().await.unwrap();
    assert_eq!(users, vec![user.clone()]);

    let found_user = store.find_identity_user_record(user.user_id).await.unwrap();
    assert_eq!(found_user, Some(user));

    let found_key = store
        .find_canonical_api_key_record_by_hash(&api_key.key_hash)
        .await
        .unwrap();
    assert_eq!(found_key, Some(api_key));

    let found_binding = store
        .find_identity_binding_record("oauth", Some("https://issuer.example.com"), Some("sub-7101"))
        .await
        .unwrap();
    assert_eq!(found_binding, Some(binding));
}

#[tokio::test]
async fn postgres_store_round_trips_account_kernel_records_when_url_is_provided() {
    let Some(database_url) = std::env::var("SDKWORK_TEST_POSTGRES_URL").ok() else {
        return;
    };

    let pool = run_migrations(&database_url).await.unwrap();
    let store = PostgresAdminStore::new(pool);

    let account = AccountRecord::new(7401, 8101, 9101, 7101, AccountType::Primary)
        .with_allow_overdraft(true)
        .with_overdraft_limit(25.0)
        .with_created_at_ms(1_710_000_001_000)
        .with_updated_at_ms(1_710_000_001_100);
    let lot = AccountBenefitLotRecord::new(
        7402,
        8101,
        9101,
        7401,
        7101,
        AccountBenefitType::CashCredit,
    )
    .with_source_type(AccountBenefitSourceType::Recharge)
    .with_original_quantity(120.0)
    .with_remaining_quantity(120.0)
    .with_held_quantity(20.0)
    .with_created_at_ms(1_710_000_001_200)
    .with_updated_at_ms(1_710_000_001_300);
    let hold = AccountHoldRecord::new(7403, 8101, 9101, 7401, 7101, 7407)
        .with_estimated_quantity(20.0)
        .with_captured_quantity(15.0)
        .with_released_quantity(5.0)
        .with_expires_at_ms(1_710_000_002_000)
        .with_created_at_ms(1_710_000_001_400)
        .with_updated_at_ms(1_710_000_001_500);
    let hold_allocation = AccountHoldAllocationRecord::new(7404, 8101, 9101, 7403, 7402)
        .with_allocated_quantity(20.0)
        .with_captured_quantity(15.0)
        .with_released_quantity(5.0)
        .with_created_at_ms(1_710_000_001_600)
        .with_updated_at_ms(1_710_000_001_700);
    let ledger_entry = AccountLedgerEntryRecord::new(
        7405,
        8101,
        9101,
        7401,
        7101,
        AccountLedgerEntryType::SettlementCapture,
    )
    .with_request_id(Some(7407))
    .with_hold_id(Some(7403))
    .with_benefit_type(Some("cash_credit".to_owned()))
    .with_quantity(15.0)
    .with_amount(15.0)
    .with_created_at_ms(1_710_000_001_800);
    let ledger_allocation = AccountLedgerAllocationRecord::new(7406, 8101, 9101, 7405, 7402)
        .with_quantity_delta(15.0)
        .with_created_at_ms(1_710_000_001_900);
    let request_fact = RequestMeterFactRecord::new(
        7407,
        8101,
        9101,
        7101,
        7401,
        "api_key",
        "responses",
        "openai",
        "gpt-4.1",
        "provider-openai-official",
    )
    .with_api_key_id(Some(7201))
    .with_api_key_hash(Some("hash-identity-7201".to_owned()))
    .with_protocol_family("openai")
    .with_request_status(RequestStatus::Succeeded)
    .with_usage_capture_status(UsageCaptureStatus::Captured)
    .with_estimated_credit_hold(20.0)
    .with_actual_credit_charge(Some(15.0))
    .with_actual_provider_cost(Some(4.5))
    .with_started_at_ms(1_710_000_001_000)
    .with_finished_at_ms(Some(1_710_000_001_950))
    .with_created_at_ms(1_710_000_001_000)
    .with_updated_at_ms(1_710_000_001_950);
    let request_metric = RequestMeterMetricRecord::new(7408, 8101, 9101, 7407, "total_tokens", 512.0)
        .with_provider_field(Some("usage.total_tokens".to_owned()))
        .with_capture_stage("final")
        .with_captured_at_ms(1_710_000_001_960);
    let settlement = RequestSettlementRecord::new(7409, 8101, 9101, 7407, 7401, 7101)
        .with_hold_id(Some(7403))
        .with_status(RequestSettlementStatus::PartiallyReleased)
        .with_estimated_credit_hold(20.0)
        .with_released_credit_amount(5.0)
        .with_captured_credit_amount(15.0)
        .with_provider_cost_amount(4.5)
        .with_retail_charge_amount(15.0)
        .with_settled_at_ms(1_710_000_001_970)
        .with_created_at_ms(1_710_000_001_970)
        .with_updated_at_ms(1_710_000_001_980);

    store
        .commit_account_kernel_batch(&AccountKernelCommandBatch {
            account_records: vec![account.clone()],
            benefit_lot_records: vec![lot.clone()],
            hold_records: vec![hold.clone()],
            hold_allocation_records: vec![hold_allocation.clone()],
            ledger_entry_records: vec![ledger_entry.clone()],
            ledger_allocation_records: vec![ledger_allocation.clone()],
            request_meter_fact_records: vec![request_fact.clone()],
            request_meter_metric_records: vec![request_metric.clone()],
            request_settlement_records: vec![settlement.clone()],
        })
        .await
        .unwrap();

    let pricing_plan = PricingPlanRecord::new(7410, 8101, 9101, "standard", 1)
        .with_display_name("Standard")
        .with_status("published")
        .with_created_at_ms(1_710_000_002_000)
        .with_updated_at_ms(1_710_000_002_100);
    store.insert_pricing_plan_record(&pricing_plan).await.unwrap();

    let pricing_rate = PricingRateRecord::new(7411, 8101, 9101, 7410, "tokens.input")
        .with_model_code(Some("gpt-4.1".to_owned()))
        .with_provider_code(Some("provider-openai-official".to_owned()))
        .with_quantity_step(1_000.0)
        .with_unit_price(0.25)
        .with_created_at_ms(1_710_000_002_200);
    store.insert_pricing_rate_record(&pricing_rate).await.unwrap();

    assert_eq!(
        store.find_account_record(account.account_id).await.unwrap(),
        Some(account.clone())
    );
    assert_eq!(
        store
            .find_account_record_by_owner(8101, 9101, 7101, AccountType::Primary)
            .await
            .unwrap(),
        Some(account)
    );
    assert_eq!(store.list_account_benefit_lots().await.unwrap(), vec![lot]);
    assert_eq!(store.list_account_holds().await.unwrap(), vec![hold]);
    assert_eq!(
        store.list_account_hold_allocations().await.unwrap(),
        vec![hold_allocation]
    );
    assert_eq!(
        store.list_account_ledger_entry_records().await.unwrap(),
        vec![ledger_entry]
    );
    assert_eq!(
        store.list_account_ledger_allocations().await.unwrap(),
        vec![ledger_allocation]
    );
    assert_eq!(store.list_request_meter_facts().await.unwrap(), vec![request_fact]);
    assert_eq!(
        store.list_request_meter_metrics().await.unwrap(),
        vec![request_metric]
    );
    assert_eq!(
        store.list_request_settlement_records().await.unwrap(),
        vec![settlement]
    );
    assert_eq!(store.list_pricing_plan_records().await.unwrap(), vec![pricing_plan]);
    assert_eq!(store.list_pricing_rate_records().await.unwrap(), vec![pricing_rate]);
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

async fn assert_pg_column(
    pool: &PgPool,
    table_name: &str,
    column_name: &str,
    data_type: &str,
    nullable: bool,
    default_contains: Option<&str>,
) {
    let row: (String, String, Option<String>) = sqlx::query_as(
        "select data_type, is_nullable, column_default
         from information_schema.columns
         where table_schema = 'public'
           and table_name = $1
           and column_name = $2",
    )
    .bind(table_name)
    .bind(column_name)
    .fetch_one(pool)
    .await
    .unwrap();

    assert_eq!(row.0, data_type);
    assert_eq!(row.1 == "YES", nullable);
    match default_contains {
        Some(expected) => assert!(
            row.2
                .as_deref()
                .is_some_and(|value| value.contains(expected)),
            "expected default for {table_name}.{column_name} to contain {expected:?}, got {:?}",
            row.2
        ),
        None => {}
    }
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
async fn postgres_store_persists_billing_events_when_url_is_provided() {
    let Some(database_url) = std::env::var("SDKWORK_TEST_POSTGRES_URL").ok() else {
        return;
    };

    let pool = run_migrations(&database_url).await.unwrap();
    let store = PostgresAdminStore::new(pool);

    let event = BillingEventRecord::new(
        "evt-postgres-1",
        "tenant-1",
        "project-1",
        "responses",
        "gpt-4.1",
        "gpt-4.1",
        "provider-openrouter",
        BillingAccountingMode::PlatformCredit,
        1_717_171_717,
    )
    .with_api_key_group_id("group-blue")
    .with_operation("responses.create", "multimodal")
    .with_request_facts(
        Some("key-live"),
        Some("openai"),
        Some("resp_123"),
        Some(850),
    )
    .with_units(240)
    .with_token_usage(120, 80, 200)
    .with_cache_token_usage(30, 10)
    .with_media_usage(2, 3.5, 0.0, 12.0)
    .with_financials(0.42, 0.89)
    .with_routing_evidence(
        Some("route-profile-1"),
        Some("snapshot-1"),
        Some("latency_guardrail"),
    );

    store.insert_billing_event(&event).await.unwrap();

    let events = store.list_billing_events().await.unwrap();
    assert!(events.iter().any(|entry| entry == &event));
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
