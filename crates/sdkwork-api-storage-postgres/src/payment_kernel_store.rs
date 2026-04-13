use super::*;

fn decode_payment_order_row(row: &PgRow) -> Result<PaymentOrderRecord> {
    let mut record = PaymentOrderRecord::new(
        row.try_get::<String, _>("payment_order_id")?,
        row.try_get::<String, _>("commerce_order_id")?,
        u64::try_from(row.try_get::<i64, _>("tenant_id")?)?,
        u64::try_from(row.try_get::<i64, _>("organization_id")?)?,
        u64::try_from(row.try_get::<i64, _>("user_id")?)?,
        row.try_get::<String, _>("project_id")?,
        row.try_get::<String, _>("order_kind")?,
        row.try_get::<String, _>("subject_type")?,
        row.try_get::<String, _>("subject_id")?,
        row.try_get::<String, _>("currency_code")?,
        u64::try_from(row.try_get::<i64, _>("amount_minor")?)?,
    )
    .with_discount_minor(u64::try_from(row.try_get::<i64, _>("discount_minor")?)?)
    .with_subsidy_minor(u64::try_from(row.try_get::<i64, _>("subsidy_minor")?)?)
    .with_payable_minor(u64::try_from(row.try_get::<i64, _>("payable_minor")?)?)
    .with_captured_amount_minor(u64::try_from(
        row.try_get::<i64, _>("captured_amount_minor")?,
    )?)
    .with_provider_code(
        PaymentProviderCode::from_str(&row.try_get::<String, _>("provider_code")?)
            .map_err(anyhow::Error::msg)?,
    )
    .with_payment_status(
        PaymentOrderStatus::from_str(&row.try_get::<String, _>("payment_status")?)
            .map_err(anyhow::Error::msg)?,
    )
    .with_fulfillment_status(row.try_get::<String, _>("fulfillment_status")?)
    .with_refund_status(
        PaymentRefundStatus::from_str(&row.try_get::<String, _>("refund_status")?)
            .map_err(anyhow::Error::msg)?,
    )
    .with_quote_snapshot_json(row.try_get("quote_snapshot_json")?)
    .with_metadata_json(row.try_get("metadata_json")?)
    .with_version(u64::try_from(row.try_get::<i64, _>("version")?)?)
    .with_created_at_ms(u64::try_from(row.try_get::<i64, _>("created_at_ms")?)?)
    .with_updated_at_ms(u64::try_from(row.try_get::<i64, _>("updated_at_ms")?)?);

    if let Some(method_code) = row.try_get::<Option<String>, _>("method_code")? {
        record = record.with_method_code(method_code);
    }

    Ok(record)
}

fn decode_payment_gateway_account_row(row: &PgRow) -> Result<PaymentGatewayAccountRecord> {
    Ok(PaymentGatewayAccountRecord::new(
        row.try_get::<String, _>("gateway_account_id")?,
        u64::try_from(row.try_get::<i64, _>("tenant_id")?)?,
        u64::try_from(row.try_get::<i64, _>("organization_id")?)?,
        PaymentProviderCode::from_str(&row.try_get::<String, _>("provider_code")?)
            .map_err(anyhow::Error::msg)?,
    )
    .with_environment(row.try_get::<String, _>("environment")?)
    .with_merchant_id(row.try_get::<String, _>("merchant_id")?)
    .with_app_id(row.try_get::<String, _>("app_id")?)
    .with_status(row.try_get::<String, _>("status")?)
    .with_priority(i32::try_from(row.try_get::<i64, _>("priority")?)?)
    .with_created_at_ms(u64::try_from(row.try_get::<i64, _>("created_at_ms")?)?)
    .with_updated_at_ms(u64::try_from(row.try_get::<i64, _>("updated_at_ms")?)?))
}

fn decode_payment_channel_policy_row(row: &PgRow) -> Result<PaymentChannelPolicyRecord> {
    Ok(PaymentChannelPolicyRecord::new(
        row.try_get::<String, _>("channel_policy_id")?,
        u64::try_from(row.try_get::<i64, _>("tenant_id")?)?,
        u64::try_from(row.try_get::<i64, _>("organization_id")?)?,
        PaymentProviderCode::from_str(&row.try_get::<String, _>("provider_code")?)
            .map_err(anyhow::Error::msg)?,
        row.try_get::<String, _>("method_code")?,
    )
    .with_scene_code(row.try_get::<String, _>("scene_code")?)
    .with_country_code(row.try_get::<String, _>("country_code")?)
    .with_currency_code(row.try_get::<String, _>("currency_code")?)
    .with_client_kind(row.try_get::<String, _>("client_kind")?)
    .with_priority(i32::try_from(row.try_get::<i64, _>("priority")?)?)
    .with_status(row.try_get::<String, _>("status")?)
    .with_created_at_ms(u64::try_from(row.try_get::<i64, _>("created_at_ms")?)?)
    .with_updated_at_ms(u64::try_from(row.try_get::<i64, _>("updated_at_ms")?)?))
}

fn decode_payment_attempt_row(row: &PgRow) -> Result<PaymentAttemptRecord> {
    Ok(PaymentAttemptRecord::new(
        row.try_get::<String, _>("payment_attempt_id")?,
        u64::try_from(row.try_get::<i64, _>("tenant_id")?)?,
        u64::try_from(row.try_get::<i64, _>("organization_id")?)?,
        row.try_get::<String, _>("payment_order_id")?,
        u32::try_from(row.try_get::<i64, _>("attempt_no")?)?,
        row.try_get::<String, _>("gateway_account_id")?,
        PaymentProviderCode::from_str(&row.try_get::<String, _>("provider_code")?)
            .map_err(anyhow::Error::msg)?,
        row.try_get::<String, _>("method_code")?,
        row.try_get::<String, _>("client_kind")?,
        row.try_get::<String, _>("idempotency_key")?,
    )
    .with_provider_request_id(row.try_get("provider_request_id")?)
    .with_provider_payment_reference(row.try_get("provider_payment_reference")?)
    .with_attempt_status(
        PaymentAttemptStatus::from_str(&row.try_get::<String, _>("attempt_status")?)
            .map_err(anyhow::Error::msg)?,
    )
    .with_request_payload_hash(row.try_get::<String, _>("request_payload_hash")?)
    .with_expires_at_ms(
        row.try_get::<Option<i64>, _>("expires_at_ms")?
            .map(u64::try_from)
            .transpose()?,
    )
    .with_created_at_ms(u64::try_from(row.try_get::<i64, _>("created_at_ms")?)?)
    .with_updated_at_ms(u64::try_from(row.try_get::<i64, _>("updated_at_ms")?)?))
}

