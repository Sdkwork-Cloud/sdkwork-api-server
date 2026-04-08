use super::*;

pub(crate) async fn apply_sqlite_billing_schema(pool: &SqlitePool) -> Result<()> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_account (
            account_id INTEGER PRIMARY KEY NOT NULL,
            tenant_id INTEGER NOT NULL,
            organization_id INTEGER NOT NULL DEFAULT 0,
            user_id INTEGER NOT NULL,
            account_type TEXT NOT NULL,
            currency_code TEXT NOT NULL DEFAULT 'USD',
            credit_unit_code TEXT NOT NULL DEFAULT 'credit',
            status TEXT NOT NULL DEFAULT 'active',
            allow_overdraft INTEGER NOT NULL DEFAULT 0,
            overdraft_limit REAL NOT NULL DEFAULT 0,
            created_at_ms INTEGER NOT NULL DEFAULT 0,
            updated_at_ms INTEGER NOT NULL DEFAULT 0
        )",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_ai_account_user_type
         ON ai_account (tenant_id, organization_id, user_id, account_type)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_account_benefit_lot (
            lot_id INTEGER PRIMARY KEY NOT NULL,
            tenant_id INTEGER NOT NULL,
            organization_id INTEGER NOT NULL DEFAULT 0,
            account_id INTEGER NOT NULL,
            user_id INTEGER NOT NULL,
            benefit_type TEXT NOT NULL,
            source_type TEXT NOT NULL,
            source_id INTEGER,
            scope_json TEXT,
            original_quantity REAL NOT NULL DEFAULT 0,
            remaining_quantity REAL NOT NULL DEFAULT 0,
            held_quantity REAL NOT NULL DEFAULT 0,
            priority INTEGER NOT NULL DEFAULT 0,
            acquired_unit_cost REAL,
            issued_at_ms INTEGER NOT NULL DEFAULT 0,
            expires_at_ms INTEGER,
            status TEXT NOT NULL DEFAULT 'active',
            created_at_ms INTEGER NOT NULL DEFAULT 0,
            updated_at_ms INTEGER NOT NULL DEFAULT 0
        )",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_account_benefit_lot_account_status_expiry
         ON ai_account_benefit_lot (tenant_id, organization_id, account_id, status, expires_at_ms)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_account_hold (
            hold_id INTEGER PRIMARY KEY NOT NULL,
            tenant_id INTEGER NOT NULL,
            organization_id INTEGER NOT NULL DEFAULT 0,
            account_id INTEGER NOT NULL,
            user_id INTEGER NOT NULL,
            request_id INTEGER NOT NULL,
            hold_status TEXT NOT NULL DEFAULT 'held',
            estimated_quantity REAL NOT NULL DEFAULT 0,
            captured_quantity REAL NOT NULL DEFAULT 0,
            released_quantity REAL NOT NULL DEFAULT 0,
            expires_at_ms INTEGER NOT NULL DEFAULT 0,
            created_at_ms INTEGER NOT NULL DEFAULT 0,
            updated_at_ms INTEGER NOT NULL DEFAULT 0
        )",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_ai_account_hold_request
         ON ai_account_hold (tenant_id, organization_id, request_id)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_account_hold_allocation (
            hold_allocation_id INTEGER PRIMARY KEY NOT NULL,
            tenant_id INTEGER NOT NULL,
            organization_id INTEGER NOT NULL DEFAULT 0,
            hold_id INTEGER NOT NULL,
            lot_id INTEGER NOT NULL,
            allocated_quantity REAL NOT NULL DEFAULT 0,
            captured_quantity REAL NOT NULL DEFAULT 0,
            released_quantity REAL NOT NULL DEFAULT 0,
            created_at_ms INTEGER NOT NULL DEFAULT 0,
            updated_at_ms INTEGER NOT NULL DEFAULT 0
        )",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_account_hold_allocation_hold_lot
         ON ai_account_hold_allocation (tenant_id, organization_id, hold_id, lot_id)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_account_ledger_entry (
            ledger_entry_id INTEGER PRIMARY KEY NOT NULL,
            tenant_id INTEGER NOT NULL,
            organization_id INTEGER NOT NULL DEFAULT 0,
            account_id INTEGER NOT NULL,
            user_id INTEGER NOT NULL,
            request_id INTEGER,
            hold_id INTEGER,
            entry_type TEXT NOT NULL,
            benefit_type TEXT,
            quantity REAL NOT NULL DEFAULT 0,
            amount REAL NOT NULL DEFAULT 0,
            created_at_ms INTEGER NOT NULL DEFAULT 0
        )",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_account_ledger_entry_account_created_at
         ON ai_account_ledger_entry (tenant_id, organization_id, account_id, created_at_ms DESC)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_account_ledger_allocation (
            ledger_allocation_id INTEGER PRIMARY KEY NOT NULL,
            tenant_id INTEGER NOT NULL,
            organization_id INTEGER NOT NULL DEFAULT 0,
            ledger_entry_id INTEGER NOT NULL,
            lot_id INTEGER NOT NULL,
            quantity_delta REAL NOT NULL DEFAULT 0,
            created_at_ms INTEGER NOT NULL DEFAULT 0
        )",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_account_ledger_allocation_ledger_lot
         ON ai_account_ledger_allocation (tenant_id, organization_id, ledger_entry_id, lot_id)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_account_commerce_reconciliation_state (
            tenant_id INTEGER NOT NULL,
            organization_id INTEGER NOT NULL DEFAULT 0,
            account_id INTEGER NOT NULL,
            project_id TEXT NOT NULL,
            last_order_updated_at_ms INTEGER NOT NULL DEFAULT 0,
            last_order_created_at_ms INTEGER NOT NULL DEFAULT 0,
            last_order_id TEXT NOT NULL,
            updated_at_ms INTEGER NOT NULL DEFAULT 0,
            PRIMARY KEY (account_id, project_id)
        )",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_account_commerce_reconciliation_state_account_updated
         ON ai_account_commerce_reconciliation_state (
            tenant_id, organization_id, account_id, updated_at_ms DESC
         )",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_request_meter_fact (
            request_id INTEGER PRIMARY KEY NOT NULL,
            tenant_id INTEGER NOT NULL,
            organization_id INTEGER NOT NULL DEFAULT 0,
            user_id INTEGER NOT NULL,
            account_id INTEGER NOT NULL,
            api_key_id INTEGER,
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
            cost_pricing_plan_id INTEGER,
            retail_pricing_plan_id INTEGER,
            estimated_credit_hold REAL NOT NULL DEFAULT 0,
            actual_credit_charge REAL,
            actual_provider_cost REAL,
            started_at_ms INTEGER NOT NULL DEFAULT 0,
            finished_at_ms INTEGER,
            created_at_ms INTEGER NOT NULL DEFAULT 0,
            updated_at_ms INTEGER NOT NULL DEFAULT 0
        )",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_request_meter_fact_user_created_at
         ON ai_request_meter_fact (tenant_id, organization_id, user_id, created_at_ms DESC)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_request_meter_fact_api_key_created_at
         ON ai_request_meter_fact (tenant_id, organization_id, api_key_id, created_at_ms DESC)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_request_meter_fact_provider_model_created_at
         ON ai_request_meter_fact (tenant_id, organization_id, provider_code, model_code, created_at_ms DESC)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_request_meter_metric (
            request_metric_id INTEGER PRIMARY KEY NOT NULL,
            tenant_id INTEGER NOT NULL,
            organization_id INTEGER NOT NULL DEFAULT 0,
            request_id INTEGER NOT NULL,
            metric_code TEXT NOT NULL,
            quantity REAL NOT NULL DEFAULT 0,
            provider_field TEXT,
            source_kind TEXT NOT NULL DEFAULT 'provider',
            capture_stage TEXT NOT NULL DEFAULT 'final',
            is_billable INTEGER NOT NULL DEFAULT 1,
            captured_at_ms INTEGER NOT NULL DEFAULT 0
        )",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_request_meter_metric_request_metric
         ON ai_request_meter_metric (tenant_id, organization_id, request_id, metric_code)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_request_settlement (
            request_settlement_id INTEGER PRIMARY KEY NOT NULL,
            tenant_id INTEGER NOT NULL,
            organization_id INTEGER NOT NULL DEFAULT 0,
            request_id INTEGER NOT NULL,
            account_id INTEGER NOT NULL,
            user_id INTEGER NOT NULL,
            hold_id INTEGER,
            settlement_status TEXT NOT NULL DEFAULT 'pending',
            estimated_credit_hold REAL NOT NULL DEFAULT 0,
            released_credit_amount REAL NOT NULL DEFAULT 0,
            captured_credit_amount REAL NOT NULL DEFAULT 0,
            provider_cost_amount REAL NOT NULL DEFAULT 0,
            retail_charge_amount REAL NOT NULL DEFAULT 0,
            shortfall_amount REAL NOT NULL DEFAULT 0,
            refunded_amount REAL NOT NULL DEFAULT 0,
            settled_at_ms INTEGER NOT NULL DEFAULT 0,
            created_at_ms INTEGER NOT NULL DEFAULT 0,
            updated_at_ms INTEGER NOT NULL DEFAULT 0
        )",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_ai_request_settlement_request
         ON ai_request_settlement (tenant_id, organization_id, request_id)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_pricing_plan (
            pricing_plan_id INTEGER PRIMARY KEY NOT NULL,
            tenant_id INTEGER NOT NULL,
            organization_id INTEGER NOT NULL DEFAULT 0,
            plan_code TEXT NOT NULL,
            plan_version INTEGER NOT NULL,
            display_name TEXT NOT NULL DEFAULT '',
            currency_code TEXT NOT NULL DEFAULT 'USD',
            credit_unit_code TEXT NOT NULL DEFAULT 'credit',
            status TEXT NOT NULL DEFAULT 'draft',
            effective_from_ms INTEGER NOT NULL DEFAULT 0,
            effective_to_ms INTEGER,
            created_at_ms INTEGER NOT NULL DEFAULT 0,
            updated_at_ms INTEGER NOT NULL DEFAULT 0
        )",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_ai_pricing_plan_code_version
         ON ai_pricing_plan (tenant_id, organization_id, plan_code, plan_version)",
    )
    .execute(pool)
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_pricing_plan",
        "effective_from_ms",
        "effective_from_ms INTEGER NOT NULL DEFAULT 0",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_pricing_plan",
        "effective_to_ms",
        "effective_to_ms INTEGER",
    )
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_pricing_rate (
            pricing_rate_id INTEGER PRIMARY KEY NOT NULL,
            tenant_id INTEGER NOT NULL,
            organization_id INTEGER NOT NULL DEFAULT 0,
            pricing_plan_id INTEGER NOT NULL,
            metric_code TEXT NOT NULL,
            capability_code TEXT,
            model_code TEXT,
            provider_code TEXT,
            charge_unit TEXT NOT NULL DEFAULT 'unit',
            pricing_method TEXT NOT NULL DEFAULT 'per_unit',
            quantity_step REAL NOT NULL DEFAULT 1,
            unit_price REAL NOT NULL DEFAULT 0,
            display_price_unit TEXT NOT NULL DEFAULT '',
            minimum_billable_quantity REAL NOT NULL DEFAULT 0,
            minimum_charge REAL NOT NULL DEFAULT 0,
            rounding_increment REAL NOT NULL DEFAULT 1,
            rounding_mode TEXT NOT NULL DEFAULT 'none',
            included_quantity REAL NOT NULL DEFAULT 0,
            priority INTEGER NOT NULL DEFAULT 0,
            notes TEXT,
            status TEXT NOT NULL DEFAULT 'draft',
            created_at_ms INTEGER NOT NULL DEFAULT 0,
            updated_at_ms INTEGER NOT NULL DEFAULT 0
        )",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_pricing_rate_plan_metric
         ON ai_pricing_rate (tenant_id, organization_id, pricing_plan_id, metric_code)",
    )
    .execute(pool)
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_pricing_rate",
        "capability_code",
        "capability_code TEXT",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_pricing_rate",
        "charge_unit",
        "charge_unit TEXT NOT NULL DEFAULT 'unit'",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_pricing_rate",
        "pricing_method",
        "pricing_method TEXT NOT NULL DEFAULT 'per_unit'",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_pricing_rate",
        "display_price_unit",
        "display_price_unit TEXT NOT NULL DEFAULT ''",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_pricing_rate",
        "minimum_billable_quantity",
        "minimum_billable_quantity REAL NOT NULL DEFAULT 0",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_pricing_rate",
        "minimum_charge",
        "minimum_charge REAL NOT NULL DEFAULT 0",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_pricing_rate",
        "rounding_increment",
        "rounding_increment REAL NOT NULL DEFAULT 1",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_pricing_rate",
        "rounding_mode",
        "rounding_mode TEXT NOT NULL DEFAULT 'none'",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_pricing_rate",
        "included_quantity",
        "included_quantity REAL NOT NULL DEFAULT 0",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_pricing_rate",
        "priority",
        "priority INTEGER NOT NULL DEFAULT 0",
    )
    .await?;
    ensure_sqlite_column(pool, "ai_pricing_rate", "notes", "notes TEXT").await?;
    ensure_sqlite_column(
        pool,
        "ai_pricing_rate",
        "status",
        "status TEXT NOT NULL DEFAULT 'draft'",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_pricing_rate",
        "updated_at_ms",
        "updated_at_ms INTEGER NOT NULL DEFAULT 0",
    )
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_usage_records (
            project_id TEXT NOT NULL,
            model TEXT NOT NULL,
            provider_id TEXT NOT NULL,
            units INTEGER NOT NULL DEFAULT 0,
            amount REAL NOT NULL DEFAULT 0,
            input_tokens INTEGER NOT NULL DEFAULT 0,
            output_tokens INTEGER NOT NULL DEFAULT 0,
            total_tokens INTEGER NOT NULL DEFAULT 0,
            api_key_hash TEXT,
            channel_id TEXT,
            latency_ms INTEGER,
            reference_amount REAL,
            created_at_ms INTEGER NOT NULL DEFAULT 0
        )",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_usage_records_project_created_at
         ON ai_usage_records (project_id, created_at_ms DESC, provider_id, model)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_usage_records_created_at
         ON ai_usage_records (created_at_ms DESC, project_id, provider_id, model)",
    )
    .execute(pool)
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_usage_records",
        "units",
        "units INTEGER NOT NULL DEFAULT 0",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_usage_records",
        "amount",
        "amount REAL NOT NULL DEFAULT 0",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_usage_records",
        "input_tokens",
        "input_tokens INTEGER NOT NULL DEFAULT 0",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_usage_records",
        "output_tokens",
        "output_tokens INTEGER NOT NULL DEFAULT 0",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_usage_records",
        "total_tokens",
        "total_tokens INTEGER NOT NULL DEFAULT 0",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_usage_records",
        "api_key_hash",
        "api_key_hash TEXT",
    )
    .await?;
    ensure_sqlite_column(pool, "ai_usage_records", "channel_id", "channel_id TEXT").await?;
    ensure_sqlite_column(
        pool,
        "ai_usage_records",
        "latency_ms",
        "latency_ms INTEGER",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_usage_records",
        "reference_amount",
        "reference_amount REAL",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_usage_records",
        "created_at_ms",
        "created_at_ms INTEGER NOT NULL DEFAULT 0",
    )
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_usage_records_project_fact_filters
         ON ai_usage_records (project_id, created_at_ms DESC, api_key_hash, channel_id, model)",
    )
    .execute(pool)
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
            latency_ms INTEGER,
            units INTEGER NOT NULL DEFAULT 0,
            request_count INTEGER NOT NULL DEFAULT 1,
            input_tokens INTEGER NOT NULL DEFAULT 0,
            output_tokens INTEGER NOT NULL DEFAULT 0,
            total_tokens INTEGER NOT NULL DEFAULT 0,
            cache_read_tokens INTEGER NOT NULL DEFAULT 0,
            cache_write_tokens INTEGER NOT NULL DEFAULT 0,
            image_count INTEGER NOT NULL DEFAULT 0,
            audio_seconds REAL NOT NULL DEFAULT 0,
            video_seconds REAL NOT NULL DEFAULT 0,
            music_seconds REAL NOT NULL DEFAULT 0,
            upstream_cost REAL NOT NULL DEFAULT 0,
            customer_charge REAL NOT NULL DEFAULT 0,
            applied_routing_profile_id TEXT,
            compiled_routing_snapshot_id TEXT,
            fallback_reason TEXT,
            created_at_ms INTEGER NOT NULL DEFAULT 0
        )",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_billing_events_project_created_at
         ON ai_billing_events (project_id, created_at_ms DESC, capability, provider_id)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_billing_events_group_created_at
         ON ai_billing_events (api_key_group_id, created_at_ms DESC, project_id, capability)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_billing_events_capability_created_at
         ON ai_billing_events (capability, created_at_ms DESC, project_id, provider_id)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_billing_ledger_entries (
            project_id TEXT NOT NULL,
            units INTEGER NOT NULL,
            amount REAL NOT NULL,
            created_at_ms INTEGER NOT NULL DEFAULT 0
        )",
    )
    .execute(pool)
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_billing_ledger_entries",
        "created_at_ms",
        "created_at_ms INTEGER NOT NULL DEFAULT 0",
    )
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_billing_ledger_entries_project_created_at
         ON ai_billing_ledger_entries (project_id, created_at_ms DESC)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_billing_ledger_entries_created_at
         ON ai_billing_ledger_entries (created_at_ms DESC, project_id)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_billing_quota_policies (
            policy_id TEXT PRIMARY KEY NOT NULL,
            project_id TEXT NOT NULL,
            max_units INTEGER NOT NULL,
            enabled INTEGER NOT NULL DEFAULT 1
        )",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_billing_quota_policies_project_enabled
         ON ai_billing_quota_policies (project_id, enabled, policy_id)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_gateway_rate_limit_policies (
            policy_id TEXT PRIMARY KEY NOT NULL,
            project_id TEXT NOT NULL,
            api_key_hash TEXT,
            route_key TEXT,
            model_name TEXT,
            requests_per_window INTEGER NOT NULL,
            window_seconds INTEGER NOT NULL DEFAULT 60,
            burst_requests INTEGER NOT NULL DEFAULT 0,
            enabled INTEGER NOT NULL DEFAULT 1,
            notes TEXT,
            created_at_ms INTEGER NOT NULL DEFAULT 0,
            updated_at_ms INTEGER NOT NULL DEFAULT 0
        )",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_gateway_rate_limit_policies_project_enabled
         ON ai_gateway_rate_limit_policies (project_id, enabled, policy_id)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_gateway_rate_limit_policies_project_scope
         ON ai_gateway_rate_limit_policies (project_id, api_key_hash, route_key, model_name, enabled, policy_id)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_gateway_rate_limit_windows (
            policy_id TEXT NOT NULL,
            window_start_ms INTEGER NOT NULL,
            request_count INTEGER NOT NULL DEFAULT 0,
            updated_at_ms INTEGER NOT NULL DEFAULT 0,
            PRIMARY KEY (policy_id, window_start_ms)
        )",
    )
    .execute(pool)
    .await?;

    Ok(())
}
