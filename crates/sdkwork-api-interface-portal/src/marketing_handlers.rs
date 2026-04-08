use super::*;

pub(crate) async fn validate_marketing_coupon_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
    Json(request): Json<PortalCouponValidationRequest>,
) -> Result<Json<PortalCouponValidationResponse>, StatusCode> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub).await?;
    let subjects = PortalMarketingSubjectSet::new(&workspace, claims.claims());
    let Some(subject_id) = subjects.subject_id_for_scope(request.subject_scope) else {
        return Err(StatusCode::BAD_REQUEST);
    };
    let target_kind = request.target_kind.trim();
    if target_kind.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }
    enforce_portal_coupon_rate_limit(
        state.store.as_ref(),
        &workspace.project.id,
        CouponRateLimitAction::Validate,
        request.subject_scope,
        &subject_id,
        &request.coupon_code,
    )
    .await?;

    let now_ms = current_time_millis();
    let Some(context) =
        load_marketing_coupon_context_by_value(state.store.as_ref(), &request.coupon_code, now_ms)
            .await?
    else {
        return Err(StatusCode::NOT_FOUND);
    };

    let decision = validate_coupon_stack(
        &context.template,
        &context.campaign,
        &context.budget,
        &context.code,
        now_ms,
        request.order_amount_minor,
        request.reserve_amount_minor,
    );
    let decision = if decision.eligible
        && !portal_marketing_target_kind_allowed(&context.template, target_kind)
    {
        CouponValidationDecision::rejected("target_kind_not_eligible")
    } else {
        decision
    };

    Ok(Json(PortalCouponValidationResponse {
        decision: coupon_validation_decision_response(decision),
        template: context.template,
        campaign: context.campaign,
        budget: context.budget,
        code: context.code,
    }))
}

