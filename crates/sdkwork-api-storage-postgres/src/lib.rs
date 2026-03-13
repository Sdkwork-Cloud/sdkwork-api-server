use anyhow::Result;
use sdkwork_api_domain_billing::LedgerEntry;
use sdkwork_api_domain_catalog::{Channel, ModelCatalogEntry, ProxyProvider};
use sdkwork_api_domain_credential::UpstreamCredential;
use sdkwork_api_domain_identity::GatewayApiKeyRecord;
use sdkwork_api_domain_tenant::{Project, Tenant};
use sdkwork_api_domain_usage::UsageRecord;
use sdkwork_api_secret_core::SecretEnvelope;
use sdkwork_api_storage_core::{AdminStore, StorageDialect};
use sqlx::{postgres::PgPoolOptions, PgPool};

pub fn dialect_name() -> &'static str {
    "postgres"
}

pub async fn run_migrations(url: &str) -> Result<PgPool> {
    let pool = PgPoolOptions::new().max_connections(5).connect(url).await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS identity_users (
            id TEXT PRIMARY KEY NOT NULL,
            email TEXT NOT NULL
        )",
    )
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
            adapter_kind TEXT NOT NULL DEFAULT 'openai',
            base_url TEXT NOT NULL DEFAULT 'http://localhost',
            display_name TEXT NOT NULL
        )",
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
        "CREATE TABLE IF NOT EXISTS credential_records (
            tenant_id TEXT NOT NULL,
            provider_id TEXT NOT NULL,
            key_reference TEXT NOT NULL,
            secret_backend TEXT NOT NULL DEFAULT 'database_encrypted',
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
        "CREATE TABLE IF NOT EXISTS usage_records (
            project_id TEXT NOT NULL,
            model TEXT NOT NULL,
            provider_id TEXT NOT NULL
        )",
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
        "CREATE TABLE IF NOT EXISTS identity_gateway_api_keys (
            hashed_key TEXT PRIMARY KEY NOT NULL,
            tenant_id TEXT NOT NULL,
            project_id TEXT NOT NULL,
            environment TEXT NOT NULL,
            active BOOLEAN NOT NULL
        )",
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

    pub async fn insert_provider(&self, provider: &ProxyProvider) -> Result<ProxyProvider> {
        sqlx::query(
            "INSERT INTO catalog_proxy_providers (id, channel_id, adapter_kind, base_url, display_name) VALUES ($1, $2, $3, $4, $5)
             ON CONFLICT(id) DO UPDATE SET channel_id = excluded.channel_id, adapter_kind = excluded.adapter_kind, base_url = excluded.base_url, display_name = excluded.display_name",
        )
        .bind(&provider.id)
        .bind(&provider.channel_id)
        .bind(&provider.adapter_kind)
        .bind(&provider.base_url)
        .bind(&provider.display_name)
        .execute(&self.pool)
        .await?;
        Ok(provider.clone())
    }

    pub async fn list_providers(&self) -> Result<Vec<ProxyProvider>> {
        let rows = sqlx::query_as::<_, (String, String, String, String, String)>(
            "SELECT id, channel_id, adapter_kind, base_url, display_name FROM catalog_proxy_providers ORDER BY id",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(
                |(id, channel_id, adapter_kind, base_url, display_name)| ProxyProvider {
                    id,
                    channel_id,
                    adapter_kind,
                    base_url,
                    display_name,
                },
            )
            .collect())
    }

    pub async fn find_provider(&self, provider_id: &str) -> Result<Option<ProxyProvider>> {
        let row = sqlx::query_as::<_, (String, String, String, String, String)>(
            "SELECT id, channel_id, adapter_kind, base_url, display_name FROM catalog_proxy_providers WHERE id = $1",
        )
        .bind(provider_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(
            |(id, channel_id, adapter_kind, base_url, display_name)| ProxyProvider {
                id,
                channel_id,
                adapter_kind,
                base_url,
                display_name,
            },
        ))
    }

    pub async fn insert_credential(
        &self,
        credential: &UpstreamCredential,
    ) -> Result<UpstreamCredential> {
        sqlx::query(
            "INSERT INTO credential_records (tenant_id, provider_id, key_reference, secret_backend, secret_ciphertext, secret_key_version) VALUES ($1, $2, $3, $4, NULL, NULL)
             ON CONFLICT(tenant_id, provider_id, key_reference) DO UPDATE SET secret_backend = excluded.secret_backend, secret_ciphertext = NULL, secret_key_version = NULL",
        )
        .bind(&credential.tenant_id)
        .bind(&credential.provider_id)
        .bind(&credential.key_reference)
        .bind(&credential.secret_backend)
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
            "INSERT INTO credential_records (tenant_id, provider_id, key_reference, secret_backend, secret_ciphertext, secret_key_version) VALUES ($1, $2, $3, $4, $5, $6)
             ON CONFLICT(tenant_id, provider_id, key_reference) DO UPDATE SET secret_backend = excluded.secret_backend, secret_ciphertext = excluded.secret_ciphertext, secret_key_version = excluded.secret_key_version",
        )
        .bind(&credential.tenant_id)
        .bind(&credential.provider_id)
        .bind(&credential.key_reference)
        .bind(&credential.secret_backend)
        .bind(&envelope.ciphertext)
        .bind(i32::try_from(envelope.key_version)?)
        .execute(&self.pool)
        .await?;
        Ok(credential.clone())
    }

    pub async fn list_credentials(&self) -> Result<Vec<UpstreamCredential>> {
        let rows = sqlx::query_as::<_, (String, String, String, String)>(
            "SELECT tenant_id, provider_id, key_reference, secret_backend FROM credential_records ORDER BY provider_id, tenant_id",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(
                |(tenant_id, provider_id, key_reference, secret_backend)| UpstreamCredential {
                    tenant_id,
                    provider_id,
                    key_reference,
                    secret_backend,
                },
            )
            .collect())
    }

    pub async fn find_credential(
        &self,
        tenant_id: &str,
        provider_id: &str,
        key_reference: &str,
    ) -> Result<Option<UpstreamCredential>> {
        let row = sqlx::query_as::<_, (String, String, String, String)>(
            "SELECT tenant_id, provider_id, key_reference, secret_backend FROM credential_records WHERE tenant_id = $1 AND provider_id = $2 AND key_reference = $3",
        )
        .bind(tenant_id)
        .bind(provider_id)
        .bind(key_reference)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(
            |(tenant_id, provider_id, key_reference, secret_backend)| UpstreamCredential {
                tenant_id,
                provider_id,
                key_reference,
                secret_backend,
            },
        ))
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
        let row = sqlx::query_as::<_, (String, String, String, String)>(
            "SELECT tenant_id, provider_id, key_reference, secret_backend FROM credential_records WHERE tenant_id = $1 AND provider_id = $2 ORDER BY ctid DESC LIMIT 1",
        )
        .bind(tenant_id)
        .bind(provider_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(
            |(tenant_id, provider_id, key_reference, secret_backend)| UpstreamCredential {
                tenant_id,
                provider_id,
                key_reference,
                secret_backend,
            },
        ))
    }

    pub async fn insert_model(&self, model: &ModelCatalogEntry) -> Result<ModelCatalogEntry> {
        sqlx::query(
            "INSERT INTO catalog_models (external_name, provider_id) VALUES ($1, $2)
             ON CONFLICT(external_name, provider_id) DO NOTHING",
        )
        .bind(&model.external_name)
        .bind(&model.provider_id)
        .execute(&self.pool)
        .await?;
        Ok(model.clone())
    }

    pub async fn list_models(&self) -> Result<Vec<ModelCatalogEntry>> {
        let rows = sqlx::query_as::<_, (String, String)>(
            "SELECT external_name, provider_id FROM catalog_models ORDER BY external_name, provider_id",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|(external_name, provider_id)| ModelCatalogEntry {
                external_name,
                provider_id,
            })
            .collect())
    }

    pub async fn insert_usage_record(&self, record: &UsageRecord) -> Result<UsageRecord> {
        sqlx::query(
            "INSERT INTO usage_records (project_id, model, provider_id) VALUES ($1, $2, $3)",
        )
        .bind(&record.project_id)
        .bind(&record.model)
        .bind(&record.provider)
        .execute(&self.pool)
        .await?;
        Ok(record.clone())
    }

    pub async fn list_usage_records(&self) -> Result<Vec<UsageRecord>> {
        let rows = sqlx::query_as::<_, (String, String, String)>(
            "SELECT project_id, model, provider_id FROM usage_records",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|(project_id, model, provider)| UsageRecord {
                project_id,
                model,
                provider,
            })
            .collect())
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

    pub async fn insert_gateway_api_key(
        &self,
        record: &GatewayApiKeyRecord,
    ) -> Result<GatewayApiKeyRecord> {
        sqlx::query(
            "INSERT INTO identity_gateway_api_keys (hashed_key, tenant_id, project_id, environment, active) VALUES ($1, $2, $3, $4, $5)
             ON CONFLICT(hashed_key) DO UPDATE SET tenant_id = excluded.tenant_id, project_id = excluded.project_id, environment = excluded.environment, active = excluded.active",
        )
        .bind(&record.hashed_key)
        .bind(&record.tenant_id)
        .bind(&record.project_id)
        .bind(&record.environment)
        .bind(record.active)
        .execute(&self.pool)
        .await?;
        Ok(record.clone())
    }

    pub async fn list_gateway_api_keys(&self) -> Result<Vec<GatewayApiKeyRecord>> {
        let rows = sqlx::query_as::<_, (String, String, String, String, bool)>(
            "SELECT tenant_id, project_id, environment, hashed_key, active FROM identity_gateway_api_keys",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(
                |(tenant_id, project_id, environment, hashed_key, active)| GatewayApiKeyRecord {
                    tenant_id,
                    project_id,
                    environment,
                    hashed_key,
                    active,
                },
            )
            .collect())
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

    async fn insert_provider(&self, provider: &ProxyProvider) -> Result<ProxyProvider> {
        PostgresAdminStore::insert_provider(self, provider).await
    }

    async fn list_providers(&self) -> Result<Vec<ProxyProvider>> {
        PostgresAdminStore::list_providers(self).await
    }

    async fn find_provider(&self, provider_id: &str) -> Result<Option<ProxyProvider>> {
        PostgresAdminStore::find_provider(self, provider_id).await
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

    async fn insert_model(&self, model: &ModelCatalogEntry) -> Result<ModelCatalogEntry> {
        PostgresAdminStore::insert_model(self, model).await
    }

    async fn list_models(&self) -> Result<Vec<ModelCatalogEntry>> {
        PostgresAdminStore::list_models(self).await
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

    async fn insert_tenant(&self, tenant: &Tenant) -> Result<Tenant> {
        PostgresAdminStore::insert_tenant(self, tenant).await
    }

    async fn list_tenants(&self) -> Result<Vec<Tenant>> {
        PostgresAdminStore::list_tenants(self).await
    }

    async fn insert_project(&self, project: &Project) -> Result<Project> {
        PostgresAdminStore::insert_project(self, project).await
    }

    async fn list_projects(&self) -> Result<Vec<Project>> {
        PostgresAdminStore::list_projects(self).await
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
}
