use super::*;

pub async fn load_portal_commerce_catalog(
    store: &dyn AdminStore,
) -> CommerceResult<PortalCommerceCatalog> {
    let plans = subscription_plan_catalog();
    let packs = recharge_pack_catalog();
    let recharge_options = recharge_option_catalog();
    let custom_recharge_policy = Some(build_custom_recharge_policy());
    let canonical_catalog = current_canonical_commercial_catalog_for_store(store).await?;
    Ok(PortalCommerceCatalog {
        products: portal_api_products_from_canonical_catalog(&canonical_catalog),
        offers: portal_product_offers_from_canonical_catalog(&canonical_catalog),
        plans,
        packs,
        recharge_options,
        custom_recharge_policy,
        coupons: load_coupon_catalog(store).await?,
    })
}

pub async fn preview_portal_commerce_quote(
    store: &dyn AdminStore,
    request: &PortalCommerceQuoteRequest,
) -> CommerceResult<PortalCommerceQuote> {
    Ok(preview_portal_commerce_quote_internal(store, request)
        .await?
        .0)
}

pub(crate) async fn preview_portal_commerce_quote_internal(
    store: &dyn AdminStore,
    request: &PortalCommerceQuoteRequest,
) -> CommerceResult<(PortalCommerceQuote, Option<ResolvedCouponDefinition>)> {
    let target_kind = request.target_kind.trim();
    let target_id = request.target_id.trim();

    if target_kind.is_empty() {
        return Err(CommerceError::InvalidInput(
            "target_kind is required".to_owned(),
        ));
    }
    if target_id.is_empty() {
        return Err(CommerceError::InvalidInput(
            "target_id is required".to_owned(),
        ));
    }

    let catalog_binding =
        current_quote_target_catalog_binding_for_store(store, target_kind, target_id).await?;

    match target_kind {
        "subscription_plan" => {
            let plan = subscription_plan_seeds()
                .into_iter()
                .find(|candidate| candidate.id.eq_ignore_ascii_case(target_id))
                .ok_or_else(|| CommerceError::NotFound("subscription plan not found".to_owned()))?;
            let applied_coupon = load_optional_applied_coupon(
                store,
                request.coupon_code.as_deref(),
                target_kind,
                plan.price_cents,
            )
            .await?;
            let quote = build_priced_quote(
                "subscription_plan",
                plan.id,
                plan.name,
                plan.price_cents,
                plan.included_units,
                "workspace_seed",
                request.current_remaining_units,
                catalog_binding,
                applied_coupon.as_ref().map(|item| item.definition.clone()),
            );
            Ok((quote, applied_coupon))
        }
        "recharge_pack" => {
            let pack = recharge_pack_seeds()
                .into_iter()
                .find(|candidate| candidate.id.eq_ignore_ascii_case(target_id))
                .ok_or_else(|| CommerceError::NotFound("recharge pack not found".to_owned()))?;
            let applied_coupon = load_optional_applied_coupon(
                store,
                request.coupon_code.as_deref(),
                target_kind,
                pack.price_cents,
            )
            .await?;
            let quote = build_priced_quote(
                "recharge_pack",
                pack.id,
                pack.label,
                pack.price_cents,
                pack.points,
                "workspace_seed",
                request.current_remaining_units,
                catalog_binding,
                applied_coupon.as_ref().map(|item| item.definition.clone()),
            );
            Ok((quote, applied_coupon))
        }
        "custom_recharge" => {
            let custom_amount_cents =
                resolve_custom_recharge_amount_cents(target_id, request.custom_amount_cents)?;
            let applied_coupon = load_optional_applied_coupon(
                store,
                request.coupon_code.as_deref(),
                target_kind,
                custom_amount_cents,
            )
            .await?;
            let quote = build_custom_recharge_quote(
                custom_amount_cents,
                request.current_remaining_units,
                catalog_binding,
                applied_coupon.as_ref().map(|item| item.definition.clone()),
            )?;
            Ok((quote, applied_coupon))
        }
        "coupon_redemption" => {
            let coupon = find_resolved_coupon_definition(store, target_id).await?;
            if coupon.definition.benefit.bonus_units == 0 {
                return Err(CommerceError::InvalidInput(format!(
                    "coupon {} does not grant redeemable bonus units",
                    coupon.definition.coupon.code
                )));
            }
            let quote = build_redemption_quote(
                coupon.definition.clone(),
                request.current_remaining_units,
                catalog_binding,
            );
            Ok((quote, Some(coupon)))
        }
        _ => Err(CommerceError::InvalidInput(format!(
            "unsupported target_kind: {target_kind}"
        ))),
    }
}

