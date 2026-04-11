use super::*;

impl SqliteAdminStore {
    pub(crate) fn map_sqlite_commerce_order_row(
        row: sqlx::sqlite::SqliteRow,
    ) -> Result<CommerceOrderRecord> {
        Ok(CommerceOrderRecord {
            order_id: row.try_get::<String, _>("order_id")?,
            project_id: row.try_get::<String, _>("project_id")?,
            user_id: row.try_get::<String, _>("user_id")?,
            target_kind: row.try_get::<String, _>("target_kind")?,
            target_id: row.try_get::<String, _>("target_id")?,
            target_name: row.try_get::<String, _>("target_name")?,
            list_price_cents: u64::try_from(row.try_get::<i64, _>("list_price_cents")?)?,
            payable_price_cents: u64::try_from(row.try_get::<i64, _>("payable_price_cents")?)?,
            list_price_label: row.try_get::<String, _>("list_price_label")?,
            payable_price_label: row.try_get::<String, _>("payable_price_label")?,
            granted_units: u64::try_from(row.try_get::<i64, _>("granted_units")?)?,
            bonus_units: u64::try_from(row.try_get::<i64, _>("bonus_units")?)?,
            currency_code: row.try_get::<String, _>("currency_code")?,
            pricing_plan_id: row.try_get::<Option<String>, _>("pricing_plan_id")?,
            pricing_plan_version: row
                .try_get::<Option<i64>, _>("pricing_plan_version")?
                .map(u64::try_from)
                .transpose()?,
            pricing_snapshot_json: row.try_get::<String, _>("pricing_snapshot_json")?,
            applied_coupon_code: row.try_get::<Option<String>, _>("applied_coupon_code")?,
            coupon_reservation_id: row.try_get::<Option<String>, _>("coupon_reservation_id")?,
            coupon_redemption_id: row.try_get::<Option<String>, _>("coupon_redemption_id")?,
            marketing_campaign_id: row.try_get::<Option<String>, _>("marketing_campaign_id")?,
            subsidy_amount_minor: u64::try_from(row.try_get::<i64, _>("subsidy_amount_minor")?)?,
            payment_method_id: row.try_get::<Option<String>, _>("payment_method_id")?,
            latest_payment_attempt_id: row
                .try_get::<Option<String>, _>("latest_payment_attempt_id")?,
            status: row.try_get::<String, _>("status")?,
            settlement_status: row.try_get::<String, _>("settlement_status")?,
            source: row.try_get::<String, _>("source")?,
            refundable_amount_minor: u64::try_from(
                row.try_get::<i64, _>("refundable_amount_minor")?,
            )?,
            refunded_amount_minor: u64::try_from(row.try_get::<i64, _>("refunded_amount_minor")?)?,
            created_at_ms: u64::try_from(row.try_get::<i64, _>("created_at_ms")?)?,
            updated_at_ms: u64::try_from(row.try_get::<i64, _>("updated_at_ms")?)?,
        })
    }

    pub(crate) fn map_sqlite_commerce_payment_event_row(
        row: sqlx::sqlite::SqliteRow,
    ) -> Result<CommercePaymentEventRecord> {
        Ok(CommercePaymentEventRecord {
            payment_event_id: row.try_get::<String, _>("payment_event_id")?,
            order_id: row.try_get::<String, _>("order_id")?,
            project_id: row.try_get::<String, _>("project_id")?,
            user_id: row.try_get::<String, _>("user_id")?,
            provider: row.try_get::<String, _>("provider")?,
            provider_event_id: row.try_get::<Option<String>, _>("provider_event_id")?,
            dedupe_key: row.try_get::<String, _>("dedupe_key")?,
            event_type: row.try_get::<String, _>("event_type")?,
            payload_json: row.try_get::<String, _>("payload_json")?,
            processing_status: CommercePaymentEventProcessingStatus::from_str(
                &row.try_get::<String, _>("processing_status")?,
            )
            .map_err(anyhow::Error::msg)?,
            processing_message: row.try_get::<Option<String>, _>("processing_message")?,
            received_at_ms: u64::try_from(row.try_get::<i64, _>("received_at_ms")?)?,
            processed_at_ms: row
                .try_get::<Option<i64>, _>("processed_at_ms")?
                .map(u64::try_from)
                .transpose()?,
            order_status_after: row.try_get::<Option<String>, _>("order_status_after")?,
        })
    }