fn decode_payment_session_row(row: &PgRow) -> Result<PaymentSessionRecord> {
    Ok(PaymentSessionRecord::new(
        row.try_get::<String, _>("payment_session_id")?,
        u64::try_from(row.try_get::<i64, _>("tenant_id")?)?,
        u64::try_from(row.try_get::<i64, _>("organization_id")?)?,
        row.try_get::<String, _>("payment_attempt_id")?,
        PaymentSessionKind::from_str(&row.try_get::<String, _>("session_kind")?)
            .map_err(anyhow::Error::msg)?,
        PaymentSessionStatus::from_str(&row.try_get::<String, _>("session_status")?)
            .map_err(anyhow::Error::msg)?,
    )
    .with_display_reference(row.try_get("display_reference")?)
    .with_qr_payload(row.try_get("qr_payload")?)
    .with_redirect_url(row.try_get("redirect_url")?)
    .with_expires_at_ms(u64::try_from(row.try_get::<i64, _>("expires_at_ms")?)?)
    .with_created_at_ms(u64::try_from(row.try_get::<i64, _>("created_at_ms")?)?)
    .with_updated_at_ms(u64::try_from(row.try_get::<i64, _>("updated_at_ms")?)?))
}

fn decode_payment_callback_event_row(row: &PgRow) -> Result<PaymentCallbackEventRecord> {
    Ok(PaymentCallbackEventRecord::new(
        row.try_get::<String, _>("callback_event_id")?,
        u64::try_from(row.try_get::<i64, _>("tenant_id")?)?,
        u64::try_from(row.try_get::<i64, _>("organization_id")?)?,
        PaymentProviderCode::from_str(&row.try_get::<String, _>("provider_code")?)
            .map_err(anyhow::Error::msg)?,
        row.try_get::<String, _>("gateway_account_id")?,
        row.try_get::<String, _>("event_type")?,
        row.try_get::<String, _>("event_identity")?,
        row.try_get::<String, _>("dedupe_key")?,
        u64::try_from(row.try_get::<i64, _>("received_at_ms")?)?,
    )
    .with_payment_order_id(row.try_get("payment_order_id")?)
    .with_payment_attempt_id(row.try_get("payment_attempt_id")?)
    .with_provider_transaction_id(row.try_get("provider_transaction_id")?)
    .with_signature_status(row.try_get::<String, _>("signature_status")?)
    .with_processing_status(
        PaymentCallbackProcessingStatus::from_str(&row.try_get::<String, _>("processing_status")?)
            .map_err(anyhow::Error::msg)?,
    )
    .with_payload_json(row.try_get("payload_json")?)
    .with_processed_at_ms(
        row.try_get::<Option<i64>, _>("processed_at_ms")?
            .map(u64::try_from)
            .transpose()?,
    ))
}

fn decode_refund_order_row(row: &PgRow) -> Result<RefundOrderRecord> {
    Ok(RefundOrderRecord::new(
        row.try_get::<String, _>("refund_order_id")?,
        u64::try_from(row.try_get::<i64, _>("tenant_id")?)?,
        u64::try_from(row.try_get::<i64, _>("organization_id")?)?,
        row.try_get::<String, _>("payment_order_id")?,
        row.try_get::<String, _>("commerce_order_id")?,
        row.try_get::<String, _>("refund_reason_code")?,
        row.try_get::<String, _>("requested_by_type")?,
        row.try_get::<String, _>("requested_by_id")?,
        row.try_get::<String, _>("currency_code")?,
        u64::try_from(row.try_get::<i64, _>("requested_amount_minor")?)?,
    )
    .with_approved_amount_minor(
        row.try_get::<Option<i64>, _>("approved_amount_minor")?
            .map(u64::try_from)
            .transpose()?,
    )
    .with_refunded_amount_minor(u64::try_from(
        row.try_get::<i64, _>("refunded_amount_minor")?,
    )?)
    .with_refund_status(
        RefundOrderStatus::from_str(&row.try_get::<String, _>("refund_status")?)
            .map_err(anyhow::Error::msg)?,
    )
    .with_created_at_ms(u64::try_from(row.try_get::<i64, _>("created_at_ms")?)?)
    .with_updated_at_ms(u64::try_from(row.try_get::<i64, _>("updated_at_ms")?)?))
}

fn decode_payment_transaction_row(row: &PgRow) -> Result<PaymentTransactionRecord> {
    Ok(PaymentTransactionRecord::new(
        row.try_get::<String, _>("payment_transaction_id")?,
        u64::try_from(row.try_get::<i64, _>("tenant_id")?)?,
        u64::try_from(row.try_get::<i64, _>("organization_id")?)?,
        row.try_get::<String, _>("payment_order_id")?,
        PaymentTransactionKind::from_str(&row.try_get::<String, _>("transaction_kind")?)
            .map_err(anyhow::Error::msg)?,
        PaymentProviderCode::from_str(&row.try_get::<String, _>("provider_code")?)
            .map_err(anyhow::Error::msg)?,
        row.try_get::<String, _>("provider_transaction_id")?,
        row.try_get::<String, _>("currency_code")?,
        u64::try_from(row.try_get::<i64, _>("amount_minor")?)?,
        u64::try_from(row.try_get::<i64, _>("occurred_at_ms")?)?,
    )
    .with_payment_attempt_id(row.try_get("payment_attempt_id")?)
    .with_fee_minor(
        row.try_get::<Option<i64>, _>("fee_minor")?
            .map(u64::try_from)
            .transpose()?,
    )
    .with_net_amount_minor(
        row.try_get::<Option<i64>, _>("net_amount_minor")?
            .map(u64::try_from)
            .transpose()?,
    )
    .with_provider_status(row.try_get::<String, _>("provider_status")?)
    .with_raw_event_id(row.try_get("raw_event_id")?)
    .with_created_at_ms(u64::try_from(row.try_get::<i64, _>("created_at_ms")?)?))
}

fn decode_finance_journal_entry_row(row: &PgRow) -> Result<FinanceJournalEntryRecord> {
    Ok(FinanceJournalEntryRecord::new(
        row.try_get::<String, _>("finance_journal_entry_id")?,
        u64::try_from(row.try_get::<i64, _>("tenant_id")?)?,
        u64::try_from(row.try_get::<i64, _>("organization_id")?)?,
        row.try_get::<String, _>("source_kind")?,
        row.try_get::<String, _>("source_id")?,
        FinanceEntryCode::from_str(&row.try_get::<String, _>("entry_code")?)
            .map_err(anyhow::Error::msg)?,
        row.try_get::<String, _>("currency_code")?,
        u64::try_from(row.try_get::<i64, _>("occurred_at_ms")?)?,
    )
    .with_entry_status(row.try_get::<String, _>("entry_status")?)
    .with_created_at_ms(u64::try_from(row.try_get::<i64, _>("created_at_ms")?)?))
}