pub(crate) async fn reserve_marketing_coupon_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
    headers: HeaderMap,
    Json(request): Json<PortalCouponReservationRequest>,
) -> Result<(StatusCode, Json<PortalCouponReservationResponse>), StatusCode> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub).await?;
    let subjects = PortalMarketingSubjectSet::new(&workspace, claims.claims());
    let target_kind = request.target_kind.trim();
    let Some(subject_id) = subjects.subject_id_for_scope(request.subject_scope) else {
        return Err(StatusCode::BAD_REQUEST);
    };
    if target_kind.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let idempotency_key =
        resolve_portal_idempotency_key(&headers, request.idempotency_key.as_deref())?;
    let now_ms = current_time_millis();
    let coupon_reservation_id = idempotency_key
        .as_deref()
        .map(|key| {
            derive_coupon_reservation_id(request.subject_scope, &subject_id, target_kind, key)
        })
        .unwrap_or_else(|| {
            format!(
                "coupon_reservation_{}_{}",
                normalize_coupon_code(&request.coupon_code).to_ascii_lowercase(),
                now_ms
            )
        });
    if idempotency_key.is_some() {
        if let Some(existing_reservation) = state
            .store
            .find_coupon_reservation_record(&coupon_reservation_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        {
            let Some(existing_code) = state
                .store
                .find_coupon_code_record(&existing_reservation.coupon_code_id)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            else {
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            };
            let existing_ttl_ms = existing_reservation
                .expires_at_ms
                .saturating_sub(existing_reservation.created_at_ms);
            if existing_reservation.subject_scope != request.subject_scope
                || existing_reservation.subject_id != subject_id
                || normalize_coupon_code(&existing_code.code_value)
                    != normalize_coupon_code(&request.coupon_code)
                || existing_reservation.budget_reserved_minor != request.reserve_amount_minor
                || existing_ttl_ms != request.ttl_ms
            {
                return Err(StatusCode::CONFLICT);
            }

            let context = load_marketing_coupon_context_from_code_record(
                state.store.as_ref(),
                existing_code,
                now_ms,
            )
            .await?
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

            return Ok((
                StatusCode::OK,
                Json(PortalCouponReservationResponse {
                    reservation: existing_reservation,
                    template: context.template,
                    campaign: context.campaign,
                    budget: context.budget,
                    code: context.code,
                }),
            ));
        }
    }
    enforce_portal_coupon_rate_limit(
        state.store.as_ref(),
        &workspace.project.id,
        CouponRateLimitAction::Reserve,
        request.subject_scope,
        &subject_id,
        &request.coupon_code,
    )
    .await?;

    let Some(context) =
        load_marketing_coupon_context_by_value(state.store.as_ref(), &request.coupon_code, now_ms)
            .await?
    else {
        return Err(StatusCode::NOT_FOUND);
    };
    if !portal_marketing_target_kind_allowed(&context.template, target_kind) {
        return Err(StatusCode::BAD_REQUEST);
    }

    let decision = validate_coupon_stack(
        &context.template,
        &context.campaign,
        &context.budget,
        &context.code,
        now_ms,
        request.reserve_amount_minor,
        request.reserve_amount_minor,
    );
    if !decision.eligible {
        return Err(StatusCode::CONFLICT);
    }

    let (reserved_code, reservation) = reserve_coupon_redemption(
        &context.code,
        coupon_reservation_id,
        request.subject_scope,
        subject_id,
        request.reserve_amount_minor,
        now_ms,
        request.ttl_ms,
    )
    .map_err(|_| StatusCode::BAD_REQUEST)?;

    let atomic_result = state
        .store
        .reserve_coupon_redemption_atomic(&AtomicCouponReservationCommand {
            template_to_persist: context
                .compatibility_source
                .then_some(context.template.clone()),
            campaign_to_persist: context
                .compatibility_source
                .then_some(context.campaign.clone()),
            expected_budget: context.budget.clone(),
            next_budget: reserve_campaign_budget(
                &context.budget,
                request.reserve_amount_minor,
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
        .map_err(marketing_atomic_status)?;

    Ok((
        StatusCode::CREATED,
        Json(PortalCouponReservationResponse {
            reservation: atomic_result.reservation,
            template: context.template,
            campaign: context.campaign,
            budget: atomic_result.budget,
            code: atomic_result.code,
        }),
    ))
}

pub(crate) async fn confirm_marketing_coupon_redemption_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
    headers: HeaderMap,
    Json(request): Json<PortalCouponRedemptionConfirmRequest>,
) -> Result<Json<PortalCouponRedemptionConfirmResponse>, StatusCode> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub).await?;
    let subjects = PortalMarketingSubjectSet::new(&workspace, claims.claims());

    let reservation = portal_marketing_reservation_owned_by_subject(
        state.store.as_ref(),
        &subjects,
        &request.coupon_reservation_id,
    )
    .await?;
    if request.subsidy_amount_minor > reservation.budget_reserved_minor {
        return Err(StatusCode::BAD_REQUEST);
    }

    let idempotency_key =
        resolve_portal_idempotency_key(&headers, request.idempotency_key.as_deref())?;
    let now_ms = current_time_millis();
    let coupon_redemption_id = idempotency_key
        .as_deref()
        .map(|key| derive_coupon_redemption_id(&reservation, key))
        .unwrap_or_else(|| {
            format!(
                "coupon_redemption_{}_{}",
                reservation.coupon_reservation_id, now_ms
            )
        });
    if idempotency_key.is_some() {
        if let Some(existing_redemption) = state
            .store
            .find_coupon_redemption_record(&coupon_redemption_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        {
            if existing_redemption.coupon_reservation_id != reservation.coupon_reservation_id
                || existing_redemption.subsidy_amount_minor != request.subsidy_amount_minor
                || existing_redemption.order_id != request.order_id
                || existing_redemption.payment_event_id != request.payment_event_id
            {
                return Err(StatusCode::CONFLICT);
            }

            let current_reservation = portal_marketing_reservation_owned_by_subject(
                state.store.as_ref(),
                &subjects,
                &existing_redemption.coupon_reservation_id,
            )
            .await?;
            let context = load_marketing_coupon_context_for_code_id(
                state.store.as_ref(),
                &existing_redemption.coupon_code_id,
                now_ms,
            )
            .await?;

            return Ok(Json(PortalCouponRedemptionConfirmResponse {
                reservation: current_reservation,
                redemption: existing_redemption,
                budget: context.budget,
                code: context.code,
            }));
        }
    }

    let Some(code) = state
        .store
        .find_coupon_code_record(&reservation.coupon_code_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    else {
        return Err(StatusCode::NOT_FOUND);
    };
    enforce_portal_coupon_rate_limit(
        state.store.as_ref(),
        &workspace.project.id,
        CouponRateLimitAction::Confirm,
        reservation.subject_scope,
        &reservation.subject_id,
        &code.code_value,
    )
    .await?;
    let Some(context) =
        load_marketing_coupon_context_from_code_record(state.store.as_ref(), code, now_ms).await?
    else {
        return Err(StatusCode::NOT_FOUND);
    };

    let (confirmed_reservation, redemption) = confirm_coupon_redemption(
        &reservation,
        coupon_redemption_id,
        context.code.coupon_code_id.clone(),
        context.template.coupon_template_id.clone(),
        request.subsidy_amount_minor,
        request.order_id.clone(),
        request.payment_event_id.clone(),
        now_ms,
    )
    .map_err(|_| StatusCode::CONFLICT)?;
    let atomic_result = state
        .store
        .confirm_coupon_redemption_atomic(&AtomicCouponConfirmationCommand {
            expected_budget: context.budget.clone(),
            next_budget: confirm_campaign_budget(
                &context.budget,
                request.subsidy_amount_minor,
                now_ms,
            ),
            expected_code: context.code.clone(),
            next_code: code_after_confirmation(&context.template, &context.code, now_ms),
            expected_reservation: reservation.clone(),
            next_reservation: confirmed_reservation,
            redemption,
        })
        .await
        .map_err(marketing_atomic_status)?;

    Ok(Json(PortalCouponRedemptionConfirmResponse {
        reservation: atomic_result.reservation,
        redemption: atomic_result.redemption,
        budget: atomic_result.budget,
        code: atomic_result.code,
    }))
}

pub(crate) async fn rollback_marketing_coupon_redemption_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
    headers: HeaderMap,
    Json(request): Json<PortalCouponRedemptionRollbackRequest>,
) -> Result<Json<PortalCouponRedemptionRollbackResponse>, StatusCode> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub).await?;
    let subjects = PortalMarketingSubjectSet::new(&workspace, claims.claims());

    let redemption = portal_marketing_redemption_owned_by_subject(
        state.store.as_ref(),
        &subjects,
        &request.coupon_redemption_id,
    )
    .await?;
    if request.restored_budget_minor > redemption.subsidy_amount_minor {
        return Err(StatusCode::BAD_REQUEST);
    }

    let reservation = portal_marketing_reservation_owned_by_subject(
        state.store.as_ref(),
        &subjects,
        &redemption.coupon_reservation_id,
    )
    .await?;
    let idempotency_key =
        resolve_portal_idempotency_key(&headers, request.idempotency_key.as_deref())?;
    let now_ms = current_time_millis();
    let coupon_rollback_id = idempotency_key
        .as_deref()
        .map(|key| derive_coupon_rollback_id(&reservation, key))
        .unwrap_or_else(|| {
            format!(
                "coupon_rollback_{}_{}",
                redemption.coupon_redemption_id, now_ms
            )
        });
    if idempotency_key.is_some() {
        if let Some(existing_rollback) =
            find_coupon_rollback_record(state.store.as_ref(), &coupon_rollback_id).await?
        {
            if existing_rollback.coupon_redemption_id != redemption.coupon_redemption_id
                || existing_rollback.rollback_type != request.rollback_type
                || existing_rollback.restored_budget_minor != request.restored_budget_minor
                || existing_rollback.restored_inventory_count != request.restored_inventory_count
            {
                return Err(StatusCode::CONFLICT);
            }

            let current_redemption = portal_marketing_redemption_owned_by_subject(
                state.store.as_ref(),
                &subjects,
                &existing_rollback.coupon_redemption_id,
            )
            .await?;
            let context = load_marketing_coupon_context_for_code_id(
                state.store.as_ref(),
                &current_redemption.coupon_code_id,
                now_ms,
            )
            .await?;

            return Ok(Json(PortalCouponRedemptionRollbackResponse {
                redemption: current_redemption,
                rollback: existing_rollback,
                budget: context.budget,
                code: context.code,
            }));
        }
    }

    let Some(code) = state
        .store
        .find_coupon_code_record(&redemption.coupon_code_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    else {
        return Err(StatusCode::NOT_FOUND);
    };
    enforce_portal_coupon_rate_limit(
        state.store.as_ref(),
        &workspace.project.id,
        CouponRateLimitAction::Rollback,
        reservation.subject_scope,
        &reservation.subject_id,
        &code.code_value,
    )
    .await?;
    let Some(context) =
        load_marketing_coupon_context_from_code_record(state.store.as_ref(), code, now_ms).await?
    else {
        return Err(StatusCode::NOT_FOUND);
    };

    let (rolled_back_redemption, rollback) = rollback_coupon_redemption(
        &redemption,
        coupon_rollback_id,
        request.rollback_type,
        request.restored_budget_minor,
        request.restored_inventory_count,
        now_ms,
    )
    .map_err(|_| StatusCode::CONFLICT)?;
    let atomic_result = state
        .store
        .rollback_coupon_redemption_atomic(&AtomicCouponRollbackCommand {
            expected_budget: context.budget.clone(),
            next_budget: rollback_campaign_budget(
                &context.budget,
                request.restored_budget_minor,
                now_ms,
            ),
            expected_code: context.code.clone(),
            next_code: code_after_rollback(&context.template, &context.code, now_ms),
            expected_redemption: redemption.clone(),
            next_redemption: rolled_back_redemption,
            rollback,
        })
        .await
        .map_err(marketing_atomic_status)?;

    Ok(Json(PortalCouponRedemptionRollbackResponse {
        redemption: atomic_result.redemption,
        rollback: atomic_result.rollback,
        budget: atomic_result.budget,
        code: atomic_result.code,
    }))
}

