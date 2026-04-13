use sdkwork_api_app_billing::{
    plan_account_hold, resolve_payable_account_for_gateway_request_context,
    resolve_payable_account_for_gateway_subject, summarize_account_balance,
};
use sdkwork_api_app_identity::gateway_auth_subject_from_request_context;
use sdkwork_api_domain_billing::{
    AccountBenefitLotRecord, AccountBenefitSourceType, AccountBenefitType, AccountRecord,
    AccountStatus, AccountType,
};
use sdkwork_api_domain_identity::GatewayAuthSubject;
use sdkwork_api_storage_core::AccountKernelStore;
use sdkwork_api_storage_sqlite::{run_migrations, SqliteAdminStore};

#[tokio::test]
async fn summarizes_account_balance_from_canonical_lots() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    let account = AccountRecord::new(7001, 1001, 2002, 9001, AccountType::Primary)
        .with_created_at_ms(10)
        .with_updated_at_ms(10);
    let expiring_promo = AccountBenefitLotRecord::new(
        8001,
        1001,
        2002,
        7001,
        9001,
        AccountBenefitType::PromoCredit,
    )
    .with_source_type(AccountBenefitSourceType::Coupon)
    .with_original_quantity(100.0)
    .with_remaining_quantity(80.0)
    .with_held_quantity(10.0)
    .with_expires_at_ms(Some(120))
    .with_created_at_ms(11)
    .with_updated_at_ms(11);
    let cash_credit =
        AccountBenefitLotRecord::new(8002, 1001, 2002, 7001, 9001, AccountBenefitType::CashCredit)
            .with_source_type(AccountBenefitSourceType::Recharge)
            .with_original_quantity(200.0)
            .with_remaining_quantity(150.0)
            .with_held_quantity(20.0)
            .with_created_at_ms(12)
            .with_updated_at_ms(12);

    store.insert_account_record(&account).await.unwrap();
    store
        .insert_account_benefit_lot(&expiring_promo)
        .await
        .unwrap();
    store
        .insert_account_benefit_lot(&cash_credit)
        .await
        .unwrap();

    let snapshot = summarize_account_balance(&store, 7001, 100).await.unwrap();

    assert_eq!(snapshot.account_id, 7001);
    assert_eq!(snapshot.grant_balance, 300.0);
    assert_eq!(snapshot.consumed_balance, 70.0);
    assert_eq!(snapshot.held_balance, 30.0);
    assert_eq!(snapshot.available_balance, 200.0);
    assert_eq!(snapshot.active_lot_count, 2);
    assert_eq!(
        snapshot
            .lots
            .iter()
            .map(|lot| lot.lot_id)
            .collect::<Vec<_>>(),
        vec![8001, 8002]
    );
}

#[tokio::test]
async fn plans_hold_across_lots_in_spend_order_and_reports_shortfall() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    let account = AccountRecord::new(7101, 1001, 2002, 9001, AccountType::Primary)
        .with_created_at_ms(10)
        .with_updated_at_ms(10);
    let scoped_allowance = AccountBenefitLotRecord::new(
        8103,
        1001,
        2002,
        7101,
        9001,
        AccountBenefitType::RequestAllowance,
    )
    .with_source_type(AccountBenefitSourceType::Grant)
    .with_scope_json(Some("{\"model\":\"gpt-4.1\"}".to_owned()))
    .with_original_quantity(20.0)
    .with_remaining_quantity(20.0)
    .with_held_quantity(0.0)
    .with_expires_at_ms(Some(200))
    .with_created_at_ms(11)
    .with_updated_at_ms(11);
    let promo_credit = AccountBenefitLotRecord::new(
        8102,
        1001,
        2002,
        7101,
        9001,
        AccountBenefitType::PromoCredit,
    )
    .with_source_type(AccountBenefitSourceType::Coupon)
    .with_original_quantity(40.0)
    .with_remaining_quantity(35.0)
    .with_held_quantity(5.0)
    .with_expires_at_ms(Some(200))
    .with_created_at_ms(12)
    .with_updated_at_ms(12);
    let cash_credit =
        AccountBenefitLotRecord::new(8101, 1001, 2002, 7101, 9001, AccountBenefitType::CashCredit)
            .with_source_type(AccountBenefitSourceType::Recharge)
            .with_original_quantity(90.0)
            .with_remaining_quantity(80.0)
            .with_held_quantity(0.0)
            .with_created_at_ms(13)
            .with_updated_at_ms(13);

    store.insert_account_record(&account).await.unwrap();
    store
        .insert_account_benefit_lot(&cash_credit)
        .await
        .unwrap();
    store
        .insert_account_benefit_lot(&promo_credit)
        .await
        .unwrap();
    store
        .insert_account_benefit_lot(&scoped_allowance)
        .await
        .unwrap();

    let plan = plan_account_hold(&store, 7101, 150.0, 100).await.unwrap();

    assert_eq!(plan.account_id, 7101);
    assert_eq!(plan.requested_quantity, 150.0);
    assert_eq!(plan.covered_quantity, 130.0);
    assert_eq!(plan.shortfall_quantity, 20.0);
    assert!(!plan.sufficient_balance);
    assert_eq!(
        plan.allocations
            .iter()
            .map(|allocation| (allocation.lot_id, allocation.quantity))
            .collect::<Vec<_>>(),
        vec![(8103, 20.0), (8102, 30.0), (8101, 80.0)]
    );
}

