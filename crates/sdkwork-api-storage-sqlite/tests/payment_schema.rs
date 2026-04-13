use sdkwork_api_storage_sqlite::run_migrations;
use sqlx::SqlitePool;

#[tokio::test]
async fn creates_canonical_payment_kernel_tables() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();

    for table_name in [
        "ai_payment_gateway_account",
        "ai_payment_channel_policy",
        "ai_payment_order",
        "ai_payment_attempt",
        "ai_payment_session",
        "ai_payment_transaction",
        "ai_payment_callback_event",
        "ai_refund_order",
        "ai_refund_attempt",
        "ai_refund_order_processing_steps",
        "ai_finance_journal_entry",
        "ai_finance_journal_line",
        "ai_payment_reconciliation_batch",
        "ai_payment_reconciliation_line",
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
async fn payment_kernel_tables_keep_scope_and_money_columns() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();

    assert_has_column(
        &pool,
        "ai_payment_order",
        "tenant_id",
        "INTEGER",
        true,
        None,
    )
    .await;
    assert_has_column(
        &pool,
        "ai_payment_order",
        "organization_id",
        "INTEGER",
        true,
        Some("0"),
    )
    .await;
    assert_has_column(&pool, "ai_payment_order", "user_id", "INTEGER", true, None).await;
    assert_has_column(
        &pool,
        "ai_payment_order",
        "payable_minor",
        "INTEGER",
        true,
        Some("0"),
    )
    .await;
    assert_has_column(
        &pool,
        "ai_payment_order",
        "captured_amount_minor",
        "INTEGER",
        true,
        Some("0"),
    )
    .await;
    assert_has_column(
        &pool,
        "ai_refund_order",
        "refund_reason_code",
        "TEXT",
        true,
        Some("''"),
    )
    .await;
    assert_has_column(
        &pool,
        "ai_refund_order",
        "requested_by_type",
        "TEXT",
        true,
        Some("''"),
    )
    .await;
    assert_has_column(
        &pool,
        "ai_refund_order",
        "requested_by_id",
        "TEXT",
        true,
        Some("''"),
    )
    .await;
    assert_has_column(
        &pool,
        "ai_refund_order",
        "requested_amount_minor",
        "INTEGER",
        true,
        Some("0"),
    )
    .await;
    assert_has_column(
        &pool,
        "ai_finance_journal_line",
        "amount_minor",
        "INTEGER",
        true,
        Some("0"),
    )
    .await;
    assert_has_column(
        &pool,
        "ai_payment_reconciliation_line",
        "updated_at_ms",
        "INTEGER",
        true,
        Some("0"),
    )
    .await;
}

#[tokio::test]
async fn payment_kernel_tables_create_operational_indexes() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();

    for index_name in [
        "idx_ai_payment_gateway_account_provider_status_priority",
        "idx_ai_payment_order_user_created_at",
        "idx_ai_payment_order_provider_status_updated_at",
        "idx_ai_payment_attempt_order_attempt",
        "idx_ai_payment_session_attempt",
        "idx_ai_payment_transaction_order_occurred_at",
        "idx_ai_payment_callback_event_processing_received_at",
        "idx_ai_refund_order_payment_created_at",
        "idx_ai_finance_journal_entry_source",
        "idx_ai_payment_reconciliation_line_batch_status",
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
