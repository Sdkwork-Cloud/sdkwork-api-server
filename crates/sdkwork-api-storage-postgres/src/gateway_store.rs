use super::*;

impl PostgresAdminStore {
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
                api_key_group_id,
                label,
                notes,
                created_at_ms,
                last_used_at_ms,
                expires_at_ms,
                active
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
             ON CONFLICT(hashed_key) DO UPDATE SET
                raw_key = excluded.raw_key,
                tenant_id = excluded.tenant_id,
                project_id = excluded.project_id,
                environment = excluded.environment,
                api_key_group_id = excluded.api_key_group_id,
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
        .bind(&record.api_key_group_id)
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
        let rows = sqlx::query_as::<_, (String, Option<String>, String, String, String, Option<String>, String, Option<String>, i64, Option<i64>, Option<i64>, bool)>(
            "SELECT hashed_key, raw_key, tenant_id, project_id, environment, api_key_group_id, label, notes, created_at_ms, last_used_at_ms, expires_at_ms, active
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
                    raw_key,
                    api_key_group_id,
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
        let row = sqlx::query_as::<_, (String, Option<String>, String, String, String, Option<String>, String, Option<String>, i64, Option<i64>, Option<i64>, bool)>(
            "SELECT hashed_key, raw_key, tenant_id, project_id, environment, api_key_group_id, label, notes, created_at_ms, last_used_at_ms, expires_at_ms, active
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
                    raw_key,
                    api_key_group_id,
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
}
