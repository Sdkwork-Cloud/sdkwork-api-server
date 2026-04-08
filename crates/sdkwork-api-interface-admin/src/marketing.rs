use super::*;

pub(crate) async fn list_coupons_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<CouponCampaign>>, StatusCode> {
    let mut coupons = list_coupons(state.store.as_ref())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let marketing_coupons = list_legacy_coupon_projections_from_marketing(state.store.as_ref())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let mut seen_codes = coupons
        .iter()
        .map(|coupon| coupon.code.to_ascii_uppercase())
        .collect::<HashSet<_>>();
    for coupon in marketing_coupons {
        if seen_codes.insert(coupon.code.to_ascii_uppercase()) {
            coupons.push(coupon);
        }
    }
    coupons.sort_by(|left, right| {
        left.code
            .cmp(&right.code)
            .then_with(|| left.id.cmp(&right.id))
    });
    Ok(Json(coupons))
}

pub(crate) async fn create_coupon_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(request): Json<CreateCouponRequest>,
) -> Result<(StatusCode, Json<CouponCampaign>), StatusCode> {
    let coupon = persist_coupon(
        state.store.as_ref(),
        &CouponCampaign::new(
            &request.id,
            &request.code,
            &request.discount_label,
            &request.audience,
            request.remaining,
            request.active,
            &request.note,
            &request.expires_on,
        ),
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    sync_legacy_coupon_marketing_projection(state.store.as_ref(), &coupon)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::CREATED, Json(coupon)))
}

pub(crate) async fn delete_coupon_handler(
    _claims: AuthenticatedAdminClaims,
    Path(coupon_id): Path<String>,
    State(state): State<AdminApiState>,
) -> Result<StatusCode, StatusCode> {
    let deleted = delete_coupon(state.store.as_ref(), &coupon_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

pub(crate) async fn list_marketing_coupon_templates_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<CouponTemplateRecord>>, StatusCode> {
    let mut templates = state
        .store
        .list_coupon_template_records()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    templates.sort_by(|left, right| left.template_key.cmp(&right.template_key));
    Ok(Json(templates))
}

pub(crate) async fn create_marketing_coupon_template_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(record): Json<CouponTemplateRecord>,
) -> Result<(StatusCode, Json<CouponTemplateRecord>), StatusCode> {
    let record = state
        .store
        .insert_coupon_template_record(&record)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::CREATED, Json(record)))
}

pub(crate) async fn update_marketing_coupon_template_status_handler(
    _claims: AuthenticatedAdminClaims,
    Path(coupon_template_id): Path<String>,
    State(state): State<AdminApiState>,
    Json(request): Json<UpdateCouponTemplateStatusRequest>,
) -> Result<Json<CouponTemplateRecord>, (StatusCode, Json<ErrorResponse>)> {
    update_marketing_coupon_template_status(
        state.store.as_ref(),
        &coupon_template_id,
        request.status,
    )
    .await
    .map(Json)
    .map_err(|(status, message)| error_response(status, message))
}

pub(crate) async fn list_marketing_campaigns_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<MarketingCampaignRecord>>, StatusCode> {
    let mut campaigns = state
        .store
        .list_marketing_campaign_records()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    campaigns.sort_by(|left, right| left.marketing_campaign_id.cmp(&right.marketing_campaign_id));
    Ok(Json(campaigns))
}

pub(crate) async fn create_marketing_campaign_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(record): Json<MarketingCampaignRecord>,
) -> Result<(StatusCode, Json<MarketingCampaignRecord>), StatusCode> {
    let record = state
        .store
        .insert_marketing_campaign_record(&record)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::CREATED, Json(record)))
}

pub(crate) async fn update_marketing_campaign_status_handler(
    _claims: AuthenticatedAdminClaims,
    Path(marketing_campaign_id): Path<String>,
    State(state): State<AdminApiState>,
    Json(request): Json<UpdateMarketingCampaignStatusRequest>,
) -> Result<Json<MarketingCampaignRecord>, (StatusCode, Json<ErrorResponse>)> {
    update_marketing_campaign_status(state.store.as_ref(), &marketing_campaign_id, request.status)
        .await
        .map(Json)
        .map_err(|(status, message)| error_response(status, message))
}

pub(crate) async fn list_marketing_budgets_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<CampaignBudgetRecord>>, StatusCode> {
    let mut budgets = state
        .store
        .list_campaign_budget_records()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    budgets.sort_by(|left, right| left.campaign_budget_id.cmp(&right.campaign_budget_id));
    Ok(Json(budgets))
}

pub(crate) async fn create_marketing_budget_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(record): Json<CampaignBudgetRecord>,
) -> Result<(StatusCode, Json<CampaignBudgetRecord>), StatusCode> {
    let record = state
        .store
        .insert_campaign_budget_record(&record)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::CREATED, Json(record)))
}

