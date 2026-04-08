use super::*;

impl SqliteAdminStore {
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
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
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
            "SELECT order_id, project_id, user_id, target_kind, target_id, target_name, list_price_cents, payable_price_cents, list_price_label, payable_price_label, granted_units, bonus_units, currency_code, pricing_plan_id, pricing_plan_version, pricing_snapshot_json, applied_coupon_code, coupon_reservation_id, coupon_redemption_id, marketing_campaign_id, subsidy_amount_minor, payment_method_id, latest_payment_attempt_id, status, settlement_status, source, refundable_amount_minor, refunded_amount_minor, created_at_ms, updated_at_ms
             FROM ai_commerce_orders
             ORDER BY updated_at_ms DESC, created_at_ms DESC, order_id DESC",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(Self::map_sqlite_commerce_order_row)
            .collect::<Result<Vec<_>>>()?)
    }

    pub async fn list_commerce_orders_for_project(
        &self,
        project_id: &str,
    ) -> Result<Vec<CommerceOrderRecord>> {
        let rows = sqlx::query(
            "SELECT order_id, project_id, user_id, target_kind, target_id, target_name, list_price_cents, payable_price_cents, list_price_label, payable_price_label, granted_units, bonus_units, currency_code, pricing_plan_id, pricing_plan_version, pricing_snapshot_json, applied_coupon_code, coupon_reservation_id, coupon_redemption_id, marketing_campaign_id, subsidy_amount_minor, payment_method_id, latest_payment_attempt_id, status, settlement_status, source, refundable_amount_minor, refunded_amount_minor, created_at_ms, updated_at_ms
             FROM ai_commerce_orders
             WHERE project_id = ?
             ORDER BY updated_at_ms DESC, created_at_ms DESC, order_id DESC",
        )
        .bind(project_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(Self::map_sqlite_commerce_order_row)
            .collect::<Result<Vec<_>>>()?)
    }

    pub async fn list_recent_commerce_orders(
        &self,
        limit: usize,
    ) -> Result<Vec<CommerceOrderRecord>> {
        let rows = sqlx::query(
            "SELECT order_id, project_id, user_id, target_kind, target_id, target_name, list_price_cents, payable_price_cents, list_price_label, payable_price_label, granted_units, bonus_units, currency_code, pricing_plan_id, pricing_plan_version, pricing_snapshot_json, applied_coupon_code, coupon_reservation_id, coupon_redemption_id, marketing_campaign_id, subsidy_amount_minor, payment_method_id, latest_payment_attempt_id, status, settlement_status, source, refundable_amount_minor, refunded_amount_minor, created_at_ms, updated_at_ms
             FROM ai_commerce_orders
             ORDER BY updated_at_ms DESC, created_at_ms DESC, order_id DESC
             LIMIT ?",
        )
        .bind(i64::try_from(limit)?)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(Self::map_sqlite_commerce_order_row)
            .collect::<Result<Vec<_>>>()?)
    }

    pub async fn list_commerce_orders_for_project_after(
        &self,
        project_id: &str,
        last_order_updated_at_ms: u64,
        last_order_created_at_ms: u64,
        last_order_id: &str,
    ) -> Result<Vec<CommerceOrderRecord>> {
        let rows = sqlx::query(
            "SELECT order_id, project_id, user_id, target_kind, target_id, target_name, list_price_cents, payable_price_cents, list_price_label, payable_price_label, granted_units, bonus_units, currency_code, pricing_plan_id, pricing_plan_version, pricing_snapshot_json, applied_coupon_code, coupon_reservation_id, coupon_redemption_id, marketing_campaign_id, subsidy_amount_minor, payment_method_id, latest_payment_attempt_id, status, settlement_status, source, refundable_amount_minor, refunded_amount_minor, created_at_ms, updated_at_ms
             FROM ai_commerce_orders
             WHERE project_id = ?
               AND (
                    updated_at_ms > ?
                    OR (
                        updated_at_ms = ?
                        AND (
                            created_at_ms > ?
                            OR (created_at_ms = ? AND order_id > ?)
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
        Ok(rows
            .into_iter()
            .map(Self::map_sqlite_commerce_order_row)
            .collect::<Result<Vec<_>>>()?)
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
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
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
            return Err(anyhow::anyhow!(
                "commerce payment event {} already belongs to another order",
                event.dedupe_key
            ));
        }

        Ok(event.clone())
    }

    pub async fn list_commerce_payment_events(&self) -> Result<Vec<CommercePaymentEventRecord>> {
        let rows = sqlx::query(
            "SELECT payment_event_id, order_id, project_id, user_id, provider, provider_event_id, dedupe_key, event_type, payload_json, processing_status, processing_message, received_at_ms, processed_at_ms, order_status_after
             FROM ai_commerce_payment_events
             ORDER BY received_at_ms DESC, payment_event_id DESC",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(Self::map_sqlite_commerce_payment_event_row)
            .collect::<Result<Vec<_>>>()?)
    }

    pub async fn find_commerce_payment_event_by_dedupe_key(
        &self,
        dedupe_key: &str,
    ) -> Result<Option<CommercePaymentEventRecord>> {
        let row = sqlx::query(
            "SELECT payment_event_id, order_id, project_id, user_id, provider, provider_event_id, dedupe_key, event_type, payload_json, processing_status, processing_message, received_at_ms, processed_at_ms, order_status_after
             FROM ai_commerce_payment_events
             WHERE dedupe_key = ?",
        )
        .bind(dedupe_key)
        .fetch_optional(&self.pool)
        .await?;
        row.map(Self::map_sqlite_commerce_payment_event_row)
            .transpose()
    }

    pub async fn upsert_payment_method(
        &self,
        payment_method: &PaymentMethodRecord,
    ) -> Result<PaymentMethodRecord> {
        sqlx::query(
            "INSERT INTO ai_payment_methods (
                payment_method_id,
                display_name,
                description,
                provider,
                channel,
                mode,
                enabled,
                sort_order,
                capability_codes_json,
                supported_currency_codes_json,
                supported_country_codes_json,
                supported_order_kinds_json,
                callback_strategy,
                webhook_path,
                webhook_tolerance_seconds,
                replay_window_seconds,
                max_retry_count,
                config_json,
                created_at_ms,
                updated_at_ms
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(payment_method_id) DO UPDATE SET
                display_name = excluded.display_name,
                description = excluded.description,
                provider = excluded.provider,
                channel = excluded.channel,
                mode = excluded.mode,
                enabled = excluded.enabled,
                sort_order = excluded.sort_order,
                capability_codes_json = excluded.capability_codes_json,
                supported_currency_codes_json = excluded.supported_currency_codes_json,
                supported_country_codes_json = excluded.supported_country_codes_json,
                supported_order_kinds_json = excluded.supported_order_kinds_json,
                callback_strategy = excluded.callback_strategy,
                webhook_path = excluded.webhook_path,
                webhook_tolerance_seconds = excluded.webhook_tolerance_seconds,
                replay_window_seconds = excluded.replay_window_seconds,
                max_retry_count = excluded.max_retry_count,
                config_json = excluded.config_json,
                created_at_ms = excluded.created_at_ms,
                updated_at_ms = excluded.updated_at_ms",
        )
        .bind(&payment_method.payment_method_id)
        .bind(&payment_method.display_name)
        .bind(&payment_method.description)
        .bind(&payment_method.provider)
        .bind(&payment_method.channel)
        .bind(&payment_method.mode)
        .bind(payment_method.enabled)
        .bind(payment_method.sort_order)
        .bind(encode_string_list(&payment_method.capability_codes)?)
        .bind(encode_string_list(&payment_method.supported_currency_codes)?)
        .bind(encode_string_list(&payment_method.supported_country_codes)?)
        .bind(encode_string_list(&payment_method.supported_order_kinds)?)
        .bind(&payment_method.callback_strategy)
        .bind(&payment_method.webhook_path)
        .bind(i64::try_from(payment_method.webhook_tolerance_seconds)?)
        .bind(i64::try_from(payment_method.replay_window_seconds)?)
        .bind(i64::from(payment_method.max_retry_count))
        .bind(&payment_method.config_json)
        .bind(i64::try_from(payment_method.created_at_ms)?)
        .bind(i64::try_from(payment_method.updated_at_ms)?)
        .execute(&self.pool)
        .await?;
        Ok(payment_method.clone())
    }

    pub async fn list_payment_methods(&self) -> Result<Vec<PaymentMethodRecord>> {
        let rows = sqlx::query(
            "SELECT payment_method_id, display_name, description, provider, channel, mode, enabled,
                    sort_order, capability_codes_json, supported_currency_codes_json,
                    supported_country_codes_json, supported_order_kinds_json, callback_strategy,
                    webhook_path, webhook_tolerance_seconds, replay_window_seconds,
                    max_retry_count, config_json, created_at_ms, updated_at_ms
             FROM ai_payment_methods
             ORDER BY enabled DESC, sort_order ASC, updated_at_ms DESC, payment_method_id ASC",
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter()
            .map(Self::map_sqlite_payment_method_row)
            .collect()
    }

    pub async fn find_payment_method(
        &self,
        payment_method_id: &str,
    ) -> Result<Option<PaymentMethodRecord>> {
        let row = sqlx::query(
            "SELECT payment_method_id, display_name, description, provider, channel, mode, enabled,
                    sort_order, capability_codes_json, supported_currency_codes_json,
                    supported_country_codes_json, supported_order_kinds_json, callback_strategy,
                    webhook_path, webhook_tolerance_seconds, replay_window_seconds,
                    max_retry_count, config_json, created_at_ms, updated_at_ms
             FROM ai_payment_methods
             WHERE payment_method_id = ?",
        )
        .bind(payment_method_id)
        .fetch_optional(&self.pool)
        .await?;
        row.map(Self::map_sqlite_payment_method_row).transpose()
    }

    pub async fn delete_payment_method(&self, payment_method_id: &str) -> Result<bool> {
        let mut tx = self.pool.begin().await?;
        sqlx::query(
            "DELETE FROM ai_payment_method_credential_bindings
             WHERE payment_method_id = ?",
        )
        .bind(payment_method_id)
        .execute(&mut *tx)
        .await?;
        let result = sqlx::query(
            "DELETE FROM ai_payment_methods
             WHERE payment_method_id = ?",
        )
        .bind(payment_method_id)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn upsert_payment_method_credential_binding(
        &self,
        binding: &PaymentMethodCredentialBindingRecord,
    ) -> Result<PaymentMethodCredentialBindingRecord> {
        sqlx::query(
            "INSERT INTO ai_payment_method_credential_bindings (
                binding_id,
                payment_method_id,
                usage_kind,
                credential_tenant_id,
                credential_provider_id,
                credential_key_reference,
                created_at_ms,
                updated_at_ms
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(payment_method_id, usage_kind) DO UPDATE SET
                binding_id = excluded.binding_id,
                credential_tenant_id = excluded.credential_tenant_id,
                credential_provider_id = excluded.credential_provider_id,
                credential_key_reference = excluded.credential_key_reference,
                created_at_ms = excluded.created_at_ms,
                updated_at_ms = excluded.updated_at_ms",
        )
        .bind(&binding.binding_id)
        .bind(&binding.payment_method_id)
        .bind(&binding.usage_kind)
        .bind(&binding.credential_tenant_id)
        .bind(&binding.credential_provider_id)
        .bind(&binding.credential_key_reference)
        .bind(i64::try_from(binding.created_at_ms)?)
        .bind(i64::try_from(binding.updated_at_ms)?)
        .execute(&self.pool)
        .await?;
        Ok(binding.clone())
    }

    pub async fn list_payment_method_credential_bindings(
        &self,
        payment_method_id: &str,
    ) -> Result<Vec<PaymentMethodCredentialBindingRecord>> {
        let rows = sqlx::query(
            "SELECT binding_id, payment_method_id, usage_kind, credential_tenant_id,
                    credential_provider_id, credential_key_reference, created_at_ms, updated_at_ms
             FROM ai_payment_method_credential_bindings
             WHERE payment_method_id = ?
             ORDER BY updated_at_ms DESC, binding_id DESC",
        )
        .bind(payment_method_id)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter()
            .map(Self::map_sqlite_payment_method_credential_binding_row)
            .collect()
    }

    pub async fn delete_payment_method_credential_binding(
        &self,
        payment_method_id: &str,
        binding_id: &str,
    ) -> Result<bool> {
        let result = sqlx::query(
            "DELETE FROM ai_payment_method_credential_bindings
             WHERE payment_method_id = ? AND binding_id = ?",
        )
        .bind(payment_method_id)
        .bind(binding_id)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn upsert_commerce_payment_attempt(
        &self,
        attempt: &CommercePaymentAttemptRecord,
    ) -> Result<CommercePaymentAttemptRecord> {
        if self
            .find_commerce_payment_attempt(&attempt.payment_attempt_id)
            .await?
            .is_some()
        {
            sqlx::query(
                "UPDATE ai_commerce_payment_attempts
                 SET order_id = ?, project_id = ?, user_id = ?, payment_method_id = ?, provider = ?,
                     channel = ?, status = ?, idempotency_key = ?, attempt_sequence = ?,
                     amount_minor = ?, currency_code = ?, captured_amount_minor = ?,
                     refunded_amount_minor = ?, provider_payment_intent_id = ?,
                     provider_checkout_session_id = ?, provider_reference = ?, checkout_url = ?,
                     qr_code_payload = ?, request_payload_json = ?, response_payload_json = ?,
                     error_code = ?, error_message = ?, initiated_at_ms = ?, expires_at_ms = ?,
                     completed_at_ms = ?, updated_at_ms = ?
                 WHERE payment_attempt_id = ?",
            )
            .bind(&attempt.order_id)
            .bind(&attempt.project_id)
            .bind(&attempt.user_id)
            .bind(&attempt.payment_method_id)
            .bind(&attempt.provider)
            .bind(&attempt.channel)
            .bind(&attempt.status)
            .bind(&attempt.idempotency_key)
            .bind(i64::from(attempt.attempt_sequence))
            .bind(i64::try_from(attempt.amount_minor)?)
            .bind(&attempt.currency_code)
            .bind(i64::try_from(attempt.captured_amount_minor)?)
            .bind(i64::try_from(attempt.refunded_amount_minor)?)
            .bind(&attempt.provider_payment_intent_id)
            .bind(&attempt.provider_checkout_session_id)
            .bind(&attempt.provider_reference)
            .bind(&attempt.checkout_url)
            .bind(&attempt.qr_code_payload)
            .bind(&attempt.request_payload_json)
            .bind(&attempt.response_payload_json)
            .bind(&attempt.error_code)
            .bind(&attempt.error_message)
            .bind(i64::try_from(attempt.initiated_at_ms)?)
            .bind(attempt.expires_at_ms.map(i64::try_from).transpose()?)
            .bind(attempt.completed_at_ms.map(i64::try_from).transpose()?)
            .bind(i64::try_from(attempt.updated_at_ms)?)
            .bind(&attempt.payment_attempt_id)
            .execute(&self.pool)
            .await?;
            return Ok(attempt.clone());
        }

        if let Some(existing) = self
            .find_commerce_payment_attempt_by_idempotency_key(&attempt.idempotency_key)
            .await?
        {
            if existing.order_id == attempt.order_id
                && existing.payment_method_id == attempt.payment_method_id
            {
                return Ok(existing);
            }
            return Err(anyhow::anyhow!(
                "payment attempt idempotency key {} already belongs to another order or payment method",
                attempt.idempotency_key
            ));
        }

        sqlx::query(
            "INSERT INTO ai_commerce_payment_attempts (
                payment_attempt_id,
                order_id,
                project_id,
                user_id,
                payment_method_id,
                provider,
                channel,
                status,
                idempotency_key,
                attempt_sequence,
                amount_minor,
                currency_code,
                captured_amount_minor,
                refunded_amount_minor,
                provider_payment_intent_id,
                provider_checkout_session_id,
                provider_reference,
                checkout_url,
                qr_code_payload,
                request_payload_json,
                response_payload_json,
                error_code,
                error_message,
                initiated_at_ms,
                expires_at_ms,
                completed_at_ms,
                updated_at_ms
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&attempt.payment_attempt_id)
        .bind(&attempt.order_id)
        .bind(&attempt.project_id)
        .bind(&attempt.user_id)
        .bind(&attempt.payment_method_id)
        .bind(&attempt.provider)
        .bind(&attempt.channel)
        .bind(&attempt.status)
        .bind(&attempt.idempotency_key)
        .bind(i64::from(attempt.attempt_sequence))
        .bind(i64::try_from(attempt.amount_minor)?)
        .bind(&attempt.currency_code)
        .bind(i64::try_from(attempt.captured_amount_minor)?)
        .bind(i64::try_from(attempt.refunded_amount_minor)?)
        .bind(&attempt.provider_payment_intent_id)
        .bind(&attempt.provider_checkout_session_id)
        .bind(&attempt.provider_reference)
        .bind(&attempt.checkout_url)
        .bind(&attempt.qr_code_payload)
        .bind(&attempt.request_payload_json)
        .bind(&attempt.response_payload_json)
        .bind(&attempt.error_code)
        .bind(&attempt.error_message)
        .bind(i64::try_from(attempt.initiated_at_ms)?)
        .bind(attempt.expires_at_ms.map(i64::try_from).transpose()?)
        .bind(attempt.completed_at_ms.map(i64::try_from).transpose()?)
        .bind(i64::try_from(attempt.updated_at_ms)?)
        .execute(&self.pool)
        .await?;
        Ok(attempt.clone())
    }

    pub async fn list_commerce_payment_attempts(&self) -> Result<Vec<CommercePaymentAttemptRecord>> {
        let rows = sqlx::query(
            "SELECT payment_attempt_id, order_id, project_id, user_id, payment_method_id, provider,
                    channel, status, idempotency_key, attempt_sequence, amount_minor,
                    currency_code, captured_amount_minor, refunded_amount_minor,
                    provider_payment_intent_id, provider_checkout_session_id, provider_reference,
                    checkout_url, qr_code_payload, request_payload_json, response_payload_json,
                    error_code, error_message, initiated_at_ms, expires_at_ms, completed_at_ms,
                    updated_at_ms
             FROM ai_commerce_payment_attempts
             ORDER BY updated_at_ms DESC, initiated_at_ms DESC, payment_attempt_id DESC",
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter()
            .map(Self::map_sqlite_commerce_payment_attempt_row)
            .collect()
    }

    pub async fn find_commerce_payment_attempt(
        &self,
        payment_attempt_id: &str,
    ) -> Result<Option<CommercePaymentAttemptRecord>> {
        let row = sqlx::query(
            "SELECT payment_attempt_id, order_id, project_id, user_id, payment_method_id, provider,
                    channel, status, idempotency_key, attempt_sequence, amount_minor,
                    currency_code, captured_amount_minor, refunded_amount_minor,
                    provider_payment_intent_id, provider_checkout_session_id, provider_reference,
                    checkout_url, qr_code_payload, request_payload_json, response_payload_json,
                    error_code, error_message, initiated_at_ms, expires_at_ms, completed_at_ms,
                    updated_at_ms
             FROM ai_commerce_payment_attempts
             WHERE payment_attempt_id = ?",
        )
        .bind(payment_attempt_id)
        .fetch_optional(&self.pool)
        .await?;
        row.map(Self::map_sqlite_commerce_payment_attempt_row)
            .transpose()
    }

    pub async fn find_commerce_payment_attempt_by_idempotency_key(
        &self,
        idempotency_key: &str,
    ) -> Result<Option<CommercePaymentAttemptRecord>> {
        let row = sqlx::query(
            "SELECT payment_attempt_id, order_id, project_id, user_id, payment_method_id, provider,
                    channel, status, idempotency_key, attempt_sequence, amount_minor,
                    currency_code, captured_amount_minor, refunded_amount_minor,
                    provider_payment_intent_id, provider_checkout_session_id, provider_reference,
                    checkout_url, qr_code_payload, request_payload_json, response_payload_json,
                    error_code, error_message, initiated_at_ms, expires_at_ms, completed_at_ms,
                    updated_at_ms
             FROM ai_commerce_payment_attempts
             WHERE idempotency_key = ?",
        )
        .bind(idempotency_key)
        .fetch_optional(&self.pool)
        .await?;
        row.map(Self::map_sqlite_commerce_payment_attempt_row)
            .transpose()
    }

    pub async fn list_commerce_payment_attempts_for_order(
        &self,
        order_id: &str,
    ) -> Result<Vec<CommercePaymentAttemptRecord>> {
        let rows = sqlx::query(
            "SELECT payment_attempt_id, order_id, project_id, user_id, payment_method_id, provider,
                    channel, status, idempotency_key, attempt_sequence, amount_minor,
                    currency_code, captured_amount_minor, refunded_amount_minor,
                    provider_payment_intent_id, provider_checkout_session_id, provider_reference,
                    checkout_url, qr_code_payload, request_payload_json, response_payload_json,
                    error_code, error_message, initiated_at_ms, expires_at_ms, completed_at_ms,
                    updated_at_ms
             FROM ai_commerce_payment_attempts
             WHERE order_id = ?
             ORDER BY updated_at_ms DESC, initiated_at_ms DESC, payment_attempt_id DESC",
        )
        .bind(order_id)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter()
            .map(Self::map_sqlite_commerce_payment_attempt_row)
            .collect()
    }

}
