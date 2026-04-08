use super::*;

pub(crate) async fn fulfill_order_on_create<T>(
    store: &T,
    normalized_user_id: &str,
    normalized_project_id: &str,
    quote: &PortalCommerceQuote,
    order: &mut CommerceOrderRecord,
) -> CommerceResult<()>
where
    T: AdminStore + CommerceQuotaStore + ?Sized,
{
    apply_quote_to_project_quota(store, normalized_project_id, quote).await?;
    activate_project_membership_if_needed(store, normalized_user_id, normalized_project_id, quote)
        .await?;
    confirm_order_coupon_if_needed(store, order, None).await
}

pub(crate) async fn capture_settlement_side_effect_snapshot<T>(
    store: &T,
    project_id: &str,
) -> CommerceResult<CommerceSettlementSideEffectSnapshot>
where
    T: AdminStore + CommerceQuotaStore + ?Sized,
{
    Ok(CommerceSettlementSideEffectSnapshot {
        previous_quota_policy: load_effective_quota_policy(store, project_id).await?,
        previous_membership: store
            .find_project_membership(project_id)
            .await
            .map_err(CommerceError::from)?,
    })
}

pub(crate) async fn restore_settlement_side_effects<T>(
    store: &T,
    project_id: &str,
    quote: &PortalCommerceQuote,
    order: &mut CommerceOrderRecord,
    snapshot: &CommerceSettlementSideEffectSnapshot,
) -> CommerceResult<()>
where
    T: AdminStore + CommerceQuotaStore + ?Sized,
{
    if order.coupon_redemption_id.is_some() {
        let _ =
            rollback_order_coupon_redemption_if_needed(store, order, CouponRollbackType::Manual)
                .await?;
    }

    restore_project_membership_snapshot(
        store,
        project_id,
        quote,
        snapshot.previous_membership.as_ref(),
    )
    .await?;
    restore_quota_policy_snapshot(
        store,
        snapshot.previous_quota_policy.as_ref(),
        build_next_quota_policy(project_id, quote, snapshot.previous_quota_policy.as_ref())
            .as_ref(),
    )
    .await
}

pub(crate) async fn restore_project_membership_snapshot<T>(
    store: &T,
    project_id: &str,
    quote: &PortalCommerceQuote,
    previous_membership: Option<&ProjectMembershipRecord>,
) -> CommerceResult<()>
where
    T: AdminStore + ?Sized,
{
    if quote.target_kind != "subscription_plan" {
        return Ok(());
    }

    match previous_membership {
        Some(membership) => {
            store
                .upsert_project_membership(membership)
                .await
                .map_err(CommerceError::from)?;
        }
        None => {
            store
                .delete_project_membership(project_id)
                .await
                .map_err(CommerceError::from)?;
        }
    }

    Ok(())
}

pub(crate) async fn restore_quota_policy_snapshot<T>(
    store: &T,
    previous_quota_policy: Option<&QuotaPolicy>,
    applied_quota_policy: Option<&QuotaPolicy>,
) -> CommerceResult<()>
where
    T: AdminStore + ?Sized,
{
    if let Some(policy) = previous_quota_policy {
        store
            .insert_quota_policy(policy)
            .await
            .map_err(CommerceError::from)?;
    } else if let Some(policy) = applied_quota_policy {
        store
            .delete_quota_policy(&policy.policy_id)
            .await
            .map_err(CommerceError::from)?;
    }

    Ok(())
}

pub(crate) fn build_next_quota_policy(
    project_id: &str,
    quote: &PortalCommerceQuote,
    effective_policy: Option<&QuotaPolicy>,
) -> Option<QuotaPolicy> {
    let target_units = quote.granted_units.saturating_add(quote.bonus_units);
    if target_units == 0 {
        return None;
    }

    let current_limit = effective_policy.map(|policy| policy.max_units).unwrap_or(0);
    let policy_id = effective_policy
        .map(|policy| policy.policy_id.clone())
        .unwrap_or_else(|| format!("portal_commerce_{project_id}"));
    let next_limit = match quote.target_kind.as_str() {
        "subscription_plan" => current_limit.max(target_units),
        "recharge_pack" | "custom_recharge" | "coupon_redemption" => {
            current_limit.saturating_add(target_units)
        }
        _ => current_limit,
    };

    (next_limit != current_limit)
        .then(|| QuotaPolicy::new(policy_id, project_id.to_owned(), next_limit))
}

pub(crate) async fn apply_quote_to_project_quota<T>(
    store: &T,
    project_id: &str,
    quote: &PortalCommerceQuote,
) -> CommerceResult<()>
where
    T: AdminStore + CommerceQuotaStore + ?Sized,
{
    let effective_policy = load_effective_quota_policy(store, project_id).await?;
    let Some(next_policy) = build_next_quota_policy(project_id, quote, effective_policy.as_ref())
    else {
        return Ok(());
    };
    store
        .insert_quota_policy(&next_policy)
        .await
        .map_err(CommerceError::from)?;
    Ok(())
}

