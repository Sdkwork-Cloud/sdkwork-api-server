use super::*;

pub(crate) fn eligible_lots_for_hold(
    lots: &[AccountBenefitLotRecord],
    now_ms: u64,
) -> Vec<&AccountBenefitLotRecord> {
    let mut eligible = lots
        .iter()
        .filter(|lot| {
            lot.status == AccountBenefitLotStatus::Active
                && lot
                    .expires_at_ms
                    .map(|expires_at_ms| expires_at_ms > now_ms)
                    .unwrap_or(true)
                && free_quantity(lot) > 0.0
        })
        .collect::<Vec<_>>();
    eligible.sort_by(|left, right| {
        left.expires_at_ms
            .unwrap_or(u64::MAX)
            .cmp(&right.expires_at_ms.unwrap_or(u64::MAX))
            .then_with(|| right.scope_json.is_some().cmp(&left.scope_json.is_some()))
            .then_with(|| {
                benefit_cash_rank(left.benefit_type).cmp(&benefit_cash_rank(right.benefit_type))
            })
            .then_with(|| {
                left.acquired_unit_cost
                    .unwrap_or(f64::INFINITY)
                    .total_cmp(&right.acquired_unit_cost.unwrap_or(f64::INFINITY))
            })
            .then_with(|| left.lot_id.cmp(&right.lot_id))
    });
    eligible
}

pub(crate) fn free_quantity(lot: &AccountBenefitLotRecord) -> f64 {
    (lot.remaining_quantity - lot.held_quantity).max(0.0)
}

fn benefit_cash_rank(benefit_type: AccountBenefitType) -> u8 {
    match benefit_type {
        AccountBenefitType::CashCredit => 1,
        _ => 0,
    }
}

const ACCOUNT_LEDGER_ID_MULTIPLIER: u64 = 10;
pub(crate) const HOLD_CREATE_LEDGER_SUFFIX: u64 = 1;
pub(crate) const HOLD_RELEASE_LEDGER_SUFFIX: u64 = 2;
pub(crate) const SETTLEMENT_CAPTURE_LEDGER_SUFFIX: u64 = 3;
pub(crate) const SETTLEMENT_RELEASE_LEDGER_SUFFIX: u64 = 4;
pub(crate) const ACCOUNTING_EPSILON: f64 = 0.000_001;
const FNV1A_64_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
const FNV1A_64_PRIME: u64 = 0x100000001b3;

pub(crate) fn account_ledger_entry_id(base_id: u64, suffix: u64) -> u64 {
    base_id
        .saturating_mul(ACCOUNT_LEDGER_ID_MULTIPLIER)
        .saturating_add(suffix)
}

pub(crate) fn account_ledger_allocation_id(base_id: u64, suffix: u64) -> u64 {
    base_id
        .saturating_mul(ACCOUNT_LEDGER_ID_MULTIPLIER)
        .saturating_add(suffix)
}

fn stable_commerce_u64(namespace: &str, order_id: &str) -> u64 {
    let mut hash = FNV1A_64_OFFSET_BASIS;
    for byte in namespace
        .as_bytes()
        .iter()
        .copied()
        .chain(std::iter::once(0xff))
        .chain(order_id.as_bytes().iter().copied())
    {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(FNV1A_64_PRIME);
    }

    let bounded = hash & (i64::MAX as u64);

    if bounded == 0 {
        1
    } else {
        bounded
    }
}

pub(crate) fn commerce_order_source_id(order_id: &str) -> u64 {
    stable_commerce_u64("commerce_source", order_id)
}

pub(crate) fn commerce_order_lot_id(order_id: &str) -> u64 {
    stable_commerce_u64("commerce_lot", order_id)
}

pub(crate) fn commerce_order_issue_ledger_entry_id(order_id: &str) -> u64 {
    stable_commerce_u64("commerce_issue_ledger", order_id)
}

pub(crate) fn commerce_order_issue_ledger_allocation_id(order_id: &str) -> u64 {
    stable_commerce_u64("commerce_issue_allocation", order_id)
}

pub(crate) fn commerce_order_refund_ledger_entry_id(order_id: &str) -> u64 {
    stable_commerce_u64("commerce_refund_ledger", order_id)
}

pub(crate) fn commerce_order_refund_ledger_allocation_id(order_id: &str) -> u64 {
    stable_commerce_u64("commerce_refund_allocation", order_id)
}

