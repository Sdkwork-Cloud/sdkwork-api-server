use serde::{Deserialize, Serialize};
use std::str::FromStr;
use utoipa::ToSchema;

pub fn normalize_coupon_code(value: &str) -> String {
    value.trim().to_ascii_uppercase()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum MarketingBenefitKind {
    PercentageOff,
    FixedAmountOff,
    GrantUnits,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum MarketingStackingPolicy {
    Exclusive,
    Stackable,
    BestOfGroup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum MarketingSubjectScope {
    User,
    Project,
    Workspace,
    Account,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum CouponTemplateStatus {
    Draft,
    Scheduled,
    Active,
    Archived,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema, Default,
)]
#[serde(rename_all = "snake_case")]
pub enum CouponTemplateApprovalState {
    Draft,
    InReview,
    #[default]
    Approved,
    Rejected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum CouponDistributionKind {
    SharedCode,
    UniqueCode,
    AutoClaim,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum MarketingCampaignStatus {
    Draft,
    Scheduled,
    Active,
    Paused,
    Ended,
    Archived,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema, Default,
)]
#[serde(rename_all = "snake_case")]
pub enum MarketingCampaignApprovalState {
    Draft,
    InReview,
    #[default]
    Approved,
    Rejected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum CouponTemplateLifecycleAction {
    Clone,
    SubmitForApproval,
    Approve,
    Reject,
    Publish,
    Schedule,
    Retire,
}

impl CouponTemplateLifecycleAction {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Clone => "clone",
            Self::SubmitForApproval => "submit_for_approval",
            Self::Approve => "approve",
            Self::Reject => "reject",
            Self::Publish => "publish",
            Self::Schedule => "schedule",
            Self::Retire => "retire",
        }
    }
}

impl FromStr for CouponTemplateLifecycleAction {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "clone" => Ok(Self::Clone),
            "submit_for_approval" => Ok(Self::SubmitForApproval),
            "approve" => Ok(Self::Approve),
            "reject" => Ok(Self::Reject),
            "publish" => Ok(Self::Publish),
            "schedule" => Ok(Self::Schedule),
            "retire" => Ok(Self::Retire),
            _ => Err(anyhow::anyhow!(
                "unsupported coupon template lifecycle action {value}"
            )),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum CouponTemplateLifecycleAuditOutcome {
    Applied,
    Rejected,
}

impl CouponTemplateLifecycleAuditOutcome {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Applied => "applied",
            Self::Rejected => "rejected",
        }
    }
}

impl FromStr for CouponTemplateLifecycleAuditOutcome {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "applied" => Ok(Self::Applied),
            "rejected" => Ok(Self::Rejected),
            _ => Err(anyhow::anyhow!(
                "unsupported coupon template lifecycle audit outcome {value}"
            )),
        }
    }
}

fn default_coupon_template_revision() -> u32 {
    1
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct CouponTemplateLifecycleAuditRecord {
    pub audit_id: String,
    pub coupon_template_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_coupon_template_id: Option<String>,
    pub action: CouponTemplateLifecycleAction,
    pub outcome: CouponTemplateLifecycleAuditOutcome,
    pub previous_status: CouponTemplateStatus,
    pub resulting_status: CouponTemplateStatus,
    #[serde(default)]
    pub previous_approval_state: CouponTemplateApprovalState,
    #[serde(default)]
    pub resulting_approval_state: CouponTemplateApprovalState,
    #[serde(default = "default_coupon_template_revision")]
    pub previous_revision: u32,
    #[serde(default = "default_coupon_template_revision")]
    pub resulting_revision: u32,
    pub operator_id: String,
    pub request_id: String,
    pub reason: String,
    #[serde(default)]
    pub decision_reasons: Vec<String>,
    pub requested_at_ms: u64,
}

impl CouponTemplateLifecycleAuditRecord {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        audit_id: impl Into<String>,
        coupon_template_id: impl Into<String>,
        action: CouponTemplateLifecycleAction,
        outcome: CouponTemplateLifecycleAuditOutcome,
        previous_status: CouponTemplateStatus,
        resulting_status: CouponTemplateStatus,
        operator_id: impl Into<String>,
        request_id: impl Into<String>,
        reason: impl Into<String>,
        requested_at_ms: u64,
    ) -> Self {
        Self {
            audit_id: audit_id.into(),
            coupon_template_id: coupon_template_id.into(),
            source_coupon_template_id: None,
            action,
            outcome,
            previous_status,
            resulting_status,
            previous_approval_state: CouponTemplateApprovalState::default(),
            resulting_approval_state: CouponTemplateApprovalState::default(),
            previous_revision: default_coupon_template_revision(),
            resulting_revision: default_coupon_template_revision(),
            operator_id: operator_id.into(),
            request_id: request_id.into(),
            reason: reason.into(),
            decision_reasons: Vec::new(),
            requested_at_ms,
        }
    }

    pub fn with_decision_reasons(mut self, decision_reasons: Vec<String>) -> Self {
        self.decision_reasons = decision_reasons;
        self
    }

    pub fn with_source_coupon_template_id(
        mut self,
        source_coupon_template_id: Option<String>,
    ) -> Self {
        self.source_coupon_template_id = source_coupon_template_id;
        self
    }

    pub fn with_approval_states(
        mut self,
        previous_approval_state: CouponTemplateApprovalState,
        resulting_approval_state: CouponTemplateApprovalState,
    ) -> Self {
        self.previous_approval_state = previous_approval_state;
        self.resulting_approval_state = resulting_approval_state;
        self
    }

    pub fn with_revisions(mut self, previous_revision: u32, resulting_revision: u32) -> Self {
        self.previous_revision = previous_revision;
        self.resulting_revision = resulting_revision;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum MarketingCampaignLifecycleAction {
    Clone,
    SubmitForApproval,
    Approve,
    Reject,
    Publish,
    Schedule,
    Retire,
}

impl MarketingCampaignLifecycleAction {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Clone => "clone",
            Self::SubmitForApproval => "submit_for_approval",
            Self::Approve => "approve",
            Self::Reject => "reject",
            Self::Publish => "publish",
            Self::Schedule => "schedule",
            Self::Retire => "retire",
        }
    }
}

