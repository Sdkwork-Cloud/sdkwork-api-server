use super::*;

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
        "idx_ai_account_benefit_lot_account_lot",
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
async fn postgres_store_round_trips_pricing_plans_and_rates_when_url_is_provided() {
    let Some(database_url) = std::env::var("SDKWORK_TEST_POSTGRES_URL").ok() else {
        return;
    };

    let pool = run_migrations(&database_url).await.unwrap();
    let store = PostgresAdminStore::new(pool);
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    let now_ms = 1_717_171_000 + seed;
    let plan_id = 9_100_000 + seed;
    let rate_id = 9_200_000 + seed;

    let plan = PricingPlanRecord::new(plan_id, 1001, 2002, format!("workspace-retail-{seed}"), 1)
        .with_display_name("Workspace Retail")
        .with_currency_code("USD")
        .with_credit_unit_code("credit")
        .with_status("active")
        .with_effective_from_ms(now_ms - 10_000)
        .with_effective_to_ms(Some(now_ms + 86_400_000))
        .with_created_at_ms(now_ms)
        .with_updated_at_ms(now_ms + 1);
    store.insert_pricing_plan_record(&plan).await.unwrap();

    let rate = PricingRateRecord::new(rate_id, 1001, 2002, plan_id, "token.input")
        .with_capability_code(Some("responses".to_owned()))
        .with_model_code(Some("gpt-4.1".to_owned()))
        .with_provider_code(Some("provider-openrouter".to_owned()))
        .with_charge_unit("input_token")
        .with_pricing_method("per_unit")
        .with_quantity_step(1_000_000.0)
        .with_unit_price(2.5)
        .with_display_price_unit("USD / 1M input tokens")
        .with_minimum_billable_quantity(0.0)
        .with_minimum_charge(0.0)
        .with_rounding_increment(1.0)
        .with_rounding_mode("ceil")
        .with_included_quantity(0.0)
        .with_priority(100)
        .with_notes(Some("Retail text input pricing".to_owned()))
        .with_status("active")
        .with_created_at_ms(now_ms)
        .with_updated_at_ms(now_ms + 2);
    store.insert_pricing_rate_record(&rate).await.unwrap();

    let stored_plan = store
        .list_pricing_plan_records()
        .await
        .unwrap()
        .into_iter()
        .find(|entry| entry.pricing_plan_id == plan_id)
        .expect("pricing plan");
    assert_eq!(stored_plan, plan);

    let stored_rate = store
        .list_pricing_rate_records()
        .await
        .unwrap()
        .into_iter()
        .find(|entry| entry.pricing_rate_id == rate_id)
        .expect("pricing rate");
    assert_eq!(stored_rate, rate);
}

#[tokio::test]
async fn postgres_store_round_trips_commercial_account_read_models_when_url_is_provided() {
    let Some(database_url) = std::env::var("SDKWORK_TEST_POSTGRES_URL").ok() else {
        return;
    };

    let pool = run_migrations(&database_url).await.unwrap();
    let store = PostgresAdminStore::new(pool);
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    let now_ms = 1_717_181_000 + seed;
    let account_id = 8_100_000 + seed;
    let lot_id = 8_200_000 + seed;
    let hold_id = 8_300_000 + seed;
    let request_id = 8_400_000 + seed;
    let settlement_id = 8_500_000 + seed;

    let account = AccountRecord::new(account_id, 1001, 2002, 9001, AccountType::Primary)
        .with_created_at_ms(now_ms)
        .with_updated_at_ms(now_ms + 1);
    store.insert_account_record(&account).await.unwrap();

    let lot = AccountBenefitLotRecord::new(
        lot_id,
        1001,
        2002,
        account_id,
        9001,
        AccountBenefitType::CashCredit,
    )
    .with_source_type(AccountBenefitSourceType::Recharge)
    .with_original_quantity(200.0)
    .with_remaining_quantity(160.0)
    .with_held_quantity(10.0)
    .with_priority(10)
    .with_issued_at_ms(now_ms)
    .with_created_at_ms(now_ms + 2)
    .with_updated_at_ms(now_ms + 3);
    store.insert_account_benefit_lot(&lot).await.unwrap();

    let hold = AccountHoldRecord::new(hold_id, 1001, 2002, account_id, 9001, request_id)
        .with_estimated_quantity(10.0)
        .with_captured_quantity(10.0)
        .with_expires_at_ms(now_ms + 60_000)
        .with_created_at_ms(now_ms + 4)
        .with_updated_at_ms(now_ms + 5);
    store.insert_account_hold(&hold).await.unwrap();

    let settlement =
        RequestSettlementRecord::new(settlement_id, 1001, 2002, request_id, account_id, 9001)
            .with_hold_id(Some(hold_id))
            .with_status(RequestSettlementStatus::Captured)
            .with_estimated_credit_hold(10.0)
            .with_captured_credit_amount(10.0)
            .with_provider_cost_amount(5.0)
            .with_retail_charge_amount(10.0)
            .with_settled_at_ms(now_ms + 6)
            .with_created_at_ms(now_ms + 6)
            .with_updated_at_ms(now_ms + 7);
    store
        .insert_request_settlement_record(&settlement)
        .await
        .unwrap();

    let stored_account = store
        .find_account_record(account_id)
        .await
        .unwrap()
        .expect("account");
    assert_eq!(stored_account, account);

    let owner_account = store
        .find_account_record_by_owner(1001, 2002, 9001, AccountType::Primary)
        .await
        .unwrap()
        .expect("owner account");
    assert_eq!(owner_account, account);

    assert!(store
        .list_account_records()
        .await
        .unwrap()
        .iter()
        .any(|entry| entry == &account));
    assert!(store
        .list_account_benefit_lots()
        .await
        .unwrap()
        .iter()
        .any(|entry| entry == &lot));
    assert!(store
        .list_account_holds()
        .await
        .unwrap()
        .iter()
        .any(|entry| entry == &hold));
    assert!(store
        .list_request_settlement_records()
        .await
        .unwrap()
        .iter()
        .any(|entry| entry == &settlement));
}

