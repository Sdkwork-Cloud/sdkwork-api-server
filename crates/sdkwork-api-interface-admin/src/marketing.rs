use super::*;
use axum::Extension;
use sdkwork_api_observability::RequestId;

#[derive(Debug, Clone, Serialize, ToSchema)]
pub(crate) struct CouponTemplateActionDecision {
    pub(crate) allowed: bool,
    #[serde(default)]
    pub(crate) reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub(crate) struct CouponTemplateActionability {
    pub(crate) clone: CouponTemplateActionDecision,
    pub(crate) submit_for_approval: CouponTemplateActionDecision,
    pub(crate) approve: CouponTemplateActionDecision,
    pub(crate) reject: CouponTemplateActionDecision,
    pub(crate) publish: CouponTemplateActionDecision,
    pub(crate) schedule: CouponTemplateActionDecision,
    pub(crate) retire: CouponTemplateActionDecision,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub(crate) struct CouponTemplateDetail {
    pub(crate) coupon_template: CouponTemplateRecord,
    pub(crate) actionability: CouponTemplateActionability,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub(crate) struct CouponTemplateMutationResult {
    pub(crate) detail: CouponTemplateDetail,
    pub(crate) audit: CouponTemplateLifecycleAuditRecord,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub(crate) struct CouponTemplateComparisonFieldChange {
    pub(crate) field: String,
    pub(crate) source_value: String,
    pub(crate) target_value: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub(crate) struct CouponTemplateComparisonResult {
    pub(crate) source_coupon_template: CouponTemplateRecord,
    pub(crate) target_coupon_template: CouponTemplateRecord,
    pub(crate) same_lineage: bool,
    #[serde(default)]
    pub(crate) field_changes: Vec<CouponTemplateComparisonFieldChange>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub(crate) struct MarketingCampaignActionDecision {
    pub(crate) allowed: bool,
    #[serde(default)]
    pub(crate) reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub(crate) struct MarketingCampaignActionability {
    pub(crate) clone: MarketingCampaignActionDecision,
    pub(crate) submit_for_approval: MarketingCampaignActionDecision,
    pub(crate) approve: MarketingCampaignActionDecision,
    pub(crate) reject: MarketingCampaignActionDecision,
    pub(crate) publish: MarketingCampaignActionDecision,
    pub(crate) schedule: MarketingCampaignActionDecision,
    pub(crate) retire: MarketingCampaignActionDecision,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub(crate) struct MarketingCampaignDetail {
    pub(crate) campaign: MarketingCampaignRecord,
    pub(crate) coupon_template: CouponTemplateRecord,
    pub(crate) actionability: MarketingCampaignActionability,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub(crate) struct MarketingCampaignMutationResult {
    pub(crate) detail: MarketingCampaignDetail,
    pub(crate) audit: MarketingCampaignLifecycleAuditRecord,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub(crate) struct MarketingCampaignComparisonFieldChange {
    pub(crate) field: String,
    pub(crate) source_value: String,
    pub(crate) target_value: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub(crate) struct MarketingCampaignComparisonResult {
    pub(crate) source_marketing_campaign: MarketingCampaignRecord,
    pub(crate) target_marketing_campaign: MarketingCampaignRecord,
    pub(crate) same_lineage: bool,
    #[serde(default)]
    pub(crate) field_changes: Vec<MarketingCampaignComparisonFieldChange>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub(crate) struct CampaignBudgetActionDecision {
    pub(crate) allowed: bool,
    #[serde(default)]
    pub(crate) reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub(crate) struct CampaignBudgetActionability {
    pub(crate) activate: CampaignBudgetActionDecision,
    pub(crate) close: CampaignBudgetActionDecision,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub(crate) struct CampaignBudgetDetail {
    pub(crate) budget: CampaignBudgetRecord,
    pub(crate) campaign: MarketingCampaignRecord,
    pub(crate) actionability: CampaignBudgetActionability,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub(crate) struct CampaignBudgetMutationResult {
    pub(crate) detail: CampaignBudgetDetail,
    pub(crate) audit: CampaignBudgetLifecycleAuditRecord,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub(crate) struct CouponCodeActionDecision {
    pub(crate) allowed: bool,
    #[serde(default)]
    pub(crate) reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub(crate) struct CouponCodeActionability {
    pub(crate) disable: CouponCodeActionDecision,
    pub(crate) restore: CouponCodeActionDecision,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub(crate) struct CouponCodeDetail {
    pub(crate) coupon_code: CouponCodeRecord,
    pub(crate) coupon_template: CouponTemplateRecord,
    pub(crate) actionability: CouponCodeActionability,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub(crate) struct CouponCodeMutationResult {
    pub(crate) detail: CouponCodeDetail,
    pub(crate) audit: CouponCodeLifecycleAuditRecord,
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
    let record = normalize_coupon_template_record_for_admin_upsert(record);
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

pub(crate) async fn clone_marketing_coupon_template_handler(
    claims: AuthenticatedAdminClaims,
    Path(coupon_template_id): Path<String>,
    State(state): State<AdminApiState>,
    Extension(request_id): Extension<RequestId>,
    Json(request): Json<CloneCouponTemplateRequest>,
) -> Result<(StatusCode, Json<CouponTemplateMutationResult>), (StatusCode, Json<ErrorResponse>)> {
    let reason = normalized_marketing_lifecycle_reason(&request.reason, "coupon template")?;
    clone_marketing_coupon_template_revision(
        state.store.as_ref(),
        &coupon_template_id,
        request,
        &claims.claims().sub,
        request_id.as_str(),
        &reason,
    )
    .await
    .map(|result| (StatusCode::CREATED, Json(result)))
}

pub(crate) async fn compare_marketing_coupon_templates_handler(
    _claims: AuthenticatedAdminClaims,
    Path(coupon_template_id): Path<String>,
    State(state): State<AdminApiState>,
    Json(request): Json<CompareCouponTemplateRequest>,
) -> Result<Json<CouponTemplateComparisonResult>, (StatusCode, Json<ErrorResponse>)> {
    compare_marketing_coupon_template_revisions(
        state.store.as_ref(),
        &coupon_template_id,
        &request.target_coupon_template_id,
    )
    .await
    .map(Json)
}

pub(crate) async fn submit_marketing_coupon_template_for_approval_handler(
    claims: AuthenticatedAdminClaims,
    Path(coupon_template_id): Path<String>,
    State(state): State<AdminApiState>,
    Extension(request_id): Extension<RequestId>,
    Json(request): Json<SubmitCouponTemplateForApprovalRequest>,
) -> Result<Json<CouponTemplateMutationResult>, (StatusCode, Json<ErrorResponse>)> {
    let reason = normalized_marketing_lifecycle_reason(&request.reason, "coupon template")?;
    mutate_marketing_coupon_template_lifecycle(
        state.store.as_ref(),
        &coupon_template_id,
        CouponTemplateLifecycleAction::SubmitForApproval,
        &claims.claims().sub,
        request_id.as_str(),
        &reason,
    )
    .await
    .map(Json)
}

pub(crate) async fn approve_marketing_coupon_template_handler(
    claims: AuthenticatedAdminClaims,
    Path(coupon_template_id): Path<String>,
    State(state): State<AdminApiState>,
    Extension(request_id): Extension<RequestId>,
    Json(request): Json<ApproveCouponTemplateRequest>,
) -> Result<Json<CouponTemplateMutationResult>, (StatusCode, Json<ErrorResponse>)> {
    let reason = normalized_marketing_lifecycle_reason(&request.reason, "coupon template")?;
    mutate_marketing_coupon_template_lifecycle(
        state.store.as_ref(),
        &coupon_template_id,
        CouponTemplateLifecycleAction::Approve,
        &claims.claims().sub,
        request_id.as_str(),
        &reason,
    )
    .await
    .map(Json)
}

pub(crate) async fn reject_marketing_coupon_template_handler(
    claims: AuthenticatedAdminClaims,
    Path(coupon_template_id): Path<String>,
    State(state): State<AdminApiState>,
    Extension(request_id): Extension<RequestId>,
    Json(request): Json<RejectCouponTemplateRequest>,
) -> Result<Json<CouponTemplateMutationResult>, (StatusCode, Json<ErrorResponse>)> {
    let reason = normalized_marketing_lifecycle_reason(&request.reason, "coupon template")?;
    mutate_marketing_coupon_template_lifecycle(
        state.store.as_ref(),
        &coupon_template_id,
        CouponTemplateLifecycleAction::Reject,
        &claims.claims().sub,
        request_id.as_str(),
        &reason,
    )
    .await
    .map(Json)
}

pub(crate) async fn publish_marketing_coupon_template_handler(
    claims: AuthenticatedAdminClaims,
    Path(coupon_template_id): Path<String>,
    State(state): State<AdminApiState>,
    Extension(request_id): Extension<RequestId>,
    Json(request): Json<PublishCouponTemplateRequest>,
) -> Result<Json<CouponTemplateMutationResult>, (StatusCode, Json<ErrorResponse>)> {
    let reason = normalized_marketing_lifecycle_reason(&request.reason, "coupon template")?;
    mutate_marketing_coupon_template_lifecycle(
        state.store.as_ref(),
        &coupon_template_id,
        CouponTemplateLifecycleAction::Publish,
        &claims.claims().sub,
        request_id.as_str(),
        &reason,
    )
    .await
    .map(Json)
}

pub(crate) async fn schedule_marketing_coupon_template_handler(
    claims: AuthenticatedAdminClaims,
    Path(coupon_template_id): Path<String>,
    State(state): State<AdminApiState>,
    Extension(request_id): Extension<RequestId>,
    Json(request): Json<ScheduleCouponTemplateRequest>,
) -> Result<Json<CouponTemplateMutationResult>, (StatusCode, Json<ErrorResponse>)> {
    let reason = normalized_marketing_lifecycle_reason(&request.reason, "coupon template")?;
    mutate_marketing_coupon_template_lifecycle(
        state.store.as_ref(),
        &coupon_template_id,
        CouponTemplateLifecycleAction::Schedule,
        &claims.claims().sub,
        request_id.as_str(),
        &reason,
    )
    .await
    .map(Json)
}

pub(crate) async fn retire_marketing_coupon_template_handler(
    claims: AuthenticatedAdminClaims,
    Path(coupon_template_id): Path<String>,
    State(state): State<AdminApiState>,
    Extension(request_id): Extension<RequestId>,
    Json(request): Json<RetireCouponTemplateRequest>,
) -> Result<Json<CouponTemplateMutationResult>, (StatusCode, Json<ErrorResponse>)> {
    let reason = normalized_marketing_lifecycle_reason(&request.reason, "coupon template")?;
    mutate_marketing_coupon_template_lifecycle(
        state.store.as_ref(),
        &coupon_template_id,
        CouponTemplateLifecycleAction::Retire,
        &claims.claims().sub,
        request_id.as_str(),
        &reason,
    )
    .await
    .map(Json)
}

pub(crate) async fn list_marketing_coupon_template_lifecycle_audits_handler(
    _claims: AuthenticatedAdminClaims,
    Path(coupon_template_id): Path<String>,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<CouponTemplateLifecycleAuditRecord>>, (StatusCode, Json<ErrorResponse>)> {
    state
        .store
        .list_coupon_template_lifecycle_audit_records_for_template(&coupon_template_id)
        .await
        .map(Json)
        .map_err(|error| {
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!(
                    "failed to load coupon template lifecycle audits for {coupon_template_id}: {error}"
                ),
            )
        })
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
    let record = normalize_marketing_campaign_record_for_admin_upsert(record);
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

pub(crate) async fn clone_marketing_campaign_handler(
    claims: AuthenticatedAdminClaims,
    Path(marketing_campaign_id): Path<String>,
    State(state): State<AdminApiState>,
    Extension(request_id): Extension<RequestId>,
    Json(request): Json<CloneMarketingCampaignRequest>,
) -> Result<(StatusCode, Json<MarketingCampaignMutationResult>), (StatusCode, Json<ErrorResponse>)>
{
    let reason = normalized_marketing_lifecycle_reason(&request.reason, "campaign")?;
    clone_marketing_campaign_revision(
        state.store.as_ref(),
        &marketing_campaign_id,
        request,
        &claims.claims().sub,
        request_id.as_str(),
        &reason,
    )
    .await
    .map(|result| (StatusCode::CREATED, Json(result)))
}

pub(crate) async fn compare_marketing_campaigns_handler(
    _claims: AuthenticatedAdminClaims,
    Path(marketing_campaign_id): Path<String>,
    State(state): State<AdminApiState>,
    Json(request): Json<CompareMarketingCampaignRequest>,
) -> Result<Json<MarketingCampaignComparisonResult>, (StatusCode, Json<ErrorResponse>)> {
    compare_marketing_campaign_revisions(
        state.store.as_ref(),
        &marketing_campaign_id,
        &request.target_marketing_campaign_id,
    )
    .await
    .map(Json)
}

pub(crate) async fn submit_marketing_campaign_for_approval_handler(
    claims: AuthenticatedAdminClaims,
    Path(marketing_campaign_id): Path<String>,
    State(state): State<AdminApiState>,
    Extension(request_id): Extension<RequestId>,
    Json(request): Json<SubmitMarketingCampaignForApprovalRequest>,
) -> Result<Json<MarketingCampaignMutationResult>, (StatusCode, Json<ErrorResponse>)> {
    let reason = normalized_marketing_lifecycle_reason(&request.reason, "campaign")?;
    mutate_marketing_campaign_lifecycle(
        state.store.as_ref(),
        &marketing_campaign_id,
        MarketingCampaignLifecycleAction::SubmitForApproval,
        &claims.claims().sub,
        request_id.as_str(),
        &reason,
    )
    .await
    .map(Json)
}

pub(crate) async fn approve_marketing_campaign_handler(
    claims: AuthenticatedAdminClaims,
    Path(marketing_campaign_id): Path<String>,
    State(state): State<AdminApiState>,
    Extension(request_id): Extension<RequestId>,
    Json(request): Json<ApproveMarketingCampaignRequest>,
) -> Result<Json<MarketingCampaignMutationResult>, (StatusCode, Json<ErrorResponse>)> {
    let reason = normalized_marketing_lifecycle_reason(&request.reason, "campaign")?;
    mutate_marketing_campaign_lifecycle(
        state.store.as_ref(),
        &marketing_campaign_id,
        MarketingCampaignLifecycleAction::Approve,
        &claims.claims().sub,
        request_id.as_str(),
        &reason,
    )
    .await
    .map(Json)
}

pub(crate) async fn reject_marketing_campaign_handler(
    claims: AuthenticatedAdminClaims,
    Path(marketing_campaign_id): Path<String>,
    State(state): State<AdminApiState>,
    Extension(request_id): Extension<RequestId>,
    Json(request): Json<RejectMarketingCampaignRequest>,
) -> Result<Json<MarketingCampaignMutationResult>, (StatusCode, Json<ErrorResponse>)> {
    let reason = normalized_marketing_lifecycle_reason(&request.reason, "campaign")?;
    mutate_marketing_campaign_lifecycle(
        state.store.as_ref(),
        &marketing_campaign_id,
        MarketingCampaignLifecycleAction::Reject,
        &claims.claims().sub,
        request_id.as_str(),
        &reason,
    )
    .await
    .map(Json)
}

pub(crate) async fn publish_marketing_campaign_handler(
    claims: AuthenticatedAdminClaims,
    Path(marketing_campaign_id): Path<String>,
    State(state): State<AdminApiState>,
    Extension(request_id): Extension<RequestId>,
    Json(request): Json<PublishMarketingCampaignRequest>,
) -> Result<Json<MarketingCampaignMutationResult>, (StatusCode, Json<ErrorResponse>)> {
    let reason = normalized_marketing_lifecycle_reason(&request.reason, "campaign")?;
    mutate_marketing_campaign_lifecycle(
        state.store.as_ref(),
        &marketing_campaign_id,
        MarketingCampaignLifecycleAction::Publish,
        &claims.claims().sub,
        request_id.as_str(),
        &reason,
    )
    .await
    .map(Json)
}

pub(crate) async fn schedule_marketing_campaign_handler(
    claims: AuthenticatedAdminClaims,
    Path(marketing_campaign_id): Path<String>,
    State(state): State<AdminApiState>,
    Extension(request_id): Extension<RequestId>,
    Json(request): Json<ScheduleMarketingCampaignRequest>,
) -> Result<Json<MarketingCampaignMutationResult>, (StatusCode, Json<ErrorResponse>)> {
    let reason = normalized_marketing_lifecycle_reason(&request.reason, "campaign")?;
    mutate_marketing_campaign_lifecycle(
        state.store.as_ref(),
        &marketing_campaign_id,
        MarketingCampaignLifecycleAction::Schedule,
        &claims.claims().sub,
        request_id.as_str(),
        &reason,
    )
    .await
    .map(Json)
}

pub(crate) async fn retire_marketing_campaign_handler(
    claims: AuthenticatedAdminClaims,
    Path(marketing_campaign_id): Path<String>,
    State(state): State<AdminApiState>,
    Extension(request_id): Extension<RequestId>,
    Json(request): Json<RetireMarketingCampaignRequest>,
) -> Result<Json<MarketingCampaignMutationResult>, (StatusCode, Json<ErrorResponse>)> {
    let reason = normalized_marketing_lifecycle_reason(&request.reason, "campaign")?;
    mutate_marketing_campaign_lifecycle(
        state.store.as_ref(),
        &marketing_campaign_id,
        MarketingCampaignLifecycleAction::Retire,
        &claims.claims().sub,
        request_id.as_str(),
        &reason,
    )
    .await
    .map(Json)
}

pub(crate) async fn list_marketing_campaign_lifecycle_audits_handler(
    _claims: AuthenticatedAdminClaims,
    Path(marketing_campaign_id): Path<String>,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<MarketingCampaignLifecycleAuditRecord>>, (StatusCode, Json<ErrorResponse>)> {
    state
        .store
        .list_marketing_campaign_lifecycle_audit_records_for_campaign(&marketing_campaign_id)
        .await
        .map(Json)
        .map_err(|error| {
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!(
                    "failed to load marketing campaign lifecycle audits for {marketing_campaign_id}: {error}"
                ),
            )
        })
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

pub(crate) async fn activate_marketing_campaign_budget_handler(
    claims: AuthenticatedAdminClaims,
    Path(campaign_budget_id): Path<String>,
    State(state): State<AdminApiState>,
    Extension(request_id): Extension<RequestId>,
    Json(request): Json<ActivateCampaignBudgetRequest>,
) -> Result<Json<CampaignBudgetMutationResult>, (StatusCode, Json<ErrorResponse>)> {
    let reason = normalized_marketing_lifecycle_reason(&request.reason, "campaign budget")?;
    mutate_marketing_campaign_budget_lifecycle(
        state.store.as_ref(),
        &campaign_budget_id,
        CampaignBudgetLifecycleAction::Activate,
        &claims.claims().sub,
        request_id.as_str(),
        &reason,
    )
    .await
    .map(Json)
}

pub(crate) async fn close_marketing_campaign_budget_handler(
    claims: AuthenticatedAdminClaims,
    Path(campaign_budget_id): Path<String>,
    State(state): State<AdminApiState>,
    Extension(request_id): Extension<RequestId>,
    Json(request): Json<CloseCampaignBudgetRequest>,
) -> Result<Json<CampaignBudgetMutationResult>, (StatusCode, Json<ErrorResponse>)> {
    let reason = normalized_marketing_lifecycle_reason(&request.reason, "campaign budget")?;
    mutate_marketing_campaign_budget_lifecycle(
        state.store.as_ref(),
        &campaign_budget_id,
        CampaignBudgetLifecycleAction::Close,
        &claims.claims().sub,
        request_id.as_str(),
        &reason,
    )
    .await
    .map(Json)
}

pub(crate) async fn list_marketing_campaign_budget_lifecycle_audits_handler(
    _claims: AuthenticatedAdminClaims,
    Path(campaign_budget_id): Path<String>,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<CampaignBudgetLifecycleAuditRecord>>, (StatusCode, Json<ErrorResponse>)> {
    state
        .store
        .list_campaign_budget_lifecycle_audit_records_for_budget(&campaign_budget_id)
        .await
        .map(Json)
        .map_err(|error| {
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!(
                    "failed to load campaign budget lifecycle audits for {campaign_budget_id}: {error}"
                ),
            )
        })
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

pub(crate) async fn disable_marketing_coupon_code_handler(
    claims: AuthenticatedAdminClaims,
    Path(coupon_code_id): Path<String>,
    State(state): State<AdminApiState>,
    Extension(request_id): Extension<RequestId>,
    Json(request): Json<DisableCouponCodeRequest>,
) -> Result<Json<CouponCodeMutationResult>, (StatusCode, Json<ErrorResponse>)> {
    let reason = normalized_marketing_lifecycle_reason(&request.reason, "coupon code")?;
    mutate_marketing_coupon_code_lifecycle(
        state.store.as_ref(),
        &coupon_code_id,
        CouponCodeLifecycleAction::Disable,
        &claims.claims().sub,
        request_id.as_str(),
        &reason,
    )
    .await
    .map(Json)
}

pub(crate) async fn restore_marketing_coupon_code_handler(
    claims: AuthenticatedAdminClaims,
    Path(coupon_code_id): Path<String>,
    State(state): State<AdminApiState>,
    Extension(request_id): Extension<RequestId>,
    Json(request): Json<RestoreCouponCodeRequest>,
) -> Result<Json<CouponCodeMutationResult>, (StatusCode, Json<ErrorResponse>)> {
    let reason = normalized_marketing_lifecycle_reason(&request.reason, "coupon code")?;
    mutate_marketing_coupon_code_lifecycle(
        state.store.as_ref(),
        &coupon_code_id,
        CouponCodeLifecycleAction::Restore,
        &claims.claims().sub,
        request_id.as_str(),
        &reason,
    )
    .await
    .map(Json)
}

pub(crate) async fn list_marketing_coupon_code_lifecycle_audits_handler(
    _claims: AuthenticatedAdminClaims,
    Path(coupon_code_id): Path<String>,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<CouponCodeLifecycleAuditRecord>>, (StatusCode, Json<ErrorResponse>)> {
    state
        .store
        .list_coupon_code_lifecycle_audit_records_for_code(&coupon_code_id)
        .await
        .map(Json)
        .map_err(|error| {
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!(
                    "failed to load coupon code lifecycle audits for {coupon_code_id}: {error}"
                ),
            )
        })
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

fn normalize_coupon_template_record_for_admin_upsert(
    mut record: CouponTemplateRecord,
) -> CouponTemplateRecord {
    if record.revision == 0 {
        record.revision = 1;
    }
    if record.root_coupon_template_id.is_none() {
        record.root_coupon_template_id = Some(record.coupon_template_id.clone());
    }
    if record.status != CouponTemplateStatus::Draft
        && record.approval_state == CouponTemplateApprovalState::Draft
    {
        record.approval_state = CouponTemplateApprovalState::Approved;
    }
    record
}

fn coupon_template_root_id(record: &CouponTemplateRecord) -> String {
    record
        .root_coupon_template_id
        .clone()
        .unwrap_or_else(|| record.coupon_template_id.clone())
}

fn coupon_template_revision(record: &CouponTemplateRecord) -> u32 {
    record.revision.max(1)
}

fn normalized_required_admin_identifier(
    value: &str,
    field_label: &str,
) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
    let normalized = value.trim();
    if normalized.is_empty() {
        return Err(error_response(
            StatusCode::BAD_REQUEST,
            format!("{field_label} is required"),
        ));
    }
    Ok(normalized.to_owned())
}

async fn load_coupon_template_record(
    store: &dyn AdminStore,
    coupon_template_id: &str,
) -> Result<CouponTemplateRecord, (StatusCode, Json<ErrorResponse>)> {
    store
        .find_coupon_template_record(coupon_template_id)
        .await
        .map_err(|_| {
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to load canonical coupon template",
            )
        })?
        .ok_or_else(|| {
            error_response(
                StatusCode::NOT_FOUND,
                format!("coupon template {coupon_template_id} not found"),
            )
        })
}

async fn next_coupon_template_revision(
    store: &dyn AdminStore,
    source_coupon_template: &CouponTemplateRecord,
) -> Result<u32, (StatusCode, Json<ErrorResponse>)> {
    let root_coupon_template_id = coupon_template_root_id(source_coupon_template);
    let next_revision = store
        .list_coupon_template_records()
        .await
        .map_err(|_| {
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to load canonical coupon template lineage",
            )
        })?
        .into_iter()
        .filter(|record| coupon_template_root_id(record) == root_coupon_template_id)
        .map(|record| coupon_template_revision(&record))
        .max()
        .unwrap_or_else(|| coupon_template_revision(source_coupon_template))
        .saturating_add(1);
    Ok(next_revision.max(coupon_template_revision(source_coupon_template) + 1))
}

fn normalize_marketing_campaign_record_for_admin_upsert(
    mut record: MarketingCampaignRecord,
) -> MarketingCampaignRecord {
    if record.revision == 0 {
        record.revision = 1;
    }
    if record.root_marketing_campaign_id.is_none() {
        record.root_marketing_campaign_id = Some(record.marketing_campaign_id.clone());
    }
    if record.status != MarketingCampaignStatus::Draft
        && record.approval_state == MarketingCampaignApprovalState::Draft
    {
        record.approval_state = MarketingCampaignApprovalState::Approved;
    }
    record
}

fn marketing_campaign_root_id(record: &MarketingCampaignRecord) -> String {
    record
        .root_marketing_campaign_id
        .clone()
        .unwrap_or_else(|| record.marketing_campaign_id.clone())
}

fn marketing_campaign_revision(record: &MarketingCampaignRecord) -> u32 {
    record.revision.max(1)
}

async fn next_marketing_campaign_revision(
    store: &dyn AdminStore,
    source_campaign: &MarketingCampaignRecord,
) -> Result<u32, (StatusCode, Json<ErrorResponse>)> {
    let root_marketing_campaign_id = marketing_campaign_root_id(source_campaign);
    let next_revision = store
        .list_marketing_campaign_records()
        .await
        .map_err(|_| {
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to load canonical marketing campaign lineage",
            )
        })?
        .into_iter()
        .filter(|record| marketing_campaign_root_id(record) == root_marketing_campaign_id)
        .map(|record| marketing_campaign_revision(&record))
        .max()
        .unwrap_or_else(|| marketing_campaign_revision(source_campaign))
        .saturating_add(1);
    Ok(next_revision.max(marketing_campaign_revision(source_campaign) + 1))
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

fn allowed_coupon_template_action() -> CouponTemplateActionDecision {
    CouponTemplateActionDecision {
        allowed: true,
        reasons: Vec::new(),
    }
}

fn blocked_coupon_template_action(reason: impl Into<String>) -> CouponTemplateActionDecision {
    CouponTemplateActionDecision {
        allowed: false,
        reasons: vec![reason.into()],
    }
}

fn build_coupon_template_actionability(
    coupon_template: &CouponTemplateRecord,
    now_ms: u64,
) -> CouponTemplateActionability {
    let template_archived = coupon_template.status == CouponTemplateStatus::Archived;
    let template_active = coupon_template.status == CouponTemplateStatus::Active;
    let template_scheduled = coupon_template.status == CouponTemplateStatus::Scheduled;
    let template_draft = coupon_template.status == CouponTemplateStatus::Draft;
    let approval_in_review =
        coupon_template.approval_state == CouponTemplateApprovalState::InReview;
    let approval_approved =
        coupon_template.approval_state == CouponTemplateApprovalState::Approved;
    let has_future_activation = coupon_template
        .activation_at_ms
        .is_some_and(|value| value > now_ms);

    let clone = allowed_coupon_template_action();

    let submit_for_approval = if !template_draft {
        blocked_coupon_template_action("coupon template must remain draft before approval submission")
    } else if approval_in_review {
        blocked_coupon_template_action("coupon template is already in approval review")
    } else if approval_approved {
        blocked_coupon_template_action("coupon template is already approved")
    } else {
        allowed_coupon_template_action()
    };

    let approve = if !template_draft {
        blocked_coupon_template_action("coupon template must remain draft before approval")
    } else if !approval_in_review {
        blocked_coupon_template_action("coupon template must be in_review before approve")
    } else {
        allowed_coupon_template_action()
    };

    let reject = if !template_draft {
        blocked_coupon_template_action("coupon template must remain draft before rejection")
    } else if !approval_in_review {
        blocked_coupon_template_action("coupon template must be in_review before reject")
    } else {
        allowed_coupon_template_action()
    };

    let publish = if template_archived {
        blocked_coupon_template_action("coupon template is already retired or archived")
    } else if template_active {
        blocked_coupon_template_action("coupon template is already published")
    } else if !approval_approved {
        blocked_coupon_template_action("coupon template must be approved before publish")
    } else if has_future_activation {
        blocked_coupon_template_action(
            "coupon template has future activation_at_ms and must be scheduled before publish",
        )
    } else {
        allowed_coupon_template_action()
    };

    let schedule = if template_archived {
        blocked_coupon_template_action("coupon template is already retired or archived")
    } else if template_active {
        blocked_coupon_template_action("coupon template is already published")
    } else if template_scheduled {
        blocked_coupon_template_action("coupon template is already scheduled")
    } else if !approval_approved {
        blocked_coupon_template_action("coupon template must be approved before schedule")
    } else if !has_future_activation {
        blocked_coupon_template_action(
            "coupon template must define a future activation_at_ms before schedule",
        )
    } else {
        allowed_coupon_template_action()
    };

    let retire = if template_archived {
        blocked_coupon_template_action("coupon template is already retired")
    } else if template_draft {
        blocked_coupon_template_action(
            "draft coupon template should be archived via status update before rollout",
        )
    } else {
        allowed_coupon_template_action()
    };

    CouponTemplateActionability {
        clone,
        submit_for_approval,
        approve,
        reject,
        publish,
        schedule,
        retire,
    }
}

fn build_coupon_template_detail(
    coupon_template: CouponTemplateRecord,
    now_ms: u64,
) -> CouponTemplateDetail {
    let actionability = build_coupon_template_actionability(&coupon_template, now_ms);
    CouponTemplateDetail {
        coupon_template,
        actionability,
    }
}

fn normalized_marketing_lifecycle_reason(
    reason: &str,
    aggregate_label: &str,
) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
    let normalized = reason.trim();
    if normalized.is_empty() {
        return Err(error_response(
            StatusCode::BAD_REQUEST,
            format!("{aggregate_label} lifecycle reason is required"),
        ));
    }
    Ok(normalized.to_owned())
}

fn build_coupon_template_lifecycle_audit_record(
    before: &CouponTemplateRecord,
    after: Option<&CouponTemplateRecord>,
    source_coupon_template_id: Option<String>,
    action: CouponTemplateLifecycleAction,
    outcome: CouponTemplateLifecycleAuditOutcome,
    operator_id: &str,
    request_id: &str,
    reason: &str,
    requested_at_ms: u64,
    decision_reasons: Vec<String>,
) -> CouponTemplateLifecycleAuditRecord {
    let after_template = after.unwrap_or(before);
    let audit_coupon_template_id = if action == CouponTemplateLifecycleAction::Clone {
        after_template.coupon_template_id.clone()
    } else {
        before.coupon_template_id.clone()
    };
    CouponTemplateLifecycleAuditRecord::new(
        format!(
            "coupon_template_audit:{request_id}:{}:{}",
            audit_coupon_template_id,
            action.as_str()
        ),
        audit_coupon_template_id,
        action,
        outcome,
        before.status,
        after_template.status,
        operator_id.to_owned(),
        request_id.to_owned(),
        reason.to_owned(),
        requested_at_ms,
    )
    .with_source_coupon_template_id(source_coupon_template_id)
    .with_approval_states(before.approval_state, after_template.approval_state)
    .with_revisions(coupon_template_revision(before), coupon_template_revision(after_template))
    .with_decision_reasons(decision_reasons)
}

async fn persist_coupon_template_lifecycle_audit_record(
    store: &dyn AdminStore,
    record: &CouponTemplateLifecycleAuditRecord,
) -> Result<CouponTemplateLifecycleAuditRecord, (StatusCode, Json<ErrorResponse>)> {
    store
        .insert_coupon_template_lifecycle_audit_record(record)
        .await
        .map_err(|error| {
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!(
                    "failed to persist coupon template lifecycle audit for {}: {error}",
                    record.coupon_template_id
                ),
            )
        })
}

async fn mutate_marketing_coupon_template_lifecycle(
    store: &dyn AdminStore,
    coupon_template_id: &str,
    action: CouponTemplateLifecycleAction,
    operator_id: &str,
    request_id: &str,
    reason: &str,
) -> Result<CouponTemplateMutationResult, (StatusCode, Json<ErrorResponse>)> {
    let now_ms = unix_timestamp_ms();
    let coupon_template = load_coupon_template_record(store, coupon_template_id).await?;

    let actionability = build_coupon_template_actionability(&coupon_template, now_ms);
    let decision = match action {
        CouponTemplateLifecycleAction::Clone => &actionability.clone,
        CouponTemplateLifecycleAction::SubmitForApproval => &actionability.submit_for_approval,
        CouponTemplateLifecycleAction::Approve => &actionability.approve,
        CouponTemplateLifecycleAction::Reject => &actionability.reject,
        CouponTemplateLifecycleAction::Publish => &actionability.publish,
        CouponTemplateLifecycleAction::Schedule => &actionability.schedule,
        CouponTemplateLifecycleAction::Retire => &actionability.retire,
    };
    if !decision.allowed {
        let audit = build_coupon_template_lifecycle_audit_record(
            &coupon_template,
            None,
            None,
            action,
            CouponTemplateLifecycleAuditOutcome::Rejected,
            operator_id,
            request_id,
            reason,
            now_ms,
            decision.reasons.clone(),
        );
        persist_coupon_template_lifecycle_audit_record(store, &audit).await?;
        let message = decision
            .reasons
            .first()
            .cloned()
            .unwrap_or_else(|| "coupon template lifecycle action is not allowed".to_owned());
        return Err(error_response(StatusCode::BAD_REQUEST, message));
    }

    let (next_status, next_approval_state) = match action {
        CouponTemplateLifecycleAction::Clone => unreachable!("clone uses dedicated helper"),
        CouponTemplateLifecycleAction::SubmitForApproval => {
            (coupon_template.status, CouponTemplateApprovalState::InReview)
        }
        CouponTemplateLifecycleAction::Approve => {
            (coupon_template.status, CouponTemplateApprovalState::Approved)
        }
        CouponTemplateLifecycleAction::Reject => {
            (coupon_template.status, CouponTemplateApprovalState::Rejected)
        }
        CouponTemplateLifecycleAction::Publish => {
            (CouponTemplateStatus::Active, coupon_template.approval_state)
        }
        CouponTemplateLifecycleAction::Schedule => {
            (CouponTemplateStatus::Scheduled, coupon_template.approval_state)
        }
        CouponTemplateLifecycleAction::Retire => {
            (CouponTemplateStatus::Archived, coupon_template.approval_state)
        }
    };

    let updated_coupon_template = coupon_template
        .clone()
        .with_status(next_status)
        .with_approval_state(next_approval_state)
        .with_updated_at_ms(now_ms);
    let updated_coupon_template = store
        .insert_coupon_template_record(&updated_coupon_template)
        .await
        .map_err(|_| {
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to persist canonical coupon template lifecycle mutation",
            )
        })?;

    let detail = build_coupon_template_detail(updated_coupon_template.clone(), now_ms);
    let audit = build_coupon_template_lifecycle_audit_record(
        &coupon_template,
        Some(&updated_coupon_template),
        None,
        action,
        CouponTemplateLifecycleAuditOutcome::Applied,
        operator_id,
        request_id,
        reason,
        now_ms,
        Vec::new(),
    );
    let audit = persist_coupon_template_lifecycle_audit_record(store, &audit).await?;
    Ok(CouponTemplateMutationResult { detail, audit })
}

async fn clone_marketing_coupon_template_revision(
    store: &dyn AdminStore,
    source_coupon_template_id: &str,
    request: CloneCouponTemplateRequest,
    operator_id: &str,
    request_id: &str,
    reason: &str,
) -> Result<CouponTemplateMutationResult, (StatusCode, Json<ErrorResponse>)> {
    let now_ms = unix_timestamp_ms();
    let source_coupon_template = load_coupon_template_record(store, source_coupon_template_id).await?;
    let actionability = build_coupon_template_actionability(&source_coupon_template, now_ms);
    if !actionability.clone.allowed {
        let audit = build_coupon_template_lifecycle_audit_record(
            &source_coupon_template,
            None,
            None,
            CouponTemplateLifecycleAction::Clone,
            CouponTemplateLifecycleAuditOutcome::Rejected,
            operator_id,
            request_id,
            reason,
            now_ms,
            actionability.clone.reasons.clone(),
        );
        persist_coupon_template_lifecycle_audit_record(store, &audit).await?;
        let message = actionability
            .clone
            .reasons
            .first()
            .cloned()
            .unwrap_or_else(|| "coupon template clone is not allowed".to_owned());
        return Err(error_response(StatusCode::BAD_REQUEST, message));
    }

    let cloned_coupon_template_id =
        normalized_required_admin_identifier(&request.coupon_template_id, "coupon_template_id")?;
    let cloned_template_key =
        normalized_required_admin_identifier(&request.template_key, "template_key")?;
    let cloned_display_name = request
        .display_name
        .and_then(|value| {
            let trimmed = value.trim().to_owned();
            (!trimmed.is_empty()).then_some(trimmed)
        })
        .unwrap_or_else(|| source_coupon_template.display_name.clone());

    if cloned_coupon_template_id == source_coupon_template.coupon_template_id {
        return Err(error_response(
            StatusCode::BAD_REQUEST,
            "cloned coupon template must use a new coupon_template_id",
        ));
    }
    if store
        .find_coupon_template_record(&cloned_coupon_template_id)
        .await
        .map_err(|_| {
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to validate cloned coupon template id uniqueness",
            )
        })?
        .is_some()
    {
        return Err(error_response(
            StatusCode::BAD_REQUEST,
            format!("coupon template {cloned_coupon_template_id} already exists"),
        ));
    }
    if store
        .find_coupon_template_record_by_template_key(&cloned_template_key)
        .await
        .map_err(|_| {
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to validate cloned coupon template key uniqueness",
            )
        })?
        .is_some()
    {
        return Err(error_response(
            StatusCode::BAD_REQUEST,
            format!("coupon template key {cloned_template_key} already exists"),
        ));
    }

    let root_coupon_template_id = coupon_template_root_id(&source_coupon_template);
    let cloned_coupon_template = source_coupon_template
        .clone()
        .with_status(CouponTemplateStatus::Draft)
        .with_approval_state(CouponTemplateApprovalState::Draft)
        .with_revision(next_coupon_template_revision(store, &source_coupon_template).await?)
        .with_root_coupon_template_id(Some(root_coupon_template_id))
        .with_parent_coupon_template_id(Some(source_coupon_template.coupon_template_id.clone()))
        .with_activation_at_ms(None)
        .with_created_at_ms(now_ms)
        .with_updated_at_ms(now_ms);
    let cloned_coupon_template = CouponTemplateRecord {
        coupon_template_id: cloned_coupon_template_id,
        template_key: cloned_template_key,
        display_name: cloned_display_name,
        ..cloned_coupon_template
    };
    let cloned_coupon_template = store
        .insert_coupon_template_record(&cloned_coupon_template)
        .await
        .map_err(|_| {
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to persist cloned coupon template revision",
            )
        })?;

    let detail = build_coupon_template_detail(cloned_coupon_template.clone(), now_ms);
    let audit = build_coupon_template_lifecycle_audit_record(
        &source_coupon_template,
        Some(&cloned_coupon_template),
        Some(source_coupon_template.coupon_template_id.clone()),
        CouponTemplateLifecycleAction::Clone,
        CouponTemplateLifecycleAuditOutcome::Applied,
        operator_id,
        request_id,
        reason,
        now_ms,
        Vec::new(),
    );
    let audit = persist_coupon_template_lifecycle_audit_record(store, &audit).await?;
    Ok(CouponTemplateMutationResult { detail, audit })
}

fn coupon_template_field_value(
    record: &CouponTemplateRecord,
    field: &str,
) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
    match field {
        "template_key" => Ok(record.template_key.clone()),
        "display_name" => Ok(record.display_name.clone()),
        "status" => Ok(serde_json::to_string(&record.status).unwrap_or_default()),
        "approval_state" => Ok(serde_json::to_string(&record.approval_state).unwrap_or_default()),
        "revision" => Ok(record.revision.to_string()),
        "distribution_kind" => {
            Ok(serde_json::to_string(&record.distribution_kind).unwrap_or_default())
        }
        "benefit" => serde_json::to_string(&record.benefit).map_err(|error| {
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("failed to serialize coupon template benefit for compare: {error}"),
            )
        }),
        "restriction" => serde_json::to_string(&record.restriction).map_err(|error| {
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("failed to serialize coupon template restriction for compare: {error}"),
            )
        }),
        "activation_at_ms" => Ok(record
            .activation_at_ms
            .map(|value| value.to_string())
            .unwrap_or_else(|| "null".to_owned())),
        _ => Ok(String::new()),
    }
}

async fn compare_marketing_coupon_template_revisions(
    store: &dyn AdminStore,
    source_coupon_template_id: &str,
    target_coupon_template_id: &str,
) -> Result<CouponTemplateComparisonResult, (StatusCode, Json<ErrorResponse>)> {
    let source_coupon_template = load_coupon_template_record(store, source_coupon_template_id).await?;
    let target_coupon_template = load_coupon_template_record(store, target_coupon_template_id).await?;
    let mut field_changes = Vec::new();
    for field in [
        "template_key",
        "display_name",
        "status",
        "approval_state",
        "revision",
        "distribution_kind",
        "benefit",
        "restriction",
        "activation_at_ms",
    ] {
        let source_value = coupon_template_field_value(&source_coupon_template, field)?;
        let target_value = coupon_template_field_value(&target_coupon_template, field)?;
        if source_value != target_value {
            field_changes.push(CouponTemplateComparisonFieldChange {
                field: field.to_owned(),
                source_value,
                target_value,
            });
        }
    }

    Ok(CouponTemplateComparisonResult {
        same_lineage: coupon_template_root_id(&source_coupon_template)
            == coupon_template_root_id(&target_coupon_template),
        source_coupon_template,
        target_coupon_template,
        field_changes,
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

fn allowed_marketing_campaign_action() -> MarketingCampaignActionDecision {
    MarketingCampaignActionDecision {
        allowed: true,
        reasons: Vec::new(),
    }
}

fn blocked_marketing_campaign_action(
    reason: impl Into<String>,
) -> MarketingCampaignActionDecision {
    MarketingCampaignActionDecision {
        allowed: false,
        reasons: vec![reason.into()],
    }
}

fn build_marketing_campaign_actionability(
    campaign: &MarketingCampaignRecord,
    coupon_template: &CouponTemplateRecord,
    now_ms: u64,
) -> MarketingCampaignActionability {
    let campaign_closed = matches!(
        campaign.status,
        MarketingCampaignStatus::Ended | MarketingCampaignStatus::Archived
    );
    let template_active = coupon_template.status == CouponTemplateStatus::Active;
    let campaign_active = campaign.status == MarketingCampaignStatus::Active;
    let campaign_scheduled = campaign.status == MarketingCampaignStatus::Scheduled;
    let campaign_draft = campaign.status == MarketingCampaignStatus::Draft;
    let approval_in_review =
        campaign.approval_state == MarketingCampaignApprovalState::InReview;
    let approval_approved =
        campaign.approval_state == MarketingCampaignApprovalState::Approved;
    let has_future_start = campaign.start_at_ms.is_some_and(|value| value > now_ms);
    let already_expired = campaign.end_at_ms.is_some_and(|value| value <= now_ms);

    let clone = allowed_marketing_campaign_action();

    let submit_for_approval = if !campaign_draft {
        blocked_marketing_campaign_action("campaign must remain draft before approval submission")
    } else if approval_in_review {
        blocked_marketing_campaign_action("campaign is already in approval review")
    } else if approval_approved {
        blocked_marketing_campaign_action("campaign is already approved")
    } else {
        allowed_marketing_campaign_action()
    };

    let approve = if !campaign_draft {
        blocked_marketing_campaign_action("campaign must remain draft before approval")
    } else if !approval_in_review {
        blocked_marketing_campaign_action("campaign must be in_review before approve")
    } else {
        allowed_marketing_campaign_action()
    };

    let reject = if !campaign_draft {
        blocked_marketing_campaign_action("campaign must remain draft before rejection")
    } else if !approval_in_review {
        blocked_marketing_campaign_action("campaign must be in_review before reject")
    } else {
        allowed_marketing_campaign_action()
    };

    let publish = if !template_active {
        blocked_marketing_campaign_action("coupon template must be active before campaign publish")
    } else if campaign_closed {
        blocked_marketing_campaign_action("campaign is already ended or archived")
    } else if campaign_active {
        blocked_marketing_campaign_action("campaign is already published")
    } else if !approval_approved {
        blocked_marketing_campaign_action("campaign must be approved before publish")
    } else if has_future_start {
        blocked_marketing_campaign_action(
            "campaign has future start_at_ms and must be scheduled before publish",
        )
    } else if already_expired {
        blocked_marketing_campaign_action("campaign end_at_ms is already in the past")
    } else {
        allowed_marketing_campaign_action()
    };

    let schedule = if !template_active {
        blocked_marketing_campaign_action("coupon template must be active before campaign schedule")
    } else if campaign_closed {
        blocked_marketing_campaign_action("campaign is already ended or archived")
    } else if campaign_active {
        blocked_marketing_campaign_action("campaign is already published")
    } else if campaign_scheduled {
        blocked_marketing_campaign_action("campaign is already scheduled")
    } else if !approval_approved {
        blocked_marketing_campaign_action("campaign must be approved before schedule")
    } else if !has_future_start {
        blocked_marketing_campaign_action(
            "campaign must define a future start_at_ms before schedule",
        )
    } else {
        allowed_marketing_campaign_action()
    };

    let retire = if campaign_closed {
        blocked_marketing_campaign_action("campaign is already retired")
    } else {
        allowed_marketing_campaign_action()
    };

    MarketingCampaignActionability {
        clone,
        submit_for_approval,
        approve,
        reject,
        publish,
        schedule,
        retire,
    }
}

async fn load_marketing_campaign_context(
    store: &dyn AdminStore,
    marketing_campaign_id: &str,
) -> Result<(MarketingCampaignRecord, CouponTemplateRecord), (StatusCode, Json<ErrorResponse>)> {
    let campaign = store
        .list_marketing_campaign_records()
        .await
        .map_err(|_| {
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to load canonical marketing campaigns",
            )
        })?
        .into_iter()
        .find(|record| record.marketing_campaign_id == marketing_campaign_id)
        .ok_or_else(|| {
            error_response(
                StatusCode::NOT_FOUND,
                format!("marketing campaign {marketing_campaign_id} not found"),
            )
        })?;

    let coupon_template = store
        .find_coupon_template_record(&campaign.coupon_template_id)
        .await
        .map_err(|_| {
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to load canonical coupon template for marketing campaign",
            )
        })?
        .ok_or_else(|| {
            error_response(
                StatusCode::NOT_FOUND,
                format!(
                    "coupon template {} for marketing campaign {} not found",
                    campaign.coupon_template_id, marketing_campaign_id
                ),
            )
        })?;

    Ok((campaign, coupon_template))
}

fn build_marketing_campaign_detail(
    campaign: MarketingCampaignRecord,
    coupon_template: CouponTemplateRecord,
    now_ms: u64,
) -> MarketingCampaignDetail {
    let actionability = build_marketing_campaign_actionability(&campaign, &coupon_template, now_ms);
    MarketingCampaignDetail {
        campaign,
        coupon_template,
        actionability,
    }
}

fn build_marketing_campaign_lifecycle_audit_record(
    before: &MarketingCampaignRecord,
    after: Option<&MarketingCampaignRecord>,
    source_marketing_campaign_id: Option<String>,
    action: MarketingCampaignLifecycleAction,
    outcome: MarketingCampaignLifecycleAuditOutcome,
    operator_id: &str,
    request_id: &str,
    reason: &str,
    requested_at_ms: u64,
    decision_reasons: Vec<String>,
) -> MarketingCampaignLifecycleAuditRecord {
    let after_campaign = after.unwrap_or(before);
    let audit_marketing_campaign_id = if action == MarketingCampaignLifecycleAction::Clone {
        after_campaign.marketing_campaign_id.clone()
    } else {
        before.marketing_campaign_id.clone()
    };
    MarketingCampaignLifecycleAuditRecord::new(
        format!(
            "marketing_campaign_audit:{request_id}:{}:{}",
            audit_marketing_campaign_id,
            action.as_str()
        ),
        audit_marketing_campaign_id,
        before.coupon_template_id.clone(),
        action,
        outcome,
        before.status,
        after_campaign.status,
        operator_id.to_owned(),
        request_id.to_owned(),
        reason.to_owned(),
        requested_at_ms,
    )
    .with_source_marketing_campaign_id(source_marketing_campaign_id)
    .with_approval_states(before.approval_state, after_campaign.approval_state)
    .with_revisions(
        marketing_campaign_revision(before),
        marketing_campaign_revision(after_campaign),
    )
    .with_decision_reasons(decision_reasons)
}

async fn persist_marketing_campaign_lifecycle_audit_record(
    store: &dyn AdminStore,
    record: &MarketingCampaignLifecycleAuditRecord,
) -> Result<MarketingCampaignLifecycleAuditRecord, (StatusCode, Json<ErrorResponse>)> {
    store
        .insert_marketing_campaign_lifecycle_audit_record(record)
        .await
        .map_err(|error| {
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!(
                    "failed to persist marketing campaign lifecycle audit for {}: {error}",
                    record.marketing_campaign_id
                ),
            )
        })
}

async fn mutate_marketing_campaign_lifecycle(
    store: &dyn AdminStore,
    marketing_campaign_id: &str,
    action: MarketingCampaignLifecycleAction,
    operator_id: &str,
    request_id: &str,
    reason: &str,
) -> Result<MarketingCampaignMutationResult, (StatusCode, Json<ErrorResponse>)> {
    let now_ms = unix_timestamp_ms();
    let (campaign, coupon_template) =
        load_marketing_campaign_context(store, marketing_campaign_id).await?;
    let actionability = build_marketing_campaign_actionability(&campaign, &coupon_template, now_ms);
    let decision = match action {
        MarketingCampaignLifecycleAction::Clone => unreachable!("clone uses dedicated helper"),
        MarketingCampaignLifecycleAction::SubmitForApproval => &actionability.submit_for_approval,
        MarketingCampaignLifecycleAction::Approve => &actionability.approve,
        MarketingCampaignLifecycleAction::Reject => &actionability.reject,
        MarketingCampaignLifecycleAction::Publish => &actionability.publish,
        MarketingCampaignLifecycleAction::Schedule => &actionability.schedule,
        MarketingCampaignLifecycleAction::Retire => &actionability.retire,
    };
    if !decision.allowed {
        let audit = build_marketing_campaign_lifecycle_audit_record(
            &campaign,
            None,
            None,
            action,
            MarketingCampaignLifecycleAuditOutcome::Rejected,
            operator_id,
            request_id,
            reason,
            now_ms,
            decision.reasons.clone(),
        );
        persist_marketing_campaign_lifecycle_audit_record(store, &audit).await?;
        let message = decision
            .reasons
            .first()
            .cloned()
            .unwrap_or_else(|| "campaign lifecycle action is not allowed".to_owned());
        return Err(error_response(StatusCode::BAD_REQUEST, message));
    }

    let (next_status, next_approval_state) = match action {
        MarketingCampaignLifecycleAction::Clone => unreachable!("clone uses dedicated helper"),
        MarketingCampaignLifecycleAction::SubmitForApproval => (
            MarketingCampaignStatus::Draft,
            MarketingCampaignApprovalState::InReview,
        ),
        MarketingCampaignLifecycleAction::Approve => (
            MarketingCampaignStatus::Draft,
            MarketingCampaignApprovalState::Approved,
        ),
        MarketingCampaignLifecycleAction::Reject => (
            MarketingCampaignStatus::Draft,
            MarketingCampaignApprovalState::Rejected,
        ),
        MarketingCampaignLifecycleAction::Publish => (
            MarketingCampaignStatus::Active,
            campaign.approval_state,
        ),
        MarketingCampaignLifecycleAction::Schedule => (
            MarketingCampaignStatus::Scheduled,
            campaign.approval_state,
        ),
        MarketingCampaignLifecycleAction::Retire => (
            MarketingCampaignStatus::Ended,
            campaign.approval_state,
        ),
    };

    let updated_campaign = campaign
        .clone()
        .with_status(next_status)
        .with_approval_state(next_approval_state)
        .with_updated_at_ms(now_ms);
    let updated_campaign = store
        .insert_marketing_campaign_record(&updated_campaign)
        .await
        .map_err(|_| {
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to persist canonical marketing campaign lifecycle mutation",
            )
        })?;

    let detail = build_marketing_campaign_detail(updated_campaign.clone(), coupon_template, now_ms);
    let audit = build_marketing_campaign_lifecycle_audit_record(
        &campaign,
        Some(&updated_campaign),
        None,
        action,
        MarketingCampaignLifecycleAuditOutcome::Applied,
        operator_id,
        request_id,
        reason,
        now_ms,
        Vec::new(),
    );
    let audit = persist_marketing_campaign_lifecycle_audit_record(store, &audit).await?;
    Ok(MarketingCampaignMutationResult { detail, audit })
}

fn marketing_campaign_field_value(
    record: &MarketingCampaignRecord,
    field: &str,
) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
    match field {
        "coupon_template_id" => Ok(record.coupon_template_id.clone()),
        "display_name" => Ok(record.display_name.clone()),
        "status" => Ok(serde_json::to_string(&record.status).unwrap_or_default()),
        "approval_state" => Ok(serde_json::to_string(&record.approval_state).unwrap_or_default()),
        "revision" => Ok(record.revision.to_string()),
        "start_at_ms" => Ok(record
            .start_at_ms
            .map(|value| value.to_string())
            .unwrap_or_else(|| "null".to_owned())),
        "end_at_ms" => Ok(record
            .end_at_ms
            .map(|value| value.to_string())
            .unwrap_or_else(|| "null".to_owned())),
        _ => Ok(String::new()),
    }
}

async fn compare_marketing_campaign_revisions(
    store: &dyn AdminStore,
    source_marketing_campaign_id: &str,
    target_marketing_campaign_id: &str,
) -> Result<MarketingCampaignComparisonResult, (StatusCode, Json<ErrorResponse>)> {
    let (source_marketing_campaign, _) =
        load_marketing_campaign_context(store, source_marketing_campaign_id).await?;
    let (target_marketing_campaign, _) =
        load_marketing_campaign_context(store, target_marketing_campaign_id).await?;
    let mut field_changes = Vec::new();
    for field in [
        "coupon_template_id",
        "display_name",
        "status",
        "approval_state",
        "revision",
        "start_at_ms",
        "end_at_ms",
    ] {
        let source_value = marketing_campaign_field_value(&source_marketing_campaign, field)?;
        let target_value = marketing_campaign_field_value(&target_marketing_campaign, field)?;
        if source_value != target_value {
            field_changes.push(MarketingCampaignComparisonFieldChange {
                field: field.to_owned(),
                source_value,
                target_value,
            });
        }
    }

    Ok(MarketingCampaignComparisonResult {
        same_lineage: marketing_campaign_root_id(&source_marketing_campaign)
            == marketing_campaign_root_id(&target_marketing_campaign),
        source_marketing_campaign,
        target_marketing_campaign,
        field_changes,
    })
}

async fn clone_marketing_campaign_revision(
    store: &dyn AdminStore,
    source_marketing_campaign_id: &str,
    request: CloneMarketingCampaignRequest,
    operator_id: &str,
    request_id: &str,
    reason: &str,
) -> Result<MarketingCampaignMutationResult, (StatusCode, Json<ErrorResponse>)> {
    let now_ms = unix_timestamp_ms();
    let (source_campaign, coupon_template) =
        load_marketing_campaign_context(store, source_marketing_campaign_id).await?;
    let actionability = build_marketing_campaign_actionability(&source_campaign, &coupon_template, now_ms);
    if !actionability.clone.allowed {
        let audit = build_marketing_campaign_lifecycle_audit_record(
            &source_campaign,
            None,
            None,
            MarketingCampaignLifecycleAction::Clone,
            MarketingCampaignLifecycleAuditOutcome::Rejected,
            operator_id,
            request_id,
            reason,
            now_ms,
            actionability.clone.reasons.clone(),
        );
        persist_marketing_campaign_lifecycle_audit_record(store, &audit).await?;
        let message = actionability
            .clone
            .reasons
            .first()
            .cloned()
            .unwrap_or_else(|| "campaign clone is not allowed".to_owned());
        return Err(error_response(StatusCode::BAD_REQUEST, message));
    }

    let cloned_marketing_campaign_id = normalized_required_admin_identifier(
        &request.marketing_campaign_id,
        "marketing_campaign_id",
    )?;
    let cloned_display_name = request
        .display_name
        .and_then(|value| {
            let trimmed = value.trim().to_owned();
            (!trimmed.is_empty()).then_some(trimmed)
        })
        .unwrap_or_else(|| source_campaign.display_name.clone());

    if cloned_marketing_campaign_id == source_campaign.marketing_campaign_id {
        return Err(error_response(
            StatusCode::BAD_REQUEST,
            "cloned marketing campaign must use a new marketing_campaign_id",
        ));
    }

    let exists = store
        .list_marketing_campaign_records()
        .await
        .map_err(|_| {
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to validate cloned marketing campaign id uniqueness",
            )
        })?
        .into_iter()
        .any(|record| record.marketing_campaign_id == cloned_marketing_campaign_id);
    if exists {
        return Err(error_response(
            StatusCode::BAD_REQUEST,
            format!("marketing campaign {cloned_marketing_campaign_id} already exists"),
        ));
    }

    let root_marketing_campaign_id = marketing_campaign_root_id(&source_campaign);
    let cloned_campaign = source_campaign
        .clone()
        .with_status(MarketingCampaignStatus::Draft)
        .with_approval_state(MarketingCampaignApprovalState::Draft)
        .with_revision(next_marketing_campaign_revision(store, &source_campaign).await?)
        .with_root_marketing_campaign_id(Some(root_marketing_campaign_id))
        .with_parent_marketing_campaign_id(Some(source_campaign.marketing_campaign_id.clone()))
        .with_created_at_ms(now_ms)
        .with_updated_at_ms(now_ms);
    let cloned_campaign = MarketingCampaignRecord {
        marketing_campaign_id: cloned_marketing_campaign_id,
        display_name: cloned_display_name,
        ..cloned_campaign
    };
    let cloned_campaign = store
        .insert_marketing_campaign_record(&cloned_campaign)
        .await
        .map_err(|_| {
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to persist cloned marketing campaign revision",
            )
        })?;

    let detail = build_marketing_campaign_detail(cloned_campaign.clone(), coupon_template, now_ms);
    let audit = build_marketing_campaign_lifecycle_audit_record(
        &source_campaign,
        Some(&cloned_campaign),
        Some(source_campaign.marketing_campaign_id.clone()),
        MarketingCampaignLifecycleAction::Clone,
        MarketingCampaignLifecycleAuditOutcome::Applied,
        operator_id,
        request_id,
        reason,
        now_ms,
        Vec::new(),
    );
    let audit = persist_marketing_campaign_lifecycle_audit_record(store, &audit).await?;
    Ok(MarketingCampaignMutationResult { detail, audit })
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

fn allowed_campaign_budget_action() -> CampaignBudgetActionDecision {
    CampaignBudgetActionDecision {
        allowed: true,
        reasons: Vec::new(),
    }
}

fn blocked_campaign_budget_action(reason: impl Into<String>) -> CampaignBudgetActionDecision {
    CampaignBudgetActionDecision {
        allowed: false,
        reasons: vec![reason.into()],
    }
}

fn build_campaign_budget_actionability(
    budget: &CampaignBudgetRecord,
    campaign: &MarketingCampaignRecord,
) -> CampaignBudgetActionability {
    let campaign_closed = matches!(
        campaign.status,
        MarketingCampaignStatus::Ended | MarketingCampaignStatus::Archived
    );
    let available_headroom = budget.available_budget_minor();

    let activate = if budget.status == CampaignBudgetStatus::Closed {
        blocked_campaign_budget_action("campaign budget is already closed")
    } else if budget.status == CampaignBudgetStatus::Active {
        blocked_campaign_budget_action("campaign budget is already active")
    } else if campaign_closed {
        blocked_campaign_budget_action("linked marketing campaign is ended or archived")
    } else if available_headroom == 0 {
        blocked_campaign_budget_action("campaign budget has no available headroom")
    } else {
        allowed_campaign_budget_action()
    };

    let close = if budget.status == CampaignBudgetStatus::Closed {
        blocked_campaign_budget_action("campaign budget is already closed")
    } else {
        allowed_campaign_budget_action()
    };

    CampaignBudgetActionability { activate, close }
}

async fn load_campaign_budget_context(
    store: &dyn AdminStore,
    campaign_budget_id: &str,
) -> Result<(CampaignBudgetRecord, MarketingCampaignRecord), (StatusCode, Json<ErrorResponse>)> {
    let budget = store
        .list_campaign_budget_records()
        .await
        .map_err(|_| {
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to load canonical campaign budgets",
            )
        })?
        .into_iter()
        .find(|record| record.campaign_budget_id == campaign_budget_id)
        .ok_or_else(|| {
            error_response(
                StatusCode::NOT_FOUND,
                format!("campaign budget {campaign_budget_id} not found"),
            )
        })?;

    let campaign = store
        .list_marketing_campaign_records()
        .await
        .map_err(|_| {
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to load canonical marketing campaigns for budget",
            )
        })?
        .into_iter()
        .find(|record| record.marketing_campaign_id == budget.marketing_campaign_id)
        .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            format!(
                "marketing campaign {} for campaign budget {} not found",
                budget.marketing_campaign_id, campaign_budget_id
            ),
        )
    })?;

    Ok((budget, campaign))
}

fn build_campaign_budget_detail(
    budget: CampaignBudgetRecord,
    campaign: MarketingCampaignRecord,
) -> CampaignBudgetDetail {
    let actionability = build_campaign_budget_actionability(&budget, &campaign);
    CampaignBudgetDetail {
        budget,
        campaign,
        actionability,
    }
}

fn build_campaign_budget_lifecycle_audit_record(
    before: &CampaignBudgetRecord,
    after: Option<&CampaignBudgetRecord>,
    action: CampaignBudgetLifecycleAction,
    outcome: CampaignBudgetLifecycleAuditOutcome,
    operator_id: &str,
    request_id: &str,
    reason: &str,
    requested_at_ms: u64,
    decision_reasons: Vec<String>,
) -> CampaignBudgetLifecycleAuditRecord {
    let after_budget = after.unwrap_or(before);
    CampaignBudgetLifecycleAuditRecord::new(
        format!(
            "campaign_budget_audit:{request_id}:{}:{}",
            before.campaign_budget_id,
            action.as_str()
        ),
        before.campaign_budget_id.clone(),
        before.marketing_campaign_id.clone(),
        action,
        outcome,
        before.status,
        after_budget.status,
        operator_id.to_owned(),
        request_id.to_owned(),
        reason.to_owned(),
        requested_at_ms,
    )
    .with_decision_reasons(decision_reasons)
}

async fn persist_campaign_budget_lifecycle_audit_record(
    store: &dyn AdminStore,
    record: &CampaignBudgetLifecycleAuditRecord,
) -> Result<CampaignBudgetLifecycleAuditRecord, (StatusCode, Json<ErrorResponse>)> {
    store
        .insert_campaign_budget_lifecycle_audit_record(record)
        .await
        .map_err(|error| {
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!(
                    "failed to persist campaign budget lifecycle audit for {}: {error}",
                    record.campaign_budget_id
                ),
            )
        })
}

async fn mutate_marketing_campaign_budget_lifecycle(
    store: &dyn AdminStore,
    campaign_budget_id: &str,
    action: CampaignBudgetLifecycleAction,
    operator_id: &str,
    request_id: &str,
    reason: &str,
) -> Result<CampaignBudgetMutationResult, (StatusCode, Json<ErrorResponse>)> {
    let now_ms = unix_timestamp_ms();
    let (budget, campaign) = load_campaign_budget_context(store, campaign_budget_id).await?;
    let actionability = build_campaign_budget_actionability(&budget, &campaign);
    let decision = match action {
        CampaignBudgetLifecycleAction::Activate => &actionability.activate,
        CampaignBudgetLifecycleAction::Close => &actionability.close,
    };
    if !decision.allowed {
        let audit = build_campaign_budget_lifecycle_audit_record(
            &budget,
            None,
            action,
            CampaignBudgetLifecycleAuditOutcome::Rejected,
            operator_id,
            request_id,
            reason,
            now_ms,
            decision.reasons.clone(),
        );
        persist_campaign_budget_lifecycle_audit_record(store, &audit).await?;
        let message = decision
            .reasons
            .first()
            .cloned()
            .unwrap_or_else(|| "campaign budget lifecycle action is not allowed".to_owned());
        return Err(error_response(StatusCode::BAD_REQUEST, message));
    }

    let next_status = match action {
        CampaignBudgetLifecycleAction::Activate => CampaignBudgetStatus::Active,
        CampaignBudgetLifecycleAction::Close => CampaignBudgetStatus::Closed,
    };

    let updated_budget = budget.clone().with_status(next_status).with_updated_at_ms(now_ms);
    let updated_budget = store
        .insert_campaign_budget_record(&updated_budget)
        .await
        .map_err(|_| {
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to persist canonical campaign budget lifecycle mutation",
            )
        })?;

    let detail = build_campaign_budget_detail(updated_budget.clone(), campaign);
    let audit = build_campaign_budget_lifecycle_audit_record(
        &budget,
        Some(&updated_budget),
        action,
        CampaignBudgetLifecycleAuditOutcome::Applied,
        operator_id,
        request_id,
        reason,
        now_ms,
        Vec::new(),
    );
    let audit = persist_campaign_budget_lifecycle_audit_record(store, &audit).await?;
    Ok(CampaignBudgetMutationResult { detail, audit })
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

fn allowed_coupon_code_action() -> CouponCodeActionDecision {
    CouponCodeActionDecision {
        allowed: true,
        reasons: Vec::new(),
    }
}

fn blocked_coupon_code_action(reason: impl Into<String>) -> CouponCodeActionDecision {
    CouponCodeActionDecision {
        allowed: false,
        reasons: vec![reason.into()],
    }
}

fn coupon_code_is_expired(coupon_code: &CouponCodeRecord, now_ms: u64) -> bool {
    coupon_code.status == CouponCodeStatus::Expired
        || coupon_code.expires_at_ms.is_some_and(|value| value <= now_ms)
}

fn build_coupon_code_actionability(
    coupon_code: &CouponCodeRecord,
    now_ms: u64,
) -> CouponCodeActionability {
    let expired = coupon_code_is_expired(coupon_code, now_ms);

    let disable = if expired {
        blocked_coupon_code_action("coupon code is expired and cannot be disabled")
    } else {
        match coupon_code.status {
            CouponCodeStatus::Available => allowed_coupon_code_action(),
            CouponCodeStatus::Disabled => {
                blocked_coupon_code_action("coupon code is already disabled")
            }
            CouponCodeStatus::Reserved => {
                blocked_coupon_code_action("reserved coupon code is governed by runtime and cannot be disabled")
            }
            CouponCodeStatus::Redeemed => {
                blocked_coupon_code_action("redeemed coupon code cannot be disabled")
            }
            CouponCodeStatus::Expired => {
                blocked_coupon_code_action("coupon code is expired and cannot be disabled")
            }
        }
    };

    let restore = if expired {
        blocked_coupon_code_action("coupon code is expired and cannot be restored")
    } else {
        match coupon_code.status {
            CouponCodeStatus::Disabled => allowed_coupon_code_action(),
            CouponCodeStatus::Available => {
                blocked_coupon_code_action("coupon code is already available")
            }
            CouponCodeStatus::Reserved => {
                blocked_coupon_code_action("reserved coupon code is governed by runtime and cannot be restored")
            }
            CouponCodeStatus::Redeemed => {
                blocked_coupon_code_action("redeemed coupon code cannot be restored")
            }
            CouponCodeStatus::Expired => {
                blocked_coupon_code_action("coupon code is expired and cannot be restored")
            }
        }
    };

    CouponCodeActionability { disable, restore }
}

async fn load_coupon_code_context(
    store: &dyn AdminStore,
    coupon_code_id: &str,
) -> Result<(CouponCodeRecord, CouponTemplateRecord), (StatusCode, Json<ErrorResponse>)> {
    let coupon_code = store
        .find_coupon_code_record(coupon_code_id)
        .await
        .map_err(|_| {
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to load canonical coupon code",
            )
        })?
        .ok_or_else(|| {
            error_response(
                StatusCode::NOT_FOUND,
                format!("coupon code {coupon_code_id} not found"),
            )
        })?;

    let coupon_template = store
        .find_coupon_template_record(&coupon_code.coupon_template_id)
        .await
        .map_err(|_| {
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to load canonical coupon template for coupon code",
            )
        })?
        .ok_or_else(|| {
            error_response(
                StatusCode::NOT_FOUND,
                format!(
                    "coupon template {} for coupon code {} not found",
                    coupon_code.coupon_template_id, coupon_code_id
                ),
            )
        })?;

    Ok((coupon_code, coupon_template))
}

fn build_coupon_code_detail(
    coupon_code: CouponCodeRecord,
    coupon_template: CouponTemplateRecord,
    now_ms: u64,
) -> CouponCodeDetail {
    let actionability = build_coupon_code_actionability(&coupon_code, now_ms);
    CouponCodeDetail {
        coupon_code,
        coupon_template,
        actionability,
    }
}

fn build_coupon_code_lifecycle_audit_record(
    before: &CouponCodeRecord,
    after: Option<&CouponCodeRecord>,
    action: CouponCodeLifecycleAction,
    outcome: CouponCodeLifecycleAuditOutcome,
    operator_id: &str,
    request_id: &str,
    reason: &str,
    requested_at_ms: u64,
    decision_reasons: Vec<String>,
) -> CouponCodeLifecycleAuditRecord {
    let after_code = after.unwrap_or(before);
    CouponCodeLifecycleAuditRecord::new(
        format!(
            "coupon_code_audit:{request_id}:{}:{}",
            before.coupon_code_id,
            action.as_str()
        ),
        before.coupon_code_id.clone(),
        before.coupon_template_id.clone(),
        action,
        outcome,
        before.status,
        after_code.status,
        operator_id.to_owned(),
        request_id.to_owned(),
        reason.to_owned(),
        requested_at_ms,
    )
    .with_decision_reasons(decision_reasons)
}

async fn persist_coupon_code_lifecycle_audit_record(
    store: &dyn AdminStore,
    record: &CouponCodeLifecycleAuditRecord,
) -> Result<CouponCodeLifecycleAuditRecord, (StatusCode, Json<ErrorResponse>)> {
    store
        .insert_coupon_code_lifecycle_audit_record(record)
        .await
        .map_err(|error| {
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!(
                    "failed to persist coupon code lifecycle audit for {}: {error}",
                    record.coupon_code_id
                ),
            )
        })
}

