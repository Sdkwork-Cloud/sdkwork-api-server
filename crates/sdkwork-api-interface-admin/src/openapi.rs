use super::*;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "SDKWORK Admin API",
        version = env!("CARGO_PKG_VERSION"),
        description = "OpenAPI 3.1 schema generated directly from the current admin router implementation."
    ),
    modifiers(&AdminApiDocModifier),
    tags(
        (name = "system", description = "Admin health and system-facing routes."),
        (name = "auth", description = "Admin authentication and session management routes."),
        (name = "catalog", description = "Provider and model catalog administration routes."),
        (name = "marketing", description = "Coupon template, campaign, budget, and redemption administration routes."),
        (name = "tenants", description = "Tenant and project administration routes."),
        (name = "users", description = "Operator and portal user administration routes."),
        (name = "gateway", description = "Gateway API key and API key group administration routes."),
        (name = "billing", description = "Billing summary, event, and ledger administration routes."),
        (name = "commerce", description = "Recent order and payment callback audit routes.")
    )
)]
struct AdminApiDoc;

struct AdminApiDocModifier;

impl Modify for AdminApiDocModifier {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        openapi.servers = Some(vec![Server::new("/")]);
        openapi
            .components
            .get_or_insert_with(utoipa::openapi::Components::new)
            .add_security_scheme(
                "bearerAuth",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            );
    }
}

mod openapi_paths {
    use super::*;
    use crate::marketing::{
        CampaignBudgetMutationResult, CouponCodeMutationResult, CouponTemplateComparisonResult,
        CouponTemplateMutationResult, MarketingCampaignComparisonResult,
        MarketingCampaignMutationResult,
    };

