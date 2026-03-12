use anyhow::Result;
use sdkwork_api_domain_catalog::{Channel, ModelCatalogEntry, ProxyProvider};
use sdkwork_api_domain_credential::UpstreamCredential;
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

pub async fn run_migrations(url: &str) -> Result<SqlitePool> {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect(url)
        .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS identity_users (
            id TEXT PRIMARY KEY NOT NULL,
            email TEXT NOT NULL
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
            display_name TEXT NOT NULL
        )",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS credential_records (
            tenant_id TEXT NOT NULL,
            provider_id TEXT NOT NULL,
            key_reference TEXT NOT NULL,
            PRIMARY KEY (tenant_id, provider_id, key_reference)
        )",
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
    Ok(pool)
}

#[derive(Debug, Clone)]
pub struct SqliteAdminStore {
    pool: SqlitePool,
}

impl SqliteAdminStore {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn insert_channel(&self, channel: &Channel) -> Result<Channel> {
        sqlx::query(
            "INSERT INTO catalog_channels (id, name) VALUES (?, ?)
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
            "INSERT INTO catalog_proxy_providers (id, channel_id, display_name) VALUES (?, ?, ?)
             ON CONFLICT(id) DO UPDATE SET channel_id = excluded.channel_id, display_name = excluded.display_name",
        )
        .bind(&provider.id)
        .bind(&provider.channel_id)
        .bind(&provider.display_name)
        .execute(&self.pool)
        .await?;
        Ok(provider.clone())
    }

    pub async fn list_providers(&self) -> Result<Vec<ProxyProvider>> {
        let rows = sqlx::query_as::<_, (String, String, String)>(
            "SELECT id, channel_id, display_name FROM catalog_proxy_providers ORDER BY id",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|(id, channel_id, display_name)| ProxyProvider {
                id,
                channel_id,
                display_name,
            })
            .collect())
    }

    pub async fn insert_credential(
        &self,
        credential: &UpstreamCredential,
    ) -> Result<UpstreamCredential> {
        sqlx::query(
            "INSERT INTO credential_records (tenant_id, provider_id, key_reference) VALUES (?, ?, ?)
             ON CONFLICT(tenant_id, provider_id, key_reference) DO NOTHING",
        )
        .bind(&credential.tenant_id)
        .bind(&credential.provider_id)
        .bind(&credential.key_reference)
        .execute(&self.pool)
        .await?;
        Ok(credential.clone())
    }

    pub async fn list_credentials(&self) -> Result<Vec<UpstreamCredential>> {
        let rows = sqlx::query_as::<_, (String, String, String)>(
            "SELECT tenant_id, provider_id, key_reference FROM credential_records ORDER BY provider_id, tenant_id",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(
                |(tenant_id, provider_id, key_reference)| UpstreamCredential {
                    tenant_id,
                    provider_id,
                    key_reference,
                },
            )
            .collect())
    }

    pub async fn insert_model(&self, model: &ModelCatalogEntry) -> Result<ModelCatalogEntry> {
        sqlx::query(
            "INSERT INTO catalog_models (external_name, provider_id) VALUES (?, ?)
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
}
