use super::*;

pub async fn create_account_hold<S>(
    store: &S,
    input: CreateAccountHoldInput,
) -> Result<AccountHoldMutationResult>
where
    S: AccountKernelStore + AccountKernelTransactionExecutor + ?Sized,
{
    ensure!(
        input.requested_quantity > 0.0,
        "requested_quantity must be positive"
    );

    store
        .with_account_kernel_transaction(|tx| {
            Box::pin(async move {
                if let Some(existing_hold) =
                    tx.find_account_hold_by_request_id(input.request_id).await?
                {
                    let (allocations, updated_lots) =
                        load_hold_allocations_and_lots(tx, existing_hold.hold_id).await?;
                    return Ok(AccountHoldMutationResult {
                        idempotent_replay: true,
                        hold: existing_hold,
                        allocations,
                        updated_lots,
                    });
                }

                let account = tx
                    .find_account_record(input.account_id)
                    .await?
                    .ok_or_else(|| {
                        anyhow::anyhow!("account {} does not exist", input.account_id)
                    })?;
                let lots = tx
                    .list_account_benefit_lots_for_account(input.account_id)
                    .await?;
                let eligible_lots = eligible_lots_for_hold(&lots, input.now_ms);
                let mut remaining = input.requested_quantity;
                let mut allocations = Vec::new();
                let mut updated_lots = Vec::new();

                for (index, lot) in eligible_lots.into_iter().enumerate() {
                    if remaining <= f64::EPSILON {
                        break;
                    }
                    let quantity = free_quantity(lot).min(remaining);
                    if quantity <= f64::EPSILON {
                        continue;
                    }

                    let updated_lot = lot
                        .clone()
                        .with_held_quantity(lot.held_quantity + quantity)
                        .with_updated_at_ms(input.now_ms);
                    tx.upsert_account_benefit_lot(&updated_lot).await?;
                    updated_lots.push(updated_lot.clone());

                    let allocation = AccountHoldAllocationRecord::new(
                        input.hold_allocation_start_id + index as u64,
                        account.tenant_id,
                        account.organization_id,
                        input.hold_id,
                        lot.lot_id,
                    )
                    .with_allocated_quantity(quantity)
                    .with_created_at_ms(input.now_ms)
                    .with_updated_at_ms(input.now_ms);
                    tx.upsert_account_hold_allocation(&allocation).await?;
                    allocations.push(allocation);

                    remaining -= quantity;
                }

                ensure!(
                    remaining <= f64::EPSILON,
                    "account {} has insufficient available balance for request {}",
                    input.account_id,
                    input.request_id
                );

                let hold = AccountHoldRecord::new(
                    input.hold_id,
                    account.tenant_id,
                    account.organization_id,
                    account.account_id,
                    account.user_id,
                    input.request_id,
                )
                .with_estimated_quantity(input.requested_quantity)
                .with_expires_at_ms(input.expires_at_ms)
                .with_created_at_ms(input.now_ms)
                .with_updated_at_ms(input.now_ms);
                tx.upsert_account_hold(&hold).await?;

                let hold_ledger_entry = AccountLedgerEntryRecord::new(
                    account_ledger_entry_id(hold.hold_id, HOLD_CREATE_LEDGER_SUFFIX),
                    account.tenant_id,
                    account.organization_id,
                    account.account_id,
                    account.user_id,
                    AccountLedgerEntryType::HoldCreate,
                )
                .with_request_id(Some(input.request_id))
                .with_hold_id(Some(hold.hold_id))
                .with_quantity(input.requested_quantity)
                .with_created_at_ms(input.now_ms);
                let hold_ledger_allocations = allocations
                    .iter()
                    .map(|allocation| {
                        (
                            account_ledger_allocation_id(
                                allocation.hold_allocation_id,
                                HOLD_CREATE_LEDGER_SUFFIX,
                            ),
                            allocation.lot_id,
                            -allocation.allocated_quantity,
                        )
                    })
                    .collect::<Vec<_>>();
                write_account_ledger_entry(
                    tx,
                    &hold_ledger_entry,
                    &hold_ledger_allocations,
                    input.now_ms,
                )
                .await?;

                Ok(AccountHoldMutationResult {
                    idempotent_replay: false,
                    hold,
                    allocations,
                    updated_lots,
                })
            })
        })
        .await
}

