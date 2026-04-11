use super::*;

#[async_trait]
impl MarketingStore for PostgresAdminStore {
    async fn insert_coupon_template_record(
        &self,
        record: &CouponTemplateRecord,
    ) -> Result<CouponTemplateRecord> {
        sqlx::query(
            "INSERT INTO ai_marketing_coupon_template (
                coupon_template_id, template_key, status, distribution_kind,
                created_at_ms, updated_at_ms, record_json
            ) VALUES ($1, $2, $3, $4, $5, $6, $7)
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
        .execute(&self.pool)
        .await?;
        Ok(record.clone())
    }

    async fn list_coupon_template_records(&self) -> Result<Vec<CouponTemplateRecord>> {
        let rows = sqlx::query_as::<_, (String,)>(
            "SELECT record_json
             FROM ai_marketing_coupon_template
             ORDER BY updated_at_ms DESC, coupon_template_id",
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter()
            .map(|(json,)| Ok(serde_json::from_str::<CouponTemplateRecord>(&json)?))
            .collect()
    }

    async fn find_coupon_template_record(
        &self,
        coupon_template_id: &str,
    ) -> Result<Option<CouponTemplateRecord>> {
        let row = sqlx::query_as::<_, (String,)>(
            "SELECT record_json
             FROM ai_marketing_coupon_template
             WHERE coupon_template_id = $1",
        )
        .bind(coupon_template_id)
        .fetch_optional(&self.pool)
        .await?;
        row.map(|(json,)| Ok(serde_json::from_str::<CouponTemplateRecord>(&json)?))
            .transpose()
    }

    async fn find_coupon_template_record_by_template_key(
        &self,
        template_key: &str,
    ) -> Result<Option<CouponTemplateRecord>> {
        let row = sqlx::query_as::<_, (String,)>(
            "SELECT record_json
             FROM ai_marketing_coupon_template
             WHERE template_key = $1",
        )
        .bind(template_key)
        .fetch_optional(&self.pool)
        .await?;
        row.map(|(json,)| Ok(serde_json::from_str::<CouponTemplateRecord>(&json)?))
            .transpose()
    }

    async fn insert_coupon_template_lifecycle_audit_record(
        &self,
        record: &CouponTemplateLifecycleAuditRecord,
    ) -> Result<CouponTemplateLifecycleAuditRecord> {
        sqlx::query(
            "INSERT INTO ai_marketing_coupon_template_lifecycle_audit (
                audit_id, coupon_template_id, action, outcome, operator_id, request_id,
                previous_status, resulting_status, reason, decision_reasons_json,
                requested_at_ms, record_json
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
             ON CONFLICT(audit_id) DO UPDATE SET
                coupon_template_id = excluded.coupon_template_id,
                action = excluded.action,
                outcome = excluded.outcome,
                operator_id = excluded.operator_id,
                request_id = excluded.request_id,
                previous_status = excluded.previous_status,
                resulting_status = excluded.resulting_status,
                reason = excluded.reason,
                decision_reasons_json = excluded.decision_reasons_json,
                requested_at_ms = excluded.requested_at_ms,
                record_json = excluded.record_json",
        )
        .bind(&record.audit_id)
        .bind(&record.coupon_template_id)
        .bind(record.action.as_str())
        .bind(record.outcome.as_str())
        .bind(&record.operator_id)
        .bind(&record.request_id)
        .bind(coupon_template_status_as_str(record.previous_status))
        .bind(coupon_template_status_as_str(record.resulting_status))
        .bind(&record.reason)
        .bind(encode_string_list(&record.decision_reasons)?)
        .bind(i64::try_from(record.requested_at_ms)?)
        .bind(serde_json::to_string(record)?)
        .execute(&self.pool)
        .await?;
        Ok(record.clone())
    }

    async fn list_coupon_template_lifecycle_audit_records(
        &self,
    ) -> Result<Vec<CouponTemplateLifecycleAuditRecord>> {
        let rows = sqlx::query_as::<_, (String,)>(
            "SELECT record_json
             FROM ai_marketing_coupon_template_lifecycle_audit
             ORDER BY requested_at_ms DESC, audit_id DESC",
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter()
            .map(|(json,)| {
                Ok(serde_json::from_str::<CouponTemplateLifecycleAuditRecord>(&json)?)
            })
            .collect()
    }

    async fn list_coupon_template_lifecycle_audit_records_for_template(
        &self,
        coupon_template_id: &str,
    ) -> Result<Vec<CouponTemplateLifecycleAuditRecord>> {
        let rows = sqlx::query_as::<_, (String,)>(
            "SELECT record_json
             FROM ai_marketing_coupon_template_lifecycle_audit
             WHERE coupon_template_id = $1
             ORDER BY requested_at_ms DESC, audit_id DESC",
        )
        .bind(coupon_template_id)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter()
            .map(|(json,)| {
                Ok(serde_json::from_str::<CouponTemplateLifecycleAuditRecord>(&json)?)
            })
            .collect()
    }

    async fn insert_marketing_campaign_record(
        &self,
        record: &MarketingCampaignRecord,
    ) -> Result<MarketingCampaignRecord> {
        sqlx::query(
            "INSERT INTO ai_marketing_campaign (
                marketing_campaign_id, coupon_template_id, status, start_at_ms, end_at_ms,
                created_at_ms, updated_at_ms, record_json
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
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
        .execute(&self.pool)
        .await?;
        Ok(record.clone())
    }

    async fn list_marketing_campaign_records(&self) -> Result<Vec<MarketingCampaignRecord>> {
        let rows = sqlx::query_as::<_, (String,)>(
            "SELECT record_json
             FROM ai_marketing_campaign
             ORDER BY updated_at_ms DESC, marketing_campaign_id",
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter()
            .map(|(json,)| Ok(serde_json::from_str::<MarketingCampaignRecord>(&json)?))
            .collect()
    }

    async fn list_marketing_campaign_records_for_template(
        &self,
        coupon_template_id: &str,
    ) -> Result<Vec<MarketingCampaignRecord>> {
        let rows = sqlx::query_as::<_, (String,)>(
            "SELECT record_json
             FROM ai_marketing_campaign
             WHERE coupon_template_id = $1
             ORDER BY updated_at_ms DESC, marketing_campaign_id",
        )
        .bind(coupon_template_id)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter()
            .map(|(json,)| Ok(serde_json::from_str::<MarketingCampaignRecord>(&json)?))
            .collect()
    }

    async fn insert_marketing_campaign_lifecycle_audit_record(
        &self,
        record: &MarketingCampaignLifecycleAuditRecord,
    ) -> Result<MarketingCampaignLifecycleAuditRecord> {
        sqlx::query(
            "INSERT INTO ai_marketing_campaign_lifecycle_audit (
                audit_id, marketing_campaign_id, coupon_template_id, action, outcome,
                operator_id, request_id, previous_status, resulting_status, reason,
                decision_reasons_json, requested_at_ms, record_json
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
             ON CONFLICT(audit_id) DO UPDATE SET
                marketing_campaign_id = excluded.marketing_campaign_id,
                coupon_template_id = excluded.coupon_template_id,
                action = excluded.action,
                outcome = excluded.outcome,
                operator_id = excluded.operator_id,
                request_id = excluded.request_id,
                previous_status = excluded.previous_status,
                resulting_status = excluded.resulting_status,
                reason = excluded.reason,
                decision_reasons_json = excluded.decision_reasons_json,
                requested_at_ms = excluded.requested_at_ms,
                record_json = excluded.record_json",
        )
        .bind(&record.audit_id)
        .bind(&record.marketing_campaign_id)
        .bind(&record.coupon_template_id)
        .bind(record.action.as_str())
        .bind(record.outcome.as_str())
        .bind(&record.operator_id)
        .bind(&record.request_id)
        .bind(marketing_campaign_status_as_str(record.previous_status))
        .bind(marketing_campaign_status_as_str(record.resulting_status))
        .bind(&record.reason)
        .bind(encode_string_list(&record.decision_reasons)?)
        .bind(i64::try_from(record.requested_at_ms)?)
        .bind(serde_json::to_string(record)?)
        .execute(&self.pool)
        .await?;
        Ok(record.clone())
    }

    async fn list_marketing_campaign_lifecycle_audit_records(
        &self,
    ) -> Result<Vec<MarketingCampaignLifecycleAuditRecord>> {
        let rows = sqlx::query_as::<_, (String,)>(
            "SELECT record_json
             FROM ai_marketing_campaign_lifecycle_audit
             ORDER BY requested_at_ms DESC, audit_id DESC",
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter()
            .map(|(json,)| {
                Ok(serde_json::from_str::<MarketingCampaignLifecycleAuditRecord>(&json)?)
            })
            .collect()
    }

    async fn list_marketing_campaign_lifecycle_audit_records_for_campaign(
        &self,
        marketing_campaign_id: &str,
    ) -> Result<Vec<MarketingCampaignLifecycleAuditRecord>> {
        let rows = sqlx::query_as::<_, (String,)>(
            "SELECT record_json
             FROM ai_marketing_campaign_lifecycle_audit
             WHERE marketing_campaign_id = $1
             ORDER BY requested_at_ms DESC, audit_id DESC",
        )
        .bind(marketing_campaign_id)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter()
            .map(|(json,)| {
                Ok(serde_json::from_str::<MarketingCampaignLifecycleAuditRecord>(&json)?)
            })
            .collect()
    }

    async fn insert_campaign_budget_record(
        &self,
        record: &CampaignBudgetRecord,
    ) -> Result<CampaignBudgetRecord> {
        sqlx::query(
            "INSERT INTO ai_marketing_campaign_budget (
                campaign_budget_id, marketing_campaign_id, status,
                created_at_ms, updated_at_ms, record_json
            ) VALUES ($1, $2, $3, $4, $5, $6)
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
        .execute(&self.pool)
        .await?;
        Ok(record.clone())
    }

    async fn list_campaign_budget_records(&self) -> Result<Vec<CampaignBudgetRecord>> {
        let rows = sqlx::query_as::<_, (String,)>(
            "SELECT record_json
             FROM ai_marketing_campaign_budget
             ORDER BY updated_at_ms DESC, campaign_budget_id",
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter()
            .map(|(json,)| Ok(serde_json::from_str::<CampaignBudgetRecord>(&json)?))
            .collect()
    }

    async fn list_campaign_budget_records_for_campaign(
        &self,
        marketing_campaign_id: &str,
    ) -> Result<Vec<CampaignBudgetRecord>> {
        let rows = sqlx::query_as::<_, (String,)>(
            "SELECT record_json
             FROM ai_marketing_campaign_budget
             WHERE marketing_campaign_id = $1
             ORDER BY updated_at_ms DESC, campaign_budget_id",
        )
        .bind(marketing_campaign_id)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter()
            .map(|(json,)| Ok(serde_json::from_str::<CampaignBudgetRecord>(&json)?))
            .collect()
    }

    async fn insert_campaign_budget_lifecycle_audit_record(
        &self,
        record: &CampaignBudgetLifecycleAuditRecord,
    ) -> Result<CampaignBudgetLifecycleAuditRecord> {
        sqlx::query(
            "INSERT INTO ai_marketing_campaign_budget_lifecycle_audit (
                audit_id, campaign_budget_id, marketing_campaign_id, action, outcome,
                operator_id, request_id, previous_status, resulting_status, reason,
                decision_reasons_json, requested_at_ms, record_json
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
             ON CONFLICT(audit_id) DO UPDATE SET
                campaign_budget_id = excluded.campaign_budget_id,
                marketing_campaign_id = excluded.marketing_campaign_id,
                action = excluded.action,
                outcome = excluded.outcome,
                operator_id = excluded.operator_id,
                request_id = excluded.request_id,
                previous_status = excluded.previous_status,
                resulting_status = excluded.resulting_status,
                reason = excluded.reason,
                decision_reasons_json = excluded.decision_reasons_json,
                requested_at_ms = excluded.requested_at_ms,
                record_json = excluded.record_json",
        )
        .bind(&record.audit_id)
        .bind(&record.campaign_budget_id)
        .bind(&record.marketing_campaign_id)
        .bind(record.action.as_str())
        .bind(record.outcome.as_str())
        .bind(&record.operator_id)
        .bind(&record.request_id)
        .bind(campaign_budget_status_as_str(record.previous_status))
        .bind(campaign_budget_status_as_str(record.resulting_status))
        .bind(&record.reason)
        .bind(encode_string_list(&record.decision_reasons)?)
        .bind(i64::try_from(record.requested_at_ms)?)
        .bind(serde_json::to_string(record)?)
        .execute(&self.pool)
        .await?;
        Ok(record.clone())
    }

    async fn list_campaign_budget_lifecycle_audit_records(
        &self,
    ) -> Result<Vec<CampaignBudgetLifecycleAuditRecord>> {
        let rows = sqlx::query_as::<_, (String,)>(
            "SELECT record_json
             FROM ai_marketing_campaign_budget_lifecycle_audit
             ORDER BY requested_at_ms DESC, audit_id DESC",
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter()
            .map(|(json,)| {
                Ok(serde_json::from_str::<CampaignBudgetLifecycleAuditRecord>(&json)?)
            })
            .collect()
    }

    async fn list_campaign_budget_lifecycle_audit_records_for_budget(
        &self,
        campaign_budget_id: &str,
    ) -> Result<Vec<CampaignBudgetLifecycleAuditRecord>> {
        let rows = sqlx::query_as::<_, (String,)>(
            "SELECT record_json
             FROM ai_marketing_campaign_budget_lifecycle_audit
             WHERE campaign_budget_id = $1
             ORDER BY requested_at_ms DESC, audit_id DESC",
        )
        .bind(campaign_budget_id)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter()
            .map(|(json,)| {
                Ok(serde_json::from_str::<CampaignBudgetLifecycleAuditRecord>(&json)?)
            })
            .collect()
    }

    async fn insert_coupon_code_record(
        &self,
        record: &CouponCodeRecord,
    ) -> Result<CouponCodeRecord> {
        sqlx::query(
            "INSERT INTO ai_marketing_coupon_code (
                coupon_code_id, coupon_template_id, code_value, normalized_code_value, status,
                claimed_subject_scope, claimed_subject_id, expires_at_ms,
                created_at_ms, updated_at_ms, record_json
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
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
        .execute(&self.pool)
        .await?;
        Ok(record.clone())
    }

    async fn list_coupon_code_records(&self) -> Result<Vec<CouponCodeRecord>> {
        let rows = sqlx::query_as::<_, (String,)>(
            "SELECT record_json
             FROM ai_marketing_coupon_code
             ORDER BY updated_at_ms DESC, coupon_code_id",
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter()
            .map(|(json,)| Ok(serde_json::from_str::<CouponCodeRecord>(&json)?))
            .collect()
    }

    async fn find_coupon_code_record(
        &self,
        coupon_code_id: &str,
    ) -> Result<Option<CouponCodeRecord>> {
        let row = sqlx::query_as::<_, (String,)>(
            "SELECT record_json
             FROM ai_marketing_coupon_code
             WHERE coupon_code_id = $1",
        )
        .bind(coupon_code_id)
        .fetch_optional(&self.pool)
        .await?;
        row.map(|(json,)| Ok(serde_json::from_str::<CouponCodeRecord>(&json)?))
            .transpose()
    }

    async fn find_coupon_code_record_by_value(
        &self,
        code_value: &str,
    ) -> Result<Option<CouponCodeRecord>> {
        let row = sqlx::query_as::<_, (String,)>(
            "SELECT record_json
             FROM ai_marketing_coupon_code
             WHERE normalized_code_value = $1",
        )
        .bind(normalize_coupon_code_value(code_value))
        .fetch_optional(&self.pool)
        .await?;
        row.map(|(json,)| Ok(serde_json::from_str::<CouponCodeRecord>(&json)?))
            .transpose()
    }

    async fn list_redeemable_coupon_code_records_at(
        &self,
        now_ms: u64,
    ) -> Result<Vec<CouponCodeRecord>> {
        let rows = sqlx::query_as::<_, (String,)>(
            "SELECT record_json
             FROM ai_marketing_coupon_code
             WHERE status = $1
               AND (expires_at_ms IS NULL OR expires_at_ms >= $2)
             ORDER BY updated_at_ms DESC, coupon_code_id",
        )
        .bind(coupon_code_status_as_str(CouponCodeStatus::Available))
        .bind(i64::try_from(now_ms)?)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter()
            .map(|(json,)| Ok(serde_json::from_str::<CouponCodeRecord>(&json)?))
            .collect()
    }

    async fn insert_coupon_code_lifecycle_audit_record(
        &self,
        record: &CouponCodeLifecycleAuditRecord,
    ) -> Result<CouponCodeLifecycleAuditRecord> {
        sqlx::query(
            "INSERT INTO ai_marketing_coupon_code_lifecycle_audit (
                audit_id, coupon_code_id, coupon_template_id, action, outcome,
                operator_id, request_id, previous_status, resulting_status, reason,
                decision_reasons_json, requested_at_ms, record_json
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
             ON CONFLICT(audit_id) DO UPDATE SET
                coupon_code_id = excluded.coupon_code_id,
                coupon_template_id = excluded.coupon_template_id,
                action = excluded.action,
                outcome = excluded.outcome,
                operator_id = excluded.operator_id,
                request_id = excluded.request_id,
                previous_status = excluded.previous_status,
                resulting_status = excluded.resulting_status,
                reason = excluded.reason,
                decision_reasons_json = excluded.decision_reasons_json,
                requested_at_ms = excluded.requested_at_ms,
                record_json = excluded.record_json",
        )
        .bind(&record.audit_id)
        .bind(&record.coupon_code_id)
        .bind(&record.coupon_template_id)
        .bind(record.action.as_str())
        .bind(record.outcome.as_str())
        .bind(&record.operator_id)
        .bind(&record.request_id)
        .bind(coupon_code_status_as_str(record.previous_status))
        .bind(coupon_code_status_as_str(record.resulting_status))
        .bind(&record.reason)
        .bind(encode_string_list(&record.decision_reasons)?)
        .bind(i64::try_from(record.requested_at_ms)?)
        .bind(serde_json::to_string(record)?)
        .execute(&self.pool)
        .await?;
        Ok(record.clone())
    }

    async fn list_coupon_code_lifecycle_audit_records(
        &self,
    ) -> Result<Vec<CouponCodeLifecycleAuditRecord>> {
        let rows = sqlx::query_as::<_, (String,)>(
            "SELECT record_json
             FROM ai_marketing_coupon_code_lifecycle_audit
             ORDER BY requested_at_ms DESC, audit_id DESC",
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter()
            .map(|(json,)| Ok(serde_json::from_str::<CouponCodeLifecycleAuditRecord>(&json)?))
            .collect()
    }

    async fn list_coupon_code_lifecycle_audit_records_for_code(
        &self,
        coupon_code_id: &str,
    ) -> Result<Vec<CouponCodeLifecycleAuditRecord>> {
        let rows = sqlx::query_as::<_, (String,)>(
            "SELECT record_json
             FROM ai_marketing_coupon_code_lifecycle_audit
             WHERE coupon_code_id = $1
             ORDER BY requested_at_ms DESC, audit_id DESC",
        )
        .bind(coupon_code_id)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter()
            .map(|(json,)| Ok(serde_json::from_str::<CouponCodeLifecycleAuditRecord>(&json)?))
            .collect()
    }

    async fn insert_coupon_reservation_record(
        &self,
        record: &CouponReservationRecord,
    ) -> Result<CouponReservationRecord> {
        sqlx::query(
            "INSERT INTO ai_marketing_coupon_reservation (
                coupon_reservation_id, coupon_code_id, subject_scope, subject_id,
                reservation_status, expires_at_ms, created_at_ms, updated_at_ms, record_json
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
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
        .execute(&self.pool)
        .await?;
        Ok(record.clone())
    }

    async fn list_coupon_reservation_records(&self) -> Result<Vec<CouponReservationRecord>> {
        let rows = sqlx::query_as::<_, (String,)>(
            "SELECT record_json
             FROM ai_marketing_coupon_reservation
             ORDER BY updated_at_ms DESC, coupon_reservation_id",
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter()
            .map(|(json,)| Ok(serde_json::from_str::<CouponReservationRecord>(&json)?))
            .collect()
    }

    async fn find_coupon_reservation_record(
        &self,
        coupon_reservation_id: &str,
    ) -> Result<Option<CouponReservationRecord>> {
        let row = sqlx::query_as::<_, (String,)>(
            "SELECT record_json
             FROM ai_marketing_coupon_reservation
             WHERE coupon_reservation_id = $1",
        )
        .bind(coupon_reservation_id)
        .fetch_optional(&self.pool)
        .await?;
        row.map(|(json,)| Ok(serde_json::from_str::<CouponReservationRecord>(&json)?))
            .transpose()
    }

    async fn list_active_coupon_reservation_records_at(
        &self,
        now_ms: u64,
    ) -> Result<Vec<CouponReservationRecord>> {
        let rows = sqlx::query_as::<_, (String,)>(
            "SELECT record_json
             FROM ai_marketing_coupon_reservation
             WHERE reservation_status = $1
               AND expires_at_ms >= $2
             ORDER BY updated_at_ms DESC, coupon_reservation_id",
        )
        .bind(coupon_reservation_status_as_str(
            CouponReservationStatus::Reserved,
        ))
        .bind(i64::try_from(now_ms)?)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter()
            .map(|(json,)| Ok(serde_json::from_str::<CouponReservationRecord>(&json)?))
            .collect()
    }

    async fn insert_coupon_redemption_record(
        &self,
        record: &CouponRedemptionRecord,
    ) -> Result<CouponRedemptionRecord> {
        sqlx::query(
            "INSERT INTO ai_marketing_coupon_redemption (
                coupon_redemption_id, coupon_reservation_id, coupon_code_id, coupon_template_id,
                redemption_status, order_id, payment_event_id, redeemed_at_ms, updated_at_ms,
                record_json
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
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
        .execute(&self.pool)
        .await?;
        Ok(record.clone())
    }

    async fn list_coupon_redemption_records(&self) -> Result<Vec<CouponRedemptionRecord>> {
        let rows = sqlx::query_as::<_, (String,)>(
            "SELECT record_json
             FROM ai_marketing_coupon_redemption
             ORDER BY updated_at_ms DESC, coupon_redemption_id",
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter()
            .map(|(json,)| Ok(serde_json::from_str::<CouponRedemptionRecord>(&json)?))
            .collect()
    }

    async fn find_coupon_redemption_record(
        &self,
        coupon_redemption_id: &str,
    ) -> Result<Option<CouponRedemptionRecord>> {
        let row = sqlx::query_as::<_, (String,)>(
            "SELECT record_json
             FROM ai_marketing_coupon_redemption
             WHERE coupon_redemption_id = $1",
        )
        .bind(coupon_redemption_id)
        .fetch_optional(&self.pool)
        .await?;
        row.map(|(json,)| Ok(serde_json::from_str::<CouponRedemptionRecord>(&json)?))
            .transpose()
    }

    async fn insert_coupon_rollback_record(
        &self,
        record: &CouponRollbackRecord,
    ) -> Result<CouponRollbackRecord> {
        sqlx::query(
            "INSERT INTO ai_marketing_coupon_rollback (
                coupon_rollback_id, coupon_redemption_id, rollback_type, rollback_status,
                created_at_ms, updated_at_ms, record_json
            ) VALUES ($1, $2, $3, $4, $5, $6, $7)
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
        .execute(&self.pool)
        .await?;
        Ok(record.clone())
    }

    async fn list_coupon_rollback_records(&self) -> Result<Vec<CouponRollbackRecord>> {
        let rows = sqlx::query_as::<_, (String,)>(
            "SELECT record_json
             FROM ai_marketing_coupon_rollback
             ORDER BY updated_at_ms DESC, coupon_rollback_id",
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter()
            .map(|(json,)| Ok(serde_json::from_str::<CouponRollbackRecord>(&json)?))
            .collect()
    }

    async fn insert_marketing_outbox_event_record(
        &self,
        record: &MarketingOutboxEventRecord,
    ) -> Result<MarketingOutboxEventRecord> {
        sqlx::query(
            "INSERT INTO ai_marketing_outbox_event (
                marketing_outbox_event_id, aggregate_type, aggregate_id, event_type, status,
                created_at_ms, updated_at_ms, record_json
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
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
        .execute(&self.pool)
        .await?;
        Ok(record.clone())
    }

    async fn list_marketing_outbox_event_records(&self) -> Result<Vec<MarketingOutboxEventRecord>> {
        let rows = sqlx::query_as::<_, (String,)>(
            "SELECT record_json
             FROM ai_marketing_outbox_event
             ORDER BY created_at_ms ASC, marketing_outbox_event_id",
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter()
            .map(|(json,)| Ok(serde_json::from_str::<MarketingOutboxEventRecord>(&json)?))
            .collect()
    }
}

struct PostgresMarketingKernelTx<'a> {
    tx: Transaction<'a, Postgres>,
}

#[async_trait]
impl MarketingKernelTransaction for PostgresMarketingKernelTx<'_> {
    async fn upsert_coupon_template_record(
        &mut self,
        record: &CouponTemplateRecord,
    ) -> Result<CouponTemplateRecord> {
        sqlx::query(
            "INSERT INTO ai_marketing_coupon_template (
                coupon_template_id, template_key, status, distribution_kind,
                created_at_ms, updated_at_ms, record_json
            ) VALUES ($1, $2, $3, $4, $5, $6, $7)
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
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
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
             WHERE coupon_reservation_id = $1",
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
             WHERE coupon_code_id = $1",
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
             WHERE campaign_budget_id = $1",
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
             WHERE coupon_redemption_id = $1",
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
             WHERE coupon_rollback_id = $1",
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
             WHERE coupon_template_id = $1
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
             WHERE marketing_campaign_id = $1
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
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
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
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
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
            ) VALUES ($1, $2, $3, $4, $5, $6)
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
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
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
            ) VALUES ($1, $2, $3, $4, $5, $6, $7)
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
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
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

#[async_trait]
impl MarketingKernelTransactionExecutor for PostgresAdminStore {
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
            let mut tx = PostgresMarketingKernelTx {
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
use sdkwork_api_domain_marketing::{
    CampaignBudgetLifecycleAuditRecord, CouponCodeLifecycleAuditRecord,
};