pub async fn submit_portal_commerce_order(
    store: &dyn AdminStore,
    user_id: &str,
    project_id: &str,
    request: &PortalCommerceQuoteRequest,
) -> CommerceResult<CommerceOrderRecord> {
    let normalized_user_id = user_id.trim();
    let normalized_project_id = project_id.trim();
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

    let (quote, resolved_coupon) = preview_portal_commerce_quote_internal(store, request).await?;
    if let Some(existing_order) = find_reusable_pending_payable_order(
        store,
        normalized_user_id,
        normalized_project_id,
        &quote,
    )
    .await?
    {
        return Ok(existing_order);
    }

    let status = initial_order_status(&quote);
    let order_id = generate_entity_id("commerce_order")?;
    let reserved_coupon = reserve_order_coupon_if_needed(
        store,
        &order_id,
        normalized_project_id,
        &quote,
        resolved_coupon.as_ref(),
    )
    .await?;

    let created_at_ms = current_time_ms()?;
    let mut order = CommerceOrderRecord::new(
        order_id,
        normalized_project_id,
        normalized_user_id,
        quote.target_kind.clone(),
        quote.target_id.clone(),
        quote.target_name.clone(),
        quote.list_price_cents,
        quote.payable_price_cents,
        quote.list_price_label.clone(),
        quote.payable_price_label.clone(),
        quote.granted_units,
        quote.bonus_units,
        status,
        quote.source.clone(),
        created_at_ms,
    )
    .with_pricing_plan_id_option(quote.pricing_plan_id.clone())
    .with_pricing_plan_version_option(quote.pricing_plan_version)
    .with_pricing_snapshot_json(build_order_pricing_snapshot_json(
        request,
        &quote,
        resolved_coupon.as_ref(),
        reserved_coupon.as_ref(),
        created_at_ms,
    )?)
    .with_applied_coupon_code_option(
        quote
            .applied_coupon
            .as_ref()
            .map(|coupon| coupon.code.clone()),
    )
    .with_coupon_reservation_id_option(
        reserved_coupon
            .as_ref()
            .map(|coupon| coupon.coupon_reservation_id.clone()),
    )
    .with_marketing_campaign_id_option(
        reserved_coupon
            .as_ref()
            .map(|coupon| coupon.marketing_campaign_id.clone()),
    )
    .with_subsidy_amount_minor(
        reserved_coupon
            .as_ref()
            .map(|coupon| coupon.subsidy_amount_minor)
            .unwrap_or(0),
    );

    if should_fulfill_on_order_create(&quote) {
        if let Err(error) = fulfill_order_on_create(
            store,
            normalized_user_id,
            normalized_project_id,
            &quote,
            &mut order,
        )
        .await
        {
            let _ = release_order_coupon_reservation_if_needed(store, &mut order).await;
            return Err(error);
        }
    }

    match store
        .insert_commerce_order(&order)
        .await
        .map_err(CommerceError::from)
    {
        Ok(order) => Ok(order),
        Err(error) => {
            if order.coupon_redemption_id.is_some() {
                let _ = rollback_order_coupon_redemption_if_needed(
                    store,
                    &mut order,
                    CouponRollbackType::Manual,
                )
                .await;
            } else {
                let _ = release_order_coupon_reservation_if_needed(store, &mut order).await;
            }
            Err(error)
        }
    }
}

pub async fn settle_portal_commerce_order(
    store: &dyn AdminStore,
    user_id: &str,
    project_id: &str,
    order_id: &str,
) -> CommerceResult<CommerceOrderRecord> {
    settle_portal_commerce_order_with_billing(store, None, user_id, project_id, order_id).await
}