pub async fn release_account_hold<S>(
    store: &S,
    input: ReleaseAccountHoldInput,
) -> Result<AccountHoldMutationResult>
where
    S: AccountKernelStore + AccountKernelTransactionExecutor + ?Sized,
{
    store
        .with_account_kernel_transaction(|tx| {
            Box::pin(async move {
                let hold = tx
                    .find_account_hold_by_request_id(input.request_id)
                    .await?
                    .ok_or_else(|| {
                        anyhow::anyhow!("hold for request {} does not exist", input.request_id)
                    })?;
                if hold.status == AccountHoldStatus::Released {
                    let (allocations, updated_lots) =
                        load_hold_allocations_and_lots(tx, hold.hold_id).await?;
                    return Ok(AccountHoldMutationResult {
                        idempotent_replay: true,
                        hold,
                        allocations,
                        updated_lots,
                    });
                }

                let existing_allocations = tx
                    .list_account_hold_allocations_for_hold(hold.hold_id)
                    .await?;
                let mut allocations = Vec::with_capacity(existing_allocations.len());
                let mut updated_lots = Vec::with_capacity(existing_allocations.len());
                let mut released_quantity = hold.released_quantity;
                let mut released_now_total = 0.0;
                let mut released_ledger_allocations = Vec::new();

                for allocation in existing_allocations {
                    let releasable = (allocation.allocated_quantity
                        - allocation.captured_quantity
                        - allocation.released_quantity)
                        .max(0.0);
                    let lot = tx
                        .find_account_benefit_lot(allocation.lot_id)
                        .await?
                        .ok_or_else(|| {
                            anyhow::anyhow!("lot {} does not exist", allocation.lot_id)
                        })?;

                    let updated_lot = lot
                        .clone()
                        .with_held_quantity((lot.held_quantity - releasable).max(0.0))
                        .with_updated_at_ms(input.released_at_ms);
                    tx.upsert_account_benefit_lot(&updated_lot).await?;
                    updated_lots.push(updated_lot);

                    let updated_allocation = allocation
                        .clone()
                        .with_released_quantity(allocation.released_quantity + releasable)
                        .with_updated_at_ms(input.released_at_ms);
                    tx.upsert_account_hold_allocation(&updated_allocation)
                        .await?;
                    allocations.push(updated_allocation);

                    released_quantity += releasable;
                    released_now_total += releasable;
                    if releasable > f64::EPSILON {
                        released_ledger_allocations.push((
                            account_ledger_allocation_id(
                                allocation.hold_allocation_id,
                                HOLD_RELEASE_LEDGER_SUFFIX,
                            ),
                            allocation.lot_id,
                            releasable,
                        ));
                    }
                }

                let partially_captured = hold.captured_quantity > 0.0;
                let updated_hold = hold
                    .with_status(if partially_captured {
                        AccountHoldStatus::PartiallyReleased
                    } else {
                        AccountHoldStatus::Released
                    })
                    .with_released_quantity(released_quantity)
                    .with_updated_at_ms(input.released_at_ms);
                tx.upsert_account_hold(&updated_hold).await?;

                if released_now_total > f64::EPSILON {
                    let release_ledger_entry = AccountLedgerEntryRecord::new(
                        account_ledger_entry_id(updated_hold.hold_id, HOLD_RELEASE_LEDGER_SUFFIX),
                        updated_hold.tenant_id,
                        updated_hold.organization_id,
                        updated_hold.account_id,
                        updated_hold.user_id,
                        AccountLedgerEntryType::HoldRelease,
                    )
                    .with_request_id(Some(updated_hold.request_id))
                    .with_hold_id(Some(updated_hold.hold_id))
                    .with_quantity(released_now_total)
                    .with_created_at_ms(input.released_at_ms);
                    write_account_ledger_entry(
                        tx,
                        &release_ledger_entry,
                        &released_ledger_allocations,
                        input.released_at_ms,
                    )
                    .await?;
                }

                Ok(AccountHoldMutationResult {
                    idempotent_replay: false,
                    hold: updated_hold,
                    allocations,
                    updated_lots,
                })
            })
        })
        .await
}

