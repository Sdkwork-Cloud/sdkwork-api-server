use super::*;

#[path = "marketing_handlers.rs"]
mod handlers;

pub(crate) use handlers::{
    confirm_marketing_coupon_redemption_handler,
    list_marketing_codes_handler,
    list_marketing_redemptions_handler,
    list_marketing_reward_history_handler,
    list_my_coupons_handler,
    reserve_marketing_coupon_handler,
    rollback_marketing_coupon_redemption_handler,
    validate_marketing_coupon_handler,
};
#[derive(Debug, Clone)]
struct MarketingCouponContext {
    template: CouponTemplateRecord,
    campaign: MarketingCampaignRecord,
    budget: CampaignBudgetRecord,
    code: CouponCodeRecord,
}

#[derive(Debug, Clone)]
struct PortalMarketingSubjectSet {
    user_id: String,
    project_id: String,
    workspace_id: String,
}

#[derive(Debug, Clone, Default)]
struct PortalCouponAccountArrivalContext {
    account_id: Option<u64>,
    lots_by_order_id: HashMap<String, Vec<AccountBenefitLotRecord>>,
}

impl PortalMarketingSubjectSet {
    fn new(workspace: &PortalWorkspaceSummary, claims: &PortalClaims) -> Self {
        Self {
            user_id: claims.sub.clone(),
            project_id: workspace.project.id.clone(),
            workspace_id: format!("{}:{}", workspace.tenant.id, workspace.project.id),
        }
    }

    fn subject_id_for_scope(&self, scope: MarketingSubjectScope) -> Option<String> {
        match scope {
            MarketingSubjectScope::User => Some(self.user_id.clone()),
            MarketingSubjectScope::Project => Some(self.project_id.clone()),
            MarketingSubjectScope::Workspace => Some(self.workspace_id.clone()),
            MarketingSubjectScope::Account => None,
        }
    }

    fn matches(&self, scope: MarketingSubjectScope, subject_id: &str) -> bool {
        match scope {
            MarketingSubjectScope::User => self.user_id == subject_id,
            MarketingSubjectScope::Project => self.project_id == subject_id,
            MarketingSubjectScope::Workspace => self.workspace_id == subject_id,
            MarketingSubjectScope::Account => false,
        }
    }
}