pub async fn settle_portal_commerce_order_with_billing(
    store: &dyn AdminStore,
    commercial_billing: Option<&dyn CommercialBillingAdminKernel>,
    user_id: &str,
    project_id: &str,
    order_id: &str,
) -> CommerceResult<CommerceOrderRecord> {
    settle_portal_commerce_order_with_payment_event(
        store,
        commercial_billing,
        user_id,
        project_id,
        order_id,
        None,
    )
    .await
}

pub async fn settle_portal_commerce_order_from_verified_payment(
    store: &dyn AdminStore,
    user_id: &str,
    project_id: &str,
    order_id: &str,
) -> CommerceResult<CommerceOrderRecord> {
    settle_portal_commerce_order_with_payment_event(
        store, None, user_id, project_id, order_id, None,
    )
    .await
}

pub(crate) async fn settle_portal_commerce_order_with_payment_event(
    store: &dyn AdminStore,
    commercial_billing: Option<&dyn CommercialBillingAdminKernel>,
    user_id: &str,
    project_id: &str,
    order_id: &str,
    payment_event_id: Option<&str>,
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
        "fulfilled" => {
            sync_fulfilled_commerce_order_account_state(
                store,
                commercial_billing,
                normalized_project_id,
                &order,
            )
            .await?;
            return Ok(order);
        }
        "pending_payment" => {
            if settlement_side_effects_already_applied(&order) {
                let order = persist_fulfilled_order(store, &mut order).await?;
                sync_fulfilled_commerce_order_account_state(
                    store,
                    commercial_billing,
                    normalized_project_id,
                    &order,
                )
                .await?;
                return Ok(order);
            }
        }
        other => {
            return Err(CommerceError::Conflict(format!(
                "order {normalized_order_id} cannot be settled from status {other}"
            )));
        }
    }

    let settlement_quote = load_order_settlement_quote(store, &order).await?;
    let settlement_snapshot =
        capture_settlement_side_effect_snapshot(store, normalized_project_id).await?;

    let settle_result: CommerceResult<CommerceOrderRecord> = async {
        apply_quote_to_project_quota(store, normalized_project_id, &settlement_quote).await?;
        activate_project_membership_if_needed(
            store,
            normalized_user_id,
            normalized_project_id,
            &settlement_quote,
        )
        .await?;
        confirm_order_coupon_if_needed(store, &mut order, payment_event_id).await?;
        persist_fulfilled_order(store, &mut order).await
    }
    .await;

    match settle_result {
        Ok(order) => {
            sync_fulfilled_commerce_order_account_state(
                store,
                commercial_billing,
                normalized_project_id,
                &order,
            )
            .await?;
            Ok(order)
        }
        Err(error) => {
            let _ = restore_settlement_side_effects(
                store,
                normalized_project_id,
                &settlement_quote,
                &mut order,
                &settlement_snapshot,
            )
            .await;
            Err(error)
        }
    }
}

pub async fn cancel_portal_commerce_order(
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
        "canceled" => return Ok(order),
        "pending_payment" => {}
        other => {
            return Err(CommerceError::Conflict(format!(
                "order {normalized_order_id} cannot be canceled from status {other}"
            )));
        }
    }

    release_order_coupon_reservation_if_needed(store, &mut order).await?;
    order.status = "canceled".to_owned();
    order.settlement_status = "canceled".to_owned();
    order.updated_at_ms = current_time_ms()?;
    store
        .insert_commerce_order(&order)
        .await
        .map_err(CommerceError::from)
}