pub async fn capture_account_hold<S>(
    store: &S,
    input: CaptureAccountHoldInput,
) -> Result<CaptureAccountHoldResult>
where
    S: AccountKernelStore + AccountKernelTransactionExecutor + ?Sized,
{
    ensure!(
        input.captured_quantity > 0.0,
        "captured_quantity must be positive"
    );
    ensure!(
        input.provider_cost_amount >= 0.0,
        "provider_cost_amount must not be negative"
    );
    ensure!(
        input.retail_charge_amount >= 0.0,
        "retail_charge_amount must not be negative"
    );

    store
        .with_account_kernel_transaction(|tx| {
            Box::pin(async move {
                let hold = tx
                    .find_account_hold_by_request_id(input.request_id)
                    .await?
                    .ok_or_else(|| {
                        anyhow::anyhow!("hold for request {} does not exist", input.request_id)
                    })?;

                if let Some(existing_settlement) = tx
                    .find_request_settlement_by_request_id(input.request_id)
                    .await?
                {
                    let (allocations, updated_lots) =
                        load_hold_allocations_and_lots(tx, hold.hold_id).await?;
                    return Ok(CaptureAccountHoldResult {
                        idempotent_replay: true,
                        hold,
                        allocations,
                        updated_lots,
                        settlement: existing_settlement,
                    });
                }

                let existing_allocations = tx
                    .list_account_hold_allocations_for_hold(hold.hold_id)
                    .await?;
                let mut allocations = Vec::with_capacity(existing_allocations.len());
                let mut updated_lots = Vec::with_capacity(existing_allocations.len());
                let mut remaining_capture = input.captured_quantity;
                let mut captured_quantity = hold.captured_quantity;
                let mut released_quantity = hold.released_quantity;
                let mut captured_now_total = 0.0;
                let mut released_now_total = 0.0;
                let mut captured_ledger_allocations = Vec::new();
                let mut released_ledger_allocations = Vec::new();

                for allocation in existing_allocations {
                    let available = (allocation.allocated_quantity
                        - allocation.captured_quantity
                        - allocation.released_quantity)
                        .max(0.0);
                    let captured_now = available.min(remaining_capture);
                    remaining_capture -= captured_now;
                    let released_now = available - captured_now;

                    let lot = tx
                        .find_account_benefit_lot(allocation.lot_id)
                        .await?
                        .ok_or_else(|| {
                            anyhow::anyhow!("lot {} does not exist", allocation.lot_id)
                        })?;
                    let updated_lot = lot
                        .clone()
                        .with_remaining_quantity((lot.remaining_quantity - captured_now).max(0.0))
                        .with_held_quantity((lot.held_quantity - available).max(0.0))
                        .with_updated_at_ms(input.settled_at_ms);
                    tx.upsert_account_benefit_lot(&updated_lot).await?;
                    updated_lots.push(updated_lot);

                    let updated_allocation = allocation
                        .clone()
                        .with_captured_quantity(allocation.captured_quantity + captured_now)
                        .with_released_quantity(allocation.released_quantity + released_now)
                        .with_updated_at_ms(input.settled_at_ms);
                    tx.upsert_account_hold_allocation(&updated_allocation)
                        .await?;
                    allocations.push(updated_allocation);

                    captured_quantity += captured_now;
                    released_quantity += released_now;
                    captured_now_total += captured_now;
                    released_now_total += released_now;
                    if captured_now > f64::EPSILON {
                        captured_ledger_allocations.push((
                            account_ledger_allocation_id(
                                allocation.hold_allocation_id,
                                SETTLEMENT_CAPTURE_LEDGER_SUFFIX,
                            ),
                            allocation.lot_id,
                            -captured_now,
                        ));
                    }
                    if released_now > f64::EPSILON {
                        released_ledger_allocations.push((
                            account_ledger_allocation_id(
                                allocation.hold_allocation_id,
                                SETTLEMENT_RELEASE_LEDGER_SUFFIX,
                            ),
                            allocation.lot_id,
                            released_now,
                        ));
                    }
                }

                ensure!(
                    remaining_capture <= f64::EPSILON,
                    "capture quantity {} exceeds held quantity for request {}",
                    input.captured_quantity,
                    input.request_id
                );

                let status = if released_quantity > f64::EPSILON {
                    AccountHoldStatus::PartiallyReleased
                } else {
                    AccountHoldStatus::Captured
                };
                let updated_hold = hold
                    .with_status(status)
                    .with_captured_quantity(captured_quantity)
                    .with_released_quantity(released_quantity)
                    .with_updated_at_ms(input.settled_at_ms);
                tx.upsert_account_hold(&updated_hold).await?;

                let settlement = RequestSettlementRecord::new(
                    input.request_settlement_id,
                    updated_hold.tenant_id,
                    updated_hold.organization_id,
                    input.request_id,
                    updated_hold.account_id,
                    updated_hold.user_id,
                )
                .with_hold_id(Some(updated_hold.hold_id))
                .with_status(if released_quantity > f64::EPSILON {
                    RequestSettlementStatus::PartiallyReleased
                } else {
                    RequestSettlementStatus::Captured
                })
                .with_estimated_credit_hold(updated_hold.estimated_quantity)
                .with_released_credit_amount(released_quantity)
                .with_captured_credit_amount(captured_quantity)
                .with_provider_cost_amount(input.provider_cost_amount)
                .with_retail_charge_amount(input.retail_charge_amount)
                .with_shortfall_amount(
                    (input.retail_charge_amount - input.captured_quantity).max(0.0),
                )
                .with_settled_at_ms(input.settled_at_ms)
                .with_created_at_ms(input.settled_at_ms)
                .with_updated_at_ms(input.settled_at_ms);
                tx.upsert_request_settlement_record(&settlement).await?;

                if captured_now_total > f64::EPSILON {
                    let capture_ledger_entry = AccountLedgerEntryRecord::new(
                        account_ledger_entry_id(
                            input.request_settlement_id,
                            SETTLEMENT_CAPTURE_LEDGER_SUFFIX,
                        ),
                        updated_hold.tenant_id,
                        updated_hold.organization_id,
                        updated_hold.account_id,
                        updated_hold.user_id,
                        AccountLedgerEntryType::SettlementCapture,
                    )
                    .with_request_id(Some(input.request_id))
                    .with_hold_id(Some(updated_hold.hold_id))
                    .with_quantity(captured_now_total)
                    .with_amount(input.retail_charge_amount)
                    .with_created_at_ms(input.settled_at_ms);
                    write_account_ledger_entry(
                        tx,
                        &capture_ledger_entry,
                        &captured_ledger_allocations,
                        input.settled_at_ms,
                    )
                    .await?;
                }

                if released_now_total > f64::EPSILON {
                    let release_ledger_entry = AccountLedgerEntryRecord::new(
                        account_ledger_entry_id(
                            input.request_settlement_id,
                            SETTLEMENT_RELEASE_LEDGER_SUFFIX,
                        ),
                        updated_hold.tenant_id,
                        updated_hold.organization_id,
                        updated_hold.account_id,
                        updated_hold.user_id,
                        AccountLedgerEntryType::HoldRelease,
                    )
                    .with_request_id(Some(input.request_id))
                    .with_hold_id(Some(updated_hold.hold_id))
                    .with_quantity(released_now_total)
                    .with_created_at_ms(input.settled_at_ms);
                    write_account_ledger_entry(
                        tx,
                        &release_ledger_entry,
                        &released_ledger_allocations,
                        input.settled_at_ms,
                    )
                    .await?;
                }

                Ok(CaptureAccountHoldResult {
                    idempotent_replay: false,
                    hold: updated_hold,
                    allocations,
                    updated_lots,
                    settlement,
                })
            })
        })
        .await
}

