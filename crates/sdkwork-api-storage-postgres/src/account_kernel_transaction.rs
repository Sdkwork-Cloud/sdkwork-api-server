use super::*;

struct PostgresAccountKernelTx<'a> {
    tx: Transaction<'a, Postgres>,
}

#[async_trait]
impl AccountKernelTransaction for PostgresAccountKernelTx<'_> {
    async fn find_account_record(&mut self, account_id: u64) -> Result<Option<AccountRecord>> {
        let row = sqlx::query(
            "SELECT account_id, tenant_id, organization_id, user_id, account_type,
                    currency_code, credit_unit_code, status, allow_overdraft, overdraft_limit,
                    created_at_ms, updated_at_ms
             FROM ai_account
             WHERE account_id = $1",
        )
        .bind(i64::try_from(account_id)?)
        .fetch_optional(&mut *self.tx)
        .await?;
        row.map(decode_account_record_row).transpose()
    }

    async fn find_account_benefit_lot(
        &mut self,
        lot_id: u64,
    ) -> Result<Option<AccountBenefitLotRecord>> {
        let row = sqlx::query(
            "SELECT lot_id, tenant_id, organization_id, account_id, user_id, benefit_type,
                    source_type, source_id, scope_json, original_quantity, remaining_quantity,
                    held_quantity, priority, acquired_unit_cost, issued_at_ms, expires_at_ms,
                    status, created_at_ms, updated_at_ms
             FROM ai_account_benefit_lot
             WHERE lot_id = $1",
        )
        .bind(i64::try_from(lot_id)?)
        .fetch_optional(&mut *self.tx)
        .await?;
        row.map(decode_account_benefit_lot_row).transpose()
    }

    async fn list_account_benefit_lots_for_account(
        &mut self,
        account_id: u64,
    ) -> Result<Vec<AccountBenefitLotRecord>> {
        let rows = sqlx::query(
            "SELECT lot_id, tenant_id, organization_id, account_id, user_id, benefit_type,
                    source_type, source_id, scope_json, original_quantity, remaining_quantity,
                    held_quantity, priority, acquired_unit_cost, issued_at_ms, expires_at_ms,
                    status, created_at_ms, updated_at_ms
             FROM ai_account_benefit_lot
             WHERE account_id = $1
             ORDER BY created_at_ms DESC, lot_id",
        )
        .bind(i64::try_from(account_id)?)
        .fetch_all(&mut *self.tx)
        .await?;
        rows.into_iter()
            .map(decode_account_benefit_lot_row)
            .collect()
    }

    async fn find_account_hold_by_request_id(
        &mut self,
        request_id: u64,
    ) -> Result<Option<AccountHoldRecord>> {
        let row = sqlx::query(
            "SELECT hold_id, tenant_id, organization_id, account_id, user_id, request_id,
                    hold_status, estimated_quantity, captured_quantity, released_quantity,
                    expires_at_ms, created_at_ms, updated_at_ms
             FROM ai_account_hold
             WHERE request_id = $1
             ORDER BY created_at_ms DESC, hold_id
             LIMIT 1",
        )
        .bind(i64::try_from(request_id)?)
        .fetch_optional(&mut *self.tx)
        .await?;
        row.map(decode_account_hold_row).transpose()
    }

    async fn list_account_hold_allocations_for_hold(
        &mut self,
        hold_id: u64,
    ) -> Result<Vec<AccountHoldAllocationRecord>> {
        let rows = sqlx::query(
            "SELECT hold_allocation_id, tenant_id, organization_id, hold_id, lot_id,
                    allocated_quantity, captured_quantity, released_quantity,
                    created_at_ms, updated_at_ms
             FROM ai_account_hold_allocation
             WHERE hold_id = $1
             ORDER BY created_at_ms DESC, hold_allocation_id",
        )
        .bind(i64::try_from(hold_id)?)
        .fetch_all(&mut *self.tx)
        .await?;
        rows.into_iter()
            .map(decode_account_hold_allocation_row)
            .collect()
    }

    async fn find_request_settlement_by_request_id(
        &mut self,
        request_id: u64,
    ) -> Result<Option<RequestSettlementRecord>> {
        let row = sqlx::query(
            "SELECT request_settlement_id, tenant_id, organization_id, request_id, account_id, user_id,
                    hold_id, settlement_status, estimated_credit_hold, released_credit_amount,
                    captured_credit_amount, provider_cost_amount, retail_charge_amount, shortfall_amount,
                    refunded_amount, settled_at_ms, created_at_ms, updated_at_ms
             FROM ai_request_settlement
             WHERE request_id = $1
             ORDER BY created_at_ms DESC, request_settlement_id
             LIMIT 1",
        )
        .bind(i64::try_from(request_id)?)
        .fetch_optional(&mut *self.tx)
        .await?;
        row.map(decode_request_settlement_row).transpose()
    }

    async fn find_request_settlement_record(
        &mut self,
        request_settlement_id: u64,
    ) -> Result<Option<RequestSettlementRecord>> {
        let row = sqlx::query(
            "SELECT request_settlement_id, tenant_id, organization_id, request_id, account_id, user_id,
                    hold_id, settlement_status, estimated_credit_hold, released_credit_amount,
                    captured_credit_amount, provider_cost_amount, retail_charge_amount, shortfall_amount,
                    refunded_amount, settled_at_ms, created_at_ms, updated_at_ms
             FROM ai_request_settlement
             WHERE request_settlement_id = $1
             LIMIT 1",
        )
        .bind(i64::try_from(request_settlement_id)?)
        .fetch_optional(&mut *self.tx)
        .await?;
        row.map(decode_request_settlement_row).transpose()
    }

    async fn find_account_ledger_entry_record(
        &mut self,
        ledger_entry_id: u64,
    ) -> Result<Option<AccountLedgerEntryRecord>> {
        let row = sqlx::query(
            "SELECT ledger_entry_id, tenant_id, organization_id, account_id, user_id,
                    request_id, hold_id, entry_type, benefit_type, quantity, amount, created_at_ms
             FROM ai_account_ledger_entry
             WHERE ledger_entry_id = $1
             LIMIT 1",
        )
        .bind(i64::try_from(ledger_entry_id)?)
        .fetch_optional(&mut *self.tx)
        .await?;
        row.map(decode_account_ledger_entry_row).transpose()
    }

    async fn list_account_ledger_allocations_for_entry(
        &mut self,
        ledger_entry_id: u64,
    ) -> Result<Vec<AccountLedgerAllocationRecord>> {
        let rows = sqlx::query(
            "SELECT ledger_allocation_id, tenant_id, organization_id, ledger_entry_id, lot_id,
                    quantity_delta, created_at_ms
             FROM ai_account_ledger_allocation
             WHERE ledger_entry_id = $1
             ORDER BY created_at_ms DESC, ledger_allocation_id",
        )
        .bind(i64::try_from(ledger_entry_id)?)
        .fetch_all(&mut *self.tx)
        .await?;
        rows.into_iter()
            .map(decode_account_ledger_allocation_row)
            .collect()
    }

    async fn upsert_account_benefit_lot(
        &mut self,
        record: &AccountBenefitLotRecord,
    ) -> Result<AccountBenefitLotRecord> {
        sqlx::query(
            "INSERT INTO ai_account_benefit_lot (
                lot_id, tenant_id, organization_id, account_id, user_id, benefit_type,
                source_type, source_id, scope_json, original_quantity, remaining_quantity,
                held_quantity, priority, acquired_unit_cost, issued_at_ms, expires_at_ms, status,
                created_at_ms, updated_at_ms
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19)
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
        .execute(&mut *self.tx)
        .await?;
        Ok(record.clone())
    }

    async fn upsert_account_hold(
        &mut self,
        record: &AccountHoldRecord,
    ) -> Result<AccountHoldRecord> {
        sqlx::query(
            "INSERT INTO ai_account_hold (
                hold_id, tenant_id, organization_id, account_id, user_id, request_id,
                hold_status, estimated_quantity, captured_quantity, released_quantity,
                expires_at_ms, created_at_ms, updated_at_ms
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
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
        .execute(&mut *self.tx)
        .await?;
        Ok(record.clone())
    }

    async fn upsert_account_hold_allocation(
        &mut self,
        record: &AccountHoldAllocationRecord,
    ) -> Result<AccountHoldAllocationRecord> {
        sqlx::query(
            "INSERT INTO ai_account_hold_allocation (
                hold_allocation_id, tenant_id, organization_id, hold_id, lot_id,
                allocated_quantity, captured_quantity, released_quantity,
                created_at_ms, updated_at_ms
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
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
        .execute(&mut *self.tx)
        .await?;
        Ok(record.clone())
    }

    async fn upsert_request_settlement_record(
        &mut self,
        record: &RequestSettlementRecord,
    ) -> Result<RequestSettlementRecord> {
        sqlx::query(
            "INSERT INTO ai_request_settlement (
                request_settlement_id, tenant_id, organization_id, request_id, account_id, user_id,
                hold_id, settlement_status, estimated_credit_hold, released_credit_amount,
                captured_credit_amount, provider_cost_amount, retail_charge_amount, shortfall_amount,
                refunded_amount, settled_at_ms, created_at_ms, updated_at_ms
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18)
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
        .execute(&mut *self.tx)
        .await?;
        Ok(record.clone())
    }

    async fn upsert_account_ledger_entry_record(
        &mut self,
        record: &AccountLedgerEntryRecord,
    ) -> Result<AccountLedgerEntryRecord> {
        sqlx::query(
            "INSERT INTO ai_account_ledger_entry (
                ledger_entry_id, tenant_id, organization_id, account_id, user_id,
                request_id, hold_id, entry_type, benefit_type, quantity, amount, created_at_ms
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
             ON CONFLICT (ledger_entry_id) DO UPDATE SET
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
        .execute(&mut *self.tx)
        .await?;
        Ok(record.clone())
    }

    async fn upsert_account_ledger_allocation(
        &mut self,
        record: &AccountLedgerAllocationRecord,
    ) -> Result<AccountLedgerAllocationRecord> {
        sqlx::query(
            "INSERT INTO ai_account_ledger_allocation (
                ledger_allocation_id, tenant_id, organization_id, ledger_entry_id, lot_id,
                quantity_delta, created_at_ms
            ) VALUES ($1, $2, $3, $4, $5, $6, $7)
             ON CONFLICT (ledger_allocation_id) DO UPDATE SET
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
        .execute(&mut *self.tx)
        .await?;
        Ok(record.clone())
    }
}

#[async_trait]
impl AccountKernelTransactionExecutor for PostgresAdminStore {
    fn with_account_kernel_transaction<'a, T, F>(
        &'a self,
        operation: F,
    ) -> sdkwork_api_storage_core::AccountKernelTransactionFuture<'a, T>
    where
        T: Send + 'a,
        F: for<'tx> FnOnce(
                &'tx mut dyn AccountKernelTransaction,
            )
                -> sdkwork_api_storage_core::AccountKernelTransactionFuture<'tx, T>
            + Send
            + 'a,
    {
        Box::pin(async move {
            let mut tx = PostgresAccountKernelTx {
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