fn decode_finance_journal_line_row(row: &PgRow) -> Result<FinanceJournalLineRecord> {
    Ok(FinanceJournalLineRecord::new(
        row.try_get::<String, _>("finance_journal_line_id")?,
        u64::try_from(row.try_get::<i64, _>("tenant_id")?)?,
        u64::try_from(row.try_get::<i64, _>("organization_id")?)?,
        row.try_get::<String, _>("finance_journal_entry_id")?,
        u32::try_from(row.try_get::<i64, _>("line_no")?)?,
        row.try_get::<String, _>("account_code")?,
        FinanceDirection::from_str(&row.try_get::<String, _>("direction")?)
            .map_err(anyhow::Error::msg)?,
        u64::try_from(row.try_get::<i64, _>("amount_minor")?)?,
    )
    .with_party_type(row.try_get("party_type")?)
    .with_party_id(row.try_get("party_id")?))
}

fn decode_reconciliation_match_summary_row(
    row: &PgRow,
) -> Result<ReconciliationMatchSummaryRecord> {
    Ok(ReconciliationMatchSummaryRecord::new(
        row.try_get::<String, _>("reconciliation_line_id")?,
        u64::try_from(row.try_get::<i64, _>("tenant_id")?)?,
        u64::try_from(row.try_get::<i64, _>("organization_id")?)?,
        row.try_get::<String, _>("reconciliation_batch_id")?,
        row.try_get::<String, _>("provider_transaction_id")?,
        ReconciliationMatchStatus::from_str(&row.try_get::<String, _>("match_status")?)
            .map_err(anyhow::Error::msg)?,
        u64::try_from(row.try_get::<i64, _>("provider_amount_minor")?)?,
    )
    .with_local_amount_minor(
        row.try_get::<Option<i64>, _>("local_amount_minor")?
            .map(u64::try_from)
            .transpose()?,
    )
    .with_payment_order_id(row.try_get("payment_order_id")?)
    .with_refund_order_id(row.try_get("refund_order_id")?)
    .with_reason_code(row.try_get("reason_code")?)
    .with_created_at_ms(u64::try_from(row.try_get::<i64, _>("created_at_ms")?)?)
    .with_updated_at_ms(u64::try_from(row.try_get::<i64, _>("updated_at_ms")?)?))
}

impl PostgresAdminStore {
    async fn apply_refund_order_quota_reversal(
        &self,
        refund_order_id: &str,
        project_id: &str,
        target_kind: &str,
        target_units: u64,
    ) -> Result<bool> {
        if !matches!(target_kind, "recharge_pack" | "custom_recharge") {
            return Err(anyhow::anyhow!(
                "quota reversal does not support target kind {target_kind}"
            ));
        }
        if target_units == 0 {
            return Ok(false);
        }

        let mut tx = self.pool.begin().await?;
        if !try_insert_refund_order_quota_processing_step(&mut tx, refund_order_id).await? {
            tx.rollback().await?;
            return Ok(false);
        }

        let Some((policy_id, max_units)) = sqlx::query_as::<_, (String, i64)>(
            "SELECT policy_id, max_units
             FROM ai_billing_quota_policies
             WHERE project_id = $1
               AND enabled = TRUE
             ORDER BY max_units ASC, policy_id ASC
             LIMIT 1",
        )
        .bind(project_id)
        .fetch_optional(&mut *tx)
        .await?
        else {
            tx.rollback().await?;
            return Err(anyhow::anyhow!(
                "project {project_id} cannot be refunded because no active quota policy exists"
            ));
        };
        let max_units = u64::try_from(max_units)?;
        if max_units < target_units {
            tx.rollback().await?;
            return Err(anyhow::anyhow!(
                "project {project_id} cannot be refunded because quota baseline drifted"
            ));
        }

        let used_units = sqlx::query_scalar::<_, i64>(
            "SELECT COALESCE(SUM(units), 0)
             FROM ai_billing_ledger_entries
             WHERE project_id = $1",
        )
        .bind(project_id)
        .fetch_one(&mut *tx)
        .await?;
        let used_units = u64::try_from(used_units)?;
        let remaining_units = max_units.saturating_sub(used_units);
        if remaining_units < target_units {
            tx.rollback().await?;
            return Err(anyhow::anyhow!(
                "project {project_id} cannot be refunded because recharge headroom has already been consumed"
            ));
        }

        sqlx::query(
            "UPDATE ai_billing_quota_policies
             SET max_units = $1
             WHERE policy_id = $2",
        )
        .bind(i64::try_from(max_units.saturating_sub(target_units))?)
        .bind(&policy_id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(true)
    }
}

async fn try_insert_refund_order_quota_processing_step(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    refund_order_id: &str,
) -> Result<bool> {
    let result = sqlx::query(
        "INSERT INTO ai_refund_order_processing_steps (refund_order_id, step_key, applied_at_ms)
         VALUES ($1, $2, (EXTRACT(EPOCH FROM NOW()) * 1000)::BIGINT)
         ON CONFLICT(refund_order_id, step_key) DO NOTHING",
    )
    .bind(refund_order_id)
    .bind("quota")
    .execute(&mut **tx)
    .await?;
    Ok(result.rows_affected() == 1)
}

#[async_trait]
impl PaymentKernelStore for PostgresAdminStore {
    async fn insert_payment_order_record(
        &self,
        record: &PaymentOrderRecord,
    ) -> Result<PaymentOrderRecord> {
        sqlx::query(
            "INSERT INTO ai_payment_order (
                payment_order_id, tenant_id, organization_id, user_id, commerce_order_id,
                project_id, order_kind, subject_type, subject_id, currency_code,
                amount_minor, discount_minor, subsidy_minor, payable_minor,
                captured_amount_minor, provider_code, method_code, payment_status,
                fulfillment_status, refund_status, quote_snapshot_json, metadata_json,
                version, created_at_ms, updated_at_ms
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14,
                $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25
            )
            ON CONFLICT (payment_order_id) DO UPDATE SET
                tenant_id = EXCLUDED.tenant_id,
                organization_id = EXCLUDED.organization_id,
                user_id = EXCLUDED.user_id,
                commerce_order_id = EXCLUDED.commerce_order_id,
                project_id = EXCLUDED.project_id,
                order_kind = EXCLUDED.order_kind,
                subject_type = EXCLUDED.subject_type,
                subject_id = EXCLUDED.subject_id,
                currency_code = EXCLUDED.currency_code,
                amount_minor = EXCLUDED.amount_minor,
                discount_minor = EXCLUDED.discount_minor,
                subsidy_minor = EXCLUDED.subsidy_minor,
                payable_minor = EXCLUDED.payable_minor,
                captured_amount_minor = EXCLUDED.captured_amount_minor,
                provider_code = EXCLUDED.provider_code,
                method_code = EXCLUDED.method_code,
                payment_status = EXCLUDED.payment_status,
                fulfillment_status = EXCLUDED.fulfillment_status,
                refund_status = EXCLUDED.refund_status,
                quote_snapshot_json = EXCLUDED.quote_snapshot_json,
                metadata_json = EXCLUDED.metadata_json,
                version = EXCLUDED.version,
                created_at_ms = EXCLUDED.created_at_ms,
                updated_at_ms = EXCLUDED.updated_at_ms",
        )
        .bind(&record.payment_order_id)
        .bind(i64::try_from(record.tenant_id)?)
        .bind(i64::try_from(record.organization_id)?)
        .bind(i64::try_from(record.user_id)?)
        .bind(&record.commerce_order_id)
        .bind(&record.project_id)
        .bind(&record.order_kind)
        .bind(&record.subject_type)
        .bind(&record.subject_id)
        .bind(&record.currency_code)
        .bind(i64::try_from(record.amount_minor)?)
        .bind(i64::try_from(record.discount_minor)?)
        .bind(i64::try_from(record.subsidy_minor)?)
        .bind(i64::try_from(record.payable_minor)?)
        .bind(i64::try_from(record.captured_amount_minor)?)
        .bind(record.provider_code.as_str())
        .bind(record.method_code.as_deref())
        .bind(record.payment_status.as_str())
        .bind(&record.fulfillment_status)
        .bind(record.refund_status.as_str())
        .bind(record.quote_snapshot_json.as_deref())
        .bind(record.metadata_json.as_deref())
        .bind(i64::try_from(record.version)?)
        .bind(i64::try_from(record.created_at_ms)?)
        .bind(i64::try_from(record.updated_at_ms)?)
        .execute(&self.pool)
        .await?;
        Ok(record.clone())
    }

