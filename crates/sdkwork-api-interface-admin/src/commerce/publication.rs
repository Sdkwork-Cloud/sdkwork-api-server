use super::*;

#[derive(Debug, Clone, Serialize, ToSchema)]
pub(crate) struct CommercialCatalogPublicationProjection {
    pub(crate) product: ApiProduct,
    pub(crate) offer: ProductOffer,
    pub(crate) publication: CatalogPublication,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub(crate) struct CommercialCatalogPublicationActionDecision {
    pub(crate) allowed: bool,
    #[serde(default)]
    pub(crate) reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub(crate) struct CommercialCatalogPublicationActionability {
    pub(crate) publish: CommercialCatalogPublicationActionDecision,
    pub(crate) schedule: CommercialCatalogPublicationActionDecision,
    pub(crate) retire: CommercialCatalogPublicationActionDecision,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub(crate) struct CommercialCatalogPublicationDetail {
    pub(crate) projection: CommercialCatalogPublicationProjection,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) governed_pricing_plan: Option<PricingPlanRecord>,
    #[serde(default)]
    pub(crate) governed_pricing_rates: Vec<PricingRateRecord>,
    pub(crate) actionability: CommercialCatalogPublicationActionability,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub(crate) struct CommercialCatalogPublicationMutationResult {
    pub(crate) detail: CommercialCatalogPublicationDetail,
    pub(crate) audit: CatalogPublicationLifecycleAuditRecord,
}

#[derive(Debug, Clone)]
struct CommercialCatalogPublicationContext {
    projection: CommercialCatalogPublicationProjection,
    governed_pricing_plan: Option<PricingPlanRecord>,
    governed_pricing_rates: Vec<PricingRateRecord>,
    pricing_plans: Vec<PricingPlanRecord>,
    pricing_rates: Vec<PricingRateRecord>,
}

fn build_commercial_catalog_publication_projections(
    catalog: &sdkwork_api_app_catalog::CanonicalCommercialCatalog,
) -> Result<Vec<CommercialCatalogPublicationProjection>, (StatusCode, Json<ErrorResponse>)> {
    let products_by_id = catalog
        .products
        .iter()
        .map(|product| (product.product_id.clone(), product.clone()))
        .collect::<HashMap<_, _>>();
    let offers_by_id = catalog
        .offers
        .iter()
        .map(|offer| (offer.offer_id.clone(), offer.clone()))
        .collect::<HashMap<_, _>>();

    let mut projections = Vec::with_capacity(catalog.publications.len());
    for publication in &catalog.publications {
        let product = products_by_id
            .get(&publication.product_id)
            .cloned()
            .ok_or_else(|| {
                error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!(
                        "failed to resolve canonical product {} for publication {}",
                        publication.product_id, publication.publication_id
                    ),
                )
            })?;
        let offer = offers_by_id
            .get(&publication.offer_id)
            .cloned()
            .ok_or_else(|| {
                error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!(
                        "failed to resolve canonical offer {} for publication {}",
                        publication.offer_id, publication.publication_id
                    ),
                )
            })?;
        projections.push(CommercialCatalogPublicationProjection {
            product,
            offer,
            publication: publication.clone(),
        });
    }

    projections.sort_by(|left, right| {
        left.publication
            .publication_id
            .cmp(&right.publication.publication_id)
    });
    Ok(projections)
}

fn resolve_governed_pricing_plan_for_publication(
    projection: &CommercialCatalogPublicationProjection,
    plans: &[PricingPlanRecord],
) -> Result<Option<PricingPlanRecord>, (StatusCode, Json<ErrorResponse>)> {
    if projection.publication.publication_source_kind != "pricing_plan" {
        return Ok(None);
    }

    let expected_plan_code = canonical_catalog_pricing_plan_code(
        projection.offer.quote_target_kind,
        &projection.offer.quote_target_id,
    );
    let expected_version = projection.publication.publication_version;
    let plan = plans
        .iter()
        .filter(|plan| {
            normalize_commercial_pricing_plan_code(&plan.plan_code)
                .ok()
                .flatten()
                .as_deref()
                == Some(expected_plan_code.as_str())
                && plan.plan_version == expected_version
        })
        .max_by(|left, right| {
            left.updated_at_ms
                .cmp(&right.updated_at_ms)
                .then(left.created_at_ms.cmp(&right.created_at_ms))
                .then(left.pricing_plan_id.cmp(&right.pricing_plan_id))
        })
        .cloned()
        .ok_or_else(|| {
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!(
                    "failed to resolve governed pricing plan for publication {}",
                    projection.publication.publication_id
                ),
            )
        })?;