    #[utoipa::path(
        get,
        path = "/admin/health",
        tag = "system",
        responses((status = 200, description = "Admin health check response.", body = String))
    )]
    pub(super) async fn health() {}

    #[utoipa::path(
        post,
        path = "/admin/auth/login",
        tag = "auth",
        request_body = LoginRequest,
        responses(
            (status = 200, description = "Admin login session.", body = LoginResponse),
            (status = 401, description = "Invalid admin credentials.", body = ErrorResponse),
            (status = 500, description = "Admin authentication failed.", body = ErrorResponse)
        )
    )]
    pub(super) async fn auth_login() {}

    #[utoipa::path(
        post,
        path = "/admin/auth/change-password",
        tag = "auth",
        request_body = ChangePasswordRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Updated admin profile after password change.", body = AdminUserProfile),
            (status = 400, description = "Invalid password change request.", body = ErrorResponse),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 500, description = "Password change failed.", body = ErrorResponse)
        )
    )]
    pub(super) async fn auth_change_password() {}

    #[utoipa::path(
        get,
        path = "/admin/tenants",
        tag = "tenants",
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible tenant catalog.", body = [Tenant]),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 500, description = "Failed to load tenants.")
        )
    )]
    pub(super) async fn tenants_list() {}

    #[utoipa::path(
        post,
        path = "/admin/tenants",
        tag = "tenants",
        request_body = CreateTenantRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 201, description = "Created tenant.", body = Tenant),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 500, description = "Failed to create tenant.")
        )
    )]
    pub(super) async fn tenants_create() {}

    #[utoipa::path(
        get,
        path = "/admin/tenants/{tenant_id}/providers/readiness",
        tag = "catalog",
        security(("bearerAuth" = [])),
        params(
            ("tenant_id" = String, Path, description = "Tenant identifier whose provider credential-readiness overlay should be listed.")
        ),
        responses(
            (status = 200, description = "Tenant-scoped provider readiness inventory. This endpoint focuses on tenant overlay state and keeps global execution truth on `/admin/providers`.", body = [TenantProviderReadinessResponse]),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 500, description = "Failed to load tenant-scoped provider readiness.", body = ErrorResponse)
        )
    )]
    pub(super) async fn tenant_provider_readiness_list() {}

    #[utoipa::path(
        get,
        path = "/admin/projects",
        tag = "tenants",
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible project catalog.", body = [Project]),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 500, description = "Failed to load projects.")
        )
    )]
    pub(super) async fn projects_list() {}

    #[utoipa::path(
        post,
        path = "/admin/projects",
        tag = "tenants",
        request_body = CreateProjectRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 201, description = "Created project.", body = Project),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 500, description = "Failed to create project.")
        )
    )]
    pub(super) async fn projects_create() {}

    #[utoipa::path(
        get,
        path = "/admin/providers",
        tag = "catalog",
        security(("bearerAuth" = [])),
        params(
            ("tenant_id" = Option<String>, Query, description = "Optional tenant scope. When present, each provider entry adds tenant-specific `credential_readiness` without changing the global catalog semantics of `integration` and `execution`.")
        ),
        responses(
            (status = 200, description = "Visible provider catalog. `credential_readiness` is returned only when `tenant_id` is requested.", body = [ProviderCatalogResponse]),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 500, description = "Failed to load providers.", body = ErrorResponse)
        )
    )]
    pub(super) async fn providers_list() {}

    #[utoipa::path(
        post,
        path = "/admin/providers",
        tag = "catalog",
        request_body = CreateProviderRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 201, description = "Created provider with normalized integration metadata.", body = ProviderCreateResponse),
            (status = 400, description = "Invalid provider payload.", body = ErrorResponse),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 500, description = "Failed to create provider.", body = ErrorResponse)
        )
    )]
    pub(super) async fn providers_create() {}

    #[utoipa::path(
        get,
        path = "/admin/users/operators",
        tag = "users",
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible operator users.", body = [AdminUserProfile]),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 500, description = "Failed to load operator users.", body = ErrorResponse)
        )
    )]
    pub(super) async fn operator_users_list() {}

    #[utoipa::path(
        post,
        path = "/admin/users/operators",
        tag = "users",
        request_body = UpsertOperatorUserRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 201, description = "Created or updated operator user.", body = AdminUserProfile),
            (status = 400, description = "Invalid operator user payload.", body = ErrorResponse),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 500, description = "Failed to persist operator user.", body = ErrorResponse)
        )
    )]
    pub(super) async fn operator_users_upsert() {}

    #[utoipa::path(
        post,
        path = "/admin/users/operators/{user_id}/status",
        tag = "users",
        params(("user_id" = String, Path, description = "Operator user id.")),
        request_body = UpdateUserStatusRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Updated operator user status.", body = AdminUserProfile),
            (status = 400, description = "Invalid operator user status payload.", body = ErrorResponse),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 500, description = "Failed to update operator user status.", body = ErrorResponse)
        )
    )]
    pub(super) async fn operator_user_status_update() {}

    #[utoipa::path(
        get,
        path = "/admin/users/portal",
        tag = "users",
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible portal users.", body = [PortalUserProfile]),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 500, description = "Failed to load portal users.", body = ErrorResponse)
        )
    )]
    pub(super) async fn portal_users_list() {}

    #[utoipa::path(
        post,
        path = "/admin/users/portal",
        tag = "users",
        request_body = UpsertPortalUserRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 201, description = "Created or updated portal user.", body = PortalUserProfile),
            (status = 400, description = "Invalid portal user payload.", body = ErrorResponse),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 500, description = "Failed to persist portal user.", body = ErrorResponse)
        )
    )]
    pub(super) async fn portal_users_upsert() {}

    #[utoipa::path(
        post,
        path = "/admin/users/portal/{user_id}/status",
        tag = "users",
        params(("user_id" = String, Path, description = "Portal user id.")),
        request_body = UpdateUserStatusRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Updated portal user status.", body = PortalUserProfile),
            (status = 400, description = "Invalid portal user status payload.", body = ErrorResponse),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 500, description = "Failed to update portal user status.", body = ErrorResponse)
        )
    )]
    pub(super) async fn portal_user_status_update() {}

    #[utoipa::path(
        get,
        path = "/admin/marketing/coupon-templates",
        tag = "marketing",
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible canonical coupon templates.", body = [CouponTemplateRecord]),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 500, description = "Failed to load canonical coupon templates.", body = ErrorResponse)
        )
    )]
    pub(super) async fn marketing_coupon_templates_list() {}

    #[utoipa::path(
        post,
        path = "/admin/marketing/coupon-templates",
        tag = "marketing",
        request_body = CouponTemplateRecord,
        security(("bearerAuth" = [])),
        responses(
            (status = 201, description = "Created or updated canonical coupon template.", body = CouponTemplateRecord),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 500, description = "Failed to persist canonical coupon template.", body = ErrorResponse)
        )
    )]
    pub(super) async fn marketing_coupon_templates_create() {}

    #[utoipa::path(
        post,
        path = "/admin/marketing/coupon-templates/{coupon_template_id}/status",
        tag = "marketing",
        params(("coupon_template_id" = String, Path, description = "Coupon template id")),
        request_body = UpdateCouponTemplateStatusRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Updated canonical coupon template status.", body = CouponTemplateRecord),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 404, description = "Canonical coupon template not found.", body = ErrorResponse),
            (status = 500, description = "Failed to update canonical coupon template status.", body = ErrorResponse)
        )
    )]
    pub(super) async fn marketing_coupon_templates_status_update() {}

    #[utoipa::path(
        post,
        path = "/admin/marketing/coupon-templates/{coupon_template_id}/clone",
        tag = "marketing",
        params(("coupon_template_id" = String, Path, description = "Source coupon template id")),
        request_body = CloneCouponTemplateRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 201, description = "Cloned the selected canonical coupon template into a governed draft revision.", body = CouponTemplateMutationResult),
            (status = 400, description = "Coupon template clone request is invalid.", body = ErrorResponse),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 404, description = "Canonical coupon template not found.", body = ErrorResponse),
            (status = 500, description = "Failed to clone canonical coupon template.", body = ErrorResponse)
        )
    )]
    pub(super) async fn marketing_coupon_templates_clone() {}

    #[utoipa::path(
        post,
        path = "/admin/marketing/coupon-templates/{coupon_template_id}/compare",
        tag = "marketing",
        params(("coupon_template_id" = String, Path, description = "Source coupon template id")),
        request_body = CompareCouponTemplateRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Compared two coupon template revisions.", body = CouponTemplateComparisonResult),
            (status = 400, description = "Coupon template compare request is invalid.", body = ErrorResponse),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 404, description = "Canonical coupon template not found.", body = ErrorResponse),
            (status = 500, description = "Failed to compare canonical coupon templates.", body = ErrorResponse)
        )
    )]
    pub(super) async fn marketing_coupon_templates_compare() {}

    #[utoipa::path(
        post,
        path = "/admin/marketing/coupon-templates/{coupon_template_id}/submit-for-approval",
        tag = "marketing",
        params(("coupon_template_id" = String, Path, description = "Coupon template id")),
        request_body = SubmitCouponTemplateForApprovalRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Submitted the selected coupon template revision for approval.", body = CouponTemplateMutationResult),
            (status = 400, description = "Coupon template cannot enter approval from the current governance state.", body = ErrorResponse),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 404, description = "Canonical coupon template not found.", body = ErrorResponse),
            (status = 500, description = "Failed to submit coupon template for approval.", body = ErrorResponse)
        )
    )]
    pub(super) async fn marketing_coupon_templates_submit_for_approval() {}

    #[utoipa::path(
        post,
        path = "/admin/marketing/coupon-templates/{coupon_template_id}/approve",
        tag = "marketing",
        params(("coupon_template_id" = String, Path, description = "Coupon template id")),
        request_body = ApproveCouponTemplateRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Approved the selected coupon template revision.", body = CouponTemplateMutationResult),
            (status = 400, description = "Coupon template cannot be approved from the current governance state.", body = ErrorResponse),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 404, description = "Canonical coupon template not found.", body = ErrorResponse),
            (status = 500, description = "Failed to approve coupon template.", body = ErrorResponse)
        )
    )]
    pub(super) async fn marketing_coupon_templates_approve() {}

    #[utoipa::path(
        post,
        path = "/admin/marketing/coupon-templates/{coupon_template_id}/reject",
        tag = "marketing",
        params(("coupon_template_id" = String, Path, description = "Coupon template id")),
        request_body = RejectCouponTemplateRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Rejected the selected coupon template revision.", body = CouponTemplateMutationResult),
            (status = 400, description = "Coupon template cannot be rejected from the current governance state.", body = ErrorResponse),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 404, description = "Canonical coupon template not found.", body = ErrorResponse),
            (status = 500, description = "Failed to reject coupon template.", body = ErrorResponse)
        )
    )]
    pub(super) async fn marketing_coupon_templates_reject() {}

    #[utoipa::path(
        post,
        path = "/admin/marketing/coupon-templates/{coupon_template_id}/publish",
        tag = "marketing",
        params(("coupon_template_id" = String, Path, description = "Coupon template id")),
        request_body = PublishCouponTemplateRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Published the selected canonical coupon template with semantic lifecycle evidence.", body = CouponTemplateMutationResult),
            (status = 400, description = "Coupon template cannot be published from the current coupon lifecycle state.", body = ErrorResponse),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 404, description = "Canonical coupon template not found.", body = ErrorResponse),
            (status = 500, description = "Failed to publish canonical coupon template.", body = ErrorResponse)
        )
    )]
    pub(super) async fn marketing_coupon_templates_publish() {}

    #[utoipa::path(
        post,
        path = "/admin/marketing/coupon-templates/{coupon_template_id}/schedule",
        tag = "marketing",
        params(("coupon_template_id" = String, Path, description = "Coupon template id")),
        request_body = ScheduleCouponTemplateRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Scheduled the selected canonical coupon template with semantic lifecycle evidence.", body = CouponTemplateMutationResult),
            (status = 400, description = "Coupon template cannot be scheduled from the current coupon lifecycle state.", body = ErrorResponse),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 404, description = "Canonical coupon template not found.", body = ErrorResponse),
            (status = 500, description = "Failed to schedule canonical coupon template.", body = ErrorResponse)
        )
    )]
    pub(super) async fn marketing_coupon_templates_schedule() {}

    #[utoipa::path(
        post,
        path = "/admin/marketing/coupon-templates/{coupon_template_id}/retire",
        tag = "marketing",
        params(("coupon_template_id" = String, Path, description = "Coupon template id")),
        request_body = RetireCouponTemplateRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Retired the selected canonical coupon template with semantic lifecycle evidence.", body = CouponTemplateMutationResult),
            (status = 400, description = "Coupon template cannot be retired from the current coupon lifecycle state.", body = ErrorResponse),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 404, description = "Canonical coupon template not found.", body = ErrorResponse),
            (status = 500, description = "Failed to retire canonical coupon template.", body = ErrorResponse)
        )
    )]
    pub(super) async fn marketing_coupon_templates_retire() {}

    #[utoipa::path(
        get,
        path = "/admin/marketing/coupon-templates/{coupon_template_id}/lifecycle-audits",
        tag = "marketing",
        params(("coupon_template_id" = String, Path, description = "Coupon template id")),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Lifecycle audit trail for the selected canonical coupon template.", body = [CouponTemplateLifecycleAuditRecord]),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 500, description = "Failed to load coupon template lifecycle audit trail.", body = ErrorResponse)
        )
    )]
    pub(super) async fn marketing_coupon_template_lifecycle_audits_list() {}

    #[utoipa::path(
        get,
        path = "/admin/marketing/campaigns",
        tag = "marketing",
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible canonical marketing campaigns.", body = [MarketingCampaignRecord]),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 500, description = "Failed to load canonical marketing campaigns.", body = ErrorResponse)
        )
    )]
    pub(super) async fn marketing_campaigns_list() {}

    #[utoipa::path(
        post,
        path = "/admin/marketing/campaigns",
        tag = "marketing",
        request_body = MarketingCampaignRecord,
        security(("bearerAuth" = [])),
        responses(
            (status = 201, description = "Created or updated canonical marketing campaign.", body = MarketingCampaignRecord),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 500, description = "Failed to persist canonical marketing campaign.", body = ErrorResponse)
        )
    )]
    pub(super) async fn marketing_campaigns_create() {}

    #[utoipa::path(
        post,
        path = "/admin/marketing/campaigns/{marketing_campaign_id}/status",
        tag = "marketing",
        params(("marketing_campaign_id" = String, Path, description = "Marketing campaign id")),
        request_body = UpdateMarketingCampaignStatusRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Updated canonical marketing campaign status.", body = MarketingCampaignRecord),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 404, description = "Canonical marketing campaign not found.", body = ErrorResponse),
            (status = 500, description = "Failed to update canonical marketing campaign status.", body = ErrorResponse)
        )
    )]
    pub(super) async fn marketing_campaigns_status_update() {}

    #[utoipa::path(
        post,
        path = "/admin/marketing/campaigns/{marketing_campaign_id}/clone",
        tag = "marketing",
        params(("marketing_campaign_id" = String, Path, description = "Source marketing campaign id")),
        request_body = CloneMarketingCampaignRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 201, description = "Cloned the selected canonical coupon campaign into a governed draft revision.", body = MarketingCampaignMutationResult),
            (status = 400, description = "Campaign clone request is invalid.", body = ErrorResponse),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 404, description = "Canonical marketing campaign not found.", body = ErrorResponse),
            (status = 500, description = "Failed to clone canonical marketing campaign.", body = ErrorResponse)
        )
    )]
    pub(super) async fn marketing_campaigns_clone() {}

    #[utoipa::path(
        post,
        path = "/admin/marketing/campaigns/{marketing_campaign_id}/compare",
        tag = "marketing",
        params(("marketing_campaign_id" = String, Path, description = "Source marketing campaign id")),
        request_body = CompareMarketingCampaignRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Compared two coupon campaign revisions.", body = MarketingCampaignComparisonResult),
            (status = 400, description = "Campaign compare request is invalid.", body = ErrorResponse),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 404, description = "Canonical marketing campaign not found.", body = ErrorResponse),
            (status = 500, description = "Failed to compare canonical marketing campaigns.", body = ErrorResponse)
        )
    )]
    pub(super) async fn marketing_campaigns_compare() {}

    #[utoipa::path(
        post,
        path = "/admin/marketing/campaigns/{marketing_campaign_id}/submit-for-approval",
        tag = "marketing",
        params(("marketing_campaign_id" = String, Path, description = "Marketing campaign id")),
        request_body = SubmitMarketingCampaignForApprovalRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Submitted the selected coupon campaign revision for approval.", body = MarketingCampaignMutationResult),
            (status = 400, description = "Campaign cannot enter approval from the current governance state.", body = ErrorResponse),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 404, description = "Canonical marketing campaign not found.", body = ErrorResponse),
            (status = 500, description = "Failed to submit marketing campaign for approval.", body = ErrorResponse)
        )
    )]
    pub(super) async fn marketing_campaigns_submit_for_approval() {}

    #[utoipa::path(
        post,
        path = "/admin/marketing/campaigns/{marketing_campaign_id}/approve",
        tag = "marketing",
        params(("marketing_campaign_id" = String, Path, description = "Marketing campaign id")),
        request_body = ApproveMarketingCampaignRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Approved the selected coupon campaign revision.", body = MarketingCampaignMutationResult),
            (status = 400, description = "Campaign cannot be approved from the current governance state.", body = ErrorResponse),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 404, description = "Canonical marketing campaign not found.", body = ErrorResponse),
            (status = 500, description = "Failed to approve marketing campaign.", body = ErrorResponse)
        )
    )]
    pub(super) async fn marketing_campaigns_approve() {}

    #[utoipa::path(
        post,
        path = "/admin/marketing/campaigns/{marketing_campaign_id}/reject",
        tag = "marketing",
        params(("marketing_campaign_id" = String, Path, description = "Marketing campaign id")),
        request_body = RejectMarketingCampaignRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Rejected the selected coupon campaign revision.", body = MarketingCampaignMutationResult),
            (status = 400, description = "Campaign cannot be rejected from the current governance state.", body = ErrorResponse),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 404, description = "Canonical marketing campaign not found.", body = ErrorResponse),
            (status = 500, description = "Failed to reject marketing campaign.", body = ErrorResponse)
        )
    )]
    pub(super) async fn marketing_campaigns_reject() {}

    #[utoipa::path(
        post,
        path = "/admin/marketing/campaigns/{marketing_campaign_id}/publish",
        tag = "marketing",
        params(("marketing_campaign_id" = String, Path, description = "Marketing campaign id")),
        request_body = PublishMarketingCampaignRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Published the selected canonical coupon campaign with semantic lifecycle evidence.", body = MarketingCampaignMutationResult),
            (status = 400, description = "Campaign cannot be published from the current coupon lifecycle state.", body = ErrorResponse),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 404, description = "Canonical marketing campaign not found.", body = ErrorResponse),
            (status = 500, description = "Failed to publish canonical marketing campaign.", body = ErrorResponse)
        )
    )]
    pub(super) async fn marketing_campaigns_publish() {}

    #[utoipa::path(
        post,
        path = "/admin/marketing/campaigns/{marketing_campaign_id}/schedule",
        tag = "marketing",
        params(("marketing_campaign_id" = String, Path, description = "Marketing campaign id")),
        request_body = ScheduleMarketingCampaignRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Scheduled the selected canonical coupon campaign with semantic lifecycle evidence.", body = MarketingCampaignMutationResult),
            (status = 400, description = "Campaign cannot be scheduled from the current coupon lifecycle state.", body = ErrorResponse),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 404, description = "Canonical marketing campaign not found.", body = ErrorResponse),
            (status = 500, description = "Failed to schedule canonical marketing campaign.", body = ErrorResponse)
        )
    )]
    pub(super) async fn marketing_campaigns_schedule() {}

    #[utoipa::path(
        post,
        path = "/admin/marketing/campaigns/{marketing_campaign_id}/retire",
        tag = "marketing",
        params(("marketing_campaign_id" = String, Path, description = "Marketing campaign id")),
        request_body = RetireMarketingCampaignRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Retired the selected canonical coupon campaign with semantic lifecycle evidence.", body = MarketingCampaignMutationResult),
            (status = 400, description = "Campaign cannot be retired from the current coupon lifecycle state.", body = ErrorResponse),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 404, description = "Canonical marketing campaign not found.", body = ErrorResponse),
            (status = 500, description = "Failed to retire canonical marketing campaign.", body = ErrorResponse)
        )
    )]
    pub(super) async fn marketing_campaigns_retire() {}

    #[utoipa::path(
        get,
        path = "/admin/marketing/campaigns/{marketing_campaign_id}/lifecycle-audits",
        tag = "marketing",
        params(("marketing_campaign_id" = String, Path, description = "Marketing campaign id")),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Lifecycle audit trail for the selected canonical coupon campaign.", body = [MarketingCampaignLifecycleAuditRecord]),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 500, description = "Failed to load marketing campaign lifecycle audit trail.", body = ErrorResponse)
        )
    )]
    pub(super) async fn marketing_campaign_lifecycle_audits_list() {}

    #[utoipa::path(
        get,
        path = "/admin/marketing/budgets",
        tag = "marketing",
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible canonical campaign budgets.", body = [CampaignBudgetRecord]),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 500, description = "Failed to load canonical campaign budgets.", body = ErrorResponse)
        )
    )]
    pub(super) async fn marketing_budgets_list() {}

    #[utoipa::path(
        post,
        path = "/admin/marketing/budgets",
        tag = "marketing",
        request_body = CampaignBudgetRecord,
        security(("bearerAuth" = [])),
        responses(
            (status = 201, description = "Created or updated canonical campaign budget.", body = CampaignBudgetRecord),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 500, description = "Failed to persist canonical campaign budget.", body = ErrorResponse)
        )
    )]
    pub(super) async fn marketing_budgets_create() {}

    #[utoipa::path(
        post,
        path = "/admin/marketing/budgets/{campaign_budget_id}/status",
        tag = "marketing",
        params(("campaign_budget_id" = String, Path, description = "Campaign budget id")),
        request_body = UpdateCampaignBudgetStatusRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Updated canonical campaign budget status.", body = CampaignBudgetRecord),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 404, description = "Canonical campaign budget not found.", body = ErrorResponse),
            (status = 500, description = "Failed to update canonical campaign budget status.", body = ErrorResponse)
        )
    )]
    pub(super) async fn marketing_budgets_status_update() {}

    #[utoipa::path(
        post,
        path = "/admin/marketing/budgets/{campaign_budget_id}/activate",
        tag = "marketing",
        params(("campaign_budget_id" = String, Path, description = "Campaign budget id")),
        request_body = ActivateCampaignBudgetRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Activated the selected canonical campaign budget with semantic lifecycle evidence.", body = CampaignBudgetMutationResult),
            (status = 400, description = "Campaign budget cannot be activated from the current governance state.", body = ErrorResponse),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 404, description = "Canonical campaign budget not found.", body = ErrorResponse),
            (status = 500, description = "Failed to activate canonical campaign budget.", body = ErrorResponse)
        )
    )]
    pub(super) async fn marketing_budgets_activate() {}

    #[utoipa::path(
        post,
        path = "/admin/marketing/budgets/{campaign_budget_id}/close",
        tag = "marketing",
        params(("campaign_budget_id" = String, Path, description = "Campaign budget id")),
        request_body = CloseCampaignBudgetRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Closed the selected canonical campaign budget with semantic lifecycle evidence.", body = CampaignBudgetMutationResult),
            (status = 400, description = "Campaign budget cannot be closed from the current governance state.", body = ErrorResponse),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 404, description = "Canonical campaign budget not found.", body = ErrorResponse),
            (status = 500, description = "Failed to close canonical campaign budget.", body = ErrorResponse)
        )
    )]
    pub(super) async fn marketing_budgets_close() {}

    #[utoipa::path(
        get,
        path = "/admin/marketing/budgets/{campaign_budget_id}/lifecycle-audits",
        tag = "marketing",
        params(("campaign_budget_id" = String, Path, description = "Campaign budget id")),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Lifecycle audit trail for the selected canonical campaign budget.", body = [CampaignBudgetLifecycleAuditRecord]),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 500, description = "Failed to load campaign budget lifecycle audit trail.", body = ErrorResponse)
        )
    )]
    pub(super) async fn marketing_budget_lifecycle_audits_list() {}

    #[utoipa::path(
        get,
        path = "/admin/marketing/codes",
        tag = "marketing",
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible canonical coupon codes.", body = [CouponCodeRecord]),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 500, description = "Failed to load canonical coupon codes.", body = ErrorResponse)
        )
    )]
    pub(super) async fn marketing_codes_list() {}

    #[utoipa::path(
        post,
        path = "/admin/marketing/codes",
        tag = "marketing",
        request_body = CouponCodeRecord,
        security(("bearerAuth" = [])),
        responses(
            (status = 201, description = "Created or updated canonical coupon code.", body = CouponCodeRecord),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 500, description = "Failed to persist canonical coupon code.", body = ErrorResponse)
        )
    )]
    pub(super) async fn marketing_codes_create() {}

    #[utoipa::path(
        post,
        path = "/admin/marketing/codes/{coupon_code_id}/status",
        tag = "marketing",
        params(("coupon_code_id" = String, Path, description = "Coupon code id")),
        request_body = UpdateCouponCodeStatusRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Updated canonical coupon code status.", body = CouponCodeRecord),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 404, description = "Canonical coupon code not found.", body = ErrorResponse),
            (status = 500, description = "Failed to update canonical coupon code status.", body = ErrorResponse)
        )
    )]
    pub(super) async fn marketing_codes_status_update() {}

    #[utoipa::path(
        post,
        path = "/admin/marketing/codes/{coupon_code_id}/disable",
        tag = "marketing",
        params(("coupon_code_id" = String, Path, description = "Coupon code id")),
        request_body = DisableCouponCodeRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Disabled the selected canonical coupon code with semantic lifecycle evidence.", body = CouponCodeMutationResult),
            (status = 400, description = "Coupon code cannot be disabled from the current governance state.", body = ErrorResponse),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 404, description = "Canonical coupon code not found.", body = ErrorResponse),
            (status = 500, description = "Failed to disable canonical coupon code.", body = ErrorResponse)
        )
    )]
    pub(super) async fn marketing_codes_disable() {}

    #[utoipa::path(
        post,
        path = "/admin/marketing/codes/{coupon_code_id}/restore",
        tag = "marketing",
        params(("coupon_code_id" = String, Path, description = "Coupon code id")),
        request_body = RestoreCouponCodeRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Restored the selected canonical coupon code with semantic lifecycle evidence.", body = CouponCodeMutationResult),
            (status = 400, description = "Coupon code cannot be restored from the current governance state.", body = ErrorResponse),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 404, description = "Canonical coupon code not found.", body = ErrorResponse),
            (status = 500, description = "Failed to restore canonical coupon code.", body = ErrorResponse)
        )
    )]
    pub(super) async fn marketing_codes_restore() {}

    #[utoipa::path(
        get,
        path = "/admin/marketing/codes/{coupon_code_id}/lifecycle-audits",
        tag = "marketing",
        params(("coupon_code_id" = String, Path, description = "Coupon code id")),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Lifecycle audit trail for the selected canonical coupon code.", body = [CouponCodeLifecycleAuditRecord]),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 500, description = "Failed to load coupon code lifecycle audit trail.", body = ErrorResponse)
        )
    )]
    pub(super) async fn marketing_code_lifecycle_audits_list() {}

    #[utoipa::path(
        get,
        path = "/admin/marketing/reservations",
        tag = "marketing",
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible canonical coupon reservations.", body = [CouponReservationRecord]),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 500, description = "Failed to load canonical coupon reservations.", body = ErrorResponse)
        )
    )]
    pub(super) async fn marketing_reservations_list() {}

    #[utoipa::path(
        get,
        path = "/admin/marketing/redemptions",
        tag = "marketing",
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible canonical coupon redemptions.", body = [CouponRedemptionRecord]),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 500, description = "Failed to load canonical coupon redemptions.", body = ErrorResponse)
        )
    )]
    pub(super) async fn marketing_redemptions_list() {}

    #[utoipa::path(
        get,
        path = "/admin/marketing/rollbacks",
        tag = "marketing",
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible canonical coupon rollback records.", body = [CouponRollbackRecord]),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 500, description = "Failed to load canonical coupon rollback records.", body = ErrorResponse)
        )
    )]
    pub(super) async fn marketing_rollbacks_list() {}

    #[utoipa::path(
        get,
        path = "/admin/api-keys",
        tag = "gateway",
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible gateway API keys.", body = [GatewayApiKeyRecord]),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 500, description = "Failed to load gateway API keys.")
        )
    )]
    pub(super) async fn api_keys_list() {}

    #[utoipa::path(
        post,
        path = "/admin/api-keys",
        tag = "gateway",
        request_body = CreateApiKeyRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 201, description = "Created gateway API key.", body = CreatedGatewayApiKey),
            (status = 400, description = "Invalid gateway API key payload.", body = ErrorResponse),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 500, description = "Failed to create gateway API key.", body = ErrorResponse)
        )
    )]
    pub(super) async fn api_keys_create() {}

    #[utoipa::path(
        put,
        path = "/admin/api-keys/{hashed_key}",
        tag = "gateway",
        params(("hashed_key" = String, Path, description = "Hashed gateway API key identifier.")),
        request_body = UpdateApiKeyRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Updated gateway API key metadata.", body = GatewayApiKeyRecord),
            (status = 400, description = "Invalid gateway API key update payload.", body = ErrorResponse),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 404, description = "Gateway API key not found.", body = ErrorResponse),
            (status = 500, description = "Failed to update gateway API key.", body = ErrorResponse)
        )
    )]
    pub(super) async fn api_key_update() {}

    #[utoipa::path(
        get,
        path = "/admin/api-key-groups",
        tag = "gateway",
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible gateway API key groups.", body = [ApiKeyGroupRecord]),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 500, description = "Failed to load gateway API key groups.")
        )
    )]
    pub(super) async fn api_key_groups_list() {}

    #[utoipa::path(
        post,
        path = "/admin/api-key-groups",
        tag = "gateway",
        request_body = CreateApiKeyGroupRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 201, description = "Created gateway API key group.", body = ApiKeyGroupRecord),
            (status = 400, description = "Invalid gateway API key group payload.", body = ErrorResponse),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 500, description = "Failed to create gateway API key group.", body = ErrorResponse)
        )
    )]
    pub(super) async fn api_key_groups_create() {}

    #[utoipa::path(
        patch,
        path = "/admin/api-key-groups/{group_id}",
        tag = "gateway",
        params(("group_id" = String, Path, description = "Gateway API key group identifier.")),
        request_body = UpdateApiKeyGroupRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Updated gateway API key group.", body = ApiKeyGroupRecord),
            (status = 400, description = "Invalid gateway API key group update payload.", body = ErrorResponse),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 404, description = "Gateway API key group not found.", body = ErrorResponse),
            (status = 500, description = "Failed to update gateway API key group.", body = ErrorResponse)
        )
    )]
    pub(super) async fn api_key_group_update() {}

    #[utoipa::path(
        get,
        path = "/admin/billing/ledger",
        tag = "billing",
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible billing ledger entries.", body = [LedgerEntry]),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 500, description = "Failed to load billing ledger.")
        )
    )]
    pub(super) async fn billing_ledger_list() {}

    #[utoipa::path(
        get,
        path = "/admin/billing/events",
        tag = "billing",
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible billing events.", body = [BillingEventRecord]),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 500, description = "Failed to load billing events.")
        )
    )]
    pub(super) async fn billing_events_list() {}

    #[utoipa::path(
        get,
        path = "/admin/billing/events/summary",
        tag = "billing",
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Billing events summary.", body = BillingEventSummary),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 500, description = "Failed to load billing event summary.")
        )
    )]
    pub(super) async fn billing_events_summary() {}

    #[utoipa::path(
        get,
        path = "/admin/billing/summary",
        tag = "billing",
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Billing summary.", body = BillingSummary),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 500, description = "Failed to load billing summary.")
        )
    )]
    pub(super) async fn billing_summary() {}

    #[utoipa::path(
        post,
        path = "/admin/billing/pricing-lifecycle/synchronize",
        tag = "billing",
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Synchronized due planned commercial pricing lifecycle state.", body = PricingLifecycleSynchronizationReport),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 500, description = "Failed to synchronize commercial pricing lifecycle.", body = ErrorResponse)
        )
    )]
    pub(super) async fn billing_pricing_lifecycle_synchronize() {}

    #[utoipa::path(
        get,
        path = "/admin/billing/accounts/{account_id}/ledger",
        tag = "billing",
        security(("bearerAuth" = [])),
        params(("account_id" = u64, Path, description = "Canonical commercial account identifier.")),
        responses(
            (status = 200, description = "Canonical account ledger history.", body = [AccountLedgerHistoryEntry]),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 404, description = "Account not found.", body = ErrorResponse),
            (status = 501, description = "Commercial billing kernel is not configured.", body = ErrorResponse),
            (status = 500, description = "Failed to load canonical account ledger history.", body = ErrorResponse)
        )
    )]
    pub(super) async fn billing_account_ledger() {}

    #[utoipa::path(
        get,
        path = "/admin/commerce/orders",
        tag = "commerce",
        security(("bearerAuth" = [])),
        params(
            ("limit" = Option<usize>, Query, description = "Maximum number of recent commerce orders to return. Defaults to 24 and is capped at 100.")
        ),
        responses(
            (status = 200, description = "Recent commerce orders ordered by newest activity first.", body = [CommerceOrderRecord]),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 500, description = "Failed to load recent commerce orders.", body = ErrorResponse)
        )
    )]
    pub(super) async fn commerce_orders_recent() {}

    #[utoipa::path(
        get,
        path = "/admin/commerce/catalog-publications",
        tag = "commerce",
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Canonical commercial publication projections derived from the current product, offer, and pricing governance truth.", body = [CommercialCatalogPublicationProjection]),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 500, description = "Failed to load canonical commercial publication projections.", body = ErrorResponse)
        )
    )]
    pub(super) async fn commerce_catalog_publications_list() {}

    #[utoipa::path(
        get,
        path = "/admin/commerce/catalog-publications/{publication_id}",
        tag = "commerce",
        security(("bearerAuth" = [])),
        params(("publication_id" = String, Path, description = "Canonical commercial publication identifier.")),
        responses(
            (status = 200, description = "Canonical commercial publication detail with resolved governed pricing context.", body = CommercialCatalogPublicationDetail),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 404, description = "Publication not found.", body = ErrorResponse),
            (status = 500, description = "Failed to load canonical commercial publication detail.", body = ErrorResponse)
        )
    )]
    pub(super) async fn commerce_catalog_publication_detail() {}

    #[utoipa::path(
        post,
        path = "/admin/commerce/catalog-publications/{publication_id}/publish",
        tag = "commerce",
        request_body = PublishCommercialCatalogPublicationRequest,
        security(("bearerAuth" = [])),
        params(("publication_id" = String, Path, description = "Canonical commercial publication identifier.")),
        responses(
            (status = 200, description = "Published the selected canonical commercial publication and recorded lifecycle audit evidence.", body = CommercialCatalogPublicationMutationResult),
            (status = 400, description = "Publication cannot be published from the current governance state.", body = ErrorResponse),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 404, description = "Publication not found.", body = ErrorResponse),
            (status = 500, description = "Failed to publish canonical commercial publication.", body = ErrorResponse)
        )
    )]
    pub(super) async fn commerce_catalog_publication_publish() {}

    #[utoipa::path(
        post,
        path = "/admin/commerce/catalog-publications/{publication_id}/schedule",
        tag = "commerce",
        request_body = ScheduleCommercialCatalogPublicationRequest,
        security(("bearerAuth" = [])),
        params(("publication_id" = String, Path, description = "Canonical commercial publication identifier.")),
        responses(
            (status = 200, description = "Scheduled the selected canonical commercial publication and recorded lifecycle audit evidence.", body = CommercialCatalogPublicationMutationResult),
            (status = 400, description = "Publication cannot be scheduled from the current governance state.", body = ErrorResponse),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 404, description = "Publication not found.", body = ErrorResponse),
            (status = 500, description = "Failed to schedule canonical commercial publication.", body = ErrorResponse)
        )
    )]
    pub(super) async fn commerce_catalog_publication_schedule() {}

    #[utoipa::path(
        post,
        path = "/admin/commerce/catalog-publications/{publication_id}/retire",
        tag = "commerce",
        request_body = RetireCommercialCatalogPublicationRequest,
        security(("bearerAuth" = [])),
        params(("publication_id" = String, Path, description = "Canonical commercial publication identifier.")),
        responses(
            (status = 200, description = "Retired the selected canonical commercial publication and recorded lifecycle audit evidence.", body = CommercialCatalogPublicationMutationResult),
            (status = 400, description = "Publication cannot be retired from the current governance state.", body = ErrorResponse),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 404, description = "Publication not found.", body = ErrorResponse),
            (status = 500, description = "Failed to retire canonical commercial publication.", body = ErrorResponse)
        )
    )]
    pub(super) async fn commerce_catalog_publication_retire() {}

    #[utoipa::path(
        get,
        path = "/admin/commerce/payment-methods",
        tag = "commerce",
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Configured payment methods ordered for admin display.", body = [PaymentMethodRecord]),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 500, description = "Failed to load payment methods.", body = ErrorResponse)
        )
    )]
    pub(super) async fn commerce_payment_methods_list() {}

    #[utoipa::path(
        put,
        path = "/admin/commerce/payment-methods/{payment_method_id}",
        tag = "commerce",
        request_body = PaymentMethodRecord,
        security(("bearerAuth" = [])),
        params(("payment_method_id" = String, Path, description = "Stable payment method identifier.")),
        responses(
            (status = 200, description = "Saved payment method configuration.", body = PaymentMethodRecord),
            (status = 400, description = "Invalid payment method payload.", body = ErrorResponse),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 500, description = "Failed to save payment method.", body = ErrorResponse)
        )
    )]
    pub(super) async fn commerce_payment_method_put() {}

    #[utoipa::path(
        delete,
        path = "/admin/commerce/payment-methods/{payment_method_id}",
        tag = "commerce",
        security(("bearerAuth" = [])),
        params(("payment_method_id" = String, Path, description = "Stable payment method identifier.")),
        responses(
            (status = 204, description = "Payment method deleted."),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 404, description = "Payment method not found.", body = ErrorResponse),
            (status = 500, description = "Failed to delete payment method.", body = ErrorResponse)
        )
    )]
    pub(super) async fn commerce_payment_method_delete() {}

    #[utoipa::path(
        get,
        path = "/admin/commerce/payment-methods/{payment_method_id}/credential-bindings",
        tag = "commerce",
        security(("bearerAuth" = [])),
        params(("payment_method_id" = String, Path, description = "Stable payment method identifier.")),
        responses(
            (status = 200, description = "Credential bindings under the selected payment method.", body = [PaymentMethodCredentialBindingRecord]),
            (status = 400, description = "Invalid payment method identifier.", body = ErrorResponse),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 500, description = "Failed to load payment method bindings.", body = ErrorResponse)
        )
    )]
    pub(super) async fn commerce_payment_method_bindings_list() {}

    #[utoipa::path(
        put,
        path = "/admin/commerce/payment-methods/{payment_method_id}/credential-bindings",
        tag = "commerce",
        request_body = [PaymentMethodCredentialBindingRecord],
        security(("bearerAuth" = [])),
        params(("payment_method_id" = String, Path, description = "Stable payment method identifier.")),
        responses(
            (status = 200, description = "Replaced credential bindings for the selected payment method.", body = [PaymentMethodCredentialBindingRecord]),
            (status = 400, description = "Invalid binding payload.", body = ErrorResponse),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 500, description = "Failed to replace payment method bindings.", body = ErrorResponse)
        )
    )]
    pub(super) async fn commerce_payment_method_bindings_replace() {}

    #[utoipa::path(
        get,
        path = "/admin/commerce/orders/{order_id}/payment-events",
        tag = "commerce",
        security(("bearerAuth" = [])),
        params(("order_id" = String, Path, description = "Commerce order id.")),
        responses(
            (status = 200, description = "Payment events recorded for the selected commerce order.", body = [CommercePaymentEventRecord]),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 500, description = "Failed to load commerce payment events.", body = ErrorResponse)
        )
    )]
    pub(super) async fn commerce_order_payment_events() {}

    #[utoipa::path(
        get,
        path = "/admin/commerce/orders/{order_id}/payment-attempts",
        tag = "commerce",
        security(("bearerAuth" = [])),
        params(("order_id" = String, Path, description = "Commerce order id.")),
        responses(
            (status = 200, description = "Payment attempts recorded for the selected commerce order.", body = [CommercePaymentAttemptRecord]),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 500, description = "Failed to load commerce payment attempts.", body = ErrorResponse)
        )
    )]
    pub(super) async fn commerce_order_payment_attempts() {}

    #[utoipa::path(
        get,
        path = "/admin/commerce/orders/{order_id}/refunds",
        tag = "commerce",
        security(("bearerAuth" = [])),
        params(("order_id" = String, Path, description = "Commerce order id.")),
        responses(
            (status = 200, description = "Refunds recorded for the selected commerce order.", body = [CommerceRefundRecord]),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 500, description = "Failed to load commerce refunds.", body = ErrorResponse)
        )
    )]
    pub(super) async fn commerce_order_refunds_list() {}

    #[utoipa::path(
        post,
        path = "/admin/commerce/orders/{order_id}/refunds",
        tag = "commerce",
        request_body = AdminCommerceRefundCreateRequest,
        security(("bearerAuth" = [])),
        params(("order_id" = String, Path, description = "Commerce order id.")),
        responses(
            (status = 200, description = "Created refund for the selected commerce order.", body = CommerceRefundRecord),
            (status = 400, description = "Invalid refund request.", body = ErrorResponse),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 500, description = "Failed to create commerce refund.", body = ErrorResponse)
        )
    )]
    pub(super) async fn commerce_order_refunds_create() {}

    #[utoipa::path(
        get,
        path = "/admin/commerce/orders/{order_id}/audit",
        tag = "commerce",
        security(("bearerAuth" = [])),
        params(("order_id" = String, Path, description = "Commerce order id.")),
        responses(
            (status = 200, description = "Aggregated payment and coupon evidence chain for the selected commerce order.", body = CommerceOrderAuditRecord),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 404, description = "Commerce order not found.", body = ErrorResponse),
            (status = 500, description = "Failed to load commerce order audit evidence.", body = ErrorResponse)
        )
    )]
    pub(super) async fn commerce_order_audit() {}

    #[utoipa::path(
        get,
        path = "/admin/commerce/webhook-inbox",
        tag = "commerce",
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Webhook inbox records ordered by newest delivery first.", body = [CommerceWebhookInboxRecord]),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 500, description = "Failed to load webhook inbox.", body = ErrorResponse)
        )
    )]
    pub(super) async fn commerce_webhook_inbox_list() {}

    #[utoipa::path(
        get,
        path = "/admin/commerce/webhook-inbox/{webhook_inbox_id}/delivery-attempts",
        tag = "commerce",
        security(("bearerAuth" = [])),
        params(("webhook_inbox_id" = String, Path, description = "Webhook inbox id.")),
        responses(
            (status = 200, description = "Delivery attempts recorded for the selected webhook inbox record.", body = [CommerceWebhookDeliveryAttemptRecord]),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 500, description = "Failed to load webhook delivery attempts.", body = ErrorResponse)
        )
    )]
    pub(super) async fn commerce_webhook_delivery_attempts_list() {}

    #[utoipa::path(
        get,
        path = "/admin/commerce/reconciliation-runs",
        tag = "commerce",
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Reconciliation runs ordered by newest execution first.", body = [CommerceReconciliationRunRecord]),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 500, description = "Failed to load reconciliation runs.", body = ErrorResponse)
        )
    )]
    pub(super) async fn commerce_reconciliation_runs_list() {}

    #[utoipa::path(
        post,
        path = "/admin/commerce/reconciliation-runs",
        tag = "commerce",
        request_body = AdminCommerceReconciliationRunCreateRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Created reconciliation run.", body = CommerceReconciliationRunRecord),
            (status = 400, description = "Invalid reconciliation request.", body = ErrorResponse),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 500, description = "Failed to create reconciliation run.", body = ErrorResponse)
        )
    )]
    pub(super) async fn commerce_reconciliation_runs_create() {}

    #[utoipa::path(
        get,
        path = "/admin/commerce/reconciliation-runs/{reconciliation_run_id}/items",
        tag = "commerce",
        security(("bearerAuth" = [])),
        params(("reconciliation_run_id" = String, Path, description = "Reconciliation run id.")),
        responses(
            (status = 200, description = "Discrepancy items recorded for the selected reconciliation run.", body = [CommerceReconciliationItemRecord]),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 500, description = "Failed to load reconciliation items.", body = ErrorResponse)
        )
    )]
    pub(super) async fn commerce_reconciliation_items_list() {}
}

