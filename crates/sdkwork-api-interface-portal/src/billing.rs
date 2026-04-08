use super::*;

#[derive(Debug, Serialize)]
pub(crate) struct PortalBillingAccountResponse {
    account: AccountRecord,
    #[serde(flatten)]
    balance: AccountBalanceSnapshot,
}

#[derive(Debug, Serialize)]
pub(crate) struct PortalBillingAccountHistoryResponse {
    account: AccountRecord,
    balance: AccountBalanceSnapshot,
    benefit_lots: Vec<AccountBenefitLotRecord>,
    holds: Vec<AccountHoldRecord>,
    request_settlements: Vec<RequestSettlementRecord>,
    ledger: Vec<AccountLedgerHistoryEntry>,
}


pub(crate) async fn billing_account_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<PortalBillingAccountResponse>, (StatusCode, Json<ErrorResponse>)> {
    let (account, balance) = load_portal_billing_account_context(&state, &claims).await?;
    Ok(Json(PortalBillingAccountResponse { account, balance }))
}

pub(crate) async fn billing_account_balance_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<AccountBalanceSnapshot>, (StatusCode, Json<ErrorResponse>)> {
    let (_, balance) = load_portal_billing_account_context(&state, &claims).await?;
    Ok(Json(balance))
}

pub(crate) async fn billing_account_history_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<PortalBillingAccountHistoryResponse>, (StatusCode, Json<ErrorResponse>)> {
    let (account, balance) = load_portal_billing_account_context(&state, &claims).await?;
    let commercial_billing = commercial_billing_kernel(&state)?.clone();
    let mut benefit_lots = commercial_billing
        .list_account_benefit_lots()
        .await
        .map_err(commercial_billing_error_response)?
        .into_iter()
        .filter(|lot| lot.account_id == account.account_id)
        .collect::<Vec<_>>();
    benefit_lots.sort_by_key(|lot| lot.lot_id);

    let mut holds = commercial_billing
        .list_account_holds()
        .await
        .map_err(commercial_billing_error_response)?
        .into_iter()
        .filter(|hold| hold.account_id == account.account_id)
        .collect::<Vec<_>>();
    holds.sort_by_key(|hold| hold.hold_id);

    let mut request_settlements = commercial_billing
        .list_request_settlement_records()
        .await
        .map_err(commercial_billing_error_response)?
        .into_iter()
        .filter(|settlement| settlement.account_id == account.account_id)
        .collect::<Vec<_>>();
    request_settlements.sort_by_key(|settlement| settlement.request_settlement_id);

    let ledger = commercial_billing
        .list_account_ledger_history(account.account_id)
        .await
        .map_err(commercial_billing_error_response)?;

    Ok(Json(PortalBillingAccountHistoryResponse {
        account,
        balance,
        benefit_lots,
        holds,
        request_settlements,
        ledger,
    }))
}

pub(crate) async fn list_billing_account_benefit_lots_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<Vec<AccountBenefitLotRecord>>, (StatusCode, Json<ErrorResponse>)> {
    let (account, _) = load_portal_billing_account_context(&state, &claims).await?;
    let commercial_billing = commercial_billing_kernel(&state)?.clone();
    let mut lots = commercial_billing
        .list_account_benefit_lots()
        .await
        .map_err(commercial_billing_error_response)?
        .into_iter()
        .filter(|lot| lot.account_id == account.account_id)
        .collect::<Vec<_>>();
    lots.sort_by_key(|lot| lot.lot_id);
    Ok(Json(lots))
}

pub(crate) async fn list_billing_account_holds_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<Vec<AccountHoldRecord>>, (StatusCode, Json<ErrorResponse>)> {
    let (account, _) = load_portal_billing_account_context(&state, &claims).await?;
    let commercial_billing = commercial_billing_kernel(&state)?.clone();
    let mut holds = commercial_billing
        .list_account_holds()
        .await
        .map_err(commercial_billing_error_response)?
        .into_iter()
        .filter(|hold| hold.account_id == account.account_id)
        .collect::<Vec<_>>();
    holds.sort_by_key(|hold| hold.hold_id);
    Ok(Json(holds))
}