async fn mutate_marketing_coupon_code_lifecycle(
    store: &dyn AdminStore,
    coupon_code_id: &str,
    action: CouponCodeLifecycleAction,
    operator_id: &str,
    request_id: &str,
    reason: &str,
) -> Result<CouponCodeMutationResult, (StatusCode, Json<ErrorResponse>)> {
    let now_ms = unix_timestamp_ms();
    let (coupon_code, coupon_template) = load_coupon_code_context(store, coupon_code_id).await?;
    let actionability = build_coupon_code_actionability(&coupon_code, now_ms);
    let decision = match action {
        CouponCodeLifecycleAction::Disable => &actionability.disable,
        CouponCodeLifecycleAction::Restore => &actionability.restore,
    };
    if !decision.allowed {
        let audit = build_coupon_code_lifecycle_audit_record(
            &coupon_code,
            None,
            action,
            CouponCodeLifecycleAuditOutcome::Rejected,
            operator_id,
            request_id,
            reason,
            now_ms,
            decision.reasons.clone(),
        );
        persist_coupon_code_lifecycle_audit_record(store, &audit).await?;
        let message = decision
            .reasons
            .first()
            .cloned()
            .unwrap_or_else(|| "coupon code lifecycle action is not allowed".to_owned());
        return Err(error_response(StatusCode::BAD_REQUEST, message));
    }

    let next_status = match action {
        CouponCodeLifecycleAction::Disable => CouponCodeStatus::Disabled,
        CouponCodeLifecycleAction::Restore => CouponCodeStatus::Available,
    };

    let updated_coupon_code = coupon_code
        .clone()
        .with_status(next_status)
        .with_updated_at_ms(now_ms);
    let updated_coupon_code = store
        .insert_coupon_code_record(&updated_coupon_code)
        .await
        .map_err(|_| {
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to persist canonical coupon code lifecycle mutation",
            )
        })?;

    let detail = build_coupon_code_detail(updated_coupon_code.clone(), coupon_template, now_ms);
    let audit = build_coupon_code_lifecycle_audit_record(
        &coupon_code,
        Some(&updated_coupon_code),
        action,
        CouponCodeLifecycleAuditOutcome::Applied,
        operator_id,
        request_id,
        reason,
        now_ms,
        Vec::new(),
    );
    let audit = persist_coupon_code_lifecycle_audit_record(store, &audit).await?;
    Ok(CouponCodeMutationResult { detail, audit })
}

