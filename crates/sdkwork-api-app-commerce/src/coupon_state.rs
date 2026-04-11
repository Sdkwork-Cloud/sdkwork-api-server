use super::*;

pub(crate) async fn reserve_order_coupon_if_needed<T>(
    store: &T,
    order_id: &str,
    project_id: &str,
    quote: &PortalCommerceQuote,
    resolved_coupon: Option<&ResolvedCouponDefinition>,
) -> CommerceResult<Option<ReservedMarketingCouponState>>
where
    T: AdminStore + ?Sized,
{
    let Some(resolved_coupon) = resolved_coupon else {
        return Ok(None);
    };
    let Some(context) = resolved_coupon.marketing.as_ref() else {
        return Ok(None);
    };

    let now_ms = current_time_ms()?;
    let reserve_amount_minor =
        compute_coupon_reserve_amount_minor(quote.list_price_cents, &context.template.benefit);
    let decision = validate_marketing_coupon_context(
        context,
        quote.target_kind.as_str(),
        now_ms,
        quote.list_price_cents,
        reserve_amount_minor,
    );
    if !decision.eligible {
        return Err(CommerceError::InvalidInput(format!(
            "coupon {} is not eligible: {}",
            resolved_coupon.definition.coupon.code,
            decision
                .rejection_reason
                .unwrap_or_else(|| "validation_failed".to_owned())
        )));
    }

    let coupon_reservation_id = format!("coupon_reservation_{order_id}");
    let (reserved_code, reservation) = reserve_coupon_redemption(
        &context.code,
        coupon_reservation_id.clone(),
        MarketingSubjectScope::Project,
        project_id.to_owned(),
        decision.reservable_budget_minor,
        now_ms,
        DEFAULT_COUPON_RESERVATION_TTL_MS,
    )
    .map_err(|error| CommerceError::Conflict(error.to_string()))?;
    store
        .reserve_coupon_redemption_atomic(&AtomicCouponReservationCommand {
            template_to_persist: None,
            campaign_to_persist: None,
            expected_budget: context.budget.clone(),
            next_budget: reserve_campaign_budget(
                &context.budget,
                decision.reservable_budget_minor,
                now_ms,
            ),
            expected_code: context.code.clone(),
            next_code: code_after_reservation(
                &context.template,
                &context.code,
                &reserved_code,
                now_ms,
            ),
            reservation,
        })
        .await
        .map_err(commerce_atomic_coupon_error)?;

    Ok(Some(ReservedMarketingCouponState {
        coupon_reservation_id,
        marketing_campaign_id: context.campaign.marketing_campaign_id.clone(),
        subsidy_amount_minor: compute_coupon_subsidy_minor(
            quote.list_price_cents,
            &context.template.benefit,
        ),
    }))
}

pub(crate) async fn confirm_order_coupon_if_needed<T>(
    store: &T,
    order: &mut CommerceOrderRecord,
    payment_event_id: Option<&str>,
) -> CommerceResult<()>
where
    T: AdminStore + ?Sized,
{
    let Some(coupon_reservation_id) = order.coupon_reservation_id.clone() else {
        return Ok(());
    };
    if order.coupon_redemption_id.is_some() {
        return Ok(());
    }

    let coupon_redemption_id = format!("coupon_redemption_{}", order.order_id);
    let reservation = store
        .find_coupon_reservation_record(&coupon_reservation_id)
        .await
        .map_err(CommerceError::from)?
        .ok_or_else(|| {
            CommerceError::Conflict(format!(
                "coupon reservation {} not found for order {}",
                coupon_reservation_id, order.order_id
            ))
        })?;
    let now_ms = current_time_ms()?;
    let context = load_order_marketing_context(store, order, now_ms).await?;
    let (confirmed_reservation, redemption) = confirm_coupon_redemption(
        &reservation,
        coupon_redemption_id.clone(),
        context.code.coupon_code_id.clone(),
        context.template.coupon_template_id.clone(),
        order.subsidy_amount_minor,
        Some(order.order_id.clone()),
        payment_event_id.map(str::to_owned),
        now_ms,
    )
    .map_err(|error| CommerceError::Conflict(error.to_string()))?;
    store
        .confirm_coupon_redemption_atomic(&AtomicCouponConfirmationCommand {
            expected_budget: context.budget.clone(),
            next_budget: confirm_campaign_budget(
                &context.budget,
                reservation.budget_reserved_minor,
                now_ms,
            ),
            expected_code: context.code.clone(),
            next_code: code_after_confirmation(&context.template, &context.code, now_ms),
            expected_reservation: reservation,
            next_reservation: confirmed_reservation,
            redemption,
        })
        .await
        .map_err(commerce_atomic_coupon_error)?;
    order.coupon_redemption_id = Some(coupon_redemption_id);
    Ok(())
}

