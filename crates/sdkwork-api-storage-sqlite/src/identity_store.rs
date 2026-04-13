use super::*;
use sdkwork_api_domain_identity::{AdminAuditEventRecord, AdminUserRole};

type PortalUserRow = (
    String,
    String,
    String,
    String,
    String,
    String,
    String,
    i64,
    i64,
);

type AdminUserRow = (String, String, String, String, String, String, i64, i64);
type AdminAuditEventRow = (
    String,
    String,
    String,
    String,
    String,
    String,
    String,
    String,
    i64,
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
                active: active != 0,
                created_at_ms: u64::try_from(created_at_ms)?,
            })
        },
    )
    .transpose()
}

fn decode_admin_user_row(row: Option<AdminUserRow>) -> Result<Option<AdminUserRecord>> {
    row.map(
        |(
            id,
            email,
            display_name,
            password_salt,
            password_hash,
            role,
            active,
            created_at_ms,
        )| {
            Ok(AdminUserRecord {
                id,
                email,
                display_name,
                password_salt,
                password_hash,
                role: AdminUserRole::from_str(&role).map_err(anyhow::Error::msg)?,
                active: active != 0,
                created_at_ms: u64::try_from(created_at_ms)?,
            })
        },
    )
    .transpose()
}

fn decode_admin_audit_event_row(row: Option<AdminAuditEventRow>) -> Result<Option<AdminAuditEventRecord>> {
    row.map(
        |(
            event_id,
            action,
            resource_type,
            resource_id,
            approval_scope,
            actor_user_id,
            actor_email,
            actor_role,
            recorded_at_ms,
        )| {
            Ok(AdminAuditEventRecord {
                event_id,
                action,
                resource_type,
                resource_id,
                approval_scope,
                actor_user_id,
                actor_email,
                actor_role: AdminUserRole::from_str(&actor_role).map_err(anyhow::Error::msg)?,
                recorded_at_ms: u64::try_from(recorded_at_ms)?,
            })
        },
    )
    .transpose()
}