pub(crate) async fn update_marketing_budget_status_handler(
    _claims: AuthenticatedAdminClaims,
    Path(campaign_budget_id): Path<String>,
    State(state): State<AdminApiState>,
    Json(request): Json<UpdateCampaignBudgetStatusRequest>,
) -> Result<Json<CampaignBudgetRecord>, (StatusCode, Json<ErrorResponse>)> {
    update_marketing_campaign_budget_status(
        state.store.as_ref(),
        &campaign_budget_id,
        request.status,
    )
    .await
    .map(Json)
    .map_err(|(status, message)| error_response(status, message))
}

pub(crate) async fn list_marketing_coupon_codes_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<CouponCodeRecord>>, StatusCode> {
    let mut codes = state
        .store
        .list_coupon_code_records()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    codes.sort_by(|left, right| left.code_value.cmp(&right.code_value));
    Ok(Json(codes))
}

pub(crate) async fn create_marketing_coupon_code_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(record): Json<CouponCodeRecord>,
) -> Result<(StatusCode, Json<CouponCodeRecord>), StatusCode> {
    let record = state
        .store
        .insert_coupon_code_record(&record)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::CREATED, Json(record)))
}

pub(crate) async fn update_marketing_coupon_code_status_handler(
    _claims: AuthenticatedAdminClaims,
    Path(coupon_code_id): Path<String>,
    State(state): State<AdminApiState>,
    Json(request): Json<UpdateCouponCodeStatusRequest>,
) -> Result<Json<CouponCodeRecord>, (StatusCode, Json<ErrorResponse>)> {
    update_marketing_coupon_code_status(state.store.as_ref(), &coupon_code_id, request.status)
        .await
        .map(Json)
        .map_err(|(status, message)| error_response(status, message))
}

pub(crate) async fn list_marketing_coupon_reservations_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<CouponReservationRecord>>, StatusCode> {
    let mut reservations = state
        .store
        .list_coupon_reservation_records()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    reservations
        .sort_by(|left, right| left.coupon_reservation_id.cmp(&right.coupon_reservation_id));
    Ok(Json(reservations))
}

pub(crate) async fn list_marketing_coupon_redemptions_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<CouponRedemptionRecord>>, StatusCode> {
    let mut redemptions = state
        .store
        .list_coupon_redemption_records()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    redemptions.sort_by(|left, right| left.coupon_redemption_id.cmp(&right.coupon_redemption_id));
    Ok(Json(redemptions))
}

pub(crate) async fn list_marketing_coupon_rollbacks_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<CouponRollbackRecord>>, StatusCode> {
    let mut rollbacks = state
        .store
        .list_coupon_rollback_records()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    rollbacks.sort_by(|left, right| left.coupon_rollback_id.cmp(&right.coupon_rollback_id));
    Ok(Json(rollbacks))
}

async fn sync_legacy_coupon_marketing_projection(
    store: &dyn AdminStore,
    coupon: &CouponCampaign,
) -> anyhow::Result<()> {
    let (template, campaign, budget, code) = project_legacy_coupon_campaign(coupon);
    store.insert_coupon_template_record(&template).await?;
    store.insert_marketing_campaign_record(&campaign).await?;
    store.insert_campaign_budget_record(&budget).await?;
    store.insert_coupon_code_record(&code).await?;
    Ok(())
}

async fn update_marketing_coupon_template_status(
    store: &dyn AdminStore,
    coupon_template_id: &str,
    status: CouponTemplateStatus,
) -> Result<CouponTemplateRecord, (StatusCode, String)> {
    let record = store
        .find_coupon_template_record(coupon_template_id)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to load canonical coupon template".to_owned(),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                format!("coupon template {coupon_template_id} not found"),
            )
        })?;
    let updated = record
        .with_status(status)
        .with_updated_at_ms(unix_timestamp_ms());
    store
        .insert_coupon_template_record(&updated)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to persist canonical coupon template status".to_owned(),
            )
        })
}

async fn update_marketing_campaign_status(
    store: &dyn AdminStore,
    marketing_campaign_id: &str,
    status: MarketingCampaignStatus,
) -> Result<MarketingCampaignRecord, (StatusCode, String)> {
    let record = store
        .list_marketing_campaign_records()
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to load canonical marketing campaigns".to_owned(),
            )
        })?
        .into_iter()
        .find(|record| record.marketing_campaign_id == marketing_campaign_id)
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                format!("marketing campaign {marketing_campaign_id} not found"),
            )
        })?;
    let updated = record
        .with_status(status)
        .with_updated_at_ms(unix_timestamp_ms());
    store
        .insert_marketing_campaign_record(&updated)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to persist canonical marketing campaign status".to_owned(),
            )
        })
}