pub(crate) async fn refund_portal_commerce_order(
    store: &dyn AdminStore,
    commercial_billing: Option<&dyn CommercialBillingAdminKernel>,
    user_id: &str,
    project_id: &str,
    order_id: &str,
    refund_provider: &str,
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
    ensure_refund_provider_matches_order_settlement(store, &order, refund_provider).await?;

    if !supports_safe_order_refund(&order) {
        return Err(CommerceError::Conflict(format!(
            "order {normalized_order_id} target_kind {} cannot be refunded safely",
            order.target_kind
        )));
    }

    match order.status.as_str() {
        "refunded" => {
            sync_refunded_commerce_order_account_state(
                store,
                commercial_billing,
                normalized_project_id,
                &order,
            )
            .await?;
            return Ok(order);
        }
        "fulfilled" => {}
        other => {
            return Err(CommerceError::Conflict(format!(
                "order {normalized_order_id} cannot be refunded from status {other}"
            )));
        }
    }

    let previous_quota_policy = load_effective_quota_policy(store, normalized_project_id).await?;
    let mut coupon_rollback_compensation = None;
    let refund_result: CommerceResult<CommerceOrderRecord> = async {
        reverse_order_quota_effect(store, normalized_project_id, &order).await?;
        coupon_rollback_compensation = rollback_order_coupon_redemption_if_needed(
            store,
            &mut order,
            CouponRollbackType::Refund,
        )
        .await?;

        order.status = "refunded".to_owned();
        order.settlement_status = "refunded".to_owned();
        order.refunded_amount_minor = order.payable_price_cents;
        order.refundable_amount_minor = 0;
        order.updated_at_ms = current_time_ms()?;
        store
            .insert_commerce_order(&order)
            .await
            .map_err(CommerceError::from)
    }
    .await;

    match refund_result {
        Ok(order) => {
            sync_refunded_commerce_order_account_state(
                store,
                commercial_billing,
                normalized_project_id,
                &order,
            )
            .await?;
            Ok(order)
        }
        Err(error) => {
            let current_quota_policy =
                load_effective_quota_policy(store, normalized_project_id).await?;
            let quota_restore_error = restore_quota_policy_snapshot(
                store,
                previous_quota_policy.as_ref(),
                current_quota_policy.as_ref(),
            )
            .await
            .err();
            let coupon_restore_error = compensate_coupon_rollback_side_effects_if_needed(
                store,
                coupon_rollback_compensation.as_ref(),
            )
            .await
            .err();
            if quota_restore_error.is_none() && coupon_restore_error.is_none() {
                return Err(error);
            }

            let mut message = format!("refund finalization failed: {error}");
            if let Some(quota_restore_error) = quota_restore_error {
                message.push_str(&format!(
                    "; quota compensation failed: {quota_restore_error}"
                ));
            }
            if let Some(coupon_restore_error) = coupon_restore_error {
                message.push_str(&format!(
                    "; coupon compensation failed: {coupon_restore_error}"
                ));
            }
            Err(CommerceError::Conflict(message))
        }
    }
}

pub async fn apply_portal_commerce_payment_event(
    store: &dyn AdminStore,
    user_id: &str,
    project_id: &str,
    order_id: &str,
    request: &PortalCommercePaymentEventRequest,
) -> CommerceResult<CommerceOrderRecord> {
    apply_portal_commerce_payment_event_with_billing(
        store, None, user_id, project_id, order_id, request,
    )
    .await
}

async fn sync_fulfilled_commerce_order_account_state<T>(
    store: &T,
    commercial_billing: Option<&dyn CommercialBillingAdminKernel>,
    project_id: &str,
    order: &CommerceOrderRecord,
) -> CommerceResult<()>
where
    T: AdminStore + ?Sized,
{
    let Some((billing, account)) =
        resolve_project_commercial_billing_account(store, commercial_billing, project_id).await?
    else {
        return Ok(());
    };

    if commerce_order_requires_account_credit(order) {
        billing
            .issue_commerce_order_credits(IssueCommerceOrderCreditsInput {
                account_id: account.account_id,
                order_id: &order.order_id,
                project_id,
                target_kind: &order.target_kind,
                granted_quantity: commerce_order_credit_quantity(order),
                payable_amount: commerce_order_payable_amount(order),
                issued_at_ms: order.updated_at_ms,
            })
            .await
            .map_err(CommerceError::from)?;
    }

    upsert_commerce_reconciliation_checkpoint(billing, &account, project_id, order).await
}

