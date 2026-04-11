use super::*;

pub(crate) async fn load_coupon_catalog(
    store: &dyn AdminStore,
) -> CommerceResult<Vec<PortalCommerceCoupon>> {
    Ok(load_coupon_definitions(store)
        .await?
        .into_iter()
        .map(|definition| definition.coupon)
        .collect())
}

async fn load_coupon_definitions(
    store: &dyn AdminStore,
) -> CommerceResult<Vec<CommerceCouponDefinition>> {
    let mut definitions = seed_coupon_definitions()
        .into_iter()
        .map(|definition| (normalize_coupon_code(&definition.coupon.code), definition))
        .collect::<BTreeMap<_, _>>();
    let now_ms = current_time_ms()?;

    for definition in load_marketing_coupon_definitions(store, now_ms).await? {
        definitions.insert(normalize_coupon_code(&definition.coupon.code), definition);
    }

    Ok(definitions.into_values().collect())
}

pub(crate) async fn find_resolved_coupon_definition(
    store: &dyn AdminStore,
    code: &str,
) -> CommerceResult<ResolvedCouponDefinition> {
    let normalized = normalize_coupon_code(code);
    let now_ms = current_time_ms()?;
    if let Some(context) =
        load_marketing_coupon_context_by_value(store, &normalized, now_ms).await?
    {
        if !coupon_context_is_catalog_visible(&context, now_ms) {
            return Err(CommerceError::NotFound(format!(
                "coupon {normalized} not found"
            )));
        }
        return Ok(ResolvedCouponDefinition {
            definition: marketing_context_to_definition(&context, now_ms),
            marketing: Some(context),
        });
    }

    load_coupon_definitions(store)
        .await?
        .into_iter()
        .find(|definition| definition.coupon.code == normalized)
        .map(|definition| ResolvedCouponDefinition {
            definition,
            marketing: None,
        })
        .ok_or_else(|| CommerceError::NotFound(format!("coupon {normalized} not found")))
}

pub(crate) async fn load_optional_applied_coupon(
    store: &dyn AdminStore,
    coupon_code: Option<&str>,
    target_kind: &str,
    order_amount_cents: u64,
) -> CommerceResult<Option<ResolvedCouponDefinition>> {
    match coupon_code.map(str::trim).filter(|value| !value.is_empty()) {
        Some(code) => {
            let resolved = find_resolved_coupon_definition(store, code).await?;
            if let Some(context) = resolved.marketing.as_ref() {
                let reserve_amount_minor = compute_coupon_reserve_amount_minor(
                    order_amount_cents,
                    &context.template.benefit,
                );
                let decision = validate_marketing_coupon_context(
                    context,
                    target_kind,
                    current_time_ms()?,
                    order_amount_cents,
                    reserve_amount_minor,
                );
                if !decision.eligible {
                    return Err(CommerceError::InvalidInput(format!(
                        "coupon {} is not eligible: {}",
                        resolved.definition.coupon.code,
                        decision
                            .rejection_reason
                            .unwrap_or_else(|| "validation_failed".to_owned())
                    )));
                }
            }
            Ok(Some(resolved))
        }
        None => Ok(None),
    }
}

async fn load_marketing_coupon_definitions(
    store: &dyn AdminStore,
    now_ms: u64,
) -> CommerceResult<Vec<CommerceCouponDefinition>> {
    let mut definitions = Vec::new();
    for code_record in store
        .list_coupon_code_records()
        .await
        .map_err(CommerceError::from)?
    {
        reclaim_expired_coupon_reservations_for_code_if_needed(
            store,
            &code_record.code_value,
            now_ms,
        )
        .await?;
        let Some(current_code_record) = store
            .find_coupon_code_record(&code_record.coupon_code_id)
            .await
            .map_err(CommerceError::from)?
        else {
            continue;
        };
        if let Some(context) =
            load_marketing_coupon_context_from_code_record(store, current_code_record, now_ms)
                .await?
        {
            if coupon_context_is_catalog_visible(&context, now_ms) {
                definitions.push(marketing_context_to_definition(&context, now_ms));
            }
        }
    }
    Ok(definitions)
}

async fn load_marketing_coupon_context_by_value(
    store: &dyn AdminStore,
    code: &str,
    now_ms: u64,
) -> CommerceResult<Option<MarketingCouponContext>> {
    let normalized = normalize_coupon_code(code);
    reclaim_expired_coupon_reservations_for_code_if_needed(store, &normalized, now_ms).await?;
    if let Some(code_record) = store
        .find_coupon_code_record_by_value(&normalized)
        .await
        .map_err(CommerceError::from)?
    {
        if let Some(context) =
            load_marketing_coupon_context_from_code_record(store, code_record, now_ms).await?
        {
            return Ok(Some(context));
        }
    }

    Ok(None)
}

