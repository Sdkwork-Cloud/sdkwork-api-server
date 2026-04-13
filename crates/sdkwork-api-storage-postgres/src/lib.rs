use anyhow::Result;
use async_trait::async_trait;
use sdkwork_api_domain_billing::{
    AccountBenefitLotRecord, AccountBenefitLotStatus, AccountBenefitSourceType, AccountBenefitType,
    AccountHoldAllocationRecord, AccountHoldRecord, AccountHoldStatus,
    AccountLedgerAllocationRecord, AccountLedgerEntryRecord, AccountLedgerEntryType, AccountRecord,
    AccountStatus, AccountType, BillingAccountingMode, BillingEventRecord, LedgerEntry,
    PricingPlanRecord, PricingRateRecord, QuotaPolicy, RequestSettlementRecord,
    RequestSettlementStatus,
};
use sdkwork_api_domain_catalog::{
    normalize_provider_extension_id, Channel, ChannelModelRecord, ModelCapability,
    ModelCatalogEntry, ModelPriceRecord, ProviderChannelBinding, ProxyProvider,
};
use sdkwork_api_domain_commerce::{CommerceOrderRecord, ProjectMembershipRecord};
use sdkwork_api_domain_coupon::CouponCampaign;
use sdkwork_api_domain_credential::UpstreamCredential;
use sdkwork_api_domain_identity::{
    AdminAuditEventRecord, AdminUserRecord, AdminUserRole, ApiKeyGroupRecord,
    CanonicalApiKeyRecord, GatewayApiKeyRecord, IdentityBindingRecord, IdentityUserRecord,
    PortalUserRecord, PortalWorkspaceMembershipRecord,
};
use sdkwork_api_domain_marketing::{
    CouponBenefitRuleRecord, CouponClaimRecord, CouponCodeBatchRecord, CouponCodeRecord,
    CouponRedemptionRecord, CouponTemplateRecord, MarketingAttributionTouchRecord,
    MarketingCampaignRecord, ReferralInviteRecord, ReferralProgramRecord,
};
use sdkwork_api_domain_payment::{
    DisputeRecord, PaymentAttemptRecord, PaymentOrderRecord, PaymentWebhookEventRecord,
    RefundRecord,
};
use sdkwork_api_domain_rate_limit::{
    RateLimitCheckResult, RateLimitPolicy, RateLimitWindowSnapshot,
};
use sdkwork_api_domain_routing::{
    CompiledRoutingSnapshotRecord, ProjectRoutingPreferences, ProviderHealthSnapshot,
    RoutingCandidateAssessment, RoutingDecisionLog, RoutingDecisionSource, RoutingPolicy,
    RoutingProfileRecord, RoutingStrategy,
};
use sdkwork_api_domain_tenant::{Project, Tenant};
use sdkwork_api_domain_usage::{
    RequestMeterFactRecord, RequestMeterMetricRecord, RequestStatus, UsageCaptureStatus,
    UsageRecord,
};
use sdkwork_api_extension_core::{ExtensionInstallation, ExtensionInstance, ExtensionRuntime};
use sdkwork_api_secret_core::SecretEnvelope;
use sdkwork_api_storage_core::{
    AccountKernelCommandBatch, AccountKernelStore, AdminStore,
    ExtensionRuntimeRolloutParticipantRecord, ExtensionRuntimeRolloutRecord, IdentityKernelStore,
    ServiceRuntimeNodeRecord, StandaloneConfigRolloutParticipantRecord,
    StandaloneConfigRolloutRecord, StorageDialect,
};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
use sqlx::{
    postgres::{PgPoolOptions, PgRow},
    Executor, PgPool, Postgres, Row,
};
use std::collections::HashMap;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

const BUILTIN_CHANNEL_SEEDS: [(&str, &str, i32); 5] = [
    ("openai", "OpenAI", 10),
    ("anthropic", "Anthropic", 20),
    ("gemini", "Gemini", 30),
    ("openrouter", "OpenRouter", 40),
    ("ollama", "Ollama", 50),
];

const LEGACY_RENAMED_TABLE_MAPPINGS: [(&str, &str); 23] = [
    ("identity_users", "ai_portal_users"),
    ("admin_users", "ai_admin_users"),
    ("tenant_records", "ai_tenants"),
    ("tenant_projects", "ai_projects"),
    ("coupon_campaigns", "ai_coupon_campaigns"),
    ("routing_policies", "ai_routing_policies"),
    ("routing_policy_providers", "ai_routing_policy_providers"),
    (
        "project_routing_preferences",
        "ai_project_routing_preferences",
    ),
    ("routing_decision_logs", "ai_routing_decision_logs"),
    ("routing_provider_health", "ai_provider_health_records"),
    ("usage_records", "ai_usage_records"),
    ("billing_events", "ai_billing_events"),
    ("billing_ledger_entries", "ai_billing_ledger_entries"),
    ("billing_quota_policies", "ai_billing_quota_policies"),
    ("commerce_orders", "ai_commerce_orders"),
    ("project_memberships", "ai_project_memberships"),
    ("extension_installations", "ai_extension_installations"),
    ("extension_instances", "ai_extension_instances"),
    ("service_runtime_nodes", "ai_service_runtime_nodes"),
    (
        "extension_runtime_rollouts",
        "ai_extension_runtime_rollouts",
    ),
    (
        "extension_runtime_rollout_participants",
        "ai_extension_runtime_rollout_participants",
    ),
    (
        "standalone_config_rollouts",
        "ai_standalone_config_rollouts",
    ),
    (
        "standalone_config_rollout_participants",
        "ai_standalone_config_rollout_participants",
    ),
];

pub fn dialect_name() -> &'static str {
    "postgres"
}

fn provider_channel_bindings(provider: &ProxyProvider) -> Vec<ProviderChannelBinding> {
    if provider.channel_bindings.is_empty() {
        vec![ProviderChannelBinding::primary(
            provider.id.clone(),
            provider.channel_id.clone(),
        )]
    } else {
        provider.channel_bindings.clone()
    }
}

async fn load_routing_policy_provider_ids(pool: &PgPool, policy_id: &str) -> Result<Vec<String>> {
    let rows = sqlx::query_as::<_, (String,)>(
        "SELECT provider_id
         FROM ai_routing_policy_providers
         WHERE policy_id = $1
         ORDER BY position, provider_id",
    )
    .bind(policy_id)
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|(provider_id,)| provider_id).collect())
}

async fn load_provider_channel_bindings(
    pool: &PgPool,
    provider_id: &str,
    channel_id: &str,
) -> Result<Vec<ProviderChannelBinding>> {
    let rows = sqlx::query_as::<_, (String, bool)>(
        "SELECT channel_id, is_primary
         FROM ai_proxy_provider_channel
         WHERE proxy_provider_id = $1
         ORDER BY is_primary DESC, channel_id",
    )
    .bind(provider_id)
    .fetch_all(pool)
    .await?;

    if rows.is_empty() {
        return Ok(vec![ProviderChannelBinding::primary(
            provider_id.to_owned(),
            channel_id.to_owned(),
        )]);
    }

    Ok(rows
        .into_iter()
        .map(|(binding_channel_id, is_primary)| ProviderChannelBinding {
            provider_id: provider_id.to_owned(),
            channel_id: binding_channel_id,
            is_primary,
        })
        .collect())
}

async fn load_provider_channel_bindings_for_providers(
    pool: &PgPool,
    providers: &[(String, String)],
) -> Result<HashMap<String, Vec<ProviderChannelBinding>>> {
    let mut bindings_by_provider = providers
        .iter()
        .map(|(provider_id, _)| (provider_id.clone(), Vec::new()))
        .collect::<HashMap<_, _>>();

    if providers.is_empty() {
        return Ok(bindings_by_provider);
    }

    let mut query = String::from(
        "SELECT proxy_provider_id, channel_id, is_primary
         FROM ai_proxy_provider_channel
         WHERE proxy_provider_id IN (",
    );
    for (index, _) in providers.iter().enumerate() {
        if index > 0 {
            query.push_str(", ");
        }
        query.push('$');
        query.push_str(&(index + 1).to_string());
    }
    query.push_str(") ORDER BY proxy_provider_id, is_primary DESC, channel_id");

    let mut statement = sqlx::query_as::<_, (String, String, bool)>(&query);
    for (provider_id, _) in providers {
        statement = statement.bind(provider_id);
    }
    let rows = statement.fetch_all(pool).await?;

    for (provider_id, channel_id, is_primary) in rows {
        bindings_by_provider
            .entry(provider_id.clone())
            .or_default()
            .push(ProviderChannelBinding {
                provider_id,
                channel_id,
                is_primary,
            });
    }

    for (provider_id, channel_id) in providers {
        let bindings = bindings_by_provider.entry(provider_id.clone()).or_default();
        if bindings.is_empty() {
            bindings.push(ProviderChannelBinding::primary(
                provider_id.clone(),
                channel_id.clone(),
            ));
        }
    }

    Ok(bindings_by_provider)
}

fn encode_model_capabilities(capabilities: &[ModelCapability]) -> Result<String> {
    Ok(serde_json::to_string(capabilities)?)
}

fn decode_model_capabilities(capabilities: &str) -> Result<Vec<ModelCapability>> {
    Ok(serde_json::from_str(capabilities)?)
}

fn encode_extension_config(config: &Value) -> Result<String> {
    Ok(serde_json::to_string(config)?)
}

fn decode_extension_config(config_json: &str) -> Result<Value> {
    Ok(serde_json::from_str(config_json)?)
}

fn encode_routing_assessments(assessments: &[RoutingCandidateAssessment]) -> Result<String> {
    Ok(serde_json::to_string(assessments)?)
}

fn decode_routing_assessments(assessments_json: &str) -> Result<Vec<RoutingCandidateAssessment>> {
    Ok(serde_json::from_str(assessments_json)?)
}

fn encode_string_list(values: &[String]) -> Result<String> {
    Ok(serde_json::to_string(values)?)
}

fn decode_string_list(values_json: &str) -> Result<Vec<String>> {
    Ok(serde_json::from_str(values_json)?)
}

fn encode_json_record<T: Serialize>(record: &T) -> Result<String> {
    Ok(serde_json::to_string(record)?)
}

fn decode_json_record<T: DeserializeOwned>(record_json: String) -> Result<T> {
    Ok(serde_json::from_str(&record_json)?)
}

fn decode_billing_event_row(row: &PgRow) -> Result<BillingEventRecord> {
    Ok(BillingEventRecord {
        event_id: row.try_get("event_id")?,
        tenant_id: row.try_get("tenant_id")?,
        project_id: row.try_get("project_id")?,
        api_key_group_id: row.try_get("api_key_group_id")?,
        capability: row.try_get("capability")?,
        route_key: row.try_get("route_key")?,
        usage_model: row.try_get("usage_model")?,
        provider_id: row.try_get("provider_id")?,
        accounting_mode: BillingAccountingMode::from_str(
            &row.try_get::<String, _>("accounting_mode")?,
        )
        .unwrap_or(BillingAccountingMode::PlatformCredit),
        operation_kind: row.try_get("operation_kind")?,
        modality: row.try_get("modality")?,
        api_key_hash: row.try_get("api_key_hash")?,
        channel_id: row.try_get("channel_id")?,
        reference_id: row.try_get("reference_id")?,
        latency_ms: row
            .try_get::<Option<i64>, _>("latency_ms")?
            .map(u64::try_from)
            .transpose()?,
        units: u64::try_from(row.try_get::<i64, _>("units")?)?,
        request_count: u64::try_from(row.try_get::<i64, _>("request_count")?)?,
        input_tokens: u64::try_from(row.try_get::<i64, _>("input_tokens")?)?,
        output_tokens: u64::try_from(row.try_get::<i64, _>("output_tokens")?)?,
        total_tokens: u64::try_from(row.try_get::<i64, _>("total_tokens")?)?,
        cache_read_tokens: u64::try_from(row.try_get::<i64, _>("cache_read_tokens")?)?,
        cache_write_tokens: u64::try_from(row.try_get::<i64, _>("cache_write_tokens")?)?,
        image_count: u64::try_from(row.try_get::<i64, _>("image_count")?)?,
        audio_seconds: row.try_get("audio_seconds")?,
        video_seconds: row.try_get("video_seconds")?,
        music_seconds: row.try_get("music_seconds")?,
        upstream_cost: row.try_get("upstream_cost")?,
        customer_charge: row.try_get("customer_charge")?,
        applied_routing_profile_id: row.try_get("applied_routing_profile_id")?,
        compiled_routing_snapshot_id: row.try_get("compiled_routing_snapshot_id")?,
        fallback_reason: row.try_get("fallback_reason")?,
        created_at_ms: u64::try_from(row.try_get::<i64, _>("created_at_ms")?)?,
    })
}

fn current_timestamp_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| i64::try_from(duration.as_millis()).unwrap_or(i64::MAX))
        .unwrap_or_default()
}

type PortalUserRow = (
    String,
    String,
    String,
    String,
    String,
    String,
    String,
    bool,
    i64,
);

type PortalWorkspaceMembershipRow = (String, String, String, String, String, i64);
type PaymentOrderRow = (
    String,
    String,
    String,
    String,
    String,
    String,
    i64,
    String,
    String,
    String,
    i64,
    i64,
);
type PaymentWebhookEventRow = (
    String,
    String,
    String,
    Option<String>,
    Option<String>,
    String,
    String,
    String,
    i64,
);
type PaymentAttemptRow = (
    String,
    String,
    String,
    String,
    String,
    String,
    String,
    i64,
    Option<String>,
    Option<String>,
    Option<String>,
    i64,
    i64,
);
type RefundRow = (
    String,
    String,
    String,
    String,
    String,
    String,
    i64,
    Option<String>,
    Option<String>,
    Option<String>,
    i64,
    i64,
);
type DisputeRow = (
    String,
    String,
    String,
    String,
    String,
    String,
    String,
    i64,
    Option<i64>,
    i64,
    i64,
);

type AdminUserRow = (String, String, String, String, String, String, bool, i64);
type AdminAuditEventRow = (
    String,
    String,
    String,
    String,
    String,
    String,
    String,
    String,
    Option<String>,
    Option<String>,
    Option<String>,
    i64,
);

type CouponRow = (
    String,
    String,
    String,
    String,
    i64,
    bool,
    String,
    String,
    i64,
);

type CredentialRow = (
    String,
    String,
    String,
    String,
    Option<String>,
    Option<String>,
    Option<String>,
);

type ChannelModelRow = (String, String, String, String, bool, Option<i64>, String);

type ModelPriceRow = (
    String,
    String,
    String,
    String,
    String,
    f64,
    f64,
    f64,
    f64,
    f64,
    bool,
);

fn decode_portal_user_row(row: Option<PortalUserRow>) -> Result<Option<PortalUserRecord>> {
    row.map(
        |(
            id,
            email,
            display_name,
            password_salt,
            password_hash,
            workspace_tenant_id,
            workspace_project_id,
            active,
            created_at_ms,
        )| {
            Ok(PortalUserRecord {
                id,
                email,
                display_name,
                password_salt,
                password_hash,
                workspace_tenant_id,
                workspace_project_id,
                active,
                created_at_ms: u64::try_from(created_at_ms)?,
            })
        },
    )
    .transpose()
}

fn decode_portal_workspace_membership_row(
    row: Option<PortalWorkspaceMembershipRow>,
) -> Result<Option<PortalWorkspaceMembershipRecord>> {
    row.map(
        |(membership_id, user_id, tenant_id, project_id, role, created_at_ms)| {
            Ok(PortalWorkspaceMembershipRecord {
                membership_id,
                user_id,
                tenant_id,
                project_id,
                role,
                created_at_ms: u64::try_from(created_at_ms)?,
            })
        },
    )
    .transpose()
}

fn decode_payment_order_row(row: Option<PaymentOrderRow>) -> Result<Option<PaymentOrderRecord>> {
    row.map(
        |(
            payment_order_id,
            commerce_order_id,
            project_id,
            user_id,
            provider,
            currency_code,
            amount_cents,
            status,
            provider_reference_id,
            checkout_url,
            created_at_ms,
            updated_at_ms,
        )| {
            Ok(PaymentOrderRecord {
                payment_order_id,
                commerce_order_id,
                project_id,
                user_id,
                provider,
                currency_code,
                amount_cents: u64::try_from(amount_cents)?,
                status,
                provider_reference_id,
                checkout_url,
                created_at_ms: u64::try_from(created_at_ms)?,
                updated_at_ms: u64::try_from(updated_at_ms)?,
            })
        },
    )
    .transpose()
}

fn decode_payment_webhook_event_row(
    row: Option<PaymentWebhookEventRow>,
) -> Result<Option<PaymentWebhookEventRecord>> {
    row.map(
        |(
            payment_webhook_event_id,
            provider,
            provider_event_id,
            payment_order_id,
            commerce_order_id,
            event_type,
            status,
            payload_json,
            created_at_ms,
        )| {
            Ok(PaymentWebhookEventRecord {
                payment_webhook_event_id,
                provider,
                provider_event_id,
                payment_order_id,
                commerce_order_id,
                event_type,
                status,
                payload_json,
                created_at_ms: u64::try_from(created_at_ms)?,
            })
        },
    )
    .transpose()
}

fn decode_payment_attempt_row(
    row: Option<PaymentAttemptRow>,
) -> Result<Option<PaymentAttemptRecord>> {
    row.map(
        |(
            payment_attempt_id,
            payment_order_id,
            provider,
            provider_attempt_id,
            attempt_kind,
            status,
            currency_code,
            amount_cents,
            idempotency_key,
            error_code,
            error_message,
            created_at_ms,
            updated_at_ms,
        )| {
            Ok(PaymentAttemptRecord {
                payment_attempt_id,
                payment_order_id,
                provider,
                provider_attempt_id,
                attempt_kind,
                status,
                currency_code,
                amount_cents: u64::try_from(amount_cents)?,
                idempotency_key,
                error_code,
                error_message,
                created_at_ms: u64::try_from(created_at_ms)?,
                updated_at_ms: u64::try_from(updated_at_ms)?,
            })
        },
    )
    .transpose()
}

fn decode_refund_row(row: Option<RefundRow>) -> Result<Option<RefundRecord>> {
    row.map(
        |(
            refund_id,
            payment_order_id,
            provider,
            provider_refund_id,
            status,
            currency_code,
            amount_cents,
            reason,
            failure_code,
            failure_message,
            created_at_ms,
            updated_at_ms,
        )| {
            Ok(RefundRecord {
                refund_id,
                payment_order_id,
                provider,
                provider_refund_id,
                status,
                currency_code,
                amount_cents: u64::try_from(amount_cents)?,
                reason,
                failure_code,
                failure_message,
                created_at_ms: u64::try_from(created_at_ms)?,
                updated_at_ms: u64::try_from(updated_at_ms)?,
            })
        },
    )
    .transpose()
}

fn decode_dispute_row(row: Option<DisputeRow>) -> Result<Option<DisputeRecord>> {
    row.map(
        |(
            dispute_id,
            payment_order_id,
            provider,
            provider_dispute_id,
            status,
            reason,
            currency_code,
            amount_cents,
            evidence_due_at_ms,
            created_at_ms,
            updated_at_ms,
        )| {
            Ok(DisputeRecord {
                dispute_id,
                payment_order_id,
                provider,
                provider_dispute_id,
                status,
                reason,
                currency_code,
                amount_cents: u64::try_from(amount_cents)?,
                evidence_due_at_ms: evidence_due_at_ms.map(u64::try_from).transpose()?,
                created_at_ms: u64::try_from(created_at_ms)?,
                updated_at_ms: u64::try_from(updated_at_ms)?,
            })
        },
    )
    .transpose()
}

fn decode_admin_user_row(row: Option<AdminUserRow>) -> Result<Option<AdminUserRecord>> {
    row.map(
        |(id, email, display_name, password_salt, password_hash, role, active, created_at_ms)| {
            Ok(AdminUserRecord {
                id,
                email,
                display_name,
                password_salt,
                password_hash,
                role: role.parse::<AdminUserRole>().unwrap_or_default(),
                active,
                created_at_ms: u64::try_from(created_at_ms)?,
            })
        },
    )
    .transpose()
}

fn decode_admin_audit_event_row(
    row: Option<AdminAuditEventRow>,
) -> Result<Option<AdminAuditEventRecord>> {
    row.map(
        |(
            event_id,
            actor_user_id,
            actor_email,
            actor_role,
            action,
            resource_type,
            resource_id,
            approval_scope,
            target_tenant_id,
            target_project_id,
            target_provider_id,
            created_at_ms,
        )| {
            Ok(AdminAuditEventRecord::new(
                event_id,
                actor_user_id,
                actor_email,
                actor_role.parse::<AdminUserRole>().unwrap_or_default(),
                action,
                resource_type,
                resource_id,
                approval_scope,
                u64::try_from(created_at_ms)?,
            )
            .with_target_tenant_id_option(target_tenant_id)
            .with_target_project_id_option(target_project_id)
            .with_target_provider_id_option(target_provider_id))
        },
    )
    .transpose()
}

fn decode_coupon_row(row: Option<CouponRow>) -> Result<Option<CouponCampaign>> {
    row.map(
        |(
            id,
            code,
            discount_label,
            audience,
            remaining,
            active,
            note,
            expires_on,
            created_at_ms,
        )| {
            Ok(CouponCampaign {
                id,
                code,
                discount_label,
                audience,
                remaining: u64::try_from(remaining)?,
                active,
                note,
                expires_on,
                created_at_ms: u64::try_from(created_at_ms)?,
            })
        },
    )
    .transpose()
}

fn decode_credential_row(row: CredentialRow) -> UpstreamCredential {
    let (
        tenant_id,
        provider_id,
        key_reference,
        secret_backend,
        secret_local_file,
        secret_keyring_service,
        secret_master_key_id,
    ) = row;

    UpstreamCredential {
        tenant_id,
        provider_id,
        key_reference,
        secret_backend,
        secret_local_file,
        secret_keyring_service,
        secret_master_key_id,
    }
}

type RoutingDecisionLogRow = PgRow;

fn decode_routing_profile_row(row: PgRow) -> Result<RoutingProfileRecord> {
    Ok(RoutingProfileRecord::new(
        row.try_get::<String, _>("profile_id")?,
        row.try_get::<String, _>("tenant_id")?,
        row.try_get::<String, _>("project_id")?,
        row.try_get::<String, _>("name")?,
        row.try_get::<String, _>("slug")?,
    )
    .with_description_option(row.try_get::<Option<String>, _>("description")?)
    .with_active(row.try_get::<bool, _>("active")?)
    .with_strategy(
        RoutingStrategy::from_str(&row.try_get::<String, _>("strategy")?)
            .unwrap_or(RoutingStrategy::DeterministicPriority),
    )
    .with_ordered_provider_ids(decode_string_list(
        &row.try_get::<String, _>("ordered_provider_ids_json")?,
    )?)
    .with_default_provider_id_option(row.try_get::<Option<String>, _>("default_provider_id")?)
    .with_max_cost_option(row.try_get::<Option<f64>, _>("max_cost")?)
    .with_max_latency_ms_option(
        row.try_get::<Option<i64>, _>("max_latency_ms")?
            .map(u64::try_from)
            .transpose()?,
    )
    .with_require_healthy(row.try_get::<bool, _>("require_healthy")?)
    .with_preferred_region_option(row.try_get::<Option<String>, _>("preferred_region")?)
    .with_created_at_ms(u64::try_from(row.try_get::<i64, _>("created_at_ms")?)?)
    .with_updated_at_ms(u64::try_from(row.try_get::<i64, _>("updated_at_ms")?)?))
}

fn decode_compiled_routing_snapshot_row(row: PgRow) -> Result<CompiledRoutingSnapshotRecord> {
    Ok(CompiledRoutingSnapshotRecord::new(
        row.try_get::<String, _>("snapshot_id")?,
        row.try_get::<String, _>("capability")?,
        row.try_get::<String, _>("route_key")?,
    )
    .with_tenant_id_option(row.try_get::<Option<String>, _>("tenant_id")?)
    .with_project_id_option(row.try_get::<Option<String>, _>("project_id")?)
    .with_api_key_group_id_option(row.try_get::<Option<String>, _>("api_key_group_id")?)
    .with_matched_policy_id_option(row.try_get::<Option<String>, _>("matched_policy_id")?)
    .with_project_routing_preferences_project_id_option(
        row.try_get::<Option<String>, _>("project_routing_preferences_project_id")?,
    )
    .with_applied_routing_profile_id_option(
        row.try_get::<Option<String>, _>("applied_routing_profile_id")?,
    )
    .with_strategy(row.try_get::<String, _>("strategy")?)
    .with_ordered_provider_ids(decode_string_list(
        &row.try_get::<String, _>("ordered_provider_ids_json")?,
    )?)
    .with_default_provider_id_option(row.try_get::<Option<String>, _>("default_provider_id")?)
    .with_max_cost_option(row.try_get::<Option<f64>, _>("max_cost")?)
    .with_max_latency_ms_option(
        row.try_get::<Option<i64>, _>("max_latency_ms")?
            .map(u64::try_from)
            .transpose()?,
    )
    .with_require_healthy(row.try_get::<bool, _>("require_healthy")?)
    .with_preferred_region_option(row.try_get::<Option<String>, _>("preferred_region")?)
    .with_created_at_ms(u64::try_from(row.try_get::<i64, _>("created_at_ms")?)?)
    .with_updated_at_ms(u64::try_from(row.try_get::<i64, _>("updated_at_ms")?)?))
}

fn decode_routing_decision_log_row(row: RoutingDecisionLogRow) -> Result<RoutingDecisionLog> {
    Ok(RoutingDecisionLog::new(
        row.try_get::<String, _>("decision_id")?,
        RoutingDecisionSource::from_str(&row.try_get::<String, _>("decision_source")?)
            .unwrap_or(RoutingDecisionSource::Gateway),
        row.try_get::<String, _>("capability")?,
        row.try_get::<String, _>("route_key")?,
        row.try_get::<String, _>("selected_provider_id")?,
        row.try_get::<String, _>("strategy")?,
        u64::try_from(row.try_get::<i64, _>("created_at_ms")?)?,
    )
    .with_tenant_id_option(row.try_get::<Option<String>, _>("tenant_id")?)
    .with_project_id_option(row.try_get::<Option<String>, _>("project_id")?)
    .with_api_key_group_id_option(row.try_get::<Option<String>, _>("api_key_group_id")?)
    .with_matched_policy_id_option(row.try_get::<Option<String>, _>("matched_policy_id")?)
    .with_applied_routing_profile_id_option(
        row.try_get::<Option<String>, _>("applied_routing_profile_id")?,
    )
    .with_compiled_routing_snapshot_id_option(
        row.try_get::<Option<String>, _>("compiled_routing_snapshot_id")?,
    )
    .with_selection_seed_option(
        row.try_get::<Option<i64>, _>("selection_seed")?
            .map(u64::try_from)
            .transpose()?,
    )
    .with_selection_reason_option(row.try_get::<Option<String>, _>("selection_reason")?)
    .with_fallback_reason_option(row.try_get::<Option<String>, _>("fallback_reason")?)
    .with_requested_region_option(row.try_get::<Option<String>, _>("requested_region")?)
    .with_slo_state(
        row.try_get::<bool, _>("slo_applied")?,
        row.try_get::<bool, _>("slo_degraded")?,
    )
    .with_assessments(decode_routing_assessments(
        &row.try_get::<String, _>("assessments_json")?,
    )?))
}

fn decode_channel_model_row(row: ChannelModelRow) -> Result<ChannelModelRecord> {
    let (
        channel_id,
        model_id,
        model_display_name,
        capabilities_json,
        streaming_enabled,
        context_window,
        description,
    ) = row;

    let mut record = ChannelModelRecord::new(channel_id, model_id, model_display_name)
        .with_context_window_option(context_window.map(u64::try_from).transpose()?)
        .with_streaming(streaming_enabled)
        .with_description_option((!description.is_empty()).then_some(description));
    for capability in decode_model_capabilities(&capabilities_json)? {
        record = record.with_capability(capability);
    }
    Ok(record)
}

fn decode_model_price_row(row: ModelPriceRow) -> ModelPriceRecord {
    let (
        channel_id,
        model_id,
        proxy_provider_id,
        currency_code,
        price_unit,
        input_price,
        output_price,
        cache_read_price,
        cache_write_price,
        request_price,
        is_active,
    ) = row;

    ModelPriceRecord::new(channel_id, model_id, proxy_provider_id)
        .with_currency_code(currency_code)
        .with_price_unit(price_unit)
        .with_input_price(input_price)
        .with_output_price(output_price)
        .with_cache_read_price(cache_read_price)
        .with_cache_write_price(cache_write_price)
        .with_request_price(request_price)
        .with_active(is_active)
}

async fn postgres_relation_kind(pool: &PgPool, relation_name: &str) -> Result<Option<String>> {
    let row = sqlx::query_as::<_, (String,)>(
        "SELECT c.relkind::text
         FROM pg_class c
         INNER JOIN pg_namespace n
             ON n.oid = c.relnamespace
         WHERE n.nspname = current_schema()
           AND c.relname = $1",
    )
    .bind(relation_name)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|(kind,)| kind))
}

async fn postgres_table_columns(pool: &PgPool, table_name: &str) -> Result<Vec<String>> {
    let rows = sqlx::query_as::<_, (String,)>(
        "SELECT column_name
         FROM information_schema.columns
         WHERE table_schema = current_schema()
           AND table_name = $1
         ORDER BY ordinal_position",
    )
    .bind(table_name)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(|(column_name,)| column_name).collect())
}

async fn ensure_postgres_column_if_table_exists(
    pool: &PgPool,
    table_name: &str,
    alter_statement: &str,
) -> Result<()> {
    if postgres_relation_kind(pool, table_name).await?.as_deref() == Some("r") {
        sqlx::query(alter_statement).execute(pool).await?;
    }
    Ok(())
}

async fn migrate_postgres_legacy_table_with_common_columns(
    pool: &PgPool,
    legacy_table_name: &str,
    canonical_table_name: &str,
) -> Result<()> {
    if postgres_relation_kind(pool, legacy_table_name)
        .await?
        .as_deref()
        != Some("r")
    {
        return Ok(());
    }

    let legacy_columns = postgres_table_columns(pool, legacy_table_name).await?;
    let canonical_columns = postgres_table_columns(pool, canonical_table_name).await?;
    let common_columns: Vec<String> = canonical_columns
        .into_iter()
        .filter(|column_name| legacy_columns.contains(column_name))
        .collect();

    if !common_columns.is_empty() {
        let column_list = common_columns.join(", ");
        let insert = format!(
            "INSERT INTO {canonical_table_name} ({column_list})
             SELECT {column_list} FROM {legacy_table_name}
             ON CONFLICT DO NOTHING"
        );
        sqlx::query(&insert).execute(pool).await?;
    }

    let drop_table = format!("DROP TABLE {legacy_table_name}");
    sqlx::query(&drop_table).execute(pool).await?;
    Ok(())
}

async fn recreate_postgres_compatibility_view(
    pool: &PgPool,
    legacy_name: &str,
    select_sql: &str,
) -> Result<()> {
    match postgres_relation_kind(pool, legacy_name).await?.as_deref() {
        Some("r") => {
            let drop_table = format!("DROP TABLE {legacy_name}");
            sqlx::query(&drop_table).execute(pool).await?;
        }
        Some("v") => {
            let drop_view = format!("DROP VIEW {legacy_name}");
            sqlx::query(&drop_view).execute(pool).await?;
        }
        _ => {}
    }

    let create_view = format!("CREATE VIEW {legacy_name} AS {select_sql}");
    sqlx::query(&create_view).execute(pool).await?;
    Ok(())
}

pub async fn run_migrations(url: &str) -> Result<PgPool> {
    let pool = PgPoolOptions::new().max_connections(5).connect(url).await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_portal_users (
            id TEXT PRIMARY KEY NOT NULL,
            email TEXT NOT NULL,
            display_name TEXT NOT NULL DEFAULT '',
            password_salt TEXT NOT NULL DEFAULT '',
            password_hash TEXT NOT NULL DEFAULT '',
            workspace_tenant_id TEXT NOT NULL DEFAULT '',
            workspace_project_id TEXT NOT NULL DEFAULT '',
            active BOOLEAN NOT NULL DEFAULT TRUE,
            created_at_ms BIGINT NOT NULL DEFAULT 0
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_portal_users ADD COLUMN IF NOT EXISTS display_name TEXT NOT NULL DEFAULT ''",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_portal_users ADD COLUMN IF NOT EXISTS password_salt TEXT NOT NULL DEFAULT ''",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_portal_users ADD COLUMN IF NOT EXISTS password_hash TEXT NOT NULL DEFAULT ''",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_portal_users ADD COLUMN IF NOT EXISTS workspace_tenant_id TEXT NOT NULL DEFAULT ''",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_portal_users ADD COLUMN IF NOT EXISTS workspace_project_id TEXT NOT NULL DEFAULT ''",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_portal_users ADD COLUMN IF NOT EXISTS active BOOLEAN NOT NULL DEFAULT TRUE",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_portal_users ADD COLUMN IF NOT EXISTS created_at_ms BIGINT NOT NULL DEFAULT 0",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_ai_portal_users_email ON ai_portal_users (email)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_portal_workspace_memberships (
            membership_id TEXT PRIMARY KEY NOT NULL,
            user_id TEXT NOT NULL,
            tenant_id TEXT NOT NULL,
            project_id TEXT NOT NULL,
            role TEXT NOT NULL DEFAULT 'member',
            created_at_ms BIGINT NOT NULL DEFAULT 0
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_portal_workspace_memberships ADD COLUMN IF NOT EXISTS user_id TEXT NOT NULL DEFAULT ''",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_portal_workspace_memberships ADD COLUMN IF NOT EXISTS tenant_id TEXT NOT NULL DEFAULT ''",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_portal_workspace_memberships ADD COLUMN IF NOT EXISTS project_id TEXT NOT NULL DEFAULT ''",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_portal_workspace_memberships ADD COLUMN IF NOT EXISTS role TEXT NOT NULL DEFAULT 'member'",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_portal_workspace_memberships ADD COLUMN IF NOT EXISTS created_at_ms BIGINT NOT NULL DEFAULT 0",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_ai_portal_workspace_memberships_scope
         ON ai_portal_workspace_memberships (user_id, tenant_id, project_id)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_portal_workspace_memberships_user
         ON ai_portal_workspace_memberships (user_id)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_payment_orders (
            payment_order_id TEXT PRIMARY KEY NOT NULL,
            commerce_order_id TEXT NOT NULL UNIQUE,
            project_id TEXT NOT NULL,
            user_id TEXT NOT NULL,
            provider TEXT NOT NULL,
            currency_code TEXT NOT NULL DEFAULT 'usd',
            amount_cents BIGINT NOT NULL DEFAULT 0,
            status TEXT NOT NULL,
            provider_reference_id TEXT NOT NULL DEFAULT '',
            checkout_url TEXT NOT NULL DEFAULT '',
            created_at_ms BIGINT NOT NULL DEFAULT 0,
            updated_at_ms BIGINT NOT NULL DEFAULT 0
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_payment_orders ADD COLUMN IF NOT EXISTS commerce_order_id TEXT NOT NULL DEFAULT ''",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_payment_orders ADD COLUMN IF NOT EXISTS project_id TEXT NOT NULL DEFAULT ''",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_payment_orders ADD COLUMN IF NOT EXISTS user_id TEXT NOT NULL DEFAULT ''",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_payment_orders ADD COLUMN IF NOT EXISTS provider TEXT NOT NULL DEFAULT ''",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_payment_orders ADD COLUMN IF NOT EXISTS currency_code TEXT NOT NULL DEFAULT 'usd'",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_payment_orders ADD COLUMN IF NOT EXISTS amount_cents BIGINT NOT NULL DEFAULT 0",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_payment_orders ADD COLUMN IF NOT EXISTS status TEXT NOT NULL DEFAULT ''",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_payment_orders ADD COLUMN IF NOT EXISTS provider_reference_id TEXT NOT NULL DEFAULT ''",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_payment_orders ADD COLUMN IF NOT EXISTS checkout_url TEXT NOT NULL DEFAULT ''",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_payment_orders ADD COLUMN IF NOT EXISTS created_at_ms BIGINT NOT NULL DEFAULT 0",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_payment_orders ADD COLUMN IF NOT EXISTS updated_at_ms BIGINT NOT NULL DEFAULT 0",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_ai_payment_orders_commerce_order
         ON ai_payment_orders (commerce_order_id)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_payment_orders_provider_reference
         ON ai_payment_orders (provider, provider_reference_id)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_payment_attempts (
            payment_attempt_id TEXT PRIMARY KEY NOT NULL,
            payment_order_id TEXT NOT NULL,
            provider TEXT NOT NULL,
            provider_attempt_id TEXT NOT NULL,
            attempt_kind TEXT NOT NULL,
            status TEXT NOT NULL,
            currency_code TEXT NOT NULL DEFAULT 'usd',
            amount_cents BIGINT NOT NULL DEFAULT 0,
            idempotency_key TEXT,
            error_code TEXT,
            error_message TEXT,
            created_at_ms BIGINT NOT NULL DEFAULT 0,
            updated_at_ms BIGINT NOT NULL DEFAULT 0
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_payment_attempts ADD COLUMN IF NOT EXISTS payment_order_id TEXT NOT NULL DEFAULT ''",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_payment_attempts ADD COLUMN IF NOT EXISTS provider TEXT NOT NULL DEFAULT ''",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_payment_attempts ADD COLUMN IF NOT EXISTS provider_attempt_id TEXT NOT NULL DEFAULT ''",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_payment_attempts ADD COLUMN IF NOT EXISTS attempt_kind TEXT NOT NULL DEFAULT ''",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_payment_attempts ADD COLUMN IF NOT EXISTS status TEXT NOT NULL DEFAULT ''",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_payment_attempts ADD COLUMN IF NOT EXISTS currency_code TEXT NOT NULL DEFAULT 'usd'",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_payment_attempts ADD COLUMN IF NOT EXISTS amount_cents BIGINT NOT NULL DEFAULT 0",
    )
    .execute(&pool)
    .await?;
    sqlx::query("ALTER TABLE ai_payment_attempts ADD COLUMN IF NOT EXISTS idempotency_key TEXT")
        .execute(&pool)
        .await?;
    sqlx::query("ALTER TABLE ai_payment_attempts ADD COLUMN IF NOT EXISTS error_code TEXT")
        .execute(&pool)
        .await?;
    sqlx::query("ALTER TABLE ai_payment_attempts ADD COLUMN IF NOT EXISTS error_message TEXT")
        .execute(&pool)
        .await?;
    sqlx::query(
        "ALTER TABLE ai_payment_attempts ADD COLUMN IF NOT EXISTS created_at_ms BIGINT NOT NULL DEFAULT 0",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_payment_attempts ADD COLUMN IF NOT EXISTS updated_at_ms BIGINT NOT NULL DEFAULT 0",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_payment_attempts_payment_order
         ON ai_payment_attempts (payment_order_id, created_at_ms DESC)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_ai_payment_attempts_provider_reference
         ON ai_payment_attempts (provider, provider_attempt_id)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_payment_refunds (
            refund_id TEXT PRIMARY KEY NOT NULL,
            payment_order_id TEXT NOT NULL,
            provider TEXT NOT NULL,
            provider_refund_id TEXT NOT NULL,
            status TEXT NOT NULL,
            currency_code TEXT NOT NULL DEFAULT 'usd',
            amount_cents BIGINT NOT NULL DEFAULT 0,
            reason TEXT,
            failure_code TEXT,
            failure_message TEXT,
            created_at_ms BIGINT NOT NULL DEFAULT 0,
            updated_at_ms BIGINT NOT NULL DEFAULT 0
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_payment_refunds ADD COLUMN IF NOT EXISTS payment_order_id TEXT NOT NULL DEFAULT ''",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_payment_refunds ADD COLUMN IF NOT EXISTS provider TEXT NOT NULL DEFAULT ''",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_payment_refunds ADD COLUMN IF NOT EXISTS provider_refund_id TEXT NOT NULL DEFAULT ''",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_payment_refunds ADD COLUMN IF NOT EXISTS status TEXT NOT NULL DEFAULT ''",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_payment_refunds ADD COLUMN IF NOT EXISTS currency_code TEXT NOT NULL DEFAULT 'usd'",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_payment_refunds ADD COLUMN IF NOT EXISTS amount_cents BIGINT NOT NULL DEFAULT 0",
    )
    .execute(&pool)
    .await?;
    sqlx::query("ALTER TABLE ai_payment_refunds ADD COLUMN IF NOT EXISTS reason TEXT")
        .execute(&pool)
        .await?;
    sqlx::query("ALTER TABLE ai_payment_refunds ADD COLUMN IF NOT EXISTS failure_code TEXT")
        .execute(&pool)
        .await?;
    sqlx::query("ALTER TABLE ai_payment_refunds ADD COLUMN IF NOT EXISTS failure_message TEXT")
        .execute(&pool)
        .await?;
    sqlx::query(
        "ALTER TABLE ai_payment_refunds ADD COLUMN IF NOT EXISTS created_at_ms BIGINT NOT NULL DEFAULT 0",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_payment_refunds ADD COLUMN IF NOT EXISTS updated_at_ms BIGINT NOT NULL DEFAULT 0",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_payment_refunds_payment_order
         ON ai_payment_refunds (payment_order_id, created_at_ms DESC)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_ai_payment_refunds_provider_reference
         ON ai_payment_refunds (provider, provider_refund_id)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_payment_disputes (
            dispute_id TEXT PRIMARY KEY NOT NULL,
            payment_order_id TEXT NOT NULL,
            provider TEXT NOT NULL,
            provider_dispute_id TEXT NOT NULL,
            status TEXT NOT NULL,
            reason TEXT NOT NULL DEFAULT '',
            currency_code TEXT NOT NULL DEFAULT 'usd',
            amount_cents BIGINT NOT NULL DEFAULT 0,
            evidence_due_at_ms BIGINT,
            created_at_ms BIGINT NOT NULL DEFAULT 0,
            updated_at_ms BIGINT NOT NULL DEFAULT 0
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_payment_disputes ADD COLUMN IF NOT EXISTS payment_order_id TEXT NOT NULL DEFAULT ''",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_payment_disputes ADD COLUMN IF NOT EXISTS provider TEXT NOT NULL DEFAULT ''",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_payment_disputes ADD COLUMN IF NOT EXISTS provider_dispute_id TEXT NOT NULL DEFAULT ''",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_payment_disputes ADD COLUMN IF NOT EXISTS status TEXT NOT NULL DEFAULT ''",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_payment_disputes ADD COLUMN IF NOT EXISTS reason TEXT NOT NULL DEFAULT ''",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_payment_disputes ADD COLUMN IF NOT EXISTS currency_code TEXT NOT NULL DEFAULT 'usd'",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_payment_disputes ADD COLUMN IF NOT EXISTS amount_cents BIGINT NOT NULL DEFAULT 0",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_payment_disputes ADD COLUMN IF NOT EXISTS evidence_due_at_ms BIGINT",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_payment_disputes ADD COLUMN IF NOT EXISTS created_at_ms BIGINT NOT NULL DEFAULT 0",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_payment_disputes ADD COLUMN IF NOT EXISTS updated_at_ms BIGINT NOT NULL DEFAULT 0",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_payment_disputes_payment_order
         ON ai_payment_disputes (payment_order_id, created_at_ms DESC)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_ai_payment_disputes_provider_reference
         ON ai_payment_disputes (provider, provider_dispute_id)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_payment_webhook_events (
            payment_webhook_event_id TEXT PRIMARY KEY NOT NULL,
            provider TEXT NOT NULL,
            provider_event_id TEXT NOT NULL,
            payment_order_id TEXT,
            commerce_order_id TEXT,
            event_type TEXT NOT NULL,
            status TEXT NOT NULL,
            payload_json TEXT NOT NULL DEFAULT '{}',
            created_at_ms BIGINT NOT NULL DEFAULT 0
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_payment_webhook_events ADD COLUMN IF NOT EXISTS provider TEXT NOT NULL DEFAULT ''",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_payment_webhook_events ADD COLUMN IF NOT EXISTS provider_event_id TEXT NOT NULL DEFAULT ''",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_payment_webhook_events ADD COLUMN IF NOT EXISTS payment_order_id TEXT",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_payment_webhook_events ADD COLUMN IF NOT EXISTS commerce_order_id TEXT",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_payment_webhook_events ADD COLUMN IF NOT EXISTS event_type TEXT NOT NULL DEFAULT ''",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_payment_webhook_events ADD COLUMN IF NOT EXISTS status TEXT NOT NULL DEFAULT ''",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_payment_webhook_events ADD COLUMN IF NOT EXISTS payload_json TEXT NOT NULL DEFAULT '{}'",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_payment_webhook_events ADD COLUMN IF NOT EXISTS created_at_ms BIGINT NOT NULL DEFAULT 0",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_ai_payment_webhook_events_provider_event
         ON ai_payment_webhook_events (provider, provider_event_id)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_admin_users (
            id TEXT PRIMARY KEY NOT NULL,
            email TEXT NOT NULL,
            display_name TEXT NOT NULL DEFAULT '',
            password_salt TEXT NOT NULL DEFAULT '',
            password_hash TEXT NOT NULL DEFAULT '',
            role TEXT NOT NULL DEFAULT 'super_admin',
            active BOOLEAN NOT NULL DEFAULT TRUE,
            created_at_ms BIGINT NOT NULL DEFAULT 0
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_admin_users ADD COLUMN IF NOT EXISTS display_name TEXT NOT NULL DEFAULT ''",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_admin_users ADD COLUMN IF NOT EXISTS password_salt TEXT NOT NULL DEFAULT ''",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_admin_users ADD COLUMN IF NOT EXISTS password_hash TEXT NOT NULL DEFAULT ''",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_admin_users ADD COLUMN IF NOT EXISTS role TEXT NOT NULL DEFAULT 'super_admin'",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_admin_users ADD COLUMN IF NOT EXISTS active BOOLEAN NOT NULL DEFAULT TRUE",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_admin_users ADD COLUMN IF NOT EXISTS created_at_ms BIGINT NOT NULL DEFAULT 0",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_ai_admin_users_email ON ai_admin_users (email)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_admin_audit_events (
            event_id TEXT PRIMARY KEY NOT NULL,
            actor_user_id TEXT NOT NULL,
            actor_email TEXT NOT NULL,
            actor_role TEXT NOT NULL,
            action TEXT NOT NULL,
            resource_type TEXT NOT NULL,
            resource_id TEXT NOT NULL,
            approval_scope TEXT NOT NULL,
            target_tenant_id TEXT,
            target_project_id TEXT,
            target_provider_id TEXT,
            created_at_ms BIGINT NOT NULL
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_admin_audit_events_created_at
         ON ai_admin_audit_events (created_at_ms DESC, event_id DESC)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_admin_audit_events_resource
         ON ai_admin_audit_events (resource_type, resource_id, created_at_ms DESC)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_tenants (
            id TEXT PRIMARY KEY NOT NULL,
            name TEXT NOT NULL
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_projects (
            id TEXT PRIMARY KEY NOT NULL,
            tenant_id TEXT NOT NULL,
            name TEXT NOT NULL
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_user (
            user_id BIGINT PRIMARY KEY NOT NULL,
            tenant_id BIGINT NOT NULL,
            organization_id BIGINT NOT NULL DEFAULT 0,
            external_user_ref TEXT,
            username TEXT,
            display_name TEXT,
            email TEXT,
            status TEXT NOT NULL DEFAULT 'active',
            created_at_ms BIGINT NOT NULL DEFAULT 0,
            updated_at_ms BIGINT NOT NULL DEFAULT 0
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_user_scope
         ON ai_user (tenant_id, organization_id, user_id)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_user_email
         ON ai_user (tenant_id, organization_id, email)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_api_key (
            api_key_id BIGINT PRIMARY KEY NOT NULL,
            tenant_id BIGINT NOT NULL,
            organization_id BIGINT NOT NULL DEFAULT 0,
            user_id BIGINT NOT NULL,
            key_prefix TEXT NOT NULL DEFAULT '',
            key_hash TEXT NOT NULL,
            display_name TEXT NOT NULL DEFAULT '',
            status TEXT NOT NULL DEFAULT 'active',
            expires_at_ms BIGINT,
            last_used_at_ms BIGINT,
            rotated_from_api_key_id BIGINT,
            created_at_ms BIGINT NOT NULL DEFAULT 0,
            updated_at_ms BIGINT NOT NULL DEFAULT 0
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_ai_api_key_hash
         ON ai_api_key (key_hash)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_api_key_user_status
         ON ai_api_key (tenant_id, organization_id, user_id, status)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_identity_binding (
            identity_binding_id BIGINT PRIMARY KEY NOT NULL,
            tenant_id BIGINT NOT NULL,
            organization_id BIGINT NOT NULL DEFAULT 0,
            user_id BIGINT NOT NULL,
            binding_type TEXT NOT NULL,
            issuer TEXT,
            subject TEXT,
            platform TEXT,
            owner TEXT,
            external_ref TEXT,
            status TEXT NOT NULL DEFAULT 'active',
            created_at_ms BIGINT NOT NULL DEFAULT 0,
            updated_at_ms BIGINT NOT NULL DEFAULT 0
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_identity_binding_lookup
         ON ai_identity_binding (tenant_id, organization_id, binding_type, issuer, subject, status)",
    )
    .execute(&pool)
    .await?;
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
            coupon_template_id BIGINT PRIMARY KEY,
            tenant_id BIGINT NOT NULL,
            organization_id BIGINT NOT NULL DEFAULT 0,
            template_code TEXT NOT NULL,
            status TEXT NOT NULL,
            benefit_kind TEXT NOT NULL,
            distribution_kind TEXT NOT NULL,
            exclusive_group TEXT,
            created_at_ms BIGINT NOT NULL DEFAULT 0,
            updated_at_ms BIGINT NOT NULL DEFAULT 0,
            record_json JSONB NOT NULL
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_ai_marketing_coupon_template_code
         ON ai_marketing_coupon_template (tenant_id, organization_id, template_code)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_marketing_coupon_template_scope_status
         ON ai_marketing_coupon_template (tenant_id, organization_id, status, updated_at_ms DESC, coupon_template_id)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_marketing_coupon_benefit_rule (
            coupon_benefit_rule_id BIGINT PRIMARY KEY,
            tenant_id BIGINT NOT NULL,
            organization_id BIGINT NOT NULL DEFAULT 0,
            coupon_template_id BIGINT NOT NULL,
            benefit_kind TEXT NOT NULL,
            created_at_ms BIGINT NOT NULL DEFAULT 0,
            updated_at_ms BIGINT NOT NULL DEFAULT 0,
            record_json JSONB NOT NULL
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_marketing_coupon_benefit_rule_template
         ON ai_marketing_coupon_benefit_rule (coupon_template_id, updated_at_ms DESC, coupon_benefit_rule_id)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_marketing_campaign (
            marketing_campaign_id BIGINT PRIMARY KEY,
            tenant_id BIGINT NOT NULL,
            organization_id BIGINT NOT NULL DEFAULT 0,
            campaign_code TEXT NOT NULL,
            status TEXT NOT NULL,
            campaign_kind TEXT NOT NULL,
            owner_user_id BIGINT,
            created_at_ms BIGINT NOT NULL DEFAULT 0,
            updated_at_ms BIGINT NOT NULL DEFAULT 0,
            record_json JSONB NOT NULL
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_ai_marketing_campaign_code
         ON ai_marketing_campaign (tenant_id, organization_id, campaign_code)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_marketing_campaign_scope_status
         ON ai_marketing_campaign (tenant_id, organization_id, status, updated_at_ms DESC, marketing_campaign_id)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_marketing_coupon_code_batch (
            coupon_code_batch_id BIGINT PRIMARY KEY,
            tenant_id BIGINT NOT NULL,
            organization_id BIGINT NOT NULL DEFAULT 0,
            coupon_template_id BIGINT NOT NULL,
            marketing_campaign_id BIGINT,
            generation_mode TEXT NOT NULL,
            status TEXT NOT NULL,
            code_prefix TEXT,
            expires_at_ms BIGINT,
            created_at_ms BIGINT NOT NULL DEFAULT 0,
            updated_at_ms BIGINT NOT NULL DEFAULT 0,
            record_json JSONB NOT NULL
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_marketing_coupon_code_batch_template_status
         ON ai_marketing_coupon_code_batch (coupon_template_id, status, updated_at_ms DESC, coupon_code_batch_id)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_marketing_coupon_code (
            coupon_code_id BIGINT PRIMARY KEY,
            tenant_id BIGINT NOT NULL,
            organization_id BIGINT NOT NULL DEFAULT 0,
            coupon_code_batch_id BIGINT NOT NULL,
            coupon_template_id BIGINT NOT NULL,
            marketing_campaign_id BIGINT,
            code_lookup_hash TEXT NOT NULL,
            code_kind TEXT NOT NULL,
            status TEXT NOT NULL,
            display_code_prefix TEXT,
            display_code_suffix TEXT,
            claim_subject_type TEXT,
            claim_subject_id TEXT,
            expires_at_ms BIGINT,
            created_at_ms BIGINT NOT NULL DEFAULT 0,
            updated_at_ms BIGINT NOT NULL DEFAULT 0,
            record_json JSONB NOT NULL
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_ai_marketing_coupon_code_lookup_hash
         ON ai_marketing_coupon_code (code_lookup_hash)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_marketing_coupon_code_status
         ON ai_marketing_coupon_code (status, expires_at_ms, updated_at_ms DESC, coupon_code_id)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_marketing_coupon_code_subject
         ON ai_marketing_coupon_code (claim_subject_type, claim_subject_id, status, updated_at_ms DESC, coupon_code_id)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_marketing_coupon_claim (
            coupon_claim_id BIGINT PRIMARY KEY,
            tenant_id BIGINT NOT NULL,
            organization_id BIGINT NOT NULL DEFAULT 0,
            coupon_code_id BIGINT NOT NULL,
            coupon_template_id BIGINT NOT NULL,
            subject_type TEXT NOT NULL,
            subject_id TEXT NOT NULL,
            status TEXT NOT NULL,
            account_id BIGINT,
            project_id TEXT,
            expires_at_ms BIGINT,
            created_at_ms BIGINT NOT NULL DEFAULT 0,
            updated_at_ms BIGINT NOT NULL DEFAULT 0,
            record_json JSONB NOT NULL
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_marketing_coupon_claim_subject_status
         ON ai_marketing_coupon_claim (tenant_id, organization_id, subject_type, subject_id, status, updated_at_ms DESC, coupon_claim_id)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_marketing_coupon_redemption (
            coupon_redemption_id BIGINT PRIMARY KEY,
            tenant_id BIGINT NOT NULL,
            organization_id BIGINT NOT NULL DEFAULT 0,
            coupon_code_id BIGINT NOT NULL,
            coupon_template_id BIGINT NOT NULL,
            marketing_campaign_id BIGINT,
            subject_type TEXT NOT NULL,
            subject_id TEXT NOT NULL,
            status TEXT NOT NULL,
            idempotency_key TEXT,
            created_at_ms BIGINT NOT NULL DEFAULT 0,
            updated_at_ms BIGINT NOT NULL DEFAULT 0,
            record_json JSONB NOT NULL
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_marketing_coupon_claim ADD COLUMN IF NOT EXISTS subject_type TEXT NOT NULL DEFAULT ''",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_marketing_coupon_claim ADD COLUMN IF NOT EXISTS subject_id TEXT NOT NULL DEFAULT ''",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_marketing_coupon_redemption ADD COLUMN IF NOT EXISTS subject_type TEXT NOT NULL DEFAULT ''",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_marketing_coupon_redemption ADD COLUMN IF NOT EXISTS subject_id TEXT NOT NULL DEFAULT ''",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_marketing_coupon_redemption_lineage
         ON ai_marketing_coupon_redemption (coupon_code_id, coupon_template_id, marketing_campaign_id, created_at_ms DESC, coupon_redemption_id)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_marketing_coupon_redemption_idempotency
         ON ai_marketing_coupon_redemption (idempotency_key)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_marketing_referral_program (
            referral_program_id BIGINT PRIMARY KEY,
            tenant_id BIGINT NOT NULL,
            organization_id BIGINT NOT NULL DEFAULT 0,
            program_code TEXT NOT NULL,
            status TEXT NOT NULL,
            created_at_ms BIGINT NOT NULL DEFAULT 0,
            updated_at_ms BIGINT NOT NULL DEFAULT 0,
            record_json JSONB NOT NULL
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_ai_marketing_referral_program_code
         ON ai_marketing_referral_program (tenant_id, organization_id, program_code)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_marketing_referral_invite (
            referral_invite_id BIGINT PRIMARY KEY,
            tenant_id BIGINT NOT NULL,
            organization_id BIGINT NOT NULL DEFAULT 0,
            referral_program_id BIGINT NOT NULL,
            referrer_user_id BIGINT NOT NULL,
            status TEXT NOT NULL,
            coupon_code_id BIGINT,
            source_code TEXT,
            referred_user_id BIGINT,
            created_at_ms BIGINT NOT NULL DEFAULT 0,
            updated_at_ms BIGINT NOT NULL DEFAULT 0,
            record_json JSONB NOT NULL
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_marketing_referral_invite_program_referrer
         ON ai_marketing_referral_invite (referral_program_id, referrer_user_id, updated_at_ms DESC, referral_invite_id)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_marketing_attribution_touch (
            attribution_touch_id BIGINT PRIMARY KEY,
            tenant_id BIGINT NOT NULL,
            organization_id BIGINT NOT NULL DEFAULT 0,
            source_kind TEXT NOT NULL,
            source_code TEXT,
            partner_id TEXT,
            referrer_user_id BIGINT,
            invite_code_id BIGINT,
            conversion_subject_id TEXT,
            created_at_ms BIGINT NOT NULL DEFAULT 0,
            updated_at_ms BIGINT NOT NULL DEFAULT 0,
            record_json JSONB NOT NULL
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_marketing_attribution_touch_source
         ON ai_marketing_attribution_touch (source_kind, source_code, updated_at_ms DESC, attribution_touch_id)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_routing_policies (
            policy_id TEXT PRIMARY KEY NOT NULL,
            capability TEXT NOT NULL,
            model_pattern TEXT NOT NULL,
            enabled BOOLEAN NOT NULL DEFAULT TRUE,
            priority INTEGER NOT NULL DEFAULT 0,
            strategy TEXT NOT NULL DEFAULT 'deterministic_priority',
            default_provider_id TEXT
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_routing_policies ADD COLUMN IF NOT EXISTS strategy TEXT NOT NULL DEFAULT 'deterministic_priority'",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_routing_policies ADD COLUMN IF NOT EXISTS max_cost DOUBLE PRECISION",
    )
    .execute(&pool)
    .await?;
    sqlx::query("ALTER TABLE ai_routing_policies ADD COLUMN IF NOT EXISTS max_latency_ms BIGINT")
        .execute(&pool)
        .await?;
    sqlx::query(
        "ALTER TABLE ai_routing_policies ADD COLUMN IF NOT EXISTS require_healthy BOOLEAN NOT NULL DEFAULT FALSE",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_routing_policy_providers (
            policy_id TEXT NOT NULL,
            provider_id TEXT NOT NULL,
            position INTEGER NOT NULL,
            PRIMARY KEY (policy_id, provider_id)
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_routing_policy_providers_policy_position
         ON ai_routing_policy_providers (policy_id, position, provider_id)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_routing_policy_providers_provider_position
         ON ai_routing_policy_providers (provider_id, position, policy_id)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_routing_policies_capability_priority
         ON ai_routing_policies (capability, enabled, priority DESC, policy_id)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_routing_profiles (
            profile_id TEXT PRIMARY KEY NOT NULL,
            tenant_id TEXT NOT NULL,
            project_id TEXT NOT NULL,
            name TEXT NOT NULL,
            slug TEXT NOT NULL,
            description TEXT,
            active BOOLEAN NOT NULL DEFAULT TRUE,
            strategy TEXT NOT NULL DEFAULT 'deterministic_priority',
            ordered_provider_ids_json TEXT NOT NULL DEFAULT '[]',
            default_provider_id TEXT,
            max_cost DOUBLE PRECISION,
            max_latency_ms BIGINT,
            require_healthy BOOLEAN NOT NULL DEFAULT FALSE,
            preferred_region TEXT,
            created_at_ms BIGINT NOT NULL DEFAULT 0,
            updated_at_ms BIGINT NOT NULL DEFAULT 0
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_ai_routing_profiles_workspace_slug
         ON ai_routing_profiles (tenant_id, project_id, slug)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_routing_profiles_workspace_active
         ON ai_routing_profiles (tenant_id, project_id, active, updated_at_ms DESC, profile_id)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_compiled_routing_snapshots (
            snapshot_id TEXT PRIMARY KEY NOT NULL,
            tenant_id TEXT,
            project_id TEXT,
            api_key_group_id TEXT,
            capability TEXT NOT NULL,
            route_key TEXT NOT NULL,
            matched_policy_id TEXT,
            project_routing_preferences_project_id TEXT,
            applied_routing_profile_id TEXT,
            strategy TEXT NOT NULL DEFAULT '',
            ordered_provider_ids_json TEXT NOT NULL DEFAULT '[]',
            default_provider_id TEXT,
            max_cost DOUBLE PRECISION,
            max_latency_ms BIGINT,
            require_healthy BOOLEAN NOT NULL DEFAULT FALSE,
            preferred_region TEXT,
            created_at_ms BIGINT NOT NULL DEFAULT 0,
            updated_at_ms BIGINT NOT NULL DEFAULT 0
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_compiled_routing_snapshots_scope_updated_at
         ON ai_compiled_routing_snapshots (tenant_id, project_id, api_key_group_id, updated_at_ms DESC, snapshot_id)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_project_routing_preferences (
            project_id TEXT PRIMARY KEY NOT NULL,
            preset_id TEXT NOT NULL DEFAULT '',
            strategy TEXT NOT NULL DEFAULT 'deterministic_priority',
            ordered_provider_ids_json TEXT NOT NULL DEFAULT '[]',
            default_provider_id TEXT,
            max_cost DOUBLE PRECISION,
            max_latency_ms BIGINT,
            require_healthy BOOLEAN NOT NULL DEFAULT FALSE,
            preferred_region TEXT,
            updated_at_ms BIGINT NOT NULL DEFAULT 0
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_routing_decision_logs (
            decision_id TEXT PRIMARY KEY NOT NULL,
            decision_source TEXT NOT NULL,
            tenant_id TEXT,
            project_id TEXT,
            api_key_group_id TEXT,
            capability TEXT NOT NULL,
            route_key TEXT NOT NULL,
            selected_provider_id TEXT NOT NULL,
            matched_policy_id TEXT,
            applied_routing_profile_id TEXT,
            compiled_routing_snapshot_id TEXT,
            strategy TEXT NOT NULL,
            selection_seed BIGINT,
            selection_reason TEXT,
            fallback_reason TEXT,
            requested_region TEXT,
            slo_applied BOOLEAN NOT NULL DEFAULT FALSE,
            slo_degraded BOOLEAN NOT NULL DEFAULT FALSE,
            created_at_ms BIGINT NOT NULL,
            assessments_json TEXT NOT NULL DEFAULT '[]'
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_routing_decision_logs_project_created_at
         ON ai_routing_decision_logs (project_id, created_at_ms DESC, decision_id)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_routing_decision_logs_provider_created_at
         ON ai_routing_decision_logs (selected_provider_id, created_at_ms DESC, decision_id)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_routing_decision_logs_capability_created_at
         ON ai_routing_decision_logs (capability, created_at_ms DESC, decision_id)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_routing_decision_logs ADD COLUMN IF NOT EXISTS requested_region TEXT",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_routing_decision_logs ADD COLUMN IF NOT EXISTS api_key_group_id TEXT",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_routing_decision_logs ADD COLUMN IF NOT EXISTS applied_routing_profile_id TEXT",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_routing_decision_logs ADD COLUMN IF NOT EXISTS compiled_routing_snapshot_id TEXT",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_routing_decision_logs ADD COLUMN IF NOT EXISTS fallback_reason TEXT",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_provider_health_records (
            provider_id TEXT NOT NULL,
            extension_id TEXT NOT NULL,
            runtime TEXT NOT NULL,
            observed_at_ms BIGINT NOT NULL,
            instance_id TEXT,
            running BOOLEAN NOT NULL DEFAULT FALSE,
            healthy BOOLEAN NOT NULL DEFAULT FALSE,
            message TEXT
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_provider_health_records_provider_observed_at
         ON ai_provider_health_records (provider_id, observed_at_ms DESC, runtime)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_provider_health_records_extension_runtime_observed_at
         ON ai_provider_health_records (extension_id, runtime, observed_at_ms DESC, provider_id)",
    )
    .execute(&pool)
    .await?;
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
        "CREATE TABLE IF NOT EXISTS ai_pricing_rate (
            pricing_rate_id BIGINT PRIMARY KEY NOT NULL,
            tenant_id BIGINT NOT NULL,
            organization_id BIGINT NOT NULL DEFAULT 0,
            pricing_plan_id BIGINT NOT NULL,
            metric_code TEXT NOT NULL,
            model_code TEXT,
            provider_code TEXT,
            quantity_step DOUBLE PRECISION NOT NULL DEFAULT 1,
            unit_price DOUBLE PRECISION NOT NULL DEFAULT 0,
            created_at_ms BIGINT NOT NULL DEFAULT 0
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
            status TEXT NOT NULL DEFAULT 'fulfilled',
            source TEXT NOT NULL DEFAULT 'workspace_seed',
            created_at_ms BIGINT NOT NULL DEFAULT 0
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_commerce_orders_project_created_at
         ON ai_commerce_orders (project_id, created_at_ms DESC, status, order_id)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_commerce_orders_user_created_at
         ON ai_commerce_orders (user_id, created_at_ms DESC, status, order_id)",
    )
    .execute(&pool)
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
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_project_memberships_project_updated_at
         ON ai_project_memberships (project_id, updated_at_ms DESC, status)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_project_memberships_user_updated_at
         ON ai_project_memberships (user_id, updated_at_ms DESC, status)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_channel (
            channel_id TEXT PRIMARY KEY NOT NULL,
            channel_name TEXT NOT NULL,
            channel_description TEXT NOT NULL DEFAULT '',
            sort_order INTEGER NOT NULL DEFAULT 0,
            is_builtin BOOLEAN NOT NULL DEFAULT FALSE,
            is_active BOOLEAN NOT NULL DEFAULT TRUE,
            created_at_ms BIGINT NOT NULL DEFAULT 0,
            updated_at_ms BIGINT NOT NULL DEFAULT 0
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_proxy_provider (
            proxy_provider_id TEXT PRIMARY KEY NOT NULL,
            primary_channel_id TEXT NOT NULL,
            extension_id TEXT NOT NULL DEFAULT '',
            adapter_kind TEXT NOT NULL DEFAULT 'openai',
            base_url TEXT NOT NULL DEFAULT 'http://localhost',
            display_name TEXT NOT NULL,
            is_active BOOLEAN NOT NULL DEFAULT TRUE,
            created_at_ms BIGINT NOT NULL DEFAULT 0,
            updated_at_ms BIGINT NOT NULL DEFAULT 0
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_proxy_provider_primary_channel
         ON ai_proxy_provider (primary_channel_id, is_active, proxy_provider_id)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_proxy_provider_channel (
            proxy_provider_id TEXT NOT NULL,
            channel_id TEXT NOT NULL,
            is_primary BOOLEAN NOT NULL DEFAULT FALSE,
            created_at_ms BIGINT NOT NULL DEFAULT 0,
            updated_at_ms BIGINT NOT NULL DEFAULT 0,
            PRIMARY KEY (proxy_provider_id, channel_id)
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_proxy_provider_channel_channel_provider
         ON ai_proxy_provider_channel (channel_id, proxy_provider_id, is_primary)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_router_credential_records (
            tenant_id TEXT NOT NULL,
            proxy_provider_id TEXT NOT NULL,
            key_reference TEXT NOT NULL,
            secret_backend TEXT NOT NULL DEFAULT 'database_encrypted',
            secret_local_file TEXT,
            secret_keyring_service TEXT,
            secret_master_key_id TEXT,
            secret_ciphertext TEXT,
            secret_key_version INTEGER,
            created_at_ms BIGINT NOT NULL DEFAULT 0,
            updated_at_ms BIGINT NOT NULL DEFAULT 0,
            PRIMARY KEY (tenant_id, proxy_provider_id, key_reference)
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_router_credential_records_tenant_updated
         ON ai_router_credential_records (tenant_id, updated_at_ms DESC, proxy_provider_id, key_reference)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_router_credential_records_provider_updated
         ON ai_router_credential_records (proxy_provider_id, updated_at_ms DESC, tenant_id, key_reference)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_model (
            channel_id TEXT NOT NULL,
            model_id TEXT NOT NULL,
            model_display_name TEXT NOT NULL,
            capabilities_json TEXT NOT NULL DEFAULT '[]',
            streaming_enabled BOOLEAN NOT NULL DEFAULT FALSE,
            context_window BIGINT,
            description TEXT NOT NULL DEFAULT '',
            created_at_ms BIGINT NOT NULL DEFAULT 0,
            updated_at_ms BIGINT NOT NULL DEFAULT 0,
            PRIMARY KEY (channel_id, model_id)
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_model_model_streaming
         ON ai_model (model_id, streaming_enabled, channel_id)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_model_price (
            channel_id TEXT NOT NULL,
            model_id TEXT NOT NULL,
            proxy_provider_id TEXT NOT NULL,
            currency_code TEXT NOT NULL DEFAULT 'USD',
            price_unit TEXT NOT NULL DEFAULT 'per_1m_tokens',
            input_price DOUBLE PRECISION NOT NULL DEFAULT 0,
            output_price DOUBLE PRECISION NOT NULL DEFAULT 0,
            cache_read_price DOUBLE PRECISION NOT NULL DEFAULT 0,
            cache_write_price DOUBLE PRECISION NOT NULL DEFAULT 0,
            request_price DOUBLE PRECISION NOT NULL DEFAULT 0,
            is_active BOOLEAN NOT NULL DEFAULT TRUE,
            created_at_ms BIGINT NOT NULL DEFAULT 0,
            updated_at_ms BIGINT NOT NULL DEFAULT 0,
            PRIMARY KEY (channel_id, model_id, proxy_provider_id)
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_model_price_provider_active
         ON ai_model_price (proxy_provider_id, is_active, channel_id, model_id)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_model_price_channel_active
         ON ai_model_price (channel_id, model_id, is_active, proxy_provider_id)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_model_price_model_active
         ON ai_model_price (model_id, is_active, channel_id, proxy_provider_id)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_app_api_key_groups (
            group_id TEXT PRIMARY KEY NOT NULL,
            tenant_id TEXT NOT NULL,
            project_id TEXT NOT NULL,
            environment TEXT NOT NULL,
            name TEXT NOT NULL,
            slug TEXT NOT NULL,
            description TEXT,
            color TEXT,
            default_capability_scope TEXT,
            default_routing_profile_id TEXT,
            default_accounting_mode TEXT,
            active BOOLEAN NOT NULL DEFAULT TRUE,
            created_at_ms BIGINT NOT NULL DEFAULT 0,
            updated_at_ms BIGINT NOT NULL DEFAULT 0
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_ai_app_api_key_groups_workspace_slug
         ON ai_app_api_key_groups (tenant_id, project_id, environment, slug)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_app_api_key_groups_workspace_active
         ON ai_app_api_key_groups (tenant_id, project_id, environment, active, created_at_ms DESC)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_app_api_keys (
            hashed_key TEXT PRIMARY KEY NOT NULL,
            raw_key TEXT,
            tenant_id TEXT NOT NULL,
            project_id TEXT NOT NULL,
            environment TEXT NOT NULL,
            api_key_group_id TEXT,
            key_prefix TEXT NOT NULL DEFAULT '',
            label TEXT NOT NULL DEFAULT '',
            notes TEXT,
            created_at_ms BIGINT NOT NULL DEFAULT 0,
            last_used_at_ms BIGINT,
            expires_at_ms BIGINT,
            active BOOLEAN NOT NULL
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_channel ADD COLUMN IF NOT EXISTS channel_description TEXT NOT NULL DEFAULT ''",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_channel ADD COLUMN IF NOT EXISTS sort_order INTEGER NOT NULL DEFAULT 0",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_channel ADD COLUMN IF NOT EXISTS is_builtin BOOLEAN NOT NULL DEFAULT FALSE",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_channel ADD COLUMN IF NOT EXISTS is_active BOOLEAN NOT NULL DEFAULT TRUE",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_channel ADD COLUMN IF NOT EXISTS created_at_ms BIGINT NOT NULL DEFAULT 0",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_channel ADD COLUMN IF NOT EXISTS updated_at_ms BIGINT NOT NULL DEFAULT 0",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_proxy_provider ADD COLUMN IF NOT EXISTS extension_id TEXT NOT NULL DEFAULT ''",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_proxy_provider ADD COLUMN IF NOT EXISTS adapter_kind TEXT NOT NULL DEFAULT 'openai'",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_proxy_provider ADD COLUMN IF NOT EXISTS base_url TEXT NOT NULL DEFAULT 'http://localhost'",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_proxy_provider ADD COLUMN IF NOT EXISTS is_active BOOLEAN NOT NULL DEFAULT TRUE",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_proxy_provider ADD COLUMN IF NOT EXISTS created_at_ms BIGINT NOT NULL DEFAULT 0",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_proxy_provider ADD COLUMN IF NOT EXISTS updated_at_ms BIGINT NOT NULL DEFAULT 0",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_proxy_provider_channel ADD COLUMN IF NOT EXISTS created_at_ms BIGINT NOT NULL DEFAULT 0",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_proxy_provider_channel ADD COLUMN IF NOT EXISTS updated_at_ms BIGINT NOT NULL DEFAULT 0",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_router_credential_records ADD COLUMN IF NOT EXISTS secret_backend TEXT NOT NULL DEFAULT 'database_encrypted'",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_router_credential_records ADD COLUMN IF NOT EXISTS secret_local_file TEXT",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_router_credential_records ADD COLUMN IF NOT EXISTS secret_keyring_service TEXT",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_router_credential_records ADD COLUMN IF NOT EXISTS secret_master_key_id TEXT",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_router_credential_records ADD COLUMN IF NOT EXISTS secret_ciphertext TEXT",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_router_credential_records ADD COLUMN IF NOT EXISTS secret_key_version INTEGER",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_router_credential_records ADD COLUMN IF NOT EXISTS created_at_ms BIGINT NOT NULL DEFAULT 0",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_router_credential_records ADD COLUMN IF NOT EXISTS updated_at_ms BIGINT NOT NULL DEFAULT 0",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_model ADD COLUMN IF NOT EXISTS capabilities_json TEXT NOT NULL DEFAULT '[]'",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_model ADD COLUMN IF NOT EXISTS streaming_enabled BOOLEAN NOT NULL DEFAULT FALSE",
    )
    .execute(&pool)
    .await?;
    sqlx::query("ALTER TABLE ai_model ADD COLUMN IF NOT EXISTS context_window BIGINT")
        .execute(&pool)
        .await?;
    sqlx::query(
        "ALTER TABLE ai_model ADD COLUMN IF NOT EXISTS description TEXT NOT NULL DEFAULT ''",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_model ADD COLUMN IF NOT EXISTS created_at_ms BIGINT NOT NULL DEFAULT 0",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_model ADD COLUMN IF NOT EXISTS updated_at_ms BIGINT NOT NULL DEFAULT 0",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_model_price ADD COLUMN IF NOT EXISTS currency_code TEXT NOT NULL DEFAULT 'USD'",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_model_price ADD COLUMN IF NOT EXISTS price_unit TEXT NOT NULL DEFAULT 'per_1m_tokens'",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_model_price ADD COLUMN IF NOT EXISTS input_price DOUBLE PRECISION NOT NULL DEFAULT 0",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_model_price ADD COLUMN IF NOT EXISTS output_price DOUBLE PRECISION NOT NULL DEFAULT 0",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_model_price ADD COLUMN IF NOT EXISTS cache_read_price DOUBLE PRECISION NOT NULL DEFAULT 0",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_model_price ADD COLUMN IF NOT EXISTS cache_write_price DOUBLE PRECISION NOT NULL DEFAULT 0",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_model_price ADD COLUMN IF NOT EXISTS request_price DOUBLE PRECISION NOT NULL DEFAULT 0",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_model_price ADD COLUMN IF NOT EXISTS is_active BOOLEAN NOT NULL DEFAULT TRUE",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_model_price ADD COLUMN IF NOT EXISTS created_at_ms BIGINT NOT NULL DEFAULT 0",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_model_price ADD COLUMN IF NOT EXISTS updated_at_ms BIGINT NOT NULL DEFAULT 0",
    )
    .execute(&pool)
    .await?;
    sqlx::query("ALTER TABLE ai_app_api_keys ADD COLUMN IF NOT EXISTS raw_key TEXT")
        .execute(&pool)
        .await?;
    sqlx::query(
        "ALTER TABLE ai_app_api_keys ADD COLUMN IF NOT EXISTS key_prefix TEXT NOT NULL DEFAULT ''",
    )
    .execute(&pool)
    .await?;
    sqlx::query("UPDATE ai_app_api_keys SET raw_key = NULL WHERE raw_key IS NOT NULL")
        .execute(&pool)
        .await?;
    sqlx::query("ALTER TABLE ai_app_api_keys ADD COLUMN IF NOT EXISTS api_key_group_id TEXT")
        .execute(&pool)
        .await?;
    sqlx::query(
        "ALTER TABLE ai_app_api_keys ADD COLUMN IF NOT EXISTS label TEXT NOT NULL DEFAULT ''",
    )
    .execute(&pool)
    .await?;
    sqlx::query("ALTER TABLE ai_app_api_key_groups ADD COLUMN IF NOT EXISTS description TEXT")
        .execute(&pool)
        .await?;
    sqlx::query("ALTER TABLE ai_app_api_key_groups ADD COLUMN IF NOT EXISTS color TEXT")
        .execute(&pool)
        .await?;
    sqlx::query(
        "ALTER TABLE ai_app_api_key_groups ADD COLUMN IF NOT EXISTS default_capability_scope TEXT",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_app_api_key_groups ADD COLUMN IF NOT EXISTS default_routing_profile_id TEXT",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_app_api_key_groups ADD COLUMN IF NOT EXISTS default_accounting_mode TEXT",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_app_api_key_groups ADD COLUMN IF NOT EXISTS active BOOLEAN NOT NULL DEFAULT TRUE",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_app_api_key_groups ADD COLUMN IF NOT EXISTS created_at_ms BIGINT NOT NULL DEFAULT 0",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE ai_app_api_key_groups ADD COLUMN IF NOT EXISTS updated_at_ms BIGINT NOT NULL DEFAULT 0",
    )
    .execute(&pool)
    .await?;
    sqlx::query("ALTER TABLE ai_app_api_keys ADD COLUMN IF NOT EXISTS notes TEXT")
        .execute(&pool)
        .await?;
    sqlx::query(
        "ALTER TABLE ai_app_api_keys ADD COLUMN IF NOT EXISTS created_at_ms BIGINT NOT NULL DEFAULT 0",
    )
    .execute(&pool)
    .await?;
    sqlx::query("ALTER TABLE ai_app_api_keys ADD COLUMN IF NOT EXISTS last_used_at_ms BIGINT")
        .execute(&pool)
        .await?;
    sqlx::query("ALTER TABLE ai_app_api_keys ADD COLUMN IF NOT EXISTS expires_at_ms BIGINT")
        .execute(&pool)
        .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_app_api_keys_project_active
         ON ai_app_api_keys (project_id, active, created_at_ms DESC, hashed_key)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_app_api_keys_tenant_environment
         ON ai_app_api_keys (tenant_id, environment, active, created_at_ms DESC)",
    )
    .execute(&pool)
    .await?;
    ensure_postgres_column_if_table_exists(
        &pool,
        "catalog_proxy_providers",
        "ALTER TABLE catalog_proxy_providers ADD COLUMN IF NOT EXISTS extension_id TEXT NOT NULL DEFAULT ''",
    )
    .await?;
    ensure_postgres_column_if_table_exists(
        &pool,
        "catalog_proxy_providers",
        "ALTER TABLE catalog_proxy_providers ADD COLUMN IF NOT EXISTS adapter_kind TEXT NOT NULL DEFAULT 'openai'",
    )
    .await?;
    ensure_postgres_column_if_table_exists(
        &pool,
        "catalog_proxy_providers",
        "ALTER TABLE catalog_proxy_providers ADD COLUMN IF NOT EXISTS base_url TEXT NOT NULL DEFAULT 'http://localhost'",
    )
    .await?;
    ensure_postgres_column_if_table_exists(
        &pool,
        "credential_records",
        "ALTER TABLE credential_records ADD COLUMN IF NOT EXISTS secret_backend TEXT NOT NULL DEFAULT 'database_encrypted'",
    )
    .await?;
    ensure_postgres_column_if_table_exists(
        &pool,
        "credential_records",
        "ALTER TABLE credential_records ADD COLUMN IF NOT EXISTS secret_local_file TEXT",
    )
    .await?;
    ensure_postgres_column_if_table_exists(
        &pool,
        "credential_records",
        "ALTER TABLE credential_records ADD COLUMN IF NOT EXISTS secret_keyring_service TEXT",
    )
    .await?;
    ensure_postgres_column_if_table_exists(
        &pool,
        "credential_records",
        "ALTER TABLE credential_records ADD COLUMN IF NOT EXISTS secret_master_key_id TEXT",
    )
    .await?;
    ensure_postgres_column_if_table_exists(
        &pool,
        "credential_records",
        "ALTER TABLE credential_records ADD COLUMN IF NOT EXISTS secret_ciphertext TEXT",
    )
    .await?;
    ensure_postgres_column_if_table_exists(
        &pool,
        "credential_records",
        "ALTER TABLE credential_records ADD COLUMN IF NOT EXISTS secret_key_version INTEGER",
    )
    .await?;
    ensure_postgres_column_if_table_exists(
        &pool,
        "catalog_models",
        "ALTER TABLE catalog_models ADD COLUMN IF NOT EXISTS capabilities TEXT NOT NULL DEFAULT '[]'",
    )
    .await?;
    ensure_postgres_column_if_table_exists(
        &pool,
        "catalog_models",
        "ALTER TABLE catalog_models ADD COLUMN IF NOT EXISTS streaming BOOLEAN NOT NULL DEFAULT FALSE",
    )
    .await?;
    ensure_postgres_column_if_table_exists(
        &pool,
        "catalog_models",
        "ALTER TABLE catalog_models ADD COLUMN IF NOT EXISTS context_window BIGINT",
    )
    .await?;
    ensure_postgres_column_if_table_exists(
        &pool,
        "identity_gateway_api_keys",
        "ALTER TABLE identity_gateway_api_keys ADD COLUMN IF NOT EXISTS label TEXT NOT NULL DEFAULT ''",
    )
    .await?;
    ensure_postgres_column_if_table_exists(
        &pool,
        "identity_gateway_api_keys",
        "ALTER TABLE identity_gateway_api_keys ADD COLUMN IF NOT EXISTS notes TEXT",
    )
    .await?;
    ensure_postgres_column_if_table_exists(
        &pool,
        "identity_gateway_api_keys",
        "ALTER TABLE identity_gateway_api_keys ADD COLUMN IF NOT EXISTS created_at_ms BIGINT NOT NULL DEFAULT 0",
    )
    .await?;
    ensure_postgres_column_if_table_exists(
        &pool,
        "identity_gateway_api_keys",
        "ALTER TABLE identity_gateway_api_keys ADD COLUMN IF NOT EXISTS last_used_at_ms BIGINT",
    )
    .await?;
    ensure_postgres_column_if_table_exists(
        &pool,
        "identity_gateway_api_keys",
        "ALTER TABLE identity_gateway_api_keys ADD COLUMN IF NOT EXISTS expires_at_ms BIGINT",
    )
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_extension_installations (
            installation_id TEXT PRIMARY KEY NOT NULL,
            extension_id TEXT NOT NULL,
            runtime TEXT NOT NULL,
            enabled BOOLEAN NOT NULL,
            entrypoint TEXT,
            config_json TEXT NOT NULL DEFAULT '{}'
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_extension_instances (
            instance_id TEXT PRIMARY KEY NOT NULL,
            installation_id TEXT NOT NULL,
            extension_id TEXT NOT NULL,
            enabled BOOLEAN NOT NULL,
            base_url TEXT,
            credential_ref TEXT,
            config_json TEXT NOT NULL DEFAULT '{}'
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_service_runtime_nodes (
            node_id TEXT PRIMARY KEY NOT NULL,
            service_kind TEXT NOT NULL,
            started_at_ms BIGINT NOT NULL,
            last_seen_at_ms BIGINT NOT NULL
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_service_runtime_nodes_last_seen
         ON ai_service_runtime_nodes (last_seen_at_ms DESC, node_id)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_extension_runtime_rollouts (
            rollout_id TEXT PRIMARY KEY NOT NULL,
            scope TEXT NOT NULL,
            requested_extension_id TEXT,
            requested_instance_id TEXT,
            resolved_extension_id TEXT,
            created_by TEXT NOT NULL,
            created_at_ms BIGINT NOT NULL,
            deadline_at_ms BIGINT NOT NULL
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_extension_runtime_rollouts_created_at
         ON ai_extension_runtime_rollouts (created_at_ms DESC, rollout_id)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_extension_runtime_rollout_participants (
            rollout_id TEXT NOT NULL,
            node_id TEXT NOT NULL,
            service_kind TEXT NOT NULL,
            status TEXT NOT NULL,
            message TEXT,
            updated_at_ms BIGINT NOT NULL,
            PRIMARY KEY (rollout_id, node_id)
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_extension_runtime_rollout_participants_node_status
         ON ai_extension_runtime_rollout_participants (node_id, status, updated_at_ms, rollout_id)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_extension_runtime_rollout_participants_rollout
         ON ai_extension_runtime_rollout_participants (rollout_id, node_id)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_standalone_config_rollouts (
            rollout_id TEXT PRIMARY KEY NOT NULL,
            requested_service_kind TEXT,
            created_by TEXT NOT NULL,
            created_at_ms BIGINT NOT NULL,
            deadline_at_ms BIGINT NOT NULL
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_standalone_config_rollouts_created_at
         ON ai_standalone_config_rollouts (created_at_ms DESC, rollout_id)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_standalone_config_rollout_participants (
            rollout_id TEXT NOT NULL,
            node_id TEXT NOT NULL,
            service_kind TEXT NOT NULL,
            status TEXT NOT NULL,
            message TEXT,
            updated_at_ms BIGINT NOT NULL,
            PRIMARY KEY (rollout_id, node_id)
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_standalone_config_rollout_participants_node_status
         ON ai_standalone_config_rollout_participants (node_id, status, updated_at_ms, rollout_id)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_standalone_config_rollout_participants_rollout
         ON ai_standalone_config_rollout_participants (rollout_id, node_id)",
    )
    .execute(&pool)
    .await?;
    for (legacy_table_name, canonical_table_name) in LEGACY_RENAMED_TABLE_MAPPINGS {
        migrate_postgres_legacy_table_with_common_columns(
            &pool,
            legacy_table_name,
            canonical_table_name,
        )
        .await?;
    }

    if postgres_relation_kind(&pool, "catalog_channels")
        .await?
        .as_deref()
        == Some("r")
    {
        sqlx::query(
            "INSERT INTO ai_channel (
                channel_id,
                channel_name,
                channel_description,
                sort_order,
                is_builtin,
                is_active,
                created_at_ms,
                updated_at_ms
            )
            SELECT id, name, '', 0, FALSE, TRUE, 0, 0
            FROM catalog_channels
            ON CONFLICT (channel_id) DO NOTHING",
        )
        .execute(&pool)
        .await?;
        sqlx::query("DROP TABLE catalog_channels")
            .execute(&pool)
            .await?;
    }

    for (channel_id, channel_name, sort_order) in BUILTIN_CHANNEL_SEEDS {
        sqlx::query(
            "INSERT INTO ai_channel (
                channel_id,
                channel_name,
                channel_description,
                sort_order,
                is_builtin,
                is_active,
                created_at_ms,
                updated_at_ms
            ) VALUES ($1, $2, '', $3, TRUE, TRUE, 0, 0)
            ON CONFLICT (channel_id) DO UPDATE SET
                channel_name = EXCLUDED.channel_name,
                sort_order = EXCLUDED.sort_order,
                is_builtin = TRUE,
                is_active = TRUE",
        )
        .bind(channel_id)
        .bind(channel_name)
        .bind(sort_order)
        .execute(&pool)
        .await?;
    }

    if postgres_relation_kind(&pool, "catalog_proxy_providers")
        .await?
        .as_deref()
        == Some("r")
    {
        sqlx::query(
            "INSERT INTO ai_proxy_provider (
                proxy_provider_id,
                primary_channel_id,
                extension_id,
                adapter_kind,
                base_url,
                display_name,
                is_active,
                created_at_ms,
                updated_at_ms
            )
            SELECT id, channel_id, extension_id, adapter_kind, base_url, display_name, TRUE, 0, 0
            FROM catalog_proxy_providers
            ON CONFLICT (proxy_provider_id) DO NOTHING",
        )
        .execute(&pool)
        .await?;
        sqlx::query(
            "INSERT INTO ai_proxy_provider_channel (
                proxy_provider_id,
                channel_id,
                is_primary,
                created_at_ms,
                updated_at_ms
            )
            SELECT id, channel_id, TRUE, 0, 0
            FROM catalog_proxy_providers
            ON CONFLICT (proxy_provider_id, channel_id) DO UPDATE SET
                is_primary = EXCLUDED.is_primary,
                updated_at_ms = EXCLUDED.updated_at_ms",
        )
        .execute(&pool)
        .await?;
    }

    if postgres_relation_kind(&pool, "catalog_provider_channel_bindings")
        .await?
        .as_deref()
        == Some("r")
    {
        sqlx::query(
            "INSERT INTO ai_proxy_provider_channel (
                proxy_provider_id,
                channel_id,
                is_primary,
                created_at_ms,
                updated_at_ms
            )
            SELECT provider_id, channel_id, is_primary, 0, 0
            FROM catalog_provider_channel_bindings
            ON CONFLICT (proxy_provider_id, channel_id) DO UPDATE SET
                is_primary = EXCLUDED.is_primary,
                updated_at_ms = EXCLUDED.updated_at_ms",
        )
        .execute(&pool)
        .await?;
        sqlx::query("DROP TABLE catalog_provider_channel_bindings")
            .execute(&pool)
            .await?;
    }

    if postgres_relation_kind(&pool, "catalog_proxy_providers")
        .await?
        .as_deref()
        == Some("r")
    {
        sqlx::query("DROP TABLE catalog_proxy_providers")
            .execute(&pool)
            .await?;
    }

    if postgres_relation_kind(&pool, "credential_records")
        .await?
        .as_deref()
        == Some("r")
    {
        sqlx::query(
            "INSERT INTO ai_router_credential_records (
                tenant_id,
                proxy_provider_id,
                key_reference,
                secret_backend,
                secret_local_file,
                secret_keyring_service,
                secret_master_key_id,
                secret_ciphertext,
                secret_key_version,
                created_at_ms,
                updated_at_ms
            )
            SELECT
                tenant_id,
                provider_id,
                key_reference,
                secret_backend,
                secret_local_file,
                secret_keyring_service,
                secret_master_key_id,
                secret_ciphertext,
                secret_key_version,
                0,
                0
            FROM credential_records
            ON CONFLICT (tenant_id, proxy_provider_id, key_reference) DO NOTHING",
        )
        .execute(&pool)
        .await?;
        sqlx::query("DROP TABLE credential_records")
            .execute(&pool)
            .await?;
    }

    if postgres_relation_kind(&pool, "catalog_models")
        .await?
        .as_deref()
        == Some("r")
    {
        sqlx::query(
            "INSERT INTO ai_model (
                channel_id,
                model_id,
                model_display_name,
                capabilities_json,
                streaming_enabled,
                context_window,
                description,
                created_at_ms,
                updated_at_ms
            )
            SELECT
                providers.primary_channel_id,
                models.external_name,
                models.external_name,
                models.capabilities,
                models.streaming,
                models.context_window,
                '',
                0,
                0
            FROM catalog_models models
            INNER JOIN ai_proxy_provider providers
                ON providers.proxy_provider_id = models.provider_id
            ON CONFLICT (channel_id, model_id) DO NOTHING",
        )
        .execute(&pool)
        .await?;
        sqlx::query(
            "INSERT INTO ai_model_price (
                channel_id,
                model_id,
                proxy_provider_id,
                currency_code,
                price_unit,
                input_price,
                output_price,
                cache_read_price,
                cache_write_price,
                request_price,
                is_active,
                created_at_ms,
                updated_at_ms
            )
            SELECT
                providers.primary_channel_id,
                models.external_name,
                models.provider_id,
                'USD',
                'per_1m_tokens',
                0,
                0,
                0,
                0,
                0,
                TRUE,
                0,
                0
            FROM catalog_models models
            INNER JOIN ai_proxy_provider providers
                ON providers.proxy_provider_id = models.provider_id
            ON CONFLICT (channel_id, model_id, proxy_provider_id) DO NOTHING",
        )
        .execute(&pool)
        .await?;
        sqlx::query("DROP TABLE catalog_models")
            .execute(&pool)
            .await?;
    }

    if postgres_relation_kind(&pool, "identity_gateway_api_keys")
        .await?
        .as_deref()
        == Some("r")
    {
        sqlx::query(
            "INSERT INTO ai_app_api_keys (
                hashed_key,
                tenant_id,
                project_id,
                environment,
                api_key_group_id,
                key_prefix,
                label,
                notes,
                created_at_ms,
                last_used_at_ms,
                expires_at_ms,
                active
            )
            SELECT
                hashed_key,
                tenant_id,
                project_id,
                environment,
                NULL,
                '',
                label,
                notes,
                created_at_ms,
                last_used_at_ms,
                expires_at_ms,
                active
            FROM identity_gateway_api_keys
            ON CONFLICT (hashed_key) DO NOTHING",
        )
        .execute(&pool)
        .await?;
        sqlx::query("DROP TABLE identity_gateway_api_keys")
            .execute(&pool)
            .await?;
    }

    for (legacy_table_name, canonical_table_name) in LEGACY_RENAMED_TABLE_MAPPINGS {
        let select_sql = format!("SELECT * FROM {canonical_table_name}");
        recreate_postgres_compatibility_view(&pool, legacy_table_name, &select_sql).await?;
    }

    recreate_postgres_compatibility_view(
        &pool,
        "catalog_channels",
        "SELECT channel_id AS id, channel_name AS name FROM ai_channel",
    )
    .await?;
    recreate_postgres_compatibility_view(
        &pool,
        "catalog_proxy_providers",
        "SELECT
            proxy_provider_id AS id,
            primary_channel_id AS channel_id,
            extension_id,
            adapter_kind,
            base_url,
            display_name
         FROM ai_proxy_provider",
    )
    .await?;
    recreate_postgres_compatibility_view(
        &pool,
        "catalog_provider_channel_bindings",
        "SELECT
            proxy_provider_id AS provider_id,
            channel_id,
            is_primary
         FROM ai_proxy_provider_channel",
    )
    .await?;
    recreate_postgres_compatibility_view(
        &pool,
        "credential_records",
        "SELECT
            tenant_id,
            proxy_provider_id AS provider_id,
            key_reference,
            secret_backend,
            secret_local_file,
            secret_keyring_service,
            secret_master_key_id,
            secret_ciphertext,
            secret_key_version
         FROM ai_router_credential_records",
    )
    .await?;
    recreate_postgres_compatibility_view(
        &pool,
        "catalog_models",
        "SELECT
            models.model_id AS external_name,
            prices.proxy_provider_id AS provider_id,
            models.capabilities_json AS capabilities,
            models.streaming_enabled AS streaming,
            models.context_window
         FROM ai_model models
         INNER JOIN ai_model_price prices
             ON prices.channel_id = models.channel_id
            AND prices.model_id = models.model_id
         INNER JOIN ai_proxy_provider providers
             ON providers.proxy_provider_id = prices.proxy_provider_id
         WHERE models.channel_id = providers.primary_channel_id",
    )
    .await?;
    recreate_postgres_compatibility_view(
        &pool,
        "identity_gateway_api_keys",
        "SELECT
            hashed_key,
            tenant_id,
            project_id,
            environment,
            label,
            notes,
            created_at_ms,
            last_used_at_ms,
            expires_at_ms,
            active
         FROM ai_app_api_keys",
    )
    .await?;
    Ok(pool)
}

#[derive(Debug, Clone)]
pub struct PostgresAdminStore {
    pool: PgPool,
}

impl PostgresAdminStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn insert_channel(&self, channel: &Channel) -> Result<Channel> {
        let now = current_timestamp_ms();
        sqlx::query(
            "INSERT INTO ai_channel (channel_id, channel_name, created_at_ms, updated_at_ms)
             VALUES ($1, $2, $3, $4)
             ON CONFLICT(channel_id) DO UPDATE SET
                channel_name = excluded.channel_name,
                updated_at_ms = excluded.updated_at_ms,
                is_active = TRUE",
        )
        .bind(&channel.id)
        .bind(&channel.name)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await?;
        Ok(channel.clone())
    }

    pub async fn list_channels(&self) -> Result<Vec<Channel>> {
        let rows = sqlx::query_as::<_, (String, String)>(
            "SELECT channel_id, channel_name
             FROM ai_channel
             WHERE is_active = TRUE
             ORDER BY sort_order, channel_id",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|(id, name)| Channel { id, name })
            .collect())
    }

    pub async fn delete_channel(&self, channel_id: &str) -> Result<bool> {
        sqlx::query("DELETE FROM ai_proxy_provider_channel WHERE channel_id = $1")
            .bind(channel_id)
            .execute(&self.pool)
            .await?;
        sqlx::query("DELETE FROM ai_model_price WHERE channel_id = $1")
            .bind(channel_id)
            .execute(&self.pool)
            .await?;
        sqlx::query("DELETE FROM ai_model WHERE channel_id = $1")
            .bind(channel_id)
            .execute(&self.pool)
            .await?;
        let result = sqlx::query("DELETE FROM ai_channel WHERE channel_id = $1")
            .bind(channel_id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn insert_provider(&self, provider: &ProxyProvider) -> Result<ProxyProvider> {
        let now = current_timestamp_ms();
        sqlx::query(
            "INSERT INTO ai_proxy_provider (
                proxy_provider_id,
                primary_channel_id,
                extension_id,
                adapter_kind,
                base_url,
                display_name,
                is_active,
                created_at_ms,
                updated_at_ms
            ) VALUES ($1, $2, $3, $4, $5, $6, TRUE, $7, $8)
             ON CONFLICT(proxy_provider_id) DO UPDATE SET
                primary_channel_id = excluded.primary_channel_id,
                extension_id = excluded.extension_id,
                adapter_kind = excluded.adapter_kind,
                base_url = excluded.base_url,
                display_name = excluded.display_name,
                is_active = TRUE,
                updated_at_ms = excluded.updated_at_ms",
        )
        .bind(&provider.id)
        .bind(&provider.channel_id)
        .bind(&provider.extension_id)
        .bind(&provider.adapter_kind)
        .bind(&provider.base_url)
        .bind(&provider.display_name)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await?;
        sqlx::query("DELETE FROM ai_proxy_provider_channel WHERE proxy_provider_id = $1")
            .bind(&provider.id)
            .execute(&self.pool)
            .await?;

        for binding in provider_channel_bindings(provider) {
            sqlx::query(
                "INSERT INTO ai_proxy_provider_channel (
                    proxy_provider_id,
                    channel_id,
                    is_primary,
                    created_at_ms,
                    updated_at_ms
                ) VALUES ($1, $2, $3, $4, $5)
                 ON CONFLICT(proxy_provider_id, channel_id) DO UPDATE SET
                    is_primary = excluded.is_primary,
                    updated_at_ms = excluded.updated_at_ms",
            )
            .bind(&binding.provider_id)
            .bind(&binding.channel_id)
            .bind(binding.is_primary)
            .bind(now)
            .bind(now)
            .execute(&self.pool)
            .await?;
        }
        Ok(provider.clone())
    }

    pub async fn list_providers(&self) -> Result<Vec<ProxyProvider>> {
        let rows = sqlx::query_as::<_, (String, String, String, String, String, String)>(
            "SELECT proxy_provider_id, primary_channel_id, extension_id, adapter_kind, base_url, display_name
             FROM ai_proxy_provider
             WHERE is_active = TRUE
             ORDER BY proxy_provider_id",
        )
        .fetch_all(&self.pool)
        .await?;
        let provider_keys = rows
            .iter()
            .map(|(id, channel_id, _, _, _, _)| (id.clone(), channel_id.clone()))
            .collect::<Vec<_>>();
        let bindings_by_provider =
            load_provider_channel_bindings_for_providers(&self.pool, &provider_keys).await?;
        let mut providers = Vec::with_capacity(rows.len());
        for (id, channel_id, extension_id, adapter_kind, base_url, display_name) in rows {
            let channel_bindings = bindings_by_provider.get(&id).cloned().unwrap_or_else(|| {
                vec![ProviderChannelBinding::primary(
                    id.clone(),
                    channel_id.clone(),
                )]
            });
            providers.push(ProxyProvider {
                id,
                channel_id,
                extension_id: normalize_provider_extension_id(extension_id, &adapter_kind),
                adapter_kind,
                base_url,
                display_name,
                channel_bindings,
            });
        }
        Ok(providers)
    }

    pub async fn list_providers_for_model(&self, model: &str) -> Result<Vec<ProxyProvider>> {
        let rows = sqlx::query_as::<_, (String, String, String, String, String, String)>(
            "SELECT DISTINCT providers.proxy_provider_id, providers.primary_channel_id, providers.extension_id, providers.adapter_kind, providers.base_url, providers.display_name
             FROM ai_model models
             INNER JOIN ai_model_price prices
                 ON prices.channel_id = models.channel_id
                AND prices.model_id = models.model_id
             INNER JOIN ai_proxy_provider providers
                 ON providers.proxy_provider_id = prices.proxy_provider_id
             WHERE models.model_id = $1
               AND prices.is_active = TRUE
               AND providers.is_active = TRUE
             ORDER BY providers.proxy_provider_id",
        )
        .bind(model)
        .fetch_all(&self.pool)
        .await?;
        let provider_keys = rows
            .iter()
            .map(|(id, channel_id, _, _, _, _)| (id.clone(), channel_id.clone()))
            .collect::<Vec<_>>();
        let bindings_by_provider =
            load_provider_channel_bindings_for_providers(&self.pool, &provider_keys).await?;
        let mut providers = Vec::with_capacity(rows.len());
        for (id, channel_id, extension_id, adapter_kind, base_url, display_name) in rows {
            let channel_bindings = bindings_by_provider.get(&id).cloned().unwrap_or_else(|| {
                vec![ProviderChannelBinding::primary(
                    id.clone(),
                    channel_id.clone(),
                )]
            });
            providers.push(ProxyProvider {
                id,
                channel_id,
                extension_id: normalize_provider_extension_id(extension_id, &adapter_kind),
                adapter_kind,
                base_url,
                display_name,
                channel_bindings,
            });
        }
        Ok(providers)
    }

    pub async fn find_provider(&self, provider_id: &str) -> Result<Option<ProxyProvider>> {
        let row = sqlx::query_as::<_, (String, String, String, String, String, String)>(
            "SELECT proxy_provider_id, primary_channel_id, extension_id, adapter_kind, base_url, display_name
             FROM ai_proxy_provider
             WHERE proxy_provider_id = $1",
        )
        .bind(provider_id)
        .fetch_optional(&self.pool)
        .await?;

        let Some((id, channel_id, extension_id, adapter_kind, base_url, display_name)) = row else {
            return Ok(None);
        };

        let channel_bindings = load_provider_channel_bindings(&self.pool, &id, &channel_id).await?;

        Ok(Some(ProxyProvider {
            id,
            channel_id,
            extension_id: normalize_provider_extension_id(extension_id, &adapter_kind),
            adapter_kind,
            base_url,
            display_name,
            channel_bindings,
        }))
    }

    pub async fn delete_provider(&self, provider_id: &str) -> Result<bool> {
        sqlx::query("DELETE FROM ai_router_credential_records WHERE proxy_provider_id = $1")
            .bind(provider_id)
            .execute(&self.pool)
            .await?;
        sqlx::query("DELETE FROM ai_model_price WHERE proxy_provider_id = $1")
            .bind(provider_id)
            .execute(&self.pool)
            .await?;
        sqlx::query("DELETE FROM ai_routing_policy_providers WHERE provider_id = $1")
            .bind(provider_id)
            .execute(&self.pool)
            .await?;
        sqlx::query(
            "UPDATE ai_routing_policies SET default_provider_id = NULL WHERE default_provider_id = $1",
        )
        .bind(provider_id)
        .execute(&self.pool)
        .await?;
        sqlx::query("DELETE FROM ai_proxy_provider_channel WHERE proxy_provider_id = $1")
            .bind(provider_id)
            .execute(&self.pool)
            .await?;
        let result = sqlx::query("DELETE FROM ai_proxy_provider WHERE proxy_provider_id = $1")
            .bind(provider_id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn insert_credential(
        &self,
        credential: &UpstreamCredential,
    ) -> Result<UpstreamCredential> {
        let now = current_timestamp_ms();
        sqlx::query(
            "INSERT INTO ai_router_credential_records (
                tenant_id,
                proxy_provider_id,
                key_reference,
                secret_backend,
                secret_local_file,
                secret_keyring_service,
                secret_master_key_id,
                secret_ciphertext,
                secret_key_version,
                created_at_ms,
                updated_at_ms
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, NULL, NULL, $8, $9)
             ON CONFLICT(tenant_id, proxy_provider_id, key_reference) DO UPDATE SET
                secret_backend = excluded.secret_backend,
                secret_local_file = excluded.secret_local_file,
                secret_keyring_service = excluded.secret_keyring_service,
                secret_master_key_id = excluded.secret_master_key_id,
                secret_ciphertext = NULL,
                secret_key_version = NULL,
                updated_at_ms = excluded.updated_at_ms",
        )
        .bind(&credential.tenant_id)
        .bind(&credential.provider_id)
        .bind(&credential.key_reference)
        .bind(&credential.secret_backend)
        .bind(&credential.secret_local_file)
        .bind(&credential.secret_keyring_service)
        .bind(&credential.secret_master_key_id)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await?;
        Ok(credential.clone())
    }

    pub async fn insert_encrypted_credential(
        &self,
        credential: &UpstreamCredential,
        envelope: &SecretEnvelope,
    ) -> Result<UpstreamCredential> {
        let now = current_timestamp_ms();
        sqlx::query(
            "INSERT INTO ai_router_credential_records (
                tenant_id,
                proxy_provider_id,
                key_reference,
                secret_backend,
                secret_local_file,
                secret_keyring_service,
                secret_master_key_id,
                secret_ciphertext,
                secret_key_version,
                created_at_ms,
                updated_at_ms
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
             ON CONFLICT(tenant_id, proxy_provider_id, key_reference) DO UPDATE SET
                secret_backend = excluded.secret_backend,
                secret_local_file = excluded.secret_local_file,
                secret_keyring_service = excluded.secret_keyring_service,
                secret_master_key_id = excluded.secret_master_key_id,
                secret_ciphertext = excluded.secret_ciphertext,
                secret_key_version = excluded.secret_key_version,
                updated_at_ms = excluded.updated_at_ms",
        )
        .bind(&credential.tenant_id)
        .bind(&credential.provider_id)
        .bind(&credential.key_reference)
        .bind(&credential.secret_backend)
        .bind(&credential.secret_local_file)
        .bind(&credential.secret_keyring_service)
        .bind(&credential.secret_master_key_id)
        .bind(&envelope.ciphertext)
        .bind(i32::try_from(envelope.key_version)?)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await?;
        Ok(credential.clone())
    }

    pub async fn list_credentials(&self) -> Result<Vec<UpstreamCredential>> {
        let rows = sqlx::query_as::<_, CredentialRow>(
            "SELECT tenant_id, proxy_provider_id, key_reference, secret_backend, secret_local_file, secret_keyring_service, secret_master_key_id
             FROM ai_router_credential_records
             ORDER BY proxy_provider_id, tenant_id, updated_at_ms DESC, created_at_ms DESC",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(decode_credential_row).collect())
    }

    pub async fn list_credentials_for_tenant(
        &self,
        tenant_id: &str,
    ) -> Result<Vec<UpstreamCredential>> {
        let rows = sqlx::query_as::<_, CredentialRow>(
            "SELECT tenant_id, proxy_provider_id, key_reference, secret_backend, secret_local_file, secret_keyring_service, secret_master_key_id
             FROM ai_router_credential_records
             WHERE tenant_id = $1
             ORDER BY updated_at_ms DESC, proxy_provider_id, key_reference",
        )
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(decode_credential_row).collect())
    }

    pub async fn list_credentials_for_provider(
        &self,
        provider_id: &str,
    ) -> Result<Vec<UpstreamCredential>> {
        let rows = sqlx::query_as::<_, CredentialRow>(
            "SELECT tenant_id, proxy_provider_id, key_reference, secret_backend, secret_local_file, secret_keyring_service, secret_master_key_id
             FROM ai_router_credential_records
             WHERE proxy_provider_id = $1
             ORDER BY updated_at_ms DESC, tenant_id, key_reference",
        )
        .bind(provider_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(decode_credential_row).collect())
    }

    pub async fn find_credential(
        &self,
        tenant_id: &str,
        provider_id: &str,
        key_reference: &str,
    ) -> Result<Option<UpstreamCredential>> {
        let row = sqlx::query_as::<_, CredentialRow>(
            "SELECT tenant_id, proxy_provider_id, key_reference, secret_backend, secret_local_file, secret_keyring_service, secret_master_key_id
             FROM ai_router_credential_records
             WHERE tenant_id = $1 AND proxy_provider_id = $2 AND key_reference = $3",
        )
        .bind(tenant_id)
        .bind(provider_id)
        .bind(key_reference)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(decode_credential_row))
    }

    pub async fn find_credential_envelope(
        &self,
        tenant_id: &str,
        provider_id: &str,
        key_reference: &str,
    ) -> Result<Option<SecretEnvelope>> {
        let row = sqlx::query_as::<_, (Option<String>, Option<i32>)>(
            "SELECT secret_ciphertext, secret_key_version
             FROM ai_router_credential_records
             WHERE tenant_id = $1 AND proxy_provider_id = $2 AND key_reference = $3",
        )
        .bind(tenant_id)
        .bind(provider_id)
        .bind(key_reference)
        .fetch_optional(&self.pool)
        .await?;

        let Some((Some(ciphertext), Some(key_version))) = row else {
            return Ok(None);
        };

        Ok(Some(SecretEnvelope {
            ciphertext,
            key_version: u32::try_from(key_version)?,
        }))
    }

    pub async fn find_provider_credential(
        &self,
        tenant_id: &str,
        provider_id: &str,
    ) -> Result<Option<UpstreamCredential>> {
        let row = sqlx::query_as::<_, CredentialRow>(
            "SELECT tenant_id, proxy_provider_id, key_reference, secret_backend, secret_local_file, secret_keyring_service, secret_master_key_id
             FROM ai_router_credential_records
             WHERE tenant_id = $1 AND proxy_provider_id = $2
             ORDER BY updated_at_ms DESC, created_at_ms DESC
             LIMIT 1",
        )
        .bind(tenant_id)
        .bind(provider_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(decode_credential_row))
    }

    pub async fn delete_credential(
        &self,
        tenant_id: &str,
        provider_id: &str,
        key_reference: &str,
    ) -> Result<bool> {
        let result = sqlx::query(
            "DELETE FROM ai_router_credential_records
             WHERE tenant_id = $1 AND proxy_provider_id = $2 AND key_reference = $3",
        )
        .bind(tenant_id)
        .bind(provider_id)
        .bind(key_reference)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn insert_model(&self, model: &ModelCatalogEntry) -> Result<ModelCatalogEntry> {
        let provider = self
            .find_provider(&model.provider_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("provider_id is not registered"))?;
        let mut channel_model = ChannelModelRecord::new(
            &provider.channel_id,
            &model.external_name,
            &model.external_name,
        )
        .with_context_window_option(model.context_window)
        .with_streaming(model.streaming);
        for capability in &model.capabilities {
            channel_model = channel_model.with_capability(capability.clone());
        }
        self.insert_channel_model(&channel_model).await?;
        self.insert_model_price(&ModelPriceRecord::new(
            &provider.channel_id,
            &model.external_name,
            &model.provider_id,
        ))
        .await?;
        Ok(model.clone())
    }

    pub async fn list_models(&self) -> Result<Vec<ModelCatalogEntry>> {
        let rows = sqlx::query_as::<_, (String, String, String, bool, Option<i64>)>(
            "SELECT
                models.model_id,
                prices.proxy_provider_id,
                models.capabilities_json,
                models.streaming_enabled,
                models.context_window
             FROM ai_model models
             INNER JOIN ai_model_price prices
                 ON prices.channel_id = models.channel_id
                AND prices.model_id = models.model_id
             WHERE prices.is_active = TRUE
             ORDER BY models.model_id, prices.proxy_provider_id",
        )
        .fetch_all(&self.pool)
        .await?;
        let mut models = Vec::with_capacity(rows.len());
        for (external_name, provider_id, capabilities, streaming, context_window) in rows {
            models.push(ModelCatalogEntry {
                external_name,
                provider_id,
                capabilities: decode_model_capabilities(&capabilities)?,
                streaming,
                context_window: context_window.map(u64::try_from).transpose()?,
            });
        }
        Ok(models)
    }

    pub async fn list_models_for_external_name(
        &self,
        external_name: &str,
    ) -> Result<Vec<ModelCatalogEntry>> {
        let rows = sqlx::query_as::<_, (String, String, String, bool, Option<i64>)>(
            "SELECT
                models.model_id,
                prices.proxy_provider_id,
                models.capabilities_json,
                models.streaming_enabled,
                models.context_window
             FROM ai_model models
             INNER JOIN ai_model_price prices
                 ON prices.channel_id = models.channel_id
                AND prices.model_id = models.model_id
             WHERE models.model_id = $1
               AND prices.is_active = TRUE
             ORDER BY prices.proxy_provider_id",
        )
        .bind(external_name)
        .fetch_all(&self.pool)
        .await?;

        let mut models = Vec::with_capacity(rows.len());
        for (external_name, provider_id, capabilities, streaming, context_window) in rows {
            models.push(ModelCatalogEntry {
                external_name,
                provider_id,
                capabilities: decode_model_capabilities(&capabilities)?,
                streaming,
                context_window: context_window.map(u64::try_from).transpose()?,
            });
        }
        Ok(models)
    }

    pub async fn find_any_model(&self) -> Result<Option<ModelCatalogEntry>> {
        let row = sqlx::query_as::<_, (String, String, String, bool, Option<i64>)>(
            "SELECT
                models.model_id,
                prices.proxy_provider_id,
                models.capabilities_json,
                models.streaming_enabled,
                models.context_window
             FROM ai_model models
             INNER JOIN ai_model_price prices
                 ON prices.channel_id = models.channel_id
                AND prices.model_id = models.model_id
             WHERE prices.is_active = TRUE
             ORDER BY models.model_id, prices.proxy_provider_id
             LIMIT 1",
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(match row {
            Some((external_name, provider_id, capabilities, streaming, context_window)) => {
                Some(ModelCatalogEntry {
                    external_name,
                    provider_id,
                    capabilities: decode_model_capabilities(&capabilities)?,
                    streaming,
                    context_window: context_window.map(u64::try_from).transpose()?,
                })
            }
            None => None,
        })
    }

    pub async fn find_model(&self, external_name: &str) -> Result<Option<ModelCatalogEntry>> {
        let row = sqlx::query_as::<_, (String, String, String, bool, Option<i64>)>(
            "SELECT
                models.model_id,
                prices.proxy_provider_id,
                models.capabilities_json,
                models.streaming_enabled,
                models.context_window
             FROM ai_model models
             INNER JOIN ai_model_price prices
                 ON prices.channel_id = models.channel_id
                AND prices.model_id = models.model_id
             WHERE models.model_id = $1
               AND prices.is_active = TRUE
             ORDER BY prices.proxy_provider_id
             LIMIT 1",
        )
        .bind(external_name)
        .fetch_optional(&self.pool)
        .await?;

        Ok(match row {
            Some((external_name, provider_id, capabilities, streaming, context_window)) => {
                Some(ModelCatalogEntry {
                    external_name,
                    provider_id,
                    capabilities: decode_model_capabilities(&capabilities)?,
                    streaming,
                    context_window: context_window.map(u64::try_from).transpose()?,
                })
            }
            None => None,
        })
    }

    pub async fn delete_model(&self, external_name: &str) -> Result<bool> {
        sqlx::query("DELETE FROM ai_model_price WHERE model_id = $1")
            .bind(external_name)
            .execute(&self.pool)
            .await?;
        let result = sqlx::query("DELETE FROM ai_model WHERE model_id = $1")
            .bind(external_name)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn delete_model_variant(
        &self,
        external_name: &str,
        provider_id: &str,
    ) -> Result<bool> {
        let result = sqlx::query(
            "DELETE FROM ai_model_price WHERE model_id = $1 AND proxy_provider_id = $2",
        )
        .bind(external_name)
        .bind(provider_id)
        .execute(&self.pool)
        .await?;
        sqlx::query(
            "DELETE FROM ai_model
             WHERE model_id = $1
               AND NOT EXISTS (
                   SELECT 1
                   FROM ai_model_price prices
                   WHERE prices.channel_id = ai_model.channel_id
                     AND prices.model_id = ai_model.model_id
               )",
        )
        .bind(external_name)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn insert_channel_model(
        &self,
        record: &ChannelModelRecord,
    ) -> Result<ChannelModelRecord> {
        let now = current_timestamp_ms();
        sqlx::query(
            "INSERT INTO ai_model (
                channel_id,
                model_id,
                model_display_name,
                capabilities_json,
                streaming_enabled,
                context_window,
                description,
                created_at_ms,
                updated_at_ms
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
             ON CONFLICT(channel_id, model_id) DO UPDATE SET
                model_display_name = excluded.model_display_name,
                capabilities_json = excluded.capabilities_json,
                streaming_enabled = excluded.streaming_enabled,
                context_window = excluded.context_window,
                description = excluded.description,
                updated_at_ms = excluded.updated_at_ms",
        )
        .bind(&record.channel_id)
        .bind(&record.model_id)
        .bind(&record.model_display_name)
        .bind(encode_model_capabilities(&record.capabilities)?)
        .bind(record.streaming)
        .bind(record.context_window.map(i64::try_from).transpose()?)
        .bind(record.description.clone().unwrap_or_default())
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await?;
        Ok(record.clone())
    }

    pub async fn list_channel_models(&self) -> Result<Vec<ChannelModelRecord>> {
        let rows = sqlx::query_as::<_, ChannelModelRow>(
            "SELECT
                channel_id,
                model_id,
                model_display_name,
                capabilities_json,
                streaming_enabled,
                context_window,
                description
             FROM ai_model
             ORDER BY channel_id, model_id",
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(decode_channel_model_row).collect()
    }

    pub async fn delete_channel_model(&self, channel_id: &str, model_id: &str) -> Result<bool> {
        sqlx::query("DELETE FROM ai_model_price WHERE channel_id = $1 AND model_id = $2")
            .bind(channel_id)
            .bind(model_id)
            .execute(&self.pool)
            .await?;
        let result = sqlx::query("DELETE FROM ai_model WHERE channel_id = $1 AND model_id = $2")
            .bind(channel_id)
            .bind(model_id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn insert_model_price(&self, record: &ModelPriceRecord) -> Result<ModelPriceRecord> {
        let now = current_timestamp_ms();
        sqlx::query(
            "INSERT INTO ai_model_price (
                channel_id,
                model_id,
                proxy_provider_id,
                currency_code,
                price_unit,
                input_price,
                output_price,
                cache_read_price,
                cache_write_price,
                request_price,
                is_active,
                created_at_ms,
                updated_at_ms
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
             ON CONFLICT(channel_id, model_id, proxy_provider_id) DO UPDATE SET
                currency_code = excluded.currency_code,
                price_unit = excluded.price_unit,
                input_price = excluded.input_price,
                output_price = excluded.output_price,
                cache_read_price = excluded.cache_read_price,
                cache_write_price = excluded.cache_write_price,
                request_price = excluded.request_price,
                is_active = excluded.is_active,
                updated_at_ms = excluded.updated_at_ms",
        )
        .bind(&record.channel_id)
        .bind(&record.model_id)
        .bind(&record.proxy_provider_id)
        .bind(&record.currency_code)
        .bind(&record.price_unit)
        .bind(record.input_price)
        .bind(record.output_price)
        .bind(record.cache_read_price)
        .bind(record.cache_write_price)
        .bind(record.request_price)
        .bind(record.is_active)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await?;
        Ok(record.clone())
    }

    pub async fn list_model_prices(&self) -> Result<Vec<ModelPriceRecord>> {
        let rows = sqlx::query_as::<_, ModelPriceRow>(
            "SELECT
                channel_id,
                model_id,
                proxy_provider_id,
                currency_code,
                price_unit,
                input_price,
                output_price,
                cache_read_price,
                cache_write_price,
                request_price,
                is_active
             FROM ai_model_price
             ORDER BY channel_id, model_id, proxy_provider_id",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(decode_model_price_row).collect())
    }

    pub async fn delete_model_price(
        &self,
        channel_id: &str,
        model_id: &str,
        proxy_provider_id: &str,
    ) -> Result<bool> {
        let result = sqlx::query(
            "DELETE FROM ai_model_price
             WHERE channel_id = $1 AND model_id = $2 AND proxy_provider_id = $3",
        )
        .bind(channel_id)
        .bind(model_id)
        .bind(proxy_provider_id)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn insert_routing_policy(&self, policy: &RoutingPolicy) -> Result<RoutingPolicy> {
        sqlx::query(
            "INSERT INTO ai_routing_policies (policy_id, capability, model_pattern, enabled, priority, strategy, default_provider_id, max_cost, max_latency_ms, require_healthy) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
             ON CONFLICT(policy_id) DO UPDATE SET capability = excluded.capability, model_pattern = excluded.model_pattern, enabled = excluded.enabled, priority = excluded.priority, strategy = excluded.strategy, default_provider_id = excluded.default_provider_id, max_cost = excluded.max_cost, max_latency_ms = excluded.max_latency_ms, require_healthy = excluded.require_healthy",
        )
        .bind(&policy.policy_id)
        .bind(&policy.capability)
        .bind(&policy.model_pattern)
        .bind(policy.enabled)
        .bind(policy.priority)
        .bind(policy.strategy.as_str())
        .bind(&policy.default_provider_id)
        .bind(policy.max_cost)
        .bind(policy.max_latency_ms.map(i64::try_from).transpose()?)
        .bind(policy.require_healthy)
        .execute(&self.pool)
        .await?;

        sqlx::query("DELETE FROM ai_routing_policy_providers WHERE policy_id = $1")
            .bind(&policy.policy_id)
            .execute(&self.pool)
            .await?;

        for (position, provider_id) in policy.ordered_provider_ids.iter().enumerate() {
            sqlx::query(
                "INSERT INTO ai_routing_policy_providers (policy_id, provider_id, position) VALUES ($1, $2, $3)
                 ON CONFLICT(policy_id, provider_id) DO UPDATE SET position = excluded.position",
            )
            .bind(&policy.policy_id)
            .bind(provider_id)
            .bind(i32::try_from(position)?)
            .execute(&self.pool)
            .await?;
        }

        Ok(policy.clone())
    }

    pub async fn list_routing_policies(&self) -> Result<Vec<RoutingPolicy>> {
        let rows = sqlx::query_as::<
            _,
            (
                String,
                String,
                String,
                bool,
                i32,
                String,
                Option<String>,
                Option<f64>,
                Option<i64>,
                bool,
            ),
        >(
            "SELECT policy_id, capability, model_pattern, enabled, priority, strategy, default_provider_id, max_cost, max_latency_ms, require_healthy
             FROM ai_routing_policies
             ORDER BY priority DESC, policy_id",
        )
        .fetch_all(&self.pool)
        .await?;

        let mut policies = Vec::with_capacity(rows.len());
        for (
            policy_id,
            capability,
            model_pattern,
            enabled,
            priority,
            strategy,
            default_provider_id,
            max_cost,
            max_latency_ms,
            require_healthy,
        ) in rows
        {
            policies.push(
                RoutingPolicy::new(policy_id.clone(), capability, model_pattern)
                    .with_enabled(enabled)
                    .with_priority(priority)
                    .with_strategy(
                        RoutingStrategy::from_str(&strategy)
                            .unwrap_or(RoutingStrategy::DeterministicPriority),
                    )
                    .with_ordered_provider_ids(
                        load_routing_policy_provider_ids(&self.pool, &policy_id).await?,
                    )
                    .with_default_provider_id_option(default_provider_id)
                    .with_max_cost_option(max_cost)
                    .with_max_latency_ms_option(max_latency_ms.map(u64::try_from).transpose()?)
                    .with_require_healthy(require_healthy),
            );
        }
        Ok(policies)
    }

    pub async fn insert_routing_profile(
        &self,
        profile: &RoutingProfileRecord,
    ) -> Result<RoutingProfileRecord> {
        sqlx::query(
            "INSERT INTO ai_routing_profiles (
                profile_id,
                tenant_id,
                project_id,
                name,
                slug,
                description,
                active,
                strategy,
                ordered_provider_ids_json,
                default_provider_id,
                max_cost,
                max_latency_ms,
                require_healthy,
                preferred_region,
                created_at_ms,
                updated_at_ms
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
            ON CONFLICT(profile_id) DO UPDATE SET
                tenant_id = excluded.tenant_id,
                project_id = excluded.project_id,
                name = excluded.name,
                slug = excluded.slug,
                description = excluded.description,
                active = excluded.active,
                strategy = excluded.strategy,
                ordered_provider_ids_json = excluded.ordered_provider_ids_json,
                default_provider_id = excluded.default_provider_id,
                max_cost = excluded.max_cost,
                max_latency_ms = excluded.max_latency_ms,
                require_healthy = excluded.require_healthy,
                preferred_region = excluded.preferred_region,
                created_at_ms = excluded.created_at_ms,
                updated_at_ms = excluded.updated_at_ms",
        )
        .bind(&profile.profile_id)
        .bind(&profile.tenant_id)
        .bind(&profile.project_id)
        .bind(&profile.name)
        .bind(&profile.slug)
        .bind(&profile.description)
        .bind(profile.active)
        .bind(profile.strategy.as_str())
        .bind(encode_string_list(&profile.ordered_provider_ids)?)
        .bind(&profile.default_provider_id)
        .bind(profile.max_cost)
        .bind(profile.max_latency_ms.map(i64::try_from).transpose()?)
        .bind(profile.require_healthy)
        .bind(&profile.preferred_region)
        .bind(i64::try_from(profile.created_at_ms)?)
        .bind(i64::try_from(profile.updated_at_ms)?)
        .execute(&self.pool)
        .await?;

        Ok(profile.clone())
    }

    pub async fn list_routing_profiles(&self) -> Result<Vec<RoutingProfileRecord>> {
        let rows = sqlx::query(
            "SELECT profile_id, tenant_id, project_id, name, slug, description, active, strategy, ordered_provider_ids_json, default_provider_id, max_cost, max_latency_ms, require_healthy, preferred_region, created_at_ms, updated_at_ms
             FROM ai_routing_profiles
             ORDER BY updated_at_ms DESC, profile_id",
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(decode_routing_profile_row).collect()
    }

    pub async fn find_routing_profile(
        &self,
        profile_id: &str,
    ) -> Result<Option<RoutingProfileRecord>> {
        let row = sqlx::query(
            "SELECT profile_id, tenant_id, project_id, name, slug, description, active, strategy, ordered_provider_ids_json, default_provider_id, max_cost, max_latency_ms, require_healthy, preferred_region, created_at_ms, updated_at_ms
             FROM ai_routing_profiles
             WHERE profile_id = $1",
        )
        .bind(profile_id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(decode_routing_profile_row).transpose()
    }

    pub async fn insert_compiled_routing_snapshot(
        &self,
        snapshot: &CompiledRoutingSnapshotRecord,
    ) -> Result<CompiledRoutingSnapshotRecord> {
        sqlx::query(
            "INSERT INTO ai_compiled_routing_snapshots (
                snapshot_id,
                tenant_id,
                project_id,
                api_key_group_id,
                capability,
                route_key,
                matched_policy_id,
                project_routing_preferences_project_id,
                applied_routing_profile_id,
                strategy,
                ordered_provider_ids_json,
                default_provider_id,
                max_cost,
                max_latency_ms,
                require_healthy,
                preferred_region,
                created_at_ms,
                updated_at_ms
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18)
            ON CONFLICT(snapshot_id) DO UPDATE SET
                tenant_id = excluded.tenant_id,
                project_id = excluded.project_id,
                api_key_group_id = excluded.api_key_group_id,
                capability = excluded.capability,
                route_key = excluded.route_key,
                matched_policy_id = excluded.matched_policy_id,
                project_routing_preferences_project_id = excluded.project_routing_preferences_project_id,
                applied_routing_profile_id = excluded.applied_routing_profile_id,
                strategy = excluded.strategy,
                ordered_provider_ids_json = excluded.ordered_provider_ids_json,
                default_provider_id = excluded.default_provider_id,
                max_cost = excluded.max_cost,
                max_latency_ms = excluded.max_latency_ms,
                require_healthy = excluded.require_healthy,
                preferred_region = excluded.preferred_region,
                created_at_ms = excluded.created_at_ms,
                updated_at_ms = excluded.updated_at_ms",
        )
        .bind(&snapshot.snapshot_id)
        .bind(&snapshot.tenant_id)
        .bind(&snapshot.project_id)
        .bind(&snapshot.api_key_group_id)
        .bind(&snapshot.capability)
        .bind(&snapshot.route_key)
        .bind(&snapshot.matched_policy_id)
        .bind(&snapshot.project_routing_preferences_project_id)
        .bind(&snapshot.applied_routing_profile_id)
        .bind(&snapshot.strategy)
        .bind(encode_string_list(&snapshot.ordered_provider_ids)?)
        .bind(&snapshot.default_provider_id)
        .bind(snapshot.max_cost)
        .bind(snapshot.max_latency_ms.map(i64::try_from).transpose()?)
        .bind(snapshot.require_healthy)
        .bind(&snapshot.preferred_region)
        .bind(i64::try_from(snapshot.created_at_ms)?)
        .bind(i64::try_from(snapshot.updated_at_ms)?)
        .execute(&self.pool)
        .await?;

        Ok(snapshot.clone())
    }

    pub async fn list_compiled_routing_snapshots(
        &self,
    ) -> Result<Vec<CompiledRoutingSnapshotRecord>> {
        let rows = sqlx::query(
            "SELECT snapshot_id, tenant_id, project_id, api_key_group_id, capability, route_key, matched_policy_id, project_routing_preferences_project_id, applied_routing_profile_id, strategy, ordered_provider_ids_json, default_provider_id, max_cost, max_latency_ms, require_healthy, preferred_region, created_at_ms, updated_at_ms
             FROM ai_compiled_routing_snapshots
             ORDER BY updated_at_ms DESC, snapshot_id",
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(decode_compiled_routing_snapshot_row)
            .collect()
    }

    pub async fn insert_project_routing_preferences(
        &self,
        preferences: &ProjectRoutingPreferences,
    ) -> Result<ProjectRoutingPreferences> {
        sqlx::query(
            "INSERT INTO ai_project_routing_preferences (
                project_id,
                preset_id,
                strategy,
                ordered_provider_ids_json,
                default_provider_id,
                max_cost,
                max_latency_ms,
                require_healthy,
                preferred_region,
                updated_at_ms
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT(project_id) DO UPDATE SET
                preset_id = excluded.preset_id,
                strategy = excluded.strategy,
                ordered_provider_ids_json = excluded.ordered_provider_ids_json,
                default_provider_id = excluded.default_provider_id,
                max_cost = excluded.max_cost,
                max_latency_ms = excluded.max_latency_ms,
                require_healthy = excluded.require_healthy,
                preferred_region = excluded.preferred_region,
                updated_at_ms = excluded.updated_at_ms",
        )
        .bind(&preferences.project_id)
        .bind(&preferences.preset_id)
        .bind(preferences.strategy.as_str())
        .bind(encode_string_list(&preferences.ordered_provider_ids)?)
        .bind(&preferences.default_provider_id)
        .bind(preferences.max_cost)
        .bind(preferences.max_latency_ms.map(i64::try_from).transpose()?)
        .bind(preferences.require_healthy)
        .bind(&preferences.preferred_region)
        .bind(i64::try_from(preferences.updated_at_ms)?)
        .execute(&self.pool)
        .await?;

        Ok(preferences.clone())
    }

    pub async fn find_project_routing_preferences(
        &self,
        project_id: &str,
    ) -> Result<Option<ProjectRoutingPreferences>> {
        let row = sqlx::query_as::<
            _,
            (
                String,
                String,
                String,
                String,
                Option<String>,
                Option<f64>,
                Option<i64>,
                bool,
                Option<String>,
                i64,
            ),
        >(
            "SELECT project_id, preset_id, strategy, ordered_provider_ids_json, default_provider_id, max_cost, max_latency_ms, require_healthy, preferred_region, updated_at_ms
             FROM ai_project_routing_preferences
             WHERE project_id = $1",
        )
        .bind(project_id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(
            |(
                project_id,
                preset_id,
                strategy,
                ordered_provider_ids_json,
                default_provider_id,
                max_cost,
                max_latency_ms,
                require_healthy,
                preferred_region,
                updated_at_ms,
            )| {
                Ok(ProjectRoutingPreferences::new(project_id)
                    .with_preset_id(preset_id)
                    .with_strategy(
                        RoutingStrategy::from_str(&strategy)
                            .unwrap_or(RoutingStrategy::DeterministicPriority),
                    )
                    .with_ordered_provider_ids(decode_string_list(&ordered_provider_ids_json)?)
                    .with_default_provider_id_option(default_provider_id)
                    .with_max_cost_option(max_cost)
                    .with_max_latency_ms_option(max_latency_ms.map(u64::try_from).transpose()?)
                    .with_require_healthy(require_healthy)
                    .with_preferred_region_option(preferred_region)
                    .with_updated_at_ms(u64::try_from(updated_at_ms)?))
            },
        )
        .transpose()
    }

    pub async fn insert_routing_decision_log(
        &self,
        log: &RoutingDecisionLog,
    ) -> Result<RoutingDecisionLog> {
        sqlx::query(
            "INSERT INTO ai_routing_decision_logs (decision_id, decision_source, tenant_id, project_id, api_key_group_id, capability, route_key, selected_provider_id, matched_policy_id, applied_routing_profile_id, compiled_routing_snapshot_id, strategy, selection_seed, selection_reason, fallback_reason, requested_region, slo_applied, slo_degraded, created_at_ms, assessments_json)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20)
             ON CONFLICT(decision_id) DO UPDATE SET decision_source = excluded.decision_source, tenant_id = excluded.tenant_id, project_id = excluded.project_id, api_key_group_id = excluded.api_key_group_id, capability = excluded.capability, route_key = excluded.route_key, selected_provider_id = excluded.selected_provider_id, matched_policy_id = excluded.matched_policy_id, applied_routing_profile_id = excluded.applied_routing_profile_id, compiled_routing_snapshot_id = excluded.compiled_routing_snapshot_id, strategy = excluded.strategy, selection_seed = excluded.selection_seed, selection_reason = excluded.selection_reason, fallback_reason = excluded.fallback_reason, requested_region = excluded.requested_region, slo_applied = excluded.slo_applied, slo_degraded = excluded.slo_degraded, created_at_ms = excluded.created_at_ms, assessments_json = excluded.assessments_json",
        )
        .bind(&log.decision_id)
        .bind(log.decision_source.as_str())
        .bind(&log.tenant_id)
        .bind(&log.project_id)
        .bind(&log.api_key_group_id)
        .bind(&log.capability)
        .bind(&log.route_key)
        .bind(&log.selected_provider_id)
        .bind(&log.matched_policy_id)
        .bind(&log.applied_routing_profile_id)
        .bind(&log.compiled_routing_snapshot_id)
        .bind(&log.strategy)
        .bind(log.selection_seed.map(i64::try_from).transpose()?)
        .bind(&log.selection_reason)
        .bind(&log.fallback_reason)
        .bind(&log.requested_region)
        .bind(log.slo_applied)
        .bind(log.slo_degraded)
        .bind(i64::try_from(log.created_at_ms)?)
        .bind(encode_routing_assessments(&log.assessments)?)
        .execute(&self.pool)
        .await?;

        Ok(log.clone())
    }

    pub async fn list_routing_decision_logs(&self) -> Result<Vec<RoutingDecisionLog>> {
        let rows = sqlx::query(
            "SELECT decision_id, decision_source, tenant_id, project_id, api_key_group_id, capability, route_key, selected_provider_id, matched_policy_id, applied_routing_profile_id, compiled_routing_snapshot_id, strategy, selection_seed, selection_reason, fallback_reason, requested_region, slo_applied, slo_degraded, created_at_ms, assessments_json
             FROM ai_routing_decision_logs
             ORDER BY created_at_ms DESC, decision_id DESC",
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(decode_routing_decision_log_row)
            .collect()
    }

    pub async fn list_routing_decision_logs_for_project(
        &self,
        project_id: &str,
    ) -> Result<Vec<RoutingDecisionLog>> {
        let rows = sqlx::query(
            "SELECT decision_id, decision_source, tenant_id, project_id, api_key_group_id, capability, route_key, selected_provider_id, matched_policy_id, applied_routing_profile_id, compiled_routing_snapshot_id, strategy, selection_seed, selection_reason, fallback_reason, requested_region, slo_applied, slo_degraded, created_at_ms, assessments_json
             FROM ai_routing_decision_logs
             WHERE project_id = $1
             ORDER BY created_at_ms DESC, decision_id DESC",
        )
        .bind(project_id)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(decode_routing_decision_log_row)
            .collect()
    }

    pub async fn find_latest_routing_decision_log_for_project(
        &self,
        project_id: &str,
    ) -> Result<Option<RoutingDecisionLog>> {
        let row = sqlx::query(
            "SELECT decision_id, decision_source, tenant_id, project_id, api_key_group_id, capability, route_key, selected_provider_id, matched_policy_id, applied_routing_profile_id, compiled_routing_snapshot_id, strategy, selection_seed, selection_reason, fallback_reason, requested_region, slo_applied, slo_degraded, created_at_ms, assessments_json
             FROM ai_routing_decision_logs
             WHERE project_id = $1
             ORDER BY created_at_ms DESC, decision_id DESC
             LIMIT 1",
        )
        .bind(project_id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(decode_routing_decision_log_row).transpose()
    }

    pub async fn insert_provider_health_snapshot(
        &self,
        snapshot: &ProviderHealthSnapshot,
    ) -> Result<ProviderHealthSnapshot> {
        sqlx::query(
            "INSERT INTO ai_provider_health_records (provider_id, extension_id, runtime, observed_at_ms, instance_id, running, healthy, message)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        )
        .bind(&snapshot.provider_id)
        .bind(&snapshot.extension_id)
        .bind(&snapshot.runtime)
        .bind(i64::try_from(snapshot.observed_at_ms)?)
        .bind(&snapshot.instance_id)
        .bind(snapshot.running)
        .bind(snapshot.healthy)
        .bind(&snapshot.message)
        .execute(&self.pool)
        .await?;

        Ok(snapshot.clone())
    }

    pub async fn list_provider_health_snapshots(&self) -> Result<Vec<ProviderHealthSnapshot>> {
        let rows = sqlx::query_as::<
            _,
            (
                String,
                String,
                String,
                i64,
                Option<String>,
                bool,
                bool,
                Option<String>,
            ),
        >(
            "SELECT provider_id, extension_id, runtime, observed_at_ms, instance_id, running, healthy, message
             FROM ai_provider_health_records
             ORDER BY observed_at_ms DESC, provider_id, runtime, instance_id",
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(
                |(
                    provider_id,
                    extension_id,
                    runtime,
                    observed_at_ms,
                    instance_id,
                    running,
                    healthy,
                    message,
                )| {
                    Ok(ProviderHealthSnapshot::new(
                        provider_id,
                        extension_id,
                        runtime,
                        u64::try_from(observed_at_ms)?,
                    )
                    .with_instance_id_option(instance_id)
                    .with_running(running)
                    .with_healthy(healthy)
                    .with_message_option(message))
                },
            )
            .collect()
    }

    pub async fn insert_usage_record(&self, record: &UsageRecord) -> Result<UsageRecord> {
        sqlx::query(
            "INSERT INTO ai_usage_records (
                project_id,
                model,
                provider_id,
                units,
                amount,
                input_tokens,
                output_tokens,
                total_tokens,
                api_key_hash,
                channel_id,
                latency_ms,
                reference_amount,
                created_at_ms
             )
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)",
        )
        .bind(&record.project_id)
        .bind(&record.model)
        .bind(&record.provider)
        .bind(i64::try_from(record.units)?)
        .bind(record.amount)
        .bind(i64::try_from(record.input_tokens)?)
        .bind(i64::try_from(record.output_tokens)?)
        .bind(i64::try_from(record.total_tokens)?)
        .bind(record.api_key_hash.as_deref())
        .bind(record.channel_id.as_deref())
        .bind(record.latency_ms.map(i64::try_from).transpose()?)
        .bind(record.reference_amount)
        .bind(i64::try_from(record.created_at_ms)?)
        .execute(&self.pool)
        .await?;
        Ok(record.clone())
    }

    pub async fn list_usage_records(&self) -> Result<Vec<UsageRecord>> {
        let rows = sqlx::query_as::<
            _,
            (
                String,
                String,
                String,
                i64,
                f64,
                i64,
                i64,
                i64,
                Option<String>,
                Option<String>,
                Option<i64>,
                Option<f64>,
                i64,
            ),
        >(
            "SELECT project_id, model, provider_id, units, amount, input_tokens, output_tokens, total_tokens, api_key_hash, channel_id, latency_ms, reference_amount, created_at_ms FROM ai_usage_records ORDER BY created_at_ms DESC, project_id, provider_id, model",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(
                |(
                    project_id,
                    model,
                    provider,
                    units,
                    amount,
                    input_tokens,
                    output_tokens,
                    total_tokens,
                    api_key_hash,
                    channel_id,
                    latency_ms,
                    reference_amount,
                    created_at_ms,
                )|
                 -> Result<UsageRecord> {
                    Ok(UsageRecord {
                        project_id,
                        model,
                        provider,
                        units: u64::try_from(units)?,
                        amount,
                        input_tokens: u64::try_from(input_tokens)?,
                        output_tokens: u64::try_from(output_tokens)?,
                        total_tokens: u64::try_from(total_tokens)?,
                        api_key_hash,
                        channel_id,
                        latency_ms: latency_ms.map(u64::try_from).transpose()?,
                        reference_amount,
                        created_at_ms: u64::try_from(created_at_ms)?,
                    })
                },
            )
            .collect::<Result<Vec<_>>>()?)
    }

    pub async fn list_usage_records_for_project(
        &self,
        project_id: &str,
    ) -> Result<Vec<UsageRecord>> {
        let rows = sqlx::query_as::<
            _,
            (
                String,
                String,
                String,
                i64,
                f64,
                i64,
                i64,
                i64,
                Option<String>,
                Option<String>,
                Option<i64>,
                Option<f64>,
                i64,
            ),
        >(
            "SELECT project_id, model, provider_id, units, amount, input_tokens, output_tokens, total_tokens, api_key_hash, channel_id, latency_ms, reference_amount, created_at_ms
             FROM ai_usage_records
             WHERE project_id = $1
             ORDER BY created_at_ms DESC, project_id, model",
        )
        .bind(project_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(
                |(
                    project_id,
                    model,
                    provider,
                    units,
                    amount,
                    input_tokens,
                    output_tokens,
                    total_tokens,
                    api_key_hash,
                    channel_id,
                    latency_ms,
                    reference_amount,
                    created_at_ms,
                )|
                 -> Result<UsageRecord> {
                    Ok(UsageRecord {
                        project_id,
                        model,
                        provider,
                        units: u64::try_from(units)?,
                        amount,
                        input_tokens: u64::try_from(input_tokens)?,
                        output_tokens: u64::try_from(output_tokens)?,
                        total_tokens: u64::try_from(total_tokens)?,
                        api_key_hash,
                        channel_id,
                        latency_ms: latency_ms.map(u64::try_from).transpose()?,
                        reference_amount,
                        created_at_ms: u64::try_from(created_at_ms)?,
                    })
                },
            )
            .collect::<Result<Vec<_>>>()?)
    }

    pub async fn find_latest_usage_record_for_project(
        &self,
        project_id: &str,
    ) -> Result<Option<UsageRecord>> {
        let row = sqlx::query_as::<
            _,
            (
                String,
                String,
                String,
                i64,
                f64,
                i64,
                i64,
                i64,
                Option<String>,
                Option<String>,
                Option<i64>,
                Option<f64>,
                i64,
            ),
        >(
            "SELECT project_id, model, provider_id, units, amount, input_tokens, output_tokens, total_tokens, api_key_hash, channel_id, latency_ms, reference_amount, created_at_ms
             FROM ai_usage_records
             WHERE project_id = $1
             ORDER BY created_at_ms DESC, project_id, provider_id, model
             LIMIT 1",
        )
        .bind(project_id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(
            |(
                project_id,
                model,
                provider,
                units,
                amount,
                input_tokens,
                output_tokens,
                total_tokens,
                api_key_hash,
                channel_id,
                latency_ms,
                reference_amount,
                created_at_ms,
            )| {
                Ok(UsageRecord {
                    project_id,
                    model,
                    provider,
                    units: u64::try_from(units)?,
                    amount,
                    input_tokens: u64::try_from(input_tokens)?,
                    output_tokens: u64::try_from(output_tokens)?,
                    total_tokens: u64::try_from(total_tokens)?,
                    api_key_hash,
                    channel_id,
                    latency_ms: latency_ms.map(u64::try_from).transpose()?,
                    reference_amount,
                    created_at_ms: u64::try_from(created_at_ms)?,
                })
            },
        )
        .transpose()
    }

    pub async fn insert_billing_event(
        &self,
        event: &BillingEventRecord,
    ) -> Result<BillingEventRecord> {
        sqlx::query(
            "INSERT INTO ai_billing_events (
                event_id,
                tenant_id,
                project_id,
                api_key_group_id,
                capability,
                route_key,
                usage_model,
                provider_id,
                accounting_mode,
                operation_kind,
                modality,
                api_key_hash,
                channel_id,
                reference_id,
                latency_ms,
                units,
                request_count,
                input_tokens,
                output_tokens,
                total_tokens,
                cache_read_tokens,
                cache_write_tokens,
                image_count,
                audio_seconds,
                video_seconds,
                music_seconds,
                upstream_cost,
                customer_charge,
                applied_routing_profile_id,
                compiled_routing_snapshot_id,
                fallback_reason,
                created_at_ms
             )
             VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16,
                $17, $18, $19, $20, $21, $22, $23, $24, $25, $26, $27, $28, $29, $30, $31, $32
             )
             ON CONFLICT(event_id) DO UPDATE SET
                tenant_id = excluded.tenant_id,
                project_id = excluded.project_id,
                api_key_group_id = excluded.api_key_group_id,
                capability = excluded.capability,
                route_key = excluded.route_key,
                usage_model = excluded.usage_model,
                provider_id = excluded.provider_id,
                accounting_mode = excluded.accounting_mode,
                operation_kind = excluded.operation_kind,
                modality = excluded.modality,
                api_key_hash = excluded.api_key_hash,
                channel_id = excluded.channel_id,
                reference_id = excluded.reference_id,
                latency_ms = excluded.latency_ms,
                units = excluded.units,
                request_count = excluded.request_count,
                input_tokens = excluded.input_tokens,
                output_tokens = excluded.output_tokens,
                total_tokens = excluded.total_tokens,
                cache_read_tokens = excluded.cache_read_tokens,
                cache_write_tokens = excluded.cache_write_tokens,
                image_count = excluded.image_count,
                audio_seconds = excluded.audio_seconds,
                video_seconds = excluded.video_seconds,
                music_seconds = excluded.music_seconds,
                upstream_cost = excluded.upstream_cost,
                customer_charge = excluded.customer_charge,
                applied_routing_profile_id = excluded.applied_routing_profile_id,
                compiled_routing_snapshot_id = excluded.compiled_routing_snapshot_id,
                fallback_reason = excluded.fallback_reason,
                created_at_ms = excluded.created_at_ms",
        )
        .bind(&event.event_id)
        .bind(&event.tenant_id)
        .bind(&event.project_id)
        .bind(event.api_key_group_id.as_deref())
        .bind(&event.capability)
        .bind(&event.route_key)
        .bind(&event.usage_model)
        .bind(&event.provider_id)
        .bind(event.accounting_mode.as_str())
        .bind(&event.operation_kind)
        .bind(&event.modality)
        .bind(event.api_key_hash.as_deref())
        .bind(event.channel_id.as_deref())
        .bind(event.reference_id.as_deref())
        .bind(event.latency_ms.map(i64::try_from).transpose()?)
        .bind(i64::try_from(event.units)?)
        .bind(i64::try_from(event.request_count)?)
        .bind(i64::try_from(event.input_tokens)?)
        .bind(i64::try_from(event.output_tokens)?)
        .bind(i64::try_from(event.total_tokens)?)
        .bind(i64::try_from(event.cache_read_tokens)?)
        .bind(i64::try_from(event.cache_write_tokens)?)
        .bind(i64::try_from(event.image_count)?)
        .bind(event.audio_seconds)
        .bind(event.video_seconds)
        .bind(event.music_seconds)
        .bind(event.upstream_cost)
        .bind(event.customer_charge)
        .bind(event.applied_routing_profile_id.as_deref())
        .bind(event.compiled_routing_snapshot_id.as_deref())
        .bind(event.fallback_reason.as_deref())
        .bind(i64::try_from(event.created_at_ms)?)
        .execute(&self.pool)
        .await?;
        Ok(event.clone())
    }

    pub async fn list_billing_events(&self) -> Result<Vec<BillingEventRecord>> {
        let rows = sqlx::query(
            "SELECT
                event_id,
                tenant_id,
                project_id,
                api_key_group_id,
                capability,
                route_key,
                usage_model,
                provider_id,
                accounting_mode,
                operation_kind,
                modality,
                api_key_hash,
                channel_id,
                reference_id,
                latency_ms,
                units,
                request_count,
                input_tokens,
                output_tokens,
                total_tokens,
                cache_read_tokens,
                cache_write_tokens,
                image_count,
                audio_seconds,
                video_seconds,
                music_seconds,
                upstream_cost,
                customer_charge,
                applied_routing_profile_id,
                compiled_routing_snapshot_id,
                fallback_reason,
                created_at_ms
             FROM ai_billing_events
             ORDER BY created_at_ms DESC, event_id",
        )
        .fetch_all(&self.pool)
        .await?;

        rows.iter()
            .map(decode_billing_event_row)
            .collect::<Result<Vec<_>>>()
    }

    pub async fn insert_ledger_entry(&self, entry: &LedgerEntry) -> Result<LedgerEntry> {
        sqlx::query(
            "INSERT INTO ai_billing_ledger_entries (project_id, units, amount, created_at_ms) VALUES ($1, $2, $3, $4)",
        )
        .bind(&entry.project_id)
        .bind(i64::try_from(entry.units)?)
        .bind(entry.amount)
        .bind(current_timestamp_ms())
        .execute(&self.pool)
        .await?;
        Ok(entry.clone())
    }

    pub async fn list_ledger_entries(&self) -> Result<Vec<LedgerEntry>> {
        let rows = sqlx::query_as::<_, (String, i64, f64)>(
            "SELECT project_id, units, amount FROM ai_billing_ledger_entries ORDER BY created_at_ms DESC, project_id",
        )
        .fetch_all(&self.pool)
        .await?;
        let entries = rows
            .into_iter()
            .map(|(project_id, units, amount)| {
                Ok(LedgerEntry {
                    project_id,
                    units: u64::try_from(units)?,
                    amount,
                })
            })
            .collect::<std::result::Result<Vec<_>, std::num::TryFromIntError>>()?;
        Ok(entries)
    }

    pub async fn list_ledger_entries_for_project(
        &self,
        project_id: &str,
    ) -> Result<Vec<LedgerEntry>> {
        let rows = sqlx::query_as::<_, (String, i64, f64)>(
            "SELECT project_id, units, amount
             FROM ai_billing_ledger_entries
             WHERE project_id = $1
             ORDER BY created_at_ms DESC, project_id",
        )
        .bind(project_id)
        .fetch_all(&self.pool)
        .await?;
        let entries = rows
            .into_iter()
            .map(|(project_id, units, amount)| {
                Ok(LedgerEntry {
                    project_id,
                    units: u64::try_from(units)?,
                    amount,
                })
            })
            .collect::<std::result::Result<Vec<_>, std::num::TryFromIntError>>()?;
        Ok(entries)
    }

    pub async fn insert_quota_policy(&self, policy: &QuotaPolicy) -> Result<QuotaPolicy> {
        sqlx::query(
            "INSERT INTO ai_billing_quota_policies (policy_id, project_id, max_units, enabled)
             VALUES ($1, $2, $3, $4)
             ON CONFLICT(policy_id) DO UPDATE SET
             project_id = excluded.project_id,
             max_units = excluded.max_units,
             enabled = excluded.enabled",
        )
        .bind(&policy.policy_id)
        .bind(&policy.project_id)
        .bind(i64::try_from(policy.max_units)?)
        .bind(policy.enabled)
        .execute(&self.pool)
        .await?;
        Ok(policy.clone())
    }

    pub async fn list_quota_policies(&self) -> Result<Vec<QuotaPolicy>> {
        let rows = sqlx::query_as::<_, (String, String, i64, bool)>(
            "SELECT policy_id, project_id, max_units, enabled
             FROM ai_billing_quota_policies
             ORDER BY policy_id",
        )
        .fetch_all(&self.pool)
        .await?;

        let policies = rows
            .into_iter()
            .map(|(policy_id, project_id, max_units, enabled)| {
                Ok(QuotaPolicy {
                    policy_id,
                    project_id,
                    max_units: u64::try_from(max_units)?,
                    enabled,
                })
            })
            .collect::<std::result::Result<Vec<_>, std::num::TryFromIntError>>()?;
        Ok(policies)
    }

    pub async fn list_quota_policies_for_project(
        &self,
        project_id: &str,
    ) -> Result<Vec<QuotaPolicy>> {
        let rows = sqlx::query_as::<_, (String, String, i64, bool)>(
            "SELECT policy_id, project_id, max_units, enabled
             FROM ai_billing_quota_policies
             WHERE project_id = $1
             ORDER BY policy_id",
        )
        .bind(project_id)
        .fetch_all(&self.pool)
        .await?;
        let policies = rows
            .into_iter()
            .map(|(policy_id, project_id, max_units, enabled)| {
                Ok(QuotaPolicy {
                    policy_id,
                    project_id,
                    max_units: u64::try_from(max_units)?,
                    enabled,
                })
            })
            .collect::<std::result::Result<Vec<_>, std::num::TryFromIntError>>()?;
        Ok(policies)
    }

    pub async fn insert_rate_limit_policy(
        &self,
        policy: &RateLimitPolicy,
    ) -> Result<RateLimitPolicy> {
        sqlx::query(
            "INSERT INTO ai_gateway_rate_limit_policies (
                policy_id, project_id, api_key_hash, route_key, model_name,
                requests_per_window, window_seconds, burst_requests, enabled,
                notes, created_at_ms, updated_at_ms
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
             ON CONFLICT(policy_id) DO UPDATE SET
             project_id = excluded.project_id,
             api_key_hash = excluded.api_key_hash,
             route_key = excluded.route_key,
             model_name = excluded.model_name,
             requests_per_window = excluded.requests_per_window,
             window_seconds = excluded.window_seconds,
             burst_requests = excluded.burst_requests,
             enabled = excluded.enabled,
             notes = excluded.notes,
             created_at_ms = excluded.created_at_ms,
             updated_at_ms = excluded.updated_at_ms",
        )
        .bind(&policy.policy_id)
        .bind(&policy.project_id)
        .bind(&policy.api_key_hash)
        .bind(&policy.route_key)
        .bind(&policy.model_name)
        .bind(i64::try_from(policy.requests_per_window)?)
        .bind(i64::try_from(policy.window_seconds)?)
        .bind(i64::try_from(policy.burst_requests)?)
        .bind(policy.enabled)
        .bind(&policy.notes)
        .bind(i64::try_from(policy.created_at_ms)?)
        .bind(i64::try_from(policy.updated_at_ms)?)
        .execute(&self.pool)
        .await?;
        Ok(policy.clone())
    }

    pub async fn list_rate_limit_policies(&self) -> Result<Vec<RateLimitPolicy>> {
        let rows = sqlx::query_as::<_, (
            String,
            String,
            Option<String>,
            Option<String>,
            Option<String>,
            i64,
            i64,
            i64,
            bool,
            Option<String>,
            i64,
            i64,
        )>(
            "SELECT policy_id, project_id, api_key_hash, route_key, model_name, requests_per_window, window_seconds, burst_requests, enabled, notes, created_at_ms, updated_at_ms
             FROM ai_gateway_rate_limit_policies
             ORDER BY project_id, enabled DESC, policy_id",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(
                |(
                    policy_id,
                    project_id,
                    api_key_hash,
                    route_key,
                    model_name,
                    requests_per_window,
                    window_seconds,
                    burst_requests,
                    enabled,
                    notes,
                    created_at_ms,
                    updated_at_ms,
                )| {
                    Ok(RateLimitPolicy {
                        policy_id,
                        project_id,
                        api_key_hash,
                        route_key,
                        model_name,
                        requests_per_window: u64::try_from(requests_per_window)?,
                        window_seconds: u64::try_from(window_seconds)?,
                        burst_requests: u64::try_from(burst_requests)?,
                        enabled,
                        notes,
                        created_at_ms: u64::try_from(created_at_ms)?,
                        updated_at_ms: u64::try_from(updated_at_ms)?,
                    })
                },
            )
            .collect::<std::result::Result<Vec<_>, std::num::TryFromIntError>>()?)
    }

    pub async fn list_rate_limit_policies_for_project(
        &self,
        project_id: &str,
    ) -> Result<Vec<RateLimitPolicy>> {
        let rows = sqlx::query_as::<_, (
            String,
            String,
            Option<String>,
            Option<String>,
            Option<String>,
            i64,
            i64,
            i64,
            bool,
            Option<String>,
            i64,
            i64,
        )>(
            "SELECT policy_id, project_id, api_key_hash, route_key, model_name, requests_per_window, window_seconds, burst_requests, enabled, notes, created_at_ms, updated_at_ms
             FROM ai_gateway_rate_limit_policies
             WHERE project_id = $1
             ORDER BY enabled DESC, policy_id",
        )
        .bind(project_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(
                |(
                    policy_id,
                    project_id,
                    api_key_hash,
                    route_key,
                    model_name,
                    requests_per_window,
                    window_seconds,
                    burst_requests,
                    enabled,
                    notes,
                    created_at_ms,
                    updated_at_ms,
                )| {
                    Ok(RateLimitPolicy {
                        policy_id,
                        project_id,
                        api_key_hash,
                        route_key,
                        model_name,
                        requests_per_window: u64::try_from(requests_per_window)?,
                        window_seconds: u64::try_from(window_seconds)?,
                        burst_requests: u64::try_from(burst_requests)?,
                        enabled,
                        notes,
                        created_at_ms: u64::try_from(created_at_ms)?,
                        updated_at_ms: u64::try_from(updated_at_ms)?,
                    })
                },
            )
            .collect::<std::result::Result<Vec<_>, std::num::TryFromIntError>>()?)
    }

    pub async fn list_rate_limit_window_snapshots(&self) -> Result<Vec<RateLimitWindowSnapshot>> {
        let rows = sqlx::query_as::<
            _,
            (
                String,
                String,
                Option<String>,
                Option<String>,
                Option<String>,
                i64,
                i64,
                i64,
                i64,
                i64,
                i64,
                bool,
            ),
        >(
            "SELECT
                p.policy_id,
                p.project_id,
                p.api_key_hash,
                p.route_key,
                p.model_name,
                p.requests_per_window,
                p.window_seconds,
                p.burst_requests,
                w.request_count,
                w.window_start_ms,
                w.updated_at_ms,
                p.enabled
             FROM ai_gateway_rate_limit_windows w
             INNER JOIN ai_gateway_rate_limit_policies p ON p.policy_id = w.policy_id
             ORDER BY p.project_id, w.updated_at_ms DESC, p.policy_id, w.window_start_ms DESC",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(
                |(
                    policy_id,
                    project_id,
                    api_key_hash,
                    route_key,
                    model_name,
                    requests_per_window,
                    window_seconds,
                    burst_requests,
                    request_count,
                    window_start_ms,
                    updated_at_ms,
                    enabled,
                )| {
                    let requests_per_window = u64::try_from(requests_per_window)?;
                    let window_seconds = u64::try_from(window_seconds)?;
                    let burst_requests = u64::try_from(burst_requests)?;
                    let request_count = u64::try_from(request_count)?;
                    let window_start_ms = u64::try_from(window_start_ms)?;
                    let updated_at_ms = u64::try_from(updated_at_ms)?;
                    let limit_requests = match burst_requests {
                        0 => requests_per_window,
                        burst => burst.max(requests_per_window),
                    };
                    let remaining_requests = limit_requests.saturating_sub(request_count);

                    Ok(RateLimitWindowSnapshot {
                        policy_id,
                        project_id,
                        api_key_hash,
                        route_key,
                        model_name,
                        requests_per_window,
                        window_seconds,
                        burst_requests,
                        limit_requests,
                        request_count,
                        remaining_requests,
                        window_start_ms,
                        window_end_ms: window_start_ms
                            .saturating_add(window_seconds.saturating_mul(1000)),
                        updated_at_ms,
                        enabled,
                        exceeded: request_count > limit_requests,
                    })
                },
            )
            .collect::<std::result::Result<Vec<_>, std::num::TryFromIntError>>()?)
    }

    pub async fn check_and_consume_rate_limit(
        &self,
        policy_id: &str,
        requested_requests: u64,
        limit_requests: u64,
        window_seconds: u64,
        now_ms: u64,
    ) -> Result<RateLimitCheckResult> {
        let window_seconds = window_seconds.max(1);
        let window_ms = window_seconds.saturating_mul(1000);
        let window_start_ms = now_ms - (now_ms % window_ms);
        let requested = i64::try_from(requested_requests)?;
        let limit = i64::try_from(limit_requests)?;
        let window_start = i64::try_from(window_start_ms)?;
        let now = i64::try_from(now_ms)?;

        let mut tx = self.pool.begin().await?;

        sqlx::query(
            "INSERT INTO ai_gateway_rate_limit_windows (policy_id, window_start_ms, request_count, updated_at_ms)
             VALUES ($1, $2, 0, $3)
             ON CONFLICT(policy_id, window_start_ms) DO NOTHING",
        )
        .bind(policy_id)
        .bind(window_start)
        .bind(now)
        .execute(&mut *tx)
        .await?;

        loop {
            let used_before = sqlx::query_as::<_, (i64,)>(
                "SELECT request_count
                 FROM ai_gateway_rate_limit_windows
                 WHERE policy_id = $1 AND window_start_ms = $2",
            )
            .bind(policy_id)
            .bind(window_start)
            .fetch_one(&mut *tx)
            .await?
            .0;

            if used_before.saturating_add(requested) > limit {
                tx.rollback().await?;
                return Ok(RateLimitCheckResult {
                    allowed: false,
                    policy_id: Some(policy_id.to_owned()),
                    requested_requests,
                    used_requests: u64::try_from(used_before)?,
                    limit_requests: Some(limit_requests),
                    remaining_requests: Some(
                        limit_requests.saturating_sub(u64::try_from(used_before)?),
                    ),
                    window_seconds: Some(window_seconds),
                    window_start_ms: Some(window_start_ms),
                    window_end_ms: Some(window_start_ms.saturating_add(window_ms)),
                });
            }

            let updated = sqlx::query(
                "UPDATE ai_gateway_rate_limit_windows
                 SET request_count = request_count + $1, updated_at_ms = $2
                 WHERE policy_id = $3 AND window_start_ms = $4 AND request_count = $5",
            )
            .bind(requested)
            .bind(now)
            .bind(policy_id)
            .bind(window_start)
            .bind(used_before)
            .execute(&mut *tx)
            .await?;

            if updated.rows_affected() == 1 {
                tx.commit().await?;
                return Ok(RateLimitCheckResult {
                    allowed: true,
                    policy_id: Some(policy_id.to_owned()),
                    requested_requests,
                    used_requests: u64::try_from(used_before)?,
                    limit_requests: Some(limit_requests),
                    remaining_requests: Some(limit_requests.saturating_sub(
                        u64::try_from(used_before)?.saturating_add(requested_requests),
                    )),
                    window_seconds: Some(window_seconds),
                    window_start_ms: Some(window_start_ms),
                    window_end_ms: Some(window_start_ms.saturating_add(window_ms)),
                });
            }
        }
    }

    pub async fn insert_tenant(&self, tenant: &Tenant) -> Result<Tenant> {
        sqlx::query(
            "INSERT INTO ai_tenants (id, name) VALUES ($1, $2)
             ON CONFLICT(id) DO UPDATE SET name = excluded.name",
        )
        .bind(&tenant.id)
        .bind(&tenant.name)
        .execute(&self.pool)
        .await?;
        Ok(tenant.clone())
    }

    pub async fn list_tenants(&self) -> Result<Vec<Tenant>> {
        let rows =
            sqlx::query_as::<_, (String, String)>("SELECT id, name FROM ai_tenants ORDER BY id")
                .fetch_all(&self.pool)
                .await?;
        Ok(rows
            .into_iter()
            .map(|(id, name)| Tenant { id, name })
            .collect())
    }

    pub async fn find_tenant(&self, tenant_id: &str) -> Result<Option<Tenant>> {
        let row =
            sqlx::query_as::<_, (String, String)>("SELECT id, name FROM ai_tenants WHERE id = $1")
                .bind(tenant_id)
                .fetch_optional(&self.pool)
                .await?;
        Ok(row.map(|(id, name)| Tenant { id, name }))
    }

    pub async fn delete_tenant(&self, tenant_id: &str) -> Result<bool> {
        sqlx::query("DELETE FROM ai_router_credential_records WHERE tenant_id = $1")
            .bind(tenant_id)
            .execute(&self.pool)
            .await?;
        let result = sqlx::query("DELETE FROM ai_tenants WHERE id = $1")
            .bind(tenant_id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn insert_project(&self, project: &Project) -> Result<Project> {
        sqlx::query(
            "INSERT INTO ai_projects (id, tenant_id, name) VALUES ($1, $2, $3)
             ON CONFLICT(id) DO UPDATE SET tenant_id = excluded.tenant_id, name = excluded.name",
        )
        .bind(&project.id)
        .bind(&project.tenant_id)
        .bind(&project.name)
        .execute(&self.pool)
        .await?;
        Ok(project.clone())
    }

    pub async fn list_projects(&self) -> Result<Vec<Project>> {
        let rows = sqlx::query_as::<_, (String, String, String)>(
            "SELECT tenant_id, id, name FROM ai_projects ORDER BY tenant_id, id",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|(tenant_id, id, name)| Project {
                tenant_id,
                id,
                name,
            })
            .collect())
    }

    pub async fn find_project(&self, project_id: &str) -> Result<Option<Project>> {
        let row = sqlx::query_as::<_, (String, String, String)>(
            "SELECT tenant_id, id, name FROM ai_projects WHERE id = $1",
        )
        .bind(project_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|(tenant_id, id, name)| Project {
            tenant_id,
            id,
            name,
        }))
    }

    pub async fn delete_project(&self, project_id: &str) -> Result<bool> {
        sqlx::query("DELETE FROM ai_app_api_keys WHERE project_id = $1")
            .bind(project_id)
            .execute(&self.pool)
            .await?;
        sqlx::query("DELETE FROM ai_billing_quota_policies WHERE project_id = $1")
            .bind(project_id)
            .execute(&self.pool)
            .await?;
        let result = sqlx::query("DELETE FROM ai_projects WHERE id = $1")
            .bind(project_id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn insert_coupon(&self, coupon: &CouponCampaign) -> Result<CouponCampaign> {
        sqlx::query(
            "INSERT INTO ai_coupon_campaigns (id, code, discount_label, audience, remaining, active, note, expires_on, created_at_ms)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
             ON CONFLICT(id) DO UPDATE SET
             code = excluded.code,
             discount_label = excluded.discount_label,
             audience = excluded.audience,
             remaining = excluded.remaining,
             active = excluded.active,
             note = excluded.note,
             expires_on = excluded.expires_on,
             created_at_ms = excluded.created_at_ms",
        )
        .bind(&coupon.id)
        .bind(&coupon.code)
        .bind(&coupon.discount_label)
        .bind(&coupon.audience)
        .bind(i64::try_from(coupon.remaining)?)
        .bind(coupon.active)
        .bind(&coupon.note)
        .bind(&coupon.expires_on)
        .bind(i64::try_from(coupon.created_at_ms)?)
        .execute(&self.pool)
        .await?;
        Ok(coupon.clone())
    }

    pub async fn list_coupons(&self) -> Result<Vec<CouponCampaign>> {
        let rows = sqlx::query_as::<_, CouponRow>(
            "SELECT id, code, discount_label, audience, remaining, active, note, expires_on, created_at_ms
             FROM ai_coupon_campaigns
             ORDER BY active DESC, created_at_ms DESC, code ASC",
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|row| decode_coupon_row(Some(row)))
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .map(|row| row.ok_or_else(|| anyhow::anyhow!("coupon row decode returned empty")))
            .collect()
    }

    pub async fn list_active_coupons(&self) -> Result<Vec<CouponCampaign>> {
        let rows = sqlx::query_as::<_, CouponRow>(
            "SELECT id, code, discount_label, audience, remaining, active, note, expires_on, created_at_ms
             FROM ai_coupon_campaigns
             WHERE active = TRUE AND remaining > 0
             ORDER BY remaining DESC, created_at_ms DESC, code ASC",
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|row| decode_coupon_row(Some(row)))
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .map(|row| row.ok_or_else(|| anyhow::anyhow!("coupon row decode returned empty")))
            .collect()
    }

    pub async fn find_coupon(&self, coupon_id: &str) -> Result<Option<CouponCampaign>> {
        let row = sqlx::query_as::<_, CouponRow>(
            "SELECT id, code, discount_label, audience, remaining, active, note, expires_on, created_at_ms
             FROM ai_coupon_campaigns
             WHERE id = $1",
        )
        .bind(coupon_id)
        .fetch_optional(&self.pool)
        .await?;
        decode_coupon_row(row)
    }

    pub async fn delete_coupon(&self, coupon_id: &str) -> Result<bool> {
        let result = sqlx::query("DELETE FROM ai_coupon_campaigns WHERE id = $1")
            .bind(coupon_id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn insert_coupon_template_record(
        &self,
        record: &CouponTemplateRecord,
    ) -> Result<CouponTemplateRecord> {
        let record_json = encode_json_record(record)?;
        sqlx::query(
            "INSERT INTO ai_marketing_coupon_template (
                coupon_template_id, tenant_id, organization_id, template_code, status,
                benefit_kind, distribution_kind, exclusive_group, created_at_ms, updated_at_ms,
                record_json
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, CAST($11 AS JSONB))
             ON CONFLICT(coupon_template_id) DO UPDATE SET
                tenant_id = excluded.tenant_id,
                organization_id = excluded.organization_id,
                template_code = excluded.template_code,
                status = excluded.status,
                benefit_kind = excluded.benefit_kind,
                distribution_kind = excluded.distribution_kind,
                exclusive_group = excluded.exclusive_group,
                created_at_ms = excluded.created_at_ms,
                updated_at_ms = excluded.updated_at_ms,
                record_json = excluded.record_json",
        )
        .bind(i64::try_from(record.coupon_template_id)?)
        .bind(i64::try_from(record.tenant_id)?)
        .bind(i64::try_from(record.organization_id)?)
        .bind(&record.template_code)
        .bind(format!("{:?}", record.status).to_ascii_lowercase())
        .bind(format!("{:?}", record.benefit_kind).to_ascii_lowercase())
        .bind(format!("{:?}", record.distribution_kind).to_ascii_lowercase())
        .bind(&record.exclusive_group)
        .bind(i64::try_from(record.created_at_ms)?)
        .bind(i64::try_from(record.updated_at_ms)?)
        .bind(record_json)
        .execute(&self.pool)
        .await?;
        Ok(record.clone())
    }

    pub async fn list_coupon_template_records(&self) -> Result<Vec<CouponTemplateRecord>> {
        let rows = sqlx::query_scalar::<_, String>(
            "SELECT record_json::text
             FROM ai_marketing_coupon_template
             ORDER BY updated_at_ms DESC, coupon_template_id",
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(decode_json_record).collect()
    }

    pub async fn find_coupon_template_record(
        &self,
        coupon_template_id: u64,
    ) -> Result<Option<CouponTemplateRecord>> {
        let row = sqlx::query_scalar::<_, String>(
            "SELECT record_json::text
             FROM ai_marketing_coupon_template
             WHERE coupon_template_id = $1",
        )
        .bind(i64::try_from(coupon_template_id)?)
        .fetch_optional(&self.pool)
        .await?;
        row.map(decode_json_record).transpose()
    }

    pub async fn insert_coupon_benefit_rule_record(
        &self,
        record: &CouponBenefitRuleRecord,
    ) -> Result<CouponBenefitRuleRecord> {
        let record_json = encode_json_record(record)?;
        sqlx::query(
            "INSERT INTO ai_marketing_coupon_benefit_rule (
                coupon_benefit_rule_id, tenant_id, organization_id, coupon_template_id,
                benefit_kind, created_at_ms, updated_at_ms, record_json
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, CAST($8 AS JSONB))
             ON CONFLICT(coupon_benefit_rule_id) DO UPDATE SET
                tenant_id = excluded.tenant_id,
                organization_id = excluded.organization_id,
                coupon_template_id = excluded.coupon_template_id,
                benefit_kind = excluded.benefit_kind,
                created_at_ms = excluded.created_at_ms,
                updated_at_ms = excluded.updated_at_ms,
                record_json = excluded.record_json",
        )
        .bind(i64::try_from(record.coupon_benefit_rule_id)?)
        .bind(i64::try_from(record.tenant_id)?)
        .bind(i64::try_from(record.organization_id)?)
        .bind(i64::try_from(record.coupon_template_id)?)
        .bind(format!("{:?}", record.benefit_kind).to_ascii_lowercase())
        .bind(i64::try_from(record.created_at_ms)?)
        .bind(i64::try_from(record.updated_at_ms)?)
        .bind(record_json)
        .execute(&self.pool)
        .await?;
        Ok(record.clone())
    }

    pub async fn list_coupon_benefit_rule_records(&self) -> Result<Vec<CouponBenefitRuleRecord>> {
        let rows = sqlx::query_scalar::<_, String>(
            "SELECT record_json::text
             FROM ai_marketing_coupon_benefit_rule
             ORDER BY updated_at_ms DESC, coupon_benefit_rule_id",
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(decode_json_record).collect()
    }

    pub async fn insert_marketing_campaign_record(
        &self,
        record: &MarketingCampaignRecord,
    ) -> Result<MarketingCampaignRecord> {
        let record_json = encode_json_record(record)?;
        sqlx::query(
            "INSERT INTO ai_marketing_campaign (
                marketing_campaign_id, tenant_id, organization_id, campaign_code, status,
                campaign_kind, owner_user_id, created_at_ms, updated_at_ms, record_json
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, CAST($10 AS JSONB))
             ON CONFLICT(marketing_campaign_id) DO UPDATE SET
                tenant_id = excluded.tenant_id,
                organization_id = excluded.organization_id,
                campaign_code = excluded.campaign_code,
                status = excluded.status,
                campaign_kind = excluded.campaign_kind,
                owner_user_id = excluded.owner_user_id,
                created_at_ms = excluded.created_at_ms,
                updated_at_ms = excluded.updated_at_ms,
                record_json = excluded.record_json",
        )
        .bind(i64::try_from(record.marketing_campaign_id)?)
        .bind(i64::try_from(record.tenant_id)?)
        .bind(i64::try_from(record.organization_id)?)
        .bind(&record.campaign_code)
        .bind(format!("{:?}", record.status).to_ascii_lowercase())
        .bind(format!("{:?}", record.campaign_kind).to_ascii_lowercase())
        .bind(record.owner_user_id.map(i64::try_from).transpose()?)
        .bind(i64::try_from(record.created_at_ms)?)
        .bind(i64::try_from(record.updated_at_ms)?)
        .bind(record_json)
        .execute(&self.pool)
        .await?;
        Ok(record.clone())
    }

    pub async fn list_marketing_campaign_records(&self) -> Result<Vec<MarketingCampaignRecord>> {
        let rows = sqlx::query_scalar::<_, String>(
            "SELECT record_json::text
             FROM ai_marketing_campaign
             ORDER BY updated_at_ms DESC, marketing_campaign_id",
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(decode_json_record).collect()
    }

    pub async fn find_marketing_campaign_record(
        &self,
        marketing_campaign_id: u64,
    ) -> Result<Option<MarketingCampaignRecord>> {
        let row = sqlx::query_scalar::<_, String>(
            "SELECT record_json::text
             FROM ai_marketing_campaign
             WHERE marketing_campaign_id = $1",
        )
        .bind(i64::try_from(marketing_campaign_id)?)
        .fetch_optional(&self.pool)
        .await?;
        row.map(decode_json_record).transpose()
    }

    pub async fn insert_coupon_code_batch_record(
        &self,
        record: &CouponCodeBatchRecord,
    ) -> Result<CouponCodeBatchRecord> {
        let record_json = encode_json_record(record)?;
        sqlx::query(
            "INSERT INTO ai_marketing_coupon_code_batch (
                coupon_code_batch_id, tenant_id, organization_id, coupon_template_id,
                marketing_campaign_id, generation_mode, status, code_prefix, expires_at_ms,
                created_at_ms, updated_at_ms, record_json
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, CAST($12 AS JSONB))
             ON CONFLICT(coupon_code_batch_id) DO UPDATE SET
                tenant_id = excluded.tenant_id,
                organization_id = excluded.organization_id,
                coupon_template_id = excluded.coupon_template_id,
                marketing_campaign_id = excluded.marketing_campaign_id,
                generation_mode = excluded.generation_mode,
                status = excluded.status,
                code_prefix = excluded.code_prefix,
                expires_at_ms = excluded.expires_at_ms,
                created_at_ms = excluded.created_at_ms,
                updated_at_ms = excluded.updated_at_ms,
                record_json = excluded.record_json",
        )
        .bind(i64::try_from(record.coupon_code_batch_id)?)
        .bind(i64::try_from(record.tenant_id)?)
        .bind(i64::try_from(record.organization_id)?)
        .bind(i64::try_from(record.coupon_template_id)?)
        .bind(
            record
                .marketing_campaign_id
                .map(i64::try_from)
                .transpose()?,
        )
        .bind(format!("{:?}", record.generation_mode).to_ascii_lowercase())
        .bind(format!("{:?}", record.status).to_ascii_lowercase())
        .bind(&record.code_prefix)
        .bind(record.expires_at_ms.map(i64::try_from).transpose()?)
        .bind(i64::try_from(record.created_at_ms)?)
        .bind(i64::try_from(record.updated_at_ms)?)
        .bind(record_json)
        .execute(&self.pool)
        .await?;
        Ok(record.clone())
    }

    pub async fn list_coupon_code_batch_records(&self) -> Result<Vec<CouponCodeBatchRecord>> {
        let rows = sqlx::query_scalar::<_, String>(
            "SELECT record_json::text
             FROM ai_marketing_coupon_code_batch
             ORDER BY updated_at_ms DESC, coupon_code_batch_id",
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(decode_json_record).collect()
    }

    pub async fn insert_coupon_code_record(
        &self,
        record: &CouponCodeRecord,
    ) -> Result<CouponCodeRecord> {
        let record_json = encode_json_record(record)?;
        sqlx::query(
            "INSERT INTO ai_marketing_coupon_code (
                coupon_code_id, tenant_id, organization_id, coupon_code_batch_id,
                coupon_template_id, marketing_campaign_id, code_lookup_hash, code_kind, status,
                display_code_prefix, display_code_suffix, claim_subject_type, claim_subject_id,
                expires_at_ms, created_at_ms, updated_at_ms, record_json
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, CAST($17 AS JSONB))
             ON CONFLICT(coupon_code_id) DO UPDATE SET
                tenant_id = excluded.tenant_id,
                organization_id = excluded.organization_id,
                coupon_code_batch_id = excluded.coupon_code_batch_id,
                coupon_template_id = excluded.coupon_template_id,
                marketing_campaign_id = excluded.marketing_campaign_id,
                code_lookup_hash = excluded.code_lookup_hash,
                code_kind = excluded.code_kind,
                status = excluded.status,
                display_code_prefix = excluded.display_code_prefix,
                display_code_suffix = excluded.display_code_suffix,
                claim_subject_type = excluded.claim_subject_type,
                claim_subject_id = excluded.claim_subject_id,
                expires_at_ms = excluded.expires_at_ms,
                created_at_ms = excluded.created_at_ms,
                updated_at_ms = excluded.updated_at_ms,
                record_json = excluded.record_json",
        )
        .bind(i64::try_from(record.coupon_code_id)?)
        .bind(i64::try_from(record.tenant_id)?)
        .bind(i64::try_from(record.organization_id)?)
        .bind(i64::try_from(record.coupon_code_batch_id)?)
        .bind(i64::try_from(record.coupon_template_id)?)
        .bind(record.marketing_campaign_id.map(i64::try_from).transpose()?)
        .bind(&record.code_lookup_hash)
        .bind(format!("{:?}", record.code_kind).to_ascii_lowercase())
        .bind(format!("{:?}", record.status).to_ascii_lowercase())
        .bind(&record.display_code_prefix)
        .bind(&record.display_code_suffix)
        .bind(&record.claim_subject_type)
        .bind(&record.claim_subject_id)
        .bind(record.expires_at_ms.map(i64::try_from).transpose()?)
        .bind(i64::try_from(record.created_at_ms)?)
        .bind(i64::try_from(record.updated_at_ms)?)
        .bind(record_json)
        .execute(&self.pool)
        .await?;
        Ok(record.clone())
    }

    pub async fn list_coupon_code_records(&self) -> Result<Vec<CouponCodeRecord>> {
        let rows = sqlx::query_scalar::<_, String>(
            "SELECT record_json::text
             FROM ai_marketing_coupon_code
             ORDER BY updated_at_ms DESC, coupon_code_id",
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(decode_json_record).collect()
    }

    pub async fn find_coupon_code_record(
        &self,
        coupon_code_id: u64,
    ) -> Result<Option<CouponCodeRecord>> {
        let row = sqlx::query_scalar::<_, String>(
            "SELECT record_json::text
             FROM ai_marketing_coupon_code
             WHERE coupon_code_id = $1",
        )
        .bind(i64::try_from(coupon_code_id)?)
        .fetch_optional(&self.pool)
        .await?;
        row.map(decode_json_record).transpose()
    }

    pub async fn find_coupon_code_record_by_lookup_hash(
        &self,
        code_lookup_hash: &str,
    ) -> Result<Option<CouponCodeRecord>> {
        let row = sqlx::query_scalar::<_, String>(
            "SELECT record_json::text
             FROM ai_marketing_coupon_code
             WHERE code_lookup_hash = $1",
        )
        .bind(code_lookup_hash)
        .fetch_optional(&self.pool)
        .await?;
        row.map(decode_json_record).transpose()
    }

    pub async fn list_coupon_code_records_for_subject(
        &self,
        claim_subject_type: &str,
        claim_subject_id: &str,
    ) -> Result<Vec<CouponCodeRecord>> {
        let rows = sqlx::query_scalar::<_, String>(
            "SELECT record_json::text
             FROM ai_marketing_coupon_code
             WHERE claim_subject_type = $1 AND claim_subject_id = $2
             ORDER BY updated_at_ms DESC, coupon_code_id",
        )
        .bind(claim_subject_type)
        .bind(claim_subject_id)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(decode_json_record).collect()
    }

    pub async fn insert_coupon_claim_record(
        &self,
        record: &CouponClaimRecord,
    ) -> Result<CouponClaimRecord> {
        let record_json = encode_json_record(record)?;
        sqlx::query(
            "INSERT INTO ai_marketing_coupon_claim (
                coupon_claim_id, tenant_id, organization_id, coupon_code_id, coupon_template_id,
                subject_type, subject_id, status, account_id, project_id, expires_at_ms,
                created_at_ms, updated_at_ms, record_json
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, CAST($14 AS JSONB))
             ON CONFLICT(coupon_claim_id) DO UPDATE SET
                tenant_id = excluded.tenant_id,
                organization_id = excluded.organization_id,
                coupon_code_id = excluded.coupon_code_id,
                coupon_template_id = excluded.coupon_template_id,
                subject_type = excluded.subject_type,
                subject_id = excluded.subject_id,
                status = excluded.status,
                account_id = excluded.account_id,
                project_id = excluded.project_id,
                expires_at_ms = excluded.expires_at_ms,
                created_at_ms = excluded.created_at_ms,
                updated_at_ms = excluded.updated_at_ms,
                record_json = excluded.record_json",
        )
        .bind(i64::try_from(record.coupon_claim_id)?)
        .bind(i64::try_from(record.tenant_id)?)
        .bind(i64::try_from(record.organization_id)?)
        .bind(i64::try_from(record.coupon_code_id)?)
        .bind(i64::try_from(record.coupon_template_id)?)
        .bind(&record.subject_type)
        .bind(&record.subject_id)
        .bind(format!("{:?}", record.status).to_ascii_lowercase())
        .bind(record.account_id.map(i64::try_from).transpose()?)
        .bind(&record.project_id)
        .bind(record.expires_at_ms.map(i64::try_from).transpose()?)
        .bind(i64::try_from(record.created_at_ms)?)
        .bind(i64::try_from(record.updated_at_ms)?)
        .bind(record_json)
        .execute(&self.pool)
        .await?;
        Ok(record.clone())
    }

    pub async fn list_coupon_claim_records(&self) -> Result<Vec<CouponClaimRecord>> {
        let rows = sqlx::query_scalar::<_, String>(
            "SELECT record_json::text
             FROM ai_marketing_coupon_claim
             ORDER BY updated_at_ms DESC, coupon_claim_id",
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(decode_json_record).collect()
    }

    pub async fn insert_coupon_redemption_record(
        &self,
        record: &CouponRedemptionRecord,
    ) -> Result<CouponRedemptionRecord> {
        let record_json = encode_json_record(record)?;
        sqlx::query(
            "INSERT INTO ai_marketing_coupon_redemption (
                coupon_redemption_id, tenant_id, organization_id, coupon_code_id,
                coupon_template_id, marketing_campaign_id, subject_type, subject_id, status,
                idempotency_key, created_at_ms, updated_at_ms, record_json
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, CAST($13 AS JSONB))
             ON CONFLICT(coupon_redemption_id) DO UPDATE SET
                tenant_id = excluded.tenant_id,
                organization_id = excluded.organization_id,
                coupon_code_id = excluded.coupon_code_id,
                coupon_template_id = excluded.coupon_template_id,
                marketing_campaign_id = excluded.marketing_campaign_id,
                subject_type = excluded.subject_type,
                subject_id = excluded.subject_id,
                status = excluded.status,
                idempotency_key = excluded.idempotency_key,
                created_at_ms = excluded.created_at_ms,
                updated_at_ms = excluded.updated_at_ms,
                record_json = excluded.record_json",
        )
        .bind(i64::try_from(record.coupon_redemption_id)?)
        .bind(i64::try_from(record.tenant_id)?)
        .bind(i64::try_from(record.organization_id)?)
        .bind(i64::try_from(record.coupon_code_id)?)
        .bind(i64::try_from(record.coupon_template_id)?)
        .bind(
            record
                .marketing_campaign_id
                .map(i64::try_from)
                .transpose()?,
        )
        .bind(&record.subject_type)
        .bind(&record.subject_id)
        .bind(format!("{:?}", record.status).to_ascii_lowercase())
        .bind(&record.idempotency_key)
        .bind(i64::try_from(record.created_at_ms)?)
        .bind(i64::try_from(record.updated_at_ms)?)
        .bind(record_json)
        .execute(&self.pool)
        .await?;
        Ok(record.clone())
    }

    pub async fn list_coupon_redemption_records(&self) -> Result<Vec<CouponRedemptionRecord>> {
        let rows = sqlx::query_scalar::<_, String>(
            "SELECT record_json::text
             FROM ai_marketing_coupon_redemption
             ORDER BY updated_at_ms DESC, coupon_redemption_id",
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(decode_json_record).collect()
    }

    pub async fn find_coupon_redemption_record_by_idempotency_key(
        &self,
        idempotency_key: &str,
    ) -> Result<Option<CouponRedemptionRecord>> {
        let row = sqlx::query_scalar::<_, String>(
            "SELECT record_json::text
             FROM ai_marketing_coupon_redemption
             WHERE idempotency_key = $1",
        )
        .bind(idempotency_key)
        .fetch_optional(&self.pool)
        .await?;
        row.map(decode_json_record).transpose()
    }

    pub async fn insert_referral_program_record(
        &self,
        record: &ReferralProgramRecord,
    ) -> Result<ReferralProgramRecord> {
        let record_json = encode_json_record(record)?;
        sqlx::query(
            "INSERT INTO ai_marketing_referral_program (
                referral_program_id, tenant_id, organization_id, program_code, status,
                created_at_ms, updated_at_ms, record_json
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, CAST($8 AS JSONB))
             ON CONFLICT(referral_program_id) DO UPDATE SET
                tenant_id = excluded.tenant_id,
                organization_id = excluded.organization_id,
                program_code = excluded.program_code,
                status = excluded.status,
                created_at_ms = excluded.created_at_ms,
                updated_at_ms = excluded.updated_at_ms,
                record_json = excluded.record_json",
        )
        .bind(i64::try_from(record.referral_program_id)?)
        .bind(i64::try_from(record.tenant_id)?)
        .bind(i64::try_from(record.organization_id)?)
        .bind(&record.program_code)
        .bind(format!("{:?}", record.status).to_ascii_lowercase())
        .bind(i64::try_from(record.created_at_ms)?)
        .bind(i64::try_from(record.updated_at_ms)?)
        .bind(record_json)
        .execute(&self.pool)
        .await?;
        Ok(record.clone())
    }

    pub async fn list_referral_program_records(&self) -> Result<Vec<ReferralProgramRecord>> {
        let rows = sqlx::query_scalar::<_, String>(
            "SELECT record_json::text
             FROM ai_marketing_referral_program
             ORDER BY updated_at_ms DESC, referral_program_id",
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(decode_json_record).collect()
    }

    pub async fn insert_referral_invite_record(
        &self,
        record: &ReferralInviteRecord,
    ) -> Result<ReferralInviteRecord> {
        let record_json = encode_json_record(record)?;
        sqlx::query(
            "INSERT INTO ai_marketing_referral_invite (
                referral_invite_id, tenant_id, organization_id, referral_program_id,
                referrer_user_id, status, coupon_code_id, source_code, referred_user_id,
                created_at_ms, updated_at_ms, record_json
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, CAST($12 AS JSONB))
             ON CONFLICT(referral_invite_id) DO UPDATE SET
                tenant_id = excluded.tenant_id,
                organization_id = excluded.organization_id,
                referral_program_id = excluded.referral_program_id,
                referrer_user_id = excluded.referrer_user_id,
                status = excluded.status,
                coupon_code_id = excluded.coupon_code_id,
                source_code = excluded.source_code,
                referred_user_id = excluded.referred_user_id,
                created_at_ms = excluded.created_at_ms,
                updated_at_ms = excluded.updated_at_ms,
                record_json = excluded.record_json",
        )
        .bind(i64::try_from(record.referral_invite_id)?)
        .bind(i64::try_from(record.tenant_id)?)
        .bind(i64::try_from(record.organization_id)?)
        .bind(i64::try_from(record.referral_program_id)?)
        .bind(i64::try_from(record.referrer_user_id)?)
        .bind(format!("{:?}", record.status).to_ascii_lowercase())
        .bind(record.coupon_code_id.map(i64::try_from).transpose()?)
        .bind(&record.source_code)
        .bind(record.referred_user_id.map(i64::try_from).transpose()?)
        .bind(i64::try_from(record.created_at_ms)?)
        .bind(i64::try_from(record.updated_at_ms)?)
        .bind(record_json)
        .execute(&self.pool)
        .await?;
        Ok(record.clone())
    }

    pub async fn list_referral_invite_records(&self) -> Result<Vec<ReferralInviteRecord>> {
        let rows = sqlx::query_scalar::<_, String>(
            "SELECT record_json::text
             FROM ai_marketing_referral_invite
             ORDER BY updated_at_ms DESC, referral_invite_id",
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(decode_json_record).collect()
    }

    pub async fn insert_marketing_attribution_touch_record(
        &self,
        record: &MarketingAttributionTouchRecord,
    ) -> Result<MarketingAttributionTouchRecord> {
        let record_json = encode_json_record(record)?;
        sqlx::query(
            "INSERT INTO ai_marketing_attribution_touch (
                attribution_touch_id, tenant_id, organization_id, source_kind, source_code,
                partner_id, referrer_user_id, invite_code_id, conversion_subject_id,
                created_at_ms, updated_at_ms, record_json
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, CAST($12 AS JSONB))
             ON CONFLICT(attribution_touch_id) DO UPDATE SET
                tenant_id = excluded.tenant_id,
                organization_id = excluded.organization_id,
                source_kind = excluded.source_kind,
                source_code = excluded.source_code,
                partner_id = excluded.partner_id,
                referrer_user_id = excluded.referrer_user_id,
                invite_code_id = excluded.invite_code_id,
                conversion_subject_id = excluded.conversion_subject_id,
                created_at_ms = excluded.created_at_ms,
                updated_at_ms = excluded.updated_at_ms,
                record_json = excluded.record_json",
        )
        .bind(i64::try_from(record.attribution_touch_id)?)
        .bind(i64::try_from(record.tenant_id)?)
        .bind(i64::try_from(record.organization_id)?)
        .bind(format!("{:?}", record.source_kind).to_ascii_lowercase())
        .bind(&record.source_code)
        .bind(&record.partner_id)
        .bind(record.referrer_user_id.map(i64::try_from).transpose()?)
        .bind(record.invite_code_id.map(i64::try_from).transpose()?)
        .bind(&record.conversion_subject_id)
        .bind(i64::try_from(record.created_at_ms)?)
        .bind(i64::try_from(record.updated_at_ms)?)
        .bind(record_json)
        .execute(&self.pool)
        .await?;
        Ok(record.clone())
    }

    pub async fn list_marketing_attribution_touch_records(
        &self,
    ) -> Result<Vec<MarketingAttributionTouchRecord>> {
        let rows = sqlx::query_scalar::<_, String>(
            "SELECT record_json::text
             FROM ai_marketing_attribution_touch
             ORDER BY updated_at_ms DESC, attribution_touch_id",
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(decode_json_record).collect()
    }

    pub async fn insert_commerce_order(
        &self,
        order: &CommerceOrderRecord,
    ) -> Result<CommerceOrderRecord> {
        sqlx::query(
            "INSERT INTO ai_commerce_orders (
                order_id,
                project_id,
                user_id,
                target_kind,
                target_id,
                target_name,
                list_price_cents,
                payable_price_cents,
                list_price_label,
                payable_price_label,
                granted_units,
                bonus_units,
                applied_coupon_code,
                status,
                source,
                created_at_ms
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
             ON CONFLICT(order_id) DO UPDATE SET
                project_id = excluded.project_id,
                user_id = excluded.user_id,
                target_kind = excluded.target_kind,
                target_id = excluded.target_id,
                target_name = excluded.target_name,
                list_price_cents = excluded.list_price_cents,
                payable_price_cents = excluded.payable_price_cents,
                list_price_label = excluded.list_price_label,
                payable_price_label = excluded.payable_price_label,
                granted_units = excluded.granted_units,
                bonus_units = excluded.bonus_units,
                applied_coupon_code = excluded.applied_coupon_code,
                status = excluded.status,
                source = excluded.source,
                created_at_ms = excluded.created_at_ms",
        )
        .bind(&order.order_id)
        .bind(&order.project_id)
        .bind(&order.user_id)
        .bind(&order.target_kind)
        .bind(&order.target_id)
        .bind(&order.target_name)
        .bind(i64::try_from(order.list_price_cents)?)
        .bind(i64::try_from(order.payable_price_cents)?)
        .bind(&order.list_price_label)
        .bind(&order.payable_price_label)
        .bind(i64::try_from(order.granted_units)?)
        .bind(i64::try_from(order.bonus_units)?)
        .bind(&order.applied_coupon_code)
        .bind(&order.status)
        .bind(&order.source)
        .bind(i64::try_from(order.created_at_ms)?)
        .execute(&self.pool)
        .await?;
        Ok(order.clone())
    }

    pub async fn list_commerce_orders(&self) -> Result<Vec<CommerceOrderRecord>> {
        let rows = sqlx::query_as::<_, (
            String,
            String,
            String,
            String,
            String,
            String,
            i64,
            i64,
            String,
            String,
            i64,
            i64,
            Option<String>,
            String,
            String,
            i64,
        )>(
            "SELECT order_id, project_id, user_id, target_kind, target_id, target_name, list_price_cents, payable_price_cents, list_price_label, payable_price_label, granted_units, bonus_units, applied_coupon_code, status, source, created_at_ms
             FROM ai_commerce_orders
             ORDER BY created_at_ms DESC, order_id DESC",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(
                |(
                    order_id,
                    project_id,
                    user_id,
                    target_kind,
                    target_id,
                    target_name,
                    list_price_cents,
                    payable_price_cents,
                    list_price_label,
                    payable_price_label,
                    granted_units,
                    bonus_units,
                    applied_coupon_code,
                    status,
                    source,
                    created_at_ms,
                )| CommerceOrderRecord {
                    order_id,
                    project_id,
                    user_id,
                    target_kind,
                    target_id,
                    target_name,
                    list_price_cents: list_price_cents as u64,
                    payable_price_cents: payable_price_cents as u64,
                    list_price_label,
                    payable_price_label,
                    granted_units: granted_units as u64,
                    bonus_units: bonus_units as u64,
                    applied_coupon_code,
                    status,
                    source,
                    created_at_ms: created_at_ms as u64,
                },
            )
            .collect())
    }

    pub async fn list_commerce_orders_for_project(
        &self,
        project_id: &str,
    ) -> Result<Vec<CommerceOrderRecord>> {
        let rows = sqlx::query_as::<_, (
            String,
            String,
            String,
            String,
            String,
            String,
            i64,
            i64,
            String,
            String,
            i64,
            i64,
            Option<String>,
            String,
            String,
            i64,
        )>(
            "SELECT order_id, project_id, user_id, target_kind, target_id, target_name, list_price_cents, payable_price_cents, list_price_label, payable_price_label, granted_units, bonus_units, applied_coupon_code, status, source, created_at_ms
             FROM ai_commerce_orders
             WHERE project_id = $1
             ORDER BY created_at_ms DESC, order_id DESC",
        )
        .bind(project_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(
                |(
                    order_id,
                    project_id,
                    user_id,
                    target_kind,
                    target_id,
                    target_name,
                    list_price_cents,
                    payable_price_cents,
                    list_price_label,
                    payable_price_label,
                    granted_units,
                    bonus_units,
                    applied_coupon_code,
                    status,
                    source,
                    created_at_ms,
                )| CommerceOrderRecord {
                    order_id,
                    project_id,
                    user_id,
                    target_kind,
                    target_id,
                    target_name,
                    list_price_cents: list_price_cents as u64,
                    payable_price_cents: payable_price_cents as u64,
                    list_price_label,
                    payable_price_label,
                    granted_units: granted_units as u64,
                    bonus_units: bonus_units as u64,
                    applied_coupon_code,
                    status,
                    source,
                    created_at_ms: created_at_ms as u64,
                },
            )
            .collect())
    }

    pub async fn upsert_project_membership(
        &self,
        membership: &ProjectMembershipRecord,
    ) -> Result<ProjectMembershipRecord> {
        sqlx::query(
            "INSERT INTO ai_project_memberships (
                project_id,
                membership_id,
                user_id,
                plan_id,
                plan_name,
                price_cents,
                price_label,
                cadence,
                included_units,
                status,
                source,
                activated_at_ms,
                updated_at_ms
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
             ON CONFLICT(project_id) DO UPDATE SET
                membership_id = excluded.membership_id,
                user_id = excluded.user_id,
                plan_id = excluded.plan_id,
                plan_name = excluded.plan_name,
                price_cents = excluded.price_cents,
                price_label = excluded.price_label,
                cadence = excluded.cadence,
                included_units = excluded.included_units,
                status = excluded.status,
                source = excluded.source,
                activated_at_ms = excluded.activated_at_ms,
                updated_at_ms = excluded.updated_at_ms",
        )
        .bind(&membership.project_id)
        .bind(&membership.membership_id)
        .bind(&membership.user_id)
        .bind(&membership.plan_id)
        .bind(&membership.plan_name)
        .bind(i64::try_from(membership.price_cents)?)
        .bind(&membership.price_label)
        .bind(&membership.cadence)
        .bind(i64::try_from(membership.included_units)?)
        .bind(&membership.status)
        .bind(&membership.source)
        .bind(i64::try_from(membership.activated_at_ms)?)
        .bind(i64::try_from(membership.updated_at_ms)?)
        .execute(&self.pool)
        .await?;
        Ok(membership.clone())
    }

    pub async fn find_project_membership(
        &self,
        project_id: &str,
    ) -> Result<Option<ProjectMembershipRecord>> {
        let row = sqlx::query_as::<_, (
            String,
            String,
            String,
            String,
            String,
            i64,
            String,
            String,
            i64,
            String,
            String,
            i64,
            i64,
        )>(
            "SELECT membership_id, project_id, user_id, plan_id, plan_name, price_cents, price_label, cadence, included_units, status, source, activated_at_ms, updated_at_ms
             FROM ai_project_memberships
             WHERE project_id = $1",
        )
        .bind(project_id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(
            |(
                membership_id,
                project_id,
                user_id,
                plan_id,
                plan_name,
                price_cents,
                price_label,
                cadence,
                included_units,
                status,
                source,
                activated_at_ms,
                updated_at_ms,
            )| {
                Ok(ProjectMembershipRecord {
                    membership_id,
                    project_id,
                    user_id,
                    plan_id,
                    plan_name,
                    price_cents: u64::try_from(price_cents)?,
                    price_label,
                    cadence,
                    included_units: u64::try_from(included_units)?,
                    status,
                    source,
                    activated_at_ms: u64::try_from(activated_at_ms)?,
                    updated_at_ms: u64::try_from(updated_at_ms)?,
                })
            },
        )
        .transpose()
    }

    pub async fn insert_portal_workspace_membership(
        &self,
        membership: &PortalWorkspaceMembershipRecord,
    ) -> Result<PortalWorkspaceMembershipRecord> {
        sqlx::query(
            "INSERT INTO ai_portal_workspace_memberships (
                membership_id,
                user_id,
                tenant_id,
                project_id,
                role,
                created_at_ms
            ) VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT(user_id, tenant_id, project_id) DO UPDATE SET
                role = excluded.role,
                created_at_ms = excluded.created_at_ms",
        )
        .bind(&membership.membership_id)
        .bind(&membership.user_id)
        .bind(&membership.tenant_id)
        .bind(&membership.project_id)
        .bind(&membership.role)
        .bind(i64::try_from(membership.created_at_ms)?)
        .execute(&self.pool)
        .await?;

        self.find_portal_workspace_membership(
            &membership.user_id,
            &membership.tenant_id,
            &membership.project_id,
        )
        .await?
        .ok_or_else(|| anyhow::anyhow!("portal workspace membership was not persisted"))
    }

    pub async fn list_portal_workspace_memberships_for_user(
        &self,
        user_id: &str,
    ) -> Result<Vec<PortalWorkspaceMembershipRecord>> {
        let rows = sqlx::query_as::<_, PortalWorkspaceMembershipRow>(
            "SELECT membership_id, user_id, tenant_id, project_id, role, created_at_ms
             FROM ai_portal_workspace_memberships
             WHERE user_id = $1
             ORDER BY created_at_ms, membership_id",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter()
            .map(|row| {
                decode_portal_workspace_membership_row(Some(row))?
                    .ok_or_else(|| anyhow::anyhow!("portal workspace membership row should decode"))
            })
            .collect()
    }

    pub async fn find_portal_workspace_membership(
        &self,
        user_id: &str,
        tenant_id: &str,
        project_id: &str,
    ) -> Result<Option<PortalWorkspaceMembershipRecord>> {
        let row = sqlx::query_as::<_, PortalWorkspaceMembershipRow>(
            "SELECT membership_id, user_id, tenant_id, project_id, role, created_at_ms
             FROM ai_portal_workspace_memberships
             WHERE user_id = $1
               AND tenant_id = $2
               AND project_id = $3",
        )
        .bind(user_id)
        .bind(tenant_id)
        .bind(project_id)
        .fetch_optional(&self.pool)
        .await?;
        decode_portal_workspace_membership_row(row)
    }

    pub async fn insert_payment_order_record(
        &self,
        record: &PaymentOrderRecord,
    ) -> Result<PaymentOrderRecord> {
        sqlx::query(
            "INSERT INTO ai_payment_orders (
                payment_order_id,
                commerce_order_id,
                project_id,
                user_id,
                provider,
                currency_code,
                amount_cents,
                status,
                provider_reference_id,
                checkout_url,
                created_at_ms,
                updated_at_ms
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            ON CONFLICT(payment_order_id) DO UPDATE SET
                commerce_order_id = excluded.commerce_order_id,
                project_id = excluded.project_id,
                user_id = excluded.user_id,
                provider = excluded.provider,
                currency_code = excluded.currency_code,
                amount_cents = excluded.amount_cents,
                status = excluded.status,
                provider_reference_id = excluded.provider_reference_id,
                checkout_url = excluded.checkout_url,
                created_at_ms = excluded.created_at_ms,
                updated_at_ms = excluded.updated_at_ms",
        )
        .bind(&record.payment_order_id)
        .bind(&record.commerce_order_id)
        .bind(&record.project_id)
        .bind(&record.user_id)
        .bind(&record.provider)
        .bind(&record.currency_code)
        .bind(i64::try_from(record.amount_cents)?)
        .bind(&record.status)
        .bind(&record.provider_reference_id)
        .bind(&record.checkout_url)
        .bind(i64::try_from(record.created_at_ms)?)
        .bind(i64::try_from(record.updated_at_ms)?)
        .execute(&self.pool)
        .await?;
        self.find_payment_order_record(&record.payment_order_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("payment order was not persisted"))
    }

    pub async fn find_payment_order_record(
        &self,
        payment_order_id: &str,
    ) -> Result<Option<PaymentOrderRecord>> {
        let row = sqlx::query_as::<_, PaymentOrderRow>(
            "SELECT payment_order_id, commerce_order_id, project_id, user_id, provider, currency_code, amount_cents, status, provider_reference_id, checkout_url, created_at_ms, updated_at_ms
             FROM ai_payment_orders
             WHERE payment_order_id = $1",
        )
        .bind(payment_order_id)
        .fetch_optional(&self.pool)
        .await?;
        decode_payment_order_row(row)
    }

    pub async fn find_payment_order_record_by_commerce_order_id(
        &self,
        commerce_order_id: &str,
    ) -> Result<Option<PaymentOrderRecord>> {
        let row = sqlx::query_as::<_, PaymentOrderRow>(
            "SELECT payment_order_id, commerce_order_id, project_id, user_id, provider, currency_code, amount_cents, status, provider_reference_id, checkout_url, created_at_ms, updated_at_ms
             FROM ai_payment_orders
             WHERE commerce_order_id = $1",
        )
        .bind(commerce_order_id)
        .fetch_optional(&self.pool)
        .await?;
        decode_payment_order_row(row)
    }

    pub async fn find_payment_order_record_by_provider_reference(
        &self,
        provider: &str,
        provider_reference_id: &str,
    ) -> Result<Option<PaymentOrderRecord>> {
        let row = sqlx::query_as::<_, PaymentOrderRow>(
            "SELECT payment_order_id, commerce_order_id, project_id, user_id, provider, currency_code, amount_cents, status, provider_reference_id, checkout_url, created_at_ms, updated_at_ms
             FROM ai_payment_orders
             WHERE provider = $1
               AND provider_reference_id = $2",
        )
        .bind(provider)
        .bind(provider_reference_id)
        .fetch_optional(&self.pool)
        .await?;
        decode_payment_order_row(row)
    }

    pub async fn insert_payment_attempt_record(
        &self,
        record: &PaymentAttemptRecord,
    ) -> Result<PaymentAttemptRecord> {
        sqlx::query(
            "INSERT INTO ai_payment_attempts (
                payment_attempt_id,
                payment_order_id,
                provider,
                provider_attempt_id,
                attempt_kind,
                status,
                currency_code,
                amount_cents,
                idempotency_key,
                error_code,
                error_message,
                created_at_ms,
                updated_at_ms
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            ON CONFLICT(payment_attempt_id) DO UPDATE SET
                payment_order_id = excluded.payment_order_id,
                provider = excluded.provider,
                provider_attempt_id = excluded.provider_attempt_id,
                attempt_kind = excluded.attempt_kind,
                status = excluded.status,
                currency_code = excluded.currency_code,
                amount_cents = excluded.amount_cents,
                idempotency_key = excluded.idempotency_key,
                error_code = excluded.error_code,
                error_message = excluded.error_message,
                created_at_ms = excluded.created_at_ms,
                updated_at_ms = excluded.updated_at_ms",
        )
        .bind(&record.payment_attempt_id)
        .bind(&record.payment_order_id)
        .bind(&record.provider)
        .bind(&record.provider_attempt_id)
        .bind(&record.attempt_kind)
        .bind(&record.status)
        .bind(&record.currency_code)
        .bind(i64::try_from(record.amount_cents)?)
        .bind(&record.idempotency_key)
        .bind(&record.error_code)
        .bind(&record.error_message)
        .bind(i64::try_from(record.created_at_ms)?)
        .bind(i64::try_from(record.updated_at_ms)?)
        .execute(&self.pool)
        .await?;
        self.find_payment_attempt_record(&record.payment_attempt_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("payment attempt was not persisted"))
    }

    pub async fn find_payment_attempt_record(
        &self,
        payment_attempt_id: &str,
    ) -> Result<Option<PaymentAttemptRecord>> {
        let row = sqlx::query_as::<_, PaymentAttemptRow>(
            "SELECT payment_attempt_id, payment_order_id, provider, provider_attempt_id, attempt_kind, status, currency_code, amount_cents, idempotency_key, error_code, error_message, created_at_ms, updated_at_ms
             FROM ai_payment_attempts
             WHERE payment_attempt_id = $1",
        )
        .bind(payment_attempt_id)
        .fetch_optional(&self.pool)
        .await?;
        decode_payment_attempt_row(row)
    }

    pub async fn find_payment_attempt_record_by_provider_reference(
        &self,
        provider: &str,
        provider_attempt_id: &str,
    ) -> Result<Option<PaymentAttemptRecord>> {
        let row = sqlx::query_as::<_, PaymentAttemptRow>(
            "SELECT payment_attempt_id, payment_order_id, provider, provider_attempt_id, attempt_kind, status, currency_code, amount_cents, idempotency_key, error_code, error_message, created_at_ms, updated_at_ms
             FROM ai_payment_attempts
             WHERE provider = $1
               AND provider_attempt_id = $2",
        )
        .bind(provider)
        .bind(provider_attempt_id)
        .fetch_optional(&self.pool)
        .await?;
        decode_payment_attempt_row(row)
    }

    pub async fn list_payment_attempt_records_for_payment_order(
        &self,
        payment_order_id: &str,
    ) -> Result<Vec<PaymentAttemptRecord>> {
        let rows = sqlx::query_as::<_, PaymentAttemptRow>(
            "SELECT payment_attempt_id, payment_order_id, provider, provider_attempt_id, attempt_kind, status, currency_code, amount_cents, idempotency_key, error_code, error_message, created_at_ms, updated_at_ms
             FROM ai_payment_attempts
             WHERE payment_order_id = $1
             ORDER BY created_at_ms ASC, payment_attempt_id ASC",
        )
        .bind(payment_order_id)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter()
            .map(Some)
            .map(decode_payment_attempt_row)
            .collect::<Result<Vec<_>>>()
            .map(|records| records.into_iter().flatten().collect())
    }

    pub async fn insert_refund_record(&self, record: &RefundRecord) -> Result<RefundRecord> {
        sqlx::query(
            "INSERT INTO ai_payment_refunds (
                refund_id,
                payment_order_id,
                provider,
                provider_refund_id,
                status,
                currency_code,
                amount_cents,
                reason,
                failure_code,
                failure_message,
                created_at_ms,
                updated_at_ms
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            ON CONFLICT(refund_id) DO UPDATE SET
                payment_order_id = excluded.payment_order_id,
                provider = excluded.provider,
                provider_refund_id = excluded.provider_refund_id,
                status = excluded.status,
                currency_code = excluded.currency_code,
                amount_cents = excluded.amount_cents,
                reason = excluded.reason,
                failure_code = excluded.failure_code,
                failure_message = excluded.failure_message,
                created_at_ms = excluded.created_at_ms,
                updated_at_ms = excluded.updated_at_ms",
        )
        .bind(&record.refund_id)
        .bind(&record.payment_order_id)
        .bind(&record.provider)
        .bind(&record.provider_refund_id)
        .bind(&record.status)
        .bind(&record.currency_code)
        .bind(i64::try_from(record.amount_cents)?)
        .bind(&record.reason)
        .bind(&record.failure_code)
        .bind(&record.failure_message)
        .bind(i64::try_from(record.created_at_ms)?)
        .bind(i64::try_from(record.updated_at_ms)?)
        .execute(&self.pool)
        .await?;
        self.find_refund_record(&record.refund_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("refund record was not persisted"))
    }

    pub async fn find_refund_record(&self, refund_id: &str) -> Result<Option<RefundRecord>> {
        let row = sqlx::query_as::<_, RefundRow>(
            "SELECT refund_id, payment_order_id, provider, provider_refund_id, status, currency_code, amount_cents, reason, failure_code, failure_message, created_at_ms, updated_at_ms
             FROM ai_payment_refunds
             WHERE refund_id = $1",
        )
        .bind(refund_id)
        .fetch_optional(&self.pool)
        .await?;
        decode_refund_row(row)
    }

    pub async fn find_refund_record_by_provider_reference(
        &self,
        provider: &str,
        provider_refund_id: &str,
    ) -> Result<Option<RefundRecord>> {
        let row = sqlx::query_as::<_, RefundRow>(
            "SELECT refund_id, payment_order_id, provider, provider_refund_id, status, currency_code, amount_cents, reason, failure_code, failure_message, created_at_ms, updated_at_ms
             FROM ai_payment_refunds
             WHERE provider = $1
               AND provider_refund_id = $2",
        )
        .bind(provider)
        .bind(provider_refund_id)
        .fetch_optional(&self.pool)
        .await?;
        decode_refund_row(row)
    }

    pub async fn list_refund_records_for_payment_order(
        &self,
        payment_order_id: &str,
    ) -> Result<Vec<RefundRecord>> {
        let rows = sqlx::query_as::<_, RefundRow>(
            "SELECT refund_id, payment_order_id, provider, provider_refund_id, status, currency_code, amount_cents, reason, failure_code, failure_message, created_at_ms, updated_at_ms
             FROM ai_payment_refunds
             WHERE payment_order_id = $1
             ORDER BY created_at_ms ASC, refund_id ASC",
        )
        .bind(payment_order_id)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter()
            .map(Some)
            .map(decode_refund_row)
            .collect::<Result<Vec<_>>>()
            .map(|records| records.into_iter().flatten().collect())
    }

    pub async fn insert_dispute_record(&self, record: &DisputeRecord) -> Result<DisputeRecord> {
        sqlx::query(
            "INSERT INTO ai_payment_disputes (
                dispute_id,
                payment_order_id,
                provider,
                provider_dispute_id,
                status,
                reason,
                currency_code,
                amount_cents,
                evidence_due_at_ms,
                created_at_ms,
                updated_at_ms
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            ON CONFLICT(dispute_id) DO UPDATE SET
                payment_order_id = excluded.payment_order_id,
                provider = excluded.provider,
                provider_dispute_id = excluded.provider_dispute_id,
                status = excluded.status,
                reason = excluded.reason,
                currency_code = excluded.currency_code,
                amount_cents = excluded.amount_cents,
                evidence_due_at_ms = excluded.evidence_due_at_ms,
                created_at_ms = excluded.created_at_ms,
                updated_at_ms = excluded.updated_at_ms",
        )
        .bind(&record.dispute_id)
        .bind(&record.payment_order_id)
        .bind(&record.provider)
        .bind(&record.provider_dispute_id)
        .bind(&record.status)
        .bind(&record.reason)
        .bind(&record.currency_code)
        .bind(i64::try_from(record.amount_cents)?)
        .bind(record.evidence_due_at_ms.map(i64::try_from).transpose()?)
        .bind(i64::try_from(record.created_at_ms)?)
        .bind(i64::try_from(record.updated_at_ms)?)
        .execute(&self.pool)
        .await?;
        self.find_dispute_record(&record.dispute_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("dispute record was not persisted"))
    }

    pub async fn find_dispute_record(&self, dispute_id: &str) -> Result<Option<DisputeRecord>> {
        let row = sqlx::query_as::<_, DisputeRow>(
            "SELECT dispute_id, payment_order_id, provider, provider_dispute_id, status, reason, currency_code, amount_cents, evidence_due_at_ms, created_at_ms, updated_at_ms
             FROM ai_payment_disputes
             WHERE dispute_id = $1",
        )
        .bind(dispute_id)
        .fetch_optional(&self.pool)
        .await?;
        decode_dispute_row(row)
    }

    pub async fn find_dispute_record_by_provider_reference(
        &self,
        provider: &str,
        provider_dispute_id: &str,
    ) -> Result<Option<DisputeRecord>> {
        let row = sqlx::query_as::<_, DisputeRow>(
            "SELECT dispute_id, payment_order_id, provider, provider_dispute_id, status, reason, currency_code, amount_cents, evidence_due_at_ms, created_at_ms, updated_at_ms
             FROM ai_payment_disputes
             WHERE provider = $1
               AND provider_dispute_id = $2",
        )
        .bind(provider)
        .bind(provider_dispute_id)
        .fetch_optional(&self.pool)
        .await?;
        decode_dispute_row(row)
    }

    pub async fn list_dispute_records_for_payment_order(
        &self,
        payment_order_id: &str,
    ) -> Result<Vec<DisputeRecord>> {
        let rows = sqlx::query_as::<_, DisputeRow>(
            "SELECT dispute_id, payment_order_id, provider, provider_dispute_id, status, reason, currency_code, amount_cents, evidence_due_at_ms, created_at_ms, updated_at_ms
             FROM ai_payment_disputes
             WHERE payment_order_id = $1
             ORDER BY created_at_ms ASC, dispute_id ASC",
        )
        .bind(payment_order_id)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter()
            .map(Some)
            .map(decode_dispute_row)
            .collect::<Result<Vec<_>>>()
            .map(|records| records.into_iter().flatten().collect())
    }

    pub async fn insert_payment_webhook_event_record(
        &self,
        record: &PaymentWebhookEventRecord,
    ) -> Result<PaymentWebhookEventRecord> {
        sqlx::query(
            "INSERT INTO ai_payment_webhook_events (
                payment_webhook_event_id,
                provider,
                provider_event_id,
                payment_order_id,
                commerce_order_id,
                event_type,
                status,
                payload_json,
                created_at_ms
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT(provider, provider_event_id) DO UPDATE SET
                payment_order_id = excluded.payment_order_id,
                commerce_order_id = excluded.commerce_order_id,
                event_type = excluded.event_type,
                status = excluded.status,
                payload_json = excluded.payload_json,
                created_at_ms = excluded.created_at_ms",
        )
        .bind(&record.payment_webhook_event_id)
        .bind(&record.provider)
        .bind(&record.provider_event_id)
        .bind(&record.payment_order_id)
        .bind(&record.commerce_order_id)
        .bind(&record.event_type)
        .bind(&record.status)
        .bind(&record.payload_json)
        .bind(i64::try_from(record.created_at_ms)?)
        .execute(&self.pool)
        .await?;
        self.find_payment_webhook_event_record(&record.provider, &record.provider_event_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("payment webhook event was not persisted"))
    }

    pub async fn find_payment_webhook_event_record(
        &self,
        provider: &str,
        provider_event_id: &str,
    ) -> Result<Option<PaymentWebhookEventRecord>> {
        let row = sqlx::query_as::<_, PaymentWebhookEventRow>(
            "SELECT payment_webhook_event_id, provider, provider_event_id, payment_order_id, commerce_order_id, event_type, status, payload_json, created_at_ms
             FROM ai_payment_webhook_events
             WHERE provider = $1
               AND provider_event_id = $2",
        )
        .bind(provider)
        .bind(provider_event_id)
        .fetch_optional(&self.pool)
        .await?;
        decode_payment_webhook_event_row(row)
    }

    pub async fn insert_portal_user(&self, user: &PortalUserRecord) -> Result<PortalUserRecord> {
        sqlx::query(
            "INSERT INTO ai_portal_users (id, email, display_name, password_salt, password_hash, workspace_tenant_id, workspace_project_id, active, created_at_ms)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
             ON CONFLICT(id) DO UPDATE SET
             email = excluded.email,
             display_name = excluded.display_name,
             password_salt = excluded.password_salt,
             password_hash = excluded.password_hash,
             workspace_tenant_id = excluded.workspace_tenant_id,
             workspace_project_id = excluded.workspace_project_id,
             active = excluded.active,
             created_at_ms = excluded.created_at_ms",
        )
        .bind(&user.id)
        .bind(&user.email)
        .bind(&user.display_name)
        .bind(&user.password_salt)
        .bind(&user.password_hash)
        .bind(&user.workspace_tenant_id)
        .bind(&user.workspace_project_id)
        .bind(user.active)
        .bind(i64::try_from(user.created_at_ms)?)
        .execute(&self.pool)
        .await?;
        Ok(user.clone())
    }

    pub async fn list_portal_users(&self) -> Result<Vec<PortalUserRecord>> {
        let rows = sqlx::query_as::<_, PortalUserRow>(
            "SELECT id, email, display_name, password_salt, password_hash, workspace_tenant_id, workspace_project_id, active, created_at_ms
             FROM ai_portal_users
             ORDER BY created_at_ms DESC, email ASC",
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|row| decode_portal_user_row(Some(row)))
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .map(|row| row.ok_or_else(|| anyhow::anyhow!("portal user row decode returned empty")))
            .collect()
    }

    pub async fn find_portal_user_by_email(&self, email: &str) -> Result<Option<PortalUserRecord>> {
        let row = sqlx::query_as::<_, (String, String, String, String, String, String, String, bool, i64)>(
            "SELECT id, email, display_name, password_salt, password_hash, workspace_tenant_id, workspace_project_id, active, created_at_ms
             FROM ai_portal_users
             WHERE email = $1",
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await?;
        decode_portal_user_row(row)
    }

    pub async fn find_portal_user_by_id(&self, user_id: &str) -> Result<Option<PortalUserRecord>> {
        let row = sqlx::query_as::<_, (String, String, String, String, String, String, String, bool, i64)>(
            "SELECT id, email, display_name, password_salt, password_hash, workspace_tenant_id, workspace_project_id, active, created_at_ms
             FROM ai_portal_users
             WHERE id = $1",
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;
        decode_portal_user_row(row)
    }

    pub async fn delete_portal_user(&self, user_id: &str) -> Result<bool> {
        let result = sqlx::query("DELETE FROM ai_portal_users WHERE id = $1")
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn insert_admin_user(&self, user: &AdminUserRecord) -> Result<AdminUserRecord> {
        sqlx::query(
            "INSERT INTO ai_admin_users (id, email, display_name, password_salt, password_hash, role, active, created_at_ms)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
             ON CONFLICT(id) DO UPDATE SET
              email = excluded.email,
              display_name = excluded.display_name,
              password_salt = excluded.password_salt,
              password_hash = excluded.password_hash,
              role = excluded.role,
              active = excluded.active,
              created_at_ms = excluded.created_at_ms",
        )
        .bind(&user.id)
        .bind(&user.email)
        .bind(&user.display_name)
        .bind(&user.password_salt)
        .bind(&user.password_hash)
        .bind(user.role.as_str())
        .bind(user.active)
        .bind(i64::try_from(user.created_at_ms)?)
        .execute(&self.pool)
        .await?;
        Ok(user.clone())
    }

    pub async fn list_admin_users(&self) -> Result<Vec<AdminUserRecord>> {
        let rows = sqlx::query_as::<_, AdminUserRow>(
            "SELECT id, email, display_name, password_salt, password_hash, role, active, created_at_ms
             FROM ai_admin_users
             ORDER BY created_at_ms DESC, email ASC",
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|row| decode_admin_user_row(Some(row)))
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .map(|row| row.ok_or_else(|| anyhow::anyhow!("admin user row decode returned empty")))
            .collect()
    }

    pub async fn find_admin_user_by_email(&self, email: &str) -> Result<Option<AdminUserRecord>> {
        let row = sqlx::query_as::<_, AdminUserRow>(
            "SELECT id, email, display_name, password_salt, password_hash, role, active, created_at_ms
             FROM ai_admin_users
             WHERE email = $1",
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await?;
        decode_admin_user_row(row)
    }

    pub async fn find_admin_user_by_id(&self, user_id: &str) -> Result<Option<AdminUserRecord>> {
        let row = sqlx::query_as::<_, AdminUserRow>(
            "SELECT id, email, display_name, password_salt, password_hash, role, active, created_at_ms
             FROM ai_admin_users
             WHERE id = $1",
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;
        decode_admin_user_row(row)
    }

    pub async fn delete_admin_user(&self, user_id: &str) -> Result<bool> {
        let result = sqlx::query("DELETE FROM ai_admin_users WHERE id = $1")
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn insert_admin_audit_event(
        &self,
        event: &AdminAuditEventRecord,
    ) -> Result<AdminAuditEventRecord> {
        sqlx::query(
            "INSERT INTO ai_admin_audit_events (
                event_id,
                actor_user_id,
                actor_email,
                actor_role,
                action,
                resource_type,
                resource_id,
                approval_scope,
                target_tenant_id,
                target_project_id,
                target_provider_id,
                created_at_ms
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
             ON CONFLICT(event_id) DO UPDATE SET
                actor_user_id = EXCLUDED.actor_user_id,
                actor_email = EXCLUDED.actor_email,
                actor_role = EXCLUDED.actor_role,
                action = EXCLUDED.action,
                resource_type = EXCLUDED.resource_type,
                resource_id = EXCLUDED.resource_id,
                approval_scope = EXCLUDED.approval_scope,
                target_tenant_id = EXCLUDED.target_tenant_id,
                target_project_id = EXCLUDED.target_project_id,
                target_provider_id = EXCLUDED.target_provider_id,
                created_at_ms = EXCLUDED.created_at_ms",
        )
        .bind(&event.event_id)
        .bind(&event.actor_user_id)
        .bind(&event.actor_email)
        .bind(event.actor_role.as_str())
        .bind(&event.action)
        .bind(&event.resource_type)
        .bind(&event.resource_id)
        .bind(&event.approval_scope)
        .bind(&event.target_tenant_id)
        .bind(&event.target_project_id)
        .bind(&event.target_provider_id)
        .bind(i64::try_from(event.created_at_ms)?)
        .execute(&self.pool)
        .await?;
        Ok(event.clone())
    }

    pub async fn list_admin_audit_events(&self) -> Result<Vec<AdminAuditEventRecord>> {
        let rows = sqlx::query_as::<_, AdminAuditEventRow>(
            "SELECT event_id, actor_user_id, actor_email, actor_role, action, resource_type, resource_id, approval_scope, target_tenant_id, target_project_id, target_provider_id, created_at_ms
             FROM ai_admin_audit_events
             ORDER BY created_at_ms DESC, event_id DESC",
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|row| decode_admin_audit_event_row(Some(row)))
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .map(|row| {
                row.ok_or_else(|| anyhow::anyhow!("admin audit event row decode returned empty"))
            })
            .collect()
    }

    pub async fn insert_gateway_api_key(
        &self,
        record: &GatewayApiKeyRecord,
    ) -> Result<GatewayApiKeyRecord> {
        sqlx::query(
            "INSERT INTO ai_app_api_keys (
                hashed_key,
                tenant_id,
                project_id,
                environment,
                api_key_group_id,
                key_prefix,
                label,
                notes,
                created_at_ms,
                last_used_at_ms,
                expires_at_ms,
                active
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
             ON CONFLICT(hashed_key) DO UPDATE SET
                raw_key = NULL,
                tenant_id = excluded.tenant_id,
                project_id = excluded.project_id,
                environment = excluded.environment,
                api_key_group_id = excluded.api_key_group_id,
                key_prefix = excluded.key_prefix,
                label = excluded.label,
                notes = excluded.notes,
                created_at_ms = excluded.created_at_ms,
                last_used_at_ms = excluded.last_used_at_ms,
                expires_at_ms = excluded.expires_at_ms,
                active = excluded.active",
        )
        .bind(&record.hashed_key)
        .bind(&record.tenant_id)
        .bind(&record.project_id)
        .bind(&record.environment)
        .bind(&record.api_key_group_id)
        .bind(&record.key_prefix)
        .bind(&record.label)
        .bind(&record.notes)
        .bind(i64::try_from(record.created_at_ms).unwrap_or(i64::MAX))
        .bind(
            record
                .last_used_at_ms
                .map(|value| i64::try_from(value).unwrap_or(i64::MAX)),
        )
        .bind(
            record
                .expires_at_ms
                .map(|value| i64::try_from(value).unwrap_or(i64::MAX)),
        )
        .bind(record.active)
        .execute(&self.pool)
        .await?;
        Ok(record.clone())
    }

    pub async fn list_gateway_api_keys(&self) -> Result<Vec<GatewayApiKeyRecord>> {
        let rows = sqlx::query_as::<_, (String, String, String, String, String, Option<String>, String, Option<String>, i64, Option<i64>, Option<i64>, bool)>(
            "SELECT hashed_key, key_prefix, tenant_id, project_id, environment, api_key_group_id, label, notes, created_at_ms, last_used_at_ms, expires_at_ms, active
             FROM ai_app_api_keys
             ORDER BY created_at_ms DESC, hashed_key",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(
                |(
                    hashed_key,
                    key_prefix,
                    tenant_id,
                    project_id,
                    environment,
                    api_key_group_id,
                    label,
                    notes,
                    created_at_ms,
                    last_used_at_ms,
                    expires_at_ms,
                    active,
                )| GatewayApiKeyRecord {
                    tenant_id,
                    project_id,
                    environment,
                    hashed_key,
                    api_key_group_id,
                    key_prefix,
                    label,
                    notes,
                    created_at_ms: u64::try_from(created_at_ms).unwrap_or_default(),
                    last_used_at_ms: last_used_at_ms.and_then(|value| u64::try_from(value).ok()),
                    expires_at_ms: expires_at_ms.and_then(|value| u64::try_from(value).ok()),
                    active,
                },
            )
            .collect())
    }

    pub async fn find_gateway_api_key(
        &self,
        hashed_key: &str,
    ) -> Result<Option<GatewayApiKeyRecord>> {
        let row = sqlx::query_as::<_, (String, String, String, String, String, Option<String>, String, Option<String>, i64, Option<i64>, Option<i64>, bool)>(
            "SELECT hashed_key, key_prefix, tenant_id, project_id, environment, api_key_group_id, label, notes, created_at_ms, last_used_at_ms, expires_at_ms, active
             FROM ai_app_api_keys
             WHERE hashed_key = $1",
        )
        .bind(hashed_key)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(
            |(
                hashed_key,
                key_prefix,
                tenant_id,
                project_id,
                environment,
                api_key_group_id,
                label,
                notes,
                created_at_ms,
                last_used_at_ms,
                expires_at_ms,
                active,
            )| {
                GatewayApiKeyRecord {
                    tenant_id,
                    project_id,
                    environment,
                    hashed_key,
                    api_key_group_id,
                    key_prefix,
                    label,
                    notes,
                    created_at_ms: u64::try_from(created_at_ms).unwrap_or_default(),
                    last_used_at_ms: last_used_at_ms.and_then(|value| u64::try_from(value).ok()),
                    expires_at_ms: expires_at_ms.and_then(|value| u64::try_from(value).ok()),
                    active,
                }
            },
        ))
    }

    pub async fn delete_gateway_api_key(&self, hashed_key: &str) -> Result<bool> {
        let result = sqlx::query("DELETE FROM ai_app_api_keys WHERE hashed_key = $1")
            .bind(hashed_key)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn insert_api_key_group(
        &self,
        record: &ApiKeyGroupRecord,
    ) -> Result<ApiKeyGroupRecord> {
        sqlx::query(
            "INSERT INTO ai_app_api_key_groups (
                group_id,
                tenant_id,
                project_id,
                environment,
                name,
                slug,
                description,
                color,
                default_capability_scope,
                default_routing_profile_id,
                default_accounting_mode,
                active,
                created_at_ms,
                updated_at_ms
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
             ON CONFLICT(group_id) DO UPDATE SET
                tenant_id = excluded.tenant_id,
                project_id = excluded.project_id,
                environment = excluded.environment,
                name = excluded.name,
                slug = excluded.slug,
                description = excluded.description,
                color = excluded.color,
                default_capability_scope = excluded.default_capability_scope,
                default_routing_profile_id = excluded.default_routing_profile_id,
                default_accounting_mode = excluded.default_accounting_mode,
                active = excluded.active,
                created_at_ms = excluded.created_at_ms,
                updated_at_ms = excluded.updated_at_ms",
        )
        .bind(&record.group_id)
        .bind(&record.tenant_id)
        .bind(&record.project_id)
        .bind(&record.environment)
        .bind(&record.name)
        .bind(&record.slug)
        .bind(&record.description)
        .bind(&record.color)
        .bind(&record.default_capability_scope)
        .bind(&record.default_routing_profile_id)
        .bind(&record.default_accounting_mode)
        .bind(record.active)
        .bind(i64::try_from(record.created_at_ms).unwrap_or(i64::MAX))
        .bind(i64::try_from(record.updated_at_ms).unwrap_or(i64::MAX))
        .execute(&self.pool)
        .await?;
        Ok(record.clone())
    }

    pub async fn list_api_key_groups(&self) -> Result<Vec<ApiKeyGroupRecord>> {
        let rows = sqlx::query_as::<_, (String, String, String, String, String, String, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, bool, i64, i64)>(
            "SELECT group_id, tenant_id, project_id, environment, name, slug, description, color, default_capability_scope, default_routing_profile_id, default_accounting_mode, active, created_at_ms, updated_at_ms
             FROM ai_app_api_key_groups
             ORDER BY created_at_ms DESC, group_id",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(
                |(
                    group_id,
                    tenant_id,
                    project_id,
                    environment,
                    name,
                    slug,
                    description,
                    color,
                    default_capability_scope,
                    default_routing_profile_id,
                    default_accounting_mode,
                    active,
                    created_at_ms,
                    updated_at_ms,
                )| ApiKeyGroupRecord {
                    group_id,
                    tenant_id,
                    project_id,
                    environment,
                    name,
                    slug,
                    description,
                    color,
                    default_capability_scope,
                    default_routing_profile_id,
                    default_accounting_mode,
                    active,
                    created_at_ms: u64::try_from(created_at_ms).unwrap_or_default(),
                    updated_at_ms: u64::try_from(updated_at_ms).unwrap_or_default(),
                },
            )
            .collect())
    }

    pub async fn find_api_key_group(&self, group_id: &str) -> Result<Option<ApiKeyGroupRecord>> {
        let row = sqlx::query_as::<_, (String, String, String, String, String, String, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, bool, i64, i64)>(
            "SELECT group_id, tenant_id, project_id, environment, name, slug, description, color, default_capability_scope, default_routing_profile_id, default_accounting_mode, active, created_at_ms, updated_at_ms
             FROM ai_app_api_key_groups
             WHERE group_id = $1",
        )
        .bind(group_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(
            |(
                group_id,
                tenant_id,
                project_id,
                environment,
                name,
                slug,
                description,
                color,
                default_capability_scope,
                default_routing_profile_id,
                default_accounting_mode,
                active,
                created_at_ms,
                updated_at_ms,
            )| ApiKeyGroupRecord {
                group_id,
                tenant_id,
                project_id,
                environment,
                name,
                slug,
                description,
                color,
                default_capability_scope,
                default_routing_profile_id,
                default_accounting_mode,
                active,
                created_at_ms: u64::try_from(created_at_ms).unwrap_or_default(),
                updated_at_ms: u64::try_from(updated_at_ms).unwrap_or_default(),
            },
        ))
    }

    pub async fn delete_api_key_group(&self, group_id: &str) -> Result<bool> {
        let result = sqlx::query("DELETE FROM ai_app_api_key_groups WHERE group_id = $1")
            .bind(group_id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn insert_extension_installation(
        &self,
        installation: &ExtensionInstallation,
    ) -> Result<ExtensionInstallation> {
        sqlx::query(
            "INSERT INTO ai_extension_installations (installation_id, extension_id, runtime, enabled, entrypoint, config_json) VALUES ($1, $2, $3, $4, $5, $6)
             ON CONFLICT(installation_id) DO UPDATE SET extension_id = excluded.extension_id, runtime = excluded.runtime, enabled = excluded.enabled, entrypoint = excluded.entrypoint, config_json = excluded.config_json",
        )
        .bind(&installation.installation_id)
        .bind(&installation.extension_id)
        .bind(installation.runtime.as_str())
        .bind(installation.enabled)
        .bind(&installation.entrypoint)
        .bind(encode_extension_config(&installation.config)?)
        .execute(&self.pool)
        .await?;
        Ok(installation.clone())
    }

    pub async fn list_extension_installations(&self) -> Result<Vec<ExtensionInstallation>> {
        let rows = sqlx::query_as::<_, (String, String, String, bool, Option<String>, String)>(
            "SELECT installation_id, extension_id, runtime, enabled, entrypoint, config_json
             FROM ai_extension_installations
             ORDER BY installation_id",
        )
        .fetch_all(&self.pool)
        .await?;

        let mut installations = Vec::with_capacity(rows.len());
        for (installation_id, extension_id, runtime, enabled, entrypoint, config_json) in rows {
            installations.push(ExtensionInstallation {
                installation_id,
                extension_id,
                runtime: ExtensionRuntime::from_str(&runtime)?,
                enabled,
                entrypoint,
                config: decode_extension_config(&config_json)?,
            });
        }
        Ok(installations)
    }

    pub async fn insert_extension_instance(
        &self,
        instance: &ExtensionInstance,
    ) -> Result<ExtensionInstance> {
        sqlx::query(
            "INSERT INTO ai_extension_instances (instance_id, installation_id, extension_id, enabled, base_url, credential_ref, config_json) VALUES ($1, $2, $3, $4, $5, $6, $7)
             ON CONFLICT(instance_id) DO UPDATE SET installation_id = excluded.installation_id, extension_id = excluded.extension_id, enabled = excluded.enabled, base_url = excluded.base_url, credential_ref = excluded.credential_ref, config_json = excluded.config_json",
        )
        .bind(&instance.instance_id)
        .bind(&instance.installation_id)
        .bind(&instance.extension_id)
        .bind(instance.enabled)
        .bind(&instance.base_url)
        .bind(&instance.credential_ref)
        .bind(encode_extension_config(&instance.config)?)
        .execute(&self.pool)
        .await?;
        Ok(instance.clone())
    }

    pub async fn list_extension_instances(&self) -> Result<Vec<ExtensionInstance>> {
        let rows = sqlx::query_as::<
            _,
            (
                String,
                String,
                String,
                bool,
                Option<String>,
                Option<String>,
                String,
            ),
        >(
            "SELECT instance_id, installation_id, extension_id, enabled, base_url, credential_ref, config_json
             FROM ai_extension_instances
             ORDER BY instance_id",
        )
        .fetch_all(&self.pool)
        .await?;

        let mut instances = Vec::with_capacity(rows.len());
        for (
            instance_id,
            installation_id,
            extension_id,
            enabled,
            base_url,
            credential_ref,
            config_json,
        ) in rows
        {
            instances.push(ExtensionInstance {
                instance_id,
                installation_id,
                extension_id,
                enabled,
                base_url,
                credential_ref,
                config: decode_extension_config(&config_json)?,
            });
        }
        Ok(instances)
    }

    pub async fn upsert_service_runtime_node(
        &self,
        record: &ServiceRuntimeNodeRecord,
    ) -> Result<ServiceRuntimeNodeRecord> {
        sqlx::query(
            "INSERT INTO ai_service_runtime_nodes (node_id, service_kind, started_at_ms, last_seen_at_ms)
             VALUES ($1, $2, $3, $4)
             ON CONFLICT(node_id) DO UPDATE SET
                 service_kind = excluded.service_kind,
                 started_at_ms = excluded.started_at_ms,
                 last_seen_at_ms = excluded.last_seen_at_ms",
        )
        .bind(&record.node_id)
        .bind(&record.service_kind)
        .bind(record.started_at_ms as i64)
        .bind(record.last_seen_at_ms as i64)
        .execute(&self.pool)
        .await?;

        Ok(record.clone())
    }

    pub async fn list_service_runtime_nodes(&self) -> Result<Vec<ServiceRuntimeNodeRecord>> {
        let rows = sqlx::query_as::<_, (String, String, i64, i64)>(
            "SELECT node_id, service_kind, started_at_ms, last_seen_at_ms
             FROM ai_service_runtime_nodes
             ORDER BY node_id",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|(node_id, service_kind, started_at_ms, last_seen_at_ms)| {
                ServiceRuntimeNodeRecord {
                    node_id,
                    service_kind,
                    started_at_ms: started_at_ms as u64,
                    last_seen_at_ms: last_seen_at_ms as u64,
                }
            })
            .collect())
    }

    pub async fn insert_extension_runtime_rollout(
        &self,
        rollout: &ExtensionRuntimeRolloutRecord,
    ) -> Result<ExtensionRuntimeRolloutRecord> {
        sqlx::query(
            "INSERT INTO ai_extension_runtime_rollouts (
                rollout_id,
                scope,
                requested_extension_id,
                requested_instance_id,
                resolved_extension_id,
                created_by,
                created_at_ms,
                deadline_at_ms
             ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
             ON CONFLICT(rollout_id) DO UPDATE SET
                 scope = excluded.scope,
                 requested_extension_id = excluded.requested_extension_id,
                 requested_instance_id = excluded.requested_instance_id,
                 resolved_extension_id = excluded.resolved_extension_id,
                 created_by = excluded.created_by,
                 created_at_ms = excluded.created_at_ms,
                 deadline_at_ms = excluded.deadline_at_ms",
        )
        .bind(&rollout.rollout_id)
        .bind(&rollout.scope)
        .bind(&rollout.requested_extension_id)
        .bind(&rollout.requested_instance_id)
        .bind(&rollout.resolved_extension_id)
        .bind(&rollout.created_by)
        .bind(rollout.created_at_ms as i64)
        .bind(rollout.deadline_at_ms as i64)
        .execute(&self.pool)
        .await?;

        Ok(rollout.clone())
    }

    pub async fn find_extension_runtime_rollout(
        &self,
        rollout_id: &str,
    ) -> Result<Option<ExtensionRuntimeRolloutRecord>> {
        let row = sqlx::query_as::<
            _,
            (
                String,
                String,
                Option<String>,
                Option<String>,
                Option<String>,
                String,
                i64,
                i64,
            ),
        >(
            "SELECT rollout_id, scope, requested_extension_id, requested_instance_id, resolved_extension_id, created_by, created_at_ms, deadline_at_ms
             FROM ai_extension_runtime_rollouts
             WHERE rollout_id = $1",
        )
        .bind(rollout_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(
            |(
                rollout_id,
                scope,
                requested_extension_id,
                requested_instance_id,
                resolved_extension_id,
                created_by,
                created_at_ms,
                deadline_at_ms,
            )| ExtensionRuntimeRolloutRecord {
                rollout_id,
                scope,
                requested_extension_id,
                requested_instance_id,
                resolved_extension_id,
                created_by,
                created_at_ms: created_at_ms as u64,
                deadline_at_ms: deadline_at_ms as u64,
            },
        ))
    }

    pub async fn list_extension_runtime_rollouts(
        &self,
    ) -> Result<Vec<ExtensionRuntimeRolloutRecord>> {
        let rows = sqlx::query_as::<
            _,
            (
                String,
                String,
                Option<String>,
                Option<String>,
                Option<String>,
                String,
                i64,
                i64,
            ),
        >(
            "SELECT rollout_id, scope, requested_extension_id, requested_instance_id, resolved_extension_id, created_by, created_at_ms, deadline_at_ms
             FROM ai_extension_runtime_rollouts
             ORDER BY created_at_ms DESC, rollout_id",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(
                |(
                    rollout_id,
                    scope,
                    requested_extension_id,
                    requested_instance_id,
                    resolved_extension_id,
                    created_by,
                    created_at_ms,
                    deadline_at_ms,
                )| ExtensionRuntimeRolloutRecord {
                    rollout_id,
                    scope,
                    requested_extension_id,
                    requested_instance_id,
                    resolved_extension_id,
                    created_by,
                    created_at_ms: created_at_ms as u64,
                    deadline_at_ms: deadline_at_ms as u64,
                },
            )
            .collect())
    }

    pub async fn insert_extension_runtime_rollout_participant(
        &self,
        participant: &ExtensionRuntimeRolloutParticipantRecord,
    ) -> Result<ExtensionRuntimeRolloutParticipantRecord> {
        sqlx::query(
            "INSERT INTO ai_extension_runtime_rollout_participants (
                rollout_id,
                node_id,
                service_kind,
                status,
                message,
                updated_at_ms
             ) VALUES ($1, $2, $3, $4, $5, $6)
             ON CONFLICT(rollout_id, node_id) DO UPDATE SET
                 service_kind = excluded.service_kind,
                 status = excluded.status,
                 message = excluded.message,
                 updated_at_ms = excluded.updated_at_ms",
        )
        .bind(&participant.rollout_id)
        .bind(&participant.node_id)
        .bind(&participant.service_kind)
        .bind(&participant.status)
        .bind(&participant.message)
        .bind(participant.updated_at_ms as i64)
        .execute(&self.pool)
        .await?;

        Ok(participant.clone())
    }

    pub async fn list_extension_runtime_rollout_participants(
        &self,
        rollout_id: &str,
    ) -> Result<Vec<ExtensionRuntimeRolloutParticipantRecord>> {
        let rows = sqlx::query_as::<_, (String, String, String, String, Option<String>, i64)>(
            "SELECT rollout_id, node_id, service_kind, status, message, updated_at_ms
             FROM ai_extension_runtime_rollout_participants
             WHERE rollout_id = $1
             ORDER BY node_id",
        )
        .bind(rollout_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(
                |(rollout_id, node_id, service_kind, status, message, updated_at_ms)| {
                    ExtensionRuntimeRolloutParticipantRecord {
                        rollout_id,
                        node_id,
                        service_kind,
                        status,
                        message,
                        updated_at_ms: updated_at_ms as u64,
                    }
                },
            )
            .collect())
    }

    pub async fn list_pending_extension_runtime_rollout_participants_for_node(
        &self,
        node_id: &str,
    ) -> Result<Vec<ExtensionRuntimeRolloutParticipantRecord>> {
        let rows = sqlx::query_as::<_, (String, String, String, String, Option<String>, i64)>(
            "SELECT participants.rollout_id, participants.node_id, participants.service_kind, participants.status, participants.message, participants.updated_at_ms
             FROM ai_extension_runtime_rollout_participants AS participants
             INNER JOIN ai_extension_runtime_rollouts AS rollouts ON rollouts.rollout_id = participants.rollout_id
             WHERE participants.node_id = $1
               AND participants.status = 'pending'
             ORDER BY rollouts.created_at_ms, participants.rollout_id",
        )
        .bind(node_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(
                |(rollout_id, node_id, service_kind, status, message, updated_at_ms)| {
                    ExtensionRuntimeRolloutParticipantRecord {
                        rollout_id,
                        node_id,
                        service_kind,
                        status,
                        message,
                        updated_at_ms: updated_at_ms as u64,
                    }
                },
            )
            .collect())
    }

    pub async fn transition_extension_runtime_rollout_participant(
        &self,
        rollout_id: &str,
        node_id: &str,
        from_status: &str,
        to_status: &str,
        message: Option<&str>,
        updated_at_ms: u64,
    ) -> Result<bool> {
        let result = sqlx::query(
            "UPDATE ai_extension_runtime_rollout_participants
             SET status = $1, message = $2, updated_at_ms = $3
             WHERE rollout_id = $4 AND node_id = $5 AND status = $6",
        )
        .bind(to_status)
        .bind(message)
        .bind(updated_at_ms as i64)
        .bind(rollout_id)
        .bind(node_id)
        .bind(from_status)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() == 1)
    }

    pub async fn insert_standalone_config_rollout(
        &self,
        rollout: &StandaloneConfigRolloutRecord,
    ) -> Result<StandaloneConfigRolloutRecord> {
        sqlx::query(
            "INSERT INTO ai_standalone_config_rollouts (
                rollout_id,
                requested_service_kind,
                created_by,
                created_at_ms,
                deadline_at_ms
             ) VALUES ($1, $2, $3, $4, $5)
             ON CONFLICT(rollout_id) DO UPDATE SET
                 requested_service_kind = excluded.requested_service_kind,
                 created_by = excluded.created_by,
                 created_at_ms = excluded.created_at_ms,
                 deadline_at_ms = excluded.deadline_at_ms",
        )
        .bind(&rollout.rollout_id)
        .bind(&rollout.requested_service_kind)
        .bind(&rollout.created_by)
        .bind(rollout.created_at_ms as i64)
        .bind(rollout.deadline_at_ms as i64)
        .execute(&self.pool)
        .await?;

        Ok(rollout.clone())
    }

    pub async fn find_standalone_config_rollout(
        &self,
        rollout_id: &str,
    ) -> Result<Option<StandaloneConfigRolloutRecord>> {
        let row = sqlx::query_as::<_, (String, Option<String>, String, i64, i64)>(
            "SELECT rollout_id, requested_service_kind, created_by, created_at_ms, deadline_at_ms
             FROM ai_standalone_config_rollouts
             WHERE rollout_id = $1",
        )
        .bind(rollout_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(
            |(rollout_id, requested_service_kind, created_by, created_at_ms, deadline_at_ms)| {
                StandaloneConfigRolloutRecord {
                    rollout_id,
                    requested_service_kind,
                    created_by,
                    created_at_ms: created_at_ms as u64,
                    deadline_at_ms: deadline_at_ms as u64,
                }
            },
        ))
    }

    pub async fn list_standalone_config_rollouts(
        &self,
    ) -> Result<Vec<StandaloneConfigRolloutRecord>> {
        let rows = sqlx::query_as::<_, (String, Option<String>, String, i64, i64)>(
            "SELECT rollout_id, requested_service_kind, created_by, created_at_ms, deadline_at_ms
             FROM ai_standalone_config_rollouts
             ORDER BY created_at_ms DESC, rollout_id",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(
                |(
                    rollout_id,
                    requested_service_kind,
                    created_by,
                    created_at_ms,
                    deadline_at_ms,
                )| {
                    StandaloneConfigRolloutRecord {
                        rollout_id,
                        requested_service_kind,
                        created_by,
                        created_at_ms: created_at_ms as u64,
                        deadline_at_ms: deadline_at_ms as u64,
                    }
                },
            )
            .collect())
    }

    pub async fn insert_standalone_config_rollout_participant(
        &self,
        participant: &StandaloneConfigRolloutParticipantRecord,
    ) -> Result<StandaloneConfigRolloutParticipantRecord> {
        sqlx::query(
            "INSERT INTO ai_standalone_config_rollout_participants (
                rollout_id,
                node_id,
                service_kind,
                status,
                message,
                updated_at_ms
             ) VALUES ($1, $2, $3, $4, $5, $6)
             ON CONFLICT(rollout_id, node_id) DO UPDATE SET
                 service_kind = excluded.service_kind,
                 status = excluded.status,
                 message = excluded.message,
                 updated_at_ms = excluded.updated_at_ms",
        )
        .bind(&participant.rollout_id)
        .bind(&participant.node_id)
        .bind(&participant.service_kind)
        .bind(&participant.status)
        .bind(&participant.message)
        .bind(participant.updated_at_ms as i64)
        .execute(&self.pool)
        .await?;

        Ok(participant.clone())
    }

    pub async fn list_standalone_config_rollout_participants(
        &self,
        rollout_id: &str,
    ) -> Result<Vec<StandaloneConfigRolloutParticipantRecord>> {
        let rows = sqlx::query_as::<_, (String, String, String, String, Option<String>, i64)>(
            "SELECT rollout_id, node_id, service_kind, status, message, updated_at_ms
             FROM ai_standalone_config_rollout_participants
             WHERE rollout_id = $1
             ORDER BY node_id",
        )
        .bind(rollout_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(
                |(rollout_id, node_id, service_kind, status, message, updated_at_ms)| {
                    StandaloneConfigRolloutParticipantRecord {
                        rollout_id,
                        node_id,
                        service_kind,
                        status,
                        message,
                        updated_at_ms: updated_at_ms as u64,
                    }
                },
            )
            .collect())
    }

    pub async fn list_pending_standalone_config_rollout_participants_for_node(
        &self,
        node_id: &str,
    ) -> Result<Vec<StandaloneConfigRolloutParticipantRecord>> {
        let rows = sqlx::query_as::<_, (String, String, String, String, Option<String>, i64)>(
            "SELECT participants.rollout_id, participants.node_id, participants.service_kind, participants.status, participants.message, participants.updated_at_ms
             FROM ai_standalone_config_rollout_participants AS participants
             INNER JOIN ai_standalone_config_rollouts AS rollouts ON rollouts.rollout_id = participants.rollout_id
             WHERE participants.node_id = $1
               AND participants.status = 'pending'
             ORDER BY rollouts.created_at_ms, participants.rollout_id",
        )
        .bind(node_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(
                |(rollout_id, node_id, service_kind, status, message, updated_at_ms)| {
                    StandaloneConfigRolloutParticipantRecord {
                        rollout_id,
                        node_id,
                        service_kind,
                        status,
                        message,
                        updated_at_ms: updated_at_ms as u64,
                    }
                },
            )
            .collect())
    }

    pub async fn transition_standalone_config_rollout_participant(
        &self,
        rollout_id: &str,
        node_id: &str,
        from_status: &str,
        to_status: &str,
        message: Option<&str>,
        updated_at_ms: u64,
    ) -> Result<bool> {
        let result = sqlx::query(
            "UPDATE ai_standalone_config_rollout_participants
             SET status = $1, message = $2, updated_at_ms = $3
             WHERE rollout_id = $4 AND node_id = $5 AND status = $6",
        )
        .bind(to_status)
        .bind(message)
        .bind(updated_at_ms as i64)
        .bind(rollout_id)
        .bind(node_id)
        .bind(from_status)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() == 1)
    }
}

#[async_trait::async_trait]
impl AdminStore for PostgresAdminStore {
    fn dialect(&self) -> StorageDialect {
        StorageDialect::Postgres
    }

    fn identity_kernel(&self) -> Option<&dyn IdentityKernelStore> {
        Some(self)
    }

    fn account_kernel(&self) -> Option<&dyn AccountKernelStore> {
        Some(self)
    }

    async fn insert_channel(&self, channel: &Channel) -> Result<Channel> {
        PostgresAdminStore::insert_channel(self, channel).await
    }

    async fn list_channels(&self) -> Result<Vec<Channel>> {
        PostgresAdminStore::list_channels(self).await
    }

    async fn delete_channel(&self, channel_id: &str) -> Result<bool> {
        PostgresAdminStore::delete_channel(self, channel_id).await
    }

    async fn insert_provider(&self, provider: &ProxyProvider) -> Result<ProxyProvider> {
        PostgresAdminStore::insert_provider(self, provider).await
    }

    async fn list_providers(&self) -> Result<Vec<ProxyProvider>> {
        PostgresAdminStore::list_providers(self).await
    }

    async fn list_providers_for_model(&self, model: &str) -> Result<Vec<ProxyProvider>> {
        PostgresAdminStore::list_providers_for_model(self, model).await
    }

    async fn find_provider(&self, provider_id: &str) -> Result<Option<ProxyProvider>> {
        PostgresAdminStore::find_provider(self, provider_id).await
    }

    async fn delete_provider(&self, provider_id: &str) -> Result<bool> {
        PostgresAdminStore::delete_provider(self, provider_id).await
    }

    async fn insert_credential(
        &self,
        credential: &UpstreamCredential,
    ) -> Result<UpstreamCredential> {
        PostgresAdminStore::insert_credential(self, credential).await
    }

    async fn insert_encrypted_credential(
        &self,
        credential: &UpstreamCredential,
        envelope: &SecretEnvelope,
    ) -> Result<UpstreamCredential> {
        PostgresAdminStore::insert_encrypted_credential(self, credential, envelope).await
    }

    async fn list_credentials(&self) -> Result<Vec<UpstreamCredential>> {
        PostgresAdminStore::list_credentials(self).await
    }

    async fn list_credentials_for_tenant(
        &self,
        tenant_id: &str,
    ) -> Result<Vec<UpstreamCredential>> {
        PostgresAdminStore::list_credentials_for_tenant(self, tenant_id).await
    }

    async fn list_credentials_for_provider(
        &self,
        provider_id: &str,
    ) -> Result<Vec<UpstreamCredential>> {
        PostgresAdminStore::list_credentials_for_provider(self, provider_id).await
    }

    async fn find_credential(
        &self,
        tenant_id: &str,
        provider_id: &str,
        key_reference: &str,
    ) -> Result<Option<UpstreamCredential>> {
        PostgresAdminStore::find_credential(self, tenant_id, provider_id, key_reference).await
    }

    async fn find_credential_envelope(
        &self,
        tenant_id: &str,
        provider_id: &str,
        key_reference: &str,
    ) -> Result<Option<SecretEnvelope>> {
        PostgresAdminStore::find_credential_envelope(self, tenant_id, provider_id, key_reference)
            .await
    }

    async fn find_provider_credential(
        &self,
        tenant_id: &str,
        provider_id: &str,
    ) -> Result<Option<UpstreamCredential>> {
        PostgresAdminStore::find_provider_credential(self, tenant_id, provider_id).await
    }

    async fn delete_credential(
        &self,
        tenant_id: &str,
        provider_id: &str,
        key_reference: &str,
    ) -> Result<bool> {
        PostgresAdminStore::delete_credential(self, tenant_id, provider_id, key_reference).await
    }

    async fn insert_model(&self, model: &ModelCatalogEntry) -> Result<ModelCatalogEntry> {
        PostgresAdminStore::insert_model(self, model).await
    }

    async fn list_models(&self) -> Result<Vec<ModelCatalogEntry>> {
        PostgresAdminStore::list_models(self).await
    }

    async fn list_models_for_external_name(
        &self,
        external_name: &str,
    ) -> Result<Vec<ModelCatalogEntry>> {
        PostgresAdminStore::list_models_for_external_name(self, external_name).await
    }

    async fn find_any_model(&self) -> Result<Option<ModelCatalogEntry>> {
        PostgresAdminStore::find_any_model(self).await
    }

    async fn find_model(&self, external_name: &str) -> Result<Option<ModelCatalogEntry>> {
        PostgresAdminStore::find_model(self, external_name).await
    }

    async fn delete_model(&self, external_name: &str) -> Result<bool> {
        PostgresAdminStore::delete_model(self, external_name).await
    }

    async fn delete_model_variant(&self, external_name: &str, provider_id: &str) -> Result<bool> {
        PostgresAdminStore::delete_model_variant(self, external_name, provider_id).await
    }

    async fn insert_channel_model(
        &self,
        record: &ChannelModelRecord,
    ) -> Result<ChannelModelRecord> {
        PostgresAdminStore::insert_channel_model(self, record).await
    }

    async fn list_channel_models(&self) -> Result<Vec<ChannelModelRecord>> {
        PostgresAdminStore::list_channel_models(self).await
    }

    async fn delete_channel_model(&self, channel_id: &str, model_id: &str) -> Result<bool> {
        PostgresAdminStore::delete_channel_model(self, channel_id, model_id).await
    }

    async fn insert_model_price(&self, record: &ModelPriceRecord) -> Result<ModelPriceRecord> {
        PostgresAdminStore::insert_model_price(self, record).await
    }

    async fn list_model_prices(&self) -> Result<Vec<ModelPriceRecord>> {
        PostgresAdminStore::list_model_prices(self).await
    }

    async fn delete_model_price(
        &self,
        channel_id: &str,
        model_id: &str,
        proxy_provider_id: &str,
    ) -> Result<bool> {
        PostgresAdminStore::delete_model_price(self, channel_id, model_id, proxy_provider_id).await
    }

    async fn insert_routing_policy(&self, policy: &RoutingPolicy) -> Result<RoutingPolicy> {
        PostgresAdminStore::insert_routing_policy(self, policy).await
    }

    async fn list_routing_policies(&self) -> Result<Vec<RoutingPolicy>> {
        PostgresAdminStore::list_routing_policies(self).await
    }

    async fn insert_routing_profile(
        &self,
        profile: &RoutingProfileRecord,
    ) -> Result<RoutingProfileRecord> {
        PostgresAdminStore::insert_routing_profile(self, profile).await
    }

    async fn list_routing_profiles(&self) -> Result<Vec<RoutingProfileRecord>> {
        PostgresAdminStore::list_routing_profiles(self).await
    }

    async fn find_routing_profile(&self, profile_id: &str) -> Result<Option<RoutingProfileRecord>> {
        PostgresAdminStore::find_routing_profile(self, profile_id).await
    }

    async fn insert_compiled_routing_snapshot(
        &self,
        snapshot: &CompiledRoutingSnapshotRecord,
    ) -> Result<CompiledRoutingSnapshotRecord> {
        PostgresAdminStore::insert_compiled_routing_snapshot(self, snapshot).await
    }

    async fn list_compiled_routing_snapshots(&self) -> Result<Vec<CompiledRoutingSnapshotRecord>> {
        PostgresAdminStore::list_compiled_routing_snapshots(self).await
    }

    async fn insert_project_routing_preferences(
        &self,
        preferences: &ProjectRoutingPreferences,
    ) -> Result<ProjectRoutingPreferences> {
        PostgresAdminStore::insert_project_routing_preferences(self, preferences).await
    }

    async fn find_project_routing_preferences(
        &self,
        project_id: &str,
    ) -> Result<Option<ProjectRoutingPreferences>> {
        PostgresAdminStore::find_project_routing_preferences(self, project_id).await
    }

    async fn insert_routing_decision_log(
        &self,
        log: &RoutingDecisionLog,
    ) -> Result<RoutingDecisionLog> {
        PostgresAdminStore::insert_routing_decision_log(self, log).await
    }

    async fn list_routing_decision_logs(&self) -> Result<Vec<RoutingDecisionLog>> {
        PostgresAdminStore::list_routing_decision_logs(self).await
    }

    async fn list_routing_decision_logs_for_project(
        &self,
        project_id: &str,
    ) -> Result<Vec<RoutingDecisionLog>> {
        PostgresAdminStore::list_routing_decision_logs_for_project(self, project_id).await
    }

    async fn find_latest_routing_decision_log_for_project(
        &self,
        project_id: &str,
    ) -> Result<Option<RoutingDecisionLog>> {
        PostgresAdminStore::find_latest_routing_decision_log_for_project(self, project_id).await
    }

    async fn insert_provider_health_snapshot(
        &self,
        snapshot: &ProviderHealthSnapshot,
    ) -> Result<ProviderHealthSnapshot> {
        PostgresAdminStore::insert_provider_health_snapshot(self, snapshot).await
    }

    async fn list_provider_health_snapshots(&self) -> Result<Vec<ProviderHealthSnapshot>> {
        PostgresAdminStore::list_provider_health_snapshots(self).await
    }

    async fn insert_usage_record(&self, record: &UsageRecord) -> Result<UsageRecord> {
        PostgresAdminStore::insert_usage_record(self, record).await
    }

    async fn list_usage_records(&self) -> Result<Vec<UsageRecord>> {
        PostgresAdminStore::list_usage_records(self).await
    }

    async fn list_usage_records_for_project(&self, project_id: &str) -> Result<Vec<UsageRecord>> {
        PostgresAdminStore::list_usage_records_for_project(self, project_id).await
    }

    async fn find_latest_usage_record_for_project(
        &self,
        project_id: &str,
    ) -> Result<Option<UsageRecord>> {
        PostgresAdminStore::find_latest_usage_record_for_project(self, project_id).await
    }

    async fn insert_billing_event(&self, event: &BillingEventRecord) -> Result<BillingEventRecord> {
        PostgresAdminStore::insert_billing_event(self, event).await
    }

    async fn list_billing_events(&self) -> Result<Vec<BillingEventRecord>> {
        PostgresAdminStore::list_billing_events(self).await
    }

    async fn insert_ledger_entry(&self, entry: &LedgerEntry) -> Result<LedgerEntry> {
        PostgresAdminStore::insert_ledger_entry(self, entry).await
    }

    async fn list_ledger_entries(&self) -> Result<Vec<LedgerEntry>> {
        PostgresAdminStore::list_ledger_entries(self).await
    }

    async fn list_ledger_entries_for_project(&self, project_id: &str) -> Result<Vec<LedgerEntry>> {
        PostgresAdminStore::list_ledger_entries_for_project(self, project_id).await
    }

    async fn insert_quota_policy(&self, policy: &QuotaPolicy) -> Result<QuotaPolicy> {
        PostgresAdminStore::insert_quota_policy(self, policy).await
    }

    async fn list_quota_policies(&self) -> Result<Vec<QuotaPolicy>> {
        PostgresAdminStore::list_quota_policies(self).await
    }

    async fn list_quota_policies_for_project(&self, project_id: &str) -> Result<Vec<QuotaPolicy>> {
        PostgresAdminStore::list_quota_policies_for_project(self, project_id).await
    }

    async fn insert_rate_limit_policy(&self, policy: &RateLimitPolicy) -> Result<RateLimitPolicy> {
        PostgresAdminStore::insert_rate_limit_policy(self, policy).await
    }

    async fn list_rate_limit_policies(&self) -> Result<Vec<RateLimitPolicy>> {
        PostgresAdminStore::list_rate_limit_policies(self).await
    }

    async fn list_rate_limit_window_snapshots(&self) -> Result<Vec<RateLimitWindowSnapshot>> {
        PostgresAdminStore::list_rate_limit_window_snapshots(self).await
    }

    async fn list_rate_limit_policies_for_project(
        &self,
        project_id: &str,
    ) -> Result<Vec<RateLimitPolicy>> {
        PostgresAdminStore::list_rate_limit_policies_for_project(self, project_id).await
    }

    async fn check_and_consume_rate_limit(
        &self,
        policy_id: &str,
        requested_requests: u64,
        limit_requests: u64,
        window_seconds: u64,
        now_ms: u64,
    ) -> Result<RateLimitCheckResult> {
        PostgresAdminStore::check_and_consume_rate_limit(
            self,
            policy_id,
            requested_requests,
            limit_requests,
            window_seconds,
            now_ms,
        )
        .await
    }

    async fn insert_tenant(&self, tenant: &Tenant) -> Result<Tenant> {
        PostgresAdminStore::insert_tenant(self, tenant).await
    }

    async fn list_tenants(&self) -> Result<Vec<Tenant>> {
        PostgresAdminStore::list_tenants(self).await
    }

    async fn find_tenant(&self, tenant_id: &str) -> Result<Option<Tenant>> {
        PostgresAdminStore::find_tenant(self, tenant_id).await
    }

    async fn delete_tenant(&self, tenant_id: &str) -> Result<bool> {
        PostgresAdminStore::delete_tenant(self, tenant_id).await
    }

    async fn insert_project(&self, project: &Project) -> Result<Project> {
        PostgresAdminStore::insert_project(self, project).await
    }

    async fn list_projects(&self) -> Result<Vec<Project>> {
        PostgresAdminStore::list_projects(self).await
    }

    async fn find_project(&self, project_id: &str) -> Result<Option<Project>> {
        PostgresAdminStore::find_project(self, project_id).await
    }

    async fn delete_project(&self, project_id: &str) -> Result<bool> {
        PostgresAdminStore::delete_project(self, project_id).await
    }

    async fn insert_coupon(&self, coupon: &CouponCampaign) -> Result<CouponCampaign> {
        PostgresAdminStore::insert_coupon(self, coupon).await
    }

    async fn list_coupons(&self) -> Result<Vec<CouponCampaign>> {
        PostgresAdminStore::list_coupons(self).await
    }

    async fn list_active_coupons(&self) -> Result<Vec<CouponCampaign>> {
        PostgresAdminStore::list_active_coupons(self).await
    }

    async fn find_coupon(&self, coupon_id: &str) -> Result<Option<CouponCampaign>> {
        PostgresAdminStore::find_coupon(self, coupon_id).await
    }

    async fn delete_coupon(&self, coupon_id: &str) -> Result<bool> {
        PostgresAdminStore::delete_coupon(self, coupon_id).await
    }

    async fn insert_commerce_order(
        &self,
        order: &CommerceOrderRecord,
    ) -> Result<CommerceOrderRecord> {
        PostgresAdminStore::insert_commerce_order(self, order).await
    }

    async fn list_commerce_orders(&self) -> Result<Vec<CommerceOrderRecord>> {
        PostgresAdminStore::list_commerce_orders(self).await
    }

    async fn list_commerce_orders_for_project(
        &self,
        project_id: &str,
    ) -> Result<Vec<CommerceOrderRecord>> {
        PostgresAdminStore::list_commerce_orders_for_project(self, project_id).await
    }

    async fn upsert_project_membership(
        &self,
        membership: &ProjectMembershipRecord,
    ) -> Result<ProjectMembershipRecord> {
        PostgresAdminStore::upsert_project_membership(self, membership).await
    }

    async fn find_project_membership(
        &self,
        project_id: &str,
    ) -> Result<Option<ProjectMembershipRecord>> {
        PostgresAdminStore::find_project_membership(self, project_id).await
    }

    async fn insert_coupon_template_record(
        &self,
        record: &CouponTemplateRecord,
    ) -> Result<CouponTemplateRecord> {
        PostgresAdminStore::insert_coupon_template_record(self, record).await
    }

    async fn list_coupon_template_records(&self) -> Result<Vec<CouponTemplateRecord>> {
        PostgresAdminStore::list_coupon_template_records(self).await
    }

    async fn find_coupon_template_record(
        &self,
        coupon_template_id: u64,
    ) -> Result<Option<CouponTemplateRecord>> {
        PostgresAdminStore::find_coupon_template_record(self, coupon_template_id).await
    }

    async fn insert_coupon_benefit_rule_record(
        &self,
        record: &CouponBenefitRuleRecord,
    ) -> Result<CouponBenefitRuleRecord> {
        PostgresAdminStore::insert_coupon_benefit_rule_record(self, record).await
    }

    async fn list_coupon_benefit_rule_records(&self) -> Result<Vec<CouponBenefitRuleRecord>> {
        PostgresAdminStore::list_coupon_benefit_rule_records(self).await
    }

    async fn insert_marketing_campaign_record(
        &self,
        record: &MarketingCampaignRecord,
    ) -> Result<MarketingCampaignRecord> {
        PostgresAdminStore::insert_marketing_campaign_record(self, record).await
    }

    async fn list_marketing_campaign_records(&self) -> Result<Vec<MarketingCampaignRecord>> {
        PostgresAdminStore::list_marketing_campaign_records(self).await
    }

    async fn find_marketing_campaign_record(
        &self,
        marketing_campaign_id: u64,
    ) -> Result<Option<MarketingCampaignRecord>> {
        PostgresAdminStore::find_marketing_campaign_record(self, marketing_campaign_id).await
    }

    async fn insert_coupon_code_batch_record(
        &self,
        record: &CouponCodeBatchRecord,
    ) -> Result<CouponCodeBatchRecord> {
        PostgresAdminStore::insert_coupon_code_batch_record(self, record).await
    }

    async fn list_coupon_code_batch_records(&self) -> Result<Vec<CouponCodeBatchRecord>> {
        PostgresAdminStore::list_coupon_code_batch_records(self).await
    }

    async fn insert_coupon_code_record(
        &self,
        record: &CouponCodeRecord,
    ) -> Result<CouponCodeRecord> {
        PostgresAdminStore::insert_coupon_code_record(self, record).await
    }

    async fn list_coupon_code_records(&self) -> Result<Vec<CouponCodeRecord>> {
        PostgresAdminStore::list_coupon_code_records(self).await
    }

    async fn find_coupon_code_record(
        &self,
        coupon_code_id: u64,
    ) -> Result<Option<CouponCodeRecord>> {
        PostgresAdminStore::find_coupon_code_record(self, coupon_code_id).await
    }

    async fn find_coupon_code_record_by_lookup_hash(
        &self,
        code_lookup_hash: &str,
    ) -> Result<Option<CouponCodeRecord>> {
        PostgresAdminStore::find_coupon_code_record_by_lookup_hash(self, code_lookup_hash).await
    }

    async fn list_coupon_code_records_for_subject(
        &self,
        claim_subject_type: &str,
        claim_subject_id: &str,
    ) -> Result<Vec<CouponCodeRecord>> {
        PostgresAdminStore::list_coupon_code_records_for_subject(
            self,
            claim_subject_type,
            claim_subject_id,
        )
        .await
    }

    async fn insert_coupon_claim_record(
        &self,
        record: &CouponClaimRecord,
    ) -> Result<CouponClaimRecord> {
        PostgresAdminStore::insert_coupon_claim_record(self, record).await
    }

    async fn list_coupon_claim_records(&self) -> Result<Vec<CouponClaimRecord>> {
        PostgresAdminStore::list_coupon_claim_records(self).await
    }

    async fn insert_coupon_redemption_record(
        &self,
        record: &CouponRedemptionRecord,
    ) -> Result<CouponRedemptionRecord> {
        PostgresAdminStore::insert_coupon_redemption_record(self, record).await
    }

    async fn list_coupon_redemption_records(&self) -> Result<Vec<CouponRedemptionRecord>> {
        PostgresAdminStore::list_coupon_redemption_records(self).await
    }

    async fn find_coupon_redemption_record_by_idempotency_key(
        &self,
        idempotency_key: &str,
    ) -> Result<Option<CouponRedemptionRecord>> {
        PostgresAdminStore::find_coupon_redemption_record_by_idempotency_key(self, idempotency_key)
            .await
    }

    async fn insert_referral_program_record(
        &self,
        record: &ReferralProgramRecord,
    ) -> Result<ReferralProgramRecord> {
        PostgresAdminStore::insert_referral_program_record(self, record).await
    }

    async fn list_referral_program_records(&self) -> Result<Vec<ReferralProgramRecord>> {
        PostgresAdminStore::list_referral_program_records(self).await
    }

    async fn insert_referral_invite_record(
        &self,
        record: &ReferralInviteRecord,
    ) -> Result<ReferralInviteRecord> {
        PostgresAdminStore::insert_referral_invite_record(self, record).await
    }

    async fn list_referral_invite_records(&self) -> Result<Vec<ReferralInviteRecord>> {
        PostgresAdminStore::list_referral_invite_records(self).await
    }

    async fn insert_marketing_attribution_touch_record(
        &self,
        record: &MarketingAttributionTouchRecord,
    ) -> Result<MarketingAttributionTouchRecord> {
        PostgresAdminStore::insert_marketing_attribution_touch_record(self, record).await
    }

    async fn list_marketing_attribution_touch_records(
        &self,
    ) -> Result<Vec<MarketingAttributionTouchRecord>> {
        PostgresAdminStore::list_marketing_attribution_touch_records(self).await
    }

    async fn insert_portal_user(&self, user: &PortalUserRecord) -> Result<PortalUserRecord> {
        PostgresAdminStore::insert_portal_user(self, user).await
    }

    async fn list_portal_users(&self) -> Result<Vec<PortalUserRecord>> {
        PostgresAdminStore::list_portal_users(self).await
    }

    async fn find_portal_user_by_email(&self, email: &str) -> Result<Option<PortalUserRecord>> {
        PostgresAdminStore::find_portal_user_by_email(self, email).await
    }

    async fn find_portal_user_by_id(&self, user_id: &str) -> Result<Option<PortalUserRecord>> {
        PostgresAdminStore::find_portal_user_by_id(self, user_id).await
    }

    async fn delete_portal_user(&self, user_id: &str) -> Result<bool> {
        PostgresAdminStore::delete_portal_user(self, user_id).await
    }

    async fn insert_portal_workspace_membership(
        &self,
        membership: &PortalWorkspaceMembershipRecord,
    ) -> Result<PortalWorkspaceMembershipRecord> {
        PostgresAdminStore::insert_portal_workspace_membership(self, membership).await
    }

    async fn list_portal_workspace_memberships_for_user(
        &self,
        user_id: &str,
    ) -> Result<Vec<PortalWorkspaceMembershipRecord>> {
        PostgresAdminStore::list_portal_workspace_memberships_for_user(self, user_id).await
    }

    async fn find_portal_workspace_membership(
        &self,
        user_id: &str,
        tenant_id: &str,
        project_id: &str,
    ) -> Result<Option<PortalWorkspaceMembershipRecord>> {
        PostgresAdminStore::find_portal_workspace_membership(self, user_id, tenant_id, project_id)
            .await
    }

    async fn insert_payment_order_record(
        &self,
        record: &PaymentOrderRecord,
    ) -> Result<PaymentOrderRecord> {
        PostgresAdminStore::insert_payment_order_record(self, record).await
    }

    async fn find_payment_order_record(
        &self,
        payment_order_id: &str,
    ) -> Result<Option<PaymentOrderRecord>> {
        PostgresAdminStore::find_payment_order_record(self, payment_order_id).await
    }

    async fn find_payment_order_record_by_commerce_order_id(
        &self,
        commerce_order_id: &str,
    ) -> Result<Option<PaymentOrderRecord>> {
        PostgresAdminStore::find_payment_order_record_by_commerce_order_id(self, commerce_order_id)
            .await
    }

    async fn find_payment_order_record_by_provider_reference(
        &self,
        provider: &str,
        provider_reference_id: &str,
    ) -> Result<Option<PaymentOrderRecord>> {
        PostgresAdminStore::find_payment_order_record_by_provider_reference(
            self,
            provider,
            provider_reference_id,
        )
        .await
    }

    async fn insert_payment_attempt_record(
        &self,
        record: &PaymentAttemptRecord,
    ) -> Result<PaymentAttemptRecord> {
        PostgresAdminStore::insert_payment_attempt_record(self, record).await
    }

    async fn find_payment_attempt_record(
        &self,
        payment_attempt_id: &str,
    ) -> Result<Option<PaymentAttemptRecord>> {
        PostgresAdminStore::find_payment_attempt_record(self, payment_attempt_id).await
    }

    async fn find_payment_attempt_record_by_provider_reference(
        &self,
        provider: &str,
        provider_attempt_id: &str,
    ) -> Result<Option<PaymentAttemptRecord>> {
        PostgresAdminStore::find_payment_attempt_record_by_provider_reference(
            self,
            provider,
            provider_attempt_id,
        )
        .await
    }

    async fn list_payment_attempt_records_for_payment_order(
        &self,
        payment_order_id: &str,
    ) -> Result<Vec<PaymentAttemptRecord>> {
        PostgresAdminStore::list_payment_attempt_records_for_payment_order(self, payment_order_id)
            .await
    }

    async fn insert_refund_record(&self, record: &RefundRecord) -> Result<RefundRecord> {
        PostgresAdminStore::insert_refund_record(self, record).await
    }

    async fn find_refund_record(&self, refund_id: &str) -> Result<Option<RefundRecord>> {
        PostgresAdminStore::find_refund_record(self, refund_id).await
    }

    async fn find_refund_record_by_provider_reference(
        &self,
        provider: &str,
        provider_refund_id: &str,
    ) -> Result<Option<RefundRecord>> {
        PostgresAdminStore::find_refund_record_by_provider_reference(
            self,
            provider,
            provider_refund_id,
        )
        .await
    }

    async fn list_refund_records_for_payment_order(
        &self,
        payment_order_id: &str,
    ) -> Result<Vec<RefundRecord>> {
        PostgresAdminStore::list_refund_records_for_payment_order(self, payment_order_id).await
    }

    async fn insert_dispute_record(&self, record: &DisputeRecord) -> Result<DisputeRecord> {
        PostgresAdminStore::insert_dispute_record(self, record).await
    }

    async fn find_dispute_record(&self, dispute_id: &str) -> Result<Option<DisputeRecord>> {
        PostgresAdminStore::find_dispute_record(self, dispute_id).await
    }

    async fn find_dispute_record_by_provider_reference(
        &self,
        provider: &str,
        provider_dispute_id: &str,
    ) -> Result<Option<DisputeRecord>> {
        PostgresAdminStore::find_dispute_record_by_provider_reference(
            self,
            provider,
            provider_dispute_id,
        )
        .await
    }

    async fn list_dispute_records_for_payment_order(
        &self,
        payment_order_id: &str,
    ) -> Result<Vec<DisputeRecord>> {
        PostgresAdminStore::list_dispute_records_for_payment_order(self, payment_order_id).await
    }

    async fn insert_payment_webhook_event_record(
        &self,
        record: &PaymentWebhookEventRecord,
    ) -> Result<PaymentWebhookEventRecord> {
        PostgresAdminStore::insert_payment_webhook_event_record(self, record).await
    }

    async fn find_payment_webhook_event_record(
        &self,
        provider: &str,
        provider_event_id: &str,
    ) -> Result<Option<PaymentWebhookEventRecord>> {
        PostgresAdminStore::find_payment_webhook_event_record(self, provider, provider_event_id)
            .await
    }

    async fn insert_admin_user(&self, user: &AdminUserRecord) -> Result<AdminUserRecord> {
        PostgresAdminStore::insert_admin_user(self, user).await
    }

    async fn list_admin_users(&self) -> Result<Vec<AdminUserRecord>> {
        PostgresAdminStore::list_admin_users(self).await
    }

    async fn find_admin_user_by_email(&self, email: &str) -> Result<Option<AdminUserRecord>> {
        PostgresAdminStore::find_admin_user_by_email(self, email).await
    }

    async fn find_admin_user_by_id(&self, user_id: &str) -> Result<Option<AdminUserRecord>> {
        PostgresAdminStore::find_admin_user_by_id(self, user_id).await
    }

    async fn delete_admin_user(&self, user_id: &str) -> Result<bool> {
        PostgresAdminStore::delete_admin_user(self, user_id).await
    }

    async fn insert_admin_audit_event(
        &self,
        event: &AdminAuditEventRecord,
    ) -> Result<AdminAuditEventRecord> {
        PostgresAdminStore::insert_admin_audit_event(self, event).await
    }

    async fn list_admin_audit_events(&self) -> Result<Vec<AdminAuditEventRecord>> {
        PostgresAdminStore::list_admin_audit_events(self).await
    }

    async fn insert_gateway_api_key(
        &self,
        record: &GatewayApiKeyRecord,
    ) -> Result<GatewayApiKeyRecord> {
        PostgresAdminStore::insert_gateway_api_key(self, record).await
    }

    async fn list_gateway_api_keys(&self) -> Result<Vec<GatewayApiKeyRecord>> {
        PostgresAdminStore::list_gateway_api_keys(self).await
    }

    async fn find_gateway_api_key(&self, hashed_key: &str) -> Result<Option<GatewayApiKeyRecord>> {
        PostgresAdminStore::find_gateway_api_key(self, hashed_key).await
    }

    async fn delete_gateway_api_key(&self, hashed_key: &str) -> Result<bool> {
        PostgresAdminStore::delete_gateway_api_key(self, hashed_key).await
    }

    async fn insert_api_key_group(&self, record: &ApiKeyGroupRecord) -> Result<ApiKeyGroupRecord> {
        PostgresAdminStore::insert_api_key_group(self, record).await
    }

    async fn list_api_key_groups(&self) -> Result<Vec<ApiKeyGroupRecord>> {
        PostgresAdminStore::list_api_key_groups(self).await
    }

    async fn find_api_key_group(&self, group_id: &str) -> Result<Option<ApiKeyGroupRecord>> {
        PostgresAdminStore::find_api_key_group(self, group_id).await
    }

    async fn delete_api_key_group(&self, group_id: &str) -> Result<bool> {
        PostgresAdminStore::delete_api_key_group(self, group_id).await
    }

    async fn insert_extension_installation(
        &self,
        installation: &ExtensionInstallation,
    ) -> Result<ExtensionInstallation> {
        PostgresAdminStore::insert_extension_installation(self, installation).await
    }

    async fn list_extension_installations(&self) -> Result<Vec<ExtensionInstallation>> {
        PostgresAdminStore::list_extension_installations(self).await
    }

    async fn insert_extension_instance(
        &self,
        instance: &ExtensionInstance,
    ) -> Result<ExtensionInstance> {
        PostgresAdminStore::insert_extension_instance(self, instance).await
    }

    async fn list_extension_instances(&self) -> Result<Vec<ExtensionInstance>> {
        PostgresAdminStore::list_extension_instances(self).await
    }

    async fn upsert_service_runtime_node(
        &self,
        record: &ServiceRuntimeNodeRecord,
    ) -> Result<ServiceRuntimeNodeRecord> {
        PostgresAdminStore::upsert_service_runtime_node(self, record).await
    }

    async fn list_service_runtime_nodes(&self) -> Result<Vec<ServiceRuntimeNodeRecord>> {
        PostgresAdminStore::list_service_runtime_nodes(self).await
    }

    async fn insert_extension_runtime_rollout(
        &self,
        rollout: &ExtensionRuntimeRolloutRecord,
    ) -> Result<ExtensionRuntimeRolloutRecord> {
        PostgresAdminStore::insert_extension_runtime_rollout(self, rollout).await
    }

    async fn find_extension_runtime_rollout(
        &self,
        rollout_id: &str,
    ) -> Result<Option<ExtensionRuntimeRolloutRecord>> {
        PostgresAdminStore::find_extension_runtime_rollout(self, rollout_id).await
    }

    async fn list_extension_runtime_rollouts(&self) -> Result<Vec<ExtensionRuntimeRolloutRecord>> {
        PostgresAdminStore::list_extension_runtime_rollouts(self).await
    }

    async fn insert_extension_runtime_rollout_participant(
        &self,
        participant: &ExtensionRuntimeRolloutParticipantRecord,
    ) -> Result<ExtensionRuntimeRolloutParticipantRecord> {
        PostgresAdminStore::insert_extension_runtime_rollout_participant(self, participant).await
    }

    async fn list_extension_runtime_rollout_participants(
        &self,
        rollout_id: &str,
    ) -> Result<Vec<ExtensionRuntimeRolloutParticipantRecord>> {
        PostgresAdminStore::list_extension_runtime_rollout_participants(self, rollout_id).await
    }

    async fn list_pending_extension_runtime_rollout_participants_for_node(
        &self,
        node_id: &str,
    ) -> Result<Vec<ExtensionRuntimeRolloutParticipantRecord>> {
        PostgresAdminStore::list_pending_extension_runtime_rollout_participants_for_node(
            self, node_id,
        )
        .await
    }

    async fn transition_extension_runtime_rollout_participant(
        &self,
        rollout_id: &str,
        node_id: &str,
        from_status: &str,
        to_status: &str,
        message: Option<&str>,
        updated_at_ms: u64,
    ) -> Result<bool> {
        PostgresAdminStore::transition_extension_runtime_rollout_participant(
            self,
            rollout_id,
            node_id,
            from_status,
            to_status,
            message,
            updated_at_ms,
        )
        .await
    }

    async fn insert_standalone_config_rollout(
        &self,
        rollout: &StandaloneConfigRolloutRecord,
    ) -> Result<StandaloneConfigRolloutRecord> {
        PostgresAdminStore::insert_standalone_config_rollout(self, rollout).await
    }

    async fn find_standalone_config_rollout(
        &self,
        rollout_id: &str,
    ) -> Result<Option<StandaloneConfigRolloutRecord>> {
        PostgresAdminStore::find_standalone_config_rollout(self, rollout_id).await
    }

    async fn list_standalone_config_rollouts(&self) -> Result<Vec<StandaloneConfigRolloutRecord>> {
        PostgresAdminStore::list_standalone_config_rollouts(self).await
    }

    async fn insert_standalone_config_rollout_participant(
        &self,
        participant: &StandaloneConfigRolloutParticipantRecord,
    ) -> Result<StandaloneConfigRolloutParticipantRecord> {
        PostgresAdminStore::insert_standalone_config_rollout_participant(self, participant).await
    }

    async fn list_standalone_config_rollout_participants(
        &self,
        rollout_id: &str,
    ) -> Result<Vec<StandaloneConfigRolloutParticipantRecord>> {
        PostgresAdminStore::list_standalone_config_rollout_participants(self, rollout_id).await
    }

    async fn list_pending_standalone_config_rollout_participants_for_node(
        &self,
        node_id: &str,
    ) -> Result<Vec<StandaloneConfigRolloutParticipantRecord>> {
        PostgresAdminStore::list_pending_standalone_config_rollout_participants_for_node(
            self, node_id,
        )
        .await
    }

    async fn transition_standalone_config_rollout_participant(
        &self,
        rollout_id: &str,
        node_id: &str,
        from_status: &str,
        to_status: &str,
        message: Option<&str>,
        updated_at_ms: u64,
    ) -> Result<bool> {
        PostgresAdminStore::transition_standalone_config_rollout_participant(
            self,
            rollout_id,
            node_id,
            from_status,
            to_status,
            message,
            updated_at_ms,
        )
        .await
    }
}

#[async_trait]
impl IdentityKernelStore for PostgresAdminStore {
    async fn insert_identity_user_record(
        &self,
        record: &IdentityUserRecord,
    ) -> Result<IdentityUserRecord> {
        upsert_identity_user_record_executor(&self.pool, record).await?;
        Ok(record.clone())
    }

    async fn list_identity_user_records(&self) -> Result<Vec<IdentityUserRecord>> {
        let query = postgres_bind_query(
            "SELECT user_id, tenant_id, organization_id, external_user_ref, username,
                    display_name, email, status, created_at_ms, updated_at_ms
             FROM ai_user
             ORDER BY created_at_ms DESC, user_id",
        );
        let rows = sqlx::query(&query).fetch_all(&self.pool).await?;
        rows.into_iter().map(decode_identity_user_row).collect()
    }

    async fn find_identity_user_record(&self, user_id: u64) -> Result<Option<IdentityUserRecord>> {
        let query = postgres_bind_query(
            "SELECT user_id, tenant_id, organization_id, external_user_ref, username,
                    display_name, email, status, created_at_ms, updated_at_ms
             FROM ai_user
             WHERE user_id = ?",
        );
        let row = sqlx::query(&query)
            .bind(i64::try_from(user_id)?)
            .fetch_optional(&self.pool)
            .await?;
        row.map(decode_identity_user_row).transpose()
    }

    async fn insert_canonical_api_key_record(
        &self,
        record: &CanonicalApiKeyRecord,
    ) -> Result<CanonicalApiKeyRecord> {
        upsert_canonical_api_key_record_executor(&self.pool, record).await?;
        Ok(record.clone())
    }

    async fn find_canonical_api_key_record_by_hash(
        &self,
        key_hash: &str,
    ) -> Result<Option<CanonicalApiKeyRecord>> {
        let query = postgres_bind_query(
            "SELECT api_key_id, tenant_id, organization_id, user_id, key_prefix, key_hash,
                    display_name, status, expires_at_ms, last_used_at_ms,
                    rotated_from_api_key_id, created_at_ms, updated_at_ms
             FROM ai_api_key
             WHERE key_hash = ?",
        );
        let row = sqlx::query(&query)
            .bind(key_hash)
            .fetch_optional(&self.pool)
            .await?;
        row.map(decode_canonical_api_key_row).transpose()
    }

    async fn insert_identity_binding_record(
        &self,
        record: &IdentityBindingRecord,
    ) -> Result<IdentityBindingRecord> {
        upsert_identity_binding_record_executor(&self.pool, record).await?;
        Ok(record.clone())
    }

    async fn find_identity_binding_record(
        &self,
        binding_type: &str,
        issuer: Option<&str>,
        subject: Option<&str>,
    ) -> Result<Option<IdentityBindingRecord>> {
        let query = postgres_bind_query(
            "SELECT identity_binding_id, tenant_id, organization_id, user_id, binding_type,
                    issuer, subject, platform, owner, external_ref, status, created_at_ms, updated_at_ms
             FROM ai_identity_binding
             WHERE binding_type = ?
               AND ((issuer IS NULL AND ? IS NULL) OR issuer = ?)
               AND ((subject IS NULL AND ? IS NULL) OR subject = ?)
             ORDER BY updated_at_ms DESC, identity_binding_id DESC
             LIMIT 1",
        );
        let row = sqlx::query(&query)
            .bind(binding_type)
            .bind(issuer)
            .bind(issuer)
            .bind(subject)
            .bind(subject)
            .fetch_optional(&self.pool)
            .await?;
        row.map(decode_identity_binding_row).transpose()
    }

    async fn list_identity_binding_records(&self) -> Result<Vec<IdentityBindingRecord>> {
        let query = postgres_bind_query(
            "SELECT identity_binding_id, tenant_id, organization_id, user_id, binding_type,
                    issuer, subject, platform, owner, external_ref, status, created_at_ms, updated_at_ms
             FROM ai_identity_binding
             ORDER BY updated_at_ms DESC, identity_binding_id DESC",
        );
        let rows = sqlx::query(&query).fetch_all(&self.pool).await?;
        rows.into_iter().map(decode_identity_binding_row).collect()
    }
}

#[async_trait]
impl AccountKernelStore for PostgresAdminStore {
    async fn insert_account_record(&self, record: &AccountRecord) -> Result<AccountRecord> {
        upsert_account_record_executor(&self.pool, record).await?;
        Ok(record.clone())
    }

    async fn list_account_records(&self) -> Result<Vec<AccountRecord>> {
        let query = postgres_bind_query(
            "SELECT account_id, tenant_id, organization_id, user_id, account_type,
                    currency_code, credit_unit_code, status, allow_overdraft, overdraft_limit,
                    created_at_ms, updated_at_ms
             FROM ai_account
             ORDER BY created_at_ms DESC, account_id",
        );
        let rows = sqlx::query(&query).fetch_all(&self.pool).await?;
        rows.into_iter().map(decode_account_record_row).collect()
    }

    async fn find_account_record(&self, account_id: u64) -> Result<Option<AccountRecord>> {
        let query = postgres_bind_query(
            "SELECT account_id, tenant_id, organization_id, user_id, account_type,
                    currency_code, credit_unit_code, status, allow_overdraft, overdraft_limit,
                    created_at_ms, updated_at_ms
             FROM ai_account
             WHERE account_id = ?",
        );
        let row = sqlx::query(&query)
            .bind(i64::try_from(account_id)?)
            .fetch_optional(&self.pool)
            .await?;
        row.map(decode_account_record_row).transpose()
    }

    async fn find_account_record_by_owner(
        &self,
        tenant_id: u64,
        organization_id: u64,
        user_id: u64,
        account_type: AccountType,
    ) -> Result<Option<AccountRecord>> {
        let query = postgres_bind_query(
            "SELECT account_id, tenant_id, organization_id, user_id, account_type,
                    currency_code, credit_unit_code, status, allow_overdraft, overdraft_limit,
                    created_at_ms, updated_at_ms
             FROM ai_account
             WHERE tenant_id = ?
               AND organization_id = ?
               AND user_id = ?
               AND account_type = ?",
        );
        let row = sqlx::query(&query)
            .bind(i64::try_from(tenant_id)?)
            .bind(i64::try_from(organization_id)?)
            .bind(i64::try_from(user_id)?)
            .bind(account_type_as_str(account_type))
            .fetch_optional(&self.pool)
            .await?;
        row.map(decode_account_record_row).transpose()
    }

    async fn insert_account_benefit_lot(
        &self,
        record: &AccountBenefitLotRecord,
    ) -> Result<AccountBenefitLotRecord> {
        upsert_account_benefit_lot_executor(&self.pool, record).await?;
        Ok(record.clone())
    }

    async fn list_account_benefit_lots(&self) -> Result<Vec<AccountBenefitLotRecord>> {
        let query = postgres_bind_query(
            "SELECT lot_id, tenant_id, organization_id, account_id, user_id, benefit_type,
                    source_type, source_id, scope_json, original_quantity, remaining_quantity,
                    held_quantity, priority, acquired_unit_cost, issued_at_ms, expires_at_ms,
                    status, created_at_ms, updated_at_ms
             FROM ai_account_benefit_lot
             ORDER BY created_at_ms DESC, lot_id",
        );
        let rows = sqlx::query(&query).fetch_all(&self.pool).await?;
        rows.into_iter()
            .map(decode_account_benefit_lot_row)
            .collect()
    }

    async fn insert_account_hold(&self, record: &AccountHoldRecord) -> Result<AccountHoldRecord> {
        upsert_account_hold_executor(&self.pool, record).await?;
        Ok(record.clone())
    }

    async fn list_account_holds(&self) -> Result<Vec<AccountHoldRecord>> {
        let query = postgres_bind_query(
            "SELECT hold_id, tenant_id, organization_id, account_id, user_id, request_id,
                    hold_status, estimated_quantity, captured_quantity, released_quantity,
                    expires_at_ms, created_at_ms, updated_at_ms
             FROM ai_account_hold
             ORDER BY created_at_ms DESC, hold_id",
        );
        let rows = sqlx::query(&query).fetch_all(&self.pool).await?;
        rows.into_iter().map(decode_account_hold_row).collect()
    }

    async fn insert_account_hold_allocation(
        &self,
        record: &AccountHoldAllocationRecord,
    ) -> Result<AccountHoldAllocationRecord> {
        upsert_account_hold_allocation_executor(&self.pool, record).await?;
        Ok(record.clone())
    }

    async fn list_account_hold_allocations(&self) -> Result<Vec<AccountHoldAllocationRecord>> {
        let query = postgres_bind_query(
            "SELECT hold_allocation_id, tenant_id, organization_id, hold_id, lot_id,
                    allocated_quantity, captured_quantity, released_quantity,
                    created_at_ms, updated_at_ms
             FROM ai_account_hold_allocation
             ORDER BY created_at_ms DESC, hold_allocation_id",
        );
        let rows = sqlx::query(&query).fetch_all(&self.pool).await?;
        rows.into_iter()
            .map(decode_account_hold_allocation_row)
            .collect()
    }

    async fn insert_account_ledger_entry_record(
        &self,
        record: &AccountLedgerEntryRecord,
    ) -> Result<AccountLedgerEntryRecord> {
        upsert_account_ledger_entry_executor(&self.pool, record).await?;
        Ok(record.clone())
    }

    async fn list_account_ledger_entry_records(&self) -> Result<Vec<AccountLedgerEntryRecord>> {
        let query = postgres_bind_query(
            "SELECT ledger_entry_id, tenant_id, organization_id, account_id, user_id,
                    request_id, hold_id, entry_type, benefit_type, quantity, amount, created_at_ms
             FROM ai_account_ledger_entry
             ORDER BY created_at_ms DESC, ledger_entry_id",
        );
        let rows = sqlx::query(&query).fetch_all(&self.pool).await?;
        rows.into_iter()
            .map(decode_account_ledger_entry_row)
            .collect()
    }

    async fn insert_account_ledger_allocation(
        &self,
        record: &AccountLedgerAllocationRecord,
    ) -> Result<AccountLedgerAllocationRecord> {
        upsert_account_ledger_allocation_executor(&self.pool, record).await?;
        Ok(record.clone())
    }

    async fn list_account_ledger_allocations(&self) -> Result<Vec<AccountLedgerAllocationRecord>> {
        let query = postgres_bind_query(
            "SELECT ledger_allocation_id, tenant_id, organization_id, ledger_entry_id, lot_id,
                    quantity_delta, created_at_ms
             FROM ai_account_ledger_allocation
             ORDER BY created_at_ms DESC, ledger_allocation_id",
        );
        let rows = sqlx::query(&query).fetch_all(&self.pool).await?;
        rows.into_iter()
            .map(decode_account_ledger_allocation_row)
            .collect()
    }

    async fn insert_request_meter_fact(
        &self,
        record: &RequestMeterFactRecord,
    ) -> Result<RequestMeterFactRecord> {
        upsert_request_meter_fact_executor(&self.pool, record).await?;
        Ok(record.clone())
    }

    async fn list_request_meter_facts(&self) -> Result<Vec<RequestMeterFactRecord>> {
        let query = postgres_bind_query(
            "SELECT request_id, tenant_id, organization_id, user_id, account_id, api_key_id,
                    api_key_hash, auth_type, jwt_subject, platform, owner, request_trace_id,
                    gateway_request_ref, upstream_request_ref, protocol_family, capability_code,
                    channel_code, model_code, provider_code, request_status, usage_capture_status,
                    cost_pricing_plan_id, retail_pricing_plan_id, estimated_credit_hold,
                    actual_credit_charge, actual_provider_cost, started_at_ms, finished_at_ms,
                    created_at_ms, updated_at_ms
             FROM ai_request_meter_fact
             ORDER BY created_at_ms DESC, request_id",
        );
        let rows = sqlx::query(&query).fetch_all(&self.pool).await?;
        rows.into_iter()
            .map(decode_request_meter_fact_row)
            .collect()
    }

    async fn insert_request_meter_metric(
        &self,
        record: &RequestMeterMetricRecord,
    ) -> Result<RequestMeterMetricRecord> {
        upsert_request_meter_metric_executor(&self.pool, record).await?;
        Ok(record.clone())
    }

    async fn list_request_meter_metrics(&self) -> Result<Vec<RequestMeterMetricRecord>> {
        let query = postgres_bind_query(
            "SELECT request_metric_id, tenant_id, organization_id, request_id, metric_code,
                    quantity, provider_field, source_kind, capture_stage, is_billable, captured_at_ms
             FROM ai_request_meter_metric
             ORDER BY captured_at_ms DESC, request_metric_id",
        );
        let rows = sqlx::query(&query).fetch_all(&self.pool).await?;
        rows.into_iter()
            .map(decode_request_meter_metric_row)
            .collect()
    }

    async fn insert_pricing_plan_record(
        &self,
        record: &PricingPlanRecord,
    ) -> Result<PricingPlanRecord> {
        let query = postgres_bind_query(
            "INSERT INTO ai_pricing_plan (
                pricing_plan_id, tenant_id, organization_id, plan_code, plan_version,
                display_name, currency_code, credit_unit_code, status, created_at_ms, updated_at_ms
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(pricing_plan_id) DO UPDATE SET
                tenant_id = excluded.tenant_id,
                organization_id = excluded.organization_id,
                plan_code = excluded.plan_code,
                plan_version = excluded.plan_version,
                display_name = excluded.display_name,
                currency_code = excluded.currency_code,
                credit_unit_code = excluded.credit_unit_code,
                status = excluded.status,
                created_at_ms = excluded.created_at_ms,
                updated_at_ms = excluded.updated_at_ms",
        );
        sqlx::query(&query)
            .bind(i64::try_from(record.pricing_plan_id)?)
            .bind(i64::try_from(record.tenant_id)?)
            .bind(i64::try_from(record.organization_id)?)
            .bind(&record.plan_code)
            .bind(i64::try_from(record.plan_version)?)
            .bind(&record.display_name)
            .bind(&record.currency_code)
            .bind(&record.credit_unit_code)
            .bind(&record.status)
            .bind(i64::try_from(record.created_at_ms)?)
            .bind(i64::try_from(record.updated_at_ms)?)
            .execute(&self.pool)
            .await?;
        Ok(record.clone())
    }

    async fn list_pricing_plan_records(&self) -> Result<Vec<PricingPlanRecord>> {
        let query = postgres_bind_query(
            "SELECT pricing_plan_id, tenant_id, organization_id, plan_code, plan_version,
                    display_name, currency_code, credit_unit_code, status, created_at_ms, updated_at_ms
             FROM ai_pricing_plan
             ORDER BY updated_at_ms DESC, pricing_plan_id",
        );
        let rows = sqlx::query(&query).fetch_all(&self.pool).await?;
        rows.into_iter().map(decode_pricing_plan_row).collect()
    }

    async fn insert_pricing_rate_record(
        &self,
        record: &PricingRateRecord,
    ) -> Result<PricingRateRecord> {
        let query = postgres_bind_query(
            "INSERT INTO ai_pricing_rate (
                pricing_rate_id, tenant_id, organization_id, pricing_plan_id, metric_code,
                model_code, provider_code, quantity_step, unit_price, created_at_ms
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(pricing_rate_id) DO UPDATE SET
                tenant_id = excluded.tenant_id,
                organization_id = excluded.organization_id,
                pricing_plan_id = excluded.pricing_plan_id,
                metric_code = excluded.metric_code,
                model_code = excluded.model_code,
                provider_code = excluded.provider_code,
                quantity_step = excluded.quantity_step,
                unit_price = excluded.unit_price,
                created_at_ms = excluded.created_at_ms",
        );
        sqlx::query(&query)
            .bind(i64::try_from(record.pricing_rate_id)?)
            .bind(i64::try_from(record.tenant_id)?)
            .bind(i64::try_from(record.organization_id)?)
            .bind(i64::try_from(record.pricing_plan_id)?)
            .bind(&record.metric_code)
            .bind(&record.model_code)
            .bind(&record.provider_code)
            .bind(record.quantity_step)
            .bind(record.unit_price)
            .bind(i64::try_from(record.created_at_ms)?)
            .execute(&self.pool)
            .await?;
        Ok(record.clone())
    }

    async fn list_pricing_rate_records(&self) -> Result<Vec<PricingRateRecord>> {
        let query = postgres_bind_query(
            "SELECT pricing_rate_id, tenant_id, organization_id, pricing_plan_id, metric_code,
                    model_code, provider_code, quantity_step, unit_price, created_at_ms
             FROM ai_pricing_rate
             ORDER BY created_at_ms DESC, pricing_rate_id",
        );
        let rows = sqlx::query(&query).fetch_all(&self.pool).await?;
        rows.into_iter().map(decode_pricing_rate_row).collect()
    }

    async fn insert_request_settlement_record(
        &self,
        record: &RequestSettlementRecord,
    ) -> Result<RequestSettlementRecord> {
        upsert_request_settlement_executor(&self.pool, record).await?;
        Ok(record.clone())
    }

    async fn list_request_settlement_records(&self) -> Result<Vec<RequestSettlementRecord>> {
        let query = postgres_bind_query(
            "SELECT request_settlement_id, tenant_id, organization_id, request_id, account_id,
                    user_id, hold_id, settlement_status, estimated_credit_hold,
                    released_credit_amount, captured_credit_amount, provider_cost_amount,
                    retail_charge_amount, shortfall_amount, refunded_amount, settled_at_ms,
                    created_at_ms, updated_at_ms
             FROM ai_request_settlement
             ORDER BY updated_at_ms DESC, request_settlement_id",
        );
        let rows = sqlx::query(&query).fetch_all(&self.pool).await?;
        rows.into_iter()
            .map(decode_request_settlement_row)
            .collect()
    }

    async fn commit_account_kernel_batch(&self, batch: &AccountKernelCommandBatch) -> Result<()> {
        if batch.is_empty() {
            return Ok(());
        }

        let mut tx = self.pool.begin().await?;

        for record in &batch.account_records {
            upsert_account_record_executor(&mut *tx, record).await?;
        }
        for record in &batch.benefit_lot_records {
            upsert_account_benefit_lot_executor(&mut *tx, record).await?;
        }
        for record in &batch.hold_records {
            upsert_account_hold_executor(&mut *tx, record).await?;
        }
        for record in &batch.hold_allocation_records {
            upsert_account_hold_allocation_executor(&mut *tx, record).await?;
        }
        for record in &batch.ledger_entry_records {
            upsert_account_ledger_entry_executor(&mut *tx, record).await?;
        }
        for record in &batch.ledger_allocation_records {
            upsert_account_ledger_allocation_executor(&mut *tx, record).await?;
        }
        for record in &batch.request_meter_fact_records {
            upsert_request_meter_fact_executor(&mut *tx, record).await?;
        }
        for record in &batch.request_meter_metric_records {
            upsert_request_meter_metric_executor(&mut *tx, record).await?;
        }
        for record in &batch.request_settlement_records {
            upsert_request_settlement_executor(&mut *tx, record).await?;
        }

        tx.commit().await?;
        Ok(())
    }
}

fn postgres_bind_query(query: &str) -> String {
    let mut rewritten = String::with_capacity(query.len() + 16);
    let mut bind_index = 1_u32;
    for ch in query.chars() {
        if ch == '?' {
            rewritten.push('$');
            rewritten.push_str(&bind_index.to_string());
            bind_index += 1;
        } else {
            rewritten.push(ch);
        }
    }
    rewritten
}

fn account_type_as_str(value: AccountType) -> &'static str {
    match value {
        AccountType::Primary => "primary",
        AccountType::Grant => "grant",
        AccountType::Postpaid => "postpaid",
    }
}

fn parse_account_type(value: &str) -> Result<AccountType> {
    match value {
        "primary" => Ok(AccountType::Primary),
        "grant" => Ok(AccountType::Grant),
        "postpaid" => Ok(AccountType::Postpaid),
        other => Err(anyhow::anyhow!("unknown account_type: {other}")),
    }
}

fn account_status_as_str(value: AccountStatus) -> &'static str {
    match value {
        AccountStatus::Active => "active",
        AccountStatus::Suspended => "suspended",
        AccountStatus::Closed => "closed",
    }
}

fn parse_account_status(value: &str) -> Result<AccountStatus> {
    match value {
        "active" => Ok(AccountStatus::Active),
        "suspended" => Ok(AccountStatus::Suspended),
        "closed" => Ok(AccountStatus::Closed),
        other => Err(anyhow::anyhow!("unknown account_status: {other}")),
    }
}

fn account_benefit_type_as_str(value: AccountBenefitType) -> &'static str {
    match value {
        AccountBenefitType::CashCredit => "cash_credit",
        AccountBenefitType::PromoCredit => "promo_credit",
        AccountBenefitType::RequestAllowance => "request_allowance",
        AccountBenefitType::TokenAllowance => "token_allowance",
        AccountBenefitType::ImageAllowance => "image_allowance",
        AccountBenefitType::AudioAllowance => "audio_allowance",
        AccountBenefitType::VideoAllowance => "video_allowance",
        AccountBenefitType::MusicAllowance => "music_allowance",
    }
}

fn parse_account_benefit_type(value: &str) -> Result<AccountBenefitType> {
    match value {
        "cash_credit" => Ok(AccountBenefitType::CashCredit),
        "promo_credit" => Ok(AccountBenefitType::PromoCredit),
        "request_allowance" => Ok(AccountBenefitType::RequestAllowance),
        "token_allowance" => Ok(AccountBenefitType::TokenAllowance),
        "image_allowance" => Ok(AccountBenefitType::ImageAllowance),
        "audio_allowance" => Ok(AccountBenefitType::AudioAllowance),
        "video_allowance" => Ok(AccountBenefitType::VideoAllowance),
        "music_allowance" => Ok(AccountBenefitType::MusicAllowance),
        other => Err(anyhow::anyhow!("unknown account_benefit_type: {other}")),
    }
}

fn account_benefit_source_type_as_str(value: AccountBenefitSourceType) -> &'static str {
    match value {
        AccountBenefitSourceType::Recharge => "recharge",
        AccountBenefitSourceType::Coupon => "coupon",
        AccountBenefitSourceType::Grant => "grant",
        AccountBenefitSourceType::Order => "order",
        AccountBenefitSourceType::ManualAdjustment => "manual_adjustment",
    }
}

fn parse_account_benefit_source_type(value: &str) -> Result<AccountBenefitSourceType> {
    match value {
        "recharge" => Ok(AccountBenefitSourceType::Recharge),
        "coupon" => Ok(AccountBenefitSourceType::Coupon),
        "grant" => Ok(AccountBenefitSourceType::Grant),
        "order" => Ok(AccountBenefitSourceType::Order),
        "manual_adjustment" => Ok(AccountBenefitSourceType::ManualAdjustment),
        other => Err(anyhow::anyhow!(
            "unknown account_benefit_source_type: {other}"
        )),
    }
}

fn account_benefit_lot_status_as_str(value: AccountBenefitLotStatus) -> &'static str {
    match value {
        AccountBenefitLotStatus::Active => "active",
        AccountBenefitLotStatus::Exhausted => "exhausted",
        AccountBenefitLotStatus::Expired => "expired",
        AccountBenefitLotStatus::Disabled => "disabled",
    }
}

fn parse_account_benefit_lot_status(value: &str) -> Result<AccountBenefitLotStatus> {
    match value {
        "active" => Ok(AccountBenefitLotStatus::Active),
        "exhausted" => Ok(AccountBenefitLotStatus::Exhausted),
        "expired" => Ok(AccountBenefitLotStatus::Expired),
        "disabled" => Ok(AccountBenefitLotStatus::Disabled),
        other => Err(anyhow::anyhow!(
            "unknown account_benefit_lot_status: {other}"
        )),
    }
}

fn account_hold_status_as_str(value: AccountHoldStatus) -> &'static str {
    match value {
        AccountHoldStatus::Held => "held",
        AccountHoldStatus::Captured => "captured",
        AccountHoldStatus::PartiallyReleased => "partially_released",
        AccountHoldStatus::Released => "released",
        AccountHoldStatus::Expired => "expired",
        AccountHoldStatus::Failed => "failed",
    }
}

fn parse_account_hold_status(value: &str) -> Result<AccountHoldStatus> {
    match value {
        "held" => Ok(AccountHoldStatus::Held),
        "captured" => Ok(AccountHoldStatus::Captured),
        "partially_released" => Ok(AccountHoldStatus::PartiallyReleased),
        "released" => Ok(AccountHoldStatus::Released),
        "expired" => Ok(AccountHoldStatus::Expired),
        "failed" => Ok(AccountHoldStatus::Failed),
        other => Err(anyhow::anyhow!("unknown account_hold_status: {other}")),
    }
}

fn account_ledger_entry_type_as_str(value: AccountLedgerEntryType) -> &'static str {
    match value {
        AccountLedgerEntryType::HoldCreate => "hold_create",
        AccountLedgerEntryType::HoldRelease => "hold_release",
        AccountLedgerEntryType::SettlementCapture => "settlement_capture",
        AccountLedgerEntryType::GrantIssue => "grant_issue",
        AccountLedgerEntryType::ManualAdjustment => "manual_adjustment",
        AccountLedgerEntryType::Refund => "refund",
    }
}

fn parse_account_ledger_entry_type(value: &str) -> Result<AccountLedgerEntryType> {
    match value {
        "hold_create" => Ok(AccountLedgerEntryType::HoldCreate),
        "hold_release" => Ok(AccountLedgerEntryType::HoldRelease),
        "settlement_capture" => Ok(AccountLedgerEntryType::SettlementCapture),
        "grant_issue" => Ok(AccountLedgerEntryType::GrantIssue),
        "manual_adjustment" => Ok(AccountLedgerEntryType::ManualAdjustment),
        "refund" => Ok(AccountLedgerEntryType::Refund),
        other => Err(anyhow::anyhow!(
            "unknown account_ledger_entry_type: {other}"
        )),
    }
}

fn request_status_as_str(value: RequestStatus) -> &'static str {
    match value {
        RequestStatus::Pending => "pending",
        RequestStatus::Running => "running",
        RequestStatus::Succeeded => "succeeded",
        RequestStatus::Failed => "failed",
        RequestStatus::Cancelled => "cancelled",
    }
}

fn parse_request_status(value: &str) -> Result<RequestStatus> {
    match value {
        "pending" => Ok(RequestStatus::Pending),
        "running" => Ok(RequestStatus::Running),
        "succeeded" => Ok(RequestStatus::Succeeded),
        "failed" => Ok(RequestStatus::Failed),
        "cancelled" => Ok(RequestStatus::Cancelled),
        other => Err(anyhow::anyhow!("unknown request_status: {other}")),
    }
}

fn usage_capture_status_as_str(value: UsageCaptureStatus) -> &'static str {
    match value {
        UsageCaptureStatus::Pending => "pending",
        UsageCaptureStatus::Estimated => "estimated",
        UsageCaptureStatus::Captured => "captured",
        UsageCaptureStatus::Reconciled => "reconciled",
        UsageCaptureStatus::Failed => "failed",
    }
}

fn parse_usage_capture_status(value: &str) -> Result<UsageCaptureStatus> {
    match value {
        "pending" => Ok(UsageCaptureStatus::Pending),
        "estimated" => Ok(UsageCaptureStatus::Estimated),
        "captured" => Ok(UsageCaptureStatus::Captured),
        "reconciled" => Ok(UsageCaptureStatus::Reconciled),
        "failed" => Ok(UsageCaptureStatus::Failed),
        other => Err(anyhow::anyhow!("unknown usage_capture_status: {other}")),
    }
}

fn request_settlement_status_as_str(value: RequestSettlementStatus) -> &'static str {
    match value {
        RequestSettlementStatus::Pending => "pending",
        RequestSettlementStatus::Captured => "captured",
        RequestSettlementStatus::PartiallyReleased => "partially_released",
        RequestSettlementStatus::Released => "released",
        RequestSettlementStatus::Refunded => "refunded",
        RequestSettlementStatus::Failed => "failed",
    }
}

fn parse_request_settlement_status(value: &str) -> Result<RequestSettlementStatus> {
    match value {
        "pending" => Ok(RequestSettlementStatus::Pending),
        "captured" => Ok(RequestSettlementStatus::Captured),
        "partially_released" => Ok(RequestSettlementStatus::PartiallyReleased),
        "released" => Ok(RequestSettlementStatus::Released),
        "refunded" => Ok(RequestSettlementStatus::Refunded),
        "failed" => Ok(RequestSettlementStatus::Failed),
        other => Err(anyhow::anyhow!(
            "unknown request_settlement_status: {other}"
        )),
    }
}

async fn upsert_identity_user_record_executor<'e, E>(
    executor: E,
    record: &IdentityUserRecord,
) -> Result<()>
where
    E: Executor<'e, Database = Postgres>,
{
    let query = postgres_bind_query(
        "INSERT INTO ai_user (
            user_id, tenant_id, organization_id, external_user_ref, username,
            display_name, email, status, created_at_ms, updated_at_ms
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT(user_id) DO UPDATE SET
            tenant_id = excluded.tenant_id,
            organization_id = excluded.organization_id,
            external_user_ref = excluded.external_user_ref,
            username = excluded.username,
            display_name = excluded.display_name,
            email = excluded.email,
            status = excluded.status,
            created_at_ms = excluded.created_at_ms,
            updated_at_ms = excluded.updated_at_ms",
    );
    sqlx::query(&query)
        .bind(i64::try_from(record.user_id)?)
        .bind(i64::try_from(record.tenant_id)?)
        .bind(i64::try_from(record.organization_id)?)
        .bind(&record.external_user_ref)
        .bind(&record.username)
        .bind(&record.display_name)
        .bind(&record.email)
        .bind(&record.status)
        .bind(i64::try_from(record.created_at_ms)?)
        .bind(i64::try_from(record.updated_at_ms)?)
        .execute(executor)
        .await?;
    Ok(())
}

async fn upsert_canonical_api_key_record_executor<'e, E>(
    executor: E,
    record: &CanonicalApiKeyRecord,
) -> Result<()>
where
    E: Executor<'e, Database = Postgres>,
{
    let query = postgres_bind_query(
        "INSERT INTO ai_api_key (
            api_key_id, tenant_id, organization_id, user_id, key_prefix, key_hash,
            display_name, status, expires_at_ms, last_used_at_ms, rotated_from_api_key_id,
            created_at_ms, updated_at_ms
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT(api_key_id) DO UPDATE SET
            tenant_id = excluded.tenant_id,
            organization_id = excluded.organization_id,
            user_id = excluded.user_id,
            key_prefix = excluded.key_prefix,
            key_hash = excluded.key_hash,
            display_name = excluded.display_name,
            status = excluded.status,
            expires_at_ms = excluded.expires_at_ms,
            last_used_at_ms = excluded.last_used_at_ms,
            rotated_from_api_key_id = excluded.rotated_from_api_key_id,
            created_at_ms = excluded.created_at_ms,
            updated_at_ms = excluded.updated_at_ms",
    );
    sqlx::query(&query)
        .bind(i64::try_from(record.api_key_id)?)
        .bind(i64::try_from(record.tenant_id)?)
        .bind(i64::try_from(record.organization_id)?)
        .bind(i64::try_from(record.user_id)?)
        .bind(&record.key_prefix)
        .bind(&record.key_hash)
        .bind(&record.display_name)
        .bind(&record.status)
        .bind(record.expires_at_ms.map(i64::try_from).transpose()?)
        .bind(record.last_used_at_ms.map(i64::try_from).transpose()?)
        .bind(
            record
                .rotated_from_api_key_id
                .map(i64::try_from)
                .transpose()?,
        )
        .bind(i64::try_from(record.created_at_ms)?)
        .bind(i64::try_from(record.updated_at_ms)?)
        .execute(executor)
        .await?;
    Ok(())
}

async fn upsert_identity_binding_record_executor<'e, E>(
    executor: E,
    record: &IdentityBindingRecord,
) -> Result<()>
where
    E: Executor<'e, Database = Postgres>,
{
    let query = postgres_bind_query(
        "INSERT INTO ai_identity_binding (
            identity_binding_id, tenant_id, organization_id, user_id, binding_type,
            issuer, subject, platform, owner, external_ref, status, created_at_ms, updated_at_ms
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT(identity_binding_id) DO UPDATE SET
            tenant_id = excluded.tenant_id,
            organization_id = excluded.organization_id,
            user_id = excluded.user_id,
            binding_type = excluded.binding_type,
            issuer = excluded.issuer,
            subject = excluded.subject,
            platform = excluded.platform,
            owner = excluded.owner,
            external_ref = excluded.external_ref,
            status = excluded.status,
            created_at_ms = excluded.created_at_ms,
            updated_at_ms = excluded.updated_at_ms",
    );
    sqlx::query(&query)
        .bind(i64::try_from(record.identity_binding_id)?)
        .bind(i64::try_from(record.tenant_id)?)
        .bind(i64::try_from(record.organization_id)?)
        .bind(i64::try_from(record.user_id)?)
        .bind(&record.binding_type)
        .bind(&record.issuer)
        .bind(&record.subject)
        .bind(&record.platform)
        .bind(&record.owner)
        .bind(&record.external_ref)
        .bind(&record.status)
        .bind(i64::try_from(record.created_at_ms)?)
        .bind(i64::try_from(record.updated_at_ms)?)
        .execute(executor)
        .await?;
    Ok(())
}

async fn upsert_account_record_executor<'e, E>(executor: E, record: &AccountRecord) -> Result<()>
where
    E: Executor<'e, Database = Postgres>,
{
    let query = postgres_bind_query(
        "INSERT INTO ai_account (
            account_id, tenant_id, organization_id, user_id, account_type,
            currency_code, credit_unit_code, status, allow_overdraft, overdraft_limit,
            created_at_ms, updated_at_ms
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT(account_id) DO UPDATE SET
            tenant_id = excluded.tenant_id,
            organization_id = excluded.organization_id,
            user_id = excluded.user_id,
            account_type = excluded.account_type,
            currency_code = excluded.currency_code,
            credit_unit_code = excluded.credit_unit_code,
            status = excluded.status,
            allow_overdraft = excluded.allow_overdraft,
            overdraft_limit = excluded.overdraft_limit,
            created_at_ms = excluded.created_at_ms,
            updated_at_ms = excluded.updated_at_ms",
    );
    sqlx::query(&query)
        .bind(i64::try_from(record.account_id)?)
        .bind(i64::try_from(record.tenant_id)?)
        .bind(i64::try_from(record.organization_id)?)
        .bind(i64::try_from(record.user_id)?)
        .bind(account_type_as_str(record.account_type))
        .bind(&record.currency_code)
        .bind(&record.credit_unit_code)
        .bind(account_status_as_str(record.status))
        .bind(record.allow_overdraft)
        .bind(record.overdraft_limit)
        .bind(i64::try_from(record.created_at_ms)?)
        .bind(i64::try_from(record.updated_at_ms)?)
        .execute(executor)
        .await?;
    Ok(())
}

async fn upsert_account_benefit_lot_executor<'e, E>(
    executor: E,
    record: &AccountBenefitLotRecord,
) -> Result<()>
where
    E: Executor<'e, Database = Postgres>,
{
    let query = postgres_bind_query(
        "INSERT INTO ai_account_benefit_lot (
            lot_id, tenant_id, organization_id, account_id, user_id, benefit_type,
            source_type, source_id, scope_json, original_quantity, remaining_quantity,
            held_quantity, priority, acquired_unit_cost, issued_at_ms, expires_at_ms, status,
            created_at_ms, updated_at_ms
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT(lot_id) DO UPDATE SET
            tenant_id = excluded.tenant_id,
            organization_id = excluded.organization_id,
            account_id = excluded.account_id,
            user_id = excluded.user_id,
            benefit_type = excluded.benefit_type,
            source_type = excluded.source_type,
            source_id = excluded.source_id,
            scope_json = excluded.scope_json,
            original_quantity = excluded.original_quantity,
            remaining_quantity = excluded.remaining_quantity,
            held_quantity = excluded.held_quantity,
            priority = excluded.priority,
            acquired_unit_cost = excluded.acquired_unit_cost,
            issued_at_ms = excluded.issued_at_ms,
            expires_at_ms = excluded.expires_at_ms,
            status = excluded.status,
            created_at_ms = excluded.created_at_ms,
            updated_at_ms = excluded.updated_at_ms",
    );
    sqlx::query(&query)
        .bind(i64::try_from(record.lot_id)?)
        .bind(i64::try_from(record.tenant_id)?)
        .bind(i64::try_from(record.organization_id)?)
        .bind(i64::try_from(record.account_id)?)
        .bind(i64::try_from(record.user_id)?)
        .bind(account_benefit_type_as_str(record.benefit_type))
        .bind(account_benefit_source_type_as_str(record.source_type))
        .bind(record.source_id.map(i64::try_from).transpose()?)
        .bind(&record.scope_json)
        .bind(record.original_quantity)
        .bind(record.remaining_quantity)
        .bind(record.held_quantity)
        .bind(record.priority)
        .bind(record.acquired_unit_cost)
        .bind(i64::try_from(record.issued_at_ms)?)
        .bind(record.expires_at_ms.map(i64::try_from).transpose()?)
        .bind(account_benefit_lot_status_as_str(record.status))
        .bind(i64::try_from(record.created_at_ms)?)
        .bind(i64::try_from(record.updated_at_ms)?)
        .execute(executor)
        .await?;
    Ok(())
}

async fn upsert_account_hold_executor<'e, E>(executor: E, record: &AccountHoldRecord) -> Result<()>
where
    E: Executor<'e, Database = Postgres>,
{
    let query = postgres_bind_query(
        "INSERT INTO ai_account_hold (
            hold_id, tenant_id, organization_id, account_id, user_id, request_id,
            hold_status, estimated_quantity, captured_quantity, released_quantity,
            expires_at_ms, created_at_ms, updated_at_ms
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT(hold_id) DO UPDATE SET
            tenant_id = excluded.tenant_id,
            organization_id = excluded.organization_id,
            account_id = excluded.account_id,
            user_id = excluded.user_id,
            request_id = excluded.request_id,
            hold_status = excluded.hold_status,
            estimated_quantity = excluded.estimated_quantity,
            captured_quantity = excluded.captured_quantity,
            released_quantity = excluded.released_quantity,
            expires_at_ms = excluded.expires_at_ms,
            created_at_ms = excluded.created_at_ms,
            updated_at_ms = excluded.updated_at_ms",
    );
    sqlx::query(&query)
        .bind(i64::try_from(record.hold_id)?)
        .bind(i64::try_from(record.tenant_id)?)
        .bind(i64::try_from(record.organization_id)?)
        .bind(i64::try_from(record.account_id)?)
        .bind(i64::try_from(record.user_id)?)
        .bind(i64::try_from(record.request_id)?)
        .bind(account_hold_status_as_str(record.status))
        .bind(record.estimated_quantity)
        .bind(record.captured_quantity)
        .bind(record.released_quantity)
        .bind(i64::try_from(record.expires_at_ms)?)
        .bind(i64::try_from(record.created_at_ms)?)
        .bind(i64::try_from(record.updated_at_ms)?)
        .execute(executor)
        .await?;
    Ok(())
}

async fn upsert_account_hold_allocation_executor<'e, E>(
    executor: E,
    record: &AccountHoldAllocationRecord,
) -> Result<()>
where
    E: Executor<'e, Database = Postgres>,
{
    let query = postgres_bind_query(
        "INSERT INTO ai_account_hold_allocation (
            hold_allocation_id, tenant_id, organization_id, hold_id, lot_id,
            allocated_quantity, captured_quantity, released_quantity,
            created_at_ms, updated_at_ms
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT(hold_allocation_id) DO UPDATE SET
            tenant_id = excluded.tenant_id,
            organization_id = excluded.organization_id,
            hold_id = excluded.hold_id,
            lot_id = excluded.lot_id,
            allocated_quantity = excluded.allocated_quantity,
            captured_quantity = excluded.captured_quantity,
            released_quantity = excluded.released_quantity,
            created_at_ms = excluded.created_at_ms,
            updated_at_ms = excluded.updated_at_ms",
    );
    sqlx::query(&query)
        .bind(i64::try_from(record.hold_allocation_id)?)
        .bind(i64::try_from(record.tenant_id)?)
        .bind(i64::try_from(record.organization_id)?)
        .bind(i64::try_from(record.hold_id)?)
        .bind(i64::try_from(record.lot_id)?)
        .bind(record.allocated_quantity)
        .bind(record.captured_quantity)
        .bind(record.released_quantity)
        .bind(i64::try_from(record.created_at_ms)?)
        .bind(i64::try_from(record.updated_at_ms)?)
        .execute(executor)
        .await?;
    Ok(())
}

async fn upsert_account_ledger_entry_executor<'e, E>(
    executor: E,
    record: &AccountLedgerEntryRecord,
) -> Result<()>
where
    E: Executor<'e, Database = Postgres>,
{
    let query = postgres_bind_query(
        "INSERT INTO ai_account_ledger_entry (
            ledger_entry_id, tenant_id, organization_id, account_id, user_id,
            request_id, hold_id, entry_type, benefit_type, quantity, amount, created_at_ms
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT(ledger_entry_id) DO UPDATE SET
            tenant_id = excluded.tenant_id,
            organization_id = excluded.organization_id,
            account_id = excluded.account_id,
            user_id = excluded.user_id,
            request_id = excluded.request_id,
            hold_id = excluded.hold_id,
            entry_type = excluded.entry_type,
            benefit_type = excluded.benefit_type,
            quantity = excluded.quantity,
            amount = excluded.amount,
            created_at_ms = excluded.created_at_ms",
    );
    sqlx::query(&query)
        .bind(i64::try_from(record.ledger_entry_id)?)
        .bind(i64::try_from(record.tenant_id)?)
        .bind(i64::try_from(record.organization_id)?)
        .bind(i64::try_from(record.account_id)?)
        .bind(i64::try_from(record.user_id)?)
        .bind(record.request_id.map(i64::try_from).transpose()?)
        .bind(record.hold_id.map(i64::try_from).transpose()?)
        .bind(account_ledger_entry_type_as_str(record.entry_type))
        .bind(&record.benefit_type)
        .bind(record.quantity)
        .bind(record.amount)
        .bind(i64::try_from(record.created_at_ms)?)
        .execute(executor)
        .await?;
    Ok(())
}

async fn upsert_account_ledger_allocation_executor<'e, E>(
    executor: E,
    record: &AccountLedgerAllocationRecord,
) -> Result<()>
where
    E: Executor<'e, Database = Postgres>,
{
    let query = postgres_bind_query(
        "INSERT INTO ai_account_ledger_allocation (
            ledger_allocation_id, tenant_id, organization_id, ledger_entry_id, lot_id,
            quantity_delta, created_at_ms
        ) VALUES (?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT(ledger_allocation_id) DO UPDATE SET
            tenant_id = excluded.tenant_id,
            organization_id = excluded.organization_id,
            ledger_entry_id = excluded.ledger_entry_id,
            lot_id = excluded.lot_id,
            quantity_delta = excluded.quantity_delta,
            created_at_ms = excluded.created_at_ms",
    );
    sqlx::query(&query)
        .bind(i64::try_from(record.ledger_allocation_id)?)
        .bind(i64::try_from(record.tenant_id)?)
        .bind(i64::try_from(record.organization_id)?)
        .bind(i64::try_from(record.ledger_entry_id)?)
        .bind(i64::try_from(record.lot_id)?)
        .bind(record.quantity_delta)
        .bind(i64::try_from(record.created_at_ms)?)
        .execute(executor)
        .await?;
    Ok(())
}

async fn upsert_request_meter_fact_executor<'e, E>(
    executor: E,
    record: &RequestMeterFactRecord,
) -> Result<()>
where
    E: Executor<'e, Database = Postgres>,
{
    let query = postgres_bind_query(
        "INSERT INTO ai_request_meter_fact (
            request_id, tenant_id, organization_id, user_id, account_id, api_key_id,
            api_key_hash, auth_type, jwt_subject, platform, owner, request_trace_id,
            gateway_request_ref, upstream_request_ref, protocol_family, capability_code,
            channel_code, model_code, provider_code, request_status, usage_capture_status,
            cost_pricing_plan_id, retail_pricing_plan_id, estimated_credit_hold,
            actual_credit_charge, actual_provider_cost, started_at_ms, finished_at_ms,
            created_at_ms, updated_at_ms
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT(request_id) DO UPDATE SET
            tenant_id = excluded.tenant_id,
            organization_id = excluded.organization_id,
            user_id = excluded.user_id,
            account_id = excluded.account_id,
            api_key_id = excluded.api_key_id,
            api_key_hash = excluded.api_key_hash,
            auth_type = excluded.auth_type,
            jwt_subject = excluded.jwt_subject,
            platform = excluded.platform,
            owner = excluded.owner,
            request_trace_id = excluded.request_trace_id,
            gateway_request_ref = excluded.gateway_request_ref,
            upstream_request_ref = excluded.upstream_request_ref,
            protocol_family = excluded.protocol_family,
            capability_code = excluded.capability_code,
            channel_code = excluded.channel_code,
            model_code = excluded.model_code,
            provider_code = excluded.provider_code,
            request_status = excluded.request_status,
            usage_capture_status = excluded.usage_capture_status,
            cost_pricing_plan_id = excluded.cost_pricing_plan_id,
            retail_pricing_plan_id = excluded.retail_pricing_plan_id,
            estimated_credit_hold = excluded.estimated_credit_hold,
            actual_credit_charge = excluded.actual_credit_charge,
            actual_provider_cost = excluded.actual_provider_cost,
            started_at_ms = excluded.started_at_ms,
            finished_at_ms = excluded.finished_at_ms,
            created_at_ms = excluded.created_at_ms,
            updated_at_ms = excluded.updated_at_ms",
    );
    sqlx::query(&query)
        .bind(i64::try_from(record.request_id)?)
        .bind(i64::try_from(record.tenant_id)?)
        .bind(i64::try_from(record.organization_id)?)
        .bind(i64::try_from(record.user_id)?)
        .bind(i64::try_from(record.account_id)?)
        .bind(record.api_key_id.map(i64::try_from).transpose()?)
        .bind(&record.api_key_hash)
        .bind(&record.auth_type)
        .bind(&record.jwt_subject)
        .bind(&record.platform)
        .bind(&record.owner)
        .bind(&record.request_trace_id)
        .bind(&record.gateway_request_ref)
        .bind(&record.upstream_request_ref)
        .bind(&record.protocol_family)
        .bind(&record.capability_code)
        .bind(&record.channel_code)
        .bind(&record.model_code)
        .bind(&record.provider_code)
        .bind(request_status_as_str(record.request_status))
        .bind(usage_capture_status_as_str(record.usage_capture_status))
        .bind(record.cost_pricing_plan_id.map(i64::try_from).transpose()?)
        .bind(
            record
                .retail_pricing_plan_id
                .map(i64::try_from)
                .transpose()?,
        )
        .bind(record.estimated_credit_hold)
        .bind(record.actual_credit_charge)
        .bind(record.actual_provider_cost)
        .bind(i64::try_from(record.started_at_ms)?)
        .bind(record.finished_at_ms.map(i64::try_from).transpose()?)
        .bind(i64::try_from(record.created_at_ms)?)
        .bind(i64::try_from(record.updated_at_ms)?)
        .execute(executor)
        .await?;
    Ok(())
}

async fn upsert_request_meter_metric_executor<'e, E>(
    executor: E,
    record: &RequestMeterMetricRecord,
) -> Result<()>
where
    E: Executor<'e, Database = Postgres>,
{
    let query = postgres_bind_query(
        "INSERT INTO ai_request_meter_metric (
            request_metric_id, tenant_id, organization_id, request_id, metric_code, quantity,
            provider_field, source_kind, capture_stage, is_billable, captured_at_ms
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT(request_metric_id) DO UPDATE SET
            tenant_id = excluded.tenant_id,
            organization_id = excluded.organization_id,
            request_id = excluded.request_id,
            metric_code = excluded.metric_code,
            quantity = excluded.quantity,
            provider_field = excluded.provider_field,
            source_kind = excluded.source_kind,
            capture_stage = excluded.capture_stage,
            is_billable = excluded.is_billable,
            captured_at_ms = excluded.captured_at_ms",
    );
    sqlx::query(&query)
        .bind(i64::try_from(record.request_metric_id)?)
        .bind(i64::try_from(record.tenant_id)?)
        .bind(i64::try_from(record.organization_id)?)
        .bind(i64::try_from(record.request_id)?)
        .bind(&record.metric_code)
        .bind(record.quantity)
        .bind(&record.provider_field)
        .bind(&record.source_kind)
        .bind(&record.capture_stage)
        .bind(record.is_billable)
        .bind(i64::try_from(record.captured_at_ms)?)
        .execute(executor)
        .await?;
    Ok(())
}

async fn upsert_request_settlement_executor<'e, E>(
    executor: E,
    record: &RequestSettlementRecord,
) -> Result<()>
where
    E: Executor<'e, Database = Postgres>,
{
    let query = postgres_bind_query(
        "INSERT INTO ai_request_settlement (
            request_settlement_id, tenant_id, organization_id, request_id, account_id, user_id,
            hold_id, settlement_status, estimated_credit_hold, released_credit_amount,
            captured_credit_amount, provider_cost_amount, retail_charge_amount, shortfall_amount,
            refunded_amount, settled_at_ms, created_at_ms, updated_at_ms
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT(request_settlement_id) DO UPDATE SET
            tenant_id = excluded.tenant_id,
            organization_id = excluded.organization_id,
            request_id = excluded.request_id,
            account_id = excluded.account_id,
            user_id = excluded.user_id,
            hold_id = excluded.hold_id,
            settlement_status = excluded.settlement_status,
            estimated_credit_hold = excluded.estimated_credit_hold,
            released_credit_amount = excluded.released_credit_amount,
            captured_credit_amount = excluded.captured_credit_amount,
            provider_cost_amount = excluded.provider_cost_amount,
            retail_charge_amount = excluded.retail_charge_amount,
            shortfall_amount = excluded.shortfall_amount,
            refunded_amount = excluded.refunded_amount,
            settled_at_ms = excluded.settled_at_ms,
            created_at_ms = excluded.created_at_ms,
            updated_at_ms = excluded.updated_at_ms",
    );
    sqlx::query(&query)
        .bind(i64::try_from(record.request_settlement_id)?)
        .bind(i64::try_from(record.tenant_id)?)
        .bind(i64::try_from(record.organization_id)?)
        .bind(i64::try_from(record.request_id)?)
        .bind(i64::try_from(record.account_id)?)
        .bind(i64::try_from(record.user_id)?)
        .bind(record.hold_id.map(i64::try_from).transpose()?)
        .bind(request_settlement_status_as_str(record.status))
        .bind(record.estimated_credit_hold)
        .bind(record.released_credit_amount)
        .bind(record.captured_credit_amount)
        .bind(record.provider_cost_amount)
        .bind(record.retail_charge_amount)
        .bind(record.shortfall_amount)
        .bind(record.refunded_amount)
        .bind(i64::try_from(record.settled_at_ms)?)
        .bind(i64::try_from(record.created_at_ms)?)
        .bind(i64::try_from(record.updated_at_ms)?)
        .execute(executor)
        .await?;
    Ok(())
}

fn decode_account_record_row(row: PgRow) -> Result<AccountRecord> {
    Ok(AccountRecord::new(
        u64::try_from(row.try_get::<i64, _>("account_id")?)?,
        u64::try_from(row.try_get::<i64, _>("tenant_id")?)?,
        u64::try_from(row.try_get::<i64, _>("organization_id")?)?,
        u64::try_from(row.try_get::<i64, _>("user_id")?)?,
        parse_account_type(&row.try_get::<String, _>("account_type")?)?,
    )
    .with_currency_code(row.try_get::<String, _>("currency_code")?)
    .with_credit_unit_code(row.try_get::<String, _>("credit_unit_code")?)
    .with_status(parse_account_status(&row.try_get::<String, _>("status")?)?)
    .with_allow_overdraft(row.try_get::<bool, _>("allow_overdraft")?)
    .with_overdraft_limit(row.try_get::<f64, _>("overdraft_limit")?)
    .with_created_at_ms(u64::try_from(row.try_get::<i64, _>("created_at_ms")?)?)
    .with_updated_at_ms(u64::try_from(row.try_get::<i64, _>("updated_at_ms")?)?))
}

fn decode_identity_user_row(row: PgRow) -> Result<IdentityUserRecord> {
    Ok(IdentityUserRecord::new(
        u64::try_from(row.try_get::<i64, _>("user_id")?)?,
        u64::try_from(row.try_get::<i64, _>("tenant_id")?)?,
        u64::try_from(row.try_get::<i64, _>("organization_id")?)?,
    )
    .with_external_user_ref(row.try_get::<Option<String>, _>("external_user_ref")?)
    .with_username(row.try_get::<Option<String>, _>("username")?)
    .with_display_name(row.try_get::<Option<String>, _>("display_name")?)
    .with_email(row.try_get::<Option<String>, _>("email")?)
    .with_status(row.try_get::<String, _>("status")?)
    .with_created_at_ms(u64::try_from(row.try_get::<i64, _>("created_at_ms")?)?)
    .with_updated_at_ms(u64::try_from(row.try_get::<i64, _>("updated_at_ms")?)?))
}

fn decode_canonical_api_key_row(row: PgRow) -> Result<CanonicalApiKeyRecord> {
    Ok(CanonicalApiKeyRecord::new(
        u64::try_from(row.try_get::<i64, _>("api_key_id")?)?,
        u64::try_from(row.try_get::<i64, _>("tenant_id")?)?,
        u64::try_from(row.try_get::<i64, _>("organization_id")?)?,
        u64::try_from(row.try_get::<i64, _>("user_id")?)?,
        row.try_get::<String, _>("key_hash")?,
    )
    .with_key_prefix(row.try_get::<String, _>("key_prefix")?)
    .with_display_name(row.try_get::<String, _>("display_name")?)
    .with_status(row.try_get::<String, _>("status")?)
    .with_expires_at_ms(
        row.try_get::<Option<i64>, _>("expires_at_ms")?
            .map(u64::try_from)
            .transpose()?,
    )
    .with_last_used_at_ms(
        row.try_get::<Option<i64>, _>("last_used_at_ms")?
            .map(u64::try_from)
            .transpose()?,
    )
    .with_rotated_from_api_key_id(
        row.try_get::<Option<i64>, _>("rotated_from_api_key_id")?
            .map(u64::try_from)
            .transpose()?,
    )
    .with_created_at_ms(u64::try_from(row.try_get::<i64, _>("created_at_ms")?)?)
    .with_updated_at_ms(u64::try_from(row.try_get::<i64, _>("updated_at_ms")?)?))
}

fn decode_identity_binding_row(row: PgRow) -> Result<IdentityBindingRecord> {
    Ok(IdentityBindingRecord::new(
        u64::try_from(row.try_get::<i64, _>("identity_binding_id")?)?,
        u64::try_from(row.try_get::<i64, _>("tenant_id")?)?,
        u64::try_from(row.try_get::<i64, _>("organization_id")?)?,
        u64::try_from(row.try_get::<i64, _>("user_id")?)?,
        row.try_get::<String, _>("binding_type")?,
    )
    .with_issuer(row.try_get::<Option<String>, _>("issuer")?)
    .with_subject(row.try_get::<Option<String>, _>("subject")?)
    .with_platform(row.try_get::<Option<String>, _>("platform")?)
    .with_owner(row.try_get::<Option<String>, _>("owner")?)
    .with_external_ref(row.try_get::<Option<String>, _>("external_ref")?)
    .with_status(row.try_get::<String, _>("status")?)
    .with_created_at_ms(u64::try_from(row.try_get::<i64, _>("created_at_ms")?)?)
    .with_updated_at_ms(u64::try_from(row.try_get::<i64, _>("updated_at_ms")?)?))
}

fn decode_account_benefit_lot_row(row: PgRow) -> Result<AccountBenefitLotRecord> {
    Ok(AccountBenefitLotRecord::new(
        u64::try_from(row.try_get::<i64, _>("lot_id")?)?,
        u64::try_from(row.try_get::<i64, _>("tenant_id")?)?,
        u64::try_from(row.try_get::<i64, _>("organization_id")?)?,
        u64::try_from(row.try_get::<i64, _>("account_id")?)?,
        u64::try_from(row.try_get::<i64, _>("user_id")?)?,
        parse_account_benefit_type(&row.try_get::<String, _>("benefit_type")?)?,
    )
    .with_source_type(parse_account_benefit_source_type(
        &row.try_get::<String, _>("source_type")?,
    )?)
    .with_source_id(
        row.try_get::<Option<i64>, _>("source_id")?
            .map(u64::try_from)
            .transpose()?,
    )
    .with_scope_json(row.try_get::<Option<String>, _>("scope_json")?)
    .with_original_quantity(row.try_get::<f64, _>("original_quantity")?)
    .with_remaining_quantity(row.try_get::<f64, _>("remaining_quantity")?)
    .with_held_quantity(row.try_get::<f64, _>("held_quantity")?)
    .with_priority(row.try_get::<i32, _>("priority")?)
    .with_acquired_unit_cost(row.try_get::<Option<f64>, _>("acquired_unit_cost")?)
    .with_issued_at_ms(u64::try_from(row.try_get::<i64, _>("issued_at_ms")?)?)
    .with_expires_at_ms(
        row.try_get::<Option<i64>, _>("expires_at_ms")?
            .map(u64::try_from)
            .transpose()?,
    )
    .with_status(parse_account_benefit_lot_status(
        &row.try_get::<String, _>("status")?,
    )?)
    .with_created_at_ms(u64::try_from(row.try_get::<i64, _>("created_at_ms")?)?)
    .with_updated_at_ms(u64::try_from(row.try_get::<i64, _>("updated_at_ms")?)?))
}

fn decode_account_hold_row(row: PgRow) -> Result<AccountHoldRecord> {
    Ok(AccountHoldRecord::new(
        u64::try_from(row.try_get::<i64, _>("hold_id")?)?,
        u64::try_from(row.try_get::<i64, _>("tenant_id")?)?,
        u64::try_from(row.try_get::<i64, _>("organization_id")?)?,
        u64::try_from(row.try_get::<i64, _>("account_id")?)?,
        u64::try_from(row.try_get::<i64, _>("user_id")?)?,
        u64::try_from(row.try_get::<i64, _>("request_id")?)?,
    )
    .with_status(parse_account_hold_status(
        &row.try_get::<String, _>("hold_status")?,
    )?)
    .with_estimated_quantity(row.try_get::<f64, _>("estimated_quantity")?)
    .with_captured_quantity(row.try_get::<f64, _>("captured_quantity")?)
    .with_released_quantity(row.try_get::<f64, _>("released_quantity")?)
    .with_expires_at_ms(u64::try_from(row.try_get::<i64, _>("expires_at_ms")?)?)
    .with_created_at_ms(u64::try_from(row.try_get::<i64, _>("created_at_ms")?)?)
    .with_updated_at_ms(u64::try_from(row.try_get::<i64, _>("updated_at_ms")?)?))
}

fn decode_account_hold_allocation_row(row: PgRow) -> Result<AccountHoldAllocationRecord> {
    Ok(AccountHoldAllocationRecord::new(
        u64::try_from(row.try_get::<i64, _>("hold_allocation_id")?)?,
        u64::try_from(row.try_get::<i64, _>("tenant_id")?)?,
        u64::try_from(row.try_get::<i64, _>("organization_id")?)?,
        u64::try_from(row.try_get::<i64, _>("hold_id")?)?,
        u64::try_from(row.try_get::<i64, _>("lot_id")?)?,
    )
    .with_allocated_quantity(row.try_get::<f64, _>("allocated_quantity")?)
    .with_captured_quantity(row.try_get::<f64, _>("captured_quantity")?)
    .with_released_quantity(row.try_get::<f64, _>("released_quantity")?)
    .with_created_at_ms(u64::try_from(row.try_get::<i64, _>("created_at_ms")?)?)
    .with_updated_at_ms(u64::try_from(row.try_get::<i64, _>("updated_at_ms")?)?))
}

fn decode_account_ledger_entry_row(row: PgRow) -> Result<AccountLedgerEntryRecord> {
    Ok(AccountLedgerEntryRecord::new(
        u64::try_from(row.try_get::<i64, _>("ledger_entry_id")?)?,
        u64::try_from(row.try_get::<i64, _>("tenant_id")?)?,
        u64::try_from(row.try_get::<i64, _>("organization_id")?)?,
        u64::try_from(row.try_get::<i64, _>("account_id")?)?,
        u64::try_from(row.try_get::<i64, _>("user_id")?)?,
        parse_account_ledger_entry_type(&row.try_get::<String, _>("entry_type")?)?,
    )
    .with_request_id(
        row.try_get::<Option<i64>, _>("request_id")?
            .map(u64::try_from)
            .transpose()?,
    )
    .with_hold_id(
        row.try_get::<Option<i64>, _>("hold_id")?
            .map(u64::try_from)
            .transpose()?,
    )
    .with_benefit_type(row.try_get::<Option<String>, _>("benefit_type")?)
    .with_quantity(row.try_get::<f64, _>("quantity")?)
    .with_amount(row.try_get::<f64, _>("amount")?)
    .with_created_at_ms(u64::try_from(row.try_get::<i64, _>("created_at_ms")?)?))
}

fn decode_account_ledger_allocation_row(row: PgRow) -> Result<AccountLedgerAllocationRecord> {
    Ok(AccountLedgerAllocationRecord::new(
        u64::try_from(row.try_get::<i64, _>("ledger_allocation_id")?)?,
        u64::try_from(row.try_get::<i64, _>("tenant_id")?)?,
        u64::try_from(row.try_get::<i64, _>("organization_id")?)?,
        u64::try_from(row.try_get::<i64, _>("ledger_entry_id")?)?,
        u64::try_from(row.try_get::<i64, _>("lot_id")?)?,
    )
    .with_quantity_delta(row.try_get::<f64, _>("quantity_delta")?)
    .with_created_at_ms(u64::try_from(row.try_get::<i64, _>("created_at_ms")?)?))
}

fn decode_request_meter_fact_row(row: PgRow) -> Result<RequestMeterFactRecord> {
    Ok(RequestMeterFactRecord::new(
        u64::try_from(row.try_get::<i64, _>("request_id")?)?,
        u64::try_from(row.try_get::<i64, _>("tenant_id")?)?,
        u64::try_from(row.try_get::<i64, _>("organization_id")?)?,
        u64::try_from(row.try_get::<i64, _>("user_id")?)?,
        u64::try_from(row.try_get::<i64, _>("account_id")?)?,
        row.try_get::<String, _>("auth_type")?,
        row.try_get::<String, _>("capability_code")?,
        row.try_get::<String, _>("channel_code")?,
        row.try_get::<String, _>("model_code")?,
        row.try_get::<String, _>("provider_code")?,
    )
    .with_api_key_id(
        row.try_get::<Option<i64>, _>("api_key_id")?
            .map(u64::try_from)
            .transpose()?,
    )
    .with_api_key_hash(row.try_get::<Option<String>, _>("api_key_hash")?)
    .with_jwt_subject(row.try_get::<Option<String>, _>("jwt_subject")?)
    .with_platform(row.try_get::<Option<String>, _>("platform")?)
    .with_owner(row.try_get::<Option<String>, _>("owner")?)
    .with_request_trace_id(row.try_get::<Option<String>, _>("request_trace_id")?)
    .with_gateway_request_ref(row.try_get::<Option<String>, _>("gateway_request_ref")?)
    .with_upstream_request_ref(row.try_get::<Option<String>, _>("upstream_request_ref")?)
    .with_protocol_family(row.try_get::<String, _>("protocol_family")?)
    .with_request_status(parse_request_status(
        &row.try_get::<String, _>("request_status")?,
    )?)
    .with_usage_capture_status(parse_usage_capture_status(
        &row.try_get::<String, _>("usage_capture_status")?,
    )?)
    .with_cost_pricing_plan_id(
        row.try_get::<Option<i64>, _>("cost_pricing_plan_id")?
            .map(u64::try_from)
            .transpose()?,
    )
    .with_retail_pricing_plan_id(
        row.try_get::<Option<i64>, _>("retail_pricing_plan_id")?
            .map(u64::try_from)
            .transpose()?,
    )
    .with_estimated_credit_hold(row.try_get::<f64, _>("estimated_credit_hold")?)
    .with_actual_credit_charge(row.try_get::<Option<f64>, _>("actual_credit_charge")?)
    .with_actual_provider_cost(row.try_get::<Option<f64>, _>("actual_provider_cost")?)
    .with_started_at_ms(u64::try_from(row.try_get::<i64, _>("started_at_ms")?)?)
    .with_finished_at_ms(
        row.try_get::<Option<i64>, _>("finished_at_ms")?
            .map(u64::try_from)
            .transpose()?,
    )
    .with_created_at_ms(u64::try_from(row.try_get::<i64, _>("created_at_ms")?)?)
    .with_updated_at_ms(u64::try_from(row.try_get::<i64, _>("updated_at_ms")?)?))
}

fn decode_request_meter_metric_row(row: PgRow) -> Result<RequestMeterMetricRecord> {
    Ok(RequestMeterMetricRecord::new(
        u64::try_from(row.try_get::<i64, _>("request_metric_id")?)?,
        u64::try_from(row.try_get::<i64, _>("tenant_id")?)?,
        u64::try_from(row.try_get::<i64, _>("organization_id")?)?,
        u64::try_from(row.try_get::<i64, _>("request_id")?)?,
        row.try_get::<String, _>("metric_code")?,
        row.try_get::<f64, _>("quantity")?,
    )
    .with_provider_field(row.try_get::<Option<String>, _>("provider_field")?)
    .with_source_kind(row.try_get::<String, _>("source_kind")?)
    .with_capture_stage(row.try_get::<String, _>("capture_stage")?)
    .with_is_billable(row.try_get::<bool, _>("is_billable")?)
    .with_captured_at_ms(u64::try_from(row.try_get::<i64, _>("captured_at_ms")?)?))
}

fn decode_pricing_plan_row(row: PgRow) -> Result<PricingPlanRecord> {
    Ok(PricingPlanRecord::new(
        u64::try_from(row.try_get::<i64, _>("pricing_plan_id")?)?,
        u64::try_from(row.try_get::<i64, _>("tenant_id")?)?,
        u64::try_from(row.try_get::<i64, _>("organization_id")?)?,
        row.try_get::<String, _>("plan_code")?,
        u64::try_from(row.try_get::<i64, _>("plan_version")?)?,
    )
    .with_display_name(row.try_get::<String, _>("display_name")?)
    .with_currency_code(row.try_get::<String, _>("currency_code")?)
    .with_credit_unit_code(row.try_get::<String, _>("credit_unit_code")?)
    .with_status(row.try_get::<String, _>("status")?)
    .with_created_at_ms(u64::try_from(row.try_get::<i64, _>("created_at_ms")?)?)
    .with_updated_at_ms(u64::try_from(row.try_get::<i64, _>("updated_at_ms")?)?))
}

fn decode_pricing_rate_row(row: PgRow) -> Result<PricingRateRecord> {
    Ok(PricingRateRecord::new(
        u64::try_from(row.try_get::<i64, _>("pricing_rate_id")?)?,
        u64::try_from(row.try_get::<i64, _>("tenant_id")?)?,
        u64::try_from(row.try_get::<i64, _>("organization_id")?)?,
        u64::try_from(row.try_get::<i64, _>("pricing_plan_id")?)?,
        row.try_get::<String, _>("metric_code")?,
    )
    .with_model_code(row.try_get::<Option<String>, _>("model_code")?)
    .with_provider_code(row.try_get::<Option<String>, _>("provider_code")?)
    .with_quantity_step(row.try_get::<f64, _>("quantity_step")?)
    .with_unit_price(row.try_get::<f64, _>("unit_price")?)
    .with_created_at_ms(u64::try_from(row.try_get::<i64, _>("created_at_ms")?)?))
}

fn decode_request_settlement_row(row: PgRow) -> Result<RequestSettlementRecord> {
    Ok(RequestSettlementRecord::new(
        u64::try_from(row.try_get::<i64, _>("request_settlement_id")?)?,
        u64::try_from(row.try_get::<i64, _>("tenant_id")?)?,
        u64::try_from(row.try_get::<i64, _>("organization_id")?)?,
        u64::try_from(row.try_get::<i64, _>("request_id")?)?,
        u64::try_from(row.try_get::<i64, _>("account_id")?)?,
        u64::try_from(row.try_get::<i64, _>("user_id")?)?,
    )
    .with_hold_id(
        row.try_get::<Option<i64>, _>("hold_id")?
            .map(u64::try_from)
            .transpose()?,
    )
    .with_status(parse_request_settlement_status(
        &row.try_get::<String, _>("settlement_status")?,
    )?)
    .with_estimated_credit_hold(row.try_get::<f64, _>("estimated_credit_hold")?)
    .with_released_credit_amount(row.try_get::<f64, _>("released_credit_amount")?)
    .with_captured_credit_amount(row.try_get::<f64, _>("captured_credit_amount")?)
    .with_provider_cost_amount(row.try_get::<f64, _>("provider_cost_amount")?)
    .with_retail_charge_amount(row.try_get::<f64, _>("retail_charge_amount")?)
    .with_shortfall_amount(row.try_get::<f64, _>("shortfall_amount")?)
    .with_refunded_amount(row.try_get::<f64, _>("refunded_amount")?)
    .with_settled_at_ms(u64::try_from(row.try_get::<i64, _>("settled_at_ms")?)?)
    .with_created_at_ms(u64::try_from(row.try_get::<i64, _>("created_at_ms")?)?)
    .with_updated_at_ms(u64::try_from(row.try_get::<i64, _>("updated_at_ms")?)?))
}
