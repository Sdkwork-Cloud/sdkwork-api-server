use anyhow::Result;
use sdkwork_api_domain_billing::{LedgerEntry, QuotaPolicy};
use sdkwork_api_domain_catalog::{
    normalize_provider_extension_id, Channel, ModelCapability, ModelCatalogEntry,
    ProviderChannelBinding, ProxyProvider,
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
         FROM routing_policy_providers
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
         FROM catalog_provider_channel_bindings
         WHERE provider_id = $1
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

pub async fn run_migrations(url: &str) -> Result<PgPool> {
    let pool = PgPoolOptions::new().max_connections(5).connect(url).await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS identity_users (
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
        "ALTER TABLE identity_users ADD COLUMN IF NOT EXISTS display_name TEXT NOT NULL DEFAULT ''",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE identity_users ADD COLUMN IF NOT EXISTS password_salt TEXT NOT NULL DEFAULT ''",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE identity_users ADD COLUMN IF NOT EXISTS password_hash TEXT NOT NULL DEFAULT ''",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE identity_users ADD COLUMN IF NOT EXISTS workspace_tenant_id TEXT NOT NULL DEFAULT ''",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE identity_users ADD COLUMN IF NOT EXISTS workspace_project_id TEXT NOT NULL DEFAULT ''",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE identity_users ADD COLUMN IF NOT EXISTS active BOOLEAN NOT NULL DEFAULT TRUE",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE identity_users ADD COLUMN IF NOT EXISTS created_at_ms BIGINT NOT NULL DEFAULT 0",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_identity_users_email ON identity_users (email)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS admin_users (
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
        "ALTER TABLE admin_users ADD COLUMN IF NOT EXISTS display_name TEXT NOT NULL DEFAULT ''",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE admin_users ADD COLUMN IF NOT EXISTS password_salt TEXT NOT NULL DEFAULT ''",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE admin_users ADD COLUMN IF NOT EXISTS password_hash TEXT NOT NULL DEFAULT ''",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE admin_users ADD COLUMN IF NOT EXISTS active BOOLEAN NOT NULL DEFAULT TRUE",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE admin_users ADD COLUMN IF NOT EXISTS created_at_ms BIGINT NOT NULL DEFAULT 0",
    )
    .execute(&pool)
    .await?;
    sqlx::query("CREATE UNIQUE INDEX IF NOT EXISTS idx_admin_users_email ON admin_users (email)")
        .execute(&pool)
        .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS tenant_records (
            id TEXT PRIMARY KEY NOT NULL,
            name TEXT NOT NULL
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS tenant_projects (
            id TEXT PRIMARY KEY NOT NULL,
            tenant_id TEXT NOT NULL,
            name TEXT NOT NULL
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS coupon_campaigns (
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
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_coupon_campaigns_code ON coupon_campaigns (code)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS catalog_channels (
            id TEXT PRIMARY KEY NOT NULL,
            name TEXT NOT NULL
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS catalog_proxy_providers (
            id TEXT PRIMARY KEY NOT NULL,
            channel_id TEXT NOT NULL,
            extension_id TEXT NOT NULL DEFAULT '',
            adapter_kind TEXT NOT NULL DEFAULT 'openai',
            base_url TEXT NOT NULL DEFAULT 'http://localhost',
            display_name TEXT NOT NULL
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE catalog_proxy_providers ADD COLUMN IF NOT EXISTS extension_id TEXT NOT NULL DEFAULT ''",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE catalog_proxy_providers ADD COLUMN IF NOT EXISTS adapter_kind TEXT NOT NULL DEFAULT 'openai'",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE catalog_proxy_providers ADD COLUMN IF NOT EXISTS base_url TEXT NOT NULL DEFAULT 'http://localhost'",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS catalog_provider_channel_bindings (
            provider_id TEXT NOT NULL,
            channel_id TEXT NOT NULL,
            is_primary BOOLEAN NOT NULL DEFAULT FALSE,
            PRIMARY KEY (provider_id, channel_id)
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS credential_records (
            tenant_id TEXT NOT NULL,
            provider_id TEXT NOT NULL,
            key_reference TEXT NOT NULL,
            secret_backend TEXT NOT NULL DEFAULT 'database_encrypted',
            secret_local_file TEXT,
            secret_keyring_service TEXT,
            secret_master_key_id TEXT,
            secret_ciphertext TEXT,
            secret_key_version INTEGER,
            PRIMARY KEY (tenant_id, provider_id, key_reference)
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE credential_records ADD COLUMN IF NOT EXISTS secret_backend TEXT NOT NULL DEFAULT 'database_encrypted'",
    )
    .execute(&pool)
    .await?;
    sqlx::query("ALTER TABLE credential_records ADD COLUMN IF NOT EXISTS secret_local_file TEXT")
        .execute(&pool)
        .await?;
    sqlx::query(
        "ALTER TABLE credential_records ADD COLUMN IF NOT EXISTS secret_keyring_service TEXT",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE credential_records ADD COLUMN IF NOT EXISTS secret_master_key_id TEXT",
    )
    .execute(&pool)
    .await?;
    sqlx::query("ALTER TABLE credential_records ADD COLUMN IF NOT EXISTS secret_ciphertext TEXT")
        .execute(&pool)
        .await?;
    sqlx::query(
        "ALTER TABLE credential_records ADD COLUMN IF NOT EXISTS secret_key_version INTEGER",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS catalog_models (
            external_name TEXT NOT NULL,
            provider_id TEXT NOT NULL,
            PRIMARY KEY (external_name, provider_id)
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE catalog_models ADD COLUMN IF NOT EXISTS capabilities TEXT NOT NULL DEFAULT '[]'",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE catalog_models ADD COLUMN IF NOT EXISTS streaming BOOLEAN NOT NULL DEFAULT FALSE",
    )
    .execute(&pool)
    .await?;
    sqlx::query("ALTER TABLE catalog_models ADD COLUMN IF NOT EXISTS context_window BIGINT")
        .execute(&pool)
        .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS routing_policies (
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
        "ALTER TABLE routing_policies ADD COLUMN IF NOT EXISTS strategy TEXT NOT NULL DEFAULT 'deterministic_priority'",
    )
    .execute(&pool)
    .await?;
    sqlx::query("ALTER TABLE routing_policies ADD COLUMN IF NOT EXISTS max_cost DOUBLE PRECISION")
        .execute(&pool)
        .await?;
    sqlx::query("ALTER TABLE routing_policies ADD COLUMN IF NOT EXISTS max_latency_ms BIGINT")
        .execute(&pool)
        .await?;
    sqlx::query(
        "ALTER TABLE routing_policies ADD COLUMN IF NOT EXISTS require_healthy BOOLEAN NOT NULL DEFAULT FALSE",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS routing_policy_providers (
            policy_id TEXT NOT NULL,
            provider_id TEXT NOT NULL,
            position INTEGER NOT NULL,
            PRIMARY KEY (policy_id, provider_id)
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS project_routing_preferences (
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
        "CREATE TABLE IF NOT EXISTS routing_decision_logs (
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
    sqlx::query("ALTER TABLE routing_decision_logs ADD COLUMN IF NOT EXISTS requested_region TEXT")
        .execute(&pool)
        .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS routing_provider_health (
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
        "CREATE TABLE IF NOT EXISTS usage_records (
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
        "ALTER TABLE usage_records ADD COLUMN IF NOT EXISTS units BIGINT NOT NULL DEFAULT 0",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE usage_records ADD COLUMN IF NOT EXISTS amount DOUBLE PRECISION NOT NULL DEFAULT 0",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE usage_records ADD COLUMN IF NOT EXISTS input_tokens BIGINT NOT NULL DEFAULT 0",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE usage_records ADD COLUMN IF NOT EXISTS output_tokens BIGINT NOT NULL DEFAULT 0",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE usage_records ADD COLUMN IF NOT EXISTS total_tokens BIGINT NOT NULL DEFAULT 0",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE usage_records ADD COLUMN IF NOT EXISTS created_at_ms BIGINT NOT NULL DEFAULT 0",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS billing_ledger_entries (
            project_id TEXT NOT NULL,
            units BIGINT NOT NULL,
            amount DOUBLE PRECISION NOT NULL
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS billing_quota_policies (
            policy_id TEXT PRIMARY KEY NOT NULL,
            project_id TEXT NOT NULL,
            max_units BIGINT NOT NULL,
            enabled BOOLEAN NOT NULL DEFAULT TRUE
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS identity_gateway_api_keys (
            hashed_key TEXT PRIMARY KEY NOT NULL,
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
        "ALTER TABLE identity_gateway_api_keys ADD COLUMN IF NOT EXISTS label TEXT NOT NULL DEFAULT ''",
    )
    .execute(&pool)
    .await?;
    sqlx::query("ALTER TABLE identity_gateway_api_keys ADD COLUMN IF NOT EXISTS notes TEXT")
        .execute(&pool)
        .await?;
    sqlx::query(
        "ALTER TABLE identity_gateway_api_keys ADD COLUMN IF NOT EXISTS created_at_ms BIGINT NOT NULL DEFAULT 0",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE identity_gateway_api_keys ADD COLUMN IF NOT EXISTS last_used_at_ms BIGINT",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "ALTER TABLE identity_gateway_api_keys ADD COLUMN IF NOT EXISTS expires_at_ms BIGINT",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS extension_installations (
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
        "CREATE TABLE IF NOT EXISTS extension_instances (
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
        "CREATE TABLE IF NOT EXISTS service_runtime_nodes (
            node_id TEXT PRIMARY KEY NOT NULL,
            service_kind TEXT NOT NULL,
            started_at_ms BIGINT NOT NULL,
            last_seen_at_ms BIGINT NOT NULL
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_service_runtime_nodes_last_seen
         ON service_runtime_nodes (last_seen_at_ms DESC, node_id)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS extension_runtime_rollouts (
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
        "CREATE INDEX IF NOT EXISTS idx_extension_runtime_rollouts_created_at
         ON extension_runtime_rollouts (created_at_ms DESC, rollout_id)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS extension_runtime_rollout_participants (
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
        "CREATE INDEX IF NOT EXISTS idx_extension_runtime_rollout_participants_node_status
         ON extension_runtime_rollout_participants (node_id, status, updated_at_ms, rollout_id)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_extension_runtime_rollout_participants_rollout
         ON extension_runtime_rollout_participants (rollout_id, node_id)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS standalone_config_rollouts (
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
        "CREATE INDEX IF NOT EXISTS idx_standalone_config_rollouts_created_at
         ON standalone_config_rollouts (created_at_ms DESC, rollout_id)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS standalone_config_rollout_participants (
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
        "CREATE INDEX IF NOT EXISTS idx_standalone_config_rollout_participants_node_status
         ON standalone_config_rollout_participants (node_id, status, updated_at_ms, rollout_id)",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_standalone_config_rollout_participants_rollout
         ON standalone_config_rollout_participants (rollout_id, node_id)",
    )
    .execute(&pool)
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
        sqlx::query(
            "INSERT INTO catalog_channels (id, name) VALUES ($1, $2)
             ON CONFLICT(id) DO UPDATE SET name = excluded.name",
        )
        .bind(&channel.id)
        .bind(&channel.name)
        .execute(&self.pool)
        .await?;
        Ok(channel.clone())
    }

    pub async fn list_channels(&self) -> Result<Vec<Channel>> {
        let rows = sqlx::query_as::<_, (String, String)>(
            "SELECT id, name FROM catalog_channels ORDER BY id",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|(id, name)| Channel { id, name })
            .collect())
    }

    pub async fn delete_channel(&self, channel_id: &str) -> Result<bool> {
        sqlx::query("DELETE FROM catalog_provider_channel_bindings WHERE channel_id = $1")
            .bind(channel_id)
            .execute(&self.pool)
            .await?;
        let result = sqlx::query("DELETE FROM catalog_channels WHERE id = $1")
            .bind(channel_id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn insert_provider(&self, provider: &ProxyProvider) -> Result<ProxyProvider> {
        sqlx::query(
            "INSERT INTO catalog_proxy_providers (id, channel_id, extension_id, adapter_kind, base_url, display_name) VALUES ($1, $2, $3, $4, $5, $6)
             ON CONFLICT(id) DO UPDATE SET channel_id = excluded.channel_id, extension_id = excluded.extension_id, adapter_kind = excluded.adapter_kind, base_url = excluded.base_url, display_name = excluded.display_name",
        )
        .bind(&provider.id)
        .bind(&provider.channel_id)
        .bind(&provider.extension_id)
        .bind(&provider.adapter_kind)
        .bind(&provider.base_url)
        .bind(&provider.display_name)
        .execute(&self.pool)
        .await?;
        sqlx::query("DELETE FROM catalog_provider_channel_bindings WHERE provider_id = $1")
            .bind(&provider.id)
            .execute(&self.pool)
            .await?;

        for binding in provider_channel_bindings(provider) {
            sqlx::query(
                "INSERT INTO catalog_provider_channel_bindings (provider_id, channel_id, is_primary) VALUES ($1, $2, $3)
                 ON CONFLICT(provider_id, channel_id) DO UPDATE SET is_primary = excluded.is_primary",
            )
            .bind(&binding.provider_id)
            .bind(&binding.channel_id)
            .bind(binding.is_primary)
            .execute(&self.pool)
            .await?;
        }
        Ok(provider.clone())
    }

    pub async fn list_providers(&self) -> Result<Vec<ProxyProvider>> {
        let rows = sqlx::query_as::<_, (String, String, String, String, String, String)>(
            "SELECT id, channel_id, extension_id, adapter_kind, base_url, display_name FROM catalog_proxy_providers ORDER BY id",
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
            "SELECT id, channel_id, extension_id, adapter_kind, base_url, display_name FROM catalog_proxy_providers WHERE id = $1",
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
        sqlx::query("DELETE FROM credential_records WHERE provider_id = $1")
            .bind(provider_id)
            .execute(&self.pool)
            .await?;
        sqlx::query("DELETE FROM routing_policy_providers WHERE provider_id = $1")
            .bind(provider_id)
            .execute(&self.pool)
            .await?;
        sqlx::query(
            "UPDATE routing_policies SET default_provider_id = NULL WHERE default_provider_id = $1",
        )
        .bind(provider_id)
        .execute(&self.pool)
        .await?;
        sqlx::query("DELETE FROM catalog_provider_channel_bindings WHERE provider_id = $1")
            .bind(provider_id)
            .execute(&self.pool)
            .await?;
        let result = sqlx::query("DELETE FROM catalog_proxy_providers WHERE id = $1")
            .bind(provider_id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn insert_credential(
        &self,
        credential: &UpstreamCredential,
    ) -> Result<UpstreamCredential> {
        sqlx::query(
            "INSERT INTO credential_records (tenant_id, provider_id, key_reference, secret_backend, secret_local_file, secret_keyring_service, secret_master_key_id, secret_ciphertext, secret_key_version) VALUES ($1, $2, $3, $4, $5, $6, $7, NULL, NULL)
             ON CONFLICT(tenant_id, provider_id, key_reference) DO UPDATE SET secret_backend = excluded.secret_backend, secret_local_file = excluded.secret_local_file, secret_keyring_service = excluded.secret_keyring_service, secret_master_key_id = excluded.secret_master_key_id, secret_ciphertext = NULL, secret_key_version = NULL",
        )
        .bind(&credential.tenant_id)
        .bind(&credential.provider_id)
        .bind(&credential.key_reference)
        .bind(&credential.secret_backend)
        .bind(&credential.secret_local_file)
        .bind(&credential.secret_keyring_service)
        .bind(&credential.secret_master_key_id)
        .execute(&self.pool)
        .await?;
        Ok(credential.clone())
    }

    pub async fn insert_encrypted_credential(
        &self,
        credential: &UpstreamCredential,
        envelope: &SecretEnvelope,
    ) -> Result<UpstreamCredential> {
        sqlx::query(
            "INSERT INTO credential_records (tenant_id, provider_id, key_reference, secret_backend, secret_local_file, secret_keyring_service, secret_master_key_id, secret_ciphertext, secret_key_version) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
             ON CONFLICT(tenant_id, provider_id, key_reference) DO UPDATE SET secret_backend = excluded.secret_backend, secret_local_file = excluded.secret_local_file, secret_keyring_service = excluded.secret_keyring_service, secret_master_key_id = excluded.secret_master_key_id, secret_ciphertext = excluded.secret_ciphertext, secret_key_version = excluded.secret_key_version",
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
        .execute(&self.pool)
        .await?;
        Ok(credential.clone())
    }

    pub async fn list_credentials(&self) -> Result<Vec<UpstreamCredential>> {
        let rows = sqlx::query_as::<_, CredentialRow>(
            "SELECT tenant_id, provider_id, key_reference, secret_backend, secret_local_file, secret_keyring_service, secret_master_key_id FROM credential_records ORDER BY provider_id, tenant_id",
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
            "SELECT tenant_id, provider_id, key_reference, secret_backend, secret_local_file, secret_keyring_service, secret_master_key_id FROM credential_records WHERE tenant_id = $1 AND provider_id = $2 AND key_reference = $3",
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
            "SELECT secret_ciphertext, secret_key_version FROM credential_records WHERE tenant_id = $1 AND provider_id = $2 AND key_reference = $3",
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
            "SELECT tenant_id, provider_id, key_reference, secret_backend, secret_local_file, secret_keyring_service, secret_master_key_id FROM credential_records WHERE tenant_id = $1 AND provider_id = $2 ORDER BY ctid DESC LIMIT 1",
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
            "DELETE FROM credential_records WHERE tenant_id = $1 AND provider_id = $2 AND key_reference = $3",
        )
        .bind(tenant_id)
        .bind(provider_id)
        .bind(key_reference)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn insert_model(&self, model: &ModelCatalogEntry) -> Result<ModelCatalogEntry> {
        let context_window = model.context_window.map(i64::try_from).transpose()?;
        sqlx::query(
            "INSERT INTO catalog_models (external_name, provider_id, capabilities, streaming, context_window) VALUES ($1, $2, $3, $4, $5)
             ON CONFLICT(external_name, provider_id) DO UPDATE SET capabilities = excluded.capabilities, streaming = excluded.streaming, context_window = excluded.context_window",
        )
        .bind(&model.external_name)
        .bind(&model.provider_id)
        .bind(encode_model_capabilities(&model.capabilities)?)
        .bind(model.streaming)
        .bind(context_window)
        .execute(&self.pool)
        .await?;
        Ok(model.clone())
    }

    pub async fn list_models(&self) -> Result<Vec<ModelCatalogEntry>> {
        let rows = sqlx::query_as::<_, (String, String, String, bool, Option<i64>)>(
            "SELECT external_name, provider_id, capabilities, streaming, context_window
             FROM catalog_models
             ORDER BY external_name, provider_id",
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
            "SELECT external_name, provider_id, capabilities, streaming, context_window FROM catalog_models
             WHERE external_name = $1
             ORDER BY provider_id
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
        let result = sqlx::query("DELETE FROM catalog_models WHERE external_name = $1")
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
        let result =
            sqlx::query("DELETE FROM catalog_models WHERE external_name = $1 AND provider_id = $2")
                .bind(external_name)
                .bind(provider_id)
                .execute(&self.pool)
                .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn insert_routing_policy(&self, policy: &RoutingPolicy) -> Result<RoutingPolicy> {
        sqlx::query(
            "INSERT INTO routing_policies (policy_id, capability, model_pattern, enabled, priority, strategy, default_provider_id, max_cost, max_latency_ms, require_healthy) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
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

        sqlx::query("DELETE FROM routing_policy_providers WHERE policy_id = $1")
            .bind(&policy.policy_id)
            .execute(&self.pool)
            .await?;

        for (position, provider_id) in policy.ordered_provider_ids.iter().enumerate() {
            sqlx::query(
                "INSERT INTO routing_policy_providers (policy_id, provider_id, position) VALUES ($1, $2, $3)
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
             FROM routing_policies
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
            "INSERT INTO project_routing_preferences (
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
             FROM project_routing_preferences
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
            "INSERT INTO routing_decision_logs (decision_id, decision_source, tenant_id, project_id, capability, route_key, selected_provider_id, matched_policy_id, strategy, selection_seed, selection_reason, requested_region, slo_applied, slo_degraded, created_at_ms, assessments_json)
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
             FROM routing_decision_logs
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
            "INSERT INTO routing_provider_health (provider_id, extension_id, runtime, observed_at_ms, instance_id, running, healthy, message)
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
             FROM routing_provider_health
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
            "INSERT INTO usage_records (project_id, model, provider_id, units, amount, input_tokens, output_tokens, total_tokens, created_at_ms)
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
            "SELECT project_id, model, provider_id, units, amount, input_tokens, output_tokens, total_tokens, created_at_ms FROM usage_records",
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
            "INSERT INTO billing_ledger_entries (project_id, units, amount) VALUES ($1, $2, $3)",
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
            "SELECT project_id, units, amount FROM billing_ledger_entries",
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
            "INSERT INTO billing_quota_policies (policy_id, project_id, max_units, enabled)
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
             FROM billing_quota_policies
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
            "INSERT INTO tenant_records (id, name) VALUES ($1, $2)
             ON CONFLICT(id) DO UPDATE SET name = excluded.name",
        )
        .bind(&tenant.id)
        .bind(&tenant.name)
        .execute(&self.pool)
        .await?;
        Ok(tenant.clone())
    }

    pub async fn list_tenants(&self) -> Result<Vec<Tenant>> {
        let rows = sqlx::query_as::<_, (String, String)>(
            "SELECT id, name FROM tenant_records ORDER BY id",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|(id, name)| Tenant { id, name })
            .collect())
    }

    pub async fn find_tenant(&self, tenant_id: &str) -> Result<Option<Tenant>> {
        let row = sqlx::query_as::<_, (String, String)>(
            "SELECT id, name FROM tenant_records WHERE id = $1",
        )
        .bind(tenant_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|(id, name)| Tenant { id, name }))
    }

    pub async fn delete_tenant(&self, tenant_id: &str) -> Result<bool> {
        sqlx::query("DELETE FROM credential_records WHERE tenant_id = $1")
            .bind(tenant_id)
            .execute(&self.pool)
            .await?;
        let result = sqlx::query("DELETE FROM tenant_records WHERE id = $1")
            .bind(tenant_id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn insert_project(&self, project: &Project) -> Result<Project> {
        sqlx::query(
            "INSERT INTO tenant_projects (id, tenant_id, name) VALUES ($1, $2, $3)
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
            "SELECT tenant_id, id, name FROM tenant_projects ORDER BY tenant_id, id",
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
            "SELECT tenant_id, id, name FROM tenant_projects WHERE id = $1",
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
        sqlx::query("DELETE FROM identity_gateway_api_keys WHERE project_id = $1")
            .bind(project_id)
            .execute(&self.pool)
            .await?;
        sqlx::query("DELETE FROM billing_quota_policies WHERE project_id = $1")
            .bind(project_id)
            .execute(&self.pool)
            .await?;
        let result = sqlx::query("DELETE FROM tenant_projects WHERE id = $1")
            .bind(project_id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn insert_coupon(&self, coupon: &CouponCampaign) -> Result<CouponCampaign> {
        sqlx::query(
            "INSERT INTO coupon_campaigns (id, code, discount_label, audience, remaining, active, note, expires_on, created_at_ms)
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
             FROM coupon_campaigns
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
             FROM coupon_campaigns
             WHERE id = $1",
        )
        .bind(coupon_id)
        .fetch_optional(&self.pool)
        .await?;
        decode_coupon_row(row)
    }

    pub async fn delete_coupon(&self, coupon_id: &str) -> Result<bool> {
        let result = sqlx::query("DELETE FROM coupon_campaigns WHERE id = $1")
            .bind(coupon_id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn insert_portal_user(&self, user: &PortalUserRecord) -> Result<PortalUserRecord> {
        sqlx::query(
            "INSERT INTO identity_users (id, email, display_name, password_salt, password_hash, workspace_tenant_id, workspace_project_id, active, created_at_ms)
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
             FROM identity_users
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
             FROM identity_users
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
             FROM identity_users
             WHERE id = $1",
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;
        decode_portal_user_row(row)
    }

    pub async fn delete_portal_user(&self, user_id: &str) -> Result<bool> {
        let result = sqlx::query("DELETE FROM identity_users WHERE id = $1")
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn insert_admin_user(&self, user: &AdminUserRecord) -> Result<AdminUserRecord> {
        sqlx::query(
            "INSERT INTO admin_users (id, email, display_name, password_salt, password_hash, active, created_at_ms)
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
             FROM admin_users
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
             FROM admin_users
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
             FROM admin_users
             WHERE id = $1",
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;
        decode_admin_user_row(row)
    }

    pub async fn delete_admin_user(&self, user_id: &str) -> Result<bool> {
        let result = sqlx::query("DELETE FROM admin_users WHERE id = $1")
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
            "INSERT INTO identity_gateway_api_keys (hashed_key, tenant_id, project_id, environment, label, notes, created_at_ms, last_used_at_ms, expires_at_ms, active) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
             ON CONFLICT(hashed_key) DO UPDATE SET tenant_id = excluded.tenant_id, project_id = excluded.project_id, environment = excluded.environment, label = excluded.label, notes = excluded.notes, created_at_ms = excluded.created_at_ms, last_used_at_ms = excluded.last_used_at_ms, expires_at_ms = excluded.expires_at_ms, active = excluded.active",
        )
        .bind(&record.hashed_key)
        .bind(&record.tenant_id)
        .bind(&record.project_id)
        .bind(&record.environment)
        .bind(&record.label)
        .bind(&record.notes)
        .bind(i64::try_from(record.created_at_ms).unwrap_or(i64::MAX))
        .bind(record.last_used_at_ms.map(|value| i64::try_from(value).unwrap_or(i64::MAX)))
        .bind(record.expires_at_ms.map(|value| i64::try_from(value).unwrap_or(i64::MAX)))
        .bind(record.active)
        .execute(&self.pool)
        .await?;
        Ok(record.clone())
    }

    pub async fn list_gateway_api_keys(&self) -> Result<Vec<GatewayApiKeyRecord>> {
        let rows = sqlx::query_as::<_, (String, String, String, String, String, Option<String>, i64, Option<i64>, Option<i64>, bool)>(
            "SELECT tenant_id, project_id, environment, hashed_key, label, notes, created_at_ms, last_used_at_ms, expires_at_ms, active FROM identity_gateway_api_keys",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(
                |(
                    tenant_id,
                    project_id,
                    environment,
                    hashed_key,
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
        let row = sqlx::query_as::<_, (String, String, String, String, String, Option<String>, i64, Option<i64>, Option<i64>, bool)>(
            "SELECT tenant_id, project_id, environment, hashed_key, label, notes, created_at_ms, last_used_at_ms, expires_at_ms, active FROM identity_gateway_api_keys WHERE hashed_key = $1",
        )
        .bind(hashed_key)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(
            |(
                tenant_id,
                project_id,
                environment,
                hashed_key,
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
        let result = sqlx::query("DELETE FROM identity_gateway_api_keys WHERE hashed_key = $1")
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
            "INSERT INTO extension_installations (installation_id, extension_id, runtime, enabled, entrypoint, config_json) VALUES ($1, $2, $3, $4, $5, $6)
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
             FROM extension_installations
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
            "INSERT INTO extension_instances (instance_id, installation_id, extension_id, enabled, base_url, credential_ref, config_json) VALUES ($1, $2, $3, $4, $5, $6, $7)
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
             FROM extension_instances
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
            "INSERT INTO service_runtime_nodes (node_id, service_kind, started_at_ms, last_seen_at_ms)
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
             FROM service_runtime_nodes
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
            "INSERT INTO extension_runtime_rollouts (
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
             FROM extension_runtime_rollouts
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
             FROM extension_runtime_rollouts
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
            "INSERT INTO extension_runtime_rollout_participants (
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
             FROM extension_runtime_rollout_participants
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
             FROM extension_runtime_rollout_participants AS participants
             INNER JOIN extension_runtime_rollouts AS rollouts ON rollouts.rollout_id = participants.rollout_id
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
            "UPDATE extension_runtime_rollout_participants
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
            "INSERT INTO standalone_config_rollouts (
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
             FROM standalone_config_rollouts
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
             FROM standalone_config_rollouts
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
            "INSERT INTO standalone_config_rollout_participants (
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
             FROM standalone_config_rollout_participants
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
             FROM standalone_config_rollout_participants AS participants
             INNER JOIN standalone_config_rollouts AS rollouts ON rollouts.rollout_id = participants.rollout_id
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
            "UPDATE standalone_config_rollout_participants
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
