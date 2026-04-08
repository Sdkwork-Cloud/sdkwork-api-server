use super::*;

impl PostgresAdminStore {
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
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
                $11, $12, $13, $14, $15, $16, $17, $18, $19, $20
            )
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
        .bind(encode_string_list(
            &payment_method.supported_currency_codes,
        )?)
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
            .map(Self::map_postgres_payment_method_row)
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
             WHERE payment_method_id = $1",
        )
        .bind(payment_method_id)
        .fetch_optional(&self.pool)
        .await?;
        row.map(Self::map_postgres_payment_method_row).transpose()
    }

    pub async fn delete_payment_method(&self, payment_method_id: &str) -> Result<bool> {
        let mut tx = self.pool.begin().await?;
        sqlx::query(
            "DELETE FROM ai_payment_method_credential_bindings
             WHERE payment_method_id = $1",
        )
        .bind(payment_method_id)
        .execute(&mut *tx)
        .await?;
        let result = sqlx::query(
            "DELETE FROM ai_payment_methods
             WHERE payment_method_id = $1",
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
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
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
             WHERE payment_method_id = $1
             ORDER BY updated_at_ms DESC, binding_id DESC",
        )
        .bind(payment_method_id)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter()
            .map(Self::map_postgres_payment_method_credential_binding_row)
            .collect()
    }

    pub async fn delete_payment_method_credential_binding(
        &self,
        payment_method_id: &str,
        binding_id: &str,
    ) -> Result<bool> {
        let result = sqlx::query(
            "DELETE FROM ai_payment_method_credential_bindings
             WHERE payment_method_id = $1 AND binding_id = $2",
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
                 SET order_id = $1, project_id = $2, user_id = $3, payment_method_id = $4,
                     provider = $5, channel = $6, status = $7, idempotency_key = $8,
                     attempt_sequence = $9, amount_minor = $10, currency_code = $11,
                     captured_amount_minor = $12, refunded_amount_minor = $13,
                     provider_payment_intent_id = $14, provider_checkout_session_id = $15,
                     provider_reference = $16, checkout_url = $17, qr_code_payload = $18,
                     request_payload_json = $19, response_payload_json = $20, error_code = $21,
                     error_message = $22, initiated_at_ms = $23, expires_at_ms = $24,
                     completed_at_ms = $25, updated_at_ms = $26
                 WHERE payment_attempt_id = $27",
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
            return Err(anyhow!(
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
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
                $11, $12, $13, $14, $15, $16, $17, $18, $19, $20,
                $21, $22, $23, $24, $25, $26, $27
            )",
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

    pub async fn list_commerce_payment_attempts(
        &self,
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
             ORDER BY updated_at_ms DESC, initiated_at_ms DESC, payment_attempt_id DESC",
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter()
            .map(Self::map_postgres_commerce_payment_attempt_row)
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
             WHERE payment_attempt_id = $1",
        )
        .bind(payment_attempt_id)
        .fetch_optional(&self.pool)
        .await?;
        row.map(Self::map_postgres_commerce_payment_attempt_row)
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
             WHERE idempotency_key = $1",
        )
        .bind(idempotency_key)
        .fetch_optional(&self.pool)
        .await?;
        row.map(Self::map_postgres_commerce_payment_attempt_row)
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
             WHERE order_id = $1
             ORDER BY updated_at_ms DESC, initiated_at_ms DESC, payment_attempt_id DESC",
        )
        .bind(order_id)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter()
            .map(Self::map_postgres_commerce_payment_attempt_row)
            .collect()
    }
}