#[tokio::test]
async fn resolves_active_primary_account_for_gateway_auth_subject() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    let primary_account = AccountRecord::new(7301, 1001, 2002, 9001, AccountType::Primary)
        .with_created_at_ms(10)
        .with_updated_at_ms(10);
    let grant_account = AccountRecord::new(7302, 1001, 2002, 9001, AccountType::Grant)
        .with_created_at_ms(11)
        .with_updated_at_ms(11);
    store.insert_account_record(&primary_account).await.unwrap();
    store.insert_account_record(&grant_account).await.unwrap();

    let subject = GatewayAuthSubject::for_api_key(1001, 2002, 9001, 778899, "hash_live");
    let resolved = resolve_payable_account_for_gateway_subject(&store, &subject)
        .await
        .unwrap();

    assert_eq!(resolved, Some(primary_account));
}

#[tokio::test]
async fn payable_account_resolution_returns_none_when_primary_account_is_missing() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    let subject = GatewayAuthSubject::for_api_key(1001, 2002, 9001, 778899, "hash_live");
    let resolved = resolve_payable_account_for_gateway_subject(&store, &subject)
        .await
        .unwrap();

    assert_eq!(resolved, None);
}

#[tokio::test]
async fn payable_account_resolution_rejects_inactive_primary_accounts() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    let suspended_account = AccountRecord::new(7401, 1001, 2002, 9001, AccountType::Primary)
        .with_status(AccountStatus::Suspended)
        .with_created_at_ms(10)
        .with_updated_at_ms(10);
    store
        .insert_account_record(&suspended_account)
        .await
        .unwrap();

    let subject = GatewayAuthSubject::for_api_key(1001, 2002, 9001, 778899, "hash_live");
    let error = resolve_payable_account_for_gateway_subject(&store, &subject)
        .await
        .unwrap_err();

    assert_eq!(error.to_string(), "primary account 7401 is not active");
}

#[tokio::test]
async fn resolves_active_primary_account_for_gateway_request_context_using_gateway_project_principal(
) {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    let request_context = sdkwork_api_app_identity::GatewayRequestContext {
        tenant_id: "tenant-gateway-commercial".to_owned(),
        project_id: "project-gateway-commercial".to_owned(),
        environment: "live".to_owned(),
        api_key_hash: "hash_live_gateway_project".to_owned(),
        api_key_group_id: Some("group-live".to_owned()),
        canonical_tenant_id: None,
        canonical_organization_id: None,
        canonical_user_id: None,
        canonical_api_key_id: None,
    };
    let derived_subject = gateway_auth_subject_from_request_context(&request_context);

    let primary_account = AccountRecord::new(
        7501,
        derived_subject.tenant_id,
        derived_subject.organization_id,
        derived_subject.user_id,
        AccountType::Primary,
    )
    .with_created_at_ms(10)
    .with_updated_at_ms(10);
    store.insert_account_record(&primary_account).await.unwrap();

    let resolved = resolve_payable_account_for_gateway_request_context(&store, &request_context)
        .await
        .unwrap();

    assert_eq!(resolved, Some(primary_account));
}

#[tokio::test]
async fn payable_account_resolution_from_gateway_request_context_returns_none_without_matching_account(
) {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    let request_context = sdkwork_api_app_identity::GatewayRequestContext {
        tenant_id: "tenant-gateway-commercial".to_owned(),
        project_id: "project-gateway-commercial".to_owned(),
        environment: "live".to_owned(),
        api_key_hash: "hash_live_gateway_project".to_owned(),
        api_key_group_id: Some("group-live".to_owned()),
        canonical_tenant_id: None,
        canonical_organization_id: None,
        canonical_user_id: None,
        canonical_api_key_id: None,
    };

    let resolved = resolve_payable_account_for_gateway_request_context(&store, &request_context)
        .await
        .unwrap();

    assert_eq!(resolved, None);
}
