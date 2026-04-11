use super::*;

pub(crate) async fn apply_postgres_billing_schema(pool: &PgPool) -> Result<()> {
    let pool = pool.clone();
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_account (
            account_id BIGINT PRIMARY KEY NOT NULL,
            tenant_id BIGINT NOT NULL,
            organization_id BIGINT NOT NULL DEFAULT 0,
            user_id BIGINT NOT NULL,
            account_type TEXT NOT NULL,
            currency_code TEXT NOT NULL DEFAULT 'USD',
            credit_unit_code TEXT NOT NULL DEFAULT 'credit',
            status TEXT NOT NULL DEFAULT 'active',
            allow_overdraft BOOLEAN NOT NULL DEFAULT FALSE,
            overdraft_limit DOUBLE PRECISION NOT NULL DEFAULT 0,
            created_at_ms BIGINT NOT NULL DEFAULT 0,
            updated_at_ms BIGINT NOT NULL DEFAULT 0
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_ai_account_user_type
         ON ai_account (tenant_id, organization_id, user_id, account_type)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_account_benefit_lot (
            lot_id BIGINT PRIMARY KEY NOT NULL,
            tenant_id BIGINT NOT NULL,
            organization_id BIGINT NOT NULL DEFAULT 0,
            account_id BIGINT NOT NULL,
            user_id BIGINT NOT NULL,
            benefit_type TEXT NOT NULL,
            source_type TEXT NOT NULL,
            source_id BIGINT,
            scope_json TEXT,
            original_quantity DOUBLE PRECISION NOT NULL DEFAULT 0,
            remaining_quantity DOUBLE PRECISION NOT NULL DEFAULT 0,
            held_quantity DOUBLE PRECISION NOT NULL DEFAULT 0,
            priority INTEGER NOT NULL DEFAULT 0,
            acquired_unit_cost DOUBLE PRECISION,
            issued_at_ms BIGINT NOT NULL DEFAULT 0,
            expires_at_ms BIGINT,
            status TEXT NOT NULL DEFAULT 'active',
            created_at_ms BIGINT NOT NULL DEFAULT 0,
            updated_at_ms BIGINT NOT NULL DEFAULT 0
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_account_benefit_lot_account_status_expiry
         ON ai_account_benefit_lot (tenant_id, organization_id, account_id, status, expires_at_ms)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_account_benefit_lot_account_lot
         ON ai_account_benefit_lot (account_id, lot_id)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_account_hold (
            hold_id BIGINT PRIMARY KEY NOT NULL,
            tenant_id BIGINT NOT NULL,
            organization_id BIGINT NOT NULL DEFAULT 0,
            account_id BIGINT NOT NULL,
            user_id BIGINT NOT NULL,
            request_id BIGINT NOT NULL,
            hold_status TEXT NOT NULL DEFAULT 'held',
            estimated_quantity DOUBLE PRECISION NOT NULL DEFAULT 0,
            captured_quantity DOUBLE PRECISION NOT NULL DEFAULT 0,
            released_quantity DOUBLE PRECISION NOT NULL DEFAULT 0,
            expires_at_ms BIGINT NOT NULL DEFAULT 0,
            created_at_ms BIGINT NOT NULL DEFAULT 0,
            updated_at_ms BIGINT NOT NULL DEFAULT 0
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_ai_account_hold_request
         ON ai_account_hold (tenant_id, organization_id, request_id)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_account_hold_allocation (
            hold_allocation_id BIGINT PRIMARY KEY NOT NULL,
            tenant_id BIGINT NOT NULL,
            organization_id BIGINT NOT NULL DEFAULT 0,
            hold_id BIGINT NOT NULL,
            lot_id BIGINT NOT NULL,
            allocated_quantity DOUBLE PRECISION NOT NULL DEFAULT 0,
            captured_quantity DOUBLE PRECISION NOT NULL DEFAULT 0,
            released_quantity DOUBLE PRECISION NOT NULL DEFAULT 0,
            created_at_ms BIGINT NOT NULL DEFAULT 0,
            updated_at_ms BIGINT NOT NULL DEFAULT 0
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_account_hold_allocation_hold_lot
         ON ai_account_hold_allocation (tenant_id, organization_id, hold_id, lot_id)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_account_ledger_entry (
            ledger_entry_id BIGINT PRIMARY KEY NOT NULL,
            tenant_id BIGINT NOT NULL,
            organization_id BIGINT NOT NULL DEFAULT 0,
            account_id BIGINT NOT NULL,
            user_id BIGINT NOT NULL,
            request_id BIGINT,
            hold_id BIGINT,
            entry_type TEXT NOT NULL,
            benefit_type TEXT,
            quantity DOUBLE PRECISION NOT NULL DEFAULT 0,
            amount DOUBLE PRECISION NOT NULL DEFAULT 0,
            created_at_ms BIGINT NOT NULL DEFAULT 0
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_account_ledger_entry_account_created_at
         ON ai_account_ledger_entry (tenant_id, organization_id, account_id, created_at_ms DESC)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_account_ledger_allocation (
            ledger_allocation_id BIGINT PRIMARY KEY NOT NULL,
            tenant_id BIGINT NOT NULL,
            organization_id BIGINT NOT NULL DEFAULT 0,
            ledger_entry_id BIGINT NOT NULL,
            lot_id BIGINT NOT NULL,
            quantity_delta DOUBLE PRECISION NOT NULL DEFAULT 0,
            created_at_ms BIGINT NOT NULL DEFAULT 0
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_account_ledger_allocation_ledger_lot
         ON ai_account_ledger_allocation (tenant_id, organization_id, ledger_entry_id, lot_id)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_account_commerce_reconciliation_state (
            tenant_id BIGINT NOT NULL,
            organization_id BIGINT NOT NULL DEFAULT 0,
            account_id BIGINT NOT NULL,
            project_id TEXT NOT NULL,
            last_order_updated_at_ms BIGINT NOT NULL DEFAULT 0,
            last_order_created_at_ms BIGINT NOT NULL DEFAULT 0,
            last_order_id TEXT NOT NULL,
            updated_at_ms BIGINT NOT NULL DEFAULT 0,
            PRIMARY KEY (account_id, project_id)
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_account_commerce_reconciliation_state_account_updated
         ON ai_account_commerce_reconciliation_state (
            tenant_id, organization_id, account_id, updated_at_ms DESC
         )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_request_meter_fact (
            request_id BIGINT PRIMARY KEY NOT NULL,
            tenant_id BIGINT NOT NULL,
            organization_id BIGINT NOT NULL DEFAULT 0,
            user_id BIGINT NOT NULL,
            account_id BIGINT NOT NULL,
            api_key_id BIGINT,
            api_key_hash TEXT,
            auth_type TEXT NOT NULL,
            jwt_subject TEXT,
            platform TEXT,
            owner TEXT,
            request_trace_id TEXT,
            gateway_request_ref TEXT,
            upstream_request_ref TEXT,
            protocol_family TEXT NOT NULL DEFAULT '',
            capability_code TEXT NOT NULL,
            channel_code TEXT NOT NULL,
            model_code TEXT NOT NULL,
            provider_code TEXT NOT NULL,
            request_status TEXT NOT NULL DEFAULT 'pending',
            usage_capture_status TEXT NOT NULL DEFAULT 'pending',
            cost_pricing_plan_id BIGINT,
            retail_pricing_plan_id BIGINT,
            estimated_credit_hold DOUBLE PRECISION NOT NULL DEFAULT 0,
            actual_credit_charge DOUBLE PRECISION,
            actual_provider_cost DOUBLE PRECISION,
            started_at_ms BIGINT NOT NULL DEFAULT 0,
            finished_at_ms BIGINT,
            created_at_ms BIGINT NOT NULL DEFAULT 0,
            updated_at_ms BIGINT NOT NULL DEFAULT 0
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_request_meter_fact_user_created_at
         ON ai_request_meter_fact (tenant_id, organization_id, user_id, created_at_ms DESC)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_request_meter_fact_api_key_created_at
         ON ai_request_meter_fact (tenant_id, organization_id, api_key_id, created_at_ms DESC)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_request_meter_fact_provider_model_created_at
         ON ai_request_meter_fact (tenant_id, organization_id, provider_code, model_code, created_at_ms DESC)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_request_meter_metric (
            request_metric_id BIGINT PRIMARY KEY NOT NULL,
            tenant_id BIGINT NOT NULL,
            organization_id BIGINT NOT NULL DEFAULT 0,
            request_id BIGINT NOT NULL,
            metric_code TEXT NOT NULL,
            quantity DOUBLE PRECISION NOT NULL DEFAULT 0,
            provider_field TEXT,
            source_kind TEXT NOT NULL DEFAULT 'provider',
            capture_stage TEXT NOT NULL DEFAULT 'final',
            is_billable BOOLEAN NOT NULL DEFAULT TRUE,
            captured_at_ms BIGINT NOT NULL DEFAULT 0
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_request_meter_metric_request_metric
         ON ai_request_meter_metric (tenant_id, organization_id, request_id, metric_code)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_request_settlement (
            request_settlement_id BIGINT PRIMARY KEY NOT NULL,
            tenant_id BIGINT NOT NULL,
            organization_id BIGINT NOT NULL DEFAULT 0,
            request_id BIGINT NOT NULL,
            account_id BIGINT NOT NULL,
            user_id BIGINT NOT NULL,
            hold_id BIGINT,
            settlement_status TEXT NOT NULL DEFAULT 'pending',
            estimated_credit_hold DOUBLE PRECISION NOT NULL DEFAULT 0,
            released_credit_amount DOUBLE PRECISION NOT NULL DEFAULT 0,
            captured_credit_amount DOUBLE PRECISION NOT NULL DEFAULT 0,
            provider_cost_amount DOUBLE PRECISION NOT NULL DEFAULT 0,
            retail_charge_amount DOUBLE PRECISION NOT NULL DEFAULT 0,
            shortfall_amount DOUBLE PRECISION NOT NULL DEFAULT 0,
            refunded_amount DOUBLE PRECISION NOT NULL DEFAULT 0,
            settled_at_ms BIGINT NOT NULL DEFAULT 0,
            created_at_ms BIGINT NOT NULL DEFAULT 0,
            updated_at_ms BIGINT NOT NULL DEFAULT 0
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_ai_request_settlement_request
         ON ai_request_settlement (tenant_id, organization_id, request_id)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_pricing_plan (
            pricing_plan_id BIGINT PRIMARY KEY NOT NULL,
            tenant_id BIGINT NOT NULL,
            organization_id BIGINT NOT NULL DEFAULT 0,
            plan_code TEXT NOT NULL,
            plan_version BIGINT NOT NULL,
            display_name TEXT NOT NULL DEFAULT '',
            currency_code TEXT NOT NULL DEFAULT 'USD',
            credit_unit_code TEXT NOT NULL DEFAULT 'credit',
            status TEXT NOT NULL DEFAULT 'draft',
            ownership_scope TEXT NOT NULL DEFAULT 'workspace',
            effective_from_ms BIGINT NOT NULL DEFAULT 0,
            effective_to_ms BIGINT,
            created_at_ms BIGINT NOT NULL DEFAULT 0,
            updated_at_ms BIGINT NOT NULL DEFAULT 0
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_ai_pricing_plan_code_version
         ON ai_pricing_plan (tenant_id, organization_id, plan_code, plan_version)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_pricing_plan ADD COLUMN IF NOT EXISTS ownership_scope TEXT NOT NULL DEFAULT 'workspace'",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_pricing_plan ADD COLUMN IF NOT EXISTS effective_from_ms BIGINT NOT NULL DEFAULT 0",
    )
    .execute(&pool)
    .await?;
    sqlx::query("ALTER TABLE ai_pricing_plan ADD COLUMN IF NOT EXISTS effective_to_ms BIGINT")
        .execute(&pool)
        .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_pricing_rate (
            pricing_rate_id BIGINT PRIMARY KEY NOT NULL,
            tenant_id BIGINT NOT NULL,
            organization_id BIGINT NOT NULL DEFAULT 0,
            pricing_plan_id BIGINT NOT NULL,
            metric_code TEXT NOT NULL,
            capability_code TEXT,
            model_code TEXT,
            provider_code TEXT,
            charge_unit TEXT NOT NULL DEFAULT 'unit',
            pricing_method TEXT NOT NULL DEFAULT 'per_unit',
            quantity_step DOUBLE PRECISION NOT NULL DEFAULT 1,
            unit_price DOUBLE PRECISION NOT NULL DEFAULT 0,
            display_price_unit TEXT NOT NULL DEFAULT '',
            minimum_billable_quantity DOUBLE PRECISION NOT NULL DEFAULT 0,
            minimum_charge DOUBLE PRECISION NOT NULL DEFAULT 0,
            rounding_increment DOUBLE PRECISION NOT NULL DEFAULT 1,
            rounding_mode TEXT NOT NULL DEFAULT 'none',
            included_quantity DOUBLE PRECISION NOT NULL DEFAULT 0,
            priority BIGINT NOT NULL DEFAULT 0,
            notes TEXT,
            status TEXT NOT NULL DEFAULT 'draft',
            created_at_ms BIGINT NOT NULL DEFAULT 0,
            updated_at_ms BIGINT NOT NULL DEFAULT 0
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_pricing_rate_plan_metric
         ON ai_pricing_rate (tenant_id, organization_id, pricing_plan_id, metric_code)",
    )
    .execute(&pool)
    .await?;
    sqlx::query("ALTER TABLE ai_pricing_rate ADD COLUMN IF NOT EXISTS capability_code TEXT")
        .execute(&pool)
        .await?;
    sqlx::query(
        "ALTER TABLE ai_pricing_rate ADD COLUMN IF NOT EXISTS charge_unit TEXT NOT NULL DEFAULT 'unit'",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_pricing_rate ADD COLUMN IF NOT EXISTS pricing_method TEXT NOT NULL DEFAULT 'per_unit'",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_pricing_rate ADD COLUMN IF NOT EXISTS display_price_unit TEXT NOT NULL DEFAULT ''",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_pricing_rate ADD COLUMN IF NOT EXISTS minimum_billable_quantity DOUBLE PRECISION NOT NULL DEFAULT 0",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_pricing_rate ADD COLUMN IF NOT EXISTS minimum_charge DOUBLE PRECISION NOT NULL DEFAULT 0",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_pricing_rate ADD COLUMN IF NOT EXISTS rounding_increment DOUBLE PRECISION NOT NULL DEFAULT 1",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_pricing_rate ADD COLUMN IF NOT EXISTS rounding_mode TEXT NOT NULL DEFAULT 'none'",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_pricing_rate ADD COLUMN IF NOT EXISTS included_quantity DOUBLE PRECISION NOT NULL DEFAULT 0",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_pricing_rate ADD COLUMN IF NOT EXISTS priority BIGINT NOT NULL DEFAULT 0",
    )
    .execute(&pool)
    .await?;
    sqlx::query("ALTER TABLE ai_pricing_rate ADD COLUMN IF NOT EXISTS notes TEXT")
        .execute(&pool)
        .await?;
    sqlx::query(
        "ALTER TABLE ai_pricing_rate ADD COLUMN IF NOT EXISTS status TEXT NOT NULL DEFAULT 'draft'",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_pricing_rate ADD COLUMN IF NOT EXISTS updated_at_ms BIGINT NOT NULL DEFAULT 0",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_usage_records (
            project_id TEXT NOT NULL,
            model TEXT NOT NULL,
            provider_id TEXT NOT NULL,
            units BIGINT NOT NULL DEFAULT 0,
            amount DOUBLE PRECISION NOT NULL DEFAULT 0,
            input_tokens BIGINT NOT NULL DEFAULT 0,
            output_tokens BIGINT NOT NULL DEFAULT 0,
            total_tokens BIGINT NOT NULL DEFAULT 0,
            api_key_hash TEXT,
            channel_id TEXT,
            latency_ms BIGINT,
            reference_amount DOUBLE PRECISION,
            created_at_ms BIGINT NOT NULL DEFAULT 0
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_usage_records ADD COLUMN IF NOT EXISTS units BIGINT NOT NULL DEFAULT 0",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_usage_records ADD COLUMN IF NOT EXISTS amount DOUBLE PRECISION NOT NULL DEFAULT 0",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_usage_records ADD COLUMN IF NOT EXISTS input_tokens BIGINT NOT NULL DEFAULT 0",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_usage_records ADD COLUMN IF NOT EXISTS output_tokens BIGINT NOT NULL DEFAULT 0",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_usage_records ADD COLUMN IF NOT EXISTS total_tokens BIGINT NOT NULL DEFAULT 0",
    )
    .execute(&pool)
    .await?;
    sqlx::query("ALTER TABLE ai_usage_records ADD COLUMN IF NOT EXISTS api_key_hash TEXT")
        .execute(&pool)
        .await?;
    sqlx::query("ALTER TABLE ai_usage_records ADD COLUMN IF NOT EXISTS channel_id TEXT")
        .execute(&pool)
        .await?;
    sqlx::query("ALTER TABLE ai_usage_records ADD COLUMN IF NOT EXISTS latency_ms BIGINT")
        .execute(&pool)
        .await?;
    sqlx::query(
        "ALTER TABLE ai_usage_records ADD COLUMN IF NOT EXISTS reference_amount DOUBLE PRECISION",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_usage_records ADD COLUMN IF NOT EXISTS created_at_ms BIGINT NOT NULL DEFAULT 0",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_usage_records_project_created_at
         ON ai_usage_records (project_id, created_at_ms DESC, provider_id, model)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_usage_records_created_at
         ON ai_usage_records (created_at_ms DESC, project_id, provider_id, model)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_usage_records_project_fact_filters
         ON ai_usage_records (project_id, created_at_ms DESC, api_key_hash, channel_id, model)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_billing_events (
            event_id TEXT PRIMARY KEY NOT NULL,
            tenant_id TEXT NOT NULL,
            project_id TEXT NOT NULL,
            api_key_group_id TEXT,
            capability TEXT NOT NULL,
            route_key TEXT NOT NULL,
            usage_model TEXT NOT NULL,
            provider_id TEXT NOT NULL,
            accounting_mode TEXT NOT NULL,
            operation_kind TEXT NOT NULL,
            modality TEXT NOT NULL,
            api_key_hash TEXT,
            channel_id TEXT,
            reference_id TEXT,
            latency_ms BIGINT,
            units BIGINT NOT NULL DEFAULT 0,
            request_count BIGINT NOT NULL DEFAULT 1,
            input_tokens BIGINT NOT NULL DEFAULT 0,
            output_tokens BIGINT NOT NULL DEFAULT 0,
            total_tokens BIGINT NOT NULL DEFAULT 0,
            cache_read_tokens BIGINT NOT NULL DEFAULT 0,
            cache_write_tokens BIGINT NOT NULL DEFAULT 0,
            image_count BIGINT NOT NULL DEFAULT 0,
            audio_seconds DOUBLE PRECISION NOT NULL DEFAULT 0,
            video_seconds DOUBLE PRECISION NOT NULL DEFAULT 0,
            music_seconds DOUBLE PRECISION NOT NULL DEFAULT 0,
            upstream_cost DOUBLE PRECISION NOT NULL DEFAULT 0,
            customer_charge DOUBLE PRECISION NOT NULL DEFAULT 0,
            applied_routing_profile_id TEXT,
            compiled_routing_snapshot_id TEXT,
            fallback_reason TEXT,
            created_at_ms BIGINT NOT NULL DEFAULT 0
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_billing_events_project_created_at
         ON ai_billing_events (project_id, created_at_ms DESC, capability, provider_id)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_billing_events_group_created_at
         ON ai_billing_events (api_key_group_id, created_at_ms DESC, project_id, capability)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_billing_events_capability_created_at
         ON ai_billing_events (capability, created_at_ms DESC, project_id, provider_id)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_billing_ledger_entries (
            project_id TEXT NOT NULL,
            units BIGINT NOT NULL,
            amount DOUBLE PRECISION NOT NULL,
            created_at_ms BIGINT NOT NULL DEFAULT 0
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_billing_ledger_entries ADD COLUMN IF NOT EXISTS created_at_ms BIGINT NOT NULL DEFAULT 0",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_billing_ledger_entries_project_created_at
         ON ai_billing_ledger_entries (project_id, created_at_ms DESC)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_billing_ledger_entries_created_at
         ON ai_billing_ledger_entries (created_at_ms DESC, project_id)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_billing_quota_policies (
            policy_id TEXT PRIMARY KEY NOT NULL,
            project_id TEXT NOT NULL,
            max_units BIGINT NOT NULL,
            enabled BOOLEAN NOT NULL DEFAULT TRUE
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_billing_quota_policies_project_enabled
         ON ai_billing_quota_policies (project_id, enabled, policy_id)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_gateway_rate_limit_policies (
            policy_id TEXT PRIMARY KEY NOT NULL,
            project_id TEXT NOT NULL,
            api_key_hash TEXT,
            route_key TEXT,
            model_name TEXT,
            requests_per_window BIGINT NOT NULL,
            window_seconds BIGINT NOT NULL DEFAULT 60,
            burst_requests BIGINT NOT NULL DEFAULT 0,
            enabled BOOLEAN NOT NULL DEFAULT TRUE,
            notes TEXT,
            created_at_ms BIGINT NOT NULL DEFAULT 0,
            updated_at_ms BIGINT NOT NULL DEFAULT 0
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_gateway_rate_limit_policies_project_enabled
         ON ai_gateway_rate_limit_policies (project_id, enabled, policy_id)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_gateway_rate_limit_policies_project_scope
         ON ai_gateway_rate_limit_policies (project_id, api_key_hash, route_key, model_name, enabled, policy_id)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_gateway_rate_limit_windows (
            policy_id TEXT NOT NULL,
            window_start_ms BIGINT NOT NULL,
            request_count BIGINT NOT NULL DEFAULT 0,
            updated_at_ms BIGINT NOT NULL DEFAULT 0,
            PRIMARY KEY (policy_id, window_start_ms)
        )",
    )
    .execute(&pool)
    .await?;
    Ok(())
}
