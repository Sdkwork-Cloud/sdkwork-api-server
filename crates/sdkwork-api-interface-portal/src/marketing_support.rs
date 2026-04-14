use super::*;

#[derive(Debug, Clone, Default)]
pub(crate) struct PortalCouponAccountArrivalContext {
    account_id: Option<u64>,
    lots_by_order_id: HashMap<String, Vec<AccountBenefitLotRecord>>,
}

pub(crate) async fn load_portal_marketing_workspace_and_subjects(
    state: &PortalApiState,
    claims: &AuthenticatedPortalClaims,
) -> Result<(PortalWorkspaceSummary, MarketingSubjectSet), StatusCode> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub).await?;
    let account_id = load_portal_marketing_account_id(state, &workspace).await?;
    let subjects = MarketingSubjectSet::new(
        Some(claims.claims().sub.clone()),
        Some(workspace.project.id.clone()),
        Some(format!("{}:{}", workspace.tenant.id, workspace.project.id)),
        account_id,
    );
    Ok((workspace, subjects))
}

async fn load_portal_marketing_account_id(
    state: &PortalApiState,
    workspace: &PortalWorkspaceSummary,
) -> Result<Option<String>, StatusCode> {
    let Some(commercial_billing) = state.commercial_billing.as_ref() else {
        return Ok(None);
    };

    let account = commercial_billing
        .resolve_payable_account_for_gateway_request_context(&portal_workspace_request_context(
            workspace,
        ))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(account.map(|record| record.account_id.to_string()))
}

impl PortalCouponAccountArrivalContext {
    pub(crate) fn from_account_lots(account_id: u64, lots: Vec<AccountBenefitLotRecord>) -> Self {
        let mut lots_by_order_id = HashMap::<String, Vec<AccountBenefitLotRecord>>::new();
        for lot in lots {
            if lot.source_type != sdkwork_api_domain_billing::AccountBenefitSourceType::Order {
                continue;
            }
            let Some(order_id) = parse_scope_order_id(lot.scope_json.as_deref()) else {
                continue;
            };
            lots_by_order_id.entry(order_id).or_default().push(lot);
        }

        Self {
            account_id: Some(account_id),
            lots_by_order_id,
        }
    }
}

pub(crate) fn coupon_validation_decision_response(
    decision: CouponValidationDecision,
) -> PortalCouponValidationDecisionResponse {
    PortalCouponValidationDecisionResponse {
        eligible: decision.eligible,
        rejection_reason: decision.rejection_reason,
        reservable_budget_minor: decision.reservable_budget_minor,
    }
}