    async fn find_payment_order_record(
        &self,
        payment_order_id: &str,
    ) -> Result<Option<PaymentOrderRecord>> {
        let row = sqlx::query(
            "SELECT payment_order_id, tenant_id, organization_id, user_id, commerce_order_id,
                    project_id, order_kind, subject_type, subject_id, currency_code,
                    amount_minor, discount_minor, subsidy_minor, payable_minor,
                    captured_amount_minor, provider_code, method_code, payment_status,
                    fulfillment_status, refund_status, quote_snapshot_json, metadata_json,
                    version, created_at_ms, updated_at_ms
             FROM ai_payment_order
             WHERE payment_order_id = $1",
        )
        .bind(payment_order_id)
        .fetch_optional(&self.pool)
        .await?;
        row.map(|row| decode_payment_order_row(&row)).transpose()
    }

    async fn list_payment_order_records(&self) -> Result<Vec<PaymentOrderRecord>> {
        let rows = sqlx::query(
            "SELECT payment_order_id, tenant_id, organization_id, user_id, commerce_order_id,
                    project_id, order_kind, subject_type, subject_id, currency_code,
                    amount_minor, discount_minor, subsidy_minor, payable_minor,
                    captured_amount_minor, provider_code, method_code, payment_status,
                    fulfillment_status, refund_status, quote_snapshot_json, metadata_json,
                    version, created_at_ms, updated_at_ms
             FROM ai_payment_order
             ORDER BY created_at_ms DESC, payment_order_id",
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter()
            .map(|row| decode_payment_order_row(&row))
            .collect()
    }

    async fn insert_payment_gateway_account_record(
        &self,
        record: &PaymentGatewayAccountRecord,
    ) -> Result<PaymentGatewayAccountRecord> {
        sqlx::query(
            "INSERT INTO ai_payment_gateway_account (
                gateway_account_id, tenant_id, organization_id, provider_code, environment,
                merchant_id, app_id, status, priority, created_at_ms, updated_at_ms
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            ON CONFLICT (gateway_account_id) DO UPDATE SET
                tenant_id = EXCLUDED.tenant_id,
                organization_id = EXCLUDED.organization_id,
                provider_code = EXCLUDED.provider_code,
                environment = EXCLUDED.environment,
                merchant_id = EXCLUDED.merchant_id,
                app_id = EXCLUDED.app_id,
                status = EXCLUDED.status,
                priority = EXCLUDED.priority,
                created_at_ms = EXCLUDED.created_at_ms,
                updated_at_ms = EXCLUDED.updated_at_ms",
        )
        .bind(&record.gateway_account_id)
        .bind(i64::try_from(record.tenant_id)?)
        .bind(i64::try_from(record.organization_id)?)
        .bind(record.provider_code.as_str())
        .bind(&record.environment)
        .bind(&record.merchant_id)
        .bind(&record.app_id)
        .bind(&record.status)
        .bind(i64::from(record.priority))
        .bind(i64::try_from(record.created_at_ms)?)
        .bind(i64::try_from(record.updated_at_ms)?)
        .execute(&self.pool)
        .await?;
        Ok(record.clone())
    }

    async fn list_payment_gateway_account_records(
        &self,
    ) -> Result<Vec<PaymentGatewayAccountRecord>> {
        let rows = sqlx::query(
            "SELECT gateway_account_id, tenant_id, organization_id, provider_code, environment,
                    merchant_id, app_id, status, priority, created_at_ms, updated_at_ms
             FROM ai_payment_gateway_account
             ORDER BY priority DESC, gateway_account_id",
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter()
            .map(|row| decode_payment_gateway_account_row(&row))
            .collect()
    }

    async fn insert_payment_channel_policy_record(
        &self,
        record: &PaymentChannelPolicyRecord,
    ) -> Result<PaymentChannelPolicyRecord> {
        sqlx::query(
            "INSERT INTO ai_payment_channel_policy (
                channel_policy_id, tenant_id, organization_id, scene_code, country_code,
                currency_code, client_kind, provider_code, method_code, priority, status,
                created_at_ms, updated_at_ms
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13
            )
            ON CONFLICT (channel_policy_id) DO UPDATE SET
                tenant_id = EXCLUDED.tenant_id,
                organization_id = EXCLUDED.organization_id,
                scene_code = EXCLUDED.scene_code,
                country_code = EXCLUDED.country_code,
                currency_code = EXCLUDED.currency_code,
                client_kind = EXCLUDED.client_kind,
                provider_code = EXCLUDED.provider_code,
                method_code = EXCLUDED.method_code,
                priority = EXCLUDED.priority,
                status = EXCLUDED.status,
                created_at_ms = EXCLUDED.created_at_ms,
                updated_at_ms = EXCLUDED.updated_at_ms",
        )
        .bind(&record.channel_policy_id)
        .bind(i64::try_from(record.tenant_id)?)
        .bind(i64::try_from(record.organization_id)?)
        .bind(&record.scene_code)
        .bind(&record.country_code)
        .bind(&record.currency_code)
        .bind(&record.client_kind)
        .bind(record.provider_code.as_str())
        .bind(&record.method_code)
        .bind(i64::from(record.priority))
        .bind(&record.status)
        .bind(i64::try_from(record.created_at_ms)?)
        .bind(i64::try_from(record.updated_at_ms)?)
        .execute(&self.pool)
        .await?;
        Ok(record.clone())
    }

    async fn list_payment_channel_policy_records(&self) -> Result<Vec<PaymentChannelPolicyRecord>> {
        let rows = sqlx::query(
            "SELECT channel_policy_id, tenant_id, organization_id, scene_code, country_code,
                    currency_code, client_kind, provider_code, method_code, priority, status,
                    created_at_ms, updated_at_ms
             FROM ai_payment_channel_policy
             ORDER BY priority DESC, channel_policy_id",
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter()
            .map(|row| decode_payment_channel_policy_row(&row))
            .collect()
    }

    async fn insert_payment_attempt_record(
        &self,
        record: &PaymentAttemptRecord,
    ) -> Result<PaymentAttemptRecord> {
        sqlx::query(
            "INSERT INTO ai_payment_attempt (
                payment_attempt_id, tenant_id, organization_id, payment_order_id, attempt_no,
                gateway_account_id, provider_code, method_code, client_kind, idempotency_key,
                provider_request_id, provider_payment_reference, attempt_status,
                request_payload_hash, expires_at_ms, created_at_ms, updated_at_ms
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17
            )
            ON CONFLICT (payment_attempt_id) DO UPDATE SET
                tenant_id = EXCLUDED.tenant_id,
                organization_id = EXCLUDED.organization_id,
                payment_order_id = EXCLUDED.payment_order_id,
                attempt_no = EXCLUDED.attempt_no,
                gateway_account_id = EXCLUDED.gateway_account_id,
                provider_code = EXCLUDED.provider_code,
                method_code = EXCLUDED.method_code,
                client_kind = EXCLUDED.client_kind,
                idempotency_key = EXCLUDED.idempotency_key,
                provider_request_id = EXCLUDED.provider_request_id,
                provider_payment_reference = EXCLUDED.provider_payment_reference,
                attempt_status = EXCLUDED.attempt_status,
                request_payload_hash = EXCLUDED.request_payload_hash,
                expires_at_ms = EXCLUDED.expires_at_ms,
                created_at_ms = EXCLUDED.created_at_ms,
                updated_at_ms = EXCLUDED.updated_at_ms",
        )
        .bind(&record.payment_attempt_id)
        .bind(i64::try_from(record.tenant_id)?)
        .bind(i64::try_from(record.organization_id)?)
        .bind(&record.payment_order_id)
        .bind(i64::from(record.attempt_no))
        .bind(&record.gateway_account_id)
        .bind(record.provider_code.as_str())
        .bind(&record.method_code)
        .bind(&record.client_kind)
        .bind(&record.idempotency_key)
        .bind(record.provider_request_id.as_deref())
        .bind(record.provider_payment_reference.as_deref())
        .bind(record.attempt_status.as_str())
        .bind(&record.request_payload_hash)
        .bind(record.expires_at_ms.map(i64::try_from).transpose()?)
        .bind(i64::try_from(record.created_at_ms)?)
        .bind(i64::try_from(record.updated_at_ms)?)
        .execute(&self.pool)
        .await?;
        Ok(record.clone())
    }

    async fn list_payment_attempt_records_for_order(
        &self,
        payment_order_id: &str,
    ) -> Result<Vec<PaymentAttemptRecord>> {
        let rows = sqlx::query(
            "SELECT payment_attempt_id, tenant_id, organization_id, payment_order_id, attempt_no,
                    gateway_account_id, provider_code, method_code, client_kind, idempotency_key,
                    provider_request_id, provider_payment_reference, attempt_status,
                    request_payload_hash, expires_at_ms, created_at_ms, updated_at_ms
             FROM ai_payment_attempt
             WHERE payment_order_id = $1
             ORDER BY attempt_no DESC, payment_attempt_id",
        )
        .bind(payment_order_id)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter()
            .map(|row| decode_payment_attempt_row(&row))
            .collect()
    }

    async fn insert_payment_session_record(
        &self,
        record: &PaymentSessionRecord,
    ) -> Result<PaymentSessionRecord> {
        sqlx::query(
            "INSERT INTO ai_payment_session (
                payment_session_id, tenant_id, organization_id, payment_attempt_id,
                session_kind, session_status, display_reference, qr_payload, redirect_url,
                expires_at_ms, created_at_ms, updated_at_ms
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            ON CONFLICT (payment_session_id) DO UPDATE SET
                tenant_id = EXCLUDED.tenant_id,
                organization_id = EXCLUDED.organization_id,
                payment_attempt_id = EXCLUDED.payment_attempt_id,
                session_kind = EXCLUDED.session_kind,
                session_status = EXCLUDED.session_status,
                display_reference = EXCLUDED.display_reference,
                qr_payload = EXCLUDED.qr_payload,
                redirect_url = EXCLUDED.redirect_url,
                expires_at_ms = EXCLUDED.expires_at_ms,
                created_at_ms = EXCLUDED.created_at_ms,
                updated_at_ms = EXCLUDED.updated_at_ms",
        )
        .bind(&record.payment_session_id)
        .bind(i64::try_from(record.tenant_id)?)
        .bind(i64::try_from(record.organization_id)?)
        .bind(&record.payment_attempt_id)
        .bind(record.session_kind.as_str())
        .bind(record.session_status.as_str())
        .bind(record.display_reference.as_deref())
        .bind(record.qr_payload.as_deref())
        .bind(record.redirect_url.as_deref())
        .bind(i64::try_from(record.expires_at_ms)?)
        .bind(i64::try_from(record.created_at_ms)?)
        .bind(i64::try_from(record.updated_at_ms)?)
        .execute(&self.pool)
        .await?;
        Ok(record.clone())
    }

    async fn list_payment_session_records_for_attempt(
        &self,
        payment_attempt_id: &str,
    ) -> Result<Vec<PaymentSessionRecord>> {
        let rows = sqlx::query(
            "SELECT payment_session_id, tenant_id, organization_id, payment_attempt_id,
                    session_kind, session_status, display_reference, qr_payload, redirect_url,
                    expires_at_ms, created_at_ms, updated_at_ms
             FROM ai_payment_session
             WHERE payment_attempt_id = $1
             ORDER BY created_at_ms DESC, payment_session_id",
        )
        .bind(payment_attempt_id)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter()
            .map(|row| decode_payment_session_row(&row))
            .collect()
    }

    async fn find_payment_callback_event_record_by_dedupe_key(
        &self,
        provider_code: PaymentProviderCode,
        gateway_account_id: &str,
        dedupe_key: &str,
    ) -> Result<Option<PaymentCallbackEventRecord>> {
        let row = sqlx::query(
            "SELECT callback_event_id, tenant_id, organization_id, provider_code,
                    gateway_account_id, event_type, event_identity, dedupe_key,
                    payment_order_id, payment_attempt_id, provider_transaction_id,
                    signature_status, processing_status, payload_json, received_at_ms,
                    processed_at_ms
             FROM ai_payment_callback_event
             WHERE provider_code = $1
               AND gateway_account_id = $2
               AND dedupe_key = $3
             LIMIT 1",
        )
        .bind(provider_code.as_str())
        .bind(gateway_account_id)
        .bind(dedupe_key)
        .fetch_optional(&self.pool)
        .await?;
        row.map(|row| decode_payment_callback_event_row(&row))
            .transpose()
    }

    async fn insert_payment_callback_event_record(
        &self,
        record: &PaymentCallbackEventRecord,
    ) -> Result<PaymentCallbackEventRecord> {
        sqlx::query(
            "INSERT INTO ai_payment_callback_event (
                callback_event_id, tenant_id, organization_id, provider_code,
                gateway_account_id, event_type, event_identity, dedupe_key, payment_order_id,
                payment_attempt_id, provider_transaction_id, signature_status,
                processing_status, payload_json, received_at_ms, processed_at_ms
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16
            )
            ON CONFLICT (callback_event_id) DO UPDATE SET
                tenant_id = EXCLUDED.tenant_id,
                organization_id = EXCLUDED.organization_id,
                provider_code = EXCLUDED.provider_code,
                gateway_account_id = EXCLUDED.gateway_account_id,
                event_type = EXCLUDED.event_type,
                event_identity = EXCLUDED.event_identity,
                dedupe_key = EXCLUDED.dedupe_key,
                payment_order_id = EXCLUDED.payment_order_id,
                payment_attempt_id = EXCLUDED.payment_attempt_id,
                provider_transaction_id = EXCLUDED.provider_transaction_id,
                signature_status = EXCLUDED.signature_status,
                processing_status = EXCLUDED.processing_status,
                payload_json = EXCLUDED.payload_json,
                received_at_ms = EXCLUDED.received_at_ms,
                processed_at_ms = EXCLUDED.processed_at_ms",
        )
        .bind(&record.callback_event_id)
        .bind(i64::try_from(record.tenant_id)?)
        .bind(i64::try_from(record.organization_id)?)
        .bind(record.provider_code.as_str())
        .bind(&record.gateway_account_id)
        .bind(&record.event_type)
        .bind(&record.event_identity)
        .bind(&record.dedupe_key)
        .bind(record.payment_order_id.as_deref())
        .bind(record.payment_attempt_id.as_deref())
        .bind(record.provider_transaction_id.as_deref())
        .bind(&record.signature_status)
        .bind(record.processing_status.as_str())
        .bind(record.payload_json.as_deref())
        .bind(i64::try_from(record.received_at_ms)?)
        .bind(record.processed_at_ms.map(i64::try_from).transpose()?)
        .execute(&self.pool)
        .await?;
        Ok(record.clone())
    }

