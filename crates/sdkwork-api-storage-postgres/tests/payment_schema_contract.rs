use sdkwork_api_storage_postgres::payment_migration_statements;

#[test]
fn payment_migrations_declare_canonical_tables() {
    let ddl = payment_migration_statements().join("\n");

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
        "ai_finance_journal_entry",
        "ai_finance_journal_line",
        "ai_payment_reconciliation_batch",
        "ai_payment_reconciliation_line",
    ] {
        assert!(
            ddl.contains(table_name),
            "expected payment postgres migrations to include table {table_name}"
        );
    }
}

#[test]
fn payment_migrations_keep_scope_and_money_columns() {
    let ddl = payment_migration_statements().join("\n");

    for fragment in [
        "tenant_id BIGINT NOT NULL",
        "organization_id BIGINT NOT NULL DEFAULT 0",
        "user_id BIGINT NOT NULL",
        "payable_minor BIGINT NOT NULL DEFAULT 0",
        "captured_amount_minor BIGINT NOT NULL DEFAULT 0",
        "refund_reason_code TEXT NOT NULL DEFAULT ''",
        "requested_by_type TEXT NOT NULL DEFAULT ''",
        "requested_by_id TEXT NOT NULL DEFAULT ''",
        "requested_amount_minor BIGINT NOT NULL DEFAULT 0",
        "payload_json TEXT",
        "amount_minor BIGINT NOT NULL DEFAULT 0",
        "updated_at_ms BIGINT NOT NULL DEFAULT 0",
    ] {
        assert!(
            ddl.contains(fragment),
            "expected payment postgres migrations to include fragment {fragment}"
        );
    }

    assert!(
        ddl.contains(
            "CREATE TABLE IF NOT EXISTS ai_payment_reconciliation_line (\n        reconciliation_line_id TEXT PRIMARY KEY NOT NULL,\n        tenant_id BIGINT NOT NULL,\n        organization_id BIGINT NOT NULL DEFAULT 0,\n        reconciliation_batch_id TEXT NOT NULL,\n        provider_transaction_id TEXT NOT NULL DEFAULT '',\n        payment_order_id TEXT,\n        refund_order_id TEXT,\n        provider_amount_minor BIGINT NOT NULL DEFAULT 0,\n        local_amount_minor BIGINT,\n        match_status TEXT NOT NULL DEFAULT 'matched',\n        reason_code TEXT,\n        created_at_ms BIGINT NOT NULL DEFAULT 0,\n        updated_at_ms BIGINT NOT NULL DEFAULT 0"
        ),
        "expected payment reconciliation table to include updated_at_ms"
    );
}

#[test]
fn payment_migrations_create_operational_indexes() {
    let ddl = payment_migration_statements().join("\n");

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
        assert!(
            ddl.contains(index_name),
            "expected payment postgres migrations to include index {index_name}"
        );
    }
}