    pub(crate) fn map_sqlite_payment_method_row(
        row: sqlx::sqlite::SqliteRow,
    ) -> Result<PaymentMethodRecord> {
        Ok(PaymentMethodRecord {
            payment_method_id: row.try_get::<String, _>("payment_method_id")?,
            display_name: row.try_get::<String, _>("display_name")?,
            description: row.try_get::<String, _>("description")?,
            provider: row.try_get::<String, _>("provider")?,
            channel: row.try_get::<String, _>("channel")?,
            mode: row.try_get::<String, _>("mode")?,
            enabled: row.try_get::<i64, _>("enabled")? != 0,
            sort_order: i32::try_from(row.try_get::<i64, _>("sort_order")?)?,
            capability_codes: decode_string_list(
                &row.try_get::<String, _>("capability_codes_json")?,
            )?,
            supported_currency_codes: decode_string_list(
                &row.try_get::<String, _>("supported_currency_codes_json")?,
            )?,
            supported_country_codes: decode_string_list(
                &row.try_get::<String, _>("supported_country_codes_json")?,
            )?,
            supported_order_kinds: decode_string_list(
                &row.try_get::<String, _>("supported_order_kinds_json")?,
            )?,
            callback_strategy: row.try_get::<String, _>("callback_strategy")?,
            webhook_path: row.try_get::<Option<String>, _>("webhook_path")?,
            webhook_tolerance_seconds: u64::try_from(
                row.try_get::<i64, _>("webhook_tolerance_seconds")?,
            )?,
            replay_window_seconds: u64::try_from(row.try_get::<i64, _>("replay_window_seconds")?)?,
            max_retry_count: u32::try_from(row.try_get::<i64, _>("max_retry_count")?)?,
            config_json: row.try_get::<String, _>("config_json")?,
            created_at_ms: u64::try_from(row.try_get::<i64, _>("created_at_ms")?)?,
            updated_at_ms: u64::try_from(row.try_get::<i64, _>("updated_at_ms")?)?,
        })
    }

    pub(crate) fn map_sqlite_payment_method_credential_binding_row(
        row: sqlx::sqlite::SqliteRow,
    ) -> Result<PaymentMethodCredentialBindingRecord> {
        Ok(PaymentMethodCredentialBindingRecord {
            binding_id: row.try_get::<String, _>("binding_id")?,
            payment_method_id: row.try_get::<String, _>("payment_method_id")?,
            usage_kind: row.try_get::<String, _>("usage_kind")?,
            credential_tenant_id: row.try_get::<String, _>("credential_tenant_id")?,
            credential_provider_id: row.try_get::<String, _>("credential_provider_id")?,
            credential_key_reference: row.try_get::<String, _>("credential_key_reference")?,
            created_at_ms: u64::try_from(row.try_get::<i64, _>("created_at_ms")?)?,
            updated_at_ms: u64::try_from(row.try_get::<i64, _>("updated_at_ms")?)?,
        })
    }

    pub(crate) fn map_sqlite_commerce_payment_attempt_row(
        row: sqlx::sqlite::SqliteRow,
    ) -> Result<CommercePaymentAttemptRecord> {
        Ok(CommercePaymentAttemptRecord {
            payment_attempt_id: row.try_get::<String, _>("payment_attempt_id")?,
            order_id: row.try_get::<String, _>("order_id")?,
            project_id: row.try_get::<String, _>("project_id")?,
            user_id: row.try_get::<String, _>("user_id")?,
            payment_method_id: row.try_get::<String, _>("payment_method_id")?,
            provider: row.try_get::<String, _>("provider")?,
            channel: row.try_get::<String, _>("channel")?,
            status: row.try_get::<String, _>("status")?,
            idempotency_key: row.try_get::<String, _>("idempotency_key")?,
            attempt_sequence: u32::try_from(row.try_get::<i64, _>("attempt_sequence")?)?,
            amount_minor: u64::try_from(row.try_get::<i64, _>("amount_minor")?)?,
            currency_code: row.try_get::<String, _>("currency_code")?,
            captured_amount_minor: u64::try_from(row.try_get::<i64, _>("captured_amount_minor")?)?,
            refunded_amount_minor: u64::try_from(row.try_get::<i64, _>("refunded_amount_minor")?)?,
            provider_payment_intent_id: row
                .try_get::<Option<String>, _>("provider_payment_intent_id")?,
            provider_checkout_session_id: row
                .try_get::<Option<String>, _>("provider_checkout_session_id")?,
            provider_reference: row.try_get::<Option<String>, _>("provider_reference")?,
            checkout_url: row.try_get::<Option<String>, _>("checkout_url")?,
            qr_code_payload: row.try_get::<Option<String>, _>("qr_code_payload")?,
            request_payload_json: row.try_get::<String, _>("request_payload_json")?,
            response_payload_json: row.try_get::<String, _>("response_payload_json")?,
            error_code: row.try_get::<Option<String>, _>("error_code")?,
            error_message: row.try_get::<Option<String>, _>("error_message")?,
            initiated_at_ms: u64::try_from(row.try_get::<i64, _>("initiated_at_ms")?)?,
            expires_at_ms: row
                .try_get::<Option<i64>, _>("expires_at_ms")?
                .map(u64::try_from)
                .transpose()?,
            completed_at_ms: row
                .try_get::<Option<i64>, _>("completed_at_ms")?
                .map(u64::try_from)
                .transpose()?,
            updated_at_ms: u64::try_from(row.try_get::<i64, _>("updated_at_ms")?)?,
        })
    }

