use super::*;

impl PostgresAdminStore {
    pub async fn upsert_commerce_webhook_inbox(
        &self,
        record: &CommerceWebhookInboxRecord,
    ) -> Result<CommerceWebhookInboxRecord> {
        if self
            .find_commerce_webhook_inbox(&record.webhook_inbox_id)
            .await?
            .is_some()
        {
            sqlx::query(
                "UPDATE ai_commerce_webhook_inbox
                 SET provider = $1, payment_method_id = $2, provider_event_id = $3,
                     dedupe_key = $4, signature_header = $5, payload_json = $6,
                     processing_status = $7, retry_count = $8, max_retry_count = $9,
                     last_error_message = $10, next_retry_at_ms = $11,
                     first_received_at_ms = $12, last_received_at_ms = $13,
                     processed_at_ms = $14
                 WHERE webhook_inbox_id = $15",
            )
            .bind(&record.provider)
            .bind(&record.payment_method_id)
            .bind(&record.provider_event_id)
            .bind(&record.dedupe_key)
            .bind(&record.signature_header)
            .bind(&record.payload_json)
            .bind(&record.processing_status)
            .bind(i64::from(record.retry_count))
            .bind(i64::from(record.max_retry_count))
            .bind(&record.last_error_message)
            .bind(record.next_retry_at_ms.map(i64::try_from).transpose()?)
            .bind(i64::try_from(record.first_received_at_ms)?)
            .bind(i64::try_from(record.last_received_at_ms)?)
            .bind(record.processed_at_ms.map(i64::try_from).transpose()?)
            .bind(&record.webhook_inbox_id)
            .execute(&self.pool)
            .await?;
            return Ok(record.clone());
        }

        if let Some(existing) = self
            .find_commerce_webhook_inbox_by_dedupe_key(&record.dedupe_key)
            .await?
        {
            if existing.provider != record.provider {
                return Err(anyhow!(
                    "webhook dedupe key {} already belongs to another provider",
                    record.dedupe_key
                ));
            }

            let merged = CommerceWebhookInboxRecord {
                webhook_inbox_id: existing.webhook_inbox_id.clone(),
                provider: existing.provider.clone(),
                payment_method_id: record
                    .payment_method_id
                    .clone()
                    .or(existing.payment_method_id.clone()),
                provider_event_id: record
                    .provider_event_id
                    .clone()
                    .or(existing.provider_event_id.clone()),
                dedupe_key: existing.dedupe_key.clone(),
                signature_header: record
                    .signature_header
                    .clone()
                    .or(existing.signature_header.clone()),
                payload_json: if record.payload_json.trim().is_empty()
                    || record.payload_json.trim() == "{}"
                {
                    existing.payload_json.clone()
                } else {
                    record.payload_json.clone()
                },
                processing_status: existing.processing_status.clone(),
                retry_count: existing.retry_count,
                max_retry_count: existing.max_retry_count.max(record.max_retry_count),
                last_error_message: existing.last_error_message.clone(),
                next_retry_at_ms: existing.next_retry_at_ms,
                first_received_at_ms: existing.first_received_at_ms,
                last_received_at_ms: existing.last_received_at_ms.max(record.last_received_at_ms),
                processed_at_ms: existing.processed_at_ms,
            };
            sqlx::query(
                "UPDATE ai_commerce_webhook_inbox
                 SET provider = $1, payment_method_id = $2, provider_event_id = $3,
                     dedupe_key = $4, signature_header = $5, payload_json = $6,
                     processing_status = $7, retry_count = $8, max_retry_count = $9,
                     last_error_message = $10, next_retry_at_ms = $11,
                     first_received_at_ms = $12, last_received_at_ms = $13,
                     processed_at_ms = $14
                 WHERE webhook_inbox_id = $15",
            )
            .bind(&merged.provider)
            .bind(&merged.payment_method_id)
            .bind(&merged.provider_event_id)
            .bind(&merged.dedupe_key)
            .bind(&merged.signature_header)
            .bind(&merged.payload_json)
            .bind(&merged.processing_status)
            .bind(i64::from(merged.retry_count))
            .bind(i64::from(merged.max_retry_count))
            .bind(&merged.last_error_message)
            .bind(merged.next_retry_at_ms.map(i64::try_from).transpose()?)
            .bind(i64::try_from(merged.first_received_at_ms)?)
            .bind(i64::try_from(merged.last_received_at_ms)?)
            .bind(merged.processed_at_ms.map(i64::try_from).transpose()?)
            .bind(&merged.webhook_inbox_id)
            .execute(&self.pool)
            .await?;
            return Ok(merged);
        }

        sqlx::query(
            "INSERT INTO ai_commerce_webhook_inbox (
                webhook_inbox_id,
                provider,
                payment_method_id,
                provider_event_id,
                dedupe_key,
                signature_header,
                payload_json,
                processing_status,
                retry_count,
                max_retry_count,
                last_error_message,
                next_retry_at_ms,
                first_received_at_ms,
                last_received_at_ms,
                processed_at_ms
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
                $11, $12, $13, $14, $15
            )",
        )
        .bind(&record.webhook_inbox_id)
        .bind(&record.provider)
        .bind(&record.payment_method_id)
        .bind(&record.provider_event_id)
        .bind(&record.dedupe_key)
        .bind(&record.signature_header)
        .bind(&record.payload_json)
        .bind(&record.processing_status)
        .bind(i64::from(record.retry_count))
        .bind(i64::from(record.max_retry_count))
        .bind(&record.last_error_message)
        .bind(record.next_retry_at_ms.map(i64::try_from).transpose()?)
        .bind(i64::try_from(record.first_received_at_ms)?)
        .bind(i64::try_from(record.last_received_at_ms)?)
        .bind(record.processed_at_ms.map(i64::try_from).transpose()?)
        .execute(&self.pool)
        .await?;
        Ok(record.clone())
    }

    pub async fn list_commerce_webhook_inbox_records(
        &self,
    ) -> Result<Vec<CommerceWebhookInboxRecord>> {
        let rows = sqlx::query(
            "SELECT webhook_inbox_id, provider, payment_method_id, provider_event_id, dedupe_key,
                    signature_header, payload_json, processing_status, retry_count,
                    max_retry_count, last_error_message, next_retry_at_ms, first_received_at_ms,
                    last_received_at_ms, processed_at_ms
             FROM ai_commerce_webhook_inbox
             ORDER BY last_received_at_ms DESC, webhook_inbox_id DESC",
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter()
            .map(Self::map_postgres_commerce_webhook_inbox_row)
            .collect()
    }

    pub async fn find_commerce_webhook_inbox(
        &self,
        webhook_inbox_id: &str,
    ) -> Result<Option<CommerceWebhookInboxRecord>> {
        let row = sqlx::query(
            "SELECT webhook_inbox_id, provider, payment_method_id, provider_event_id, dedupe_key,
                    signature_header, payload_json, processing_status, retry_count,
                    max_retry_count, last_error_message, next_retry_at_ms, first_received_at_ms,
                    last_received_at_ms, processed_at_ms
             FROM ai_commerce_webhook_inbox
             WHERE webhook_inbox_id = $1",
        )
        .bind(webhook_inbox_id)
        .fetch_optional(&self.pool)
        .await?;
        row.map(Self::map_postgres_commerce_webhook_inbox_row)
            .transpose()
    }

    pub async fn find_commerce_webhook_inbox_by_dedupe_key(
        &self,
        dedupe_key: &str,
    ) -> Result<Option<CommerceWebhookInboxRecord>> {
        let row = sqlx::query(
            "SELECT webhook_inbox_id, provider, payment_method_id, provider_event_id, dedupe_key,
                    signature_header, payload_json, processing_status, retry_count,
                    max_retry_count, last_error_message, next_retry_at_ms, first_received_at_ms,
                    last_received_at_ms, processed_at_ms
             FROM ai_commerce_webhook_inbox
             WHERE dedupe_key = $1",
        )
        .bind(dedupe_key)
        .fetch_optional(&self.pool)
        .await?;
        row.map(Self::map_postgres_commerce_webhook_inbox_row)
            .transpose()
    }

    pub async fn insert_commerce_webhook_delivery_attempt(
        &self,
        record: &CommerceWebhookDeliveryAttemptRecord,
    ) -> Result<CommerceWebhookDeliveryAttemptRecord> {
        sqlx::query(
            "INSERT INTO ai_commerce_webhook_delivery_attempts (
                delivery_attempt_id,
                webhook_inbox_id,
                processing_status,
                response_code,
                error_message,
                started_at_ms,
                finished_at_ms
            ) VALUES ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind(&record.delivery_attempt_id)
        .bind(&record.webhook_inbox_id)
        .bind(&record.processing_status)
        .bind(record.response_code.map(i32::from))
        .bind(&record.error_message)
        .bind(i64::try_from(record.started_at_ms)?)
        .bind(record.finished_at_ms.map(i64::try_from).transpose()?)
        .execute(&self.pool)
        .await?;
        Ok(record.clone())
    }

    pub async fn list_commerce_webhook_delivery_attempts(
        &self,
        webhook_inbox_id: &str,
    ) -> Result<Vec<CommerceWebhookDeliveryAttemptRecord>> {
        let rows = sqlx::query(
            "SELECT delivery_attempt_id, webhook_inbox_id, processing_status, response_code,
                    error_message, started_at_ms, finished_at_ms
             FROM ai_commerce_webhook_delivery_attempts
             WHERE webhook_inbox_id = $1
             ORDER BY started_at_ms DESC, delivery_attempt_id DESC",
        )
        .bind(webhook_inbox_id)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter()
            .map(Self::map_postgres_commerce_webhook_delivery_attempt_row)
            .collect()
    }

    pub async fn upsert_commerce_refund(
        &self,
        refund: &CommerceRefundRecord,
    ) -> Result<CommerceRefundRecord> {
        if self
            .find_commerce_refund(&refund.refund_id)
            .await?
            .is_some()
        {
            sqlx::query(
                "UPDATE ai_commerce_refunds
                 SET order_id = $1, payment_attempt_id = $2, payment_method_id = $3,
                     provider = $4, provider_refund_id = $5, idempotency_key = $6,
                     reason = $7, status = $8, amount_minor = $9, currency_code = $10,
                     request_payload_json = $11, response_payload_json = $12,
                     created_at_ms = $13, updated_at_ms = $14, completed_at_ms = $15
                 WHERE refund_id = $16",
            )
            .bind(&refund.order_id)
            .bind(&refund.payment_attempt_id)
            .bind(&refund.payment_method_id)
            .bind(&refund.provider)
            .bind(&refund.provider_refund_id)
            .bind(&refund.idempotency_key)
            .bind(&refund.reason)
            .bind(&refund.status)
            .bind(i64::try_from(refund.amount_minor)?)
            .bind(&refund.currency_code)
            .bind(&refund.request_payload_json)
            .bind(&refund.response_payload_json)
            .bind(i64::try_from(refund.created_at_ms)?)
            .bind(i64::try_from(refund.updated_at_ms)?)
            .bind(refund.completed_at_ms.map(i64::try_from).transpose()?)
            .bind(&refund.refund_id)
            .execute(&self.pool)
            .await?;
            return Ok(refund.clone());
        }

        if let Some(existing) = self
            .find_commerce_refund_by_idempotency_key(&refund.idempotency_key)
            .await?
        {
            if existing.order_id == refund.order_id && existing.provider == refund.provider {
                return Ok(existing);
            }
            return Err(anyhow!(
                "refund idempotency key {} already belongs to another order or provider",
                refund.idempotency_key
            ));
        }

        sqlx::query(
            "INSERT INTO ai_commerce_refunds (
                refund_id,
                order_id,
                payment_attempt_id,
                payment_method_id,
                provider,
                provider_refund_id,
                idempotency_key,
                reason,
                status,
                amount_minor,
                currency_code,
                request_payload_json,
                response_payload_json,
                created_at_ms,
                updated_at_ms,
                completed_at_ms
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8,
                $9, $10, $11, $12, $13, $14, $15, $16
            )",
        )
        .bind(&refund.refund_id)
        .bind(&refund.order_id)
        .bind(&refund.payment_attempt_id)
        .bind(&refund.payment_method_id)
        .bind(&refund.provider)
        .bind(&refund.provider_refund_id)
        .bind(&refund.idempotency_key)
        .bind(&refund.reason)
        .bind(&refund.status)
        .bind(i64::try_from(refund.amount_minor)?)
        .bind(&refund.currency_code)
        .bind(&refund.request_payload_json)
        .bind(&refund.response_payload_json)
        .bind(i64::try_from(refund.created_at_ms)?)
        .bind(i64::try_from(refund.updated_at_ms)?)
        .bind(refund.completed_at_ms.map(i64::try_from).transpose()?)
        .execute(&self.pool)
        .await?;
        Ok(refund.clone())
    }

    pub async fn list_commerce_refunds(&self) -> Result<Vec<CommerceRefundRecord>> {
        let rows = sqlx::query(
            "SELECT refund_id, order_id, payment_attempt_id, payment_method_id, provider,
                    provider_refund_id, idempotency_key, reason, status, amount_minor,
                    currency_code, request_payload_json, response_payload_json, created_at_ms,
                    updated_at_ms, completed_at_ms
             FROM ai_commerce_refunds
             ORDER BY updated_at_ms DESC, created_at_ms DESC, refund_id DESC",
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter()
            .map(Self::map_postgres_commerce_refund_row)
            .collect()
    }

    pub async fn find_commerce_refund(
        &self,
        refund_id: &str,
    ) -> Result<Option<CommerceRefundRecord>> {
        let row = sqlx::query(
            "SELECT refund_id, order_id, payment_attempt_id, payment_method_id, provider,
                    provider_refund_id, idempotency_key, reason, status, amount_minor,
                    currency_code, request_payload_json, response_payload_json, created_at_ms,
                    updated_at_ms, completed_at_ms
             FROM ai_commerce_refunds
             WHERE refund_id = $1",
        )
        .bind(refund_id)
        .fetch_optional(&self.pool)
        .await?;
        row.map(Self::map_postgres_commerce_refund_row).transpose()
    }

    pub async fn find_commerce_refund_by_idempotency_key(
        &self,
        idempotency_key: &str,
    ) -> Result<Option<CommerceRefundRecord>> {
        let row = sqlx::query(
            "SELECT refund_id, order_id, payment_attempt_id, payment_method_id, provider,
                    provider_refund_id, idempotency_key, reason, status, amount_minor,
                    currency_code, request_payload_json, response_payload_json, created_at_ms,
                    updated_at_ms, completed_at_ms
             FROM ai_commerce_refunds
             WHERE idempotency_key = $1",
        )
        .bind(idempotency_key)
        .fetch_optional(&self.pool)
        .await?;
        row.map(Self::map_postgres_commerce_refund_row).transpose()
    }

    pub async fn list_commerce_refunds_for_order(
        &self,
        order_id: &str,
    ) -> Result<Vec<CommerceRefundRecord>> {
        let rows = sqlx::query(
            "SELECT refund_id, order_id, payment_attempt_id, payment_method_id, provider,
                    provider_refund_id, idempotency_key, reason, status, amount_minor,
                    currency_code, request_payload_json, response_payload_json, created_at_ms,
                    updated_at_ms, completed_at_ms
             FROM ai_commerce_refunds
             WHERE order_id = $1
             ORDER BY updated_at_ms DESC, created_at_ms DESC, refund_id DESC",
        )
        .bind(order_id)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter()
            .map(Self::map_postgres_commerce_refund_row)
            .collect()
    }

    pub async fn insert_commerce_reconciliation_run(
        &self,
        record: &CommerceReconciliationRunRecord,
    ) -> Result<CommerceReconciliationRunRecord> {
        sqlx::query(
            "INSERT INTO ai_commerce_reconciliation_runs (
                reconciliation_run_id,
                provider,
                payment_method_id,
                scope_started_at_ms,
                scope_ended_at_ms,
                status,
                summary_json,
                created_at_ms,
                updated_at_ms,
                completed_at_ms
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT(reconciliation_run_id) DO UPDATE SET
                provider = excluded.provider,
                payment_method_id = excluded.payment_method_id,
                scope_started_at_ms = excluded.scope_started_at_ms,
                scope_ended_at_ms = excluded.scope_ended_at_ms,
                status = excluded.status,
                summary_json = excluded.summary_json,
                created_at_ms = excluded.created_at_ms,
                updated_at_ms = excluded.updated_at_ms,
                completed_at_ms = excluded.completed_at_ms",
        )
        .bind(&record.reconciliation_run_id)
        .bind(&record.provider)
        .bind(&record.payment_method_id)
        .bind(i64::try_from(record.scope_started_at_ms)?)
        .bind(i64::try_from(record.scope_ended_at_ms)?)
        .bind(&record.status)
        .bind(&record.summary_json)
        .bind(i64::try_from(record.created_at_ms)?)
        .bind(i64::try_from(record.updated_at_ms)?)
        .bind(record.completed_at_ms.map(i64::try_from).transpose()?)
        .execute(&self.pool)
        .await?;
        Ok(record.clone())
    }

    pub async fn list_commerce_reconciliation_runs(
        &self,
    ) -> Result<Vec<CommerceReconciliationRunRecord>> {
        let rows = sqlx::query(
            "SELECT reconciliation_run_id, provider, payment_method_id, scope_started_at_ms,
                    scope_ended_at_ms, status, summary_json, created_at_ms, updated_at_ms,
                    completed_at_ms
             FROM ai_commerce_reconciliation_runs
             ORDER BY created_at_ms DESC, reconciliation_run_id DESC",
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter()
            .map(Self::map_postgres_commerce_reconciliation_run_row)
            .collect()
    }

    pub async fn insert_commerce_reconciliation_item(
        &self,
        record: &CommerceReconciliationItemRecord,
    ) -> Result<CommerceReconciliationItemRecord> {
        sqlx::query(
            "INSERT INTO ai_commerce_reconciliation_items (
                reconciliation_item_id,
                reconciliation_run_id,
                order_id,
                payment_attempt_id,
                refund_id,
                external_reference,
                discrepancy_type,
                status,
                expected_amount_minor,
                provider_amount_minor,
                detail_json,
                created_at_ms,
                updated_at_ms
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            ON CONFLICT(reconciliation_item_id) DO UPDATE SET
                reconciliation_run_id = excluded.reconciliation_run_id,
                order_id = excluded.order_id,
                payment_attempt_id = excluded.payment_attempt_id,
                refund_id = excluded.refund_id,
                external_reference = excluded.external_reference,
                discrepancy_type = excluded.discrepancy_type,
                status = excluded.status,
                expected_amount_minor = excluded.expected_amount_minor,
                provider_amount_minor = excluded.provider_amount_minor,
                detail_json = excluded.detail_json,
                created_at_ms = excluded.created_at_ms,
                updated_at_ms = excluded.updated_at_ms",
        )
        .bind(&record.reconciliation_item_id)
        .bind(&record.reconciliation_run_id)
        .bind(&record.order_id)
        .bind(&record.payment_attempt_id)
        .bind(&record.refund_id)
        .bind(&record.external_reference)
        .bind(&record.discrepancy_type)
        .bind(&record.status)
        .bind(record.expected_amount_minor)
        .bind(record.provider_amount_minor)
        .bind(&record.detail_json)
        .bind(i64::try_from(record.created_at_ms)?)
        .bind(i64::try_from(record.updated_at_ms)?)
        .execute(&self.pool)
        .await?;
        Ok(record.clone())
    }

    pub async fn list_commerce_reconciliation_items(
        &self,
        reconciliation_run_id: &str,
    ) -> Result<Vec<CommerceReconciliationItemRecord>> {
        let rows = sqlx::query(
            "SELECT reconciliation_item_id, reconciliation_run_id, order_id, payment_attempt_id,
                    refund_id, external_reference, discrepancy_type, status,
                    expected_amount_minor, provider_amount_minor, detail_json, created_at_ms,
                    updated_at_ms
             FROM ai_commerce_reconciliation_items
             WHERE reconciliation_run_id = $1
             ORDER BY created_at_ms DESC, reconciliation_item_id DESC",
        )
        .bind(reconciliation_run_id)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter()
            .map(Self::map_postgres_commerce_reconciliation_item_row)
            .collect()
    }
}
