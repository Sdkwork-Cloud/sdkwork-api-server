use super::*;

impl PostgresAdminStore {
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
}