pub(crate) async fn list_my_coupons_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<PortalMarketingCodesResponse>, StatusCode> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub).await?;
    let subjects = PortalMarketingSubjectSet::new(&workspace, claims.claims());
    let items = load_marketing_code_items(state.store.as_ref(), &subjects).await?;
    let summary = summarize_marketing_code_items(&items);
    Ok(Json(PortalMarketingCodesResponse { summary, items }))
}

pub(crate) async fn list_marketing_reward_history_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<Vec<PortalMarketingRewardHistoryItem>>, StatusCode> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub).await?;
    let subjects = PortalMarketingSubjectSet::new(&workspace, claims.claims());
    load_marketing_reward_history_items(state.store.as_ref(), &subjects)
        .await
        .map(Json)
}

pub(crate) async fn list_marketing_redemptions_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
    Query(query): Query<PortalMarketingRedemptionsQuery>,
) -> Result<Json<PortalMarketingRedemptionsResponse>, StatusCode> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub).await?;
    let subjects = PortalMarketingSubjectSet::new(&workspace, claims.claims());
    let items =
        load_marketing_redemptions_for_subject(state.store.as_ref(), &subjects, query.status)
            .await?;
    let summary = summarize_marketing_redemptions(&items);
    Ok(Json(PortalMarketingRedemptionsResponse { summary, items }))
}

pub(crate) async fn list_marketing_codes_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
    Query(query): Query<PortalMarketingCodesQuery>,
) -> Result<Json<PortalMarketingCodesResponse>, StatusCode> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub).await?;
    let subjects = PortalMarketingSubjectSet::new(&workspace, claims.claims());
    let mut items = load_marketing_code_items(state.store.as_ref(), &subjects).await?;
    if let Some(status) = query.status {
        items.retain(|item| item.code.status == status);
    }
    let summary = summarize_marketing_code_items(&items);
    Ok(Json(PortalMarketingCodesResponse { summary, items }))
}


