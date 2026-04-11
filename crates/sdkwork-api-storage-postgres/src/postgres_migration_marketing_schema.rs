use super::*;

pub(crate) async fn apply_postgres_marketing_schema(pool: &PgPool) -> Result<()> {
    let pool = pool.clone();
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_coupon_campaigns (
            id TEXT PRIMARY KEY NOT NULL,
            code TEXT NOT NULL,
            discount_label TEXT NOT NULL,
            audience TEXT NOT NULL,
            remaining BIGINT NOT NULL DEFAULT 0,
            active BOOLEAN NOT NULL DEFAULT TRUE,
            note TEXT NOT NULL DEFAULT '',
            expires_on TEXT NOT NULL DEFAULT '',
            created_at_ms BIGINT NOT NULL DEFAULT 0
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_ai_coupon_campaigns_code ON ai_coupon_campaigns (code)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_coupon_campaigns_active_remaining_created
         ON ai_coupon_campaigns (active, remaining, created_at_ms DESC, code)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_marketing_coupon_template (
            coupon_template_id TEXT PRIMARY KEY NOT NULL,
            template_key TEXT NOT NULL,
            status TEXT NOT NULL,
            distribution_kind TEXT NOT NULL,
            created_at_ms BIGINT NOT NULL DEFAULT 0,
            updated_at_ms BIGINT NOT NULL DEFAULT 0,
            record_json TEXT NOT NULL
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_ai_marketing_coupon_template_key
         ON ai_marketing_coupon_template (template_key)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_marketing_coupon_template_status_updated
         ON ai_marketing_coupon_template (status, updated_at_ms DESC, coupon_template_id)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_marketing_coupon_template_lifecycle_audit (
            audit_id TEXT PRIMARY KEY NOT NULL,
            coupon_template_id TEXT NOT NULL,
            action TEXT NOT NULL,
            outcome TEXT NOT NULL,
            operator_id TEXT NOT NULL,
            request_id TEXT NOT NULL,
            previous_status TEXT NOT NULL,
            resulting_status TEXT NOT NULL,
            reason TEXT NOT NULL DEFAULT '',
            decision_reasons_json TEXT NOT NULL DEFAULT '[]',
            requested_at_ms BIGINT NOT NULL DEFAULT 0,
            record_json TEXT NOT NULL
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_marketing_coupon_template_lifecycle_audit_template
         ON ai_marketing_coupon_template_lifecycle_audit (
            coupon_template_id, requested_at_ms DESC, audit_id DESC
         )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_marketing_coupon_template_lifecycle_audit_request
         ON ai_marketing_coupon_template_lifecycle_audit (
            request_id, requested_at_ms DESC, audit_id DESC
         )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_marketing_coupon_template_lifecycle_audit_operator
         ON ai_marketing_coupon_template_lifecycle_audit (
            operator_id, requested_at_ms DESC, audit_id DESC
         )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_marketing_campaign (
            marketing_campaign_id TEXT PRIMARY KEY NOT NULL,
            coupon_template_id TEXT NOT NULL,
            status TEXT NOT NULL,
            start_at_ms BIGINT,
            end_at_ms BIGINT,
            created_at_ms BIGINT NOT NULL DEFAULT 0,
            updated_at_ms BIGINT NOT NULL DEFAULT 0,
            record_json TEXT NOT NULL
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_marketing_campaign_template_status_window
         ON ai_marketing_campaign (
            coupon_template_id, status, start_at_ms, end_at_ms, updated_at_ms DESC, marketing_campaign_id
         )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_marketing_campaign_lifecycle_audit (
            audit_id TEXT PRIMARY KEY NOT NULL,
            marketing_campaign_id TEXT NOT NULL,
            coupon_template_id TEXT NOT NULL,
            action TEXT NOT NULL,
            outcome TEXT NOT NULL,
            operator_id TEXT NOT NULL,
            request_id TEXT NOT NULL,
            previous_status TEXT NOT NULL,
            resulting_status TEXT NOT NULL,
            reason TEXT NOT NULL DEFAULT '',
            decision_reasons_json TEXT NOT NULL DEFAULT '[]',
            requested_at_ms BIGINT NOT NULL DEFAULT 0,
            record_json TEXT NOT NULL
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_marketing_campaign_lifecycle_audit_campaign
         ON ai_marketing_campaign_lifecycle_audit (
            marketing_campaign_id, requested_at_ms DESC, audit_id DESC
         )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_marketing_campaign_lifecycle_audit_request
         ON ai_marketing_campaign_lifecycle_audit (
            request_id, requested_at_ms DESC, audit_id DESC
         )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_marketing_campaign_lifecycle_audit_operator
         ON ai_marketing_campaign_lifecycle_audit (
            operator_id, requested_at_ms DESC, audit_id DESC
         )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_marketing_campaign_budget (
            campaign_budget_id TEXT PRIMARY KEY NOT NULL,
            marketing_campaign_id TEXT NOT NULL,
            status TEXT NOT NULL,
            created_at_ms BIGINT NOT NULL DEFAULT 0,
            updated_at_ms BIGINT NOT NULL DEFAULT 0,
            record_json TEXT NOT NULL
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_marketing_campaign_budget_campaign_status
         ON ai_marketing_campaign_budget (
            marketing_campaign_id, status, updated_at_ms DESC, campaign_budget_id
         )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_marketing_campaign_budget_lifecycle_audit (
            audit_id TEXT PRIMARY KEY NOT NULL,
            campaign_budget_id TEXT NOT NULL,
            marketing_campaign_id TEXT NOT NULL,
            action TEXT NOT NULL,
            outcome TEXT NOT NULL,
            operator_id TEXT NOT NULL,
            request_id TEXT NOT NULL,
            previous_status TEXT NOT NULL,
            resulting_status TEXT NOT NULL,
            reason TEXT NOT NULL DEFAULT '',
            decision_reasons_json TEXT NOT NULL DEFAULT '[]',
            requested_at_ms BIGINT NOT NULL DEFAULT 0,
            record_json TEXT NOT NULL
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_marketing_campaign_budget_lifecycle_audit_budget
         ON ai_marketing_campaign_budget_lifecycle_audit (
            campaign_budget_id, requested_at_ms DESC, audit_id DESC
         )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_marketing_campaign_budget_lifecycle_audit_request
         ON ai_marketing_campaign_budget_lifecycle_audit (
            request_id, requested_at_ms DESC, audit_id DESC
         )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_marketing_campaign_budget_lifecycle_audit_operator
         ON ai_marketing_campaign_budget_lifecycle_audit (
            operator_id, requested_at_ms DESC, audit_id DESC
         )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_marketing_coupon_code (
            coupon_code_id TEXT PRIMARY KEY NOT NULL,
            coupon_template_id TEXT NOT NULL,
            code_value TEXT NOT NULL,
            normalized_code_value TEXT NOT NULL,
            status TEXT NOT NULL,
            claimed_subject_scope TEXT,
            claimed_subject_id TEXT,
            expires_at_ms BIGINT,
            created_at_ms BIGINT NOT NULL DEFAULT 0,
            updated_at_ms BIGINT NOT NULL DEFAULT 0,
            record_json TEXT NOT NULL
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_ai_marketing_coupon_code_normalized
         ON ai_marketing_coupon_code (normalized_code_value)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_marketing_coupon_code_template_status_expiry
         ON ai_marketing_coupon_code (
            coupon_template_id, status, expires_at_ms, updated_at_ms DESC, coupon_code_id
         )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_marketing_coupon_code_subject
         ON ai_marketing_coupon_code (
            claimed_subject_scope, claimed_subject_id, updated_at_ms DESC, coupon_code_id
         )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_marketing_coupon_code_lifecycle_audit (
            audit_id TEXT PRIMARY KEY NOT NULL,
            coupon_code_id TEXT NOT NULL,
            coupon_template_id TEXT NOT NULL,
            action TEXT NOT NULL,
            outcome TEXT NOT NULL,
            operator_id TEXT NOT NULL,
            request_id TEXT NOT NULL,
            previous_status TEXT NOT NULL,
            resulting_status TEXT NOT NULL,
            reason TEXT NOT NULL DEFAULT '',
            decision_reasons_json TEXT NOT NULL DEFAULT '[]',
            requested_at_ms BIGINT NOT NULL DEFAULT 0,
            record_json TEXT NOT NULL
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_marketing_coupon_code_lifecycle_audit_code
         ON ai_marketing_coupon_code_lifecycle_audit (
            coupon_code_id, requested_at_ms DESC, audit_id DESC
         )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_marketing_coupon_code_lifecycle_audit_request
         ON ai_marketing_coupon_code_lifecycle_audit (
            request_id, requested_at_ms DESC, audit_id DESC
         )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_marketing_coupon_code_lifecycle_audit_operator
         ON ai_marketing_coupon_code_lifecycle_audit (
            operator_id, requested_at_ms DESC, audit_id DESC
         )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_marketing_coupon_reservation (
            coupon_reservation_id TEXT PRIMARY KEY NOT NULL,
            coupon_code_id TEXT NOT NULL,
            subject_scope TEXT NOT NULL,
            subject_id TEXT NOT NULL,
            reservation_status TEXT NOT NULL,
            expires_at_ms BIGINT NOT NULL,
            created_at_ms BIGINT NOT NULL DEFAULT 0,
            updated_at_ms BIGINT NOT NULL DEFAULT 0,
            record_json TEXT NOT NULL
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_marketing_coupon_reservation_code_status_expiry
         ON ai_marketing_coupon_reservation (
            coupon_code_id, reservation_status, expires_at_ms, updated_at_ms DESC, coupon_reservation_id
         )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_marketing_coupon_reservation_subject
         ON ai_marketing_coupon_reservation (
            subject_scope, subject_id, updated_at_ms DESC, coupon_reservation_id
         )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_marketing_coupon_redemption (
            coupon_redemption_id TEXT PRIMARY KEY NOT NULL,
            coupon_reservation_id TEXT NOT NULL,
            coupon_code_id TEXT NOT NULL,
            coupon_template_id TEXT NOT NULL,
            redemption_status TEXT NOT NULL,
            order_id TEXT,
            payment_event_id TEXT,
            redeemed_at_ms BIGINT NOT NULL DEFAULT 0,
            updated_at_ms BIGINT NOT NULL DEFAULT 0,
            record_json TEXT NOT NULL
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_ai_marketing_coupon_redemption_reservation
         ON ai_marketing_coupon_redemption (coupon_reservation_id)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_marketing_coupon_redemption_order
         ON ai_marketing_coupon_redemption (order_id, updated_at_ms DESC, coupon_redemption_id)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_marketing_coupon_redemption_payment_event
         ON ai_marketing_coupon_redemption (
            payment_event_id, updated_at_ms DESC, coupon_redemption_id
         )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_marketing_coupon_rollback (
            coupon_rollback_id TEXT PRIMARY KEY NOT NULL,
            coupon_redemption_id TEXT NOT NULL,
            rollback_type TEXT NOT NULL,
            rollback_status TEXT NOT NULL,
            created_at_ms BIGINT NOT NULL DEFAULT 0,
            updated_at_ms BIGINT NOT NULL DEFAULT 0,
            record_json TEXT NOT NULL
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_marketing_coupon_rollback_redemption_status
         ON ai_marketing_coupon_rollback (
            coupon_redemption_id, rollback_status, updated_at_ms DESC, coupon_rollback_id
         )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_marketing_outbox_event (
            marketing_outbox_event_id TEXT PRIMARY KEY NOT NULL,
            aggregate_type TEXT NOT NULL,
            aggregate_id TEXT NOT NULL,
            event_type TEXT NOT NULL,
            status TEXT NOT NULL,
            created_at_ms BIGINT NOT NULL DEFAULT 0,
            updated_at_ms BIGINT NOT NULL DEFAULT 0,
            record_json TEXT NOT NULL
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_marketing_outbox_event_status_created
         ON ai_marketing_outbox_event (status, created_at_ms ASC, marketing_outbox_event_id)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_marketing_outbox_event_aggregate
         ON ai_marketing_outbox_event (
            aggregate_type, aggregate_id, created_at_ms DESC, marketing_outbox_event_id
         )",
    )
    .execute(&pool)
    .await?;
    Ok(())
}
