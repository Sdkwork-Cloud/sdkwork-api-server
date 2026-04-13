use super::*;

fn unsupported_payment_kernel_method(dialect: StorageDialect, method: &str) -> anyhow::Error {
    anyhow::anyhow!(
        "storage dialect {} does not implement canonical payment kernel method {} yet",
        dialect.as_str(),
        method
    )
}

#[async_trait]
pub trait PaymentKernelStore: AdminStore {
    async fn insert_payment_order_record(
        &self,
        _record: &PaymentOrderRecord,
    ) -> Result<PaymentOrderRecord> {
        Err(unsupported_payment_kernel_method(
            self.dialect(),
            "insert_payment_order_record",
        ))
    }

    async fn find_payment_order_record(
        &self,
        _payment_order_id: &str,
    ) -> Result<Option<PaymentOrderRecord>> {
        Err(unsupported_payment_kernel_method(
            self.dialect(),
            "find_payment_order_record",
        ))
    }

    async fn list_payment_order_records(&self) -> Result<Vec<PaymentOrderRecord>> {
        Err(unsupported_payment_kernel_method(
            self.dialect(),
            "list_payment_order_records",
        ))
    }

    async fn insert_payment_gateway_account_record(
        &self,
        _record: &PaymentGatewayAccountRecord,
    ) -> Result<PaymentGatewayAccountRecord> {
        Err(unsupported_payment_kernel_method(
            self.dialect(),
            "insert_payment_gateway_account_record",
        ))
    }

    async fn list_payment_gateway_account_records(
        &self,
    ) -> Result<Vec<PaymentGatewayAccountRecord>> {
        Err(unsupported_payment_kernel_method(
            self.dialect(),
            "list_payment_gateway_account_records",
        ))
    }

    async fn insert_payment_channel_policy_record(
        &self,
        _record: &PaymentChannelPolicyRecord,
    ) -> Result<PaymentChannelPolicyRecord> {
        Err(unsupported_payment_kernel_method(
            self.dialect(),
            "insert_payment_channel_policy_record",
        ))
    }

    async fn list_payment_channel_policy_records(&self) -> Result<Vec<PaymentChannelPolicyRecord>> {
        Err(unsupported_payment_kernel_method(
            self.dialect(),
            "list_payment_channel_policy_records",
        ))
    }

    async fn insert_payment_attempt_record(
        &self,
        _record: &PaymentAttemptRecord,
    ) -> Result<PaymentAttemptRecord> {
        Err(unsupported_payment_kernel_method(
            self.dialect(),
            "insert_payment_attempt_record",
        ))
    }

    async fn list_payment_attempt_records_for_order(
        &self,
        _payment_order_id: &str,
    ) -> Result<Vec<PaymentAttemptRecord>> {
        Err(unsupported_payment_kernel_method(
            self.dialect(),
            "list_payment_attempt_records_for_order",
        ))
    }

    async fn insert_payment_session_record(
        &self,
        _record: &PaymentSessionRecord,
    ) -> Result<PaymentSessionRecord> {
        Err(unsupported_payment_kernel_method(
            self.dialect(),
            "insert_payment_session_record",
        ))
    }

    async fn list_payment_session_records_for_attempt(
        &self,
        _payment_attempt_id: &str,
    ) -> Result<Vec<PaymentSessionRecord>> {
        Err(unsupported_payment_kernel_method(
            self.dialect(),
            "list_payment_session_records_for_attempt",
        ))
    }

    async fn find_payment_callback_event_record_by_dedupe_key(
        &self,
        _provider_code: sdkwork_api_domain_payment::PaymentProviderCode,
        _gateway_account_id: &str,
        _dedupe_key: &str,
    ) -> Result<Option<PaymentCallbackEventRecord>> {
        Err(unsupported_payment_kernel_method(
            self.dialect(),
            "find_payment_callback_event_record_by_dedupe_key",
        ))
    }

    async fn insert_payment_callback_event_record(
        &self,
        _record: &PaymentCallbackEventRecord,
    ) -> Result<PaymentCallbackEventRecord> {
        Err(unsupported_payment_kernel_method(
            self.dialect(),
            "insert_payment_callback_event_record",
        ))
    }

    async fn list_payment_callback_event_records(&self) -> Result<Vec<PaymentCallbackEventRecord>> {
        Err(unsupported_payment_kernel_method(
            self.dialect(),
            "list_payment_callback_event_records",
        ))
    }

