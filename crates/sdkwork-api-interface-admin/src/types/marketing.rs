use super::*;

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct CloneCouponTemplateRequest {
    pub(crate) coupon_template_id: String,
    pub(crate) template_key: String,
    #[serde(default)]
    pub(crate) display_name: Option<String>,
    pub(crate) reason: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct CompareCouponTemplateRequest {
    pub(crate) target_coupon_template_id: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct SubmitCouponTemplateForApprovalRequest {
    pub(crate) reason: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct UpdateCouponTemplateStatusRequest {
    pub(crate) status: CouponTemplateStatus,
}

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct ApproveCouponTemplateRequest {
    pub(crate) reason: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct RejectCouponTemplateRequest {
    pub(crate) reason: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct PublishCouponTemplateRequest {
    pub(crate) reason: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct ScheduleCouponTemplateRequest {
    pub(crate) reason: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct RetireCouponTemplateRequest {
    pub(crate) reason: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct CloneMarketingCampaignRequest {
    pub(crate) marketing_campaign_id: String,
    #[serde(default)]
    pub(crate) display_name: Option<String>,
    pub(crate) reason: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct CompareMarketingCampaignRequest {
    pub(crate) target_marketing_campaign_id: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct SubmitMarketingCampaignForApprovalRequest {
    pub(crate) reason: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct UpdateMarketingCampaignStatusRequest {
    pub(crate) status: MarketingCampaignStatus,
}

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct ApproveMarketingCampaignRequest {
    pub(crate) reason: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct RejectMarketingCampaignRequest {
    pub(crate) reason: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct PublishMarketingCampaignRequest {
    pub(crate) reason: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct ScheduleMarketingCampaignRequest {
    pub(crate) reason: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct RetireMarketingCampaignRequest {
    pub(crate) reason: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct ActivateCampaignBudgetRequest {
    pub(crate) reason: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct UpdateCampaignBudgetStatusRequest {
    pub(crate) status: CampaignBudgetStatus,
}

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct CloseCampaignBudgetRequest {
    pub(crate) reason: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct UpdateCouponCodeStatusRequest {
    pub(crate) status: CouponCodeStatus,
}

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct DisableCouponCodeRequest {
    pub(crate) reason: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct RestoreCouponCodeRequest {
    pub(crate) reason: String,
}