    async fn list_payment_callback_event_records(&self) -> Result<Vec<PaymentCallbackEventRecord>> {
        let rows = sqlx::query(
            "SELECT callback_event_id, tenant_id, organization_id, provider_code,
                    gateway_account_id, event_type, event_identity, dedupe_key,
                    payment_order_id, payment_attempt_id, provider_transaction_id,
                    signature_status, processing_status, payload_json, received_at_ms,
                    processed_at_ms
             FROM ai_payment_callback_event
             ORDER BY received_at_ms DESC, callback_event_id",
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter()
            .map(|row| decode_payment_callback_event_row(&row))
            .collect()
    }

    async fn insert_payment_transaction_record(
        &self,
        record: &PaymentTransactionRecord,
    ) -> Result<PaymentTransactionRecord> {
        sqlx::query(
            "INSERT INTO ai_payment_transaction (
                payment_transaction_id, tenant_id, organization_id, payment_order_id,
                payment_attempt_id, transaction_kind, provider_code, provider_transaction_id,
                currency_code, amount_minor, fee_minor, net_amount_minor, provider_status,
                raw_event_id, occurred_at_ms, created_at_ms
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16
            )
            ON CONFLICT (payment_transaction_id) DO UPDATE SET
                tenant_id = EXCLUDED.tenant_id,
                organization_id = EXCLUDED.organization_id,
                payment_order_id = EXCLUDED.payment_order_id,
                payment_attempt_id = EXCLUDED.payment_attempt_id,
                transaction_kind = EXCLUDED.transaction_kind,
                provider_code = EXCLUDED.provider_code,
                provider_transaction_id = EXCLUDED.provider_transaction_id,
                currency_code = EXCLUDED.currency_code,
                amount_minor = EXCLUDED.amount_minor,
                fee_minor = EXCLUDED.fee_minor,
                net_amount_minor = EXCLUDED.net_amount_minor,
                provider_status = EXCLUDED.provider_status,
                raw_event_id = EXCLUDED.raw_event_id,
                occurred_at_ms = EXCLUDED.occurred_at_ms,
                created_at_ms = EXCLUDED.created_at_ms",
        )
        .bind(&record.payment_transaction_id)
        .bind(i64::try_from(record.tenant_id)?)
        .bind(i64::try_from(record.organization_id)?)
        .bind(&record.payment_order_id)
        .bind(record.payment_attempt_id.as_deref())
        .bind(record.transaction_kind.as_str())
        .bind(record.provider_code.as_str())
        .bind(&record.provider_transaction_id)
        .bind(&record.currency_code)
        .bind(i64::try_from(record.amount_minor)?)
        .bind(record.fee_minor.map(i64::try_from).transpose()?)
        .bind(record.net_amount_minor.map(i64::try_from).transpose()?)
        .bind(&record.provider_status)
        .bind(record.raw_event_id.as_deref())
        .bind(i64::try_from(record.occurred_at_ms)?)
        .bind(i64::try_from(record.created_at_ms)?)
        .execute(&self.pool)
        .await?;
        Ok(record.clone())
    }