pub(crate) fn portal_marketing_operation_status(error: &MarketingOperationError) -> StatusCode {
    match error {
        MarketingOperationError::InvalidInput(_) => StatusCode::BAD_REQUEST,
        MarketingOperationError::NotFound(_) => StatusCode::NOT_FOUND,
        MarketingOperationError::Conflict(_) => StatusCode::CONFLICT,
        MarketingOperationError::Forbidden(_) => StatusCode::FORBIDDEN,
        MarketingOperationError::Storage(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub(crate) async fn portal_marketing_reservation_context_owned_by_subject(
    store: &dyn AdminStore,
    subjects: &MarketingSubjectSet,
    reservation_id: &str,
) -> Result<MarketingReservationOwnershipView, StatusCode> {
    load_coupon_reservation_context_owned_by_subject(store, subjects, reservation_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)
}

pub(crate) async fn portal_marketing_redemption_context_owned_by_subject(
    store: &dyn AdminStore,
    subjects: &MarketingSubjectSet,
    redemption_id: &str,
) -> Result<MarketingRedemptionOwnershipView, StatusCode> {
    load_coupon_redemption_context_owned_by_subject(store, subjects, redemption_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)
}

pub(crate) async fn load_marketing_redemptions_for_subject(
    store: &dyn AdminStore,
    subjects: &MarketingSubjectSet,
    status: Option<CouponRedemptionStatus>,
) -> Result<Vec<CouponRedemptionRecord>, StatusCode> {
    list_coupon_redemptions_for_subjects(store, subjects, status)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub(crate) async fn load_marketing_code_items(
    store: &dyn AdminStore,
    subjects: &MarketingSubjectSet,
) -> Result<Vec<PortalMarketingCodeItem>, StatusCode> {
    let now_ms = current_time_millis();
    let views = list_coupon_code_views_for_subjects(store, subjects, now_ms)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let mut items = Vec::with_capacity(views.len());
    for MarketingCodeView {
        context,
        latest_reservation,
        latest_redemption,
    } in views
    {
        items.push(PortalMarketingCodeItem {
            code: context.code.clone(),
            template: context.template.clone(),
            campaign: context.campaign.clone(),
            applicability: portal_coupon_applicability_summary(&context.template),
            effect: portal_coupon_effect_summary(&context.template),
            ownership: portal_coupon_ownership_summary(
                subjects,
                &context.code,
                latest_reservation.as_ref(),
                latest_redemption.as_ref(),
            ),
            latest_reservation,
            latest_redemption,
        });
    }
    Ok(items)
}

pub(crate) async fn load_marketing_reward_history_items(
    store: &dyn AdminStore,
    subjects: &MarketingSubjectSet,
    account_arrival: Option<&PortalCouponAccountArrivalContext>,
) -> Result<Vec<PortalMarketingRewardHistoryItem>, StatusCode> {
    let now_ms = current_time_millis();
    let views = list_coupon_reward_history_views_for_subjects(store, subjects, now_ms)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut items = Vec::new();
    for MarketingRewardHistoryView {
        context,
        redemption,
        rollbacks,
    } in views
    {
        let ownership =
            portal_coupon_ownership_summary(subjects, &context.code, None, Some(&redemption));
        let account_arrival_summary =
            portal_coupon_account_arrival_summary(&redemption, account_arrival);
        items.push(PortalMarketingRewardHistoryItem {
            redemption,
            code: context.code.clone(),
            template: context.template.clone(),
            campaign: context.campaign.clone(),
            applicability: portal_coupon_applicability_summary(&context.template),
            effect: portal_coupon_effect_summary(&context.template),
            ownership,
            account_arrival: account_arrival_summary,
            rollbacks,
        });
    }

    items.sort_by(|left, right| {
        right
            .redemption
            .redeemed_at_ms
            .cmp(&left.redemption.redeemed_at_ms)
            .then_with(|| {
                right
                    .redemption
                    .coupon_redemption_id
                    .cmp(&left.redemption.coupon_redemption_id)
            })
    });
    Ok(items)
}

pub(crate) fn summarize_marketing_redemptions(
    items: &[CouponRedemptionRecord],
) -> PortalMarketingRedemptionSummary {
    let MarketingRedemptionSummary {
        total_count,
        redeemed_count,
        partially_rolled_back_count,
        rolled_back_count,
        failed_count,
    } = summarize_coupon_redemptions(items);
    PortalMarketingRedemptionSummary {
        total_count,
        redeemed_count,
        partially_rolled_back_count,
        rolled_back_count,
        failed_count,
    }
}

pub(crate) fn summarize_marketing_code_items(
    items: &[PortalMarketingCodeItem],
) -> PortalMarketingCodeSummary {
    let codes = items
        .iter()
        .map(|item| item.code.clone())
        .collect::<Vec<_>>();
    let summary = summarize_coupon_codes(&codes);
    let MarketingCodeSummary {
        total_count,
        available_count,
        reserved_count,
        redeemed_count,
        disabled_count,
        expired_count,
    } = summary;
    PortalMarketingCodeSummary {
        total_count,
        available_count,
        reserved_count,
        redeemed_count,
        disabled_count,
        expired_count,
    }
}

fn portal_coupon_applicability_summary(
    template: &CouponTemplateRecord,
) -> PortalCouponApplicabilitySummary {
    PortalCouponApplicabilitySummary {
        target_kinds: template.restriction.eligible_target_kinds.clone(),
        all_target_kinds_eligible: template.restriction.eligible_target_kinds.is_empty(),
    }
}

fn portal_coupon_effect_summary(template: &CouponTemplateRecord) -> PortalCouponEffectSummary {
    PortalCouponEffectSummary {
        effect_kind: if template.benefit.grant_units.is_some() {
            PortalCouponEffectKind::AccountEntitlement
        } else {
            PortalCouponEffectKind::CheckoutDiscount
        },
        discount_percent: template.benefit.discount_percent,
        discount_amount_minor: template.benefit.discount_amount_minor,
        grant_units: template.benefit.grant_units,
    }
}

fn portal_coupon_ownership_summary(
    subjects: &MarketingSubjectSet,
    code: &CouponCodeRecord,
    latest_reservation: Option<&CouponReservationRecord>,
    latest_redemption: Option<&CouponRedemptionRecord>,
) -> PortalCouponOwnershipSummary {
    let claimed_to_current_subject = code
        .claimed_subject_id
        .as_deref()
        .zip(code.claimed_subject_scope)
        .is_some_and(|(subject_id, scope)| subjects.matches(scope, subject_id));

    PortalCouponOwnershipSummary {
        owned_by_current_subject: claimed_to_current_subject
            || latest_reservation.is_some()
            || latest_redemption.is_some(),
        claimed_to_current_subject,
        claimed_subject_scope: code.claimed_subject_scope,
        claimed_subject_id: code.claimed_subject_id.clone(),
    }
}

fn portal_coupon_account_arrival_summary(
    redemption: &CouponRedemptionRecord,
    context: Option<&PortalCouponAccountArrivalContext>,
) -> PortalCouponAccountArrivalSummary {
    let order_id = redemption
        .order_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned);
    let mut lots = order_id
        .as_deref()
        .and_then(|value| context.and_then(|ctx| ctx.lots_by_order_id.get(value)))
        .cloned()
        .unwrap_or_default();
    lots.sort_by(|left, right| {
        right
            .issued_at_ms
            .cmp(&left.issued_at_ms)
            .then_with(|| left.lot_id.cmp(&right.lot_id))
    });

    let credited_quantity = lots.iter().map(|lot| lot.original_quantity).sum::<f64>();
    let benefit_lots = lots
        .iter()
        .map(portal_coupon_account_arrival_lot_item)
        .collect::<Vec<_>>();

    PortalCouponAccountArrivalSummary {
        order_id,
        account_id: context.and_then(|ctx| ctx.account_id),
        benefit_lot_count: benefit_lots.len(),
        credited_quantity,
        benefit_lots,
    }
}

fn portal_coupon_account_arrival_lot_item(
    lot: &AccountBenefitLotRecord,
) -> PortalCouponAccountArrivalLotItem {
    PortalCouponAccountArrivalLotItem {
        lot_id: lot.lot_id,
        benefit_type: lot.benefit_type,
        source_type: lot.source_type,
        source_id: lot.source_id,
        status: lot.status,
        original_quantity: lot.original_quantity,
        remaining_quantity: lot.remaining_quantity,
        issued_at_ms: lot.issued_at_ms,
        expires_at_ms: lot.expires_at_ms,
        scope_order_id: parse_scope_order_id(lot.scope_json.as_deref()),
    }
}

fn parse_scope_order_id(scope_json: Option<&str>) -> Option<String> {
    let scope_json = scope_json?.trim();
    if scope_json.is_empty() {
        return None;
    }

    let parsed: serde_json::Value = serde_json::from_str(scope_json).ok()?;
    let order_id = parsed.get("order_id")?.as_str()?.trim();
    if order_id.is_empty() {
        None
    } else {
        Some(order_id.to_owned())
    }
}

fn normalize_portal_idempotency_header_value(
    value: Option<&HeaderValue>,
) -> Result<Option<&str>, StatusCode> {
    let Some(value) = value else {
        return Ok(None);
    };
    value
        .to_str()
        .map(Some)
        .map_err(|_| StatusCode::BAD_REQUEST)
}

pub(crate) fn resolve_portal_idempotency_key(
    headers: &HeaderMap,
    body_value: Option<&str>,
) -> Result<Option<String>, StatusCode> {
    let header_value = normalize_portal_idempotency_header_value(headers.get("idempotency-key"))?;
    resolve_shared_idempotency_key(header_value, body_value).map_err(|_| StatusCode::BAD_REQUEST)
}

pub(crate) async fn enforce_portal_coupon_rate_limit(
    store: &dyn AdminStore,
    project_id: &str,
    action: CouponRateLimitAction,
    subject_scope: MarketingSubjectScope,
    subject_id: &str,
    coupon_code: &str,
) -> Result<(), StatusCode> {
    let actor_bucket =
        coupon_actor_bucket(marketing_subject_scope_token(subject_scope), subject_id);
    let evaluation = check_coupon_rate_limit(
        store,
        project_id,
        action,
        Some(actor_bucket.as_str()),
        Some(coupon_code),
        1,
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if evaluation.allowed {
        Ok(())
    } else {
        Err(StatusCode::TOO_MANY_REQUESTS)
    }
}