async fn update_marketing_campaign_budget_status(
    store: &dyn AdminStore,
    campaign_budget_id: &str,
    status: CampaignBudgetStatus,
) -> Result<CampaignBudgetRecord, (StatusCode, String)> {
    let record = store
        .list_campaign_budget_records()
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to load canonical campaign budgets".to_owned(),
            )
        })?
        .into_iter()
        .find(|record| record.campaign_budget_id == campaign_budget_id)
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                format!("campaign budget {campaign_budget_id} not found"),
            )
        })?;
    let updated = record
        .with_status(status)
        .with_updated_at_ms(unix_timestamp_ms());
    store
        .insert_campaign_budget_record(&updated)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to persist canonical campaign budget status".to_owned(),
            )
        })
}

async fn update_marketing_coupon_code_status(
    store: &dyn AdminStore,
    coupon_code_id: &str,
    status: CouponCodeStatus,
) -> Result<CouponCodeRecord, (StatusCode, String)> {
    let record = store
        .find_coupon_code_record(coupon_code_id)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to load canonical coupon code".to_owned(),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                format!("coupon code {coupon_code_id} not found"),
            )
        })?;
    let updated = record
        .with_status(status)
        .with_updated_at_ms(unix_timestamp_ms());
    store
        .insert_coupon_code_record(&updated)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to persist canonical coupon code status".to_owned(),
            )
        })
}

async fn list_legacy_coupon_projections_from_marketing(
    store: &dyn AdminStore,
) -> anyhow::Result<Vec<CouponCampaign>> {
    let templates = store.list_coupon_template_records().await?;
    let campaigns = store.list_marketing_campaign_records().await?;
    let budgets = store.list_campaign_budget_records().await?;
    let codes = store.list_coupon_code_records().await?;

    let mut preferred_campaigns_by_template = HashMap::<String, MarketingCampaignRecord>::new();
    for campaign in campaigns {
        match preferred_campaigns_by_template.get_mut(&campaign.coupon_template_id) {
            Some(existing) if should_replace_marketing_campaign(existing, &campaign) => {
                *existing = campaign;
            }
            None => {
                preferred_campaigns_by_template
                    .insert(campaign.coupon_template_id.clone(), campaign);
            }
            Some(_) => {}
        }
    }

    let mut preferred_budgets_by_campaign = HashMap::<String, CampaignBudgetRecord>::new();
    for budget in budgets {
        match preferred_budgets_by_campaign.get_mut(&budget.marketing_campaign_id) {
            Some(existing) if should_replace_campaign_budget(existing, &budget) => {
                *existing = budget;
            }
            None => {
                preferred_budgets_by_campaign.insert(budget.marketing_campaign_id.clone(), budget);
            }
            Some(_) => {}
        }
    }

    let mut coupon_codes_by_template = HashMap::<String, Vec<CouponCodeRecord>>::new();
    for code in codes {
        coupon_codes_by_template
            .entry(code.coupon_template_id.clone())
            .or_default()
            .push(code);
    }
    for template_codes in coupon_codes_by_template.values_mut() {
        template_codes.sort_by(|left, right| left.code_value.cmp(&right.code_value));
    }

    let now_ms = unix_timestamp_ms();
    let mut coupons = Vec::new();
    for template in templates {
        if template.coupon_template_id.starts_with("legacy_tpl_") {
            continue;
        }
        let Some(template_codes) = coupon_codes_by_template.get(&template.coupon_template_id)
        else {
            continue;
        };
        let campaign = preferred_campaigns_by_template.get(&template.coupon_template_id);
        let budget = campaign
            .and_then(|record| preferred_budgets_by_campaign.get(&record.marketing_campaign_id));
        for code in template_codes {
            coupons.push(project_marketing_coupon_to_legacy(
                &template, campaign, budget, code, now_ms,
            ));
        }
    }

    coupons.sort_by(|left, right| {
        left.code
            .cmp(&right.code)
            .then_with(|| left.id.cmp(&right.id))
    });
    Ok(coupons)
}

fn should_replace_marketing_campaign(
    existing: &MarketingCampaignRecord,
    candidate: &MarketingCampaignRecord,
) -> bool {
    marketing_campaign_priority(candidate.status) > marketing_campaign_priority(existing.status)
        || (marketing_campaign_priority(candidate.status)
            == marketing_campaign_priority(existing.status)
            && candidate.updated_at_ms > existing.updated_at_ms)
}