    Ok(Some(plan))
}

fn allowed_publication_action() -> CommercialCatalogPublicationActionDecision {
    CommercialCatalogPublicationActionDecision {
        allowed: true,
        reasons: Vec::new(),
    }
}

fn blocked_publication_action(
    reason: impl Into<String>,
) -> CommercialCatalogPublicationActionDecision {
    CommercialCatalogPublicationActionDecision {
        allowed: false,
        reasons: vec![reason.into()],
    }
}

fn build_publication_actionability(
    projection: &CommercialCatalogPublicationProjection,
    governed_pricing_plan: Option<&PricingPlanRecord>,
    governed_pricing_rates: &[PricingRateRecord],
    now_ms: u64,
) -> CommercialCatalogPublicationActionability {
    let Some(governed_pricing_plan) = governed_pricing_plan else {
        let reason =
            "publication is derived from catalog_seed and has no governed pricing plan".to_owned();
        return CommercialCatalogPublicationActionability {
            publish: blocked_publication_action(reason.clone()),
            schedule: blocked_publication_action(reason.clone()),
            retire: blocked_publication_action(reason),
        };
    };

    let no_rates_reason =
        "publication has no governed pricing rates and cannot apply lifecycle actions".to_owned();
    let has_rates = !governed_pricing_rates.is_empty();
    let governed_pricing_status = governed_pricing_plan.status.trim().to_ascii_lowercase();
    let effective_from_ms = projection
        .publication
        .publication_effective_from_ms
        .unwrap_or(governed_pricing_plan.effective_from_ms);

    let publish = match projection.publication.status {
        CatalogPublicationStatus::Published => {
            blocked_publication_action("publication is already published")
        }
        CatalogPublicationStatus::Archived => blocked_publication_action(
            "publication is already retired; create a new governed revision instead",
        ),
        CatalogPublicationStatus::Draft if !has_rates => {
            blocked_publication_action(no_rates_reason.clone())
        }
        CatalogPublicationStatus::Draft if effective_from_ms > now_ms => {
            blocked_publication_action(
                "publication effective_from_ms is in the future; schedule instead",
            )
        }
        CatalogPublicationStatus::Draft => allowed_publication_action(),
    };

    let schedule = match projection.publication.status {
        CatalogPublicationStatus::Published => {
            blocked_publication_action("publication is already published")
        }
        CatalogPublicationStatus::Archived => {
            blocked_publication_action("publication is already retired")
        }
        CatalogPublicationStatus::Draft if !has_rates => {
            blocked_publication_action(no_rates_reason.clone())
        }
        CatalogPublicationStatus::Draft if governed_pricing_status == "planned" => {
            blocked_publication_action("publication is already scheduled")
        }
        CatalogPublicationStatus::Draft if effective_from_ms <= now_ms => {
            blocked_publication_action(
                "publication can only be scheduled for a future effective_from_ms",
            )
        }
        CatalogPublicationStatus::Draft => allowed_publication_action(),
    };

    let retire = match projection.publication.status {
        CatalogPublicationStatus::Archived => {
            blocked_publication_action("publication is already retired")
        }
        CatalogPublicationStatus::Published | CatalogPublicationStatus::Draft => {
            allowed_publication_action()
        }
    };

    CommercialCatalogPublicationActionability {
        publish,
        schedule,
        retire,
    }
}

fn build_commercial_catalog_publication_detail(
    context: &CommercialCatalogPublicationContext,
    now_ms: u64,
) -> CommercialCatalogPublicationDetail {
    let actionability = build_publication_actionability(
        &context.projection,
        context.governed_pricing_plan.as_ref(),
        &context.governed_pricing_rates,
        now_ms,
    );
    CommercialCatalogPublicationDetail {
        projection: context.projection.clone(),
        governed_pricing_plan: context.governed_pricing_plan.clone(),
        governed_pricing_rates: context.governed_pricing_rates.clone(),
        actionability,
    }
}

