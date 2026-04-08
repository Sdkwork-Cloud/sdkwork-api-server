use super::*;

#[async_trait]
pub trait GatewayCommercialBillingKernel: Send + Sync {
    async fn resolve_payable_account_for_gateway_request_context(
        &self,
        context: &IdentityGatewayRequestContext,
    ) -> Result<Option<AccountRecord>>;

    async fn plan_account_hold(
        &self,
        account_id: u64,
        requested_quantity: f64,
        now_ms: u64,
    ) -> Result<AccountHoldPlan>;

    async fn create_account_hold(
        &self,
        input: CreateAccountHoldInput,
    ) -> Result<AccountHoldMutationResult>;

    async fn release_account_hold(
        &self,
        input: ReleaseAccountHoldInput,
    ) -> Result<AccountHoldMutationResult>;

    async fn capture_account_hold(
        &self,
        input: CaptureAccountHoldInput,
    ) -> Result<CaptureAccountHoldResult>;
}

#[async_trait]
impl<T> GatewayCommercialBillingKernel for T
where
    T: AccountKernelStore + AccountKernelTransactionExecutor + Send + Sync + ?Sized,
{
    async fn resolve_payable_account_for_gateway_request_context(
        &self,
        context: &IdentityGatewayRequestContext,
    ) -> Result<Option<AccountRecord>> {
        resolve_payable_account_for_gateway_request_context(self, context).await
    }

    async fn plan_account_hold(
        &self,
        account_id: u64,
        requested_quantity: f64,
        now_ms: u64,
    ) -> Result<AccountHoldPlan> {
        plan_account_hold(self, account_id, requested_quantity, now_ms).await
    }

    async fn create_account_hold(
        &self,
        input: CreateAccountHoldInput,
    ) -> Result<AccountHoldMutationResult> {
        create_account_hold(self, input).await
    }

    async fn release_account_hold(
        &self,
        input: ReleaseAccountHoldInput,
    ) -> Result<AccountHoldMutationResult> {
        release_account_hold(self, input).await
    }

    async fn capture_account_hold(
        &self,
        input: CaptureAccountHoldInput,
    ) -> Result<CaptureAccountHoldResult> {
        capture_account_hold(self, input).await
    }
}

#[async_trait]
pub trait CommercialBillingReadKernel: Send + Sync {
    async fn resolve_payable_account_for_gateway_request_context(
        &self,
        context: &IdentityGatewayRequestContext,
    ) -> Result<Option<AccountRecord>>;

    async fn list_account_records(&self) -> Result<Vec<AccountRecord>>;

    async fn find_account_record(&self, account_id: u64) -> Result<Option<AccountRecord>>;

    async fn summarize_account_balance(
        &self,
        account_id: u64,
        now_ms: u64,
    ) -> Result<AccountBalanceSnapshot>;

    async fn list_account_ledger_history(
        &self,
        account_id: u64,
    ) -> Result<Vec<AccountLedgerHistoryEntry>>;

    async fn list_account_benefit_lots(&self) -> Result<Vec<AccountBenefitLotRecord>>;

    async fn list_account_holds(&self) -> Result<Vec<AccountHoldRecord>>;

    async fn list_request_settlement_records(&self) -> Result<Vec<RequestSettlementRecord>>;

    async fn list_pricing_plan_records(&self) -> Result<Vec<PricingPlanRecord>>;

    async fn list_pricing_rate_records(&self) -> Result<Vec<PricingRateRecord>>;
}

#[async_trait]
pub trait CommercialBillingAdminKernel: CommercialBillingReadKernel {
    async fn insert_pricing_plan_record(
        &self,
        record: &PricingPlanRecord,
    ) -> Result<PricingPlanRecord>;

    async fn insert_pricing_rate_record(
        &self,
        record: &PricingRateRecord,
    ) -> Result<PricingRateRecord>;

    async fn issue_commerce_order_credits(
        &self,
        input: IssueCommerceOrderCreditsInput<'_>,
    ) -> Result<IssueCommerceOrderCreditsResult>;

    async fn refund_commerce_order_credits(
        &self,
        input: RefundCommerceOrderCreditsInput<'_>,
    ) -> Result<RefundCommerceOrderCreditsResult>;

