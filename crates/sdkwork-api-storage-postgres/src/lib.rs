use anyhow::Result;
use sdkwork_api_domain_billing::{LedgerEntry, QuotaPolicy};
use sdkwork_api_domain_catalog::{
    normalize_provider_extension_id, Channel, ChannelModelRecord, ModelCapability,
    ModelCatalogEntry, ModelPriceRecord, ProviderChannelBinding, ProxyProvider,
};
use sdkwork_api_domain_coupon::CouponCampaign;
use sdkwork_api_domain_credential::UpstreamCredential;
use sdkwork_api_domain_identity::{AdminUserRecord, GatewayApiKeyRecord, PortalUserRecord};
use sdkwork_api_domain_routing::{
    ProjectRoutingPreferences, ProviderHealthSnapshot, RoutingCandidateAssessment,
    RoutingDecisionLog, RoutingDecisionSource, RoutingPolicy, RoutingStrategy,
};
use sdkwork_api_domain_tenant::{Project, Tenant};
use sdkwork_api_domain_usage::UsageRecord;
use sdkwork_api_extension_core::{ExtensionInstallation, ExtensionInstance, ExtensionRuntime};
use sdkwork_api_secret_core::SecretEnvelope;
use sdkwork_api_storage_core::{
    AdminStore, ExtensionRuntimeRolloutParticipantRecord, ExtensionRuntimeRolloutRecord,
    ServiceRuntimeNodeRecord, StandaloneConfigRolloutParticipantRecord,
    StandaloneConfigRolloutRecord, StorageDialect,
};
use serde_json::Value;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

const BUILTIN_CHANNEL_SEEDS: [(&str, &str, i32); 5] = [
    ("openai", "OpenAI", 10),
    ("anthropic", "Anthropic", 20),
    ("gemini", "Gemini", 30),
    ("openrouter", "OpenRouter", 40),
    ("ollama", "Ollama", 50),
];