    pub(crate) fn map_sqlite_commerce_webhook_inbox_row(
        row: sqlx::sqlite::SqliteRow,
    ) -> Result<CommerceWebhookInboxRecord> {
        Ok(CommerceWebhookInboxRecord {
            webhook_inbox_id: row.try_get::<String, _>("webhook_inbox_id")?,
            provider: row.try_get::<String, _>("provider")?,
            payment_method_id: row.try_get::<Option<String>, _>("payment_method_id")?,
            provider_event_id: row.try_get::<Option<String>, _>("provider_event_id")?,
            dedupe_key: row.try_get::<String, _>("dedupe_key")?,
            signature_header: row.try_get::<Option<String>, _>("signature_header")?,
            payload_json: row.try_get::<String, _>("payload_json")?,
            processing_status: row.try_get::<String, _>("processing_status")?,
            retry_count: u32::try_from(row.try_get::<i64, _>("retry_count")?)?,
            max_retry_count: u32::try_from(row.try_get::<i64, _>("max_retry_count")?)?,
            last_error_message: row.try_get::<Option<String>, _>("last_error_message")?,
            next_retry_at_ms: row
                .try_get::<Option<i64>, _>("next_retry_at_ms")?
                .map(u64::try_from)
                .transpose()?,
            first_received_at_ms: u64::try_from(row.try_get::<i64, _>("first_received_at_ms")?)?,
            last_received_at_ms: u64::try_from(row.try_get::<i64, _>("last_received_at_ms")?)?,
            processed_at_ms: row
                .try_get::<Option<i64>, _>("processed_at_ms")?
                .map(u64::try_from)
                .transpose()?,
        })
    }

    pub(crate) fn map_sqlite_commerce_webhook_delivery_attempt_row(
        row: sqlx::sqlite::SqliteRow,
    ) -> Result<CommerceWebhookDeliveryAttemptRecord> {
        Ok(CommerceWebhookDeliveryAttemptRecord {
            delivery_attempt_id: row.try_get::<String, _>("delivery_attempt_id")?,
            webhook_inbox_id: row.try_get::<String, _>("webhook_inbox_id")?,
            processing_status: row.try_get::<String, _>("processing_status")?,
            response_code: row
                .try_get::<Option<i64>, _>("response_code")?
                .map(u16::try_from)
                .transpose()?,
            error_message: row.try_get::<Option<String>, _>("error_message")?,
            started_at_ms: u64::try_from(row.try_get::<i64, _>("started_at_ms")?)?,
            finished_at_ms: row
                .try_get::<Option<i64>, _>("finished_at_ms")?
                .map(u64::try_from)
                .transpose()?,
        })
    }

    pub(crate) fn map_sqlite_commerce_refund_row(
        row: sqlx::sqlite::SqliteRow,
    ) -> Result<CommerceRefundRecord> {
        Ok(CommerceRefundRecord {
            refund_id: row.try_get::<String, _>("refund_id")?,
            order_id: row.try_get::<String, _>("order_id")?,
            payment_attempt_id: row.try_get::<Option<String>, _>("payment_attempt_id")?,
            payment_method_id: row.try_get::<Option<String>, _>("payment_method_id")?,
            provider: row.try_get::<String, _>("provider")?,
            provider_refund_id: row.try_get::<Option<String>, _>("provider_refund_id")?,
            idempotency_key: row.try_get::<String, _>("idempotency_key")?,
            reason: row.try_get::<Option<String>, _>("reason")?,
            status: row.try_get::<String, _>("status")?,
            amount_minor: u64::try_from(row.try_get::<i64, _>("amount_minor")?)?,
            currency_code: row.try_get::<String, _>("currency_code")?,
            request_payload_json: row.try_get::<String, _>("request_payload_json")?,
            response_payload_json: row.try_get::<String, _>("response_payload_json")?,
            created_at_ms: u64::try_from(row.try_get::<i64, _>("created_at_ms")?)?,
            updated_at_ms: u64::try_from(row.try_get::<i64, _>("updated_at_ms")?)?,
            completed_at_ms: row
                .try_get::<Option<i64>, _>("completed_at_ms")?
                .map(u64::try_from)
                .transpose()?,
        })
    }