async fn load_commercial_catalog_publication_context(
    state: &AdminApiState,
    publication_id: &str,
) -> Result<CommercialCatalogPublicationContext, (StatusCode, Json<ErrorResponse>)> {
    let commercial_billing = commercial_billing_kernel(state)?.clone();
    synchronize_due_pricing_plan_lifecycle(commercial_billing.as_ref(), unix_timestamp_ms())
        .await
        .map_err(commercial_billing_error_response)?;
    let pricing_plans = commercial_billing
        .list_pricing_plan_records()
        .await
        .map_err(commercial_billing_error_response)?;
    let pricing_rates = commercial_billing
        .list_pricing_rate_records()
        .await
        .map_err(commercial_billing_error_response)?;

    let catalog = current_canonical_commercial_catalog_for_store(state.store.as_ref())
        .await
        .map_err(admin_commerce_error_response)?;
    let projection = build_commercial_catalog_publication_projections(&catalog)?
        .into_iter()
        .find(|projection| projection.publication.publication_id == publication_id)
        .ok_or_else(|| {
            error_response(
                StatusCode::NOT_FOUND,
                format!("publication {publication_id} does not exist"),
            )
        })?;
    let governed_pricing_plan =
        resolve_governed_pricing_plan_for_publication(&projection, &pricing_plans)?;
    let governed_pricing_rates = governed_pricing_plan
        .as_ref()
        .map(|plan| {
            pricing_rates
                .iter()
                .filter(|rate| rate.pricing_plan_id == plan.pricing_plan_id)
                .cloned()
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    Ok(CommercialCatalogPublicationContext {
        projection,
        governed_pricing_plan,
        governed_pricing_rates,
        pricing_plans,
        pricing_rates,
    })
}

fn publication_mutation_decision(
    detail: &CommercialCatalogPublicationDetail,
    action: CatalogPublicationLifecycleAction,
) -> &CommercialCatalogPublicationActionDecision {
    match action {
        CatalogPublicationLifecycleAction::Publish => &detail.actionability.publish,
        CatalogPublicationLifecycleAction::Schedule => &detail.actionability.schedule,
        CatalogPublicationLifecycleAction::Retire => &detail.actionability.retire,
    }
}

fn publication_mutation_label(action: CatalogPublicationLifecycleAction) -> &'static str {
    match action {
        CatalogPublicationLifecycleAction::Publish => "published",
        CatalogPublicationLifecycleAction::Schedule => "scheduled",
        CatalogPublicationLifecycleAction::Retire => "retired",
    }
}

fn normalized_publication_mutation_reason(
    reason: &str,
) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
    let normalized = reason.trim();
    if normalized.is_empty() {
        return Err(error_response(
            StatusCode::BAD_REQUEST,
            "publication lifecycle reason is required",
        ));
    }
    Ok(normalized.to_owned())
}