async fn sync_refunded_commerce_order_account_state<T>(
    store: &T,
    commercial_billing: Option<&dyn CommercialBillingAdminKernel>,
    project_id: &str,
    order: &CommerceOrderRecord,
) -> CommerceResult<()>
where
    T: AdminStore + ?Sized,
{
    let Some((billing, account)) =
        resolve_project_commercial_billing_account(store, commercial_billing, project_id).await?
    else {
        return Ok(());
    };

    if commerce_order_requires_account_credit(order) {
        billing
            .refund_commerce_order_credits(RefundCommerceOrderCreditsInput {
                account_id: account.account_id,
                order_id: &order.order_id,
                refunded_quantity: commerce_order_credit_quantity(order),
                refunded_amount: commerce_order_payable_amount(order),
                refunded_at_ms: order.updated_at_ms,
            })
            .await
            .map_err(CommerceError::from)?;
    }

    upsert_commerce_reconciliation_checkpoint(billing, &account, project_id, order).await
}

async fn resolve_project_commercial_billing_account<'a, T>(
    store: &T,
    commercial_billing: Option<&'a dyn CommercialBillingAdminKernel>,
    project_id: &str,
) -> CommerceResult<Option<(&'a dyn CommercialBillingAdminKernel, AccountRecord)>>
where
    T: AdminStore + ?Sized,
{
    let Some(billing) = commercial_billing else {
        return Ok(None);
    };
    let normalized_project_id = project_id.trim();
    if normalized_project_id.is_empty() {
        return Err(CommerceError::InvalidInput(
            "project_id is required".to_owned(),
        ));
    }
    let project = store
        .find_project(normalized_project_id)
        .await
        .map_err(CommerceError::from)?
        .ok_or_else(|| {
            CommerceError::NotFound(format!("project {normalized_project_id} not found"))
        })?;
    let request_context = GatewayRequestContext {
        tenant_id: project.tenant_id,
        project_id: project.id,
        environment: "portal".to_owned(),
        api_key_hash: PORTAL_WORKSPACE_SCOPE_KEY_HASH.to_owned(),
        api_key_group_id: None,
        canonical_tenant_id: None,
        canonical_organization_id: None,
        canonical_user_id: None,
        canonical_api_key_id: None,
    };
    let account = billing
        .resolve_payable_account_for_gateway_request_context(&request_context)
        .await
        .map_err(CommerceError::from)?;
    Ok(account.map(|account| (billing, account)))
}

async fn upsert_commerce_reconciliation_checkpoint(
    billing: &dyn CommercialBillingAdminKernel,
    account: &AccountRecord,
    project_id: &str,
    order: &CommerceOrderRecord,
) -> CommerceResult<()> {
    billing
        .insert_account_commerce_reconciliation_state(
            &AccountCommerceReconciliationStateRecord::new(
                account.tenant_id,
                account.organization_id,
                account.account_id,
                project_id,
                &order.order_id,
            )
            .with_last_order_updated_at_ms(order.updated_at_ms)
            .with_last_order_created_at_ms(order.created_at_ms)
            .with_updated_at_ms(order.updated_at_ms),
        )
        .await
        .map_err(CommerceError::from)?;
    Ok(())
}

fn commerce_order_requires_account_credit(order: &CommerceOrderRecord) -> bool {
    matches!(
        order.target_kind.as_str(),
        "recharge_pack" | "custom_recharge"
    )
}

fn commerce_order_credit_quantity(order: &CommerceOrderRecord) -> f64 {
    order.granted_units.saturating_add(order.bonus_units) as f64
}

fn commerce_order_payable_amount(order: &CommerceOrderRecord) -> f64 {
    order.payable_price_cents as f64 / 100.0
}

async fn find_reusable_pending_payable_order(
    store: &dyn AdminStore,
    user_id: &str,
    project_id: &str,
    quote: &PortalCommerceQuote,
) -> CommerceResult<Option<CommerceOrderRecord>> {
    if quote.payable_price_cents == 0 {
        return Ok(None);
    }

    let applied_coupon_code = quote
        .applied_coupon
        .as_ref()
        .map(|coupon| coupon.code.as_str());
    let now_ms = current_time_ms()?;
    let orders = store
        .list_commerce_orders_for_project(project_id)
        .await
        .map_err(CommerceError::from)?;

    let mut reusable_orders = Vec::new();
    for order in orders {
        if !pending_order_matches_quote_intent(&order, user_id, quote, applied_coupon_code) {
            continue;
        }
        if !pending_order_has_reusable_coupon_reservation(store, &order, now_ms).await? {
            continue;
        }
        reusable_orders.push(order);
    }

    Ok(reusable_orders.into_iter().max_by_key(|order| {
        (
            order.updated_at_ms,
            order.created_at_ms,
            order.order_id.clone(),
        )
    }))
}

fn pending_order_matches_quote_intent(
    order: &CommerceOrderRecord,
    user_id: &str,
    quote: &PortalCommerceQuote,
    applied_coupon_code: Option<&str>,
) -> bool {
    order.user_id == user_id
        && order.status == "pending_payment"
        && matches!(
            order.settlement_status.as_str(),
            "pending" | "requires_action"
        )
        && order.target_kind == quote.target_kind
        && order.target_id == quote.target_id
        && order.list_price_cents == quote.list_price_cents
        && order.payable_price_cents == quote.payable_price_cents
        && order.granted_units == quote.granted_units
        && order.bonus_units == quote.bonus_units
        && order.applied_coupon_code.as_deref() == applied_coupon_code
}

async fn pending_order_has_reusable_coupon_reservation(
    store: &dyn AdminStore,
    order: &CommerceOrderRecord,
    now_ms: u64,
) -> CommerceResult<bool> {
    let Some(coupon_code) = order.applied_coupon_code.as_deref() else {
        return Ok(true);
    };
    let Some(coupon_reservation_id) = order.coupon_reservation_id.as_deref() else {
        return Ok(false);
    };
    let Some(reservation) = store
        .find_coupon_reservation_record(coupon_reservation_id)
        .await
        .map_err(CommerceError::from)?
    else {
        return Ok(false);
    };

    Ok(reservation.subject_id == order.project_id
        && order.applied_coupon_code.as_deref() == Some(coupon_code)
        && reservation.reservation_status
            == sdkwork_api_domain_marketing::CouponReservationStatus::Reserved
        && reservation.is_active_at(now_ms))
}

fn settlement_side_effects_already_applied(order: &CommerceOrderRecord) -> bool {
    matches!(order.settlement_status.as_str(), "settled" | "not_required")
        || order.coupon_redemption_id.is_some()
}

async fn persist_fulfilled_order(
    store: &dyn AdminStore,
    order: &mut CommerceOrderRecord,
) -> CommerceResult<CommerceOrderRecord> {
    order.status = "fulfilled".to_owned();
    order.settlement_status = fulfilled_order_settlement_status(order).to_owned();
    order.refundable_amount_minor = order.payable_price_cents;
    order.refunded_amount_minor = 0;
    order.updated_at_ms = current_time_ms()?;
    store
        .insert_commerce_order(order)
        .await
        .map_err(CommerceError::from)
}

fn fulfilled_order_settlement_status(order: &CommerceOrderRecord) -> &'static str {
    if order.payable_price_cents == 0 {
        "not_required"
    } else {
        "settled"
    }
}