pub(crate) fn ensure_quantity_matches(actual: f64, expected: f64, field_name: String) -> Result<()> {
    ensure!(
        (actual - expected).abs() <= ACCOUNTING_EPSILON,
        "{field_name} mismatch: expected {expected}, found {actual}"
    );
    Ok(())
}

pub(crate) fn build_commerce_order_credit_scope_json(
    order_id: &str,
    project_id: &str,
    target_kind: &str,
) -> Result<String> {
    serde_json::to_string(&serde_json::json!({
        "order_id": order_id,
        "project_id": project_id,
        "target_kind": target_kind,
    }))
    .map_err(Into::into)
}

pub(crate) fn validate_commerce_order_credit_lot(
    lot: &AccountBenefitLotRecord,
    account_id: u64,
    order_source_id: u64,
    order_id: &str,
) -> Result<()> {
    ensure!(
        lot.account_id == account_id,
        "commerce order {order_id} lot {} belongs to another account",
        lot.lot_id
    );
    ensure!(
        lot.source_type == AccountBenefitSourceType::Order,
        "commerce order {order_id} lot {} has unexpected source type",
        lot.lot_id
    );
    ensure!(
        lot.source_id == Some(order_source_id),
        "commerce order {order_id} lot {} has unexpected source id",
        lot.lot_id
    );
    ensure!(
        commerce_order_scope_matches(lot.scope_json.as_deref(), order_id),
        "commerce order {order_id} lot {} has unexpected scope metadata",
        lot.lot_id
    );
    Ok(())
}

fn commerce_order_scope_matches(scope_json: Option<&str>, order_id: &str) -> bool {
    scope_json
        .and_then(|value| serde_json::from_str::<serde_json::Value>(value).ok())
        .and_then(|value| {
            value
                .get("order_id")
                .and_then(|order_value| order_value.as_str().map(str::to_owned))
        })
        .is_some_and(|stored_order_id| stored_order_id == order_id)
}

pub(crate) async fn write_account_ledger_entry(
    tx: &mut dyn AccountKernelTransaction,
    entry: &AccountLedgerEntryRecord,
    allocation_deltas: &[(u64, u64, f64)],
    created_at_ms: u64,
) -> Result<Vec<AccountLedgerAllocationRecord>> {
    tx.upsert_account_ledger_entry_record(entry).await?;

    let mut allocations = Vec::with_capacity(allocation_deltas.len());
    for (ledger_allocation_id, lot_id, quantity_delta) in allocation_deltas {
        let allocation = AccountLedgerAllocationRecord::new(
            *ledger_allocation_id,
            entry.tenant_id,
            entry.organization_id,
            entry.ledger_entry_id,
            *lot_id,
        )
        .with_quantity_delta(*quantity_delta)
        .with_created_at_ms(created_at_ms);
        tx.upsert_account_ledger_allocation(&allocation).await?;
        allocations.push(allocation);
    }

    Ok(allocations)
}

pub(crate) async fn load_hold_allocations_and_lots(
    tx: &mut dyn AccountKernelTransaction,
    hold_id: u64,
) -> Result<(
    Vec<AccountHoldAllocationRecord>,
    Vec<AccountBenefitLotRecord>,
)> {
    let mut allocations = tx.list_account_hold_allocations_for_hold(hold_id).await?;
    allocations.sort_by_key(|allocation| allocation.hold_allocation_id);

    let mut lots = Vec::with_capacity(allocations.len());
    for allocation in &allocations {
        let lot = tx
            .find_account_benefit_lot(allocation.lot_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("lot {} does not exist", allocation.lot_id))?;
        lots.push(lot);
    }
    lots.sort_by_key(|lot| lot.lot_id);

    Ok((allocations, lots))
}

pub(crate) async fn load_account_ledger_allocations_and_lots(
    tx: &mut dyn AccountKernelTransaction,
    ledger_entry_id: u64,
) -> Result<(
    Vec<AccountLedgerAllocationRecord>,
    Vec<AccountBenefitLotRecord>,
)> {
    let mut allocations = tx
        .list_account_ledger_allocations_for_entry(ledger_entry_id)
        .await?;
    allocations.sort_by_key(|allocation| allocation.ledger_allocation_id);

    let mut lots = Vec::with_capacity(allocations.len());
    for allocation in &allocations {
        let lot = tx
            .find_account_benefit_lot(allocation.lot_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("lot {} does not exist", allocation.lot_id))?;
        lots.push(lot);
    }
    lots.sort_by_key(|lot| lot.lot_id);

    Ok((allocations, lots))
}