#[allow(clippy::too_many_arguments)]
fn build_catalog_publication_lifecycle_audit_record(
    before: &CommercialCatalogPublicationContext,
    after: Option<&CommercialCatalogPublicationContext>,
    action: CatalogPublicationLifecycleAction,
    outcome: CatalogPublicationLifecycleAuditOutcome,
    operator_id: &str,
    request_id: &str,
    operator_reason: &str,
    recorded_at_ms: u64,
    decision_reasons: Vec<String>,
) -> CatalogPublicationLifecycleAuditRecord {
    let after_projection = after
        .map(|context| &context.projection)
        .unwrap_or(&before.projection);
    let governed_pricing_plan_before = before.governed_pricing_plan.as_ref();
    let governed_pricing_plan_after = after
        .and_then(|context| context.governed_pricing_plan.as_ref())
        .or(governed_pricing_plan_before);

    CatalogPublicationLifecycleAuditRecord::new(
        format!(
            "catalog_publication_audit:{request_id}:{}:{}",
            before.projection.publication.publication_id,
            action.as_str()
        ),
        before.projection.publication.publication_id.clone(),
        before
            .projection
            .publication
            .publication_revision_id
            .clone(),
        before.projection.publication.publication_version,
        before
            .projection
            .publication
            .publication_source_kind
            .clone(),
        action,
        outcome,
        operator_id.to_owned(),
        request_id.to_owned(),
        operator_reason.to_owned(),
        before.projection.publication.status.as_str().to_owned(),
        after_projection.publication.status.as_str().to_owned(),
        recorded_at_ms,
    )
    .with_governed_pricing_plan_id(
        governed_pricing_plan_after
            .map(|plan| plan.pricing_plan_id)
            .or_else(|| governed_pricing_plan_before.map(|plan| plan.pricing_plan_id)),
    )
    .with_governed_pricing_status_before_option(
        governed_pricing_plan_before.map(|plan| plan.status.clone()),
    )
    .with_governed_pricing_status_after_option(
        governed_pricing_plan_after.map(|plan| plan.status.clone()),
    )
    .with_decision_reasons(decision_reasons)
}

async fn persist_catalog_publication_lifecycle_audit_record(
    state: &AdminApiState,
    record: &CatalogPublicationLifecycleAuditRecord,
) -> Result<CatalogPublicationLifecycleAuditRecord, (StatusCode, Json<ErrorResponse>)> {
    state
        .store
        .insert_catalog_publication_lifecycle_audit_record(record)
        .await
        .map_err(|error| {
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!(
                    "failed to persist publication lifecycle audit for {}: {error}",
                    record.publication_id
                ),
            )
        })
}

async fn apply_publish_commercial_catalog_publication(
    state: &AdminApiState,
    context: &CommercialCatalogPublicationContext,
    now_ms: u64,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let commercial_billing = commercial_billing_kernel(state)?.clone();
    let target_plan = context
        .governed_pricing_plan
        .as_ref()
        .ok_or_else(|| {
            error_response(
                StatusCode::BAD_REQUEST,
                format!(
                    "publication {} cannot be published: publication is derived from catalog_seed and has no governed pricing plan",
                    context.projection.publication.publication_id
                ),
            )
        })?;

    let active_sibling_plan_ids = context
        .pricing_plans
        .iter()
        .filter(|plan| {
            plan.pricing_plan_id != target_plan.pricing_plan_id
                && plan.tenant_id == target_plan.tenant_id
                && plan.organization_id == target_plan.organization_id
                && plan.plan_code == target_plan.plan_code
                && plan.status == "active"
        })
        .map(|plan| plan.pricing_plan_id)
        .collect::<Vec<_>>();

    let published_plan =
        crate::pricing::build_pricing_plan_with_status(target_plan, "active", now_ms);
    commercial_billing
        .insert_pricing_plan_record(&published_plan)
        .await
        .map_err(commercial_billing_error_response)?;

    for archived_plan in context.pricing_plans.iter().filter(|plan| {
        active_sibling_plan_ids.contains(&plan.pricing_plan_id)
    }) {
        let archived_plan =
            crate::pricing::build_pricing_plan_with_status(archived_plan, "archived", now_ms);
        commercial_billing
            .insert_pricing_plan_record(&archived_plan)
            .await
            .map_err(commercial_billing_error_response)?;
    }

    for rate in &context.governed_pricing_rates {
        let published_rate = crate::pricing::build_pricing_rate_with_status(rate, "active", now_ms);
        commercial_billing
            .insert_pricing_rate_record(&published_rate)
            .await
            .map_err(commercial_billing_error_response)?;
    }

    for rate in context.pricing_rates.iter().filter(|rate| {
        active_sibling_plan_ids.contains(&rate.pricing_plan_id)
    }) {
        let archived_rate =
            crate::pricing::build_pricing_rate_with_status(rate, "archived", now_ms);
        commercial_billing
            .insert_pricing_rate_record(&archived_rate)
            .await
            .map_err(commercial_billing_error_response)?;
    }

    Ok(())
}