pub(crate) async fn release_order_coupon_reservation_if_needed<T>(
    store: &T,
    order: &mut CommerceOrderRecord,
) -> CommerceResult<()>
where
    T: AdminStore + ?Sized,
{
    if order.coupon_redemption_id.is_some() {
        return Ok(());
    }
    let Some(coupon_reservation_id) = order.coupon_reservation_id.clone() else {
        return Ok(());
    };

    let Some(reservation) = store
        .find_coupon_reservation_record(&coupon_reservation_id)
        .await
        .map_err(CommerceError::from)?
    else {
        return Ok(());
    };
    if reservation.reservation_status != CouponReservationStatus::Reserved {
        return Ok(());
    }

    let now_ms = current_time_ms()?;
    let context = load_order_marketing_context(store, order, now_ms).await?;
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
                .with_status(CouponReservationStatus::Released)
                .with_updated_at_ms(now_ms),
        })
        .await
        .map_err(commerce_atomic_coupon_error)?;
    Ok(())
}

pub(crate) async fn rollback_order_coupon_redemption_if_needed<T>(
    store: &T,
    order: &mut CommerceOrderRecord,
    rollback_type: CouponRollbackType,
) -> CommerceResult<Option<CouponRollbackCompensationSnapshot>>
where
    T: AdminStore + ?Sized,
{
    let Some(coupon_redemption_id) = order.coupon_redemption_id.clone() else {
        return Ok(None);
    };
    let Some(redemption) = store
        .find_coupon_redemption_record(&coupon_redemption_id)
        .await
        .map_err(CommerceError::from)?
    else {
        return Ok(None);
    };
    if matches!(
        redemption.redemption_status,
        CouponRedemptionStatus::RolledBack | CouponRedemptionStatus::PartiallyRolledBack
    ) {
        return Ok(None);
    }

    let reservation = store
        .find_coupon_reservation_record(&redemption.coupon_reservation_id)
        .await
        .map_err(CommerceError::from)?;
    let redemption_before_rollback = redemption.clone();
    let restored_budget_minor = reservation
        .as_ref()
        .map(|item| item.budget_reserved_minor)
        .unwrap_or(redemption.subsidy_amount_minor);
    let now_ms = current_time_ms()?;
    let context = load_order_marketing_context(store, order, now_ms).await?;
    let rollback_id = format!(
        "coupon_rollback_{}_{}",
        order.order_id,
        match rollback_type {
            CouponRollbackType::Cancel => "cancel",
            CouponRollbackType::Refund => "refund",
            CouponRollbackType::PartialRefund => "partial_refund",
            CouponRollbackType::Manual => "manual",
        }
    );
    let (rolled_back_redemption, rollback) = rollback_coupon_redemption(
        &redemption,
        rollback_id,
        rollback_type,
        restored_budget_minor,
        if coupon_code_is_exclusive(&context.template) {
            1
        } else {
            0
        },
        now_ms,
    )
    .map_err(|error| CommerceError::Conflict(error.to_string()))?;
    let atomic_result = store
        .rollback_coupon_redemption_atomic(&AtomicCouponRollbackCommand {
            expected_budget: context.budget.clone(),
            next_budget: rollback_campaign_budget(&context.budget, restored_budget_minor, now_ms),
            expected_code: context.code.clone(),
            next_code: code_after_rollback(&context.template, &context.code, now_ms),
            expected_redemption: redemption,
            next_redemption: rolled_back_redemption,
            rollback,
        })
        .await
        .map_err(commerce_atomic_coupon_error)?;
    Ok(Some(CouponRollbackCompensationSnapshot {
        previous_budget: context.budget,
        previous_code: context.code,
        previous_redemption: redemption_before_rollback,
        applied_budget: atomic_result.budget,
        applied_code: atomic_result.code,
        applied_redemption: atomic_result.redemption,
        applied_rollback: atomic_result.rollback,
    }))
}

