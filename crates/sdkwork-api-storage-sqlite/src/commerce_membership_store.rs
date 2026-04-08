use super::*;

impl SqliteAdminStore {
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
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
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
             WHERE project_id = ?",
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

    pub async fn delete_project_membership(&self, project_id: &str) -> Result<bool> {
        let result = sqlx::query(
            "DELETE FROM ai_project_memberships
             WHERE project_id = ?",
        )
        .bind(project_id)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }
}
