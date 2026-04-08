use super::*;

impl SqliteAdminStore {
    pub async fn insert_tenant(&self, tenant: &Tenant) -> Result<Tenant> {
        sqlx::query(
            "INSERT INTO ai_tenants (id, name) VALUES (?, ?)
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
            sqlx::query_as::<_, (String, String)>("SELECT id, name FROM ai_tenants WHERE id = ?")
                .bind(tenant_id)
                .fetch_optional(&self.pool)
                .await?;
        Ok(row.map(|(id, name)| Tenant { id, name }))
    }

    pub async fn delete_tenant(&self, tenant_id: &str) -> Result<bool> {
        sqlx::query("DELETE FROM ai_router_credential_records WHERE tenant_id = ?")
            .bind(tenant_id)
            .execute(&self.pool)
            .await?;
        let result = sqlx::query("DELETE FROM ai_tenants WHERE id = ?")
            .bind(tenant_id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn insert_project(&self, project: &Project) -> Result<Project> {
        sqlx::query(
            "INSERT INTO ai_projects (id, tenant_id, name) VALUES (?, ?, ?)
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
            "SELECT tenant_id, id, name FROM ai_projects WHERE id = ?",
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
        sqlx::query("DELETE FROM ai_app_api_keys WHERE project_id = ?")
            .bind(project_id)
            .execute(&self.pool)
            .await?;
        sqlx::query("DELETE FROM ai_billing_quota_policies WHERE project_id = ?")
            .bind(project_id)
            .execute(&self.pool)
            .await?;
        let result = sqlx::query("DELETE FROM ai_projects WHERE id = ?")
            .bind(project_id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }
}
