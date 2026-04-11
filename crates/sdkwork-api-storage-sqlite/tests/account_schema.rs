use sdkwork_api_storage_sqlite::run_migrations;
use sqlx::SqlitePool;

#[tokio::test]
async fn creates_canonical_account_kernel_tables() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();

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
        let row: (String,) =
            sqlx::query_as("select name from sqlite_master where type = 'table' and name = ?")
                .bind(table_name)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(row.0, table_name);
    }
}

#[tokio::test]
async fn canonical_account_kernel_tables_keep_bigint_scope_columns_and_defaults() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();

    assert_has_column(&pool, "ai_account", "tenant_id", "INTEGER", true, None).await;
    assert_has_column(
        &pool,
        "ai_account",
        "organization_id",
        "INTEGER",
        true,
        Some("0"),
    )
    .await;
    assert_has_column(&pool, "ai_account", "user_id", "INTEGER", true, None).await;
    assert_has_column(
        &pool,
        "ai_request_meter_fact",
        "tenant_id",
        "INTEGER",
        true,
        None,
    )
    .await;
    assert_has_column(
        &pool,
        "ai_request_meter_fact",
        "organization_id",
        "INTEGER",
        true,
        Some("0"),
    )
    .await;
    assert_has_column(
        &pool,
        "ai_request_meter_fact",
        "user_id",
        "INTEGER",
        true,
        None,
    )
    .await;
    assert_has_column(
        &pool,
        "ai_request_meter_fact",
        "account_id",
        "INTEGER",
        true,
        None,
    )
    .await;
    assert_has_column(
        &pool,
        "ai_request_settlement",
        "organization_id",
        "INTEGER",
        true,
        Some("0"),
    )
    .await;
    assert_has_column(
        &pool,
        "ai_account_hold_allocation",
        "organization_id",
        "INTEGER",
        true,
        Some("0"),
    )
    .await;
    assert_has_column(
        &pool,
        "ai_account_ledger_allocation",
        "organization_id",
        "INTEGER",
        true,
        Some("0"),
    )
    .await;
}

#[tokio::test]
async fn canonical_account_kernel_tables_create_operational_indexes() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();

    for index_name in [
        "idx_ai_account_user_type",
        "idx_ai_account_benefit_lot_account_status_expiry",
        "idx_ai_account_benefit_lot_account_lot",
        "idx_ai_account_hold_request",
        "idx_ai_account_hold_allocation_hold_lot",
        "idx_ai_request_meter_fact_user_created_at",
        "idx_ai_request_meter_fact_api_key_created_at",
        "idx_ai_request_settlement_request",
        "idx_ai_account_ledger_allocation_ledger_lot",
        "idx_ai_pricing_plan_code_version",
    ] {
        let row: (String,) =
            sqlx::query_as("select name from sqlite_master where type = 'index' and name = ?")
                .bind(index_name)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(row.0, index_name);
    }
}

async fn assert_has_column(
    pool: &SqlitePool,
    table_name: &str,
    column_name: &str,
    declared_type: &str,
    not_null: bool,
    default_value: Option<&str>,
) {
    let pragma = format!("pragma table_info({table_name})");
    let rows: Vec<(i64, String, String, i64, Option<String>, i64)> =
        sqlx::query_as(&pragma).fetch_all(pool).await.unwrap();
    let (_, _, actual_type, actual_not_null, actual_default, _) = rows
        .into_iter()
        .find(|(_, name, _, _, _, _)| name == column_name)
        .unwrap_or_else(|| panic!("missing column {column_name} on table {table_name}"));

    assert_eq!(actual_type, declared_type);
    assert_eq!(actual_not_null == 1, not_null);
    assert_eq!(actual_default.as_deref(), default_value);
}