const LEGACY_RENAMED_TABLE_MAPPINGS: [(&str, &str); 20] = [
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
    ("billing_ledger_entries", "ai_billing_ledger_entries"),
    ("billing_quota_policies", "ai_billing_quota_policies"),
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

type AdminUserRow = (String, String, String, String, String, bool, i64);

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

fn decode_admin_user_row(row: Option<AdminUserRow>) -> Result<Option<AdminUserRecord>> {
    row.map(
        |(id, email, display_name, password_salt, password_hash, active, created_at_ms)| {
            Ok(AdminUserRecord {
                id,
                email,
                display_name,
                password_salt,
                password_hash,
                active,
                created_at_ms: u64::try_from(created_at_ms)?,
            })
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
        "CREATE TABLE IF NOT EXISTS ai_admin_users (
            id TEXT PRIMARY KEY NOT NULL,
            email TEXT NOT NULL,
            display_name TEXT NOT NULL DEFAULT '',
            password_salt TEXT NOT NULL DEFAULT '',
            password_hash TEXT NOT NULL DEFAULT '',
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
            capability TEXT NOT NULL,
            route_key TEXT NOT NULL,
            selected_provider_id TEXT NOT NULL,
            matched_policy_id TEXT,
            strategy TEXT NOT NULL,
            selection_seed BIGINT,
            selection_reason TEXT,
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
        "ALTER TABLE ai_routing_decision_logs ADD COLUMN IF NOT EXISTS requested_region TEXT",
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
        "CREATE TABLE IF NOT EXISTS ai_usage_records (
            project_id TEXT NOT NULL,
            model TEXT NOT NULL,
            provider_id TEXT NOT NULL,
            units BIGINT NOT NULL DEFAULT 0,
            amount DOUBLE PRECISION NOT NULL DEFAULT 0,
            input_tokens BIGINT NOT NULL DEFAULT 0,
            output_tokens BIGINT NOT NULL DEFAULT 0,
            total_tokens BIGINT NOT NULL DEFAULT 0,
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
    sqlx::query(
        "ALTER TABLE ai_usage_records ADD COLUMN IF NOT EXISTS created_at_ms BIGINT NOT NULL DEFAULT 0",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_billing_ledger_entries (
            project_id TEXT NOT NULL,
            units BIGINT NOT NULL,
            amount DOUBLE PRECISION NOT NULL
        )",
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
        "CREATE TABLE IF NOT EXISTS ai_app_api_keys (
            hashed_key TEXT PRIMARY KEY NOT NULL,
            raw_key TEXT,
            tenant_id TEXT NOT NULL,
            project_id TEXT NOT NULL,
            environment TEXT NOT NULL,
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
        "ALTER TABLE ai_app_api_keys ADD COLUMN IF NOT EXISTS label TEXT NOT NULL DEFAULT ''",
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
                raw_key,
                tenant_id,
                project_id,
                environment,
                label,
                notes,
                created_at_ms,
                last_used_at_ms,
                expires_at_ms,
                active
            )
            SELECT
                hashed_key,
                NULL,
                tenant_id,
                project_id,
                environment,
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
        let mut providers = Vec::with_capacity(rows.len());
        for (id, channel_id, extension_id, adapter_kind, base_url, display_name) in rows {
            let channel_bindings =
                load_provider_channel_bindings(&self.pool, &id, &channel_id).await?;
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
            "INSERT INTO ai_routing_decision_logs (decision_id, decision_source, tenant_id, project_id, capability, route_key, selected_provider_id, matched_policy_id, strategy, selection_seed, selection_reason, requested_region, slo_applied, slo_degraded, created_at_ms, assessments_json)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
             ON CONFLICT(decision_id) DO UPDATE SET decision_source = excluded.decision_source, tenant_id = excluded.tenant_id, project_id = excluded.project_id, capability = excluded.capability, route_key = excluded.route_key, selected_provider_id = excluded.selected_provider_id, matched_policy_id = excluded.matched_policy_id, strategy = excluded.strategy, selection_seed = excluded.selection_seed, selection_reason = excluded.selection_reason, requested_region = excluded.requested_region, slo_applied = excluded.slo_applied, slo_degraded = excluded.slo_degraded, created_at_ms = excluded.created_at_ms, assessments_json = excluded.assessments_json",
        )
        .bind(&log.decision_id)
        .bind(log.decision_source.as_str())
        .bind(&log.tenant_id)
        .bind(&log.project_id)
        .bind(&log.capability)
        .bind(&log.route_key)
        .bind(&log.selected_provider_id)
        .bind(&log.matched_policy_id)
        .bind(&log.strategy)
        .bind(log.selection_seed.map(i64::try_from).transpose()?)
        .bind(&log.selection_reason)
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
        let rows = sqlx::query_as::<
            _,
            (
                String,
                String,
                Option<String>,
                Option<String>,
                String,
                String,
                String,
                Option<String>,
                String,
                Option<i64>,
                Option<String>,
                Option<String>,
                bool,
                bool,
                i64,
                String,
            ),
        >(
            "SELECT decision_id, decision_source, tenant_id, project_id, capability, route_key, selected_provider_id, matched_policy_id, strategy, selection_seed, selection_reason, requested_region, slo_applied, slo_degraded, created_at_ms, assessments_json
             FROM ai_routing_decision_logs
             ORDER BY created_at_ms DESC, decision_id DESC",
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(
                |(
                    decision_id,
                    decision_source,
                    tenant_id,
                    project_id,
                    capability,
                    route_key,
                    selected_provider_id,
                    matched_policy_id,
                    strategy,
                    selection_seed,
                    selection_reason,
                    requested_region,
                    slo_applied,
                    slo_degraded,
                    created_at_ms,
                    assessments_json,
                )| {
                    Ok(RoutingDecisionLog::new(
                        decision_id,
                        RoutingDecisionSource::from_str(&decision_source)
                            .unwrap_or(RoutingDecisionSource::Gateway),
                        capability,
                        route_key,
                        selected_provider_id,
                        strategy,
                        u64::try_from(created_at_ms)?,
                    )
                    .with_tenant_id_option(tenant_id)
                    .with_project_id_option(project_id)
                    .with_matched_policy_id_option(matched_policy_id)
                    .with_selection_seed_option(selection_seed.map(u64::try_from).transpose()?)
                    .with_selection_reason_option(selection_reason)
                    .with_requested_region_option(requested_region)
                    .with_slo_state(slo_applied, slo_degraded)
                    .with_assessments(decode_routing_assessments(&assessments_json)?))
                },
            )
            .collect()
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
            "INSERT INTO ai_usage_records (project_id, model, provider_id, units, amount, input_tokens, output_tokens, total_tokens, created_at_ms)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
        )
        .bind(&record.project_id)
        .bind(&record.model)
        .bind(&record.provider)
        .bind(i64::try_from(record.units)?)
        .bind(record.amount)
        .bind(i64::try_from(record.input_tokens)?)
        .bind(i64::try_from(record.output_tokens)?)
        .bind(i64::try_from(record.total_tokens)?)
        .bind(i64::try_from(record.created_at_ms)?)
        .execute(&self.pool)
        .await?;
        Ok(record.clone())
    }

    pub async fn list_usage_records(&self) -> Result<Vec<UsageRecord>> {
        let rows = sqlx::query_as::<_, (String, String, String, i64, f64, i64, i64, i64, i64)>(
            "SELECT project_id, model, provider_id, units, amount, input_tokens, output_tokens, total_tokens, created_at_ms FROM ai_usage_records",
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
                        created_at_ms: u64::try_from(created_at_ms)?,
                    })
                },
            )
            .collect::<Result<Vec<_>>>()?)
    }

    pub async fn insert_ledger_entry(&self, entry: &LedgerEntry) -> Result<LedgerEntry> {
        sqlx::query(
            "INSERT INTO ai_billing_ledger_entries (project_id, units, amount) VALUES ($1, $2, $3)",
        )
        .bind(&entry.project_id)
        .bind(i64::try_from(entry.units)?)
        .bind(entry.amount)
        .execute(&self.pool)
        .await?;
        Ok(entry.clone())
    }

    pub async fn list_ledger_entries(&self) -> Result<Vec<LedgerEntry>> {
        let rows = sqlx::query_as::<_, (String, i64, f64)>(
            "SELECT project_id, units, amount FROM ai_billing_ledger_entries",
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
            "INSERT INTO ai_admin_users (id, email, display_name, password_salt, password_hash, active, created_at_ms)
             VALUES ($1, $2, $3, $4, $5, $6, $7)
             ON CONFLICT(id) DO UPDATE SET
             email = excluded.email,
             display_name = excluded.display_name,
             password_salt = excluded.password_salt,
             password_hash = excluded.password_hash,
             active = excluded.active,
             created_at_ms = excluded.created_at_ms",
        )
        .bind(&user.id)
        .bind(&user.email)
        .bind(&user.display_name)
        .bind(&user.password_salt)
        .bind(&user.password_hash)
        .bind(user.active)
        .bind(i64::try_from(user.created_at_ms)?)
        .execute(&self.pool)
        .await?;
        Ok(user.clone())
    }

    pub async fn list_admin_users(&self) -> Result<Vec<AdminUserRecord>> {
        let rows = sqlx::query_as::<_, AdminUserRow>(
            "SELECT id, email, display_name, password_salt, password_hash, active, created_at_ms
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
            "SELECT id, email, display_name, password_salt, password_hash, active, created_at_ms
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
            "SELECT id, email, display_name, password_salt, password_hash, active, created_at_ms
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

    pub async fn insert_gateway_api_key(
        &self,
        record: &GatewayApiKeyRecord,
    ) -> Result<GatewayApiKeyRecord> {
        sqlx::query(
            "INSERT INTO ai_app_api_keys (
                hashed_key,
                raw_key,
                tenant_id,
                project_id,
                environment,
                label,
                notes,
                created_at_ms,
                last_used_at_ms,
                expires_at_ms,
                active
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
             ON CONFLICT(hashed_key) DO UPDATE SET
                raw_key = excluded.raw_key,
                tenant_id = excluded.tenant_id,
                project_id = excluded.project_id,
                environment = excluded.environment,
                label = excluded.label,
                notes = excluded.notes,
                created_at_ms = excluded.created_at_ms,
                last_used_at_ms = excluded.last_used_at_ms,
                expires_at_ms = excluded.expires_at_ms,
                active = excluded.active",
        )
        .bind(&record.hashed_key)
        .bind(&record.raw_key)
        .bind(&record.tenant_id)
        .bind(&record.project_id)
        .bind(&record.environment)
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
        let rows = sqlx::query_as::<_, (String, Option<String>, String, String, String, String, Option<String>, i64, Option<i64>, Option<i64>, bool)>(
            "SELECT hashed_key, raw_key, tenant_id, project_id, environment, label, notes, created_at_ms, last_used_at_ms, expires_at_ms, active
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
                    raw_key,
                    tenant_id,
                    project_id,
                    environment,
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
                    raw_key,
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
        let row = sqlx::query_as::<_, (String, Option<String>, String, String, String, String, Option<String>, i64, Option<i64>, Option<i64>, bool)>(
            "SELECT hashed_key, raw_key, tenant_id, project_id, environment, label, notes, created_at_ms, last_used_at_ms, expires_at_ms, active
             FROM ai_app_api_keys
             WHERE hashed_key = $1",
        )
        .bind(hashed_key)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(
            |(
                hashed_key,
                raw_key,
                tenant_id,
                project_id,
                environment,
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
                    raw_key,
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

    async fn insert_ledger_entry(&self, entry: &LedgerEntry) -> Result<LedgerEntry> {
        PostgresAdminStore::insert_ledger_entry(self, entry).await
    }

    async fn list_ledger_entries(&self) -> Result<Vec<LedgerEntry>> {
        PostgresAdminStore::list_ledger_entries(self).await
    }

    async fn insert_quota_policy(&self, policy: &QuotaPolicy) -> Result<QuotaPolicy> {
        PostgresAdminStore::insert_quota_policy(self, policy).await
    }

    async fn list_quota_policies(&self) -> Result<Vec<QuotaPolicy>> {
        PostgresAdminStore::list_quota_policies(self).await
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

    async fn find_coupon(&self, coupon_id: &str) -> Result<Option<CouponCampaign>> {
        PostgresAdminStore::find_coupon(self, coupon_id).await
    }

    async fn delete_coupon(&self, coupon_id: &str) -> Result<bool> {
        PostgresAdminStore::delete_coupon(self, coupon_id).await
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
