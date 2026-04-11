use super::*;

pub(crate) async fn apply_postgres_commerce_jobs_schema(pool: &PgPool) -> Result<()> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_commerce_orders (
            order_id TEXT PRIMARY KEY NOT NULL,
            project_id TEXT NOT NULL,
            user_id TEXT NOT NULL,
            target_kind TEXT NOT NULL,
            target_id TEXT NOT NULL,
            target_name TEXT NOT NULL,
            list_price_cents BIGINT NOT NULL DEFAULT 0,
            payable_price_cents BIGINT NOT NULL DEFAULT 0,
            list_price_label TEXT NOT NULL DEFAULT '$0.00',
            payable_price_label TEXT NOT NULL DEFAULT '$0.00',
            granted_units BIGINT NOT NULL DEFAULT 0,
            bonus_units BIGINT NOT NULL DEFAULT 0,
            applied_coupon_code TEXT,
            coupon_reservation_id TEXT,
            coupon_redemption_id TEXT,
            marketing_campaign_id TEXT,
            subsidy_amount_minor BIGINT NOT NULL DEFAULT 0,
            status TEXT NOT NULL DEFAULT 'fulfilled',
            source TEXT NOT NULL DEFAULT 'workspace_seed',
            created_at_ms BIGINT NOT NULL DEFAULT 0,
            updated_at_ms BIGINT NOT NULL DEFAULT 0
        )",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_commerce_orders ADD COLUMN IF NOT EXISTS updated_at_ms BIGINT NOT NULL DEFAULT 0",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_commerce_orders ADD COLUMN IF NOT EXISTS coupon_reservation_id TEXT",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_commerce_orders ADD COLUMN IF NOT EXISTS coupon_redemption_id TEXT",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_commerce_orders ADD COLUMN IF NOT EXISTS marketing_campaign_id TEXT",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_commerce_orders ADD COLUMN IF NOT EXISTS subsidy_amount_minor BIGINT NOT NULL DEFAULT 0",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_commerce_orders ADD COLUMN IF NOT EXISTS currency_code TEXT NOT NULL DEFAULT 'USD'",
    )
    .execute(pool)
    .await?;
    sqlx::query("ALTER TABLE ai_commerce_orders ADD COLUMN IF NOT EXISTS pricing_plan_id TEXT")
        .execute(pool)
        .await?;
    sqlx::query(
        "ALTER TABLE ai_commerce_orders ADD COLUMN IF NOT EXISTS pricing_plan_version BIGINT",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_commerce_orders ADD COLUMN IF NOT EXISTS pricing_snapshot_json TEXT NOT NULL DEFAULT '{}'",
    )
    .execute(pool)
    .await?;
    sqlx::query("ALTER TABLE ai_commerce_orders ADD COLUMN IF NOT EXISTS payment_method_id TEXT")
        .execute(pool)
        .await?;
    sqlx::query(
        "ALTER TABLE ai_commerce_orders ADD COLUMN IF NOT EXISTS latest_payment_attempt_id TEXT",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_commerce_orders ADD COLUMN IF NOT EXISTS settlement_status TEXT NOT NULL DEFAULT 'pending'",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_commerce_orders ADD COLUMN IF NOT EXISTS refundable_amount_minor BIGINT NOT NULL DEFAULT 0",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_commerce_orders ADD COLUMN IF NOT EXISTS refunded_amount_minor BIGINT NOT NULL DEFAULT 0",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "UPDATE ai_commerce_orders
         SET updated_at_ms = created_at_ms
         WHERE updated_at_ms = 0",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "UPDATE ai_commerce_orders
         SET currency_code = 'USD'
         WHERE TRIM(COALESCE(currency_code, '')) = ''",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "UPDATE ai_commerce_orders
         SET pricing_snapshot_json = '{}'
         WHERE TRIM(COALESCE(pricing_snapshot_json, '')) = ''",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "UPDATE ai_commerce_orders
         SET settlement_status = CASE
            WHEN status = 'fulfilled' AND payable_price_cents = 0 THEN 'not_required'
            WHEN status = 'fulfilled' THEN 'settled'
            WHEN status = 'refunded' THEN 'refunded'
            WHEN status = 'failed' THEN 'failed'
            WHEN status = 'canceled' THEN 'canceled'
            ELSE 'pending'
         END
         WHERE TRIM(COALESCE(settlement_status, '')) = '' OR settlement_status = 'pending'",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "UPDATE ai_commerce_orders
         SET refundable_amount_minor = CASE
            WHEN status = 'refunded' THEN 0
            ELSE payable_price_cents
         END
         WHERE refundable_amount_minor = 0 AND payable_price_cents > 0",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_commerce_orders_project_created_at
         ON ai_commerce_orders (project_id, created_at_ms DESC, status, order_id)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_commerce_orders_project_updated_at
         ON ai_commerce_orders (project_id, updated_at_ms DESC, status, order_id)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_commerce_orders_user_created_at
         ON ai_commerce_orders (user_id, created_at_ms DESC, status, order_id)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_commerce_orders_user_updated_at
         ON ai_commerce_orders (user_id, updated_at_ms DESC, status, order_id)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_commerce_orders_latest_attempt
         ON ai_commerce_orders (latest_payment_attempt_id)",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_commerce_payment_events (
            payment_event_id TEXT PRIMARY KEY NOT NULL,
            order_id TEXT NOT NULL,
            project_id TEXT NOT NULL,
            user_id TEXT NOT NULL,
            provider TEXT NOT NULL,
            provider_event_id TEXT,
            dedupe_key TEXT NOT NULL,
            event_type TEXT NOT NULL,
            payload_json TEXT NOT NULL DEFAULT '{}',
            processing_status TEXT NOT NULL DEFAULT 'received',
            processing_message TEXT,
            received_at_ms BIGINT NOT NULL DEFAULT 0,
            processed_at_ms BIGINT,
            order_status_after TEXT
        )",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_ai_commerce_payment_events_dedupe_key
         ON ai_commerce_payment_events (dedupe_key)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_commerce_payment_events_order_received_at
         ON ai_commerce_payment_events (order_id, received_at_ms DESC, payment_event_id DESC)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_commerce_payment_events_provider_event
         ON ai_commerce_payment_events (provider, provider_event_id, received_at_ms DESC)",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_project_memberships (
            project_id TEXT PRIMARY KEY NOT NULL,
            membership_id TEXT NOT NULL,
            user_id TEXT NOT NULL,
            plan_id TEXT NOT NULL,
            plan_name TEXT NOT NULL,
            price_cents BIGINT NOT NULL DEFAULT 0,
            price_label TEXT NOT NULL DEFAULT '$0.00',
            cadence TEXT NOT NULL DEFAULT '',
            included_units BIGINT NOT NULL DEFAULT 0,
            status TEXT NOT NULL DEFAULT 'active',
            source TEXT NOT NULL DEFAULT 'workspace_seed',
            activated_at_ms BIGINT NOT NULL DEFAULT 0,
            updated_at_ms BIGINT NOT NULL DEFAULT 0
        )",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_project_memberships_project_updated_at
         ON ai_project_memberships (project_id, updated_at_ms DESC, status)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_project_memberships_user_updated_at
         ON ai_project_memberships (user_id, updated_at_ms DESC, status)",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_payment_methods (
            payment_method_id TEXT PRIMARY KEY NOT NULL,
            display_name TEXT NOT NULL,
            description TEXT NOT NULL DEFAULT '',
            provider TEXT NOT NULL,
            channel TEXT NOT NULL,
            mode TEXT NOT NULL DEFAULT 'live',
            enabled BOOLEAN NOT NULL DEFAULT TRUE,
            sort_order INTEGER NOT NULL DEFAULT 0,
            capability_codes_json TEXT NOT NULL DEFAULT '[]',
            supported_currency_codes_json TEXT NOT NULL DEFAULT '[]',
            supported_country_codes_json TEXT NOT NULL DEFAULT '[]',
            supported_order_kinds_json TEXT NOT NULL DEFAULT '[]',
            callback_strategy TEXT NOT NULL DEFAULT 'webhook_signed',
            webhook_path TEXT,
            webhook_tolerance_seconds BIGINT NOT NULL DEFAULT 300,
            replay_window_seconds BIGINT NOT NULL DEFAULT 300,
            max_retry_count INTEGER NOT NULL DEFAULT 8,
            config_json TEXT NOT NULL DEFAULT '{}',
            created_at_ms BIGINT NOT NULL DEFAULT 0,
            updated_at_ms BIGINT NOT NULL DEFAULT 0
        )",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_payment_methods_provider_enabled_sort
         ON ai_payment_methods (provider, enabled, sort_order, payment_method_id)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_payment_methods_webhook_path
         ON ai_payment_methods (webhook_path)",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_payment_method_credential_bindings (
            binding_id TEXT PRIMARY KEY NOT NULL,
            payment_method_id TEXT NOT NULL,
            usage_kind TEXT NOT NULL,
            credential_tenant_id TEXT NOT NULL,
            credential_provider_id TEXT NOT NULL,
            credential_key_reference TEXT NOT NULL,
            created_at_ms BIGINT NOT NULL DEFAULT 0,
            updated_at_ms BIGINT NOT NULL DEFAULT 0
        )",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_ai_payment_method_credential_bindings_usage
         ON ai_payment_method_credential_bindings (payment_method_id, usage_kind)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_payment_method_credential_bindings_method
         ON ai_payment_method_credential_bindings (payment_method_id, updated_at_ms DESC)",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_commerce_payment_attempts (
            payment_attempt_id TEXT PRIMARY KEY NOT NULL,
            order_id TEXT NOT NULL,
            project_id TEXT NOT NULL,
            user_id TEXT NOT NULL,
            payment_method_id TEXT NOT NULL,
            provider TEXT NOT NULL,
            channel TEXT NOT NULL,
            status TEXT NOT NULL,
            idempotency_key TEXT NOT NULL,
            attempt_sequence INTEGER NOT NULL DEFAULT 1,
            amount_minor BIGINT NOT NULL DEFAULT 0,
            currency_code TEXT NOT NULL DEFAULT 'USD',
            captured_amount_minor BIGINT NOT NULL DEFAULT 0,
            refunded_amount_minor BIGINT NOT NULL DEFAULT 0,
            provider_payment_intent_id TEXT,
            provider_checkout_session_id TEXT,
            provider_reference TEXT,
            checkout_url TEXT,
            qr_code_payload TEXT,
            request_payload_json TEXT NOT NULL DEFAULT '{}',
            response_payload_json TEXT NOT NULL DEFAULT '{}',
            error_code TEXT,
            error_message TEXT,
            initiated_at_ms BIGINT NOT NULL DEFAULT 0,
            expires_at_ms BIGINT,
            completed_at_ms BIGINT,
            updated_at_ms BIGINT NOT NULL DEFAULT 0
        )",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_ai_commerce_payment_attempts_idempotency
         ON ai_commerce_payment_attempts (idempotency_key)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_commerce_payment_attempts_order_updated_at
         ON ai_commerce_payment_attempts (order_id, updated_at_ms DESC, payment_attempt_id DESC)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_commerce_payment_attempts_provider_session
         ON ai_commerce_payment_attempts (provider, provider_checkout_session_id, provider_payment_intent_id)",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_commerce_webhook_inbox (
            webhook_inbox_id TEXT PRIMARY KEY NOT NULL,
            provider TEXT NOT NULL,
            payment_method_id TEXT,
            provider_event_id TEXT,
            dedupe_key TEXT NOT NULL,
            signature_header TEXT,
            payload_json TEXT NOT NULL DEFAULT '{}',
            processing_status TEXT NOT NULL DEFAULT 'received',
            retry_count INTEGER NOT NULL DEFAULT 0,
            max_retry_count INTEGER NOT NULL DEFAULT 8,
            last_error_message TEXT,
            next_retry_at_ms BIGINT,
            first_received_at_ms BIGINT NOT NULL DEFAULT 0,
            last_received_at_ms BIGINT NOT NULL DEFAULT 0,
            processed_at_ms BIGINT
        )",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_ai_commerce_webhook_inbox_dedupe
         ON ai_commerce_webhook_inbox (dedupe_key)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_commerce_webhook_inbox_status_retry
         ON ai_commerce_webhook_inbox (processing_status, next_retry_at_ms, last_received_at_ms DESC)",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_commerce_webhook_delivery_attempts (
            delivery_attempt_id TEXT PRIMARY KEY NOT NULL,
            webhook_inbox_id TEXT NOT NULL,
            processing_status TEXT NOT NULL DEFAULT 'processing',
            response_code INTEGER,
            error_message TEXT,
            started_at_ms BIGINT NOT NULL DEFAULT 0,
            finished_at_ms BIGINT
        )",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_commerce_webhook_delivery_attempts_inbox_started
         ON ai_commerce_webhook_delivery_attempts (webhook_inbox_id, started_at_ms DESC, delivery_attempt_id DESC)",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_commerce_refunds (
            refund_id TEXT PRIMARY KEY NOT NULL,
            order_id TEXT NOT NULL,
            payment_attempt_id TEXT,
            payment_method_id TEXT,
            provider TEXT NOT NULL,
            provider_refund_id TEXT,
            idempotency_key TEXT NOT NULL,
            reason TEXT,
            status TEXT NOT NULL DEFAULT 'requested',
            amount_minor BIGINT NOT NULL DEFAULT 0,
            currency_code TEXT NOT NULL DEFAULT 'USD',
            request_payload_json TEXT NOT NULL DEFAULT '{}',
            response_payload_json TEXT NOT NULL DEFAULT '{}',
            created_at_ms BIGINT NOT NULL DEFAULT 0,
            updated_at_ms BIGINT NOT NULL DEFAULT 0,
            completed_at_ms BIGINT
        )",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_ai_commerce_refunds_idempotency
         ON ai_commerce_refunds (idempotency_key)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_commerce_refunds_order_updated_at
         ON ai_commerce_refunds (order_id, updated_at_ms DESC, refund_id DESC)",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_commerce_reconciliation_runs (
            reconciliation_run_id TEXT PRIMARY KEY NOT NULL,
            provider TEXT NOT NULL,
            payment_method_id TEXT,
            scope_started_at_ms BIGINT NOT NULL DEFAULT 0,
            scope_ended_at_ms BIGINT NOT NULL DEFAULT 0,
            status TEXT NOT NULL DEFAULT 'running',
            summary_json TEXT NOT NULL DEFAULT '{}',
            created_at_ms BIGINT NOT NULL DEFAULT 0,
            updated_at_ms BIGINT NOT NULL DEFAULT 0,
            completed_at_ms BIGINT
        )",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_commerce_reconciliation_runs_provider_created
         ON ai_commerce_reconciliation_runs (provider, created_at_ms DESC, reconciliation_run_id DESC)",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_commerce_reconciliation_items (
            reconciliation_item_id TEXT PRIMARY KEY NOT NULL,
            reconciliation_run_id TEXT NOT NULL,
            order_id TEXT,
            payment_attempt_id TEXT,
            refund_id TEXT,
            external_reference TEXT,
            discrepancy_type TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'open',
            expected_amount_minor BIGINT NOT NULL DEFAULT 0,
            provider_amount_minor BIGINT,
            detail_json TEXT NOT NULL DEFAULT '{}',
            created_at_ms BIGINT NOT NULL DEFAULT 0,
            updated_at_ms BIGINT NOT NULL DEFAULT 0
        )",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_commerce_reconciliation_items_run_status
         ON ai_commerce_reconciliation_items (reconciliation_run_id, status, created_at_ms DESC, reconciliation_item_id DESC)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_catalog_publication_lifecycle_audit (
            audit_id TEXT PRIMARY KEY NOT NULL,
            publication_id TEXT NOT NULL,
            publication_revision_id TEXT NOT NULL,
            publication_version BIGINT NOT NULL,
            publication_source_kind TEXT NOT NULL,
            action TEXT NOT NULL,
            outcome TEXT NOT NULL,
            operator_id TEXT NOT NULL,
            request_id TEXT NOT NULL,
            operator_reason TEXT NOT NULL DEFAULT '',
            publication_status_before TEXT NOT NULL,
            publication_status_after TEXT NOT NULL,
            governed_pricing_plan_id BIGINT,
            governed_pricing_status_before TEXT,
            governed_pricing_status_after TEXT,
            decision_reasons_json TEXT NOT NULL DEFAULT '[]',
            recorded_at_ms BIGINT NOT NULL DEFAULT 0
        )",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_catalog_publication_lifecycle_audit_publication
         ON ai_catalog_publication_lifecycle_audit (publication_id, recorded_at_ms DESC, audit_id DESC)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_catalog_publication_lifecycle_audit_request
         ON ai_catalog_publication_lifecycle_audit (request_id, recorded_at_ms DESC, audit_id DESC)",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_async_jobs (
            job_id TEXT PRIMARY KEY NOT NULL,
            tenant_id BIGINT NOT NULL,
            organization_id BIGINT NOT NULL,
            user_id BIGINT NOT NULL,
            account_id BIGINT,
            request_id BIGINT,
            provider_id TEXT,
            model_code TEXT,
            capability_code TEXT NOT NULL,
            modality TEXT NOT NULL,
            operation_kind TEXT NOT NULL,
            status TEXT NOT NULL,
            external_job_id TEXT,
            idempotency_key TEXT,
            callback_url TEXT,
            input_summary TEXT,
            progress_percent BIGINT,
            error_code TEXT,
            error_message TEXT,
            created_at_ms BIGINT NOT NULL,
            updated_at_ms BIGINT NOT NULL,
            started_at_ms BIGINT,
            completed_at_ms BIGINT
        )",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_async_jobs_scope_created_at
         ON ai_async_jobs (tenant_id, organization_id, user_id, created_at_ms DESC, job_id DESC)",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_async_job_attempts (
            attempt_id BIGINT PRIMARY KEY NOT NULL,
            job_id TEXT NOT NULL,
            attempt_number BIGINT NOT NULL,
            status TEXT NOT NULL,
            runtime_kind TEXT NOT NULL,
            endpoint TEXT,
            external_job_id TEXT,
            claimed_at_ms BIGINT,
            finished_at_ms BIGINT,
            error_message TEXT,
            created_at_ms BIGINT NOT NULL,
            updated_at_ms BIGINT NOT NULL
        )",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_async_job_attempts_job_attempt
         ON ai_async_job_attempts (job_id, attempt_number ASC, attempt_id ASC)",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_async_job_assets (
            asset_id TEXT PRIMARY KEY NOT NULL,
            job_id TEXT NOT NULL,
            asset_kind TEXT NOT NULL,
            storage_key TEXT NOT NULL,
            download_url TEXT,
            mime_type TEXT,
            size_bytes BIGINT,
            checksum_sha256 TEXT,
            created_at_ms BIGINT NOT NULL
        )",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_async_job_assets_job_created_at
         ON ai_async_job_assets (job_id, created_at_ms ASC, asset_id ASC)",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_async_job_callbacks (
            callback_id BIGINT PRIMARY KEY NOT NULL,
            job_id TEXT NOT NULL,
            event_type TEXT NOT NULL,
            dedupe_key TEXT,
            payload_json TEXT NOT NULL,
            status TEXT NOT NULL,
            received_at_ms BIGINT NOT NULL,
            processed_at_ms BIGINT
        )",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_async_job_callbacks_job_received_at
         ON ai_async_job_callbacks (job_id, received_at_ms ASC, callback_id ASC)",
    )
    .execute(pool)
    .await?;

    Ok(())
}