pub async fn reclaim_expired_coupon_reservations_for_code_if_needed(
    store: &dyn AdminStore,
    code: &str,
    now_ms: u64,
) -> CommerceResult<u64> {
    let normalized = normalize_coupon_code(code);
    let Some(code_record) = store
        .find_coupon_code_record_by_value(&normalized)
        .await
        .map_err(CommerceError::from)?
    else {
        return Ok(0);
    };

    let mut expired_reservation_ids = store
        .list_coupon_reservation_records()
        .await
        .map_err(CommerceError::from)?
        .into_iter()
        .filter(|reservation| {
            reservation.coupon_code_id == code_record.coupon_code_id
                && reservation.reservation_status == CouponReservationStatus::Reserved
                && reservation.expires_at_ms < now_ms
        })
        .map(|reservation| reservation.coupon_reservation_id)
        .collect::<Vec<_>>();
    expired_reservation_ids.sort();

    let mut reclaimed = 0;
    for reservation_id in expired_reservation_ids {
        let Some(reservation) = store
            .find_coupon_reservation_record(&reservation_id)
            .await
            .map_err(CommerceError::from)?
        else {
            continue;
        };
        if reservation.reservation_status != CouponReservationStatus::Reserved
            || reservation.expires_at_ms >= now_ms
        {
            continue;
        }

        let Some(current_code) = store
            .find_coupon_code_record(&reservation.coupon_code_id)
            .await
            .map_err(CommerceError::from)?
        else {
            continue;
        };
        let Some(context) =
            load_marketing_coupon_context_from_code_record(store, current_code, now_ms).await?
        else {
            continue;
        };

        store
            .release_coupon_reservation_atomic(&AtomicCouponReleaseCommand {
                expected_budget: context.budget.clone(),
                next_budget: release_campaign_budget(
                    &context.budget,
                    reservation.budget_reserved_minor,
                    now_ms,
                ),
                expected_code: context.code.clone(),
                next_code: code_after_release(&context.template, &context.code, now_ms),
                expected_reservation: reservation.clone(),
                next_reservation: reservation
                    .with_status(CouponReservationStatus::Expired)
                    .with_updated_at_ms(now_ms),
            })
            .await
            .map_err(commerce_atomic_coupon_error)?;
        reclaimed += 1;
    }

    Ok(reclaimed)
}

async fn load_marketing_coupon_context_from_code_record(
    store: &dyn AdminStore,
    code: CouponCodeRecord,
    now_ms: u64,
) -> CommerceResult<Option<MarketingCouponContext>> {
    let Some(template) = store
        .find_coupon_template_record(&code.coupon_template_id)
        .await
        .map_err(CommerceError::from)?
    else {
        return Ok(None);
    };

    let Some(campaign) = select_effective_marketing_campaign(
        store
            .list_marketing_campaign_records_for_template(&template.coupon_template_id)
            .await
            .map_err(CommerceError::from)?,
        now_ms,
    ) else {
        return Ok(None);
    };

    let Some(budget) = select_campaign_budget_record(
        store
            .list_campaign_budget_records_for_campaign(&campaign.marketing_campaign_id)
            .await
            .map_err(CommerceError::from)?,
    ) else {
        return Ok(None);
    };

    Ok(Some(MarketingCouponContext {
        template,
        campaign,
        budget,
        code,
        source: "marketing".to_owned(),
    }))
}