    async fn insert_payment_transaction_record(
        &self,
        _record: &PaymentTransactionRecord,
    ) -> Result<PaymentTransactionRecord> {
        Err(unsupported_payment_kernel_method(
            self.dialect(),
            "insert_payment_transaction_record",
        ))
    }

    async fn list_payment_transaction_records_for_order(
        &self,
        _payment_order_id: &str,
    ) -> Result<Vec<PaymentTransactionRecord>> {
        Err(unsupported_payment_kernel_method(
            self.dialect(),
            "list_payment_transaction_records_for_order",
        ))
    }

    async fn insert_refund_order_record(
        &self,
        _record: &RefundOrderRecord,
    ) -> Result<RefundOrderRecord> {
        Err(unsupported_payment_kernel_method(
            self.dialect(),
            "insert_refund_order_record",
        ))
    }

    async fn list_refund_order_records_for_payment_order(
        &self,
        _payment_order_id: &str,
    ) -> Result<Vec<RefundOrderRecord>> {
        Err(unsupported_payment_kernel_method(
            self.dialect(),
            "list_refund_order_records_for_payment_order",
        ))
    }

    async fn find_refund_order_record(
        &self,
        _refund_order_id: &str,
    ) -> Result<Option<RefundOrderRecord>> {
        Err(unsupported_payment_kernel_method(
            self.dialect(),
            "find_refund_order_record",
        ))
    }

    async fn apply_refund_order_quota_reversal(
        &self,
        _refund_order_id: &str,
        _project_id: &str,
        _target_kind: &str,
        _target_units: u64,
    ) -> Result<bool> {
        Err(unsupported_payment_kernel_method(
            self.dialect(),
            "apply_refund_order_quota_reversal",
        ))
    }

    async fn insert_finance_journal_entry_record(
        &self,
        _record: &FinanceJournalEntryRecord,
    ) -> Result<FinanceJournalEntryRecord> {
        Err(unsupported_payment_kernel_method(
            self.dialect(),
            "insert_finance_journal_entry_record",
        ))
    }

    async fn list_finance_journal_entry_records(&self) -> Result<Vec<FinanceJournalEntryRecord>> {
        Err(unsupported_payment_kernel_method(
            self.dialect(),
            "list_finance_journal_entry_records",
        ))
    }

    async fn insert_finance_journal_line_record(
        &self,
        _record: &FinanceJournalLineRecord,
    ) -> Result<FinanceJournalLineRecord> {
        Err(unsupported_payment_kernel_method(
            self.dialect(),
            "insert_finance_journal_line_record",
        ))
    }

    async fn list_finance_journal_line_records(
        &self,
        _finance_journal_entry_id: &str,
    ) -> Result<Vec<FinanceJournalLineRecord>> {
        Err(unsupported_payment_kernel_method(
            self.dialect(),
            "list_finance_journal_line_records",
        ))
    }

    async fn insert_reconciliation_match_summary_record(
        &self,
        _record: &ReconciliationMatchSummaryRecord,
    ) -> Result<ReconciliationMatchSummaryRecord> {
        Err(unsupported_payment_kernel_method(
            self.dialect(),
            "insert_reconciliation_match_summary_record",
        ))
    }

    async fn list_reconciliation_match_summary_records(
        &self,
        _reconciliation_batch_id: &str,
    ) -> Result<Vec<ReconciliationMatchSummaryRecord>> {
        Err(unsupported_payment_kernel_method(
            self.dialect(),
            "list_reconciliation_match_summary_records",
        ))
    }

    async fn find_reconciliation_match_summary_record(
        &self,
        _reconciliation_line_id: &str,
    ) -> Result<Option<ReconciliationMatchSummaryRecord>> {
        Err(unsupported_payment_kernel_method(
            self.dialect(),
            "find_reconciliation_match_summary_record",
        ))
    }

    async fn list_all_reconciliation_match_summary_records(
        &self,
    ) -> Result<Vec<ReconciliationMatchSummaryRecord>> {
        Err(unsupported_payment_kernel_method(
            self.dialect(),
            "list_all_reconciliation_match_summary_records",
        ))
    }
}

pub trait CommercialKernelStore: PaymentKernelStore + AccountKernelStore {}

impl<T> CommercialKernelStore for T where T: PaymentKernelStore + AccountKernelStore + ?Sized {}