    pub(crate) fn map_sqlite_commerce_reconciliation_run_row(
        row: sqlx::sqlite::SqliteRow,
    ) -> Result<CommerceReconciliationRunRecord> {
        Ok(CommerceReconciliationRunRecord {
            reconciliation_run_id: row.try_get::<String, _>("reconciliation_run_id")?,
            provider: row.try_get::<String, _>("provider")?,
            payment_method_id: row.try_get::<Option<String>, _>("payment_method_id")?,
            scope_started_at_ms: u64::try_from(row.try_get::<i64, _>("scope_started_at_ms")?)?,
            scope_ended_at_ms: u64::try_from(row.try_get::<i64, _>("scope_ended_at_ms")?)?,
            status: row.try_get::<String, _>("status")?,
            summary_json: row.try_get::<String, _>("summary_json")?,
            created_at_ms: u64::try_from(row.try_get::<i64, _>("created_at_ms")?)?,
            updated_at_ms: u64::try_from(row.try_get::<i64, _>("updated_at_ms")?)?,
            completed_at_ms: row
                .try_get::<Option<i64>, _>("completed_at_ms")?
                .map(u64::try_from)
                .transpose()?,
        })
    }

    pub(crate) fn map_sqlite_commerce_reconciliation_item_row(
        row: sqlx::sqlite::SqliteRow,
    ) -> Result<CommerceReconciliationItemRecord> {
        Ok(CommerceReconciliationItemRecord {
            reconciliation_item_id: row.try_get::<String, _>("reconciliation_item_id")?,
            reconciliation_run_id: row.try_get::<String, _>("reconciliation_run_id")?,
            order_id: row.try_get::<Option<String>, _>("order_id")?,
            payment_attempt_id: row.try_get::<Option<String>, _>("payment_attempt_id")?,
            refund_id: row.try_get::<Option<String>, _>("refund_id")?,
            external_reference: row.try_get::<Option<String>, _>("external_reference")?,
            discrepancy_type: row.try_get::<String, _>("discrepancy_type")?,
            status: row.try_get::<String, _>("status")?,
            expected_amount_minor: row.try_get::<i64, _>("expected_amount_minor")?,
            provider_amount_minor: row.try_get::<Option<i64>, _>("provider_amount_minor")?,
            detail_json: row.try_get::<String, _>("detail_json")?,
            created_at_ms: u64::try_from(row.try_get::<i64, _>("created_at_ms")?)?,
            updated_at_ms: u64::try_from(row.try_get::<i64, _>("updated_at_ms")?)?,
        })
    }

    pub(crate) fn map_sqlite_catalog_publication_lifecycle_audit_row(
        row: sqlx::sqlite::SqliteRow,
    ) -> Result<CatalogPublicationLifecycleAuditRecord> {
        Ok(CatalogPublicationLifecycleAuditRecord {
            audit_id: row.try_get::<String, _>("audit_id")?,
            publication_id: row.try_get::<String, _>("publication_id")?,
            publication_revision_id: row.try_get::<String, _>("publication_revision_id")?,
            publication_version: u64::try_from(row.try_get::<i64, _>("publication_version")?)?,
            publication_source_kind: row.try_get::<String, _>("publication_source_kind")?,
            action: CatalogPublicationLifecycleAction::from_str(
                &row.try_get::<String, _>("action")?,
            )
            .map_err(anyhow::Error::msg)?,
            outcome: CatalogPublicationLifecycleAuditOutcome::from_str(
                &row.try_get::<String, _>("outcome")?,
            )
            .map_err(anyhow::Error::msg)?,
            operator_id: row.try_get::<String, _>("operator_id")?,
            request_id: row.try_get::<String, _>("request_id")?,
            operator_reason: row.try_get::<String, _>("operator_reason")?,
            publication_status_before: row.try_get::<String, _>("publication_status_before")?,
            publication_status_after: row.try_get::<String, _>("publication_status_after")?,
            governed_pricing_plan_id: row
                .try_get::<Option<i64>, _>("governed_pricing_plan_id")?
                .map(u64::try_from)
                .transpose()?,
            governed_pricing_status_before: row
                .try_get::<Option<String>, _>("governed_pricing_status_before")?,
            governed_pricing_status_after: row
                .try_get::<Option<String>, _>("governed_pricing_status_after")?,
            decision_reasons: decode_string_list(
                &row.try_get::<String, _>("decision_reasons_json")?,
            )?,
            recorded_at_ms: u64::try_from(row.try_get::<i64, _>("recorded_at_ms")?)?,
        })
    }
}