async fn apply_schedule_commercial_catalog_publication(
    state: &AdminApiState,
    context: &CommercialCatalogPublicationContext,
    now_ms: u64,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let commercial_billing = commercial_billing_kernel(state)?.clone();
    let target_plan = context
        .governed_pricing_plan
        .as_ref()
        .ok_or_else(|| {
            error_response(
                StatusCode::BAD_REQUEST,
                format!(
                    "publication {} cannot be scheduled: publication is derived from catalog_seed and has no governed pricing plan",
                    context.projection.publication.publication_id
                ),
            )
        })?;

    let scheduled_plan =
        crate::pricing::build_pricing_plan_with_status(target_plan, "planned", now_ms);
    commercial_billing
        .insert_pricing_plan_record(&scheduled_plan)
        .await
        .map_err(commercial_billing_error_response)?;

    for rate in &context.governed_pricing_rates {
        let scheduled_rate =
            crate::pricing::build_pricing_rate_with_status(rate, "planned", now_ms);
        commercial_billing
            .insert_pricing_rate_record(&scheduled_rate)
            .await
            .map_err(commercial_billing_error_response)?;
    }

    Ok(())
}

async fn apply_retire_commercial_catalog_publication(
    state: &AdminApiState,
    context: &CommercialCatalogPublicationContext,
    now_ms: u64,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let commercial_billing = commercial_billing_kernel(state)?.clone();
    let target_plan = context
        .governed_pricing_plan
        .as_ref()
        .ok_or_else(|| {
            error_response(
                StatusCode::BAD_REQUEST,
                format!(
                    "publication {} cannot be retired: publication is derived from catalog_seed and has no governed pricing plan",
                    context.projection.publication.publication_id
                ),
            )
        })?;

    let retired_plan =
        crate::pricing::build_pricing_plan_with_status(target_plan, "archived", now_ms);
    commercial_billing
        .insert_pricing_plan_record(&retired_plan)
        .await
        .map_err(commercial_billing_error_response)?;

    for rate in &context.governed_pricing_rates {
        let retired_rate = crate::pricing::build_pricing_rate_with_status(rate, "archived", now_ms);
        commercial_billing
            .insert_pricing_rate_record(&retired_rate)
            .await
            .map_err(commercial_billing_error_response)?;
    }

    Ok(())
}

async fn mutate_commercial_catalog_publication(
    claims: AuthenticatedAdminClaims,
    request_id: RequestId,
    state: AdminApiState,
    publication_id: String,
    operator_reason: String,
    action: CatalogPublicationLifecycleAction,
) -> Result<Json<CommercialCatalogPublicationMutationResult>, (StatusCode, Json<ErrorResponse>)> {
    let before_context =
        load_commercial_catalog_publication_context(&state, &publication_id).await?;
    let before_detail =
        build_commercial_catalog_publication_detail(&before_context, unix_timestamp_ms());
    let decision = publication_mutation_decision(&before_detail, action);
    let recorded_at_ms = unix_timestamp_ms();
    let operator_id = claims.claims().sub.clone();
    let request_id_value = request_id.as_str().to_owned();

    if !decision.allowed {
        let audit = build_catalog_publication_lifecycle_audit_record(
            &before_context,
            None,
            action,
            CatalogPublicationLifecycleAuditOutcome::Rejected,
            &operator_id,
            &request_id_value,
            &operator_reason,
            recorded_at_ms,
            decision.reasons.clone(),
        );
        persist_catalog_publication_lifecycle_audit_record(&state, &audit).await?;
        let reason = decision
            .reasons
            .first()
            .cloned()
            .unwrap_or_else(|| "publication lifecycle action is not allowed".to_owned());
        return Err(error_response(
            StatusCode::BAD_REQUEST,
            format!(
                "publication {publication_id} cannot be {}: {reason}",
                publication_mutation_label(action)
            ),
        ));
    }

    match action {
        CatalogPublicationLifecycleAction::Publish => {
            apply_publish_commercial_catalog_publication(&state, &before_context, recorded_at_ms)
                .await?;
        }
        CatalogPublicationLifecycleAction::Schedule => {
            apply_schedule_commercial_catalog_publication(&state, &before_context, recorded_at_ms)
                .await?;
        }
        CatalogPublicationLifecycleAction::Retire => {
            apply_retire_commercial_catalog_publication(&state, &before_context, recorded_at_ms)
                .await?;
        }
    }

    let after_context =
        load_commercial_catalog_publication_context(&state, &publication_id).await?;
    let detail = build_commercial_catalog_publication_detail(&after_context, unix_timestamp_ms());
    let audit = build_catalog_publication_lifecycle_audit_record(
        &before_context,
        Some(&after_context),
        action,
        CatalogPublicationLifecycleAuditOutcome::Applied,
        &operator_id,
        &request_id_value,
        &operator_reason,
        recorded_at_ms,
        Vec::new(),
    );
    let audit = persist_catalog_publication_lifecycle_audit_record(&state, &audit).await?;

    Ok(Json(CommercialCatalogPublicationMutationResult {
        detail,
        audit,
    }))
}

