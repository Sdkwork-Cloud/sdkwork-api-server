use super::*;

pub async fn issue_commerce_order_credits<S>(
    store: &S,
    input: IssueCommerceOrderCreditsInput<'_>,
) -> Result<IssueCommerceOrderCreditsResult>
where
    S: AccountKernelStore + AccountKernelTransactionExecutor + ?Sized,
{
    let account_id = input.account_id;
    let order_id = input.order_id.trim().to_owned();
    let project_id = input.project_id.trim().to_owned();
    let target_kind = input.target_kind.trim().to_owned();
    let granted_quantity = input.granted_quantity;
    let payable_amount = input.payable_amount;
    let issued_at_ms = input.issued_at_ms;
    ensure!(!order_id.is_empty(), "order_id is required");
    ensure!(!project_id.is_empty(), "project_id is required");
    ensure!(!target_kind.is_empty(), "target_kind is required");
    ensure!(granted_quantity > 0.0, "granted_quantity must be positive");
    ensure!(payable_amount >= 0.0, "payable_amount must not be negative");

    let order_source_id = commerce_order_source_id(&order_id);
    let lot_id = commerce_order_lot_id(&order_id);
    let ledger_entry_id = commerce_order_issue_ledger_entry_id(&order_id);
    let ledger_allocation_id = commerce_order_issue_ledger_allocation_id(&order_id);

    store
        .with_account_kernel_transaction(|tx| {
            Box::pin(async move {
                let account = tx
                    .find_account_record(account_id)
                    .await?
                    .ok_or_else(|| anyhow::anyhow!("account {} does not exist", account_id))?;

                if let Some(existing_ledger_entry) =
                    tx.find_account_ledger_entry_record(ledger_entry_id).await?
                {
                    ensure!(
                        existing_ledger_entry.entry_type == AccountLedgerEntryType::GrantIssue,
                        "ledger entry {} is not a grant issue entry",
                        ledger_entry_id
                    );
                    ensure!(
                        existing_ledger_entry.account_id == account_id,
                        "ledger entry {} belongs to another account",
                        ledger_entry_id
                    );
                    ensure_quantity_matches(
                        existing_ledger_entry.quantity,
                        granted_quantity,
                        format!("commerce order {} granted_quantity", order_id),
                    )?;
                    ensure_quantity_matches(
                        existing_ledger_entry.amount,
                        payable_amount,
                        format!("commerce order {} payable_amount", order_id),
                    )?;

                    let lot = tx.find_account_benefit_lot(lot_id).await?.ok_or_else(|| {
                        anyhow::anyhow!(
                            "commerce order {} lot {lot_id} is missing for replay",
                            order_id
                        )
                    })?;
                    validate_commerce_order_credit_lot(
                        &lot,
                        account_id,
                        order_source_id,
                        &order_id,
                    )?;

                    let (ledger_allocations, _) =
                        load_account_ledger_allocations_and_lots(tx, ledger_entry_id).await?;
                    return Ok(IssueCommerceOrderCreditsResult {
                        idempotent_replay: true,
                        lot,
                        ledger_entry: existing_ledger_entry,
                        ledger_allocations,
                    });
                }

                if tx.find_account_benefit_lot(lot_id).await?.is_some() {
                    anyhow::bail!(
                        "commerce order {} lot {lot_id} exists without issue ledger entry",
                        order_id
                    );
                }

                let lot = AccountBenefitLotRecord::new(
                    lot_id,
                    account.tenant_id,
                    account.organization_id,
                    account.account_id,
                    account.user_id,
                    AccountBenefitType::CashCredit,
                )
                .with_source_type(AccountBenefitSourceType::Order)
                .with_source_id(Some(order_source_id))
                .with_scope_json(Some(build_commerce_order_credit_scope_json(
                    &order_id,
                    &project_id,
                    &target_kind,
                )?))
                .with_original_quantity(granted_quantity)
                .with_remaining_quantity(granted_quantity)
                .with_held_quantity(0.0)
                .with_acquired_unit_cost(Some(payable_amount / granted_quantity))
                .with_issued_at_ms(issued_at_ms)
                .with_status(AccountBenefitLotStatus::Active)
                .with_created_at_ms(issued_at_ms)
                .with_updated_at_ms(issued_at_ms);
                tx.upsert_account_benefit_lot(&lot).await?;

                let ledger_entry = AccountLedgerEntryRecord::new(
                    ledger_entry_id,
                    account.tenant_id,
                    account.organization_id,
                    account.account_id,
                    account.user_id,
                    AccountLedgerEntryType::GrantIssue,
                )
                .with_benefit_type(Some("cash_credit".to_owned()))
                .with_quantity(granted_quantity)
                .with_amount(payable_amount)
                .with_created_at_ms(issued_at_ms);
                let ledger_allocations = write_account_ledger_entry(
                    tx,
                    &ledger_entry,
                    &[(ledger_allocation_id, lot.lot_id, granted_quantity)],
                    issued_at_ms,
                )
                .await?;

                Ok(IssueCommerceOrderCreditsResult {
                    idempotent_replay: false,
                    lot,
                    ledger_entry,
                    ledger_allocations,
                })
            })
        })
        .await
}

