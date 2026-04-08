use super::*;

impl PostgresAdminStore {
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
}