fn admin_openapi() -> utoipa::openapi::OpenApi {
    OpenApiRouter::<()>::with_openapi(AdminApiDoc::openapi())
        .routes(routes!(openapi_paths::health))
        .routes(routes!(openapi_paths::auth_login))
        .routes(routes!(openapi_paths::auth_change_password))
        .routes(routes!(openapi_paths::tenants_list))
        .routes(routes!(openapi_paths::tenants_create))
        .routes(routes!(openapi_paths::tenant_provider_readiness_list))
        .routes(routes!(openapi_paths::projects_list))
        .routes(routes!(openapi_paths::projects_create))
        .routes(routes!(openapi_paths::providers_list))
        .routes(routes!(openapi_paths::providers_create))
        .routes(routes!(openapi_paths::operator_users_list))
        .routes(routes!(openapi_paths::operator_users_upsert))
        .routes(routes!(openapi_paths::operator_user_status_update))
        .routes(routes!(openapi_paths::portal_users_list))
        .routes(routes!(openapi_paths::portal_users_upsert))
        .routes(routes!(openapi_paths::portal_user_status_update))
        .routes(routes!(openapi_paths::marketing_coupon_templates_list))
        .routes(routes!(openapi_paths::marketing_coupon_templates_create))
        .routes(routes!(
            openapi_paths::marketing_coupon_templates_status_update
        ))
        .routes(routes!(openapi_paths::marketing_coupon_templates_clone))
        .routes(routes!(openapi_paths::marketing_coupon_templates_compare))
        .routes(routes!(
            openapi_paths::marketing_coupon_templates_submit_for_approval
        ))
        .routes(routes!(openapi_paths::marketing_coupon_templates_approve))
        .routes(routes!(openapi_paths::marketing_coupon_templates_reject))
        .routes(routes!(openapi_paths::marketing_coupon_templates_publish))
        .routes(routes!(openapi_paths::marketing_coupon_templates_schedule))
        .routes(routes!(openapi_paths::marketing_coupon_templates_retire))
        .routes(routes!(
            openapi_paths::marketing_coupon_template_lifecycle_audits_list
        ))
        .routes(routes!(openapi_paths::marketing_campaigns_list))
        .routes(routes!(openapi_paths::marketing_campaigns_create))
        .routes(routes!(openapi_paths::marketing_campaigns_status_update))
        .routes(routes!(openapi_paths::marketing_campaigns_clone))
        .routes(routes!(openapi_paths::marketing_campaigns_compare))
        .routes(routes!(
            openapi_paths::marketing_campaigns_submit_for_approval
        ))
        .routes(routes!(openapi_paths::marketing_campaigns_approve))
        .routes(routes!(openapi_paths::marketing_campaigns_reject))
        .routes(routes!(openapi_paths::marketing_campaigns_publish))
        .routes(routes!(openapi_paths::marketing_campaigns_schedule))
        .routes(routes!(openapi_paths::marketing_campaigns_retire))
        .routes(routes!(
            openapi_paths::marketing_campaign_lifecycle_audits_list
        ))
        .routes(routes!(openapi_paths::marketing_budgets_list))
        .routes(routes!(openapi_paths::marketing_budgets_create))
        .routes(routes!(openapi_paths::marketing_budgets_status_update))
        .routes(routes!(openapi_paths::marketing_budgets_activate))
        .routes(routes!(openapi_paths::marketing_budgets_close))
        .routes(routes!(
            openapi_paths::marketing_budget_lifecycle_audits_list
        ))
        .routes(routes!(openapi_paths::marketing_codes_list))
        .routes(routes!(openapi_paths::marketing_codes_create))
        .routes(routes!(openapi_paths::marketing_codes_status_update))
        .routes(routes!(openapi_paths::marketing_codes_disable))
        .routes(routes!(openapi_paths::marketing_codes_restore))
        .routes(routes!(openapi_paths::marketing_code_lifecycle_audits_list))
        .routes(routes!(openapi_paths::marketing_reservations_list))
        .routes(routes!(openapi_paths::marketing_redemptions_list))
        .routes(routes!(openapi_paths::marketing_rollbacks_list))
        .routes(routes!(openapi_paths::api_keys_list))
        .routes(routes!(openapi_paths::api_keys_create))
        .routes(routes!(openapi_paths::api_key_update))
        .routes(routes!(openapi_paths::api_key_groups_list))
        .routes(routes!(openapi_paths::api_key_groups_create))
        .routes(routes!(openapi_paths::api_key_group_update))
        .routes(routes!(openapi_paths::billing_ledger_list))
        .routes(routes!(openapi_paths::billing_events_list))
        .routes(routes!(openapi_paths::billing_events_summary))
        .routes(routes!(openapi_paths::billing_summary))
        .routes(routes!(
            openapi_paths::billing_pricing_lifecycle_synchronize
        ))
        .routes(routes!(openapi_paths::billing_account_ledger))
        .routes(routes!(openapi_paths::commerce_orders_recent))
        .routes(routes!(openapi_paths::commerce_catalog_publications_list))
        .routes(routes!(openapi_paths::commerce_catalog_publication_detail))
        .routes(routes!(openapi_paths::commerce_catalog_publication_publish))
        .routes(routes!(openapi_paths::commerce_catalog_publication_schedule))
        .routes(routes!(openapi_paths::commerce_catalog_publication_retire))
        .routes(routes!(openapi_paths::commerce_payment_methods_list))
        .routes(routes!(openapi_paths::commerce_payment_method_put))
        .routes(routes!(openapi_paths::commerce_payment_method_delete))
        .routes(routes!(
            openapi_paths::commerce_payment_method_bindings_list
        ))
        .routes(routes!(
            openapi_paths::commerce_payment_method_bindings_replace
        ))
        .routes(routes!(openapi_paths::commerce_order_payment_events))
        .routes(routes!(openapi_paths::commerce_order_payment_attempts))
        .routes(routes!(openapi_paths::commerce_order_refunds_list))
        .routes(routes!(openapi_paths::commerce_order_refunds_create))
        .routes(routes!(openapi_paths::commerce_order_audit))
        .routes(routes!(openapi_paths::commerce_webhook_inbox_list))
        .routes(routes!(
            openapi_paths::commerce_webhook_delivery_attempts_list
        ))
        .routes(routes!(openapi_paths::commerce_reconciliation_runs_list))
        .routes(routes!(openapi_paths::commerce_reconciliation_runs_create))
        .routes(routes!(openapi_paths::commerce_reconciliation_items_list))
        .into_openapi()
}

