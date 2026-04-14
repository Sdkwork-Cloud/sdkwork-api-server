use super::*;

pub(crate) async fn reserve_order_coupon_if_needed(
    store: &dyn AdminStore,
    order_id: &str,
    project_id: &str,
    quote: &PortalCommerceQuote,
    resolved_coupon: Option<&ResolvedCouponDefinition>,
) -> CommerceResult<Option<ReservedMarketingCouponState>> {
    let Some(resolved_coupon) = resolved_coupon else {
        return Ok(None);
    };
    let Some(marketing_context) = resolved_coupon.marketing.as_ref() else {
        return Ok(None);
    };

    let now_ms = current_time_ms()?;
    let reserve_amount_minor = compute_coupon_reserve_amount_minor(
        quote.list_price_cents,
        &marketing_context.template.benefit,
    );
    let reservation = reserve_coupon_for_subject(
        store,
        ReserveCouponInput {
            coupon_code: resolved_coupon.definition.coupon.code.as_str(),
            subject_scope: MarketingSubjectScope::Project,
            subject_id: project_id,
            target_kind: quote.target_kind.as_str(),
            order_amount_minor: quote.list_price_cents,
            reserve_amount_minor,
            ttl_ms: DEFAULT_COUPON_RESERVATION_TTL_MS,
            idempotency_key: Some(order_id),
            now_ms,
        },
    )
    .await
    .map_err(CommerceError::from)?;

    Ok(Some(ReservedMarketingCouponState {
        coupon_reservation_id: reservation.reservation.coupon_reservation_id,
        marketing_campaign_id: reservation.context.campaign.marketing_campaign_id.clone(),
        subsidy_amount_minor: compute_coupon_subsidy_minor(
            quote.list_price_cents,
            &reservation.context.template.benefit,
        ),
    }))
}

pub(crate) async fn confirm_order_coupon_if_needed(
    store: &dyn AdminStore,
    order: &mut CommerceOrderRecord,
    payment_event_id: Option<&str>,
) -> CommerceResult<()> {
    let Some(coupon_reservation_id) = order.coupon_reservation_id.clone() else {
        return Ok(());
    };
    if order.coupon_redemption_id.is_some() {
        return Ok(());
    }

    let now_ms = current_time_ms()?;
    let result = confirm_coupon_for_subject(
        store,
        ConfirmCouponInput {
            coupon_reservation_id: &coupon_reservation_id,
            subject_scope: MarketingSubjectScope::Project,
            subject_id: &order.project_id,
            subsidy_amount_minor: order.subsidy_amount_minor,
            order_id: Some(order.order_id.clone()),
            payment_event_id: payment_event_id.map(str::to_owned),
            idempotency_key: Some(order.order_id.as_str()),
            now_ms,
        },
    )
    .await
    .map_err(CommerceError::from)?;
    order.coupon_redemption_id = Some(result.redemption.coupon_redemption_id);
    Ok(())
}

pub(crate) async fn release_order_coupon_reservation_if_needed(
    store: &dyn AdminStore,
    order: &mut CommerceOrderRecord,
) -> CommerceResult<()> {
    if order.coupon_redemption_id.is_some() {
        return Ok(());
    }
    let Some(coupon_reservation_id) = order.coupon_reservation_id.clone() else {
        return Ok(());
    };
    let now_ms = current_time_ms()?;
    match release_coupon_for_subject(
        store,
        ReleaseCouponInput {
            coupon_reservation_id: &coupon_reservation_id,
            subject_scope: MarketingSubjectScope::Project,
            subject_id: &order.project_id,
            now_ms,
        },
    )
    .await
    {
        Ok(_) => Ok(()),
        Err(sdkwork_api_app_marketing::MarketingOperationError::NotFound(_)) => Ok(()),
        Err(error) => Err(error.into()),
    }
}

pub(crate) async fn rollback_order_coupon_redemption_if_needed(
    store: &dyn AdminStore,
    order: &mut CommerceOrderRecord,
    rollback_type: CouponRollbackType,
) -> CommerceResult<Option<CouponRollbackCompensationSnapshot>> {
    let Some(coupon_redemption_id) = order.coupon_redemption_id.clone() else {
        return Ok(None);
    };
    let evidence =
        load_shared_marketing_order_evidence(store, None, Some(&coupon_redemption_id), None, None)
            .await
            .map_err(CommerceError::from)?;
    let Some(redemption) = evidence.coupon_redemption else {
        return Ok(None);
    };
    if matches!(
        redemption.redemption_status,
        CouponRedemptionStatus::RolledBack | CouponRedemptionStatus::PartiallyRolledBack
    ) {
        return Ok(None);
    }

    let redemption_before_rollback = redemption.clone();
    let restored_budget_minor = evidence
        .coupon_reservation
        .as_ref()
        .map(|item| item.budget_reserved_minor)
        .unwrap_or(redemption.budget_consumed_minor);
    let now_ms = current_time_ms()?;
    let context = load_shared_marketing_coupon_context_for_reference(
        store,
        MarketingCouponContextReference {
            applied_coupon_code: order.applied_coupon_code.as_deref(),
            target_kind: &order.target_kind,
            target_id: &order.target_id,
            preferred_marketing_campaign_id: order.marketing_campaign_id.as_deref(),
        },
        now_ms,
    )
    .await
    .map_err(CommerceError::from)?
    .ok_or_else(|| {
        CommerceError::Conflict(format!(
            "marketing coupon context is unavailable for order {}",
            order.order_id
        ))
    })?;
    let rollback_key = format!(
        "{}:{}",
        order.order_id,
        match rollback_type {
            CouponRollbackType::Cancel => "cancel",
            CouponRollbackType::Refund => "refund",
            CouponRollbackType::PartialRefund => "partial_refund",
            CouponRollbackType::Manual => "manual",
        }
    );
    let atomic_result = rollback_coupon_for_subject(
        store,
        RollbackCouponInput {
            coupon_redemption_id: &coupon_redemption_id,
            subject_scope: MarketingSubjectScope::Project,
            subject_id: &order.project_id,
            rollback_type,
            restored_budget_minor,
            restored_inventory_count: if !matches!(
                context.template.distribution_kind,
                CouponDistributionKind::SharedCode
            ) {
                1
            } else {
                0
            },
            idempotency_key: Some(rollback_key.as_str()),
            now_ms,
        },
    )
    .await
    .map_err(CommerceError::from)?;
    Ok(Some(CouponRollbackCompensationSnapshot {
        previous_budget: context.budget,
        previous_code: context.code,
        previous_redemption: redemption_before_rollback,
        applied_budget: atomic_result.context.budget,
        applied_code: atomic_result.context.code,
        applied_redemption: atomic_result.redemption,
        applied_rollback: atomic_result.rollback,
    }))
}

pub(crate) async fn compensate_coupon_rollback_side_effects_if_needed(
    store: &dyn AdminStore,
    snapshot: Option<&CouponRollbackCompensationSnapshot>,
) -> CommerceResult<()> {
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
