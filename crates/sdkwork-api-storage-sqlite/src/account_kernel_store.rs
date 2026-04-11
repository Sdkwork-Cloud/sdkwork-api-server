use super::*;

#[async_trait]
impl AccountKernelStore for SqliteAdminStore {
    async fn insert_account_record(&self, record: &AccountRecord) -> Result<AccountRecord> {
        sqlx::query(
            "INSERT INTO ai_account (
                account_id, tenant_id, organization_id, user_id, account_type,
                currency_code, credit_unit_code, status, allow_overdraft, overdraft_limit,
                created_at_ms, updated_at_ms
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(account_id) DO UPDATE SET
                tenant_id = excluded.tenant_id,
                organization_id = excluded.organization_id,
                user_id = excluded.user_id,
                account_type = excluded.account_type,
                currency_code = excluded.currency_code,
                credit_unit_code = excluded.credit_unit_code,
                status = excluded.status,
                allow_overdraft = excluded.allow_overdraft,
                overdraft_limit = excluded.overdraft_limit,
                created_at_ms = excluded.created_at_ms,
                updated_at_ms = excluded.updated_at_ms",
        )
        .bind(i64::try_from(record.account_id)?)
        .bind(i64::try_from(record.tenant_id)?)
        .bind(i64::try_from(record.organization_id)?)
        .bind(i64::try_from(record.user_id)?)
        .bind(account_type_as_str(record.account_type))
        .bind(&record.currency_code)
        .bind(&record.credit_unit_code)
        .bind(account_status_as_str(record.status))
        .bind(if record.allow_overdraft { 1_i64 } else { 0_i64 })
        .bind(record.overdraft_limit)
        .bind(i64::try_from(record.created_at_ms)?)
        .bind(i64::try_from(record.updated_at_ms)?)
        .execute(&self.pool)
        .await?;
        Ok(record.clone())
    }

