use super::*;

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

#[tokio::test]
async fn postgres_account_kernel_transaction_round_trips_hold_and_settlement_when_url_is_provided()
{
    let Some(database_url) = std::env::var("SDKWORK_TEST_POSTGRES_URL").ok() else {
        return;
    };

    let pool = run_migrations(&database_url).await.unwrap();
    let store = PostgresAdminStore::new(pool.clone());
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    let account_id = 8_000_000 + seed;
    let lot_id = 8_100_000 + seed;
    let hold_id = 8_200_000 + seed;
    let hold_allocation_id = 8_300_000 + seed;
    let settlement_id = 8_400_000 + seed;
    let request_id = 8_500_000 + seed;

    let account = AccountRecord::new(account_id, 1001, 2002, 9001, AccountType::Primary)
        .with_created_at_ms(10)
        .with_updated_at_ms(10);
    sqlx::query(
        "INSERT INTO ai_account (
            account_id, tenant_id, organization_id, user_id, account_type,
            currency_code, credit_unit_code, status, allow_overdraft, overdraft_limit,
            created_at_ms, updated_at_ms
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)",
    )
    .bind(i64::try_from(account.account_id).unwrap())
    .bind(i64::try_from(account.tenant_id).unwrap())
    .bind(i64::try_from(account.organization_id).unwrap())
    .bind(i64::try_from(account.user_id).unwrap())
    .bind("primary")
    .bind(&account.currency_code)
    .bind(&account.credit_unit_code)
    .bind("active")
    .bind(account.allow_overdraft)
    .bind(account.overdraft_limit)
    .bind(i64::try_from(account.created_at_ms).unwrap())
    .bind(i64::try_from(account.updated_at_ms).unwrap())
    .execute(&pool)
    .await
    .unwrap();

    let lot = AccountBenefitLotRecord::new(
        lot_id,
        account.tenant_id,
        account.organization_id,
        account.account_id,
        account.user_id,
        AccountBenefitType::CashCredit,
    )
    .with_source_type(AccountBenefitSourceType::Recharge)
    .with_original_quantity(100.0)
    .with_remaining_quantity(100.0)
    .with_created_at_ms(20)
    .with_updated_at_ms(20);
    sqlx::query(
        "INSERT INTO ai_account_benefit_lot (
            lot_id, tenant_id, organization_id, account_id, user_id, benefit_type,
            source_type, source_id, scope_json, original_quantity, remaining_quantity,
            held_quantity, priority, acquired_unit_cost, issued_at_ms, expires_at_ms, status,
            created_at_ms, updated_at_ms
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, NULL, NULL, $8, $9, $10, $11, NULL, $12, NULL, $13, $14, $15)",
    )
    .bind(i64::try_from(lot.lot_id).unwrap())
    .bind(i64::try_from(lot.tenant_id).unwrap())
    .bind(i64::try_from(lot.organization_id).unwrap())
    .bind(i64::try_from(lot.account_id).unwrap())
    .bind(i64::try_from(lot.user_id).unwrap())
    .bind("cash_credit")
    .bind("recharge")
    .bind(lot.original_quantity)
    .bind(lot.remaining_quantity)
    .bind(lot.held_quantity)
    .bind(lot.priority)
    .bind(i64::try_from(lot.issued_at_ms).unwrap())
    .bind("active")
    .bind(i64::try_from(lot.created_at_ms).unwrap())
    .bind(i64::try_from(lot.updated_at_ms).unwrap())
    .execute(&pool)
    .await
    .unwrap();

    store
        .with_account_kernel_transaction(|tx| {
            Box::pin(async move {
                let seeded_account = tx.find_account_record(account_id).await?.unwrap();
                let seeded_lot = tx.find_account_benefit_lot(lot_id).await?.unwrap();

                tx.upsert_account_benefit_lot(
                    &seeded_lot
                        .clone()
                        .with_held_quantity(40.0)
                        .with_updated_at_ms(35),
                )
                .await?;

                let hold = AccountHoldRecord::new(
                    hold_id,
                    seeded_account.tenant_id,
                    seeded_account.organization_id,
                    seeded_account.account_id,
                    seeded_account.user_id,
                    request_id,
                )
                .with_estimated_quantity(40.0)
                .with_expires_at_ms(120)
                .with_created_at_ms(35)
                .with_updated_at_ms(35);
                tx.upsert_account_hold(&hold).await?;

                let allocation = AccountHoldAllocationRecord::new(
                    hold_allocation_id,
                    seeded_account.tenant_id,
                    seeded_account.organization_id,
                    hold_id,
                    lot_id,
                )
                .with_allocated_quantity(40.0)
                .with_created_at_ms(35)
                .with_updated_at_ms(35);
                tx.upsert_account_hold_allocation(&allocation).await?;

                let settlement = RequestSettlementRecord::new(
                    settlement_id,
                    seeded_account.tenant_id,
                    seeded_account.organization_id,
                    request_id,
                    seeded_account.account_id,
                    seeded_account.user_id,
                )
                .with_hold_id(Some(hold_id))
                .with_status(RequestSettlementStatus::Captured)
                .with_estimated_credit_hold(40.0)
                .with_captured_credit_amount(40.0)
                .with_retail_charge_amount(40.0)
                .with_settled_at_ms(36)
                .with_created_at_ms(36)
                .with_updated_at_ms(36);
                tx.upsert_request_settlement_record(&settlement).await?;

                Ok(())
            })
        })
        .await
        .unwrap();

    let verified = store
        .with_account_kernel_transaction(|tx| {
            Box::pin(async move {
                let hold = tx
                    .find_account_hold_by_request_id(request_id)
                    .await?
                    .unwrap();
                let allocations = tx
                    .list_account_hold_allocations_for_hold(hold.hold_id)
                    .await?;
                let lot = tx.find_account_benefit_lot(lot_id).await?.unwrap();
                let settlement = tx
                    .find_request_settlement_by_request_id(request_id)
                    .await?
                    .unwrap();
                Ok((hold, allocations, lot, settlement))
            })
        })
        .await
        .unwrap();

    assert_eq!(verified.0.hold_id, hold_id);
    assert_eq!(verified.1.len(), 1);
    assert_eq!(verified.1[0].allocated_quantity, 40.0);
    assert_eq!(verified.2.held_quantity, 40.0);
    assert_eq!(verified.3.request_settlement_id, settlement_id);

    let existing = store
        .with_account_kernel_transaction(|tx| {
            Box::pin(async move {
                Ok(tx
                    .find_request_settlement_by_request_id(request_id)
                    .await?
                    .unwrap())
            })
        })
        .await
        .unwrap();
    assert_eq!(existing.request_settlement_id, settlement_id);
}
