use super::*;

pub type AccountKernelTransactionFuture<'a, T> =
    Pin<Box<dyn Future<Output = Result<T>> + Send + 'a>>;

#[async_trait]
pub trait AccountKernelTransaction: Send {
    async fn find_account_record(&mut self, account_id: u64) -> Result<Option<AccountRecord>>;

    async fn find_account_benefit_lot(
        &mut self,
        lot_id: u64,
    ) -> Result<Option<AccountBenefitLotRecord>>;

    async fn list_account_benefit_lots_for_account(
        &mut self,
        account_id: u64,
    ) -> Result<Vec<AccountBenefitLotRecord>>;

    async fn find_account_hold_by_request_id(
        &mut self,
        request_id: u64,
    ) -> Result<Option<AccountHoldRecord>>;

    async fn list_account_hold_allocations_for_hold(
        &mut self,
        hold_id: u64,
    ) -> Result<Vec<AccountHoldAllocationRecord>>;

    async fn find_request_settlement_by_request_id(
        &mut self,
        request_id: u64,
    ) -> Result<Option<RequestSettlementRecord>>;

    async fn find_request_settlement_record(
        &mut self,
        request_settlement_id: u64,
    ) -> Result<Option<RequestSettlementRecord>>;

    async fn find_account_ledger_entry_record(
        &mut self,
        ledger_entry_id: u64,
    ) -> Result<Option<AccountLedgerEntryRecord>>;

    async fn list_account_ledger_allocations_for_entry(
        &mut self,
        ledger_entry_id: u64,
    ) -> Result<Vec<AccountLedgerAllocationRecord>>;

    async fn upsert_account_benefit_lot(
        &mut self,
        record: &AccountBenefitLotRecord,
    ) -> Result<AccountBenefitLotRecord>;

    async fn upsert_account_hold(
        &mut self,
        record: &AccountHoldRecord,
    ) -> Result<AccountHoldRecord>;

    async fn upsert_account_hold_allocation(
        &mut self,
        record: &AccountHoldAllocationRecord,
    ) -> Result<AccountHoldAllocationRecord>;

    async fn upsert_request_settlement_record(
        &mut self,
        record: &RequestSettlementRecord,
    ) -> Result<RequestSettlementRecord>;

    async fn upsert_account_ledger_entry_record(
        &mut self,
        record: &AccountLedgerEntryRecord,
    ) -> Result<AccountLedgerEntryRecord>;

    async fn upsert_account_ledger_allocation(
        &mut self,
        record: &AccountLedgerAllocationRecord,
    ) -> Result<AccountLedgerAllocationRecord>;
}

pub trait AccountKernelTransactionExecutor: AccountKernelStore {
    fn with_account_kernel_transaction<'a, T, F>(
        &'a self,
        operation: F,
    ) -> AccountKernelTransactionFuture<'a, T>
    where
        T: Send + 'a,
        F: for<'tx> FnOnce(
                &'tx mut dyn AccountKernelTransaction,
            ) -> AccountKernelTransactionFuture<'tx, T>
            + Send
            + 'a;
}