impl SqliteAdminStore {
    pub async fn insert_portal_user(&self, user: &PortalUserRecord) -> Result<PortalUserRecord> {
        sqlx::query(
            "INSERT INTO ai_portal_users (id, email, display_name, password_salt, password_hash, workspace_tenant_id, workspace_project_id, active, created_at_ms)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
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
        .bind(if user.active { 1_i64 } else { 0_i64 })
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
        let row = sqlx::query_as::<_, (String, String, String, String, String, String, String, i64, i64)>(
            "SELECT id, email, display_name, password_salt, password_hash, workspace_tenant_id, workspace_project_id, active, created_at_ms
             FROM ai_portal_users
             WHERE email = ?",
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await?;
        decode_portal_user_row(row)
    }

    pub async fn find_portal_user_by_id(&self, user_id: &str) -> Result<Option<PortalUserRecord>> {
        let row = sqlx::query_as::<_, (String, String, String, String, String, String, String, i64, i64)>(
            "SELECT id, email, display_name, password_salt, password_hash, workspace_tenant_id, workspace_project_id, active, created_at_ms
             FROM ai_portal_users
             WHERE id = ?",
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;
        decode_portal_user_row(row)
    }

    pub async fn delete_portal_user(&self, user_id: &str) -> Result<bool> {
        let result = sqlx::query("DELETE FROM ai_portal_users WHERE id = ?")
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn insert_admin_user(&self, user: &AdminUserRecord) -> Result<AdminUserRecord> {
        sqlx::query(
            "INSERT INTO ai_admin_users (id, email, display_name, password_salt, password_hash, role, active, created_at_ms)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)
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
        .bind(if user.active { 1_i64 } else { 0_i64 })
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
             WHERE email = ?",
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
             WHERE id = ?",
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;
        decode_admin_user_row(row)
    }

    pub async fn delete_admin_user(&self, user_id: &str) -> Result<bool> {
        let result = sqlx::query("DELETE FROM ai_admin_users WHERE id = ?")
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn insert_admin_audit_event(
        &self,
        record: &AdminAuditEventRecord,
    ) -> Result<AdminAuditEventRecord> {
        sqlx::query(
            "INSERT INTO ai_admin_audit_events (
                event_id,
                action,
                resource_type,
                resource_id,
                approval_scope,
                actor_user_id,
                actor_email,
                actor_role,
                recorded_at_ms
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(event_id) DO UPDATE SET
                action = excluded.action,
                resource_type = excluded.resource_type,
                resource_id = excluded.resource_id,
                approval_scope = excluded.approval_scope,
                actor_user_id = excluded.actor_user_id,
                actor_email = excluded.actor_email,
                actor_role = excluded.actor_role,
                recorded_at_ms = excluded.recorded_at_ms",
        )
        .bind(&record.event_id)
        .bind(&record.action)
        .bind(&record.resource_type)
        .bind(&record.resource_id)
        .bind(&record.approval_scope)
        .bind(&record.actor_user_id)
        .bind(&record.actor_email)
        .bind(record.actor_role.as_str())
        .bind(i64::try_from(record.recorded_at_ms)?)
        .execute(&self.pool)
        .await?;
        Ok(record.clone())
    }

    pub async fn list_admin_audit_events(&self) -> Result<Vec<AdminAuditEventRecord>> {
        let rows = sqlx::query_as::<_, AdminAuditEventRow>(
            "SELECT event_id, action, resource_type, resource_id, approval_scope, actor_user_id, actor_email, actor_role, recorded_at_ms
             FROM ai_admin_audit_events
             ORDER BY recorded_at_ms DESC, event_id DESC",
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|row| decode_admin_audit_event_row(Some(row)))
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .map(|row: Option<AdminAuditEventRecord>| {
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
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
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
        .bind(if record.active { 1_i64 } else { 0_i64 })
        .execute(&self.pool)
        .await?;
        Ok(record.clone())
    }

    pub async fn list_gateway_api_keys(&self) -> Result<Vec<GatewayApiKeyRecord>> {
        let rows = sqlx::query_as::<_, (String, Option<String>, String, String, String, Option<String>, String, Option<String>, i64, Option<i64>, Option<i64>, i64)>(
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
                    active: active != 0,
                },
            )
            .collect())
    }

    pub async fn find_gateway_api_key(
        &self,
        hashed_key: &str,
    ) -> Result<Option<GatewayApiKeyRecord>> {
        let row = sqlx::query_as::<_, (String, Option<String>, String, String, String, Option<String>, String, Option<String>, i64, Option<i64>, Option<i64>, i64)>(
            "SELECT hashed_key, raw_key, tenant_id, project_id, environment, api_key_group_id, label, notes, created_at_ms, last_used_at_ms, expires_at_ms, active
             FROM ai_app_api_keys
             WHERE hashed_key = ?",
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
                    active: active != 0,
                }
            },
        ))
    }

    pub async fn delete_gateway_api_key(&self, hashed_key: &str) -> Result<bool> {
        let result = sqlx::query("DELETE FROM ai_app_api_keys WHERE hashed_key = ?")
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
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
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
        .bind(if record.active { 1_i64 } else { 0_i64 })
        .bind(i64::try_from(record.created_at_ms).unwrap_or(i64::MAX))
        .bind(i64::try_from(record.updated_at_ms).unwrap_or(i64::MAX))
        .execute(&self.pool)
        .await?;
        Ok(record.clone())
    }

    pub async fn list_api_key_groups(&self) -> Result<Vec<ApiKeyGroupRecord>> {
        let rows = sqlx::query_as::<_, (String, String, String, String, String, String, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, i64, i64, i64)>(
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
                    active: active != 0,
                    created_at_ms: u64::try_from(created_at_ms).unwrap_or_default(),
                    updated_at_ms: u64::try_from(updated_at_ms).unwrap_or_default(),
                },
            )
            .collect())
    }

    pub async fn find_api_key_group(&self, group_id: &str) -> Result<Option<ApiKeyGroupRecord>> {
        let row = sqlx::query_as::<_, (String, String, String, String, String, String, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, i64, i64, i64)>(
            "SELECT group_id, tenant_id, project_id, environment, name, slug, description, color, default_capability_scope, default_routing_profile_id, default_accounting_mode, active, created_at_ms, updated_at_ms
             FROM ai_app_api_key_groups
             WHERE group_id = ?",
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
                active: active != 0,
                created_at_ms: u64::try_from(created_at_ms).unwrap_or_default(),
                updated_at_ms: u64::try_from(updated_at_ms).unwrap_or_default(),
            },
        ))
    }

    pub async fn delete_api_key_group(&self, group_id: &str) -> Result<bool> {
        let result = sqlx::query("DELETE FROM ai_app_api_key_groups WHERE group_id = ?")
            .bind(group_id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }
}