pub async fn load_portal_commerce_checkout_session(
    store: &dyn AdminStore,
    user_id: &str,
    project_id: &str,
    order_id: &str,
) -> CommerceResult<PortalCommerceCheckoutSession> {
    load_portal_commerce_checkout_session_with_policy(store, user_id, project_id, order_id, false)
        .await
}

pub async fn load_portal_commerce_checkout_session_with_policy(
    store: &dyn AdminStore,
    user_id: &str,
    project_id: &str,
    order_id: &str,
    payment_simulation_enabled: bool,
) -> CommerceResult<PortalCommerceCheckoutSession> {
    let order = load_project_commerce_order(store, user_id, project_id, order_id).await?;
    build_checkout_session(store, &order, payment_simulation_enabled).await
}

pub async fn load_portal_commerce_order(
    store: &dyn AdminStore,
    user_id: &str,
    project_id: &str,
    order_id: &str,
) -> CommerceResult<CommerceOrderRecord> {
    load_project_commerce_order(store, user_id, project_id, order_id).await
}

pub async fn list_project_commerce_orders(
    store: &dyn AdminStore,
    project_id: &str,
) -> CommerceResult<Vec<CommerceOrderRecord>> {
    let normalized_project_id = project_id.trim();
    if normalized_project_id.is_empty() {
        return Err(CommerceError::InvalidInput(
            "project_id is required".to_owned(),
        ));
    }

    let mut orders = store
        .list_commerce_orders_for_project(normalized_project_id)
        .await
        .map_err(CommerceError::from)?;
    orders.sort_by(|left, right| {
        right
            .updated_at_ms
            .cmp(&left.updated_at_ms)
            .then_with(|| right.created_at_ms.cmp(&left.created_at_ms))
            .then_with(|| right.order_id.cmp(&left.order_id))
    });
    Ok(orders)
}

