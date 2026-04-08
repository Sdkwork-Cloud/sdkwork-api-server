use super::*;

impl PostgresAdminStore {
    pub async fn insert_commerce_order(
        &self,
        order: &CommerceOrderRecord,
    ) -> Result<CommerceOrderRecord> {
        sqlx::query(
            "INSERT INTO ai_commerce_orders (
                order_id,
                project_id,
                user_id,
                target_kind,
                target_id,
                target_name,
                list_price_cents,
                payable_price_cents,
                list_price_label,
                payable_price_label,
                granted_units,
                bonus_units,
                currency_code,
                pricing_plan_id,
                pricing_plan_version,
                pricing_snapshot_json,
                applied_coupon_code,
                coupon_reservation_id,
                coupon_redemption_id,
                marketing_campaign_id,
                subsidy_amount_minor,
                payment_method_id,
                latest_payment_attempt_id,
                status,
                settlement_status,
                source,
                refundable_amount_minor,
                refunded_amount_minor,
                created_at_ms,
                updated_at_ms
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
                $11, $12, $13, $14, $15, $16, $17, $18, $19, $20,
                $21, $22, $23, $24, $25, $26, $27, $28, $29, $30
            )
            ON CONFLICT(order_id) DO UPDATE SET
                project_id = excluded.project_id,
                user_id = excluded.user_id,
                target_kind = excluded.target_kind,
                target_id = excluded.target_id,
                target_name = excluded.target_name,
                list_price_cents = excluded.list_price_cents,
                payable_price_cents = excluded.payable_price_cents,
                list_price_label = excluded.list_price_label,
                payable_price_label = excluded.payable_price_label,
                granted_units = excluded.granted_units,
                bonus_units = excluded.bonus_units,
                currency_code = excluded.currency_code,
                pricing_plan_id = excluded.pricing_plan_id,
                pricing_plan_version = excluded.pricing_plan_version,
                pricing_snapshot_json = excluded.pricing_snapshot_json,
                applied_coupon_code = excluded.applied_coupon_code,
                coupon_reservation_id = excluded.coupon_reservation_id,
                coupon_redemption_id = excluded.coupon_redemption_id,
                marketing_campaign_id = excluded.marketing_campaign_id,
                subsidy_amount_minor = excluded.subsidy_amount_minor,
                payment_method_id = excluded.payment_method_id,
                latest_payment_attempt_id = excluded.latest_payment_attempt_id,
                status = excluded.status,
                settlement_status = excluded.settlement_status,
                source = excluded.source,
                refundable_amount_minor = excluded.refundable_amount_minor,
                refunded_amount_minor = excluded.refunded_amount_minor,
                created_at_ms = excluded.created_at_ms,
                updated_at_ms = excluded.updated_at_ms",
        )
        .bind(&order.order_id)
        .bind(&order.project_id)
        .bind(&order.user_id)
        .bind(&order.target_kind)
        .bind(&order.target_id)
        .bind(&order.target_name)
        .bind(i64::try_from(order.list_price_cents)?)
        .bind(i64::try_from(order.payable_price_cents)?)
        .bind(&order.list_price_label)
        .bind(&order.payable_price_label)
        .bind(i64::try_from(order.granted_units)?)
        .bind(i64::try_from(order.bonus_units)?)
        .bind(&order.currency_code)
        .bind(&order.pricing_plan_id)
        .bind(order.pricing_plan_version.map(i64::try_from).transpose()?)
        .bind(&order.pricing_snapshot_json)
        .bind(&order.applied_coupon_code)
        .bind(&order.coupon_reservation_id)
        .bind(&order.coupon_redemption_id)
        .bind(&order.marketing_campaign_id)
        .bind(i64::try_from(order.subsidy_amount_minor)?)
        .bind(&order.payment_method_id)
        .bind(&order.latest_payment_attempt_id)
        .bind(&order.status)
        .bind(&order.settlement_status)
        .bind(&order.source)
        .bind(i64::try_from(order.refundable_amount_minor)?)
        .bind(i64::try_from(order.refunded_amount_minor)?)
        .bind(i64::try_from(order.created_at_ms)?)
        .bind(i64::try_from(order.updated_at_ms)?)
        .execute(&self.pool)
        .await?;
        Ok(order.clone())
    }

    pub async fn list_commerce_orders(&self) -> Result<Vec<CommerceOrderRecord>> {
        let rows = sqlx::query(
            "SELECT order_id, project_id, user_id, target_kind, target_id, target_name,
                    list_price_cents, payable_price_cents, list_price_label, payable_price_label,
                    granted_units, bonus_units, currency_code, pricing_plan_id,
                    pricing_plan_version, pricing_snapshot_json, applied_coupon_code,
                    coupon_reservation_id, coupon_redemption_id, marketing_campaign_id,
                    subsidy_amount_minor, payment_method_id, latest_payment_attempt_id, status,
                    settlement_status, source, refundable_amount_minor, refunded_amount_minor,
                    created_at_ms, updated_at_ms
             FROM ai_commerce_orders
             ORDER BY updated_at_ms DESC, created_at_ms DESC, order_id DESC",
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter()
            .map(Self::map_postgres_commerce_order_row)
            .collect()
    }

    pub async fn list_commerce_orders_for_project(
        &self,
        project_id: &str,
    ) -> Result<Vec<CommerceOrderRecord>> {
        let rows = sqlx::query(
            "SELECT order_id, project_id, user_id, target_kind, target_id, target_name,
                    list_price_cents, payable_price_cents, list_price_label, payable_price_label,
                    granted_units, bonus_units, currency_code, pricing_plan_id,
                    pricing_plan_version, pricing_snapshot_json, applied_coupon_code,
                    coupon_reservation_id, coupon_redemption_id, marketing_campaign_id,
                    subsidy_amount_minor, payment_method_id, latest_payment_attempt_id, status,
                    settlement_status, source, refundable_amount_minor, refunded_amount_minor,
                    created_at_ms, updated_at_ms
             FROM ai_commerce_orders
             WHERE project_id = $1
             ORDER BY updated_at_ms DESC, created_at_ms DESC, order_id DESC",
        )
        .bind(project_id)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter()
            .map(Self::map_postgres_commerce_order_row)
            .collect()
    }

    pub async fn list_recent_commerce_orders(
        &self,
        limit: usize,
    ) -> Result<Vec<CommerceOrderRecord>> {
        let rows = sqlx::query(
            "SELECT order_id, project_id, user_id, target_kind, target_id, target_name,
                    list_price_cents, payable_price_cents, list_price_label, payable_price_label,
                    granted_units, bonus_units, currency_code, pricing_plan_id,
                    pricing_plan_version, pricing_snapshot_json, applied_coupon_code,
                    coupon_reservation_id, coupon_redemption_id, marketing_campaign_id,
                    subsidy_amount_minor, payment_method_id, latest_payment_attempt_id, status,
                    settlement_status, source, refundable_amount_minor, refunded_amount_minor,
                    created_at_ms, updated_at_ms
             FROM ai_commerce_orders
             ORDER BY updated_at_ms DESC, created_at_ms DESC, order_id DESC
             LIMIT $1",
        )
        .bind(i64::try_from(limit)?)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter()
            .map(Self::map_postgres_commerce_order_row)
            .collect()
    }

    pub async fn list_commerce_orders_for_project_after(
        &self,
        project_id: &str,
        last_order_updated_at_ms: u64,
        last_order_created_at_ms: u64,
        last_order_id: &str,
    ) -> Result<Vec<CommerceOrderRecord>> {
        let rows = sqlx::query(
            "SELECT order_id, project_id, user_id, target_kind, target_id, target_name,
                    list_price_cents, payable_price_cents, list_price_label, payable_price_label,
                    granted_units, bonus_units, currency_code, pricing_plan_id,
                    pricing_plan_version, pricing_snapshot_json, applied_coupon_code,
                    coupon_reservation_id, coupon_redemption_id, marketing_campaign_id,
                    subsidy_amount_minor, payment_method_id, latest_payment_attempt_id, status,
                    settlement_status, source, refundable_amount_minor, refunded_amount_minor,
                    created_at_ms, updated_at_ms
             FROM ai_commerce_orders
             WHERE project_id = $1
               AND (
                    updated_at_ms > $2
                    OR (
                        updated_at_ms = $3
                        AND (
                            created_at_ms > $4
                            OR (created_at_ms = $5 AND order_id > $6)
                        )
                    )
               )
             ORDER BY updated_at_ms DESC, created_at_ms DESC, order_id DESC",
        )
        .bind(project_id)
        .bind(i64::try_from(last_order_updated_at_ms)?)
        .bind(i64::try_from(last_order_updated_at_ms)?)
        .bind(i64::try_from(last_order_created_at_ms)?)
        .bind(i64::try_from(last_order_created_at_ms)?)
        .bind(last_order_id)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter()
            .map(Self::map_postgres_commerce_order_row)
            .collect()
    }

    pub async fn upsert_commerce_payment_event(
        &self,
        event: &CommercePaymentEventRecord,
    ) -> Result<CommercePaymentEventRecord> {
        let result = sqlx::query(
            "INSERT INTO ai_commerce_payment_events (
                payment_event_id,
                order_id,
                project_id,
                user_id,
                provider,
                provider_event_id,
                dedupe_key,
                event_type,
                payload_json,
                processing_status,
                processing_message,
                received_at_ms,
                processed_at_ms,
                order_status_after
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
             ON CONFLICT(dedupe_key) DO UPDATE SET
                payment_event_id = excluded.payment_event_id,
                order_id = excluded.order_id,
                project_id = excluded.project_id,
                user_id = excluded.user_id,
                provider = excluded.provider,
                provider_event_id = excluded.provider_event_id,
                event_type = excluded.event_type,
                payload_json = excluded.payload_json,
                processing_status = excluded.processing_status,
                processing_message = excluded.processing_message,
                received_at_ms = excluded.received_at_ms,
                processed_at_ms = excluded.processed_at_ms,
                order_status_after = excluded.order_status_after
             WHERE ai_commerce_payment_events.order_id = excluded.order_id",
        )
        .bind(&event.payment_event_id)
        .bind(&event.order_id)
        .bind(&event.project_id)
        .bind(&event.user_id)
        .bind(&event.provider)
        .bind(&event.provider_event_id)
        .bind(&event.dedupe_key)
        .bind(&event.event_type)
        .bind(&event.payload_json)
        .bind(event.processing_status.as_str())
        .bind(&event.processing_message)
        .bind(i64::try_from(event.received_at_ms)?)
        .bind(event.processed_at_ms.map(i64::try_from).transpose()?)
        .bind(&event.order_status_after)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(anyhow!(
                "commerce payment event {} already belongs to another order",
                event.dedupe_key
            ));
        }

        Ok(event.clone())
    }

    pub async fn list_commerce_payment_events(&self) -> Result<Vec<CommercePaymentEventRecord>> {
        let rows = sqlx::query(
            "SELECT payment_event_id, order_id, project_id, user_id, provider, provider_event_id,
                    dedupe_key, event_type, payload_json, processing_status,
                    processing_message, received_at_ms, processed_at_ms, order_status_after
             FROM ai_commerce_payment_events
             ORDER BY received_at_ms DESC, payment_event_id DESC",
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter()
            .map(Self::map_postgres_commerce_payment_event_row)
            .collect()
    }

    pub async fn find_commerce_payment_event_by_dedupe_key(
        &self,
        dedupe_key: &str,
    ) -> Result<Option<CommercePaymentEventRecord>> {
        let row = sqlx::query(
            "SELECT payment_event_id, order_id, project_id, user_id, provider, provider_event_id,
                    dedupe_key, event_type, payload_json, processing_status,
                    processing_message, received_at_ms, processed_at_ms, order_status_after
             FROM ai_commerce_payment_events
             WHERE dedupe_key = $1",
        )
        .bind(dedupe_key)
        .fetch_optional(&self.pool)
        .await?;
        row.map(Self::map_postgres_commerce_payment_event_row)
            .transpose()
    }

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
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
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
        let row = sqlx::query(
            "SELECT membership_id, project_id, user_id, plan_id, plan_name, price_cents,
                    price_label, cadence, included_units, status, source, activated_at_ms,
                    updated_at_ms
             FROM ai_project_memberships
             WHERE project_id = $1",
        )
        .bind(project_id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(|row| {
            Ok(ProjectMembershipRecord {
                membership_id: row.try_get::<String, _>("membership_id")?,
                project_id: row.try_get::<String, _>("project_id")?,
                user_id: row.try_get::<String, _>("user_id")?,
                plan_id: row.try_get::<String, _>("plan_id")?,
                plan_name: row.try_get::<String, _>("plan_name")?,
                price_cents: u64::try_from(row.try_get::<i64, _>("price_cents")?)?,
                price_label: row.try_get::<String, _>("price_label")?,
                cadence: row.try_get::<String, _>("cadence")?,
                included_units: u64::try_from(row.try_get::<i64, _>("included_units")?)?,
                status: row.try_get::<String, _>("status")?,
                source: row.try_get::<String, _>("source")?,
                activated_at_ms: u64::try_from(row.try_get::<i64, _>("activated_at_ms")?)?,
                updated_at_ms: u64::try_from(row.try_get::<i64, _>("updated_at_ms")?)?,
            })
        })
        .transpose()
    }

    pub async fn delete_project_membership(&self, project_id: &str) -> Result<bool> {
        let result = sqlx::query(
            "DELETE FROM ai_project_memberships
             WHERE project_id = $1",
        )
        .bind(project_id)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }
}
