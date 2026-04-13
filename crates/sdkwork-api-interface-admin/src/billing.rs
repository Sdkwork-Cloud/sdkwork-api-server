use super::*;

pub(crate) async fn list_usage_records_handler(
    claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<UsageRecord>>, StatusCode> {
    require_admin_privilege(&claims, AdminPrivilege::BillingRead)?;
    list_usage_records(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub(crate) async fn usage_summary_handler(
    claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<UsageSummary>, StatusCode> {
    require_admin_privilege(&claims, AdminPrivilege::BillingRead)?;
    summarize_usage_records_from_store(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub(crate) async fn list_ledger_entries_handler(
    claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<LedgerEntry>>, StatusCode> {
    require_admin_privilege(&claims, AdminPrivilege::BillingRead)?;
    list_ledger_entries(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub(crate) async fn list_billing_events_handler(
    claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<BillingEventRecord>>, StatusCode> {
    require_admin_privilege(&claims, AdminPrivilege::BillingRead)?;
    list_billing_events(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub(crate) async fn billing_events_summary_handler(
    claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<BillingEventSummary>, StatusCode> {
    require_admin_privilege(&claims, AdminPrivilege::BillingRead)?;
    summarize_billing_events_from_store(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub(crate) async fn billing_summary_handler(
    claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<BillingSummary>, StatusCode> {
    require_admin_privilege(&claims, AdminPrivilege::BillingRead)?;
    summarize_billing_from_store(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub(crate) async fn list_canonical_accounts_handler(
    claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<CommercialAccountSummaryResponse>>, (StatusCode, Json<ErrorResponse>)> {
    require_admin_privilege(&claims, AdminPrivilege::BillingRead)
        .map_err(|_| admin_forbidden_response())?;
    let commercial_billing = commercial_billing_kernel(&state)?.clone();
    let now_ms = unix_timestamp_ms();
    let mut accounts = commercial_billing
        .list_account_records()
        .await
        .map_err(commercial_billing_error_response)?;
    accounts.sort_by_key(|account| account.account_id);

    let mut response = Vec::with_capacity(accounts.len());
    for account in accounts {
        let balance = commercial_billing
            .summarize_account_balance(account.account_id, now_ms)
            .await
            .map_err(commercial_billing_error_response)?;
        response.push(CommercialAccountSummaryResponse::from_balance(
            account, &balance,
        ));
    }

    Ok(Json(response))
}

pub(crate) async fn get_canonical_account_balance_handler(
    claims: AuthenticatedAdminClaims,
    Path(account_id): Path<u64>,
    State(state): State<AdminApiState>,
) -> Result<Json<AccountBalanceSnapshot>, (StatusCode, Json<ErrorResponse>)> {
    require_admin_privilege(&claims, AdminPrivilege::BillingRead)
        .map_err(|_| admin_forbidden_response())?;
    let commercial_billing = commercial_billing_kernel(&state)?.clone();
    let balance = commercial_billing
        .summarize_account_balance(account_id, unix_timestamp_ms())
        .await
        .map_err(commercial_billing_error_response)?;
    Ok(Json(balance))
}

pub(crate) async fn list_canonical_account_benefit_lots_handler(
    claims: AuthenticatedAdminClaims,
    Path(account_id): Path<u64>,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<AccountBenefitLotRecord>>, (StatusCode, Json<ErrorResponse>)> {
    require_admin_privilege(&claims, AdminPrivilege::BillingRead)
        .map_err(|_| admin_forbidden_response())?;
    let commercial_billing = commercial_billing_kernel(&state)?.clone();
    commercial_billing
        .find_account_record(account_id)
        .await
        .map_err(commercial_billing_error_response)?
        .ok_or_else(|| {
            error_response(
                StatusCode::NOT_FOUND,
                format!("account {account_id} does not exist"),
            )
        })?;

    let mut lots = commercial_billing
        .list_account_benefit_lots()
        .await
        .map_err(commercial_billing_error_response)?
        .into_iter()
        .filter(|lot| lot.account_id == account_id)
        .collect::<Vec<_>>();
    lots.sort_by_key(|lot| lot.lot_id);
    Ok(Json(lots))
}

pub(crate) async fn list_canonical_account_ledger_handler(
    claims: AuthenticatedAdminClaims,
    Path(account_id): Path<u64>,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<AccountLedgerHistoryEntry>>, (StatusCode, Json<ErrorResponse>)> {
    require_admin_privilege(&claims, AdminPrivilege::BillingRead)
        .map_err(|_| admin_forbidden_response())?;
    let commercial_billing = commercial_billing_kernel(&state)?.clone();
    let history = commercial_billing
        .list_account_ledger_history(account_id)
        .await
        .map_err(commercial_billing_error_response)?;
    Ok(Json(history))
}

pub(crate) async fn list_canonical_account_holds_handler(
    claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<AccountHoldRecord>>, (StatusCode, Json<ErrorResponse>)> {
    require_admin_privilege(&claims, AdminPrivilege::BillingRead)
        .map_err(|_| admin_forbidden_response())?;
    let commercial_billing = commercial_billing_kernel(&state)?.clone();
    let mut holds = commercial_billing
        .list_account_holds()
        .await
        .map_err(commercial_billing_error_response)?;
    holds.sort_by_key(|hold| hold.hold_id);
    Ok(Json(holds))
}

pub(crate) async fn list_canonical_request_settlements_handler(
    claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<RequestSettlementRecord>>, (StatusCode, Json<ErrorResponse>)> {
    require_admin_privilege(&claims, AdminPrivilege::BillingRead)
        .map_err(|_| admin_forbidden_response())?;
    let commercial_billing = commercial_billing_kernel(&state)?.clone();
    let mut settlements = commercial_billing
        .list_request_settlement_records()
        .await
        .map_err(commercial_billing_error_response)?;
    settlements.sort_by_key(|settlement| settlement.request_settlement_id);
    Ok(Json(settlements))
}