pub(crate) async fn compensate_coupon_rollback_side_effects_if_needed<T>(
    store: &T,
    snapshot: Option<&CouponRollbackCompensationSnapshot>,
) -> CommerceResult<()>
where
    T: AdminStore + ?Sized,
{
    let Some(snapshot) = snapshot else {
        return Ok(());
    };
    let failed_at_ms = current_time_ms()?;
    store
        .compensate_coupon_rollback_atomic(&AtomicCouponRollbackCompensationCommand {
            expected_budget: snapshot.applied_budget.clone(),
            next_budget: snapshot
                .previous_budget
                .clone()
                .with_updated_at_ms(failed_at_ms),
            expected_code: snapshot.applied_code.clone(),
            next_code: snapshot
                .previous_code
                .clone()
                .with_updated_at_ms(failed_at_ms),
            expected_redemption: snapshot.applied_redemption.clone(),
            next_redemption: snapshot
                .previous_redemption
                .clone()
                .with_updated_at_ms(failed_at_ms),
            expected_rollback: snapshot.applied_rollback.clone(),
            next_rollback: snapshot
                .applied_rollback
                .clone()
                .with_status(CouponRollbackStatus::Failed)
                .with_updated_at_ms(failed_at_ms),
        })
        .await
        .map_err(commerce_atomic_coupon_error)?;
    Ok(())
}

pub(crate) async fn load_order_marketing_context<T>(
    store: &T,
    order: &CommerceOrderRecord,
    now_ms: u64,
) -> CommerceResult<MarketingCouponContext>
where
    T: AdminStore + ?Sized,
{
    let code_value = order
        .applied_coupon_code
        .as_deref()
        .or_else(|| (order.target_kind == "coupon_redemption").then_some(order.target_id.as_str()))
        .ok_or_else(|| {
            CommerceError::Conflict(format!(
                "order {} does not reference a marketing coupon",
                order.order_id
            ))
        })?;
    let normalized_code = normalize_coupon_code(code_value);
    if let Some(code_record) = store
        .find_coupon_code_record_by_value(&normalized_code)
        .await
        .map_err(CommerceError::from)?
    {
        let template = store
            .find_coupon_template_record(&code_record.coupon_template_id)
            .await
            .map_err(CommerceError::from)?
            .ok_or_else(|| {
                CommerceError::Conflict(format!(
                    "coupon template {} not found for order {}",
                    code_record.coupon_template_id, order.order_id
                ))
            })?;
        let campaigns = store
            .list_marketing_campaign_records_for_template(&template.coupon_template_id)
            .await
            .map_err(CommerceError::from)?;
        let campaign = order
            .marketing_campaign_id
            .as_deref()
            .and_then(|marketing_campaign_id| {
                campaigns
                    .iter()
                    .find(|record| record.marketing_campaign_id == marketing_campaign_id)
                    .cloned()
            })
            .or_else(|| select_effective_marketing_campaign(campaigns.clone(), now_ms))
            .or_else(|| {
                campaigns.into_iter().max_by(|left, right| {
                    left.updated_at_ms
                        .cmp(&right.updated_at_ms)
                        .then_with(|| left.marketing_campaign_id.cmp(&right.marketing_campaign_id))
                })
            })
            .ok_or_else(|| {
                CommerceError::Conflict(format!(
                    "marketing campaign not found for order {}",
                    order.order_id
                ))
            })?;
        let budget = select_campaign_budget_record(
            store
                .list_campaign_budget_records_for_campaign(&campaign.marketing_campaign_id)
                .await
                .map_err(CommerceError::from)?,
        )
        .ok_or_else(|| {
            CommerceError::Conflict(format!(
                "campaign budget not found for order {}",
                order.order_id
            ))
        })?;
        return Ok(MarketingCouponContext {
            template,
            campaign,
            budget,
            code: code_record,
            source: "marketing".to_owned(),
        });
    }

    Err(CommerceError::Conflict(format!(
        "coupon {} no longer resolves to a marketing context",
        code_value
    )))
}

fn coupon_code_is_exclusive(template: &CouponTemplateRecord) -> bool {
    !matches!(
        template.distribution_kind,
        CouponDistributionKind::SharedCode
    )
}

pub(crate) fn code_after_reservation(
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

pub(crate) fn code_after_confirmation(
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

pub(crate) fn code_after_release(
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

pub(crate) fn code_after_rollback(
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

pub(crate) fn restore_coupon_code_availability(
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

pub(crate) fn reserve_campaign_budget(
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

pub(crate) fn release_campaign_budget(
    budget: &CampaignBudgetRecord,
    released_amount_minor: u64,
    now_ms: u64,
) -> CampaignBudgetRecord {
    let next_reserved = budget
        .reserved_budget_minor
        .saturating_sub(released_amount_minor);
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

pub(crate) fn confirm_campaign_budget(
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

pub(crate) fn rollback_campaign_budget(
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