    async fn list_account_records(&self) -> Result<Vec<AccountRecord>> {
        let rows = sqlx::query(
            "SELECT account_id, tenant_id, organization_id, user_id, account_type,
                    currency_code, credit_unit_code, status, allow_overdraft, overdraft_limit,
                    created_at_ms, updated_at_ms
             FROM ai_account
             ORDER BY created_at_ms DESC, account_id",
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(decode_account_record_row).collect()
    }

    async fn find_account_record(&self, account_id: u64) -> Result<Option<AccountRecord>> {
        let row = sqlx::query(
            "SELECT account_id, tenant_id, organization_id, user_id, account_type,
                    currency_code, credit_unit_code, status, allow_overdraft, overdraft_limit,
                    created_at_ms, updated_at_ms
             FROM ai_account
             WHERE account_id = ?",
        )
        .bind(i64::try_from(account_id)?)
        .fetch_optional(&self.pool)
        .await?;
        row.map(decode_account_record_row).transpose()
    }

    async fn find_account_record_by_owner(
        &self,
        tenant_id: u64,
        organization_id: u64,
        user_id: u64,
        account_type: AccountType,
    ) -> Result<Option<AccountRecord>> {
        let row = sqlx::query(
            "SELECT account_id, tenant_id, organization_id, user_id, account_type,
                    currency_code, credit_unit_code, status, allow_overdraft, overdraft_limit,
                    created_at_ms, updated_at_ms
             FROM ai_account
             WHERE tenant_id = ?
               AND organization_id = ?
               AND user_id = ?
               AND account_type = ?",
        )
        .bind(i64::try_from(tenant_id)?)
        .bind(i64::try_from(organization_id)?)
        .bind(i64::try_from(user_id)?)
        .bind(account_type_as_str(account_type))
        .fetch_optional(&self.pool)
        .await?;
        row.map(decode_account_record_row).transpose()
    }

    async fn insert_account_benefit_lot(
        &self,
        record: &AccountBenefitLotRecord,
    ) -> Result<AccountBenefitLotRecord> {
        sqlx::query(
            "INSERT INTO ai_account_benefit_lot (
                lot_id, tenant_id, organization_id, account_id, user_id, benefit_type,
                source_type, source_id, scope_json, original_quantity, remaining_quantity,
                held_quantity, priority, acquired_unit_cost, issued_at_ms, expires_at_ms, status,
                created_at_ms, updated_at_ms
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(lot_id) DO UPDATE SET
                tenant_id = excluded.tenant_id,
                organization_id = excluded.organization_id,
                account_id = excluded.account_id,
                user_id = excluded.user_id,
                benefit_type = excluded.benefit_type,
                source_type = excluded.source_type,
                source_id = excluded.source_id,
                scope_json = excluded.scope_json,
                original_quantity = excluded.original_quantity,
                remaining_quantity = excluded.remaining_quantity,
                held_quantity = excluded.held_quantity,
                priority = excluded.priority,
                acquired_unit_cost = excluded.acquired_unit_cost,
                issued_at_ms = excluded.issued_at_ms,
                expires_at_ms = excluded.expires_at_ms,
                status = excluded.status,
                created_at_ms = excluded.created_at_ms,
                updated_at_ms = excluded.updated_at_ms",
        )
        .bind(i64::try_from(record.lot_id)?)
        .bind(i64::try_from(record.tenant_id)?)
        .bind(i64::try_from(record.organization_id)?)
        .bind(i64::try_from(record.account_id)?)
        .bind(i64::try_from(record.user_id)?)
        .bind(account_benefit_type_as_str(record.benefit_type))
        .bind(account_benefit_source_type_as_str(record.source_type))
        .bind(record.source_id.map(i64::try_from).transpose()?)
        .bind(&record.scope_json)
        .bind(record.original_quantity)
        .bind(record.remaining_quantity)
        .bind(record.held_quantity)
        .bind(record.priority)
        .bind(record.acquired_unit_cost)
        .bind(i64::try_from(record.issued_at_ms)?)
        .bind(record.expires_at_ms.map(i64::try_from).transpose()?)
        .bind(account_benefit_lot_status_as_str(record.status))
        .bind(i64::try_from(record.created_at_ms)?)
        .bind(i64::try_from(record.updated_at_ms)?)
        .execute(&self.pool)
        .await?;
        Ok(record.clone())
    }

    async fn list_account_benefit_lots(&self) -> Result<Vec<AccountBenefitLotRecord>> {
        let rows = sqlx::query(
            "SELECT lot_id, tenant_id, organization_id, account_id, user_id, benefit_type,
                    source_type, source_id, scope_json, original_quantity, remaining_quantity,
                    held_quantity, priority, acquired_unit_cost, issued_at_ms, expires_at_ms,
                    status, created_at_ms, updated_at_ms
             FROM ai_account_benefit_lot
             ORDER BY created_at_ms DESC, lot_id",
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter()
            .map(decode_account_benefit_lot_row)
            .collect()
    }

    async fn list_account_benefit_lots_for_account(
        &self,
        account_id: u64,
        after_lot_id: Option<u64>,
        limit: usize,
    ) -> Result<Vec<AccountBenefitLotRecord>> {
        let rows = match after_lot_id {
            Some(after_lot_id) => {
                sqlx::query(
                    "SELECT lot_id, tenant_id, organization_id, account_id, user_id, benefit_type,
                            source_type, source_id, scope_json, original_quantity, remaining_quantity,
                            held_quantity, priority, acquired_unit_cost, issued_at_ms, expires_at_ms,
                            status, created_at_ms, updated_at_ms
                     FROM ai_account_benefit_lot
                     WHERE account_id = ?
                       AND lot_id > ?
                     ORDER BY lot_id
                     LIMIT ?",
                )
                .bind(i64::try_from(account_id)?)
                .bind(i64::try_from(after_lot_id)?)
                .bind(i64::try_from(limit)?)
                .fetch_all(&self.pool)
                .await?
            }
            None => {
                sqlx::query(
                    "SELECT lot_id, tenant_id, organization_id, account_id, user_id, benefit_type,
                            source_type, source_id, scope_json, original_quantity, remaining_quantity,
                            held_quantity, priority, acquired_unit_cost, issued_at_ms, expires_at_ms,
                            status, created_at_ms, updated_at_ms
                     FROM ai_account_benefit_lot
                     WHERE account_id = ?
                     ORDER BY lot_id
                     LIMIT ?",
                )
                .bind(i64::try_from(account_id)?)
                .bind(i64::try_from(limit)?)
                .fetch_all(&self.pool)
                .await?
            }
        };
        rows.into_iter()
            .map(decode_account_benefit_lot_row)
            .collect()
    }

    async fn insert_account_hold(&self, record: &AccountHoldRecord) -> Result<AccountHoldRecord> {
        sqlx::query(
            "INSERT INTO ai_account_hold (
                hold_id, tenant_id, organization_id, account_id, user_id, request_id,
                hold_status, estimated_quantity, captured_quantity, released_quantity,
                expires_at_ms, created_at_ms, updated_at_ms
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(hold_id) DO UPDATE SET
                tenant_id = excluded.tenant_id,
                organization_id = excluded.organization_id,
                account_id = excluded.account_id,
                user_id = excluded.user_id,
                request_id = excluded.request_id,
                hold_status = excluded.hold_status,
                estimated_quantity = excluded.estimated_quantity,
                captured_quantity = excluded.captured_quantity,
                released_quantity = excluded.released_quantity,
                expires_at_ms = excluded.expires_at_ms,
                created_at_ms = excluded.created_at_ms,
                updated_at_ms = excluded.updated_at_ms",
        )
        .bind(i64::try_from(record.hold_id)?)
        .bind(i64::try_from(record.tenant_id)?)
        .bind(i64::try_from(record.organization_id)?)
        .bind(i64::try_from(record.account_id)?)
        .bind(i64::try_from(record.user_id)?)
        .bind(i64::try_from(record.request_id)?)
        .bind(account_hold_status_as_str(record.status))
        .bind(record.estimated_quantity)
        .bind(record.captured_quantity)
        .bind(record.released_quantity)
        .bind(i64::try_from(record.expires_at_ms)?)
        .bind(i64::try_from(record.created_at_ms)?)
        .bind(i64::try_from(record.updated_at_ms)?)
        .execute(&self.pool)
        .await?;
        Ok(record.clone())
    }

    async fn list_account_holds(&self) -> Result<Vec<AccountHoldRecord>> {
        let rows = sqlx::query(
            "SELECT hold_id, tenant_id, organization_id, account_id, user_id, request_id,
                    hold_status, estimated_quantity, captured_quantity, released_quantity,
                    expires_at_ms, created_at_ms, updated_at_ms
             FROM ai_account_hold
             ORDER BY created_at_ms DESC, hold_id",
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(decode_account_hold_row).collect()
    }

    async fn insert_account_hold_allocation(
        &self,
        record: &AccountHoldAllocationRecord,
    ) -> Result<AccountHoldAllocationRecord> {
        sqlx::query(
            "INSERT INTO ai_account_hold_allocation (
                hold_allocation_id, tenant_id, organization_id, hold_id, lot_id,
                allocated_quantity, captured_quantity, released_quantity,
                created_at_ms, updated_at_ms
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(hold_allocation_id) DO UPDATE SET
                tenant_id = excluded.tenant_id,
                organization_id = excluded.organization_id,
                hold_id = excluded.hold_id,
                lot_id = excluded.lot_id,
                allocated_quantity = excluded.allocated_quantity,
                captured_quantity = excluded.captured_quantity,
                released_quantity = excluded.released_quantity,
                created_at_ms = excluded.created_at_ms,
                updated_at_ms = excluded.updated_at_ms",
        )
        .bind(i64::try_from(record.hold_allocation_id)?)
        .bind(i64::try_from(record.tenant_id)?)
        .bind(i64::try_from(record.organization_id)?)
        .bind(i64::try_from(record.hold_id)?)
        .bind(i64::try_from(record.lot_id)?)
        .bind(record.allocated_quantity)
        .bind(record.captured_quantity)
        .bind(record.released_quantity)
        .bind(i64::try_from(record.created_at_ms)?)
        .bind(i64::try_from(record.updated_at_ms)?)
        .execute(&self.pool)
        .await?;
        Ok(record.clone())
    }

    async fn list_account_hold_allocations(&self) -> Result<Vec<AccountHoldAllocationRecord>> {
        let rows = sqlx::query(
            "SELECT hold_allocation_id, tenant_id, organization_id, hold_id, lot_id,
                    allocated_quantity, captured_quantity, released_quantity,
                    created_at_ms, updated_at_ms
             FROM ai_account_hold_allocation
             ORDER BY created_at_ms DESC, hold_allocation_id",
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter()
            .map(decode_account_hold_allocation_row)
            .collect()
    }

    async fn insert_account_ledger_entry_record(
        &self,
        record: &AccountLedgerEntryRecord,
    ) -> Result<AccountLedgerEntryRecord> {
        sqlx::query(
            "INSERT INTO ai_account_ledger_entry (
                ledger_entry_id, tenant_id, organization_id, account_id, user_id,
                request_id, hold_id, entry_type, benefit_type, quantity, amount, created_at_ms
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(ledger_entry_id) DO UPDATE SET
                tenant_id = excluded.tenant_id,
                organization_id = excluded.organization_id,
                account_id = excluded.account_id,
                user_id = excluded.user_id,
                request_id = excluded.request_id,
                hold_id = excluded.hold_id,
                entry_type = excluded.entry_type,
                benefit_type = excluded.benefit_type,
                quantity = excluded.quantity,
                amount = excluded.amount,
                created_at_ms = excluded.created_at_ms",
        )
        .bind(i64::try_from(record.ledger_entry_id)?)
        .bind(i64::try_from(record.tenant_id)?)
        .bind(i64::try_from(record.organization_id)?)
        .bind(i64::try_from(record.account_id)?)
        .bind(i64::try_from(record.user_id)?)
        .bind(record.request_id.map(i64::try_from).transpose()?)
        .bind(record.hold_id.map(i64::try_from).transpose()?)
        .bind(account_ledger_entry_type_as_str(record.entry_type))
        .bind(&record.benefit_type)
        .bind(record.quantity)
        .bind(record.amount)
        .bind(i64::try_from(record.created_at_ms)?)
        .execute(&self.pool)
        .await?;
        Ok(record.clone())
    }

    async fn list_account_ledger_entry_records(&self) -> Result<Vec<AccountLedgerEntryRecord>> {
        let rows = sqlx::query(
            "SELECT ledger_entry_id, tenant_id, organization_id, account_id, user_id,
                    request_id, hold_id, entry_type, benefit_type, quantity, amount, created_at_ms
             FROM ai_account_ledger_entry
             ORDER BY created_at_ms DESC, ledger_entry_id",
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter()
            .map(decode_account_ledger_entry_row)
            .collect()
    }

    async fn insert_account_ledger_allocation(
        &self,
        record: &AccountLedgerAllocationRecord,
    ) -> Result<AccountLedgerAllocationRecord> {
        sqlx::query(
            "INSERT INTO ai_account_ledger_allocation (
                ledger_allocation_id, tenant_id, organization_id, ledger_entry_id, lot_id,
                quantity_delta, created_at_ms
            ) VALUES (?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(ledger_allocation_id) DO UPDATE SET
                tenant_id = excluded.tenant_id,
                organization_id = excluded.organization_id,
                ledger_entry_id = excluded.ledger_entry_id,
                lot_id = excluded.lot_id,
                quantity_delta = excluded.quantity_delta,
                created_at_ms = excluded.created_at_ms",
        )
        .bind(i64::try_from(record.ledger_allocation_id)?)
        .bind(i64::try_from(record.tenant_id)?)
        .bind(i64::try_from(record.organization_id)?)
        .bind(i64::try_from(record.ledger_entry_id)?)
        .bind(i64::try_from(record.lot_id)?)
        .bind(record.quantity_delta)
        .bind(i64::try_from(record.created_at_ms)?)
        .execute(&self.pool)
        .await?;
        Ok(record.clone())
    }

    async fn list_account_ledger_allocations(&self) -> Result<Vec<AccountLedgerAllocationRecord>> {
        let rows = sqlx::query(
            "SELECT ledger_allocation_id, tenant_id, organization_id, ledger_entry_id, lot_id,
                    quantity_delta, created_at_ms
             FROM ai_account_ledger_allocation
             ORDER BY created_at_ms DESC, ledger_allocation_id",
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter()
            .map(decode_account_ledger_allocation_row)
            .collect()
    }

    async fn insert_account_commerce_reconciliation_state(
        &self,
        record: &AccountCommerceReconciliationStateRecord,
    ) -> Result<AccountCommerceReconciliationStateRecord> {
        sqlx::query(
            "INSERT INTO ai_account_commerce_reconciliation_state (
                tenant_id, organization_id, account_id, project_id, last_order_updated_at_ms,
                last_order_created_at_ms, last_order_id, updated_at_ms
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(account_id, project_id) DO UPDATE SET
                tenant_id = excluded.tenant_id,
                organization_id = excluded.organization_id,
                last_order_updated_at_ms = excluded.last_order_updated_at_ms,
                last_order_created_at_ms = excluded.last_order_created_at_ms,
                last_order_id = excluded.last_order_id,
                updated_at_ms = excluded.updated_at_ms",
        )
        .bind(i64::try_from(record.tenant_id)?)
        .bind(i64::try_from(record.organization_id)?)
        .bind(i64::try_from(record.account_id)?)
        .bind(&record.project_id)
        .bind(i64::try_from(record.last_order_updated_at_ms)?)
        .bind(i64::try_from(record.last_order_created_at_ms)?)
        .bind(&record.last_order_id)
        .bind(i64::try_from(record.updated_at_ms)?)
        .execute(&self.pool)
        .await?;
        Ok(record.clone())
    }

    async fn find_account_commerce_reconciliation_state(
        &self,
        account_id: u64,
        project_id: &str,
    ) -> Result<Option<AccountCommerceReconciliationStateRecord>> {
        let row = sqlx::query(
            "SELECT tenant_id, organization_id, account_id, project_id,
                    last_order_updated_at_ms, last_order_created_at_ms, last_order_id,
                    updated_at_ms
             FROM ai_account_commerce_reconciliation_state
             WHERE account_id = ?
               AND project_id = ?",
        )
        .bind(i64::try_from(account_id)?)
        .bind(project_id)
        .fetch_optional(&self.pool)
        .await?;
        row.map(decode_account_commerce_reconciliation_state_row)
            .transpose()
    }

    async fn insert_request_meter_fact(
        &self,
        record: &RequestMeterFactRecord,
    ) -> Result<RequestMeterFactRecord> {
        sqlx::query(
            "INSERT INTO ai_request_meter_fact (
                request_id, tenant_id, organization_id, user_id, account_id, api_key_id,
                api_key_hash, auth_type, jwt_subject, platform, owner, request_trace_id,
                gateway_request_ref, upstream_request_ref, protocol_family, capability_code,
                channel_code, model_code, provider_code, request_status, usage_capture_status,
                cost_pricing_plan_id, retail_pricing_plan_id, estimated_credit_hold,
                actual_credit_charge, actual_provider_cost, started_at_ms, finished_at_ms,
                created_at_ms, updated_at_ms
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(request_id) DO UPDATE SET
                tenant_id = excluded.tenant_id,
                organization_id = excluded.organization_id,
                user_id = excluded.user_id,
                account_id = excluded.account_id,
                api_key_id = excluded.api_key_id,
                api_key_hash = excluded.api_key_hash,
                auth_type = excluded.auth_type,
                jwt_subject = excluded.jwt_subject,
                platform = excluded.platform,
                owner = excluded.owner,
                request_trace_id = excluded.request_trace_id,
                gateway_request_ref = excluded.gateway_request_ref,
                upstream_request_ref = excluded.upstream_request_ref,
                protocol_family = excluded.protocol_family,
                capability_code = excluded.capability_code,
                channel_code = excluded.channel_code,
                model_code = excluded.model_code,
                provider_code = excluded.provider_code,
                request_status = excluded.request_status,
                usage_capture_status = excluded.usage_capture_status,
                cost_pricing_plan_id = excluded.cost_pricing_plan_id,
                retail_pricing_plan_id = excluded.retail_pricing_plan_id,
                estimated_credit_hold = excluded.estimated_credit_hold,
                actual_credit_charge = excluded.actual_credit_charge,
                actual_provider_cost = excluded.actual_provider_cost,
                started_at_ms = excluded.started_at_ms,
                finished_at_ms = excluded.finished_at_ms,
                created_at_ms = excluded.created_at_ms,
                updated_at_ms = excluded.updated_at_ms",
        )
        .bind(i64::try_from(record.request_id)?)
        .bind(i64::try_from(record.tenant_id)?)
        .bind(i64::try_from(record.organization_id)?)
        .bind(i64::try_from(record.user_id)?)
        .bind(i64::try_from(record.account_id)?)
        .bind(record.api_key_id.map(i64::try_from).transpose()?)
        .bind(&record.api_key_hash)
        .bind(&record.auth_type)
        .bind(&record.jwt_subject)
        .bind(&record.platform)
        .bind(&record.owner)
        .bind(&record.request_trace_id)
        .bind(&record.gateway_request_ref)
        .bind(&record.upstream_request_ref)
        .bind(&record.protocol_family)
        .bind(&record.capability_code)
        .bind(&record.channel_code)
        .bind(&record.model_code)
        .bind(&record.provider_code)
        .bind(request_status_as_str(record.request_status))
        .bind(usage_capture_status_as_str(record.usage_capture_status))
        .bind(record.cost_pricing_plan_id.map(i64::try_from).transpose()?)
        .bind(record.retail_pricing_plan_id.map(i64::try_from).transpose()?)
        .bind(record.estimated_credit_hold)
        .bind(record.actual_credit_charge)
        .bind(record.actual_provider_cost)
        .bind(i64::try_from(record.started_at_ms)?)
        .bind(record.finished_at_ms.map(i64::try_from).transpose()?)
        .bind(i64::try_from(record.created_at_ms)?)
        .bind(i64::try_from(record.updated_at_ms)?)
        .execute(&self.pool)
        .await?;
        Ok(record.clone())
    }

    async fn list_request_meter_facts(&self) -> Result<Vec<RequestMeterFactRecord>> {
        let rows = sqlx::query(
            "SELECT request_id, tenant_id, organization_id, user_id, account_id, api_key_id,
                    api_key_hash, auth_type, jwt_subject, platform, owner, request_trace_id,
                    gateway_request_ref, upstream_request_ref, protocol_family, capability_code,
                    channel_code, model_code, provider_code, request_status, usage_capture_status,
                    cost_pricing_plan_id, retail_pricing_plan_id, estimated_credit_hold,
                    actual_credit_charge, actual_provider_cost, started_at_ms, finished_at_ms,
                    created_at_ms, updated_at_ms
             FROM ai_request_meter_fact
             ORDER BY created_at_ms DESC, request_id",
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter()
            .map(decode_request_meter_fact_row)
            .collect()
    }

    async fn insert_request_meter_metric(
        &self,
        record: &RequestMeterMetricRecord,
    ) -> Result<RequestMeterMetricRecord> {
        sqlx::query(
            "INSERT INTO ai_request_meter_metric (
                request_metric_id, tenant_id, organization_id, request_id, metric_code, quantity,
                provider_field, source_kind, capture_stage, is_billable, captured_at_ms
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(request_metric_id) DO UPDATE SET
                tenant_id = excluded.tenant_id,
                organization_id = excluded.organization_id,
                request_id = excluded.request_id,
                metric_code = excluded.metric_code,
                quantity = excluded.quantity,
                provider_field = excluded.provider_field,
                source_kind = excluded.source_kind,
                capture_stage = excluded.capture_stage,
                is_billable = excluded.is_billable,
                captured_at_ms = excluded.captured_at_ms",
        )
        .bind(i64::try_from(record.request_metric_id)?)
        .bind(i64::try_from(record.tenant_id)?)
        .bind(i64::try_from(record.organization_id)?)
        .bind(i64::try_from(record.request_id)?)
        .bind(&record.metric_code)
        .bind(record.quantity)
        .bind(&record.provider_field)
        .bind(&record.source_kind)
        .bind(&record.capture_stage)
        .bind(if record.is_billable { 1_i64 } else { 0_i64 })
        .bind(i64::try_from(record.captured_at_ms)?)
        .execute(&self.pool)
        .await?;
        Ok(record.clone())
    }

    async fn list_request_meter_metrics(&self) -> Result<Vec<RequestMeterMetricRecord>> {
        let rows = sqlx::query(
            "SELECT request_metric_id, tenant_id, organization_id, request_id, metric_code,
                    quantity, provider_field, source_kind, capture_stage, is_billable, captured_at_ms
             FROM ai_request_meter_metric
             ORDER BY captured_at_ms DESC, request_metric_id",
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter()
            .map(decode_request_meter_metric_row)
            .collect()
    }

    async fn insert_pricing_plan_record(
        &self,
        record: &PricingPlanRecord,
    ) -> Result<PricingPlanRecord> {
        sqlx::query(
            "INSERT INTO ai_pricing_plan (
                pricing_plan_id, tenant_id, organization_id, plan_code, plan_version,
                display_name, currency_code, credit_unit_code, status, ownership_scope,
                effective_from_ms, effective_to_ms, created_at_ms, updated_at_ms
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(pricing_plan_id) DO UPDATE SET
                tenant_id = excluded.tenant_id,
                organization_id = excluded.organization_id,
                plan_code = excluded.plan_code,
                plan_version = excluded.plan_version,
                display_name = excluded.display_name,
                currency_code = excluded.currency_code,
                credit_unit_code = excluded.credit_unit_code,
                status = excluded.status,
                ownership_scope = excluded.ownership_scope,
                effective_from_ms = excluded.effective_from_ms,
                effective_to_ms = excluded.effective_to_ms,
                created_at_ms = excluded.created_at_ms,
                updated_at_ms = excluded.updated_at_ms",
        )
        .bind(i64::try_from(record.pricing_plan_id)?)
        .bind(i64::try_from(record.tenant_id)?)
        .bind(i64::try_from(record.organization_id)?)
        .bind(&record.plan_code)
        .bind(i64::try_from(record.plan_version)?)
        .bind(&record.display_name)
        .bind(&record.currency_code)
        .bind(&record.credit_unit_code)
        .bind(&record.status)
        .bind(pricing_plan_ownership_scope_as_str(record.ownership_scope))
        .bind(i64::try_from(record.effective_from_ms)?)
        .bind(record.effective_to_ms.map(i64::try_from).transpose()?)
        .bind(i64::try_from(record.created_at_ms)?)
        .bind(i64::try_from(record.updated_at_ms)?)
        .execute(&self.pool)
        .await?;
        Ok(record.clone())
    }

    async fn list_pricing_plan_records(&self) -> Result<Vec<PricingPlanRecord>> {
        let rows = sqlx::query(
            "SELECT pricing_plan_id, tenant_id, organization_id, plan_code, plan_version,
                    display_name, currency_code, credit_unit_code, status, ownership_scope,
                    effective_from_ms, effective_to_ms, created_at_ms, updated_at_ms
             FROM ai_pricing_plan
             ORDER BY updated_at_ms DESC, pricing_plan_id",
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(decode_pricing_plan_row).collect()
    }

    async fn insert_pricing_rate_record(
        &self,
        record: &PricingRateRecord,
    ) -> Result<PricingRateRecord> {
        sqlx::query(
            "INSERT INTO ai_pricing_rate (
                pricing_rate_id, tenant_id, organization_id, pricing_plan_id, metric_code,
                capability_code, model_code, provider_code, charge_unit, pricing_method,
                quantity_step, unit_price, display_price_unit, minimum_billable_quantity,
                minimum_charge, rounding_increment, rounding_mode, included_quantity,
                priority, notes, status, created_at_ms, updated_at_ms
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(pricing_rate_id) DO UPDATE SET
                tenant_id = excluded.tenant_id,
                organization_id = excluded.organization_id,
                pricing_plan_id = excluded.pricing_plan_id,
                metric_code = excluded.metric_code,
                capability_code = excluded.capability_code,
                model_code = excluded.model_code,
                provider_code = excluded.provider_code,
                charge_unit = excluded.charge_unit,
                pricing_method = excluded.pricing_method,
                quantity_step = excluded.quantity_step,
                unit_price = excluded.unit_price,
                display_price_unit = excluded.display_price_unit,
                minimum_billable_quantity = excluded.minimum_billable_quantity,
                minimum_charge = excluded.minimum_charge,
                rounding_increment = excluded.rounding_increment,
                rounding_mode = excluded.rounding_mode,
                included_quantity = excluded.included_quantity,
                priority = excluded.priority,
                notes = excluded.notes,
                status = excluded.status,
                created_at_ms = excluded.created_at_ms,
                updated_at_ms = excluded.updated_at_ms",
        )
        .bind(i64::try_from(record.pricing_rate_id)?)
        .bind(i64::try_from(record.tenant_id)?)
        .bind(i64::try_from(record.organization_id)?)
        .bind(i64::try_from(record.pricing_plan_id)?)
        .bind(&record.metric_code)
        .bind(&record.capability_code)
        .bind(&record.model_code)
        .bind(&record.provider_code)
        .bind(&record.charge_unit)
        .bind(&record.pricing_method)
        .bind(record.quantity_step)
        .bind(record.unit_price)
        .bind(&record.display_price_unit)
        .bind(record.minimum_billable_quantity)
        .bind(record.minimum_charge)
        .bind(record.rounding_increment)
        .bind(&record.rounding_mode)
        .bind(record.included_quantity)
        .bind(i64::try_from(record.priority)?)
        .bind(&record.notes)
        .bind(&record.status)
        .bind(i64::try_from(record.created_at_ms)?)
        .bind(i64::try_from(record.updated_at_ms)?)
        .execute(&self.pool)
        .await?;
        Ok(record.clone())
    }

    async fn list_pricing_rate_records(&self) -> Result<Vec<PricingRateRecord>> {
        let rows = sqlx::query(
            "SELECT pricing_rate_id, tenant_id, organization_id, pricing_plan_id, metric_code,
                    capability_code, model_code, provider_code, charge_unit, pricing_method,
                    quantity_step, unit_price, display_price_unit, minimum_billable_quantity,
                    minimum_charge, rounding_increment, rounding_mode, included_quantity,
                    priority, notes, status, created_at_ms, updated_at_ms
             FROM ai_pricing_rate
             ORDER BY updated_at_ms DESC, priority DESC, pricing_rate_id",
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(decode_pricing_rate_row).collect()
    }

    async fn insert_request_settlement_record(
        &self,
        record: &RequestSettlementRecord,
    ) -> Result<RequestSettlementRecord> {
        sqlx::query(
            "INSERT INTO ai_request_settlement (
                request_settlement_id, tenant_id, organization_id, request_id, account_id, user_id,
                hold_id, settlement_status, estimated_credit_hold, released_credit_amount,
                captured_credit_amount, provider_cost_amount, retail_charge_amount, shortfall_amount,
                refunded_amount, settled_at_ms, created_at_ms, updated_at_ms
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(request_settlement_id) DO UPDATE SET
                tenant_id = excluded.tenant_id,
                organization_id = excluded.organization_id,
                request_id = excluded.request_id,
                account_id = excluded.account_id,
                user_id = excluded.user_id,
                hold_id = excluded.hold_id,
                settlement_status = excluded.settlement_status,
                estimated_credit_hold = excluded.estimated_credit_hold,
                released_credit_amount = excluded.released_credit_amount,
                captured_credit_amount = excluded.captured_credit_amount,
                provider_cost_amount = excluded.provider_cost_amount,
                retail_charge_amount = excluded.retail_charge_amount,
                shortfall_amount = excluded.shortfall_amount,
                refunded_amount = excluded.refunded_amount,
                settled_at_ms = excluded.settled_at_ms,
                created_at_ms = excluded.created_at_ms,
                updated_at_ms = excluded.updated_at_ms",
        )
        .bind(i64::try_from(record.request_settlement_id)?)
        .bind(i64::try_from(record.tenant_id)?)
        .bind(i64::try_from(record.organization_id)?)
        .bind(i64::try_from(record.request_id)?)
        .bind(i64::try_from(record.account_id)?)
        .bind(i64::try_from(record.user_id)?)
        .bind(record.hold_id.map(i64::try_from).transpose()?)
        .bind(request_settlement_status_as_str(record.status))
        .bind(record.estimated_credit_hold)
        .bind(record.released_credit_amount)
        .bind(record.captured_credit_amount)
        .bind(record.provider_cost_amount)
        .bind(record.retail_charge_amount)
        .bind(record.shortfall_amount)
        .bind(record.refunded_amount)
        .bind(i64::try_from(record.settled_at_ms)?)
        .bind(i64::try_from(record.created_at_ms)?)
        .bind(i64::try_from(record.updated_at_ms)?)
        .execute(&self.pool)
        .await?;
        Ok(record.clone())
    }

    async fn list_request_settlement_records(&self) -> Result<Vec<RequestSettlementRecord>> {
        let rows = sqlx::query(
            "SELECT request_settlement_id, tenant_id, organization_id, request_id, account_id,
                    user_id, hold_id, settlement_status, estimated_credit_hold,
                    released_credit_amount, captured_credit_amount, provider_cost_amount,
                    retail_charge_amount, shortfall_amount, refunded_amount, settled_at_ms,
                    created_at_ms, updated_at_ms
             FROM ai_request_settlement
             ORDER BY updated_at_ms DESC, request_settlement_id",
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter()
            .map(decode_request_settlement_row)
            .collect()
    }
}