pub(crate) async fn reverse_order_quota_effect<T>(
    store: &T,
    project_id: &str,
    order: &CommerceOrderRecord,
) -> CommerceResult<()>
where
    T: AdminStore + CommerceQuotaStore + ?Sized,
{
    let target_units = order.granted_units.saturating_add(order.bonus_units);
    if target_units == 0 {
        return Ok(());
    }

    if let Some(membership) = store
        .find_project_membership(project_id)
        .await
        .map_err(CommerceError::from)?
    {
        if membership.updated_at_ms > order.created_at_ms {
            return Err(CommerceError::Conflict(format!(
                "order {} cannot be refunded safely after subscription changes",
                order.order_id
            )));
        }
    }

    let effective_policy = load_effective_quota_policy(store, project_id)
        .await?
        .ok_or_else(|| {
            CommerceError::Conflict(format!(
                "order {} cannot be refunded because no active quota policy exists",
                order.order_id
            ))
        })?;

    if effective_policy.max_units < target_units {
        return Err(CommerceError::Conflict(format!(
            "order {} cannot be refunded because quota baseline drifted",
            order.order_id
        )));
    }

    let used_units = store
        .list_ledger_entries_for_project(project_id)
        .await
        .map_err(CommerceError::from)?
        .into_iter()
        .map(|entry| entry.units)
        .sum::<u64>();
    let remaining_units = effective_policy.max_units.saturating_sub(used_units);
    if remaining_units < target_units {
        return Err(CommerceError::Conflict(format!(
            "order {} cannot be refunded because recharge headroom has already been consumed",
            order.order_id
        )));
    }

    let next_policy = QuotaPolicy::new(
        effective_policy.policy_id,
        project_id.to_owned(),
        effective_policy.max_units.saturating_sub(target_units),
    );
    store
        .insert_quota_policy(&next_policy)
        .await
        .map_err(CommerceError::from)?;
    Ok(())
}

pub(crate) async fn load_effective_quota_policy<T>(
    store: &T,
    project_id: &str,
) -> CommerceResult<Option<QuotaPolicy>>
where
    T: CommerceQuotaStore + ?Sized,
{
    Ok(store
        .list_quota_policies_for_project(project_id)
        .await
        .map_err(CommerceError::from)?
        .into_iter()
        .filter(|policy| policy.enabled)
        .min_by(|left, right| {
            left.max_units
                .cmp(&right.max_units)
                .then_with(|| left.policy_id.cmp(&right.policy_id))
        }))
}

pub(crate) async fn activate_project_membership_if_needed<T>(
    store: &T,
    user_id: &str,
    project_id: &str,
    quote: &PortalCommerceQuote,
) -> CommerceResult<()>
where
    T: AdminStore + ?Sized,
{
    if quote.target_kind != "subscription_plan" {
        return Ok(());
    }

    let plan = subscription_plan_seeds()
        .into_iter()
        .find(|candidate| candidate.id.eq_ignore_ascii_case(&quote.target_id))
        .ok_or_else(|| CommerceError::NotFound("subscription plan not found".to_owned()))?;
    let activated_at_ms = current_time_ms()?;

    store
        .upsert_project_membership(&ProjectMembershipRecord::new(
            generate_entity_id("membership")?,
            project_id,
            user_id,
            plan.id,
            plan.name,
            quote.payable_price_cents,
            quote.payable_price_label.clone(),
            plan.cadence,
            plan.included_units,
            "active",
            quote.source.clone(),
            activated_at_ms,
            activated_at_ms,
        ))
        .await
        .map_err(CommerceError::from)?;
    Ok(())
}

pub(crate) fn should_fulfill_on_order_create(quote: &PortalCommerceQuote) -> bool {
    quote.target_kind == "coupon_redemption"
}

pub(crate) fn supports_safe_order_refund(order: &CommerceOrderRecord) -> bool {
    matches!(
        order.target_kind.as_str(),
        "recharge_pack" | "custom_recharge"
    ) && order.payable_price_cents > 0
}

pub(crate) fn initial_order_status(quote: &PortalCommerceQuote) -> &'static str {
    if should_fulfill_on_order_create(quote) {
        "fulfilled"
    } else {
        "pending_payment"
    }
}

pub(crate) async fn load_project_commerce_order<T>(
    store: &T,
    user_id: &str,
    project_id: &str,
    order_id: &str,
) -> CommerceResult<CommerceOrderRecord>
where
    T: AdminStore + ?Sized,
{
    let order = store
        .list_commerce_orders_for_project(project_id)
        .await
        .map_err(CommerceError::from)?
        .into_iter()
        .find(|candidate| candidate.order_id == order_id)
        .ok_or_else(|| CommerceError::NotFound(format!("order {order_id} not found")))?;

    if order.user_id != user_id {
        return Err(CommerceError::NotFound(format!(
            "order {order_id} not found"
        )));
    }

    Ok(order)
}