pub(crate) async fn list_commercial_catalog_publications_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<CommercialCatalogPublicationProjection>>, (StatusCode, Json<ErrorResponse>)> {
    let commercial_billing = commercial_billing_kernel(&state)?.clone();
    synchronize_due_pricing_plan_lifecycle(commercial_billing.as_ref(), unix_timestamp_ms())
        .await
        .map_err(commercial_billing_error_response)?;

    let catalog = current_canonical_commercial_catalog_for_store(state.store.as_ref())
        .await
        .map_err(admin_commerce_error_response)?;
    let projections = build_commercial_catalog_publication_projections(&catalog)?;
    Ok(Json(projections))
}

pub(crate) async fn get_commercial_catalog_publication_handler(
    _claims: AuthenticatedAdminClaims,
    Path(publication_id): Path<String>,
    State(state): State<AdminApiState>,
) -> Result<Json<CommercialCatalogPublicationDetail>, (StatusCode, Json<ErrorResponse>)> {
    let context = load_commercial_catalog_publication_context(&state, &publication_id).await?;
    Ok(Json(build_commercial_catalog_publication_detail(
        &context,
        unix_timestamp_ms(),
    )))
}

pub(crate) async fn publish_commercial_catalog_publication_handler(
    claims: AuthenticatedAdminClaims,
    Path(publication_id): Path<String>,
    State(state): State<AdminApiState>,
    Extension(request_id): Extension<RequestId>,
    Json(request): Json<PublishCommercialCatalogPublicationRequest>,
) -> Result<Json<CommercialCatalogPublicationMutationResult>, (StatusCode, Json<ErrorResponse>)> {
    mutate_commercial_catalog_publication(
        claims,
        request_id,
        state,
        publication_id,
        normalized_publication_mutation_reason(&request.reason)?,
        CatalogPublicationLifecycleAction::Publish,
    )
    .await
}

pub(crate) async fn schedule_commercial_catalog_publication_handler(
    claims: AuthenticatedAdminClaims,
    Path(publication_id): Path<String>,
    State(state): State<AdminApiState>,
    Extension(request_id): Extension<RequestId>,
    Json(request): Json<ScheduleCommercialCatalogPublicationRequest>,
) -> Result<Json<CommercialCatalogPublicationMutationResult>, (StatusCode, Json<ErrorResponse>)> {
    mutate_commercial_catalog_publication(
        claims,
        request_id,
        state,
        publication_id,
        normalized_publication_mutation_reason(&request.reason)?,
        CatalogPublicationLifecycleAction::Schedule,
    )
    .await
}

pub(crate) async fn retire_commercial_catalog_publication_handler(
    claims: AuthenticatedAdminClaims,
    Path(publication_id): Path<String>,
    State(state): State<AdminApiState>,
    Extension(request_id): Extension<RequestId>,
    Json(request): Json<RetireCommercialCatalogPublicationRequest>,
) -> Result<Json<CommercialCatalogPublicationMutationResult>, (StatusCode, Json<ErrorResponse>)> {
    mutate_commercial_catalog_publication(
        claims,
        request_id,
        state,
        publication_id,
        normalized_publication_mutation_reason(&request.reason)?,
        CatalogPublicationLifecycleAction::Retire,
    )
    .await
}
