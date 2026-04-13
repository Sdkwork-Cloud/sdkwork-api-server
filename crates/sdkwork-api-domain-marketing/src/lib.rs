use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CouponBenefitKind {
    PercentageDiscount,
    FixedAmountDiscount,
    CreditGrant,
    TokenGrant,
    RequestGrant,
    ImageGrant,
    AudioGrant,
    VideoGrant,
    MusicGrant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CouponDistributionKind {
    SharedCode,
    UniqueCode,
    InviteCode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CouponStackingPolicy {
    Stackable,
    ExclusiveWithinGroup,
    ExclusiveGlobal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CouponTemplateStatus {
    Draft,
    Active,
    Paused,
    Archived,
    Expired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MarketingCampaignKind {
    Launch,
    Lifecycle,
    Partner,
    Referral,
    Retention,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MarketingCampaignStatus {
    Draft,
    Active,
    Paused,
    Archived,
    Completed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CouponCodeGenerationMode {
    BulkRandom,
    Vanity,
    Import,
    Invite,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CouponCodeBatchStatus {
    Draft,
    Active,
    Paused,
    Exhausted,
    Archived,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CouponCodeKind {
    Shared,
    SingleUseUnique,
    Vanity,
    Invite,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CouponCodeStatus {
    Issued,
    Claimed,
    Redeemed,
    Voided,
    Expired,
    Blocked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CouponClaimStatus {
    Pending,
    Claimed,
    Cancelled,
    Expired,
    Rejected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CouponRedemptionStatus {
    Pending,
    Fulfilled,
    Voided,
    Reversed,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReferralProgramStatus {
    Draft,
    Active,
    Paused,
    Archived,
    Completed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReferralInviteStatus {
    Issued,
    Accepted,
    Rewarded,
    Expired,
    Revoked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttributionSourceKind {
    Direct,
    Campaign,
    Referral,
    Partner,
    Organic,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CouponTemplateRecord {
    pub coupon_template_id: u64,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub template_code: String,
    pub display_name: String,
    pub benefit_kind: CouponBenefitKind,
    pub distribution_kind: CouponDistributionKind,
    pub status: CouponTemplateStatus,
    pub stacking_policy: CouponStackingPolicy,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exclusive_group: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub starts_at_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ends_at_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_total_redemptions: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_redemptions_per_subject: Option<u64>,
    #[serde(default = "default_claim_required")]
    pub claim_required: bool,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

impl CouponTemplateRecord {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        coupon_template_id: u64,
        tenant_id: u64,
        organization_id: u64,
        template_code: impl Into<String>,
        display_name: impl Into<String>,
        benefit_kind: CouponBenefitKind,
        distribution_kind: CouponDistributionKind,
        created_at_ms: u64,
    ) -> Self {
        Self {
            coupon_template_id,
            tenant_id,
            organization_id,
            template_code: template_code.into(),
            display_name: display_name.into(),
            benefit_kind,
            distribution_kind,
            status: CouponTemplateStatus::Draft,
            stacking_policy: CouponStackingPolicy::Stackable,
            exclusive_group: None,
            starts_at_ms: None,
            ends_at_ms: None,
            max_total_redemptions: None,
            max_redemptions_per_subject: None,
            claim_required: default_claim_required(),
            created_at_ms,
            updated_at_ms: created_at_ms,
        }
    }

    pub fn with_status(mut self, status: CouponTemplateStatus) -> Self {
        self.status = status;
        self
    }

    pub fn with_stacking_policy(mut self, stacking_policy: CouponStackingPolicy) -> Self {
        self.stacking_policy = stacking_policy;
        self
    }

    pub fn with_exclusive_group(mut self, exclusive_group: Option<String>) -> Self {
        self.exclusive_group = exclusive_group;
        self
    }

    pub fn with_starts_at_ms(mut self, starts_at_ms: Option<u64>) -> Self {
        self.starts_at_ms = starts_at_ms;
        self
    }

    pub fn with_ends_at_ms(mut self, ends_at_ms: Option<u64>) -> Self {
        self.ends_at_ms = ends_at_ms;
        self
    }

    pub fn with_max_total_redemptions(mut self, max_total_redemptions: Option<u64>) -> Self {
        self.max_total_redemptions = max_total_redemptions;
        self
    }

    pub fn with_max_redemptions_per_subject(
        mut self,
        max_redemptions_per_subject: Option<u64>,
    ) -> Self {
        self.max_redemptions_per_subject = max_redemptions_per_subject;
        self
    }

    pub fn with_claim_required(mut self, claim_required: bool) -> Self {
        self.claim_required = claim_required;
        self
    }

    pub fn with_updated_at_ms(mut self, updated_at_ms: u64) -> Self {
        self.updated_at_ms = updated_at_ms;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CouponBenefitRuleRecord {
    pub coupon_benefit_rule_id: u64,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub coupon_template_id: u64,
    pub benefit_kind: CouponBenefitKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_order_kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_product_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub percentage_off: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fixed_discount_amount: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub maximum_subsidy_amount: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub grant_quantity: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub currency_code: Option<String>,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

impl CouponBenefitRuleRecord {
    pub fn new(
        coupon_benefit_rule_id: u64,
        tenant_id: u64,
        organization_id: u64,
        coupon_template_id: u64,
        benefit_kind: CouponBenefitKind,
        created_at_ms: u64,
    ) -> Self {
        Self {
            coupon_benefit_rule_id,
            tenant_id,
            organization_id,
            coupon_template_id,
            benefit_kind,
            target_order_kind: None,
            target_product_id: None,
            percentage_off: None,
            fixed_discount_amount: None,
            maximum_subsidy_amount: None,
            grant_quantity: None,
            currency_code: None,
            created_at_ms,
            updated_at_ms: created_at_ms,
        }
    }

    pub fn with_target_order_kind(mut self, target_order_kind: Option<String>) -> Self {
        self.target_order_kind = target_order_kind;
        self
    }

    pub fn with_target_product_id(mut self, target_product_id: Option<String>) -> Self {
        self.target_product_id = target_product_id;
        self
    }

    pub fn with_percentage_off(mut self, percentage_off: Option<f64>) -> Self {
        self.percentage_off = percentage_off;
        self
    }

    pub fn with_fixed_discount_amount(mut self, fixed_discount_amount: Option<f64>) -> Self {
        self.fixed_discount_amount = fixed_discount_amount;
        self
    }

    pub fn with_maximum_subsidy_amount(mut self, maximum_subsidy_amount: Option<f64>) -> Self {
        self.maximum_subsidy_amount = maximum_subsidy_amount;
        self
    }

    pub fn with_grant_quantity(mut self, grant_quantity: Option<f64>) -> Self {
        self.grant_quantity = grant_quantity;
        self
    }

    pub fn with_currency_code(mut self, currency_code: Option<String>) -> Self {
        self.currency_code = currency_code;
        self
    }

    pub fn with_updated_at_ms(mut self, updated_at_ms: u64) -> Self {
        self.updated_at_ms = updated_at_ms;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MarketingCampaignRecord {
    pub marketing_campaign_id: u64,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub campaign_code: String,
    pub display_name: String,
    pub campaign_kind: MarketingCampaignKind,
    pub status: MarketingCampaignStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channel_source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub budget_amount: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub currency_code: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subsidy_cap_amount: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner_user_id: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub starts_at_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ends_at_ms: Option<u64>,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

impl MarketingCampaignRecord {
    pub fn new(
        marketing_campaign_id: u64,
        tenant_id: u64,
        organization_id: u64,
        campaign_code: impl Into<String>,
        display_name: impl Into<String>,
        campaign_kind: MarketingCampaignKind,
        created_at_ms: u64,
    ) -> Self {
        Self {
            marketing_campaign_id,
            tenant_id,
            organization_id,
            campaign_code: campaign_code.into(),
            display_name: display_name.into(),
            campaign_kind,
            status: MarketingCampaignStatus::Draft,
            channel_source: None,
            budget_amount: None,
            currency_code: None,
            subsidy_cap_amount: None,
            owner_user_id: None,
            starts_at_ms: None,
            ends_at_ms: None,
            created_at_ms,
            updated_at_ms: created_at_ms,
        }
    }

    pub fn with_status(mut self, status: MarketingCampaignStatus) -> Self {
        self.status = status;
        self
    }

    pub fn with_channel_source(mut self, channel_source: Option<String>) -> Self {
        self.channel_source = channel_source;
        self
    }

    pub fn with_budget_amount(mut self, budget_amount: Option<f64>) -> Self {
        self.budget_amount = budget_amount;
        self
    }

    pub fn with_currency_code(mut self, currency_code: Option<String>) -> Self {
        self.currency_code = currency_code;
        self
    }

    pub fn with_subsidy_cap_amount(mut self, subsidy_cap_amount: Option<f64>) -> Self {
        self.subsidy_cap_amount = subsidy_cap_amount;
        self
    }

    pub fn with_owner_user_id(mut self, owner_user_id: Option<u64>) -> Self {
        self.owner_user_id = owner_user_id;
        self
    }

    pub fn with_starts_at_ms(mut self, starts_at_ms: Option<u64>) -> Self {
        self.starts_at_ms = starts_at_ms;
        self
    }

    pub fn with_ends_at_ms(mut self, ends_at_ms: Option<u64>) -> Self {
        self.ends_at_ms = ends_at_ms;
        self
    }

    pub fn with_updated_at_ms(mut self, updated_at_ms: u64) -> Self {
        self.updated_at_ms = updated_at_ms;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CouponCodeBatchRecord {
    pub coupon_code_batch_id: u64,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub coupon_template_id: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub marketing_campaign_id: Option<u64>,
    pub generation_mode: CouponCodeGenerationMode,
    pub status: CouponCodeBatchStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub batch_kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code_prefix: Option<String>,
    pub issued_count: u64,
    pub claimed_count: u64,
    pub redeemed_count: u64,
    pub voided_count: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at_ms: Option<u64>,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

impl CouponCodeBatchRecord {
    pub fn new(
        coupon_code_batch_id: u64,
        tenant_id: u64,
        organization_id: u64,
        coupon_template_id: u64,
        marketing_campaign_id: Option<u64>,
        generation_mode: CouponCodeGenerationMode,
        created_at_ms: u64,
    ) -> Self {
        Self {
            coupon_code_batch_id,
            tenant_id,
            organization_id,
            coupon_template_id,
            marketing_campaign_id,
            generation_mode,
            status: CouponCodeBatchStatus::Draft,
            batch_kind: None,
            code_prefix: None,
            issued_count: 0,
            claimed_count: 0,
            redeemed_count: 0,
            voided_count: 0,
            expires_at_ms: None,
            created_at_ms,
            updated_at_ms: created_at_ms,
        }
    }

    pub fn with_status(mut self, status: CouponCodeBatchStatus) -> Self {
        self.status = status;
        self
    }

    pub fn with_batch_kind(mut self, batch_kind: Option<String>) -> Self {
        self.batch_kind = batch_kind;
        self
    }

    pub fn with_code_prefix(mut self, code_prefix: Option<String>) -> Self {
        self.code_prefix = code_prefix;
        self
    }

    pub fn with_issued_count(mut self, issued_count: u64) -> Self {
        self.issued_count = issued_count;
        self
    }

    pub fn with_claimed_count(mut self, claimed_count: u64) -> Self {
        self.claimed_count = claimed_count;
        self
    }

    pub fn with_redeemed_count(mut self, redeemed_count: u64) -> Self {
        self.redeemed_count = redeemed_count;
        self
    }

    pub fn with_voided_count(mut self, voided_count: u64) -> Self {
        self.voided_count = voided_count;
        self
    }

    pub fn with_expires_at_ms(mut self, expires_at_ms: Option<u64>) -> Self {
        self.expires_at_ms = expires_at_ms;
        self
    }

    pub fn with_updated_at_ms(mut self, updated_at_ms: u64) -> Self {
        self.updated_at_ms = updated_at_ms;
        self
    }
}

fn default_claim_required() -> bool {
    true
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CouponCodeRecord {
    pub coupon_code_id: u64,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub coupon_code_batch_id: u64,
    pub coupon_template_id: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub marketing_campaign_id: Option<u64>,
    pub code_lookup_hash: String,
    pub code_kind: CouponCodeKind,
    pub status: CouponCodeStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_code_prefix: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_code_suffix: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub claim_subject_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub claim_subject_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub claimed_at_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub redeemed_at_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at_ms: Option<u64>,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

impl CouponCodeRecord {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        coupon_code_id: u64,
        tenant_id: u64,
        organization_id: u64,
        coupon_code_batch_id: u64,
        coupon_template_id: u64,
        marketing_campaign_id: Option<u64>,
        code_lookup_hash: impl Into<String>,
        code_kind: CouponCodeKind,
        created_at_ms: u64,
    ) -> Self {
        Self {
            coupon_code_id,
            tenant_id,
            organization_id,
            coupon_code_batch_id,
            coupon_template_id,
            marketing_campaign_id,
            code_lookup_hash: code_lookup_hash.into(),
            code_kind,
            status: CouponCodeStatus::Issued,
            display_code_prefix: None,
            display_code_suffix: None,
            claim_subject_type: None,
            claim_subject_id: None,
            claimed_at_ms: None,
            redeemed_at_ms: None,
            expires_at_ms: None,
            created_at_ms,
            updated_at_ms: created_at_ms,
        }
    }

    pub fn with_status(mut self, status: CouponCodeStatus) -> Self {
        self.status = status;
        self
    }

    pub fn with_display_code_prefix(mut self, display_code_prefix: Option<String>) -> Self {
        self.display_code_prefix = display_code_prefix;
        self
    }

    pub fn with_display_code_suffix(mut self, display_code_suffix: Option<String>) -> Self {
        self.display_code_suffix = display_code_suffix;
        self
    }

    pub fn with_claim_subject_type(mut self, claim_subject_type: Option<String>) -> Self {
        self.claim_subject_type = claim_subject_type;
        self
    }

    pub fn with_claim_subject_id(mut self, claim_subject_id: Option<String>) -> Self {
        self.claim_subject_id = claim_subject_id;
        self
    }

    pub fn with_claimed_at_ms(mut self, claimed_at_ms: Option<u64>) -> Self {
        self.claimed_at_ms = claimed_at_ms;
        self
    }

    pub fn with_redeemed_at_ms(mut self, redeemed_at_ms: Option<u64>) -> Self {
        self.redeemed_at_ms = redeemed_at_ms;
        self
    }

    pub fn with_expires_at_ms(mut self, expires_at_ms: Option<u64>) -> Self {
        self.expires_at_ms = expires_at_ms;
        self
    }

    pub fn with_updated_at_ms(mut self, updated_at_ms: u64) -> Self {
        self.updated_at_ms = updated_at_ms;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CouponClaimRecord {
    pub coupon_claim_id: u64,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub coupon_code_id: u64,
    pub coupon_template_id: u64,
    pub subject_type: String,
    pub subject_id: String,
    pub status: CouponClaimStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub account_id: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at_ms: Option<u64>,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

impl CouponClaimRecord {
    pub fn new(
        coupon_claim_id: u64,
        tenant_id: u64,
        organization_id: u64,
        coupon_code_id: u64,
        coupon_template_id: u64,
        subject_type: impl Into<String>,
        subject_id: impl Into<String>,
        created_at_ms: u64,
    ) -> Self {
        Self {
            coupon_claim_id,
            tenant_id,
            organization_id,
            coupon_code_id,
            coupon_template_id,
            subject_type: subject_type.into(),
            subject_id: subject_id.into(),
            status: CouponClaimStatus::Pending,
            account_id: None,
            project_id: None,
            expires_at_ms: None,
            created_at_ms,
            updated_at_ms: created_at_ms,
        }
    }

    pub fn with_status(mut self, status: CouponClaimStatus) -> Self {
        self.status = status;
        self
    }

    pub fn with_account_id(mut self, account_id: Option<u64>) -> Self {
        self.account_id = account_id;
        self
    }

    pub fn with_project_id(mut self, project_id: Option<String>) -> Self {
        self.project_id = project_id;
        self
    }

    pub fn with_expires_at_ms(mut self, expires_at_ms: Option<u64>) -> Self {
        self.expires_at_ms = expires_at_ms;
        self
    }

    pub fn with_updated_at_ms(mut self, updated_at_ms: u64) -> Self {
        self.updated_at_ms = updated_at_ms;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CouponRedemptionRecord {
    pub coupon_redemption_id: u64,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub coupon_code_id: u64,
    pub coupon_template_id: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub marketing_campaign_id: Option<u64>,
    pub subject_type: String,
    pub subject_id: String,
    pub status: CouponRedemptionStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub account_id: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub order_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payment_order_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub benefit_lot_id: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pricing_adjustment_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subsidy_amount: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub currency_code: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<String>,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

impl CouponRedemptionRecord {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        coupon_redemption_id: u64,
        tenant_id: u64,
        organization_id: u64,
        coupon_code_id: u64,
        coupon_template_id: u64,
        marketing_campaign_id: Option<u64>,
        subject_type: impl Into<String>,
        subject_id: impl Into<String>,
        created_at_ms: u64,
    ) -> Self {
        Self {
            coupon_redemption_id,
            tenant_id,
            organization_id,
            coupon_code_id,
            coupon_template_id,
            marketing_campaign_id,
            subject_type: subject_type.into(),
            subject_id: subject_id.into(),
            status: CouponRedemptionStatus::Pending,
            account_id: None,
            project_id: None,
            order_id: None,
            payment_order_id: None,
            benefit_lot_id: None,
            pricing_adjustment_id: None,
            subsidy_amount: None,
            currency_code: None,
            idempotency_key: None,
            created_at_ms,
            updated_at_ms: created_at_ms,
        }
    }

    pub fn with_status(mut self, status: CouponRedemptionStatus) -> Self {
        self.status = status;
        self
    }

    pub fn with_account_id(mut self, account_id: Option<u64>) -> Self {
        self.account_id = account_id;
        self
    }

    pub fn with_project_id(mut self, project_id: Option<String>) -> Self {
        self.project_id = project_id;
        self
    }

    pub fn with_order_id(mut self, order_id: Option<String>) -> Self {
        self.order_id = order_id;
        self
    }

    pub fn with_payment_order_id(mut self, payment_order_id: Option<String>) -> Self {
        self.payment_order_id = payment_order_id;
        self
    }

    pub fn with_benefit_lot_id(mut self, benefit_lot_id: Option<u64>) -> Self {
        self.benefit_lot_id = benefit_lot_id;
        self
    }

    pub fn with_pricing_adjustment_id(mut self, pricing_adjustment_id: Option<String>) -> Self {
        self.pricing_adjustment_id = pricing_adjustment_id;
        self
    }

    pub fn with_subsidy_amount(mut self, subsidy_amount: Option<f64>) -> Self {
        self.subsidy_amount = subsidy_amount;
        self
    }

    pub fn with_currency_code(mut self, currency_code: Option<String>) -> Self {
        self.currency_code = currency_code;
        self
    }

    pub fn with_idempotency_key(mut self, idempotency_key: Option<String>) -> Self {
        self.idempotency_key = idempotency_key;
        self
    }

    pub fn with_updated_at_ms(mut self, updated_at_ms: u64) -> Self {
        self.updated_at_ms = updated_at_ms;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReferralProgramRecord {
    pub referral_program_id: u64,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub program_code: String,
    pub display_name: String,
    pub status: ReferralProgramStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub invite_reward_template_id: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub referee_reward_template_id: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub starts_at_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ends_at_ms: Option<u64>,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

impl ReferralProgramRecord {
    pub fn new(
        referral_program_id: u64,
        tenant_id: u64,
        organization_id: u64,
        program_code: impl Into<String>,
        display_name: impl Into<String>,
        created_at_ms: u64,
    ) -> Self {
        Self {
            referral_program_id,
            tenant_id,
            organization_id,
            program_code: program_code.into(),
            display_name: display_name.into(),
            status: ReferralProgramStatus::Draft,
            invite_reward_template_id: None,
            referee_reward_template_id: None,
            starts_at_ms: None,
            ends_at_ms: None,
            created_at_ms,
            updated_at_ms: created_at_ms,
        }
    }

    pub fn with_status(mut self, status: ReferralProgramStatus) -> Self {
        self.status = status;
        self
    }

    pub fn with_invite_reward_template_id(
        mut self,
        invite_reward_template_id: Option<u64>,
    ) -> Self {
        self.invite_reward_template_id = invite_reward_template_id;
        self
    }

    pub fn with_referee_reward_template_id(
        mut self,
        referee_reward_template_id: Option<u64>,
    ) -> Self {
        self.referee_reward_template_id = referee_reward_template_id;
        self
    }

    pub fn with_starts_at_ms(mut self, starts_at_ms: Option<u64>) -> Self {
        self.starts_at_ms = starts_at_ms;
        self
    }

    pub fn with_ends_at_ms(mut self, ends_at_ms: Option<u64>) -> Self {
        self.ends_at_ms = ends_at_ms;
        self
    }

    pub fn with_updated_at_ms(mut self, updated_at_ms: u64) -> Self {
        self.updated_at_ms = updated_at_ms;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReferralInviteRecord {
    pub referral_invite_id: u64,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub referral_program_id: u64,
    pub referrer_user_id: u64,
    pub status: ReferralInviteStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub coupon_code_id: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_code: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub referred_user_id: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accepted_at_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rewarded_at_ms: Option<u64>,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

impl ReferralInviteRecord {
    pub fn new(
        referral_invite_id: u64,
        tenant_id: u64,
        organization_id: u64,
        referral_program_id: u64,
        referrer_user_id: u64,
        created_at_ms: u64,
    ) -> Self {
        Self {
            referral_invite_id,
            tenant_id,
            organization_id,
            referral_program_id,
            referrer_user_id,
            status: ReferralInviteStatus::Issued,
            coupon_code_id: None,
            source_code: None,
            referred_user_id: None,
            accepted_at_ms: None,
            rewarded_at_ms: None,
            created_at_ms,
            updated_at_ms: created_at_ms,
        }
    }

    pub fn with_status(mut self, status: ReferralInviteStatus) -> Self {
        self.status = status;
        self
    }

    pub fn with_coupon_code_id(mut self, coupon_code_id: Option<u64>) -> Self {
        self.coupon_code_id = coupon_code_id;
        self
    }

    pub fn with_source_code(mut self, source_code: Option<String>) -> Self {
        self.source_code = source_code;
        self
    }

    pub fn with_referred_user_id(mut self, referred_user_id: Option<u64>) -> Self {
        self.referred_user_id = referred_user_id;
        self
    }

    pub fn with_accepted_at_ms(mut self, accepted_at_ms: Option<u64>) -> Self {
        self.accepted_at_ms = accepted_at_ms;
        self
    }

    pub fn with_rewarded_at_ms(mut self, rewarded_at_ms: Option<u64>) -> Self {
        self.rewarded_at_ms = rewarded_at_ms;
        self
    }

    pub fn with_updated_at_ms(mut self, updated_at_ms: u64) -> Self {
        self.updated_at_ms = updated_at_ms;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarketingAttributionTouchRecord {
    pub attribution_touch_id: u64,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub source_kind: AttributionSourceKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_code: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub utm_source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub utm_campaign: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub utm_medium: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub partner_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub referrer_user_id: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub invite_code_id: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub conversion_subject_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub converted_at_ms: Option<u64>,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

impl MarketingAttributionTouchRecord {
    pub fn new(
        attribution_touch_id: u64,
        tenant_id: u64,
        organization_id: u64,
        source_kind: AttributionSourceKind,
        created_at_ms: u64,
    ) -> Self {
        Self {
            attribution_touch_id,
            tenant_id,
            organization_id,
            source_kind,
            source_code: None,
            utm_source: None,
            utm_campaign: None,
            utm_medium: None,
            partner_id: None,
            referrer_user_id: None,
            invite_code_id: None,
            conversion_subject_id: None,
            converted_at_ms: None,
            created_at_ms,
            updated_at_ms: created_at_ms,
        }
    }

    pub fn with_source_code(mut self, source_code: Option<String>) -> Self {
        self.source_code = source_code;
        self
    }

    pub fn with_utm_source(mut self, utm_source: Option<String>) -> Self {
        self.utm_source = utm_source;
        self
    }

    pub fn with_utm_campaign(mut self, utm_campaign: Option<String>) -> Self {
        self.utm_campaign = utm_campaign;
        self
    }

    pub fn with_utm_medium(mut self, utm_medium: Option<String>) -> Self {
        self.utm_medium = utm_medium;
        self
    }

    pub fn with_partner_id(mut self, partner_id: Option<String>) -> Self {
        self.partner_id = partner_id;
        self
    }

    pub fn with_referrer_user_id(mut self, referrer_user_id: Option<u64>) -> Self {
        self.referrer_user_id = referrer_user_id;
        self
    }

    pub fn with_invite_code_id(mut self, invite_code_id: Option<u64>) -> Self {
        self.invite_code_id = invite_code_id;
        self
    }

    pub fn with_conversion_subject_id(mut self, conversion_subject_id: Option<String>) -> Self {
        self.conversion_subject_id = conversion_subject_id;
        self
    }

    pub fn with_converted_at_ms(mut self, converted_at_ms: Option<u64>) -> Self {
        self.converted_at_ms = converted_at_ms;
        self
    }

    pub fn with_updated_at_ms(mut self, updated_at_ms: u64) -> Self {
        self.updated_at_ms = updated_at_ms;
        self
    }
}
