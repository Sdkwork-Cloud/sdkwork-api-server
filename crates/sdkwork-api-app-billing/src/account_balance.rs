use super::*;

pub async fn summarize_account_balance<S>(
    store: &S,
    account_id: u64,
    now_ms: u64,
) -> Result<AccountBalanceSnapshot>
where
    S: AccountKernelStore + ?Sized,
{
    ensure!(
        store.find_account_record(account_id).await?.is_some(),
        "account {account_id} does not exist"
    );

    let account_lots = store
        .list_account_benefit_lots()
        .await?
        .into_iter()
        .filter(|lot| lot.account_id == account_id)
        .collect::<Vec<_>>();
    let active_lots = eligible_lots_for_hold(&account_lots, now_ms);

    let available_balance = active_lots.iter().map(|lot| free_quantity(lot)).sum();
    let held_balance = account_lots.iter().map(|lot| lot.held_quantity).sum();
    let consumed_balance = account_lots
        .iter()
        .map(|lot| (lot.original_quantity - lot.remaining_quantity).max(0.0))
        .sum();
    let grant_balance = account_lots.iter().map(|lot| lot.original_quantity).sum();
    let lots = active_lots
        .into_iter()
        .map(|lot| AccountLotBalanceSnapshot {
            lot_id: lot.lot_id,
            benefit_type: lot.benefit_type,
            scope_json: lot.scope_json.clone(),
            expires_at_ms: lot.expires_at_ms,
            original_quantity: lot.original_quantity,
            remaining_quantity: lot.remaining_quantity,
            held_quantity: lot.held_quantity,
            available_quantity: free_quantity(lot),
        })
        .collect::<Vec<_>>();

    Ok(AccountBalanceSnapshot {
        account_id,
        available_balance,
        held_balance,
        consumed_balance,
        grant_balance,
        active_lot_count: lots.len() as u64,
        lots,
    })
}

pub async fn list_account_ledger_history<S>(
    store: &S,
    account_id: u64,
) -> Result<Vec<AccountLedgerHistoryEntry>>
where
    S: AccountKernelStore + ?Sized,
{
    ensure!(
        store.find_account_record(account_id).await?.is_some(),
        "account {account_id} does not exist"
    );

    let mut allocations_by_entry_id = BTreeMap::<u64, Vec<AccountLedgerAllocationRecord>>::new();
    for allocation in store.list_account_ledger_allocations().await? {
        allocations_by_entry_id
            .entry(allocation.ledger_entry_id)
            .or_default()
            .push(allocation);
    }
    for allocations in allocations_by_entry_id.values_mut() {
        allocations.sort_by_key(|allocation| allocation.ledger_allocation_id);
    }

    let mut history = store
        .list_account_ledger_entry_records()
        .await?
        .into_iter()
        .filter(|entry| entry.account_id == account_id)
        .map(|entry| AccountLedgerHistoryEntry {
            allocations: allocations_by_entry_id
                .remove(&entry.ledger_entry_id)
                .unwrap_or_default(),
            entry,
        })
        .collect::<Vec<_>>();
    history.sort_by(|left, right| {
        right
            .entry
            .created_at_ms
            .cmp(&left.entry.created_at_ms)
            .then_with(|| right.entry.ledger_entry_id.cmp(&left.entry.ledger_entry_id))
    });
    Ok(history)
}

pub async fn plan_account_hold<S>(
    store: &S,
    account_id: u64,
    requested_quantity: f64,
    now_ms: u64,
) -> Result<AccountHoldPlan>
where
    S: AccountKernelStore + ?Sized,
{
    ensure!(
        requested_quantity > 0.0,
        "requested_quantity must be positive"
    );

    ensure!(
        store.find_account_record(account_id).await?.is_some(),
        "account {account_id} does not exist"
    );

    let lots = store
        .list_account_benefit_lots()
        .await?
        .into_iter()
        .filter(|lot| lot.account_id == account_id)
        .collect::<Vec<_>>();
    let eligible_lots = eligible_lots_for_hold(&lots, now_ms);
    let mut remaining = requested_quantity;
    let mut allocations = Vec::new();

    for lot in eligible_lots {
        if remaining <= 0.0 {
            break;
        }
        let quantity = free_quantity(lot).min(remaining);
        if quantity <= 0.0 {
            continue;
        }
        allocations.push(PlannedHoldAllocation {
            lot_id: lot.lot_id,
            quantity,
        });
        remaining -= quantity;
    }

    let covered_quantity = requested_quantity - remaining.max(0.0);
    let shortfall_quantity = remaining.max(0.0);

    Ok(AccountHoldPlan {
        account_id,
        requested_quantity,
        covered_quantity,
        shortfall_quantity,
        sufficient_balance: shortfall_quantity <= f64::EPSILON,
        allocations,
    })
}

pub async fn resolve_payable_account_for_gateway_subject<S>(
    store: &S,
    subject: &GatewayAuthSubject,
) -> Result<Option<AccountRecord>>
where
    S: AccountKernelStore + ?Sized,
{
    let Some(account) = store
        .find_account_record_by_owner(
            subject.tenant_id,
            subject.organization_id,
            subject.user_id,
            AccountType::Primary,
        )
        .await?
    else {
        return Ok(None);
    };

    ensure!(
        account.status == AccountStatus::Active,
        "primary account {} is not active",
        account.account_id
    );

    Ok(Some(account))
}

pub async fn resolve_payable_account_for_gateway_request_context<S>(
    store: &S,
    context: &IdentityGatewayRequestContext,
) -> Result<Option<AccountRecord>>
where
    S: AccountKernelStore + ?Sized,
{
    let subject = gateway_auth_subject_from_request_context(context);
    resolve_payable_account_for_gateway_subject(store, &subject).await
}