    async fn insert_account_commerce_reconciliation_state(
        &self,
        record: &AccountCommerceReconciliationStateRecord,
    ) -> Result<AccountCommerceReconciliationStateRecord>;

    async fn find_account_commerce_reconciliation_state(
        &self,
        account_id: u64,
        project_id: &str,
    ) -> Result<Option<AccountCommerceReconciliationStateRecord>>;
}

#[async_trait]
impl<T> CommercialBillingReadKernel for T
where
    T: AccountKernelStore + Send + Sync + ?Sized,
{
    async fn resolve_payable_account_for_gateway_request_context(
        &self,
        context: &IdentityGatewayRequestContext,
    ) -> Result<Option<AccountRecord>> {
        resolve_payable_account_for_gateway_request_context(self, context).await
    }

    async fn list_account_records(&self) -> Result<Vec<AccountRecord>> {
        AccountKernelStore::list_account_records(self).await
    }

    async fn find_account_record(&self, account_id: u64) -> Result<Option<AccountRecord>> {
        AccountKernelStore::find_account_record(self, account_id).await
    }

    async fn summarize_account_balance(
        &self,
        account_id: u64,
        now_ms: u64,
    ) -> Result<AccountBalanceSnapshot> {
        summarize_account_balance(self, account_id, now_ms).await
    }

    async fn list_account_ledger_history(
        &self,
        account_id: u64,
    ) -> Result<Vec<AccountLedgerHistoryEntry>> {
        list_account_ledger_history(self, account_id).await
    }

    async fn list_account_benefit_lots(&self) -> Result<Vec<AccountBenefitLotRecord>> {
        AccountKernelStore::list_account_benefit_lots(self).await
    }

    async fn list_account_holds(&self) -> Result<Vec<AccountHoldRecord>> {
        AccountKernelStore::list_account_holds(self).await
    }

    async fn list_request_settlement_records(&self) -> Result<Vec<RequestSettlementRecord>> {
        AccountKernelStore::list_request_settlement_records(self).await
    }

    async fn list_pricing_plan_records(&self) -> Result<Vec<PricingPlanRecord>> {
        AccountKernelStore::list_pricing_plan_records(self).await
    }

    async fn list_pricing_rate_records(&self) -> Result<Vec<PricingRateRecord>> {
        AccountKernelStore::list_pricing_rate_records(self).await
    }
}

#[async_trait]
impl<T> CommercialBillingAdminKernel for T
where
    T: AccountKernelStore + AccountKernelTransactionExecutor + Send + Sync + ?Sized,
{
    async fn insert_pricing_plan_record(
        &self,
        record: &PricingPlanRecord,
    ) -> Result<PricingPlanRecord> {
        AccountKernelStore::insert_pricing_plan_record(self, record).await
    }

    async fn insert_pricing_rate_record(
        &self,
        record: &PricingRateRecord,
    ) -> Result<PricingRateRecord> {
        AccountKernelStore::insert_pricing_rate_record(self, record).await
    }

    async fn issue_commerce_order_credits(
        &self,
        input: IssueCommerceOrderCreditsInput<'_>,
    ) -> Result<IssueCommerceOrderCreditsResult> {
        issue_commerce_order_credits(self, input).await
    }

    async fn refund_commerce_order_credits(
        &self,
        input: RefundCommerceOrderCreditsInput<'_>,
    ) -> Result<RefundCommerceOrderCreditsResult> {
        refund_commerce_order_credits(self, input).await
    }

    async fn insert_account_commerce_reconciliation_state(
        &self,
        record: &AccountCommerceReconciliationStateRecord,
    ) -> Result<AccountCommerceReconciliationStateRecord> {
        AccountKernelStore::insert_account_commerce_reconciliation_state(self, record).await
    }

    async fn find_account_commerce_reconciliation_state(
        &self,
        account_id: u64,
        project_id: &str,
    ) -> Result<Option<AccountCommerceReconciliationStateRecord>> {
        AccountKernelStore::find_account_commerce_reconciliation_state(self, account_id, project_id)
            .await
    }
}