impl PortalCouponAccountArrivalContext {
    fn from_account_lots(account_id: u64, lots: Vec<AccountBenefitLotRecord>) -> Self {
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

fn coupon_validation_decision_response(
    decision: CouponValidationDecision,
) -> PortalCouponValidationDecisionResponse {
    PortalCouponValidationDecisionResponse {
        eligible: decision.eligible,
        rejection_reason: decision.rejection_reason,
        reservable_budget_minor: decision.reservable_budget_minor,
    }
}

fn portal_marketing_target_kind_allowed(
    template: &CouponTemplateRecord,
    target_kind: &str,
) -> bool {
    template.restriction.eligible_target_kinds.is_empty()
        || template
            .restriction
            .eligible_target_kinds
            .iter()
            .any(|eligible| eligible == target_kind)
}

async fn load_marketing_coupon_context_by_value(
    store: &dyn AdminStore,
    code: &str,
    now_ms: u64,
) -> Result<Option<MarketingCouponContext>, StatusCode> {
    let normalized = normalize_coupon_code(code);
    reclaim_expired_coupon_reservations_for_code_if_needed(store, &normalized, now_ms)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if let Some(code_record) = store
        .find_coupon_code_record_by_value(&normalized)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    {
        if let Some(context) =
            load_marketing_coupon_context_from_code_record(store, code_record, now_ms).await?
        {
            return Ok(Some(context));
        }
    }

    Ok(None)
}

async fn load_marketing_coupon_context_from_code_record(
    store: &dyn AdminStore,
    code: CouponCodeRecord,
    now_ms: u64,
) -> Result<Option<MarketingCouponContext>, StatusCode> {
    let Some(template) = store
        .find_coupon_template_record(&code.coupon_template_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    else {
        return Ok(None);
    };

    let Some(campaign) = select_effective_marketing_campaign(
        store
            .list_marketing_campaign_records_for_template(&template.coupon_template_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
        now_ms,
    ) else {
        return Ok(None);
    };

    let Some(budget) = select_campaign_budget_record(
        store
            .list_campaign_budget_records_for_campaign(&campaign.marketing_campaign_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
    ) else {
        return Ok(None);
    };

    Ok(Some(MarketingCouponContext {
        template,
        campaign,
        budget,
        code,
    }))
}

async fn load_marketing_coupon_context_for_code_id(
    store: &dyn AdminStore,
    coupon_code_id: &str,
    now_ms: u64,
) -> Result<MarketingCouponContext, StatusCode> {
    let Some(code) = store
        .find_coupon_code_record(coupon_code_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    else {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };

    load_marketing_coupon_context_from_code_record(store, code, now_ms)
        .await?
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)
}

async fn load_marketing_coupon_context_for_portal_code(
    store: &dyn AdminStore,
    code: &CouponCodeRecord,
    now_ms: u64,
) -> Result<Option<MarketingCouponContext>, StatusCode> {
    if let Some(context) =
        load_marketing_coupon_context_from_code_record(store, code.clone(), now_ms).await?
    {
        return Ok(Some(context));
    }

    load_marketing_coupon_context_by_value(store, &code.code_value, now_ms).await
}

async fn find_coupon_rollback_record(
    store: &dyn AdminStore,
    rollback_id: &str,
) -> Result<Option<CouponRollbackRecord>, StatusCode> {
    Ok(store
        .list_coupon_rollback_records()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .into_iter()
        .find(|rollback| rollback.coupon_rollback_id == rollback_id))
}

async fn portal_marketing_reservation_owned_by_subject(
    store: &dyn AdminStore,
    subjects: &PortalMarketingSubjectSet,
    reservation_id: &str,
) -> Result<CouponReservationRecord, StatusCode> {
    let Some(reservation) = store
        .find_coupon_reservation_record(reservation_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    else {
        return Err(StatusCode::NOT_FOUND);
    };

    if !subjects.matches(reservation.subject_scope, &reservation.subject_id) {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(reservation)
}

async fn portal_marketing_redemption_owned_by_subject(
    store: &dyn AdminStore,
    subjects: &PortalMarketingSubjectSet,
    redemption_id: &str,
) -> Result<CouponRedemptionRecord, StatusCode> {
    let Some(redemption) = store
        .find_coupon_redemption_record(redemption_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    else {
        return Err(StatusCode::NOT_FOUND);
    };

    let reservation = portal_marketing_reservation_owned_by_subject(
        store,
        subjects,
        &redemption.coupon_reservation_id,
    )
    .await?;
    if reservation.coupon_reservation_id != redemption.coupon_reservation_id {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(redemption)
}

async fn load_marketing_redemptions_for_subject(
    store: &dyn AdminStore,
    subjects: &PortalMarketingSubjectSet,
    status: Option<CouponRedemptionStatus>,
) -> Result<Vec<CouponRedemptionRecord>, StatusCode> {
    let reservations = store
        .list_coupon_reservation_records()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let reservation_ids = reservations
        .into_iter()
        .filter(|reservation| subjects.matches(reservation.subject_scope, &reservation.subject_id))
        .map(|reservation| reservation.coupon_reservation_id)
        .collect::<HashSet<_>>();

    let mut redemptions = store
        .list_coupon_redemption_records()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .into_iter()
        .filter(|redemption| reservation_ids.contains(&redemption.coupon_reservation_id))
        .filter(|redemption| status.is_none_or(|expected| redemption.redemption_status == expected))
        .collect::<Vec<_>>();

    redemptions.sort_by(|left, right| {
        right
            .redeemed_at_ms
            .cmp(&left.redeemed_at_ms)
            .then_with(|| right.coupon_redemption_id.cmp(&left.coupon_redemption_id))
    });
    Ok(redemptions)
}

async fn load_marketing_code_items(
    store: &dyn AdminStore,
    subjects: &PortalMarketingSubjectSet,
) -> Result<Vec<PortalMarketingCodeItem>, StatusCode> {
    let now_ms = current_time_millis();
    let reservations = store
        .list_coupon_reservation_records()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .into_iter()
        .filter(|reservation| subjects.matches(reservation.subject_scope, &reservation.subject_id))
        .collect::<Vec<_>>();

    let reservation_ids = reservations
        .iter()
        .map(|reservation| reservation.coupon_reservation_id.clone())
        .collect::<HashSet<_>>();
    let redemptions = store
        .list_coupon_redemption_records()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .into_iter()
        .filter(|redemption| reservation_ids.contains(&redemption.coupon_reservation_id))
        .collect::<Vec<_>>();
    let codes = store
        .list_coupon_code_records()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut latest_reservations = HashMap::new();
    for reservation in reservations {
        latest_reservations
            .entry(reservation.coupon_code_id.clone())
            .and_modify(|current: &mut CouponReservationRecord| {
                if reservation.updated_at_ms > current.updated_at_ms
                    || (reservation.updated_at_ms == current.updated_at_ms
                        && reservation.coupon_reservation_id > current.coupon_reservation_id)
                {
                    *current = reservation.clone();
                }
            })
            .or_insert(reservation);
    }

    let mut latest_redemptions = HashMap::new();
    for redemption in redemptions {
        latest_redemptions
            .entry(redemption.coupon_code_id.clone())
            .and_modify(|current: &mut CouponRedemptionRecord| {
                if redemption.updated_at_ms > current.updated_at_ms
                    || (redemption.updated_at_ms == current.updated_at_ms
                        && redemption.coupon_redemption_id > current.coupon_redemption_id)
                {
                    *current = redemption.clone();
                }
            })
            .or_insert(redemption);
    }

    let mut related_code_ids = latest_reservations.keys().cloned().collect::<HashSet<_>>();
    related_code_ids.extend(latest_redemptions.keys().cloned());

    let mut items = codes
        .into_iter()
        .filter(|code| {
            related_code_ids.contains(&code.coupon_code_id)
                || code
                    .claimed_subject_id
                    .as_deref()
                    .zip(code.claimed_subject_scope)
                    .is_some_and(|(subject_id, scope)| subjects.matches(scope, subject_id))
        })
        .filter_map(|code| {
            let latest_reservation = latest_reservations.get(&code.coupon_code_id).cloned();
            let latest_redemption = latest_redemptions.get(&code.coupon_code_id).cloned();
            Some((code, latest_reservation, latest_redemption))
        })
        .collect::<Vec<_>>();

    let mut enriched_items = Vec::with_capacity(items.len());
    for (code, latest_reservation, latest_redemption) in items.drain(..) {
        let Some(context) =
            load_marketing_coupon_context_for_portal_code(store, &code, now_ms).await?
        else {
            continue;
        };

        enriched_items.push(PortalMarketingCodeItem {
            code: code.clone(),
            template: context.template.clone(),
            campaign: context.campaign.clone(),
            applicability: portal_coupon_applicability_summary(&context.template),
            effect: portal_coupon_effect_summary(&context.template),
            ownership: portal_coupon_ownership_summary(
                subjects,
                &code,
                latest_reservation.as_ref(),
                latest_redemption.as_ref(),
            ),
            latest_reservation,
            latest_redemption,
        });
    }

    enriched_items.sort_by(|left, right| {
        right
            .code
            .updated_at_ms
            .cmp(&left.code.updated_at_ms)
            .then_with(|| right.code.coupon_code_id.cmp(&left.code.coupon_code_id))
    });
    Ok(enriched_items)
}

async fn load_marketing_reward_history_items(
    store: &dyn AdminStore,
    subjects: &PortalMarketingSubjectSet,
    account_arrival: Option<&PortalCouponAccountArrivalContext>,
) -> Result<Vec<PortalMarketingRewardHistoryItem>, StatusCode> {
    let now_ms = current_time_millis();
    let redemptions = load_marketing_redemptions_for_subject(store, subjects, None).await?;
    let codes = store
        .list_coupon_code_records()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .into_iter()
        .map(|code| (code.coupon_code_id.clone(), code))
        .collect::<HashMap<_, _>>();
    let mut rollbacks_by_redemption = HashMap::new();
    for rollback in store
        .list_coupon_rollback_records()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    {
        rollbacks_by_redemption
            .entry(rollback.coupon_redemption_id.clone())
            .or_insert_with(Vec::new)
            .push(rollback);
    }

    let mut items = Vec::new();
    for redemption in redemptions {
        let Some(code) = codes.get(&redemption.coupon_code_id).cloned() else {
            continue;
        };
        let Some(context) =
            load_marketing_coupon_context_for_portal_code(store, &code, now_ms).await?
        else {
            continue;
        };
        let mut rollbacks = rollbacks_by_redemption
            .remove(&redemption.coupon_redemption_id)
            .unwrap_or_default();
        rollbacks.sort_by(|left, right| {
            right
                .created_at_ms
                .cmp(&left.created_at_ms)
                .then_with(|| right.coupon_rollback_id.cmp(&left.coupon_rollback_id))
        });
        let ownership =
            portal_coupon_ownership_summary(subjects, &code, None, Some(&redemption));
        let account_arrival_summary =
            portal_coupon_account_arrival_summary(&redemption, account_arrival);
        items.push(PortalMarketingRewardHistoryItem {
            redemption,
            code: code.clone(),
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

fn summarize_marketing_redemptions(
    items: &[CouponRedemptionRecord],
) -> PortalMarketingRedemptionSummary {
    let mut summary = PortalMarketingRedemptionSummary {
        total_count: items.len(),
        ..PortalMarketingRedemptionSummary::default()
    };
    for item in items {
        match item.redemption_status {
            CouponRedemptionStatus::Redeemed => summary.redeemed_count += 1,
            CouponRedemptionStatus::PartiallyRolledBack => summary.partially_rolled_back_count += 1,
            CouponRedemptionStatus::RolledBack => summary.rolled_back_count += 1,
            CouponRedemptionStatus::Failed => summary.failed_count += 1,
            CouponRedemptionStatus::Pending => {}
        }
    }
    summary
}

fn summarize_marketing_code_items(items: &[PortalMarketingCodeItem]) -> PortalMarketingCodeSummary {
    let mut summary = PortalMarketingCodeSummary {
        total_count: items.len(),
        ..PortalMarketingCodeSummary::default()
    };
    for item in items {
        match item.code.status {
            CouponCodeStatus::Available => summary.available_count += 1,
            CouponCodeStatus::Reserved => summary.reserved_count += 1,
            CouponCodeStatus::Redeemed => summary.redeemed_count += 1,
            CouponCodeStatus::Disabled => summary.disabled_count += 1,
            CouponCodeStatus::Expired => summary.expired_count += 1,
        }
    }
    summary
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
    subjects: &PortalMarketingSubjectSet,
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

fn select_effective_marketing_campaign(
    campaigns: Vec<MarketingCampaignRecord>,
    now_ms: u64,
) -> Option<MarketingCampaignRecord> {
    campaigns
        .into_iter()
        .filter(|campaign| campaign.is_effective_at(now_ms))
        .max_by(|left, right| {
            left.updated_at_ms
                .cmp(&right.updated_at_ms)
                .then_with(|| left.marketing_campaign_id.cmp(&right.marketing_campaign_id))
        })
}

fn select_campaign_budget_record(
    budgets: Vec<CampaignBudgetRecord>,
) -> Option<CampaignBudgetRecord> {
    budgets.into_iter().max_by(|left, right| {
        left.updated_at_ms
            .cmp(&right.updated_at_ms)
            .then_with(|| left.campaign_budget_id.cmp(&right.campaign_budget_id))
    })
}

fn normalize_coupon_code(value: &str) -> String {
    value.trim().to_ascii_uppercase()
}

fn normalize_portal_idempotency_key(value: Option<&str>) -> Result<Option<String>, StatusCode> {
    let Some(value) = value.map(str::trim) else {
        return Ok(None);
    };
    if value.is_empty() || value.len() > 128 || value.chars().any(|ch| ch.is_control()) {
        return Err(StatusCode::BAD_REQUEST);
    }

    Ok(Some(value.to_owned()))
}

fn normalize_portal_idempotency_header_value(
    value: Option<&HeaderValue>,
) -> Result<Option<String>, StatusCode> {
    let Some(value) = value else {
        return Ok(None);
    };
    let value = value.to_str().map_err(|_| StatusCode::BAD_REQUEST)?;
    normalize_portal_idempotency_key(Some(value))
}

fn resolve_portal_idempotency_key(
    headers: &HeaderMap,
    body_value: Option<&str>,
) -> Result<Option<String>, StatusCode> {
    let body_value = normalize_portal_idempotency_key(body_value)?;
    let header_value = normalize_portal_idempotency_header_value(headers.get("idempotency-key"))?;
    match (body_value, header_value) {
        (Some(body_value), Some(header_value)) if body_value != header_value => {
            Err(StatusCode::BAD_REQUEST)
        }
        (Some(body_value), Some(_)) | (Some(body_value), None) => Ok(Some(body_value)),
        (None, Some(header_value)) => Ok(Some(header_value)),
        (None, None) => Ok(None),
    }
}

fn marketing_subject_scope_token(scope: MarketingSubjectScope) -> &'static str {
    match scope {
        MarketingSubjectScope::User => "user",
        MarketingSubjectScope::Project => "project",
        MarketingSubjectScope::Workspace => "workspace",
        MarketingSubjectScope::Account => "account",
    }
}

async fn enforce_portal_coupon_rate_limit(
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

fn marketing_idempotency_fingerprint(
    operation: &str,
    subject_scope: MarketingSubjectScope,
    subject_id: &str,
    idempotency_key: &str,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(operation.as_bytes());
    hasher.update([0x1f]);
    hasher.update(marketing_subject_scope_token(subject_scope).as_bytes());
    hasher.update([0x1f]);
    hasher.update(subject_id.as_bytes());
    hasher.update([0x1f]);
    hasher.update(idempotency_key.as_bytes());

    let digest = hasher.finalize();
    let mut fingerprint = String::with_capacity(32);
    for byte in digest.iter().take(16) {
        let _ = write!(&mut fingerprint, "{byte:02x}");
    }
    fingerprint
}

fn derive_coupon_reservation_id(
    subject_scope: MarketingSubjectScope,
    subject_id: &str,
    target_kind: &str,
    idempotency_key: &str,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update("reserve".as_bytes());
    hasher.update([0x1f]);
    hasher.update(marketing_subject_scope_token(subject_scope).as_bytes());
    hasher.update([0x1f]);
    hasher.update(subject_id.as_bytes());
    hasher.update([0x1f]);
    hasher.update(target_kind.as_bytes());
    hasher.update([0x1f]);
    hasher.update(idempotency_key.as_bytes());

    let digest = hasher.finalize();
    let mut fingerprint = String::with_capacity(32);
    for byte in digest.iter().take(16) {
        let _ = write!(&mut fingerprint, "{byte:02x}");
    }

    format!("coupon_reservation_{fingerprint}",)
}

fn derive_coupon_redemption_id(
    reservation: &CouponReservationRecord,
    idempotency_key: &str,
) -> String {
    format!(
        "coupon_redemption_{}",
        marketing_idempotency_fingerprint(
            "confirm",
            reservation.subject_scope,
            &reservation.subject_id,
            idempotency_key,
        )
    )
}

fn derive_coupon_rollback_id(
    reservation: &CouponReservationRecord,
    idempotency_key: &str,
) -> String {
    format!(
        "coupon_rollback_{}",
        marketing_idempotency_fingerprint(
            "rollback",
            reservation.subject_scope,
            &reservation.subject_id,
            idempotency_key,
        )
    )
}

fn coupon_code_is_exclusive(template: &CouponTemplateRecord) -> bool {
    !matches!(
        template.distribution_kind,
        CouponDistributionKind::SharedCode
    )
}

fn code_after_reservation(
    template: &CouponTemplateRecord,
    original_code: &CouponCodeRecord,
    reserved_code: &CouponCodeRecord,
    now_ms: u64,
) -> CouponCodeRecord {
    if coupon_code_is_exclusive(template) {
        reserved_code.clone()
    } else {
        original_code.clone().with_updated_at_ms(now_ms)
    }
}

fn code_after_confirmation(
    template: &CouponTemplateRecord,
    original_code: &CouponCodeRecord,
    now_ms: u64,
) -> CouponCodeRecord {
    if coupon_code_is_exclusive(template) {
        original_code
            .clone()
            .with_status(CouponCodeStatus::Redeemed)
            .with_updated_at_ms(now_ms)
    } else {
        original_code.clone().with_updated_at_ms(now_ms)
    }
}

fn code_after_rollback(
    template: &CouponTemplateRecord,
    original_code: &CouponCodeRecord,
    now_ms: u64,
) -> CouponCodeRecord {
    if coupon_code_is_exclusive(template) {
        restore_coupon_code_availability(original_code, now_ms)
    } else {
        original_code.clone().with_updated_at_ms(now_ms)
    }
}

fn restore_coupon_code_availability(
    original_code: &CouponCodeRecord,
    now_ms: u64,
) -> CouponCodeRecord {
    let next_status = if original_code
        .expires_at_ms
        .is_some_and(|value| now_ms > value)
    {
        CouponCodeStatus::Expired
    } else {
        CouponCodeStatus::Available
    };
    original_code
        .clone()
        .with_status(next_status)
        .with_updated_at_ms(now_ms)
}

fn reserve_campaign_budget(
    budget: &CampaignBudgetRecord,
    reserved_amount_minor: u64,
    now_ms: u64,
) -> CampaignBudgetRecord {
    let next_reserved = budget
        .reserved_budget_minor
        .saturating_add(reserved_amount_minor);
    budget
        .clone()
        .with_reserved_budget_minor(next_reserved)
        .with_status(campaign_budget_status_after_mutation(
            budget.total_budget_minor,
            next_reserved,
            budget.consumed_budget_minor,
            budget.status,
        ))
        .with_updated_at_ms(now_ms)
}

fn confirm_campaign_budget(
    budget: &CampaignBudgetRecord,
    consumed_amount_minor: u64,
    now_ms: u64,
) -> CampaignBudgetRecord {
    let next_reserved = budget
        .reserved_budget_minor
        .saturating_sub(consumed_amount_minor);
    let next_consumed = budget
        .consumed_budget_minor
        .saturating_add(consumed_amount_minor);
    budget
        .clone()
        .with_reserved_budget_minor(next_reserved)
        .with_consumed_budget_minor(next_consumed)
        .with_status(campaign_budget_status_after_mutation(
            budget.total_budget_minor,
            next_reserved,
            next_consumed,
            budget.status,
        ))
        .with_updated_at_ms(now_ms)
}

fn rollback_campaign_budget(
    budget: &CampaignBudgetRecord,
    restored_amount_minor: u64,
    now_ms: u64,
) -> CampaignBudgetRecord {
    let next_consumed = budget
        .consumed_budget_minor
        .saturating_sub(restored_amount_minor);
    budget
        .clone()
        .with_consumed_budget_minor(next_consumed)
        .with_status(campaign_budget_status_after_mutation(
            budget.total_budget_minor,
            budget.reserved_budget_minor,
            next_consumed,
            budget.status,
        ))
        .with_updated_at_ms(now_ms)
}

fn campaign_budget_status_after_mutation(
    total_budget_minor: u64,
    reserved_budget_minor: u64,
    consumed_budget_minor: u64,
    prior_status: CampaignBudgetStatus,
) -> CampaignBudgetStatus {
    if matches!(
        prior_status,
        CampaignBudgetStatus::Closed | CampaignBudgetStatus::Draft
    ) {
        return prior_status;
    }

    let available_budget_minor = total_budget_minor
        .saturating_sub(reserved_budget_minor)
        .saturating_sub(consumed_budget_minor);
    if available_budget_minor == 0 {
        CampaignBudgetStatus::Exhausted
    } else {
        CampaignBudgetStatus::Active
    }
}