#[tokio::test]
async fn postgres_store_round_trips_remaining_account_kernel_records_when_url_is_provided() {
    let Some(database_url) = std::env::var("SDKWORK_TEST_POSTGRES_URL").ok() else {
        return;
    };

    let pool = run_migrations(&database_url).await.unwrap();
    let store = PostgresAdminStore::new(pool);
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    let request_id = 6_001_000 + seed;
    let account = AccountRecord::new(7_001_000 + seed, 1001, 2002, 9001, AccountType::Primary)
        .with_currency_code("USD")
        .with_credit_unit_code("credit")
        .with_created_at_ms(10)
        .with_updated_at_ms(20);
    let lot = AccountBenefitLotRecord::new(
        8_001_000 + seed,
        1001,
        2002,
        account.account_id,
        9001,
        AccountBenefitType::CashCredit,
    )
    .with_source_type(AccountBenefitSourceType::Recharge)
    .with_original_quantity(1200.0)
    .with_remaining_quantity(1200.0)
    .with_issued_at_ms(30)
    .with_created_at_ms(30)
    .with_updated_at_ms(30);
    let hold = AccountHoldRecord::new(
        8_101_000 + seed,
        1001,
        2002,
        account.account_id,
        9001,
        request_id,
    )
    .with_estimated_quantity(42.5)
    .with_expires_at_ms(40)
    .with_created_at_ms(35)
    .with_updated_at_ms(35);
    let ledger_entry = AccountLedgerEntryRecord::new(
        8_201_000 + seed,
        1001,
        2002,
        account.account_id,
        9001,
        AccountLedgerEntryType::HoldCreate,
    )
    .with_request_id(Some(request_id))
    .with_hold_id(Some(hold.hold_id))
    .with_benefit_type(Some("cash_credit".to_owned()))
    .with_quantity(42.5)
    .with_amount(42.5)
    .with_created_at_ms(36);
    let hold_allocation =
        AccountHoldAllocationRecord::new(8_401_000 + seed, 1001, 2002, hold.hold_id, lot.lot_id)
            .with_allocated_quantity(42.5)
            .with_captured_quantity(40.0)
            .with_released_quantity(2.5)
            .with_created_at_ms(36)
            .with_updated_at_ms(41);
    let ledger_allocation = AccountLedgerAllocationRecord::new(
        8_501_000 + seed,
        1001,
        2002,
        ledger_entry.ledger_entry_id,
        lot.lot_id,
    )
    .with_quantity_delta(-40.0)
    .with_created_at_ms(41);
    let fact = RequestMeterFactRecord::new(
        request_id,
        1001,
        2002,
        9001,
        account.account_id,
        "api_key",
        "responses",
        "openai",
        "gpt-4.1",
        "provider-openai-official",
    )
    .with_api_key_id(Some(778899))
    .with_api_key_hash(Some("key_hash_live".to_owned()))
    .with_protocol_family("openai")
    .with_estimated_credit_hold(24.0)
    .with_created_at_ms(35)
    .with_updated_at_ms(36);
    let metric = RequestMeterMetricRecord::new(
        7_001_001 + seed,
        1001,
        2002,
        request_id,
        "token.input",
        128.0,
    )
    .with_provider_field(Some("prompt_tokens".to_owned()))
    .with_captured_at_ms(37);

    store.insert_account_record(&account).await.unwrap();
    store.insert_account_benefit_lot(&lot).await.unwrap();
    store.insert_account_hold(&hold).await.unwrap();
    store
        .insert_account_ledger_entry_record(&ledger_entry)
        .await
        .unwrap();
    store
        .insert_account_hold_allocation(&hold_allocation)
        .await
        .unwrap();
    store
        .insert_account_ledger_allocation(&ledger_allocation)
        .await
        .unwrap();
    store.insert_request_meter_fact(&fact).await.unwrap();
    store.insert_request_meter_metric(&metric).await.unwrap();

    assert!(store
        .list_account_hold_allocations()
        .await
        .unwrap()
        .iter()
        .any(|entry| entry == &hold_allocation));
    assert!(store
        .list_account_ledger_entry_records()
        .await
        .unwrap()
        .iter()
        .any(|entry| entry == &ledger_entry));
    assert!(store
        .list_account_ledger_allocations()
        .await
        .unwrap()
        .iter()
        .any(|entry| entry == &ledger_allocation));
    assert!(store
        .list_request_meter_facts()
        .await
        .unwrap()
        .iter()
        .any(|entry| entry == &fact));
    assert!(store
        .list_request_meter_metrics()
        .await
        .unwrap()
        .iter()
        .any(|entry| entry == &metric));
}