pub(crate) async fn ensure_refund_provider_matches_order_settlement<T>(
    store: &T,
    order: &CommerceOrderRecord,
    refund_provider: &str,
) -> CommerceResult<()>
where
    T: AdminStore + ?Sized,
{
    let settled_provider = resolve_processed_settlement_provider_for_order(store, order).await?;
    if refund_provider != settled_provider {
        return Err(CommerceError::Conflict(format!(
            "refund provider {refund_provider} does not match settled provider {settled_provider} for order {}",
            order.order_id
        )));
    }

    Ok(())
}

pub(crate) async fn resolve_processed_settlement_provider_for_order<T>(
    store: &T,
    order: &CommerceOrderRecord,
) -> CommerceResult<String>
where
    T: AdminStore + ?Sized,
{
    let settled_provider = store
        .list_commerce_payment_events_for_order(&order.order_id)
        .await
        .map_err(CommerceError::from)?
        .into_iter()
        .filter(|event| {
            event.event_type == "settled"
                && matches!(
                    event.processing_status,
                    CommercePaymentEventProcessingStatus::Processed
                )
                && event.order_status_after.as_deref() == Some("fulfilled")
        })
        .max_by_key(|event| event.processed_at_ms.unwrap_or(event.received_at_ms))
        .map(|event| event.provider)
        .unwrap_or_else(|| COMMERCE_PAYMENT_PROVIDER_MANUAL_LAB.to_owned());

    Ok(settled_provider)
}

pub(crate) async fn fail_portal_commerce_order(
    store: &dyn AdminStore,
    user_id: &str,
    project_id: &str,
    order_id: &str,
) -> CommerceResult<CommerceOrderRecord> {
    let normalized_user_id = user_id.trim();
    let normalized_project_id = project_id.trim();
    let normalized_order_id = order_id.trim();

    if normalized_user_id.is_empty() {
        return Err(CommerceError::InvalidInput(
            "user_id is required".to_owned(),
        ));
    }
    if normalized_project_id.is_empty() {
        return Err(CommerceError::InvalidInput(
            "project_id is required".to_owned(),
        ));
    }
    if normalized_order_id.is_empty() {
        return Err(CommerceError::InvalidInput(
            "order_id is required".to_owned(),
        ));
    }

    let mut order = load_project_commerce_order(
        store,
        normalized_user_id,
        normalized_project_id,
        normalized_order_id,
    )
    .await?;

    match order.status.as_str() {
        "failed" => return Ok(order),
        "pending_payment" => {}
        other => {
            return Err(CommerceError::Conflict(format!(
                "order {normalized_order_id} cannot be marked failed from status {other}"
            )));
        }
    }

    release_order_coupon_reservation_if_needed(store, &mut order).await?;
    order.status = "failed".to_owned();
    order.settlement_status = "failed".to_owned();
    order.updated_at_ms = current_time_ms()?;
    store
        .insert_commerce_order(&order)
        .await
        .map_err(CommerceError::from)
}

pub(crate) async fn load_order_settlement_quote(
    _store: &dyn AdminStore,
    order: &CommerceOrderRecord,
) -> CommerceResult<PortalCommerceQuote> {
    Ok(PortalCommerceQuote {
        target_kind: order.target_kind.clone(),
        target_id: order.target_id.clone(),
        target_name: order.target_name.clone(),
        list_price_cents: order.list_price_cents,
        payable_price_cents: order.payable_price_cents,
        list_price_label: order.list_price_label.clone(),
        payable_price_label: order.payable_price_label.clone(),
        granted_units: order.granted_units,
        bonus_units: order.bonus_units,
        amount_cents: if order.target_kind == "custom_recharge" {
            Some(order.list_price_cents)
        } else {
            None
        },
        projected_remaining_units: None,
        applied_coupon: order
            .applied_coupon_code
            .as_ref()
            .map(|code| PortalAppliedCoupon {
                code: code.clone(),
                discount_label: code.clone(),
                source: "order_snapshot".to_owned(),
                discount_percent: None,
                bonus_units: order.bonus_units,
            }),
        pricing_rule_label: if order.target_kind == "custom_recharge" {
            Some("Tiered custom recharge".to_owned())
        } else {
            None
        },
        effective_ratio_label: if order.target_kind == "custom_recharge"
            && order.list_price_cents > 0
        {
            Some(format_effective_ratio_label(
                order.granted_units / order.list_price_cents.max(1),
            ))
        } else {
            None
        },
        source: order.source.clone(),
    })
}
