use super::*;

#[async_trait]
pub trait AccountKernelStore: AdminStore {
    async fn insert_account_record(&self, _record: &AccountRecord) -> Result<AccountRecord> {
        Err(unsupported_account_kernel_method(
            self.dialect(),
            "insert_account_record",
        ))
    }

    async fn list_account_records(&self) -> Result<Vec<AccountRecord>> {
        Err(unsupported_account_kernel_method(
            self.dialect(),
            "list_account_records",
        ))
    }

    async fn find_account_record(&self, _account_id: u64) -> Result<Option<AccountRecord>> {
        Err(unsupported_account_kernel_method(
            self.dialect(),
            "find_account_record",
        ))
    }

    async fn find_account_record_by_owner(
        &self,
        _tenant_id: u64,
        _organization_id: u64,
        _user_id: u64,
        _account_type: AccountType,
    ) -> Result<Option<AccountRecord>> {
        Err(unsupported_account_kernel_method(
            self.dialect(),
            "find_account_record_by_owner",
        ))
    }

    async fn insert_account_benefit_lot(
        &self,
        _record: &AccountBenefitLotRecord,
    ) -> Result<AccountBenefitLotRecord> {
        Err(unsupported_account_kernel_method(
            self.dialect(),
            "insert_account_benefit_lot",
        ))
    }

    async fn list_account_benefit_lots(&self) -> Result<Vec<AccountBenefitLotRecord>> {
        Err(unsupported_account_kernel_method(
            self.dialect(),
            "list_account_benefit_lots",
        ))
    }

    async fn apply_refund_order_account_grant_reversal(
        &self,
        _refund_order_id: &str,
        _lot_id: u64,
        _reversal_quantity: f64,
        _updated_at_ms: u64,
        _ledger_entry: &AccountLedgerEntryRecord,
        _ledger_allocation: &AccountLedgerAllocationRecord,
    ) -> Result<bool> {
        Err(unsupported_account_kernel_method(
            self.dialect(),
            "apply_refund_order_account_grant_reversal",
        ))
    }

    async fn list_account_benefit_lots_for_account(
        &self,
        _account_id: u64,
        _after_lot_id: Option<u64>,
        _limit: usize,
    ) -> Result<Vec<AccountBenefitLotRecord>> {
        Err(unsupported_account_kernel_method(
            self.dialect(),
            "list_account_benefit_lots_for_account",
        ))
    }

    async fn insert_account_hold(&self, _record: &AccountHoldRecord) -> Result<AccountHoldRecord> {
        Err(unsupported_account_kernel_method(
            self.dialect(),
            "insert_account_hold",
        ))
    }

    async fn list_account_holds(&self) -> Result<Vec<AccountHoldRecord>> {
        Err(unsupported_account_kernel_method(
            self.dialect(),
            "list_account_holds",
        ))
    }

    async fn insert_account_hold_allocation(
        &self,
        _record: &AccountHoldAllocationRecord,
    ) -> Result<AccountHoldAllocationRecord> {
        Err(unsupported_account_kernel_method(
            self.dialect(),
            "insert_account_hold_allocation",
        ))
    }

    async fn list_account_hold_allocations(&self) -> Result<Vec<AccountHoldAllocationRecord>> {
        Err(unsupported_account_kernel_method(
            self.dialect(),
            "list_account_hold_allocations",
        ))
    }

    async fn insert_account_ledger_entry_record(
        &self,
        _record: &AccountLedgerEntryRecord,
    ) -> Result<AccountLedgerEntryRecord> {
        Err(unsupported_account_kernel_method(
            self.dialect(),
            "insert_account_ledger_entry_record",
        ))
    }

    async fn list_account_ledger_entry_records(&self) -> Result<Vec<AccountLedgerEntryRecord>> {
        Err(unsupported_account_kernel_method(
            self.dialect(),
            "list_account_ledger_entry_records",
        ))
    }

    async fn insert_account_ledger_allocation(
        &self,
        _record: &AccountLedgerAllocationRecord,
    ) -> Result<AccountLedgerAllocationRecord> {
        Err(unsupported_account_kernel_method(
            self.dialect(),
            "insert_account_ledger_allocation",
        ))
    }

    async fn list_account_ledger_allocations(&self) -> Result<Vec<AccountLedgerAllocationRecord>> {
        Err(unsupported_account_kernel_method(
            self.dialect(),
            "list_account_ledger_allocations",
        ))
    }

    async fn insert_account_commerce_reconciliation_state(
        &self,
        _record: &AccountCommerceReconciliationStateRecord,
    ) -> Result<AccountCommerceReconciliationStateRecord> {
        Err(unsupported_account_kernel_method(
            self.dialect(),
            "insert_account_commerce_reconciliation_state",
        ))
    }

    async fn find_account_commerce_reconciliation_state(
        &self,
        _account_id: u64,
        _project_id: &str,
    ) -> Result<Option<AccountCommerceReconciliationStateRecord>> {
        Err(unsupported_account_kernel_method(
            self.dialect(),
            "find_account_commerce_reconciliation_state",
        ))
    }

    async fn insert_request_meter_fact(
        &self,
        _record: &RequestMeterFactRecord,
    ) -> Result<RequestMeterFactRecord> {
        Err(unsupported_account_kernel_method(
            self.dialect(),
            "insert_request_meter_fact",
        ))
    }

    async fn list_request_meter_facts(&self) -> Result<Vec<RequestMeterFactRecord>> {
        Err(unsupported_account_kernel_method(
            self.dialect(),
            "list_request_meter_facts",
        ))
    }

    async fn insert_request_meter_metric(
        &self,
        _record: &RequestMeterMetricRecord,
    ) -> Result<RequestMeterMetricRecord> {
        Err(unsupported_account_kernel_method(
            self.dialect(),
            "insert_request_meter_metric",
        ))
    }

    async fn list_request_meter_metrics(&self) -> Result<Vec<RequestMeterMetricRecord>> {
        Err(unsupported_account_kernel_method(
            self.dialect(),
            "list_request_meter_metrics",
        ))
    }

    async fn insert_pricing_plan_record(
        &self,
        _record: &PricingPlanRecord,
    ) -> Result<PricingPlanRecord> {
        Err(unsupported_account_kernel_method(
            self.dialect(),
            "insert_pricing_plan_record",
        ))
    }

    async fn list_pricing_plan_records(&self) -> Result<Vec<PricingPlanRecord>> {
        Err(unsupported_account_kernel_method(
            self.dialect(),
            "list_pricing_plan_records",
        ))
    }

    async fn insert_pricing_rate_record(
        &self,
        _record: &PricingRateRecord,
    ) -> Result<PricingRateRecord> {
        Err(unsupported_account_kernel_method(
            self.dialect(),
            "insert_pricing_rate_record",
        ))
    }

    async fn list_pricing_rate_records(&self) -> Result<Vec<PricingRateRecord>> {
        Err(unsupported_account_kernel_method(
            self.dialect(),
            "list_pricing_rate_records",
        ))
    }

    async fn insert_request_settlement_record(
        &self,
        _record: &RequestSettlementRecord,
    ) -> Result<RequestSettlementRecord> {
        Err(unsupported_account_kernel_method(
            self.dialect(),
            "insert_request_settlement_record",
        ))
    }

    async fn list_request_settlement_records(&self) -> Result<Vec<RequestSettlementRecord>> {
        Err(unsupported_account_kernel_method(
            self.dialect(),
            "list_request_settlement_records",
        ))
    }
}