    async fn list_payment_transaction_records_for_order(
        &self,
        payment_order_id: &str,
    ) -> Result<Vec<PaymentTransactionRecord>> {
        let rows = sqlx::query(
            "SELECT payment_transaction_id, tenant_id, organization_id, payment_order_id,
                    payment_attempt_id, transaction_kind, provider_code, provider_transaction_id,
                    currency_code, amount_minor, fee_minor, net_amount_minor, provider_status,
                    raw_event_id, occurred_at_ms, created_at_ms
             FROM ai_payment_transaction
             WHERE payment_order_id = $1
             ORDER BY occurred_at_ms DESC, payment_transaction_id",
        )
        .bind(payment_order_id)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter()
            .map(|row| decode_payment_transaction_row(&row))
            .collect()
    }

    async fn insert_refund_order_record(
        &self,
        record: &RefundOrderRecord,
    ) -> Result<RefundOrderRecord> {
        sqlx::query(
            "INSERT INTO ai_refund_order (
                refund_order_id, tenant_id, organization_id, payment_order_id, commerce_order_id,
                refund_reason_code, requested_by_type, requested_by_id, currency_code,
                requested_amount_minor, approved_amount_minor, refunded_amount_minor,
                refund_status, created_at_ms, updated_at_ms
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15
            )
            ON CONFLICT (refund_order_id) DO UPDATE SET
                tenant_id = EXCLUDED.tenant_id,
                organization_id = EXCLUDED.organization_id,
                payment_order_id = EXCLUDED.payment_order_id,
                commerce_order_id = EXCLUDED.commerce_order_id,
                refund_reason_code = EXCLUDED.refund_reason_code,
                requested_by_type = EXCLUDED.requested_by_type,
                requested_by_id = EXCLUDED.requested_by_id,
                currency_code = EXCLUDED.currency_code,
                requested_amount_minor = EXCLUDED.requested_amount_minor,
                approved_amount_minor = EXCLUDED.approved_amount_minor,
                refunded_amount_minor = EXCLUDED.refunded_amount_minor,
                refund_status = EXCLUDED.refund_status,
                created_at_ms = EXCLUDED.created_at_ms,
                updated_at_ms = EXCLUDED.updated_at_ms",
        )
        .bind(&record.refund_order_id)
        .bind(i64::try_from(record.tenant_id)?)
        .bind(i64::try_from(record.organization_id)?)
        .bind(&record.payment_order_id)
        .bind(&record.commerce_order_id)
        .bind(&record.refund_reason_code)
        .bind(&record.requested_by_type)
        .bind(&record.requested_by_id)
        .bind(&record.currency_code)
        .bind(i64::try_from(record.requested_amount_minor)?)
        .bind(
            record
                .approved_amount_minor
                .map(i64::try_from)
                .transpose()?,
        )
        .bind(i64::try_from(record.refunded_amount_minor)?)
        .bind(record.refund_status.as_str())
        .bind(i64::try_from(record.created_at_ms)?)
        .bind(i64::try_from(record.updated_at_ms)?)
        .execute(&self.pool)
        .await?;
        Ok(record.clone())
    }

    async fn list_refund_order_records_for_payment_order(
        &self,
        payment_order_id: &str,
    ) -> Result<Vec<RefundOrderRecord>> {
        let rows = sqlx::query(
            "SELECT refund_order_id, tenant_id, organization_id, payment_order_id,
                    commerce_order_id, refund_reason_code, requested_by_type,
                    requested_by_id, currency_code, requested_amount_minor,
                    approved_amount_minor, refunded_amount_minor, refund_status,
                    created_at_ms, updated_at_ms
             FROM ai_refund_order
             WHERE payment_order_id = $1
             ORDER BY created_at_ms DESC, refund_order_id",
        )
        .bind(payment_order_id)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter()
            .map(|row| decode_refund_order_row(&row))
            .collect()
    }

    async fn find_refund_order_record(
        &self,
        refund_order_id: &str,
    ) -> Result<Option<RefundOrderRecord>> {
        let row = sqlx::query(
            "SELECT refund_order_id, tenant_id, organization_id, payment_order_id,
                    commerce_order_id, refund_reason_code, requested_by_type,
                    requested_by_id, currency_code, requested_amount_minor,
                    approved_amount_minor, refunded_amount_minor, refund_status,
                    created_at_ms, updated_at_ms
             FROM ai_refund_order
             WHERE refund_order_id = $1",
        )
        .bind(refund_order_id)
        .fetch_optional(&self.pool)
        .await?;
        row.map(|row| decode_refund_order_row(&row)).transpose()
    }

    async fn apply_refund_order_quota_reversal(
        &self,
        refund_order_id: &str,
        project_id: &str,
        target_kind: &str,
        target_units: u64,
    ) -> Result<bool> {
        PostgresAdminStore::apply_refund_order_quota_reversal(
            self,
            refund_order_id,
            project_id,
            target_kind,
            target_units,
        )
        .await
    }

    async fn insert_finance_journal_entry_record(
        &self,
        record: &FinanceJournalEntryRecord,
    ) -> Result<FinanceJournalEntryRecord> {
        sqlx::query(
            "INSERT INTO ai_finance_journal_entry (
                finance_journal_entry_id, tenant_id, organization_id, source_kind, source_id,
                entry_code, currency_code, entry_status, occurred_at_ms, created_at_ms
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (finance_journal_entry_id) DO UPDATE SET
                tenant_id = EXCLUDED.tenant_id,
                organization_id = EXCLUDED.organization_id,
                source_kind = EXCLUDED.source_kind,
                source_id = EXCLUDED.source_id,
                entry_code = EXCLUDED.entry_code,
                currency_code = EXCLUDED.currency_code,
                entry_status = EXCLUDED.entry_status,
                occurred_at_ms = EXCLUDED.occurred_at_ms,
                created_at_ms = EXCLUDED.created_at_ms",
        )
        .bind(&record.finance_journal_entry_id)
        .bind(i64::try_from(record.tenant_id)?)
        .bind(i64::try_from(record.organization_id)?)
        .bind(&record.source_kind)
        .bind(&record.source_id)
        .bind(record.entry_code.as_str())
        .bind(&record.currency_code)
        .bind(&record.entry_status)
        .bind(i64::try_from(record.occurred_at_ms)?)
        .bind(i64::try_from(record.created_at_ms)?)
        .execute(&self.pool)
        .await?;
        Ok(record.clone())
    }

    async fn list_finance_journal_entry_records(&self) -> Result<Vec<FinanceJournalEntryRecord>> {
        let rows = sqlx::query(
            "SELECT finance_journal_entry_id, tenant_id, organization_id, source_kind, source_id,
                    entry_code, currency_code, entry_status, occurred_at_ms, created_at_ms
             FROM ai_finance_journal_entry
             ORDER BY occurred_at_ms DESC, finance_journal_entry_id",
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter()
            .map(|row| decode_finance_journal_entry_row(&row))
            .collect()
    }

    async fn insert_finance_journal_line_record(
        &self,
        record: &FinanceJournalLineRecord,
    ) -> Result<FinanceJournalLineRecord> {
        sqlx::query(
            "INSERT INTO ai_finance_journal_line (
                finance_journal_line_id, tenant_id, organization_id, finance_journal_entry_id,
                line_no, account_code, direction, amount_minor, party_type, party_id
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (finance_journal_line_id) DO UPDATE SET
                tenant_id = EXCLUDED.tenant_id,
                organization_id = EXCLUDED.organization_id,
                finance_journal_entry_id = EXCLUDED.finance_journal_entry_id,
                line_no = EXCLUDED.line_no,
                account_code = EXCLUDED.account_code,
                direction = EXCLUDED.direction,
                amount_minor = EXCLUDED.amount_minor,
                party_type = EXCLUDED.party_type,
                party_id = EXCLUDED.party_id",
        )
        .bind(&record.finance_journal_line_id)
        .bind(i64::try_from(record.tenant_id)?)
        .bind(i64::try_from(record.organization_id)?)
        .bind(&record.finance_journal_entry_id)
        .bind(i64::from(record.line_no))
        .bind(&record.account_code)
        .bind(record.direction.as_str())
        .bind(i64::try_from(record.amount_minor)?)
        .bind(record.party_type.as_deref())
        .bind(record.party_id.as_deref())
        .execute(&self.pool)
        .await?;
        Ok(record.clone())
    }

    async fn list_finance_journal_line_records(
        &self,
        finance_journal_entry_id: &str,
    ) -> Result<Vec<FinanceJournalLineRecord>> {
        let rows = sqlx::query(
            "SELECT finance_journal_line_id, tenant_id, organization_id, finance_journal_entry_id,
                    line_no, account_code, direction, amount_minor, party_type, party_id
             FROM ai_finance_journal_line
             WHERE finance_journal_entry_id = $1
             ORDER BY line_no ASC, finance_journal_line_id",
        )
        .bind(finance_journal_entry_id)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter()
            .map(|row| decode_finance_journal_line_row(&row))
            .collect()
    }

    async fn insert_reconciliation_match_summary_record(
        &self,
        record: &ReconciliationMatchSummaryRecord,
    ) -> Result<ReconciliationMatchSummaryRecord> {
        sqlx::query(
            "INSERT INTO ai_payment_reconciliation_line (
                reconciliation_line_id, tenant_id, organization_id, reconciliation_batch_id,
                provider_transaction_id, payment_order_id, refund_order_id,
                provider_amount_minor, local_amount_minor, match_status, reason_code,
                created_at_ms, updated_at_ms
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            ON CONFLICT (reconciliation_line_id) DO UPDATE SET
                tenant_id = EXCLUDED.tenant_id,
                organization_id = EXCLUDED.organization_id,
                reconciliation_batch_id = EXCLUDED.reconciliation_batch_id,
                provider_transaction_id = EXCLUDED.provider_transaction_id,
                payment_order_id = EXCLUDED.payment_order_id,
                refund_order_id = EXCLUDED.refund_order_id,
                provider_amount_minor = EXCLUDED.provider_amount_minor,
                local_amount_minor = EXCLUDED.local_amount_minor,
                match_status = EXCLUDED.match_status,
                reason_code = EXCLUDED.reason_code,
                created_at_ms = EXCLUDED.created_at_ms,
                updated_at_ms = EXCLUDED.updated_at_ms",
        )
        .bind(&record.reconciliation_line_id)
        .bind(i64::try_from(record.tenant_id)?)
        .bind(i64::try_from(record.organization_id)?)
        .bind(&record.reconciliation_batch_id)
        .bind(&record.provider_transaction_id)
        .bind(record.payment_order_id.as_deref())
        .bind(record.refund_order_id.as_deref())
        .bind(i64::try_from(record.provider_amount_minor)?)
        .bind(record.local_amount_minor.map(i64::try_from).transpose()?)
        .bind(record.match_status.as_str())
        .bind(record.reason_code.as_deref())
        .bind(i64::try_from(record.created_at_ms)?)
        .bind(i64::try_from(record.updated_at_ms)?)
        .execute(&self.pool)
        .await?;
        Ok(record.clone())
    }

    async fn list_reconciliation_match_summary_records(
        &self,
        reconciliation_batch_id: &str,
    ) -> Result<Vec<ReconciliationMatchSummaryRecord>> {
        let rows = sqlx::query(
            "SELECT reconciliation_line_id, tenant_id, organization_id, reconciliation_batch_id,
                    provider_transaction_id, payment_order_id, refund_order_id,
                    provider_amount_minor, local_amount_minor, match_status, reason_code,
                    created_at_ms, updated_at_ms
             FROM ai_payment_reconciliation_line
             WHERE reconciliation_batch_id = $1
             ORDER BY created_at_ms DESC, reconciliation_line_id",
        )
        .bind(reconciliation_batch_id)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter()
            .map(|row| decode_reconciliation_match_summary_row(&row))
            .collect()
    }

    async fn find_reconciliation_match_summary_record(
        &self,
        reconciliation_line_id: &str,
    ) -> Result<Option<ReconciliationMatchSummaryRecord>> {
        sqlx::query(
            "SELECT reconciliation_line_id, tenant_id, organization_id, reconciliation_batch_id,
                    provider_transaction_id, payment_order_id, refund_order_id,
                    provider_amount_minor, local_amount_minor, match_status, reason_code,
                    created_at_ms, updated_at_ms
             FROM ai_payment_reconciliation_line
             WHERE reconciliation_line_id = $1",
        )
        .bind(reconciliation_line_id)
        .fetch_optional(&self.pool)
        .await?
        .map(|row| decode_reconciliation_match_summary_row(&row))
        .transpose()
    }

    async fn list_all_reconciliation_match_summary_records(
        &self,
    ) -> Result<Vec<ReconciliationMatchSummaryRecord>> {
        let rows = sqlx::query(
            "SELECT reconciliation_line_id, tenant_id, organization_id, reconciliation_batch_id,
                    provider_transaction_id, payment_order_id, refund_order_id,
                    provider_amount_minor, local_amount_minor, match_status, reason_code,
                    created_at_ms, updated_at_ms
             FROM ai_payment_reconciliation_line
             ORDER BY created_at_ms DESC, reconciliation_line_id",
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter()
            .map(|row| decode_reconciliation_match_summary_row(&row))
            .collect()
    }
}