impl FromStr for MarketingCampaignLifecycleAction {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "clone" => Ok(Self::Clone),
            "submit_for_approval" => Ok(Self::SubmitForApproval),
            "approve" => Ok(Self::Approve),
            "reject" => Ok(Self::Reject),
            "publish" => Ok(Self::Publish),
            "schedule" => Ok(Self::Schedule),
            "retire" => Ok(Self::Retire),
            _ => Err(anyhow::anyhow!(
                "unsupported marketing campaign lifecycle action {value}"
            )),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum MarketingCampaignLifecycleAuditOutcome {
    Applied,
    Rejected,
}

impl MarketingCampaignLifecycleAuditOutcome {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Applied => "applied",
            Self::Rejected => "rejected",
        }
    }
}

impl FromStr for MarketingCampaignLifecycleAuditOutcome {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "applied" => Ok(Self::Applied),
            "rejected" => Ok(Self::Rejected),
            _ => Err(anyhow::anyhow!(
                "unsupported marketing campaign lifecycle audit outcome {value}"
            )),
        }
    }
}

fn default_marketing_campaign_revision() -> u32 {
    1
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct MarketingCampaignLifecycleAuditRecord {
    pub audit_id: String,
    pub marketing_campaign_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_marketing_campaign_id: Option<String>,
    pub coupon_template_id: String,
    pub action: MarketingCampaignLifecycleAction,
    pub outcome: MarketingCampaignLifecycleAuditOutcome,
    pub previous_status: MarketingCampaignStatus,
    pub resulting_status: MarketingCampaignStatus,
    #[serde(default)]
    pub previous_approval_state: MarketingCampaignApprovalState,
    #[serde(default)]
    pub resulting_approval_state: MarketingCampaignApprovalState,
    #[serde(default = "default_marketing_campaign_revision")]
    pub previous_revision: u32,
    #[serde(default = "default_marketing_campaign_revision")]
    pub resulting_revision: u32,
    pub operator_id: String,
    pub request_id: String,
    pub reason: String,
    #[serde(default)]
    pub decision_reasons: Vec<String>,
    pub requested_at_ms: u64,
}

impl MarketingCampaignLifecycleAuditRecord {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        audit_id: impl Into<String>,
        marketing_campaign_id: impl Into<String>,
        coupon_template_id: impl Into<String>,
        action: MarketingCampaignLifecycleAction,
        outcome: MarketingCampaignLifecycleAuditOutcome,
        previous_status: MarketingCampaignStatus,
        resulting_status: MarketingCampaignStatus,
        operator_id: impl Into<String>,
        request_id: impl Into<String>,
        reason: impl Into<String>,
        requested_at_ms: u64,
    ) -> Self {
        Self {
            audit_id: audit_id.into(),
            marketing_campaign_id: marketing_campaign_id.into(),
            source_marketing_campaign_id: None,
            coupon_template_id: coupon_template_id.into(),
            action,
            outcome,
            previous_status,
            resulting_status,
            previous_approval_state: MarketingCampaignApprovalState::default(),
            resulting_approval_state: MarketingCampaignApprovalState::default(),
            previous_revision: default_marketing_campaign_revision(),
            resulting_revision: default_marketing_campaign_revision(),
            operator_id: operator_id.into(),
            request_id: request_id.into(),
            reason: reason.into(),
            decision_reasons: Vec::new(),
            requested_at_ms,
        }
    }

    pub fn with_decision_reasons(mut self, decision_reasons: Vec<String>) -> Self {
        self.decision_reasons = decision_reasons;
        self
    }

    pub fn with_source_marketing_campaign_id(
        mut self,
        source_marketing_campaign_id: Option<String>,
    ) -> Self {
        self.source_marketing_campaign_id = source_marketing_campaign_id;
        self
    }

    pub fn with_approval_states(
        mut self,
        previous_approval_state: MarketingCampaignApprovalState,
        resulting_approval_state: MarketingCampaignApprovalState,
    ) -> Self {
        self.previous_approval_state = previous_approval_state;
        self.resulting_approval_state = resulting_approval_state;
        self
    }

    pub fn with_revisions(mut self, previous_revision: u32, resulting_revision: u32) -> Self {
        self.previous_revision = previous_revision;
        self.resulting_revision = resulting_revision;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum CampaignBudgetStatus {
    Draft,
    Active,
    Exhausted,
    Closed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum CampaignBudgetLifecycleAction {
    Activate,
    Close,
}

impl CampaignBudgetLifecycleAction {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Activate => "activate",
            Self::Close => "close",
        }
    }
}

impl FromStr for CampaignBudgetLifecycleAction {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "activate" => Ok(Self::Activate),
            "close" => Ok(Self::Close),
            _ => Err(anyhow::anyhow!(
                "unsupported campaign budget lifecycle action {value}"
            )),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum CampaignBudgetLifecycleAuditOutcome {
    Applied,
    Rejected,
}

impl CampaignBudgetLifecycleAuditOutcome {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Applied => "applied",
            Self::Rejected => "rejected",
        }
    }
}