pub async fn refund_commerce_order_credits<S>(
    store: &S,
    input: RefundCommerceOrderCreditsInput<'_>,
) -> Result<RefundCommerceOrderCreditsResult>
where
    S: AccountKernelStore + AccountKernelTransactionExecutor + ?Sized,
{
    let account_id = input.account_id;
    let order_id = input.order_id.trim().to_owned();
    let refunded_quantity = input.refunded_quantity;
    let refunded_amount = input.refunded_amount;
    let refunded_at_ms = input.refunded_at_ms;
    ensure!(!order_id.is_empty(), "order_id is required");
    ensure!(
        refunded_quantity > 0.0,
        "refunded_quantity must be positive"
    );
    ensure!(
        refunded_amount >= 0.0,
        "refunded_amount must not be negative"
    );

    let order_source_id = commerce_order_source_id(&order_id);
    let lot_id = commerce_order_lot_id(&order_id);
    let issue_ledger_entry_id = commerce_order_issue_ledger_entry_id(&order_id);
    let refund_ledger_entry_id = commerce_order_refund_ledger_entry_id(&order_id);
    let refund_ledger_allocation_id = commerce_order_refund_ledger_allocation_id(&order_id);

    store
        .with_account_kernel_transaction(|tx| {
            Box::pin(async move {
                let account = tx
                    .find_account_record(account_id)
                    .await?
                    .ok_or_else(|| anyhow::anyhow!("account {} does not exist", account_id))?;

                if let Some(existing_ledger_entry) = tx
                    .find_account_ledger_entry_record(refund_ledger_entry_id)
                    .await?
                {
                    ensure!(
                        existing_ledger_entry.entry_type == AccountLedgerEntryType::Refund,
                        "ledger entry {} is not a refund entry",
                        refund_ledger_entry_id
                    );
                    ensure!(
                        existing_ledger_entry.account_id == account_id,
                        "ledger entry {} belongs to another account",
                        refund_ledger_entry_id
                    );
                    ensure_quantity_matches(
                        existing_ledger_entry.quantity,
                        refunded_quantity,
                        format!("commerce order {} refunded_quantity", order_id),
                    )?;
                    ensure_quantity_matches(
                        existing_ledger_entry.amount,
                        refunded_amount,
                        format!("commerce order {} refunded_amount", order_id),
                    )?;

                    let lot = tx.find_account_benefit_lot(lot_id).await?.ok_or_else(|| {
                        anyhow::anyhow!(
                            "commerce order {} lot {lot_id} is missing for refund replay",
                            order_id
                        )
                    })?;
                    validate_commerce_order_credit_lot(
                        &lot,
                        account_id,
                        order_source_id,
                        &order_id,
                    )?;
                    ensure!(
                        lot.status == AccountBenefitLotStatus::Disabled,
                        "commerce order {} refund replay found lot {} in status {:?}",
                        order_id,
                        lot.lot_id,
                        lot.status
                    );

                    let (ledger_allocations, _) =
                        load_account_ledger_allocations_and_lots(tx, refund_ledger_entry_id)
                            .await?;
                    return Ok(RefundCommerceOrderCreditsResult {
                        idempotent_replay: true,
                        lot,
                        ledger_entry: existing_ledger_entry,
                        ledger_allocations,
                    });
                }

                let issue_ledger_entry = tx
                    .find_account_ledger_entry_record(issue_ledger_entry_id)
                    .await?
                    .ok_or_else(|| {
                        anyhow::anyhow!(
                            "commerce order {} credits have not been issued yet",
                            order_id
                        )
                    })?;
                ensure!(
                    issue_ledger_entry.entry_type == AccountLedgerEntryType::GrantIssue,
                    "commerce order {} issue ledger entry is not a grant issue entry",
                    order_id
                );
                ensure!(
                    issue_ledger_entry.account_id == account_id,
                    "commerce order {} issue ledger entry belongs to another account",
                    order_id
                );

                let lot = tx.find_account_benefit_lot(lot_id).await?.ok_or_else(|| {
                    anyhow::anyhow!("commerce order {} issued lot does not exist", order_id)
                })?;
                validate_commerce_order_credit_lot(&lot, account_id, order_source_id, &order_id)?;
                ensure!(
                    lot.held_quantity <= ACCOUNTING_EPSILON,
                    "commerce order {} credits cannot be refunded while units are held",
                    order_id
                );
                ensure_quantity_matches(
                    lot.original_quantity,
                    refunded_quantity,
                    format!("commerce order {} original_quantity", order_id),
                )?;
                ensure_quantity_matches(
                    lot.remaining_quantity,
                    lot.original_quantity,
                    format!("commerce order {} remaining_quantity", order_id),
                )?;

                let updated_lot = lot
                    .clone()
                    .with_original_quantity(0.0)
                    .with_remaining_quantity(0.0)
                    .with_held_quantity(0.0)
                    .with_status(AccountBenefitLotStatus::Disabled)
                    .with_updated_at_ms(refunded_at_ms);
                tx.upsert_account_benefit_lot(&updated_lot).await?;

                let refund_ledger_entry = AccountLedgerEntryRecord::new(
                    refund_ledger_entry_id,
                    account.tenant_id,
                    account.organization_id,
                    account.account_id,
                    account.user_id,
                    AccountLedgerEntryType::Refund,
                )
                .with_benefit_type(Some("cash_credit".to_owned()))
                .with_quantity(refunded_quantity)
                .with_amount(refunded_amount)
                .with_created_at_ms(refunded_at_ms);
                let ledger_allocations = write_account_ledger_entry(
                    tx,
                    &refund_ledger_entry,
                    &[(
                        refund_ledger_allocation_id,
                        updated_lot.lot_id,
                        -refunded_quantity,
                    )],
                    refunded_at_ms,
                )
                .await?;

                Ok(RefundCommerceOrderCreditsResult {
                    idempotent_replay: false,
                    lot: updated_lot,
                    ledger_entry: refund_ledger_entry,
                    ledger_allocations,
                })
            })
        })
        .await
}