pub(crate) fn select_effective_marketing_campaign(
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

pub(crate) fn select_campaign_budget_record(
    budgets: Vec<CampaignBudgetRecord>,
) -> Option<CampaignBudgetRecord> {
    budgets.into_iter().max_by(|left, right| {
        left.updated_at_ms
            .cmp(&right.updated_at_ms)
            .then_with(|| left.campaign_budget_id.cmp(&right.campaign_budget_id))
    })
}

fn coupon_context_is_catalog_visible(context: &MarketingCouponContext, now_ms: u64) -> bool {
    context.template.status == CouponTemplateStatus::Active
        && context.campaign.is_effective_at(now_ms)
        && context.budget.available_budget_minor() > 0
        && coupon_code_is_available_for_template(&context.template, &context.code, now_ms)
}

fn coupon_code_is_available_for_template(
    template: &CouponTemplateRecord,
    code: &CouponCodeRecord,
    now_ms: u64,
) -> bool {
    match template.distribution_kind {
        CouponDistributionKind::SharedCode => {
            !matches!(
                code.status,
                CouponCodeStatus::Disabled | CouponCodeStatus::Expired
            ) && code.expires_at_ms.is_none_or(|value| now_ms <= value)
        }
        CouponDistributionKind::UniqueCode | CouponDistributionKind::AutoClaim => {
            code.is_redeemable_at(now_ms)
        }
    }
}

fn marketing_context_to_definition(
    context: &MarketingCouponContext,
    now_ms: u64,
) -> CommerceCouponDefinition {
    let benefit = CommerceCouponBenefit {
        discount_percent: context.template.benefit.discount_percent,
        bonus_units: context.template.benefit.grant_units.unwrap_or(0),
    };
    let remaining = match context.template.distribution_kind {
        CouponDistributionKind::SharedCode => context.budget.available_budget_minor(),
        CouponDistributionKind::UniqueCode | CouponDistributionKind::AutoClaim => {
            if coupon_code_is_available_for_template(&context.template, &context.code, now_ms) {
                1
            } else {
                0
            }
        }
    };
    CommerceCouponDefinition {
        coupon: PortalCommerceCoupon {
            id: context.code.coupon_code_id.clone(),
            code: normalize_coupon_code(&context.code.code_value),
            discount_label: format_marketing_discount_label(&context.template.benefit),
            audience: format!("{:?}", context.template.restriction.subject_scope)
                .to_ascii_lowercase(),
            remaining,
            active: coupon_context_is_catalog_visible(context, now_ms),
            note: if context.template.display_name.trim().is_empty() {
                context.campaign.display_name.clone()
            } else {
                context.template.display_name.clone()
            },
            expires_on: format_marketing_expires_on(context),
            source: context.source.clone(),
            discount_percent: benefit.discount_percent,
            bonus_units: benefit.bonus_units,
        },
        benefit,
    }
}

fn format_marketing_discount_label(benefit: &CouponBenefitSpec) -> String {
    match benefit.benefit_kind {
        MarketingBenefitKind::PercentageOff => benefit
            .discount_percent
            .map(|percent| format!("{percent}% off"))
            .unwrap_or_else(|| "percentage off".to_owned()),
        MarketingBenefitKind::FixedAmountOff => benefit
            .discount_amount_minor
            .map(format_quote_price_label)
            .map(|label| format!("{label} off"))
            .unwrap_or_else(|| "fixed amount off".to_owned()),
        MarketingBenefitKind::GrantUnits => benefit
            .grant_units
            .map(|units| format!("+{} bonus units", format_integer_with_commas(units)))
            .unwrap_or_else(|| "bonus units".to_owned()),
    }
}

fn format_marketing_expires_on(context: &MarketingCouponContext) -> String {
    context
        .code
        .expires_at_ms
        .or(context.campaign.end_at_ms)
        .map(|value| value.to_string())
        .unwrap_or_else(|| "rolling".to_owned())
}

pub(crate) fn compute_coupon_subsidy_minor(
    list_price_cents: u64,
    benefit: &CouponBenefitSpec,
) -> u64 {
    let subsidy = match benefit.benefit_kind {
        MarketingBenefitKind::PercentageOff => benefit
            .discount_percent
            .map(|percent| list_price_cents.saturating_mul(percent as u64) / 100)
            .unwrap_or(0),
        MarketingBenefitKind::FixedAmountOff => benefit.discount_amount_minor.unwrap_or(0),
        MarketingBenefitKind::GrantUnits => 0,
    };

    subsidy
        .min(benefit.max_discount_minor.unwrap_or(u64::MAX))
        .min(list_price_cents)
}

pub(crate) fn compute_coupon_reserve_amount_minor(
    list_price_cents: u64,
    benefit: &CouponBenefitSpec,
) -> u64 {
    let subsidy_amount_minor = compute_coupon_subsidy_minor(list_price_cents, benefit);
    if subsidy_amount_minor > 0 {
        subsidy_amount_minor
    } else if matches!(benefit.benefit_kind, MarketingBenefitKind::GrantUnits) {
        1
    } else {
        0
    }
}

pub(crate) fn validate_marketing_coupon_context(
    context: &MarketingCouponContext,
    target_kind: &str,
    now_ms: u64,
    order_amount_minor: u64,
    reserve_amount_minor: u64,
) -> CouponValidationDecision {
    let decision = validate_coupon_stack(
        &context.template,
        &context.campaign,
        &context.budget,
        &context.code,
        now_ms,
        order_amount_minor,
        reserve_amount_minor,
    );
    if !decision.eligible {
        return decision;
    }

    if marketing_coupon_target_kind_allowed(&context.template, target_kind) {
        decision
    } else {
        CouponValidationDecision::rejected("target_kind_not_eligible")
    }
}

fn marketing_coupon_target_kind_allowed(
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