fn should_replace_campaign_budget(
    existing: &CampaignBudgetRecord,
    candidate: &CampaignBudgetRecord,
) -> bool {
    campaign_budget_priority(candidate.status) > campaign_budget_priority(existing.status)
        || (campaign_budget_priority(candidate.status) == campaign_budget_priority(existing.status)
            && candidate.updated_at_ms > existing.updated_at_ms)
}

fn marketing_campaign_priority(status: MarketingCampaignStatus) -> u8 {
    match status {
        MarketingCampaignStatus::Active => 5,
        MarketingCampaignStatus::Scheduled => 4,
        MarketingCampaignStatus::Paused => 3,
        MarketingCampaignStatus::Draft => 2,
        MarketingCampaignStatus::Ended => 1,
        MarketingCampaignStatus::Archived => 0,
    }
}

fn campaign_budget_priority(status: CampaignBudgetStatus) -> u8 {
    match status {
        CampaignBudgetStatus::Active => 3,
        CampaignBudgetStatus::Exhausted => 2,
        CampaignBudgetStatus::Draft => 1,
        CampaignBudgetStatus::Closed => 0,
    }
}

fn project_marketing_coupon_to_legacy(
    template: &CouponTemplateRecord,
    campaign: Option<&MarketingCampaignRecord>,
    budget: Option<&CampaignBudgetRecord>,
    code: &CouponCodeRecord,
    now_ms: u64,
) -> CouponCampaign {
    let active = marketing_coupon_is_active(template, campaign, budget, code, now_ms);
    let remaining = budget
        .map(CampaignBudgetRecord::available_budget_minor)
        .unwrap_or(u64::from(active));
    let expires_on = code
        .expires_at_ms
        .or_else(|| campaign.and_then(|record| record.end_at_ms))
        .map(|value| value.to_string())
        .unwrap_or_else(|| "2099-12-31".to_owned());
    let created_at_ms = code
        .created_at_ms
        .max(template.created_at_ms)
        .max(campaign.map_or(0, |record| record.created_at_ms))
        .max(budget.map_or(0, |record| record.created_at_ms));
    let note = marketing_coupon_note(template, campaign);

    CouponCampaign::new(
        code.coupon_code_id.clone(),
        code.code_value.clone(),
        marketing_coupon_discount_label(template),
        marketing_coupon_audience(template.restriction.subject_scope),
        remaining,
        active,
        note,
        expires_on,
    )
    .with_created_at_ms(created_at_ms)
}

fn marketing_coupon_discount_label(template: &CouponTemplateRecord) -> String {
    match template.benefit.benefit_kind {
        MarketingBenefitKind::PercentageOff => template
            .benefit
            .discount_percent
            .map(|value| format!("{value}% off"))
            .unwrap_or_else(|| marketing_coupon_note(template, None)),
        MarketingBenefitKind::FixedAmountOff => template
            .benefit
            .discount_amount_minor
            .map(|value| format!("{value} off"))
            .unwrap_or_else(|| marketing_coupon_note(template, None)),
        MarketingBenefitKind::GrantUnits => template
            .benefit
            .grant_units
            .map(|value| format!("{value} units"))
            .unwrap_or_else(|| marketing_coupon_note(template, None)),
    }
}

fn marketing_coupon_audience(scope: MarketingSubjectScope) -> String {
    match scope {
        MarketingSubjectScope::User => "user",
        MarketingSubjectScope::Project => "project",
        MarketingSubjectScope::Workspace => "workspace",
        MarketingSubjectScope::Account => "account",
    }
    .to_owned()
}

fn marketing_coupon_note(
    template: &CouponTemplateRecord,
    campaign: Option<&MarketingCampaignRecord>,
) -> String {
    campaign
        .map(|record| record.display_name.trim())
        .filter(|value| !value.is_empty())
        .map(|value| value.to_owned())
        .or_else(|| {
            let display_name = template.display_name.trim();
            (!display_name.is_empty()).then(|| display_name.to_owned())
        })
        .unwrap_or_else(|| template.template_key.clone())
}

fn marketing_coupon_is_active(
    template: &CouponTemplateRecord,
    campaign: Option<&MarketingCampaignRecord>,
    budget: Option<&CampaignBudgetRecord>,
    code: &CouponCodeRecord,
    now_ms: u64,
) -> bool {
    let template_active = template.status == CouponTemplateStatus::Active;
    let campaign_active = campaign.is_none_or(|record| record.is_effective_at(now_ms));
    let budget_active = budget.is_none_or(|record| {
        record.status == CampaignBudgetStatus::Active && record.available_budget_minor() > 0
    });
    let code_active = matches!(
        code.status,
        CouponCodeStatus::Available | CouponCodeStatus::Reserved
    ) && code.expires_at_ms.is_none_or(|value| now_ms <= value);

    template_active && campaign_active && budget_active && code_active
}