async fn admin_openapi_handler() -> Json<utoipa::openapi::OpenApi> {
    Json(admin_openapi())
}

async fn admin_docs_index_handler() -> Html<String> {
    Html(
        r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>SDKWORK Admin API</title>
    <style>
      :root {
        color-scheme: light dark;
        font-family: Inter, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
      }

      body {
        margin: 0;
        background: #f5f7fb;
        color: #101828;
      }

      .shell {
        display: grid;
        min-height: 100vh;
        grid-template-rows: auto 1fr;
      }

      .hero {
        padding: 20px 24px 16px;
        border-bottom: 1px solid rgba(15, 23, 42, 0.08);
        background: rgba(255, 255, 255, 0.96);
      }

      .eyebrow {
        margin: 0 0 8px;
        font-size: 12px;
        font-weight: 700;
        letter-spacing: 0.12em;
        text-transform: uppercase;
        color: #475467;
      }

      h1 {
        margin: 0 0 8px;
        font-size: 28px;
        line-height: 1.1;
      }

      p {
        margin: 0;
        font-size: 14px;
        line-height: 1.6;
        color: #475467;
      }

      code {
        padding: 2px 6px;
        border-radius: 999px;
        background: rgba(15, 23, 42, 0.06);
        font-size: 12px;
      }

      iframe {
        width: 100%;
        height: 100%;
        border: 0;
        background: white;
      }

      @media (prefers-color-scheme: dark) {
        body {
          background: #09090b;
          color: #fafafa;
        }

        .hero {
          background: rgba(24, 24, 27, 0.96);
          border-bottom-color: rgba(255, 255, 255, 0.08);
        }

        .eyebrow,
        p {
          color: #a1a1aa;
        }

        code {
          background: rgba(255, 255, 255, 0.08);
        }
      }
    </style>
  </head>
  <body>
    <main class="shell">
      <section class="hero">
        <p class="eyebrow">OpenAPI 3.1</p>
        <h1>SDKWORK Admin API</h1>
        <p>Interactive documentation is backed by the live schema endpoint <code>/admin/openapi.json</code>.</p>
      </section>
      <iframe src="/admin/docs/ui/" title="SDKWORK Admin API"></iframe>
    </main>
  </body>
</html>"#
            .to_string(),
    )
}

pub(crate) fn admin_docs_router<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    Router::new()
        .route("/admin/openapi.json", get(admin_openapi_handler))
        .route("/admin/docs", get(admin_docs_index_handler))
        .merge(
            SwaggerUi::new("/admin/docs/ui/").config(SwaggerUiConfig::new([
                SwaggerUiUrl::with_primary("SDKWORK Admin API", "/admin/openapi.json", true),
            ])),
        )
}
