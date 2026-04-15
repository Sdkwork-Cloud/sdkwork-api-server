use super::*;

const PAYMENT_MIGRATION_STATEMENTS: &[&str] = &[
    "CREATE TABLE IF NOT EXISTS ai_payment_gateway_account (
        gateway_account_id TEXT PRIMARY KEY NOT NULL,
        tenant_id BIGINT NOT NULL,
        organization_id BIGINT NOT NULL DEFAULT 0,
        provider_code TEXT NOT NULL,
        environment TEXT NOT NULL DEFAULT 'production',
        merchant_id TEXT NOT NULL DEFAULT '',
        app_id TEXT NOT NULL DEFAULT '',
        status TEXT NOT NULL DEFAULT 'active',
        priority BIGINT NOT NULL DEFAULT 0,
        created_at_ms BIGINT NOT NULL DEFAULT 0,
        updated_at_ms BIGINT NOT NULL DEFAULT 0
    )",
    "CREATE INDEX IF NOT EXISTS idx_ai_payment_gateway_account_provider_status_priority
     ON ai_payment_gateway_account (
         tenant_id,
         organization_id,
         provider_code,
         status,
         priority
     )",
    "CREATE TABLE IF NOT EXISTS ai_payment_channel_policy (
        channel_policy_id TEXT PRIMARY KEY NOT NULL,
        tenant_id BIGINT NOT NULL,
        organization_id BIGINT NOT NULL DEFAULT 0,
        scene_code TEXT NOT NULL DEFAULT '',
        country_code TEXT NOT NULL DEFAULT '',
        currency_code TEXT NOT NULL DEFAULT '',
        client_kind TEXT NOT NULL DEFAULT '',
        provider_code TEXT NOT NULL,
        method_code TEXT NOT NULL,
        priority BIGINT NOT NULL DEFAULT 0,
        status TEXT NOT NULL DEFAULT 'active',
        created_at_ms BIGINT NOT NULL DEFAULT 0,
        updated_at_ms BIGINT NOT NULL DEFAULT 0
    )",
    "CREATE TABLE IF NOT EXISTS ai_payment_order (
        payment_order_id TEXT PRIMARY KEY NOT NULL,
        tenant_id BIGINT NOT NULL,
        organization_id BIGINT NOT NULL DEFAULT 0,
        user_id BIGINT NOT NULL,
        commerce_order_id TEXT NOT NULL,
        project_id TEXT NOT NULL DEFAULT '',
        order_kind TEXT NOT NULL DEFAULT '',
        subject_type TEXT NOT NULL DEFAULT '',
        subject_id TEXT NOT NULL DEFAULT '',
        currency_code TEXT NOT NULL DEFAULT 'USD',
        amount_minor BIGINT NOT NULL DEFAULT 0,
        discount_minor BIGINT NOT NULL DEFAULT 0,
        subsidy_minor BIGINT NOT NULL DEFAULT 0,
        payable_minor BIGINT NOT NULL DEFAULT 0,
        captured_amount_minor BIGINT NOT NULL DEFAULT 0,
        provider_code TEXT NOT NULL DEFAULT 'unspecified',
        method_code TEXT,
        payment_status TEXT NOT NULL DEFAULT 'created',
        fulfillment_status TEXT NOT NULL DEFAULT 'pending',
        refund_status TEXT NOT NULL DEFAULT 'not_requested',
        quote_snapshot_json TEXT,
        metadata_json TEXT,
        version BIGINT NOT NULL DEFAULT 0,
        created_at_ms BIGINT NOT NULL DEFAULT 0,
        updated_at_ms BIGINT NOT NULL DEFAULT 0
    )",
    "ALTER TABLE ai_payment_order
     ADD COLUMN IF NOT EXISTS captured_amount_minor BIGINT NOT NULL DEFAULT 0",
    "CREATE INDEX IF NOT EXISTS idx_ai_payment_order_user_created_at
     ON ai_payment_order (tenant_id, organization_id, user_id, created_at_ms DESC)",
    "CREATE INDEX IF NOT EXISTS idx_ai_payment_order_provider_status_updated_at
     ON ai_payment_order (
         tenant_id,
         organization_id,
         provider_code,
         payment_status,
         updated_at_ms DESC
     )",
    "CREATE TABLE IF NOT EXISTS ai_payment_attempt (
        payment_attempt_id TEXT PRIMARY KEY NOT NULL,
        tenant_id BIGINT NOT NULL,
        organization_id BIGINT NOT NULL DEFAULT 0,
        payment_order_id TEXT NOT NULL,
        attempt_no BIGINT NOT NULL DEFAULT 1,
        gateway_account_id TEXT NOT NULL DEFAULT '',
        provider_code TEXT NOT NULL DEFAULT 'unspecified',
        method_code TEXT NOT NULL DEFAULT '',
        client_kind TEXT NOT NULL DEFAULT '',
        idempotency_key TEXT NOT NULL,
        provider_request_id TEXT,
        provider_payment_reference TEXT,
        attempt_status TEXT NOT NULL DEFAULT 'initiated',
        request_payload_hash TEXT NOT NULL DEFAULT '',
        expires_at_ms BIGINT,
        created_at_ms BIGINT NOT NULL DEFAULT 0,
        updated_at_ms BIGINT NOT NULL DEFAULT 0
    )",
    "CREATE INDEX IF NOT EXISTS idx_ai_payment_attempt_order_attempt
     ON ai_payment_attempt (
         tenant_id,
         organization_id,
         payment_order_id,
         attempt_no DESC
     )",
    "CREATE TABLE IF NOT EXISTS ai_payment_session (
        payment_session_id TEXT PRIMARY KEY NOT NULL,
        tenant_id BIGINT NOT NULL,
        organization_id BIGINT NOT NULL DEFAULT 0,
        payment_attempt_id TEXT NOT NULL,
        session_kind TEXT NOT NULL DEFAULT 'redirect',
        session_status TEXT NOT NULL DEFAULT 'open',
        display_reference TEXT,
        qr_payload TEXT,
        redirect_url TEXT,
        expires_at_ms BIGINT NOT NULL DEFAULT 0,
        created_at_ms BIGINT NOT NULL DEFAULT 0,
        updated_at_ms BIGINT NOT NULL DEFAULT 0
    )",
    "CREATE INDEX IF NOT EXISTS idx_ai_payment_session_attempt
     ON ai_payment_session (tenant_id, organization_id, payment_attempt_id)",
    "CREATE TABLE IF NOT EXISTS ai_payment_transaction (
        payment_transaction_id TEXT PRIMARY KEY NOT NULL,
        tenant_id BIGINT NOT NULL,
        organization_id BIGINT NOT NULL DEFAULT 0,
        payment_order_id TEXT NOT NULL,
        payment_attempt_id TEXT,
        transaction_kind TEXT NOT NULL DEFAULT 'sale',
        provider_code TEXT NOT NULL DEFAULT 'unspecified',
        provider_transaction_id TEXT NOT NULL,
        currency_code TEXT NOT NULL DEFAULT 'USD',
        amount_minor BIGINT NOT NULL DEFAULT 0,
        fee_minor BIGINT,
        net_amount_minor BIGINT,
        provider_status TEXT NOT NULL DEFAULT 'pending',
        raw_event_id TEXT,
        occurred_at_ms BIGINT NOT NULL DEFAULT 0,
        created_at_ms BIGINT NOT NULL DEFAULT 0
    )",
    "CREATE INDEX IF NOT EXISTS idx_ai_payment_transaction_order_occurred_at
     ON ai_payment_transaction (
         tenant_id,
         organization_id,
         payment_order_id,
         occurred_at_ms DESC
     )",
    "CREATE TABLE IF NOT EXISTS ai_payment_callback_event (
        callback_event_id TEXT PRIMARY KEY NOT NULL,
        tenant_id BIGINT NOT NULL,
        organization_id BIGINT NOT NULL DEFAULT 0,
        provider_code TEXT NOT NULL DEFAULT 'unspecified',
        gateway_account_id TEXT NOT NULL DEFAULT '',
        event_type TEXT NOT NULL DEFAULT '',
        event_identity TEXT NOT NULL DEFAULT '',
        dedupe_key TEXT NOT NULL,
        payment_order_id TEXT,
        payment_attempt_id TEXT,
        provider_transaction_id TEXT,
        signature_status TEXT NOT NULL DEFAULT 'pending',
        processing_status TEXT NOT NULL DEFAULT 'pending',
        payload_json TEXT,
        received_at_ms BIGINT NOT NULL DEFAULT 0,
        processed_at_ms BIGINT
    )",
    "CREATE INDEX IF NOT EXISTS idx_ai_payment_callback_event_processing_received_at
     ON ai_payment_callback_event (
         tenant_id,
         organization_id,
         processing_status,
         received_at_ms DESC
     )",
    "CREATE UNIQUE INDEX IF NOT EXISTS idx_ai_payment_callback_event_provider_dedupe
     ON ai_payment_callback_event (
         provider_code,
         gateway_account_id,
         dedupe_key
     )",
    "CREATE TABLE IF NOT EXISTS ai_refund_order (
        refund_order_id TEXT PRIMARY KEY NOT NULL,
        tenant_id BIGINT NOT NULL,
        organization_id BIGINT NOT NULL DEFAULT 0,
        payment_order_id TEXT NOT NULL,
        commerce_order_id TEXT NOT NULL DEFAULT '',
        refund_reason_code TEXT NOT NULL DEFAULT '',
        requested_by_type TEXT NOT NULL DEFAULT '',
        requested_by_id TEXT NOT NULL DEFAULT '',
        currency_code TEXT NOT NULL DEFAULT 'USD',
        requested_amount_minor BIGINT NOT NULL DEFAULT 0,
        approved_amount_minor BIGINT,
        refunded_amount_minor BIGINT NOT NULL DEFAULT 0,
        refund_status TEXT NOT NULL DEFAULT 'requested',
        created_at_ms BIGINT NOT NULL DEFAULT 0,
        updated_at_ms BIGINT NOT NULL DEFAULT 0
    )",
    "ALTER TABLE ai_refund_order
     ADD COLUMN IF NOT EXISTS refund_reason_code TEXT NOT NULL DEFAULT ''",
    "ALTER TABLE ai_refund_order
     ADD COLUMN IF NOT EXISTS requested_by_type TEXT NOT NULL DEFAULT ''",
    "ALTER TABLE ai_refund_order
     ADD COLUMN IF NOT EXISTS requested_by_id TEXT NOT NULL DEFAULT ''",
    "CREATE INDEX IF NOT EXISTS idx_ai_refund_order_payment_created_at
     ON ai_refund_order (
         tenant_id,
         organization_id,
         payment_order_id,
         created_at_ms DESC
     )",
    "CREATE TABLE IF NOT EXISTS ai_refund_attempt (
        refund_attempt_id TEXT PRIMARY KEY NOT NULL,
        tenant_id BIGINT NOT NULL,
        organization_id BIGINT NOT NULL DEFAULT 0,
        refund_order_id TEXT NOT NULL,
        attempt_no BIGINT NOT NULL DEFAULT 1,
        provider_refund_id TEXT,
        provider_code TEXT NOT NULL DEFAULT 'unspecified',
        attempt_status TEXT NOT NULL DEFAULT 'requested',
        created_at_ms BIGINT NOT NULL DEFAULT 0,
        updated_at_ms BIGINT NOT NULL DEFAULT 0
    )",
    "CREATE TABLE IF NOT EXISTS ai_refund_order_processing_steps (
        refund_order_id TEXT NOT NULL,
        step_key TEXT NOT NULL,
        applied_at_ms BIGINT NOT NULL DEFAULT 0,
        PRIMARY KEY (refund_order_id, step_key)
    )",
    "CREATE TABLE IF NOT EXISTS ai_finance_journal_entry (
        finance_journal_entry_id TEXT PRIMARY KEY NOT NULL,
        tenant_id BIGINT NOT NULL,
        organization_id BIGINT NOT NULL DEFAULT 0,
        source_kind TEXT NOT NULL DEFAULT '',
        source_id TEXT NOT NULL DEFAULT '',
        entry_code TEXT NOT NULL DEFAULT '',
        currency_code TEXT NOT NULL DEFAULT 'USD',
        entry_status TEXT NOT NULL DEFAULT 'draft',
        occurred_at_ms BIGINT NOT NULL DEFAULT 0,
        created_at_ms BIGINT NOT NULL DEFAULT 0
    )",
    "CREATE INDEX IF NOT EXISTS idx_ai_finance_journal_entry_source
     ON ai_finance_journal_entry (tenant_id, organization_id, source_kind, source_id)",
    "CREATE TABLE IF NOT EXISTS ai_finance_journal_line (
        finance_journal_line_id TEXT PRIMARY KEY NOT NULL,
        tenant_id BIGINT NOT NULL,
        organization_id BIGINT NOT NULL DEFAULT 0,
        finance_journal_entry_id TEXT NOT NULL,
        line_no BIGINT NOT NULL DEFAULT 0,
        account_code TEXT NOT NULL DEFAULT '',
        direction TEXT NOT NULL DEFAULT 'debit',
        amount_minor BIGINT NOT NULL DEFAULT 0,
        party_type TEXT,
        party_id TEXT
    )",
    "CREATE TABLE IF NOT EXISTS ai_payment_reconciliation_batch (
        reconciliation_batch_id TEXT PRIMARY KEY NOT NULL,
        tenant_id BIGINT NOT NULL,
        organization_id BIGINT NOT NULL DEFAULT 0,
        provider_code TEXT NOT NULL DEFAULT 'unspecified',
        gateway_account_id TEXT NOT NULL DEFAULT '',
        artifact_date TEXT NOT NULL DEFAULT '',
        import_status TEXT NOT NULL DEFAULT 'pending',
        created_at_ms BIGINT NOT NULL DEFAULT 0
    )",
    "CREATE TABLE IF NOT EXISTS ai_payment_reconciliation_line (
        reconciliation_line_id TEXT PRIMARY KEY NOT NULL,
        tenant_id BIGINT NOT NULL,
        organization_id BIGINT NOT NULL DEFAULT 0,
        reconciliation_batch_id TEXT NOT NULL,
        provider_transaction_id TEXT NOT NULL DEFAULT '',
        payment_order_id TEXT,
        refund_order_id TEXT,
        provider_amount_minor BIGINT NOT NULL DEFAULT 0,
        local_amount_minor BIGINT,
        match_status TEXT NOT NULL DEFAULT 'matched',
        reason_code TEXT,
        created_at_ms BIGINT NOT NULL DEFAULT 0,
        updated_at_ms BIGINT NOT NULL DEFAULT 0
    )",
    "ALTER TABLE ai_payment_reconciliation_line
     ADD COLUMN IF NOT EXISTS updated_at_ms BIGINT NOT NULL DEFAULT 0",
    "CREATE INDEX IF NOT EXISTS idx_ai_payment_reconciliation_line_batch_status
     ON ai_payment_reconciliation_line (
         tenant_id,
         organization_id,
         reconciliation_batch_id,
         match_status
     )",
];

pub fn payment_migration_statements() -> &'static [&'static str] {
    PAYMENT_MIGRATION_STATEMENTS
}

pub(crate) async fn apply_postgres_payment_schema(pool: &PgPool) -> Result<()> {
    for statement in PAYMENT_MIGRATION_STATEMENTS {
        sqlx::query(statement).execute(pool).await?;
    }
    Ok(())
}