pub async fn refund_account_settlement<S>(
    store: &S,
    input: RefundAccountSettlementInput,
) -> Result<RefundAccountSettlementResult>
where
    S: AccountKernelStore + AccountKernelTransactionExecutor + ?Sized,
{
    ensure!(
        input.refunded_amount > 0.0,
        "refunded_amount must be positive"
    );

    store
        .with_account_kernel_transaction(|tx| {
            Box::pin(async move {
                let settlement = tx
                    .find_request_settlement_record(input.request_settlement_id)
                    .await?
                    .ok_or_else(|| {
                        anyhow::anyhow!(
                            "request settlement {} does not exist",
                            input.request_settlement_id
                        )
                    })?;

                if let Some(existing_ledger_entry) = tx
                    .find_account_ledger_entry_record(input.refund_ledger_entry_id)
                    .await?
                {
                    ensure!(
                        existing_ledger_entry.entry_type == AccountLedgerEntryType::Refund,
                        "ledger entry {} is not a refund entry",
                        input.refund_ledger_entry_id
                    );
                    let (ledger_allocations, updated_lots) =
                        load_account_ledger_allocations_and_lots(
                            tx,
                            existing_ledger_entry.ledger_entry_id,
                        )
                        .await?;
                    return Ok(RefundAccountSettlementResult {
                        idempotent_replay: true,
                        settlement,
                        updated_lots,
                        ledger_entry: existing_ledger_entry,
                        ledger_allocations,
                    });
                }

                ensure!(
                    matches!(
                        settlement.status,
                        RequestSettlementStatus::Captured
                            | RequestSettlementStatus::PartiallyReleased
                            | RequestSettlementStatus::Refunded
                    ),
                    "request settlement {} is not refundable",
                    input.request_settlement_id
                );

                let remaining_refundable =
                    (settlement.captured_credit_amount - settlement.refunded_amount).max(0.0);
                ensure!(
                    remaining_refundable >= input.refunded_amount,
                    "refund amount {} exceeds refundable captured balance {} for settlement {}",
                    input.refunded_amount,
                    remaining_refundable,
                    input.request_settlement_id
                );

                let hold_id = settlement.hold_id.ok_or_else(|| {
                    anyhow::anyhow!(
                        "request settlement {} does not reference an account hold",
                        input.request_settlement_id
                    )
                })?;
                let existing_allocations =
                    tx.list_account_hold_allocations_for_hold(hold_id).await?;
                let mut already_refunded_remaining = settlement.refunded_amount;
                let mut refund_remaining = input.refunded_amount;
                let mut updated_lots = Vec::new();
                let mut refund_ledger_allocations = Vec::new();

                for allocation in existing_allocations {
                    let mut refundable_from_allocation = allocation.captured_quantity.max(0.0);
                    if already_refunded_remaining > f64::EPSILON {
                        let previously_refunded =
                            refundable_from_allocation.min(already_refunded_remaining);
                        already_refunded_remaining -= previously_refunded;
                        refundable_from_allocation -= previously_refunded;
                    }

                    if refund_remaining <= f64::EPSILON {
                        break;
                    }

                    let refunded_now = refundable_from_allocation.min(refund_remaining);
                    if refunded_now <= f64::EPSILON {
                        continue;
                    }

                    let lot = tx
                        .find_account_benefit_lot(allocation.lot_id)
                        .await?
                        .ok_or_else(|| {
                            anyhow::anyhow!("lot {} does not exist", allocation.lot_id)
                        })?;
                    let updated_lot = lot
                        .clone()
                        .with_remaining_quantity(lot.remaining_quantity + refunded_now)
                        .with_updated_at_ms(input.refunded_at_ms);
                    tx.upsert_account_benefit_lot(&updated_lot).await?;
                    updated_lots.push(updated_lot);

                    refund_ledger_allocations.push((
                        input.refund_ledger_allocation_start_id
                            + refund_ledger_allocations.len() as u64,
                        allocation.lot_id,
                        refunded_now,
                    ));
                    refund_remaining -= refunded_now;
                }

                ensure!(
                    refund_remaining <= f64::EPSILON,
                    "refund amount {} exceeds captured allocations for settlement {}",
                    input.refunded_amount,
                    input.request_settlement_id
                );

                let updated_settlement = settlement
                    .clone()
                    .with_status(RequestSettlementStatus::Refunded)
                    .with_refunded_amount(settlement.refunded_amount + input.refunded_amount)
                    .with_settled_at_ms(input.refunded_at_ms)
                    .with_updated_at_ms(input.refunded_at_ms);
                tx.upsert_request_settlement_record(&updated_settlement)
                    .await?;

                let refund_ledger_entry = AccountLedgerEntryRecord::new(
                    input.refund_ledger_entry_id,
                    settlement.tenant_id,
                    settlement.organization_id,
                    settlement.account_id,
                    settlement.user_id,
                    AccountLedgerEntryType::Refund,
                )
                .with_request_id(Some(settlement.request_id))
                .with_hold_id(settlement.hold_id)
                .with_quantity(input.refunded_amount)
                .with_amount(input.refunded_amount)
                .with_created_at_ms(input.refunded_at_ms);
                let ledger_allocations = write_account_ledger_entry(
                    tx,
                    &refund_ledger_entry,
                    &refund_ledger_allocations,
                    input.refunded_at_ms,
                )
                .await?;

                Ok(RefundAccountSettlementResult {
                    idempotent_replay: false,
                    settlement: updated_settlement,
                    updated_lots,
                    ledger_entry: refund_ledger_entry,
                    ledger_allocations,
                })
            })
        })
        .await
}