impl FromStr for CampaignBudgetLifecycleAuditOutcome {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "applied" => Ok(Self::Applied),
            "rejected" => Ok(Self::Rejected),
            _ => Err(anyhow::anyhow!(
                "unsupported campaign budget lifecycle audit outcome {value}"
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct CampaignBudgetLifecycleAuditRecord {
    pub audit_id: String,
    pub campaign_budget_id: String,
    pub marketing_campaign_id: String,
    pub action: CampaignBudgetLifecycleAction,
    pub outcome: CampaignBudgetLifecycleAuditOutcome,
    pub previous_status: CampaignBudgetStatus,
    pub resulting_status: CampaignBudgetStatus,
    pub operator_id: String,
    pub request_id: String,
    pub reason: String,
    #[serde(default)]
    pub decision_reasons: Vec<String>,
    pub requested_at_ms: u64,
}

impl CampaignBudgetLifecycleAuditRecord {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        audit_id: impl Into<String>,
        campaign_budget_id: impl Into<String>,
        marketing_campaign_id: impl Into<String>,
        action: CampaignBudgetLifecycleAction,
        outcome: CampaignBudgetLifecycleAuditOutcome,
        previous_status: CampaignBudgetStatus,
        resulting_status: CampaignBudgetStatus,
        operator_id: impl Into<String>,
        request_id: impl Into<String>,
        reason: impl Into<String>,
        requested_at_ms: u64,
    ) -> Self {
        Self {
            audit_id: audit_id.into(),
            campaign_budget_id: campaign_budget_id.into(),
            marketing_campaign_id: marketing_campaign_id.into(),
            action,
            outcome,
            previous_status,
            resulting_status,
            operator_id: operator_id.into(),
            request_id: request_id.into(),
            reason: reason.into(),
            decision_reasons: Vec::new(),
            requested_at_ms,
        }
    }

    pub fn with_decision_reasons(mut self, decision_reasons: Vec<String>) -> Self {
        self.decision_reasons = decision_reasons;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum CouponCodeStatus {
    Available,
    Reserved,
    Redeemed,
    Expired,
    Disabled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum CouponCodeLifecycleAction {
    Disable,
    Restore,
}

impl CouponCodeLifecycleAction {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Disable => "disable",
            Self::Restore => "restore",
        }
    }
}

impl FromStr for CouponCodeLifecycleAction {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "disable" => Ok(Self::Disable),
            "restore" => Ok(Self::Restore),
            _ => Err(anyhow::anyhow!(
                "unsupported coupon code lifecycle action {value}"
            )),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum CouponCodeLifecycleAuditOutcome {
    Applied,
    Rejected,
}

impl CouponCodeLifecycleAuditOutcome {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Applied => "applied",
            Self::Rejected => "rejected",
        }
    }
}

impl FromStr for CouponCodeLifecycleAuditOutcome {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "applied" => Ok(Self::Applied),
            "rejected" => Ok(Self::Rejected),
            _ => Err(anyhow::anyhow!(
                "unsupported coupon code lifecycle audit outcome {value}"
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct CouponCodeLifecycleAuditRecord {
    pub audit_id: String,
    pub coupon_code_id: String,
    pub coupon_template_id: String,
    pub action: CouponCodeLifecycleAction,
    pub outcome: CouponCodeLifecycleAuditOutcome,
    pub previous_status: CouponCodeStatus,
    pub resulting_status: CouponCodeStatus,
    pub operator_id: String,
    pub request_id: String,
    pub reason: String,
    #[serde(default)]
    pub decision_reasons: Vec<String>,
    pub requested_at_ms: u64,
}

impl CouponCodeLifecycleAuditRecord {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        audit_id: impl Into<String>,
        coupon_code_id: impl Into<String>,
        coupon_template_id: impl Into<String>,
        action: CouponCodeLifecycleAction,
        outcome: CouponCodeLifecycleAuditOutcome,
        previous_status: CouponCodeStatus,
        resulting_status: CouponCodeStatus,
        operator_id: impl Into<String>,
        request_id: impl Into<String>,
        reason: impl Into<String>,
        requested_at_ms: u64,
    ) -> Self {
        Self {
            audit_id: audit_id.into(),
            coupon_code_id: coupon_code_id.into(),
            coupon_template_id: coupon_template_id.into(),
            action,
            outcome,
            previous_status,
            resulting_status,
            operator_id: operator_id.into(),
            request_id: request_id.into(),
            reason: reason.into(),
            decision_reasons: Vec::new(),
            requested_at_ms,
        }
    }

    pub fn with_decision_reasons(mut self, decision_reasons: Vec<String>) -> Self {
        self.decision_reasons = decision_reasons;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum CouponReservationStatus {
    Reserved,
    Released,
    Confirmed,
    Expired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum CouponRedemptionStatus {
    Pending,
    Redeemed,
    PartiallyRolledBack,
    RolledBack,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum CouponRollbackType {
    Cancel,
    Refund,
    PartialRefund,
    Manual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum CouponRollbackStatus {
    Pending,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum MarketingOutboxEventStatus {
    Pending,
    Delivered,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct CouponBenefitSpec {
    pub benefit_kind: MarketingBenefitKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub discount_percent: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub discount_amount_minor: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub grant_units: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub currency_code: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_discount_minor: Option<u64>,
}

impl CouponBenefitSpec {
    pub fn new(benefit_kind: MarketingBenefitKind) -> Self {
        Self {
            benefit_kind,
            discount_percent: None,
            discount_amount_minor: None,
            grant_units: None,
            currency_code: None,
            max_discount_minor: None,
        }
    }

    pub fn with_discount_percent(mut self, discount_percent: Option<u8>) -> Self {
        self.discount_percent = discount_percent.map(|value| value.min(100));
        self
    }

    pub fn with_discount_amount_minor(mut self, discount_amount_minor: Option<u64>) -> Self {
        self.discount_amount_minor = discount_amount_minor;
        self
    }

    pub fn with_grant_units(mut self, grant_units: Option<u64>) -> Self {
        self.grant_units = grant_units;
        self
    }

    pub fn with_currency_code(mut self, currency_code: Option<String>) -> Self {
        self.currency_code = currency_code;
        self
    }

    pub fn with_max_discount_minor(mut self, max_discount_minor: Option<u64>) -> Self {
        self.max_discount_minor = max_discount_minor;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct CouponRestrictionSpec {
    pub subject_scope: MarketingSubjectScope,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_order_amount_minor: Option<u64>,
    #[serde(default)]
    pub first_order_only: bool,
    #[serde(default)]
    pub new_customer_only: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exclusive_group: Option<String>,
    pub stacking_policy: MarketingStackingPolicy,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_redemptions_per_subject: Option<u64>,
    #[serde(default)]
    pub eligible_target_kinds: Vec<String>,
}

impl CouponRestrictionSpec {
    pub fn new(subject_scope: MarketingSubjectScope) -> Self {
        Self {
            subject_scope,
            min_order_amount_minor: None,
            first_order_only: false,
            new_customer_only: false,
            exclusive_group: None,
            stacking_policy: MarketingStackingPolicy::Exclusive,
            max_redemptions_per_subject: None,
            eligible_target_kinds: Vec::new(),
        }
    }

    pub fn with_min_order_amount_minor(mut self, min_order_amount_minor: Option<u64>) -> Self {
        self.min_order_amount_minor = min_order_amount_minor;
        self
    }

    pub fn with_first_order_only(mut self, first_order_only: bool) -> Self {
        self.first_order_only = first_order_only;
        self
    }

    pub fn with_new_customer_only(mut self, new_customer_only: bool) -> Self {
        self.new_customer_only = new_customer_only;
        self
    }

    pub fn with_exclusive_group(mut self, exclusive_group: Option<String>) -> Self {
        self.exclusive_group = exclusive_group;
        self
    }

    pub fn with_stacking_policy(mut self, stacking_policy: MarketingStackingPolicy) -> Self {
        self.stacking_policy = stacking_policy;
        self
    }

    pub fn with_max_redemptions_per_subject(
        mut self,
        max_redemptions_per_subject: Option<u64>,
    ) -> Self {
        self.max_redemptions_per_subject = max_redemptions_per_subject;
        self
    }

    pub fn with_eligible_target_kinds(mut self, eligible_target_kinds: Vec<String>) -> Self {
        self.eligible_target_kinds = eligible_target_kinds;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct CouponTemplateRecord {
    pub coupon_template_id: String,
    pub template_key: String,
    pub display_name: String,
    pub status: CouponTemplateStatus,
    #[serde(default)]
    pub approval_state: CouponTemplateApprovalState,
    #[serde(default = "default_coupon_template_revision")]
    pub revision: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_coupon_template_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_coupon_template_id: Option<String>,
    pub distribution_kind: CouponDistributionKind,
    pub benefit: CouponBenefitSpec,
    pub restriction: CouponRestrictionSpec,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub activation_at_ms: Option<u64>,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

impl CouponTemplateRecord {
    pub fn new(
        coupon_template_id: impl Into<String>,
        template_key: impl Into<String>,
        benefit_kind: MarketingBenefitKind,
    ) -> Self {
        Self {
            coupon_template_id: coupon_template_id.into(),
            template_key: template_key.into(),
            display_name: String::new(),
            status: CouponTemplateStatus::Draft,
            approval_state: CouponTemplateApprovalState::default(),
            revision: default_coupon_template_revision(),
            root_coupon_template_id: None,
            parent_coupon_template_id: None,
            distribution_kind: CouponDistributionKind::SharedCode,
            benefit: CouponBenefitSpec::new(benefit_kind),
            restriction: CouponRestrictionSpec::new(MarketingSubjectScope::Project),
            activation_at_ms: None,
            created_at_ms: 0,
            updated_at_ms: 0,
        }
    }

    pub fn with_display_name(mut self, display_name: impl Into<String>) -> Self {
        self.display_name = display_name.into();
        self
    }

    pub fn with_status(mut self, status: CouponTemplateStatus) -> Self {
        self.status = status;
        self
    }

    pub fn with_approval_state(mut self, approval_state: CouponTemplateApprovalState) -> Self {
        self.approval_state = approval_state;
        self
    }

    pub fn with_revision(mut self, revision: u32) -> Self {
        self.revision = revision.max(default_coupon_template_revision());
        self
    }

    pub fn with_root_coupon_template_id(mut self, root_coupon_template_id: Option<String>) -> Self {
        self.root_coupon_template_id = root_coupon_template_id;
        self
    }

    pub fn with_parent_coupon_template_id(
        mut self,
        parent_coupon_template_id: Option<String>,
    ) -> Self {
        self.parent_coupon_template_id = parent_coupon_template_id;
        self
    }

    pub fn with_distribution_kind(mut self, distribution_kind: CouponDistributionKind) -> Self {
        self.distribution_kind = distribution_kind;
        self
    }

    pub fn with_benefit(mut self, benefit: CouponBenefitSpec) -> Self {
        self.benefit = benefit;
        self
    }

    pub fn with_restriction(mut self, restriction: CouponRestrictionSpec) -> Self {
        self.restriction = restriction;
        self
    }

    pub fn with_activation_at_ms(mut self, activation_at_ms: Option<u64>) -> Self {
        self.activation_at_ms = activation_at_ms;
        self
    }

    pub fn with_created_at_ms(mut self, created_at_ms: u64) -> Self {
        self.created_at_ms = created_at_ms;
        self
    }

    pub fn with_updated_at_ms(mut self, updated_at_ms: u64) -> Self {
        self.updated_at_ms = updated_at_ms;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct MarketingCampaignRecord {
    pub marketing_campaign_id: String,
    pub coupon_template_id: String,
    pub display_name: String,
    pub status: MarketingCampaignStatus,
    #[serde(default)]
    pub approval_state: MarketingCampaignApprovalState,
    #[serde(default = "default_marketing_campaign_revision")]
    pub revision: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_marketing_campaign_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_marketing_campaign_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_at_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_at_ms: Option<u64>,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

impl MarketingCampaignRecord {
    pub fn new(
        marketing_campaign_id: impl Into<String>,
        coupon_template_id: impl Into<String>,
    ) -> Self {
        Self {
            marketing_campaign_id: marketing_campaign_id.into(),
            coupon_template_id: coupon_template_id.into(),
            display_name: String::new(),
            status: MarketingCampaignStatus::Draft,
            approval_state: MarketingCampaignApprovalState::default(),
            revision: default_marketing_campaign_revision(),
            root_marketing_campaign_id: None,
            parent_marketing_campaign_id: None,
            start_at_ms: None,
            end_at_ms: None,
            created_at_ms: 0,
            updated_at_ms: 0,
        }
    }

    pub fn with_display_name(mut self, display_name: impl Into<String>) -> Self {
        self.display_name = display_name.into();
        self
    }

    pub fn with_status(mut self, status: MarketingCampaignStatus) -> Self {
        self.status = status;
        self
    }

    pub fn with_approval_state(mut self, approval_state: MarketingCampaignApprovalState) -> Self {
        self.approval_state = approval_state;
        self
    }

    pub fn with_revision(mut self, revision: u32) -> Self {
        self.revision = revision.max(default_marketing_campaign_revision());
        self
    }

    pub fn with_root_marketing_campaign_id(
        mut self,
        root_marketing_campaign_id: Option<String>,
    ) -> Self {
        self.root_marketing_campaign_id = root_marketing_campaign_id;
        self
    }

    pub fn with_parent_marketing_campaign_id(
        mut self,
        parent_marketing_campaign_id: Option<String>,
    ) -> Self {
        self.parent_marketing_campaign_id = parent_marketing_campaign_id;
        self
    }

    pub fn with_start_at_ms(mut self, start_at_ms: Option<u64>) -> Self {
        self.start_at_ms = start_at_ms;
        self
    }

    pub fn with_end_at_ms(mut self, end_at_ms: Option<u64>) -> Self {
        self.end_at_ms = end_at_ms;
        self
    }

    pub fn with_created_at_ms(mut self, created_at_ms: u64) -> Self {
        self.created_at_ms = created_at_ms;
        self
    }

    pub fn with_updated_at_ms(mut self, updated_at_ms: u64) -> Self {
        self.updated_at_ms = updated_at_ms;
        self
    }

    pub fn is_effective_at(&self, now_ms: u64) -> bool {
        if self.status != MarketingCampaignStatus::Active {
            return false;
        }

        let after_start = self.start_at_ms.map_or(true, |value| now_ms >= value);
        let before_end = self.end_at_ms.map_or(true, |value| now_ms <= value);
        after_start && before_end
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct CampaignBudgetRecord {
    pub campaign_budget_id: String,
    pub marketing_campaign_id: String,
    pub status: CampaignBudgetStatus,
    pub total_budget_minor: u64,
    pub reserved_budget_minor: u64,
    pub consumed_budget_minor: u64,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

impl CampaignBudgetRecord {
    pub fn new(
        campaign_budget_id: impl Into<String>,
        marketing_campaign_id: impl Into<String>,
    ) -> Self {
        Self {
            campaign_budget_id: campaign_budget_id.into(),
            marketing_campaign_id: marketing_campaign_id.into(),
            status: CampaignBudgetStatus::Draft,
            total_budget_minor: 0,
            reserved_budget_minor: 0,
            consumed_budget_minor: 0,
            created_at_ms: 0,
            updated_at_ms: 0,
        }
    }

    pub fn with_status(mut self, status: CampaignBudgetStatus) -> Self {
        self.status = status;
        self
    }

    pub fn with_total_budget_minor(mut self, total_budget_minor: u64) -> Self {
        self.total_budget_minor = total_budget_minor;
        self
    }

    pub fn with_reserved_budget_minor(mut self, reserved_budget_minor: u64) -> Self {
        self.reserved_budget_minor = reserved_budget_minor;
        self
    }

    pub fn with_consumed_budget_minor(mut self, consumed_budget_minor: u64) -> Self {
        self.consumed_budget_minor = consumed_budget_minor;
        self
    }

    pub fn with_created_at_ms(mut self, created_at_ms: u64) -> Self {
        self.created_at_ms = created_at_ms;
        self
    }

    pub fn with_updated_at_ms(mut self, updated_at_ms: u64) -> Self {
        self.updated_at_ms = updated_at_ms;
        self
    }

    pub fn available_budget_minor(&self) -> u64 {
        self.total_budget_minor
            .saturating_sub(self.reserved_budget_minor)
            .saturating_sub(self.consumed_budget_minor)
    }

    pub fn can_reserve(&self, amount_minor: u64) -> bool {
        self.status == CampaignBudgetStatus::Active && amount_minor <= self.available_budget_minor()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct CouponCodeRecord {
    pub coupon_code_id: String,
    pub coupon_template_id: String,
    pub code_value: String,
    pub status: CouponCodeStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub claimed_subject_scope: Option<MarketingSubjectScope>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub claimed_subject_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at_ms: Option<u64>,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

impl CouponCodeRecord {
    pub fn new(
        coupon_code_id: impl Into<String>,
        coupon_template_id: impl Into<String>,
        code_value: impl Into<String>,
    ) -> Self {
        Self {
            coupon_code_id: coupon_code_id.into(),
            coupon_template_id: coupon_template_id.into(),
            code_value: code_value.into(),
            status: CouponCodeStatus::Available,
            claimed_subject_scope: None,
            claimed_subject_id: None,
            expires_at_ms: None,
            created_at_ms: 0,
            updated_at_ms: 0,
        }
    }

    pub fn with_status(mut self, status: CouponCodeStatus) -> Self {
        self.status = status;
        self
    }

    pub fn with_claimed_subject(
        mut self,
        claimed_subject_scope: Option<MarketingSubjectScope>,
        claimed_subject_id: Option<String>,
    ) -> Self {
        self.claimed_subject_scope = claimed_subject_scope;
        self.claimed_subject_id = claimed_subject_id;
        self
    }

    pub fn with_expires_at_ms(mut self, expires_at_ms: Option<u64>) -> Self {
        self.expires_at_ms = expires_at_ms;
        self
    }

    pub fn with_created_at_ms(mut self, created_at_ms: u64) -> Self {
        self.created_at_ms = created_at_ms;
        self
    }

    pub fn with_updated_at_ms(mut self, updated_at_ms: u64) -> Self {
        self.updated_at_ms = updated_at_ms;
        self
    }

    pub fn is_redeemable_at(&self, now_ms: u64) -> bool {
        self.status == CouponCodeStatus::Available
            && self.expires_at_ms.map_or(true, |value| now_ms <= value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct CouponReservationRecord {
    pub coupon_reservation_id: String,
    pub coupon_code_id: String,
    pub subject_scope: MarketingSubjectScope,
    pub subject_id: String,
    pub reservation_status: CouponReservationStatus,
    pub budget_reserved_minor: u64,
    pub expires_at_ms: u64,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

impl CouponReservationRecord {
    pub fn new(
        coupon_reservation_id: impl Into<String>,
        coupon_code_id: impl Into<String>,
        subject_scope: MarketingSubjectScope,
        subject_id: impl Into<String>,
        expires_at_ms: u64,
    ) -> Self {
        Self {
            coupon_reservation_id: coupon_reservation_id.into(),
            coupon_code_id: coupon_code_id.into(),
            subject_scope,
            subject_id: subject_id.into(),
            reservation_status: CouponReservationStatus::Reserved,
            budget_reserved_minor: 0,
            expires_at_ms,
            created_at_ms: 0,
            updated_at_ms: 0,
        }
    }

    pub fn with_status(mut self, reservation_status: CouponReservationStatus) -> Self {
        self.reservation_status = reservation_status;
        self
    }

    pub fn with_budget_reserved_minor(mut self, budget_reserved_minor: u64) -> Self {
        self.budget_reserved_minor = budget_reserved_minor;
        self
    }

    pub fn with_expires_at_ms(mut self, expires_at_ms: u64) -> Self {
        self.expires_at_ms = expires_at_ms;
        self
    }

    pub fn with_created_at_ms(mut self, created_at_ms: u64) -> Self {
        self.created_at_ms = created_at_ms;
        self
    }

    pub fn with_updated_at_ms(mut self, updated_at_ms: u64) -> Self {
        self.updated_at_ms = updated_at_ms;
        self
    }

    pub fn is_active_at(&self, now_ms: u64) -> bool {
        self.reservation_status == CouponReservationStatus::Reserved && now_ms <= self.expires_at_ms
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct CouponRedemptionRecord {
    pub coupon_redemption_id: String,
    pub coupon_reservation_id: String,
    pub coupon_code_id: String,
    pub coupon_template_id: String,
    pub redemption_status: CouponRedemptionStatus,
    #[serde(default)]
    pub budget_consumed_minor: u64,
    pub subsidy_amount_minor: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub order_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payment_event_id: Option<String>,
    pub redeemed_at_ms: u64,
    pub updated_at_ms: u64,
}

impl CouponRedemptionRecord {
    pub fn new(
        coupon_redemption_id: impl Into<String>,
        coupon_reservation_id: impl Into<String>,
        coupon_code_id: impl Into<String>,
        coupon_template_id: impl Into<String>,
        redeemed_at_ms: u64,
    ) -> Self {
        Self {
            coupon_redemption_id: coupon_redemption_id.into(),
            coupon_reservation_id: coupon_reservation_id.into(),
            coupon_code_id: coupon_code_id.into(),
            coupon_template_id: coupon_template_id.into(),
            redemption_status: CouponRedemptionStatus::Pending,
            budget_consumed_minor: 0,
            subsidy_amount_minor: 0,
            order_id: None,
            payment_event_id: None,
            redeemed_at_ms,
            updated_at_ms: redeemed_at_ms,
        }
    }

    pub fn with_status(mut self, redemption_status: CouponRedemptionStatus) -> Self {
        self.redemption_status = redemption_status;
        self
    }

    pub fn with_subsidy_amount_minor(mut self, subsidy_amount_minor: u64) -> Self {
        self.subsidy_amount_minor = subsidy_amount_minor;
        self
    }

    pub fn with_budget_consumed_minor(mut self, budget_consumed_minor: u64) -> Self {
        self.budget_consumed_minor = budget_consumed_minor;
        self
    }

    pub fn with_order_id(mut self, order_id: Option<String>) -> Self {
        self.order_id = order_id;
        self
    }

    pub fn with_payment_event_id(mut self, payment_event_id: Option<String>) -> Self {
        self.payment_event_id = payment_event_id;
        self
    }

    pub fn with_updated_at_ms(mut self, updated_at_ms: u64) -> Self {
        self.updated_at_ms = updated_at_ms;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct CouponRollbackRecord {
    pub coupon_rollback_id: String,
    pub coupon_redemption_id: String,
    pub rollback_type: CouponRollbackType,
    pub rollback_status: CouponRollbackStatus,
    pub restored_budget_minor: u64,
    pub restored_inventory_count: u64,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

impl CouponRollbackRecord {
    pub fn new(
        coupon_rollback_id: impl Into<String>,
        coupon_redemption_id: impl Into<String>,
        rollback_type: CouponRollbackType,
        created_at_ms: u64,
    ) -> Self {
        Self {
            coupon_rollback_id: coupon_rollback_id.into(),
            coupon_redemption_id: coupon_redemption_id.into(),
            rollback_type,
            rollback_status: CouponRollbackStatus::Pending,
            restored_budget_minor: 0,
            restored_inventory_count: 0,
            created_at_ms,
            updated_at_ms: created_at_ms,
        }
    }

    pub fn with_status(mut self, rollback_status: CouponRollbackStatus) -> Self {
        self.rollback_status = rollback_status;
        self
    }

    pub fn with_restored_budget_minor(mut self, restored_budget_minor: u64) -> Self {
        self.restored_budget_minor = restored_budget_minor;
        self
    }

    pub fn with_restored_inventory_count(mut self, restored_inventory_count: u64) -> Self {
        self.restored_inventory_count = restored_inventory_count;
        self
    }

    pub fn with_updated_at_ms(mut self, updated_at_ms: u64) -> Self {
        self.updated_at_ms = updated_at_ms;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct MarketingOutboxEventRecord {
    pub marketing_outbox_event_id: String,
    pub aggregate_type: String,
    pub aggregate_id: String,
    pub event_type: String,
    pub status: MarketingOutboxEventStatus,
    pub payload_json: String,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

impl MarketingOutboxEventRecord {
    pub fn new(
        marketing_outbox_event_id: impl Into<String>,
        aggregate_type: impl Into<String>,
        aggregate_id: impl Into<String>,
        event_type: impl Into<String>,
        payload_json: impl Into<String>,
        created_at_ms: u64,
    ) -> Self {
        Self {
            marketing_outbox_event_id: marketing_outbox_event_id.into(),
            aggregate_type: aggregate_type.into(),
            aggregate_id: aggregate_id.into(),
            event_type: event_type.into(),
            status: MarketingOutboxEventStatus::Pending,
            payload_json: payload_json.into(),
            created_at_ms,
            updated_at_ms: created_at_ms,
        }
    }

    pub fn with_status(mut self, status: MarketingOutboxEventStatus) -> Self {
        self.status = status;
        self
    }

    pub fn with_updated_at_ms(mut self, updated_at_ms: u64) -> Self {
        self.updated_at_ms = updated_at_ms;
        self
    }
}