pub(crate) async fn list_billing_request_settlements_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<Vec<RequestSettlementRecord>>, (StatusCode, Json<ErrorResponse>)> {
    let (account, _) = load_portal_billing_account_context(&state, &claims).await?;
    let commercial_billing = commercial_billing_kernel(&state)?.clone();
    let mut settlements = commercial_billing
        .list_request_settlement_records()
        .await
        .map_err(commercial_billing_error_response)?
        .into_iter()
        .filter(|settlement| settlement.account_id == account.account_id)
        .collect::<Vec<_>>();
    settlements.sort_by_key(|settlement| settlement.request_settlement_id);
    Ok(Json(settlements))
}

pub(crate) async fn list_billing_account_ledger_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<Vec<AccountLedgerHistoryEntry>>, (StatusCode, Json<ErrorResponse>)> {
    let (account, _) = load_portal_billing_account_context(&state, &claims).await?;
    let commercial_billing = commercial_billing_kernel(&state)?.clone();
    let history = commercial_billing
        .list_account_ledger_history(account.account_id)
        .await
        .map_err(commercial_billing_error_response)?;
    Ok(Json(history))
}

pub(crate) async fn list_billing_pricing_plans_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<Vec<PricingPlanRecord>>, (StatusCode, Json<ErrorResponse>)> {
    let (account, _) = load_portal_billing_account_context(&state, &claims).await?;
    let commercial_billing = commercial_billing_kernel(&state)?.clone();
    synchronize_due_pricing_plan_lifecycle(commercial_billing.as_ref(), current_time_millis())
        .await
        .map_err(commercial_billing_error_response)?;
    let mut plans = commercial_billing
        .list_pricing_plan_records()
        .await
        .map_err(commercial_billing_error_response)?
        .into_iter()
        .filter(|plan| {
            plan.tenant_id == account.tenant_id && plan.organization_id == account.organization_id
        })
        .collect::<Vec<_>>();
    plans.sort_by_key(|plan| plan.pricing_plan_id);
    Ok(Json(plans))
}

pub(crate) async fn list_billing_pricing_rates_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<Vec<PricingRateRecord>>, (StatusCode, Json<ErrorResponse>)> {
    let (account, _) = load_portal_billing_account_context(&state, &claims).await?;
    let commercial_billing = commercial_billing_kernel(&state)?.clone();
    synchronize_due_pricing_plan_lifecycle(commercial_billing.as_ref(), current_time_millis())
        .await
        .map_err(commercial_billing_error_response)?;
    let scoped_plan_ids = commercial_billing
        .list_pricing_plan_records()
        .await
        .map_err(commercial_billing_error_response)?
        .into_iter()
        .filter(|plan| {
            plan.tenant_id == account.tenant_id && plan.organization_id == account.organization_id
        })
        .map(|plan| plan.pricing_plan_id)
        .collect::<HashSet<_>>();
    let mut rates = commercial_billing
        .list_pricing_rate_records()
        .await
        .map_err(commercial_billing_error_response)?
        .into_iter()
        .filter(|rate| scoped_plan_ids.contains(&rate.pricing_plan_id))
        .collect::<Vec<_>>();
    rates.sort_by_key(|rate| rate.pricing_rate_id);
    Ok(Json(rates))
}
pub(crate) async fn billing_summary_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<ProjectBillingSummary>, StatusCode> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub).await?;
    load_project_billing_summary(state.store.as_ref(), &workspace.project.id)
        .await
        .map(Json)
}

pub(crate) async fn list_billing_ledger_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<Vec<LedgerEntry>>, StatusCode> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub).await?;
    let ledger = state
        .store
        .list_ledger_entries_for_project(&workspace.project.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(ledger))
}

pub(crate) async fn list_billing_events_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<Vec<BillingEventRecord>>, StatusCode> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub).await?;
    load_project_billing_events(
        state.store.as_ref(),
        &workspace.tenant.id,
        &workspace.project.id,
    )
    .await
    .map(Json)
}

pub(crate) async fn billing_events_summary_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<BillingEventSummary>, StatusCode> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub).await?;
    let events = load_project_billing_events(
        state.store.as_ref(),
        &workspace.tenant.id,
        &workspace.project.id,
    )
    .await?;
    Ok(Json(summarize_billing_events(&events)))
}