pub async fn load_project_membership(
    store: &dyn AdminStore,
    project_id: &str,
) -> CommerceResult<Option<ProjectMembershipRecord>> {
    let normalized_project_id = project_id.trim();
    if normalized_project_id.is_empty() {
        return Err(CommerceError::InvalidInput(
            "project_id is required".to_owned(),
        ));
    }

    store
        .find_project_membership(normalized_project_id)
        .await
        .map_err(CommerceError::from)
}

fn build_order_pricing_snapshot_json(
    request: &PortalCommerceQuoteRequest,
    quote: &PortalCommerceQuote,
    resolved_coupon: Option<&ResolvedCouponDefinition>,
    reserved_coupon: Option<&ReservedMarketingCouponState>,
    issued_at_ms: u64,
) -> CommerceResult<String> {
    let catalog_binding = PortalCommerceCatalogBinding::from_quote(quote);
    serde_json::to_string(&serde_json::json!({
        "issued_at_ms": issued_at_ms,
        "request": request,
        "quote": quote,
        "catalog_binding": {
            "product_id": catalog_binding.product_id,
            "offer_id": catalog_binding.offer_id,
            "publication_id": catalog_binding.publication_id,
            "publication_kind": catalog_binding.publication_kind,
            "publication_status": catalog_binding.publication_status,
            "publication_revision_id": catalog_binding.publication_revision_id,
            "publication_version": catalog_binding.publication_version,
            "publication_source_kind": catalog_binding.publication_source_kind,
            "publication_effective_from_ms": catalog_binding.publication_effective_from_ms,
        },
        "pricing_binding": {
            "pricing_plan_id": catalog_binding.pricing_plan_id,
            "pricing_plan_version": catalog_binding.pricing_plan_version,
            "pricing_rate_id": catalog_binding.pricing_rate_id,
            "pricing_metric_code": catalog_binding.pricing_metric_code,
        },
        "resolved_coupon": resolved_coupon.as_ref().map(|resolved_coupon| serde_json::json!({
            "coupon": resolved_coupon.definition.coupon,
            "discount_percent": resolved_coupon.definition.benefit.discount_percent,
            "bonus_units": resolved_coupon.definition.benefit.bonus_units,
            "marketing_campaign_id": resolved_coupon
                .marketing
                .as_ref()
                .map(|marketing| marketing.campaign.marketing_campaign_id.clone()),
            "coupon_template_id": resolved_coupon
                .marketing
                .as_ref()
                .map(|marketing| marketing.template.coupon_template_id.clone()),
            "coupon_code_id": resolved_coupon
                .marketing
                .as_ref()
                .map(|marketing| marketing.code.coupon_code_id.clone()),
        })),
        "reservation": reserved_coupon.as_ref().map(|reserved_coupon| serde_json::json!({
            "coupon_reservation_id": reserved_coupon.coupon_reservation_id,
            "marketing_campaign_id": reserved_coupon.marketing_campaign_id,
            "subsidy_amount_minor": reserved_coupon.subsidy_amount_minor,
        })),
    }))
    .map_err(|error| CommerceError::Storage(error.into()))
}
