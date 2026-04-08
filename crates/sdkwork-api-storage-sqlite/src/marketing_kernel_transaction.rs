use super::*;

struct SqliteMarketingKernelTx<'a> {
    tx: Transaction<'a, Sqlite>,
}

#[async_trait]
impl MarketingKernelTransaction for SqliteMarketingKernelTx<'_> {
    async fn upsert_coupon_template_record(
        &mut self,
        record: &CouponTemplateRecord,
    ) -> Result<CouponTemplateRecord> {
        sqlx::query(
            "INSERT INTO ai_marketing_coupon_template (
                coupon_template_id, template_key, status, distribution_kind,
                created_at_ms, updated_at_ms, record_json
            ) VALUES (?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(coupon_template_id) DO UPDATE SET
                template_key = excluded.template_key,
                status = excluded.status,
                distribution_kind = excluded.distribution_kind,
                created_at_ms = excluded.created_at_ms,
                updated_at_ms = excluded.updated_at_ms,
                record_json = excluded.record_json",
        )
        .bind(&record.coupon_template_id)
        .bind(&record.template_key)
        .bind(coupon_template_status_as_str(record.status))
        .bind(coupon_distribution_kind_as_str(record.distribution_kind))
        .bind(i64::try_from(record.created_at_ms)?)
        .bind(i64::try_from(record.updated_at_ms)?)
        .bind(serde_json::to_string(record)?)
        .execute(&mut *self.tx)
        .await?;
        Ok(record.clone())
    }

    async fn upsert_marketing_campaign_record(
        &mut self,
        record: &MarketingCampaignRecord,
    ) -> Result<MarketingCampaignRecord> {
        sqlx::query(
            "INSERT INTO ai_marketing_campaign (
                marketing_campaign_id, coupon_template_id, status, start_at_ms, end_at_ms,
                created_at_ms, updated_at_ms, record_json
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(marketing_campaign_id) DO UPDATE SET
                coupon_template_id = excluded.coupon_template_id,
                status = excluded.status,
                start_at_ms = excluded.start_at_ms,
                end_at_ms = excluded.end_at_ms,
                created_at_ms = excluded.created_at_ms,
                updated_at_ms = excluded.updated_at_ms,
                record_json = excluded.record_json",
        )
        .bind(&record.marketing_campaign_id)
        .bind(&record.coupon_template_id)
        .bind(marketing_campaign_status_as_str(record.status))
        .bind(record.start_at_ms.map(i64::try_from).transpose()?)
        .bind(record.end_at_ms.map(i64::try_from).transpose()?)
        .bind(i64::try_from(record.created_at_ms)?)
        .bind(i64::try_from(record.updated_at_ms)?)
        .bind(serde_json::to_string(record)?)
        .execute(&mut *self.tx)
        .await?;
        Ok(record.clone())
    }

    async fn find_coupon_reservation_record(
        &mut self,
        coupon_reservation_id: &str,
    ) -> Result<Option<CouponReservationRecord>> {
        let row = sqlx::query_as::<_, (String,)>(
            "SELECT record_json
             FROM ai_marketing_coupon_reservation
             WHERE coupon_reservation_id = ?",
        )
        .bind(coupon_reservation_id)
        .fetch_optional(&mut *self.tx)
        .await?;
        row.map(|(json,)| Ok(serde_json::from_str::<CouponReservationRecord>(&json)?))
            .transpose()
    }

    async fn find_coupon_code_record(
        &mut self,
        coupon_code_id: &str,
    ) -> Result<Option<CouponCodeRecord>> {
        let row = sqlx::query_as::<_, (String,)>(
            "SELECT record_json
             FROM ai_marketing_coupon_code
             WHERE coupon_code_id = ?",
        )
        .bind(coupon_code_id)
        .fetch_optional(&mut *self.tx)
        .await?;
        row.map(|(json,)| Ok(serde_json::from_str::<CouponCodeRecord>(&json)?))
            .transpose()
    }

    async fn find_campaign_budget_record(
        &mut self,
        campaign_budget_id: &str,
    ) -> Result<Option<CampaignBudgetRecord>> {
        let row = sqlx::query_as::<_, (String,)>(
            "SELECT record_json
             FROM ai_marketing_campaign_budget
             WHERE campaign_budget_id = ?",
        )
        .bind(campaign_budget_id)
        .fetch_optional(&mut *self.tx)
        .await?;
        row.map(|(json,)| Ok(serde_json::from_str::<CampaignBudgetRecord>(&json)?))
            .transpose()
    }

    async fn find_coupon_redemption_record(
        &mut self,
        coupon_redemption_id: &str,
    ) -> Result<Option<CouponRedemptionRecord>> {
        let row = sqlx::query_as::<_, (String,)>(
            "SELECT record_json
             FROM ai_marketing_coupon_redemption
             WHERE coupon_redemption_id = ?",
        )
        .bind(coupon_redemption_id)
        .fetch_optional(&mut *self.tx)
        .await?;
        row.map(|(json,)| Ok(serde_json::from_str::<CouponRedemptionRecord>(&json)?))
            .transpose()
    }

    async fn find_coupon_rollback_record(
        &mut self,
        coupon_rollback_id: &str,
    ) -> Result<Option<CouponRollbackRecord>> {
        let row = sqlx::query_as::<_, (String,)>(
            "SELECT record_json
             FROM ai_marketing_coupon_rollback
             WHERE coupon_rollback_id = ?",
        )
        .bind(coupon_rollback_id)
        .fetch_optional(&mut *self.tx)
        .await?;
        row.map(|(json,)| Ok(serde_json::from_str::<CouponRollbackRecord>(&json)?))
            .transpose()
    }

    async fn list_marketing_campaign_records_for_template(
        &mut self,
        coupon_template_id: &str,
    ) -> Result<Vec<MarketingCampaignRecord>> {
        let rows = sqlx::query_as::<_, (String,)>(
            "SELECT record_json
             FROM ai_marketing_campaign
             WHERE coupon_template_id = ?
             ORDER BY updated_at_ms DESC, marketing_campaign_id",
        )
        .bind(coupon_template_id)
        .fetch_all(&mut *self.tx)
        .await?;
        rows.into_iter()
            .map(|(json,)| Ok(serde_json::from_str::<MarketingCampaignRecord>(&json)?))
            .collect()
    }

    async fn list_campaign_budget_records_for_campaign(
        &mut self,
        marketing_campaign_id: &str,
    ) -> Result<Vec<CampaignBudgetRecord>> {
        let rows = sqlx::query_as::<_, (String,)>(
            "SELECT record_json
             FROM ai_marketing_campaign_budget
             WHERE marketing_campaign_id = ?
             ORDER BY updated_at_ms DESC, campaign_budget_id",
        )
        .bind(marketing_campaign_id)
        .fetch_all(&mut *self.tx)
        .await?;
        rows.into_iter()
            .map(|(json,)| Ok(serde_json::from_str::<CampaignBudgetRecord>(&json)?))
            .collect()
    }

    async fn upsert_coupon_reservation_record(
        &mut self,
        record: &CouponReservationRecord,
    ) -> Result<CouponReservationRecord> {
        sqlx::query(
            "INSERT INTO ai_marketing_coupon_reservation (
                coupon_reservation_id, coupon_code_id, subject_scope, subject_id,
                reservation_status, expires_at_ms, created_at_ms, updated_at_ms, record_json
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(coupon_reservation_id) DO UPDATE SET
                coupon_code_id = excluded.coupon_code_id,
                subject_scope = excluded.subject_scope,
                subject_id = excluded.subject_id,
                reservation_status = excluded.reservation_status,
                expires_at_ms = excluded.expires_at_ms,
                created_at_ms = excluded.created_at_ms,
                updated_at_ms = excluded.updated_at_ms,
                record_json = excluded.record_json",
        )
        .bind(&record.coupon_reservation_id)
        .bind(&record.coupon_code_id)
        .bind(marketing_subject_scope_as_str(record.subject_scope))
        .bind(&record.subject_id)
        .bind(coupon_reservation_status_as_str(record.reservation_status))
        .bind(i64::try_from(record.expires_at_ms)?)
        .bind(i64::try_from(record.created_at_ms)?)
        .bind(i64::try_from(record.updated_at_ms)?)
        .bind(serde_json::to_string(record)?)
        .execute(&mut *self.tx)
        .await?;
        Ok(record.clone())
    }

    async fn upsert_coupon_code_record(
        &mut self,
        record: &CouponCodeRecord,
    ) -> Result<CouponCodeRecord> {
        sqlx::query(
            "INSERT INTO ai_marketing_coupon_code (
                coupon_code_id, coupon_template_id, code_value, normalized_code_value, status,
                claimed_subject_scope, claimed_subject_id, expires_at_ms,
                created_at_ms, updated_at_ms, record_json
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(coupon_code_id) DO UPDATE SET
                coupon_template_id = excluded.coupon_template_id,
                code_value = excluded.code_value,
                normalized_code_value = excluded.normalized_code_value,
                status = excluded.status,
                claimed_subject_scope = excluded.claimed_subject_scope,
                claimed_subject_id = excluded.claimed_subject_id,
                expires_at_ms = excluded.expires_at_ms,
                created_at_ms = excluded.created_at_ms,
                updated_at_ms = excluded.updated_at_ms,
                record_json = excluded.record_json",
        )
        .bind(&record.coupon_code_id)
        .bind(&record.coupon_template_id)
        .bind(&record.code_value)
        .bind(normalize_coupon_code_value(&record.code_value))
        .bind(coupon_code_status_as_str(record.status))
        .bind(
            record
                .claimed_subject_scope
                .map(marketing_subject_scope_as_str),
        )
        .bind(&record.claimed_subject_id)
        .bind(record.expires_at_ms.map(i64::try_from).transpose()?)
        .bind(i64::try_from(record.created_at_ms)?)
        .bind(i64::try_from(record.updated_at_ms)?)
        .bind(serde_json::to_string(record)?)
        .execute(&mut *self.tx)
        .await?;
        Ok(record.clone())
    }

    async fn upsert_campaign_budget_record(
        &mut self,
        record: &CampaignBudgetRecord,
    ) -> Result<CampaignBudgetRecord> {
        sqlx::query(
            "INSERT INTO ai_marketing_campaign_budget (
                campaign_budget_id, marketing_campaign_id, status,
                created_at_ms, updated_at_ms, record_json
            ) VALUES (?, ?, ?, ?, ?, ?)
             ON CONFLICT(campaign_budget_id) DO UPDATE SET
                marketing_campaign_id = excluded.marketing_campaign_id,
                status = excluded.status,
                created_at_ms = excluded.created_at_ms,
                updated_at_ms = excluded.updated_at_ms,
                record_json = excluded.record_json",
        )
        .bind(&record.campaign_budget_id)
        .bind(&record.marketing_campaign_id)
        .bind(campaign_budget_status_as_str(record.status))
        .bind(i64::try_from(record.created_at_ms)?)
        .bind(i64::try_from(record.updated_at_ms)?)
        .bind(serde_json::to_string(record)?)
        .execute(&mut *self.tx)
        .await?;
        Ok(record.clone())
    }

    async fn upsert_coupon_redemption_record(
        &mut self,
        record: &CouponRedemptionRecord,
    ) -> Result<CouponRedemptionRecord> {
        sqlx::query(
            "INSERT INTO ai_marketing_coupon_redemption (
                coupon_redemption_id, coupon_reservation_id, coupon_code_id, coupon_template_id,
                redemption_status, order_id, payment_event_id, redeemed_at_ms, updated_at_ms,
                record_json
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(coupon_redemption_id) DO UPDATE SET
                coupon_reservation_id = excluded.coupon_reservation_id,
                coupon_code_id = excluded.coupon_code_id,
                coupon_template_id = excluded.coupon_template_id,
                redemption_status = excluded.redemption_status,
                order_id = excluded.order_id,
                payment_event_id = excluded.payment_event_id,
                redeemed_at_ms = excluded.redeemed_at_ms,
                updated_at_ms = excluded.updated_at_ms,
                record_json = excluded.record_json",
        )
        .bind(&record.coupon_redemption_id)
        .bind(&record.coupon_reservation_id)
        .bind(&record.coupon_code_id)
        .bind(&record.coupon_template_id)
        .bind(coupon_redemption_status_as_str(record.redemption_status))
        .bind(&record.order_id)
        .bind(&record.payment_event_id)
        .bind(i64::try_from(record.redeemed_at_ms)?)
        .bind(i64::try_from(record.updated_at_ms)?)
        .bind(serde_json::to_string(record)?)
        .execute(&mut *self.tx)
        .await?;
        Ok(record.clone())
    }

    async fn upsert_coupon_rollback_record(
        &mut self,
        record: &CouponRollbackRecord,
    ) -> Result<CouponRollbackRecord> {
        sqlx::query(
            "INSERT INTO ai_marketing_coupon_rollback (
                coupon_rollback_id, coupon_redemption_id, rollback_type, rollback_status,
                created_at_ms, updated_at_ms, record_json
            ) VALUES (?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(coupon_rollback_id) DO UPDATE SET
                coupon_redemption_id = excluded.coupon_redemption_id,
                rollback_type = excluded.rollback_type,
                rollback_status = excluded.rollback_status,
                created_at_ms = excluded.created_at_ms,
                updated_at_ms = excluded.updated_at_ms,
                record_json = excluded.record_json",
        )
        .bind(&record.coupon_rollback_id)
        .bind(&record.coupon_redemption_id)
        .bind(coupon_rollback_type_as_str(record.rollback_type))
        .bind(coupon_rollback_status_as_str(record.rollback_status))
        .bind(i64::try_from(record.created_at_ms)?)
        .bind(i64::try_from(record.updated_at_ms)?)
        .bind(serde_json::to_string(record)?)
        .execute(&mut *self.tx)
        .await?;
        Ok(record.clone())
    }

    async fn upsert_marketing_outbox_event_record(
        &mut self,
        record: &MarketingOutboxEventRecord,
    ) -> Result<MarketingOutboxEventRecord> {
        sqlx::query(
            "INSERT INTO ai_marketing_outbox_event (
                marketing_outbox_event_id, aggregate_type, aggregate_id, event_type, status,
                created_at_ms, updated_at_ms, record_json
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(marketing_outbox_event_id) DO UPDATE SET
                aggregate_type = excluded.aggregate_type,
                aggregate_id = excluded.aggregate_id,
                event_type = excluded.event_type,
                status = excluded.status,
                created_at_ms = excluded.created_at_ms,
                updated_at_ms = excluded.updated_at_ms,
                record_json = excluded.record_json",
        )
        .bind(&record.marketing_outbox_event_id)
        .bind(&record.aggregate_type)
        .bind(&record.aggregate_id)
        .bind(&record.event_type)
        .bind(marketing_outbox_event_status_as_str(record.status))
        .bind(i64::try_from(record.created_at_ms)?)
        .bind(i64::try_from(record.updated_at_ms)?)
        .bind(serde_json::to_string(record)?)
        .execute(&mut *self.tx)
        .await?;
        Ok(record.clone())
    }
}

impl MarketingKernelTransactionExecutor for SqliteAdminStore {
    fn with_marketing_kernel_transaction<'a, T, F>(
        &'a self,
        operation: F,
    ) -> sdkwork_api_storage_core::MarketingKernelTransactionFuture<'a, T>
    where
        T: Send + 'a,
        F: for<'tx> FnOnce(
                &'tx mut dyn MarketingKernelTransaction,
            )
                -> sdkwork_api_storage_core::MarketingKernelTransactionFuture<'tx, T>
            + Send
            + 'a,
    {
        Box::pin(async move {
            let mut tx = SqliteMarketingKernelTx {
                tx: self.pool.begin().await?,
            };
            let result = operation(&mut tx).await;
            match result {
                Ok(value) => {
                    tx.tx.commit().await?;
                    Ok(value)
                }
                Err(error) => {
                    tx.tx.rollback().await?;
                    Err(error)
                }
            }
        })
    }
}
