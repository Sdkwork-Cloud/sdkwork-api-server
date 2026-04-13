use anyhow::{anyhow, ensure, Result};
use sdkwork_api_domain_marketing::{
    CouponBenefitKind, CouponBenefitRuleRecord, CouponClaimRecord, CouponClaimStatus,
    CouponCodeBatchRecord, CouponCodeBatchStatus, CouponCodeKind, CouponCodeRecord,
    CouponCodeStatus, CouponRedemptionRecord, CouponRedemptionStatus, CouponTemplateRecord,
    CouponTemplateStatus, MarketingCampaignRecord, MarketingCampaignStatus,
};
use sdkwork_api_storage_core::AdminStore;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IssueCouponCodeInput {
    pub coupon_code_id: u64,
    pub coupon_code_batch_id: u64,
    pub code_lookup_hash: String,
    pub code_kind: CouponCodeKind,
    pub now_ms: u64,
    pub display_code_prefix: Option<String>,
    pub display_code_suffix: Option<String>,
    pub expires_at_ms: Option<u64>,
}

impl IssueCouponCodeInput {
    pub fn new(
        coupon_code_id: u64,
        coupon_code_batch_id: u64,
        code_lookup_hash: impl Into<String>,
        code_kind: CouponCodeKind,
        now_ms: u64,
    ) -> Self {
        Self {
            coupon_code_id,
            coupon_code_batch_id,
            code_lookup_hash: code_lookup_hash.into(),
            code_kind,
            now_ms,
            display_code_prefix: None,
            display_code_suffix: None,
            expires_at_ms: None,
        }
    }

    pub fn with_display_code_prefix(mut self, display_code_prefix: Option<String>) -> Self {
        self.display_code_prefix = display_code_prefix;
        self
    }

    pub fn with_display_code_suffix(mut self, display_code_suffix: Option<String>) -> Self {
        self.display_code_suffix = display_code_suffix;
        self
    }

    pub fn with_expires_at_ms(mut self, expires_at_ms: Option<u64>) -> Self {
        self.expires_at_ms = expires_at_ms;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaimCouponCodeInput {
    pub claim_id: u64,
    pub subject_type: String,
    pub subject_id: String,
    pub code_lookup_hash: String,
    pub now_ms: u64,
    pub account_id: Option<u64>,
    pub project_id: Option<String>,
}

impl ClaimCouponCodeInput {
    pub fn new(
        claim_id: u64,
        subject_type: impl Into<String>,
        subject_id: impl Into<String>,
        code_lookup_hash: impl Into<String>,
        now_ms: u64,
    ) -> Self {
        Self {
            claim_id,
            subject_type: subject_type.into(),
            subject_id: subject_id.into(),
            code_lookup_hash: code_lookup_hash.into(),
            now_ms,
            account_id: None,
            project_id: None,
        }
    }

    pub fn with_account_id(mut self, account_id: Option<u64>) -> Self {
        self.account_id = account_id;
        self
    }

    pub fn with_project_id(mut self, project_id: Option<String>) -> Self {
        self.project_id = project_id;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VoidCouponCodeInput {
    pub coupon_code_id: u64,
    pub now_ms: u64,
}

impl VoidCouponCodeInput {
    pub fn new(coupon_code_id: u64, now_ms: u64) -> Self {
        Self {
            coupon_code_id,
            now_ms,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpireDueCouponCodesInput {
    pub now_ms: u64,
}

impl ExpireDueCouponCodesInput {
    pub fn new(now_ms: u64) -> Self {
        Self { now_ms }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpireDueCouponCodesResult {
    pub expired_code_ids: Vec<u64>,
    pub expired_claim_ids: Vec<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidateCouponForQuoteInput {
    pub subject_type: String,
    pub subject_id: String,
    pub code_lookup_hash: String,
    pub now_ms: u64,
    pub target_order_kind: Option<String>,
    pub target_product_id: Option<String>,
    pub reservation_idempotency_key: Option<String>,
}

impl ValidateCouponForQuoteInput {
    pub fn new(
        subject_type: impl Into<String>,
        subject_id: impl Into<String>,
        code_lookup_hash: impl Into<String>,
        now_ms: u64,
    ) -> Self {
        Self {
            subject_type: subject_type.into(),
            subject_id: subject_id.into(),
            code_lookup_hash: code_lookup_hash.into(),
            now_ms,
            target_order_kind: None,
            target_product_id: None,
            reservation_idempotency_key: None,
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

    pub fn with_reservation_idempotency_key(
        mut self,
        reservation_idempotency_key: Option<String>,
    ) -> Self {
        self.reservation_idempotency_key = reservation_idempotency_key;
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CouponQuoteValidation {
    pub coupon_code_id: u64,
    pub coupon_template_id: u64,
    pub coupon_benefit_rule_id: u64,
    pub benefit_kind: CouponBenefitKind,
    pub percentage_off: Option<f64>,
    pub fixed_discount_amount: Option<f64>,
    pub maximum_subsidy_amount: Option<f64>,
    pub currency_code: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedeemCouponCodeInput {
    pub redemption_id: u64,
    pub subject_type: String,
    pub subject_id: String,
    pub code_lookup_hash: String,
    pub idempotency_key: String,
    pub now_ms: u64,
    pub account_id: Option<u64>,
    pub project_id: Option<String>,
    pub order_id: Option<String>,
    pub payment_order_id: Option<String>,
}

impl RedeemCouponCodeInput {
    pub fn new(
        redemption_id: u64,
        subject_type: impl Into<String>,
        subject_id: impl Into<String>,
        code_lookup_hash: impl Into<String>,
        idempotency_key: impl Into<String>,
        now_ms: u64,
    ) -> Self {
        Self {
            redemption_id,
            subject_type: subject_type.into(),
            subject_id: subject_id.into(),
            code_lookup_hash: code_lookup_hash.into(),
            idempotency_key: idempotency_key.into(),
            now_ms,
            account_id: None,
            project_id: None,
            order_id: None,
            payment_order_id: None,
        }
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReserveCouponRedemptionInput {
    pub redemption_id: u64,
    pub subject_type: String,
    pub subject_id: String,
    pub code_lookup_hash: String,
    pub idempotency_key: String,
    pub now_ms: u64,
    pub account_id: Option<u64>,
    pub project_id: Option<String>,
    pub order_id: Option<String>,
}

impl ReserveCouponRedemptionInput {
    pub fn new(
        redemption_id: u64,
        subject_type: impl Into<String>,
        subject_id: impl Into<String>,
        code_lookup_hash: impl Into<String>,
        idempotency_key: impl Into<String>,
        now_ms: u64,
    ) -> Self {
        Self {
            redemption_id,
            subject_type: subject_type.into(),
            subject_id: subject_id.into(),
            code_lookup_hash: code_lookup_hash.into(),
            idempotency_key: idempotency_key.into(),
            now_ms,
            account_id: None,
            project_id: None,
            order_id: None,
        }
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseCouponRedemptionReservationInput {
    pub idempotency_key: String,
    pub status: CouponRedemptionStatus,
    pub now_ms: u64,
}

impl ReleaseCouponRedemptionReservationInput {
    pub fn new(
        idempotency_key: impl Into<String>,
        status: CouponRedemptionStatus,
        now_ms: u64,
    ) -> Self {
        Self {
            idempotency_key: idempotency_key.into(),
            status,
            now_ms,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ListCouponRedemptionsInput {
    pub subject_type: Option<String>,
    pub subject_id: Option<String>,
    pub project_id: Option<String>,
    pub order_id: Option<String>,
    pub payment_order_id: Option<String>,
    pub coupon_template_id: Option<u64>,
    pub coupon_code_id: Option<u64>,
    pub marketing_campaign_id: Option<u64>,
    pub status: Option<CouponRedemptionStatus>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ListCouponCodesInput {
    pub subject_type: Option<String>,
    pub subject_id: Option<String>,
    pub coupon_template_id: Option<u64>,
    pub coupon_code_batch_id: Option<u64>,
    pub marketing_campaign_id: Option<u64>,
    pub status: Option<CouponCodeStatus>,
}

impl ListCouponCodesInput {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_subject(
        mut self,
        subject_type: impl Into<String>,
        subject_id: impl Into<String>,
    ) -> Self {
        self.subject_type = Some(subject_type.into());
        self.subject_id = Some(subject_id.into());
        self
    }

    pub fn with_coupon_template_id(mut self, coupon_template_id: Option<u64>) -> Self {
        self.coupon_template_id = coupon_template_id;
        self
    }

    pub fn with_coupon_code_batch_id(mut self, coupon_code_batch_id: Option<u64>) -> Self {
        self.coupon_code_batch_id = coupon_code_batch_id;
        self
    }

    pub fn with_marketing_campaign_id(mut self, marketing_campaign_id: Option<u64>) -> Self {
        self.marketing_campaign_id = marketing_campaign_id;
        self
    }

    pub fn with_status(mut self, status: Option<CouponCodeStatus>) -> Self {
        self.status = status;
        self
    }
}

impl ListCouponRedemptionsInput {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_subject(
        mut self,
        subject_type: impl Into<String>,
        subject_id: impl Into<String>,
    ) -> Self {
        self.subject_type = Some(subject_type.into());
        self.subject_id = Some(subject_id.into());
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

    pub fn with_coupon_template_id(mut self, coupon_template_id: Option<u64>) -> Self {
        self.coupon_template_id = coupon_template_id;
        self
    }

    pub fn with_coupon_code_id(mut self, coupon_code_id: Option<u64>) -> Self {
        self.coupon_code_id = coupon_code_id;
        self
    }

    pub fn with_marketing_campaign_id(mut self, marketing_campaign_id: Option<u64>) -> Self {
        self.marketing_campaign_id = marketing_campaign_id;
        self
    }

    pub fn with_status(mut self, status: Option<CouponRedemptionStatus>) -> Self {
        self.status = status;
        self
    }
}

pub async fn list_coupon_codes<S>(
    store: &S,
    input: &ListCouponCodesInput,
) -> Result<Vec<CouponCodeRecord>>
where
    S: AdminStore + ?Sized,
{
    let subject_filter = match (input.subject_type.as_deref(), input.subject_id.as_deref()) {
        (None, None) => None,
        (Some(subject_type), Some(subject_id)) => {
            let (subject_type, subject_id) = normalize_subject_identity(subject_type, subject_id)?;
            Some((subject_type, subject_id))
        }
        _ => {
            return Err(anyhow!(
                "subject_type and subject_id must be provided together"
            ))
        }
    };

    Ok(store
        .list_coupon_code_records()
        .await?
        .into_iter()
        .filter(|record| coupon_code_matches_filters(record, input, subject_filter.as_ref()))
        .collect())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CouponCodeSummary {
    pub total_count: usize,
    pub issued_count: usize,
    pub claimed_count: usize,
    pub redeemed_count: usize,
    pub voided_count: usize,
    pub expired_count: usize,
    pub blocked_count: usize,
    pub reserved_count: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_created_at_ms: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CouponRedemptionSummary {
    pub total_count: usize,
    pub pending_count: usize,
    pub fulfilled_count: usize,
    pub voided_count: usize,
    pub reversed_count: usize,
    pub failed_count: usize,
    pub payment_linked_count: usize,
    pub subsidized_count: usize,
    pub total_subsidy_amount: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_created_at_ms: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MarketingOverviewSummary {
    pub campaign_count: usize,
    pub active_campaign_count: usize,
    pub template_count: usize,
    pub active_template_count: usize,
    pub batch_count: usize,
    pub active_batch_count: usize,
    pub code_summary: CouponCodeSummary,
    pub claim_count: usize,
    pub pending_claim_count: usize,
    pub claimed_claim_count: usize,
    pub cancelled_claim_count: usize,
    pub expired_claim_count: usize,
    pub rejected_claim_count: usize,
    pub redemption_summary: CouponRedemptionSummary,
}

pub async fn summarize_coupon_codes<S>(
    store: &S,
    input: &ListCouponCodesInput,
) -> Result<CouponCodeSummary>
where
    S: AdminStore + ?Sized,
{
    let codes = list_coupon_codes(store, input).await?;
    let redemptions = store.list_coupon_redemption_records().await?;
    Ok(summarize_coupon_code_records(&codes, &redemptions))
}

pub async fn list_coupon_redemptions<S>(
    store: &S,
    input: &ListCouponRedemptionsInput,
) -> Result<Vec<CouponRedemptionRecord>>
where
    S: AdminStore + ?Sized,
{
    Ok(store
        .list_coupon_redemption_records()
        .await?
        .into_iter()
        .filter(|record| coupon_redemption_matches_filters(record, input))
        .collect())
}

pub async fn summarize_coupon_redemptions<S>(
    store: &S,
    input: &ListCouponRedemptionsInput,
) -> Result<CouponRedemptionSummary>
where
    S: AdminStore + ?Sized,
{
    let redemptions = list_coupon_redemptions(store, input).await?;
    Ok(summarize_coupon_redemption_records(&redemptions))
}

pub async fn summarize_marketing_overview<S>(store: &S) -> Result<MarketingOverviewSummary>
where
    S: AdminStore + ?Sized,
{
    let campaigns = store.list_marketing_campaign_records().await?;
    let templates = store.list_coupon_template_records().await?;
    let batches = store.list_coupon_code_batch_records().await?;
    let codes = store.list_coupon_code_records().await?;
    let claims = store.list_coupon_claim_records().await?;
    let redemptions = store.list_coupon_redemption_records().await?;

    let mut summary = MarketingOverviewSummary {
        campaign_count: campaigns.len(),
        active_campaign_count: campaigns
            .iter()
            .filter(|campaign| campaign.status == MarketingCampaignStatus::Active)
            .count(),
        template_count: templates.len(),
        active_template_count: templates
            .iter()
            .filter(|template| template.status == CouponTemplateStatus::Active)
            .count(),
        batch_count: batches.len(),
        active_batch_count: batches
            .iter()
            .filter(|batch| batch.status == CouponCodeBatchStatus::Active)
            .count(),
        code_summary: summarize_coupon_code_records(&codes, &redemptions),
        claim_count: claims.len(),
        pending_claim_count: 0,
        claimed_claim_count: 0,
        cancelled_claim_count: 0,
        expired_claim_count: 0,
        rejected_claim_count: 0,
        redemption_summary: summarize_coupon_redemption_records(&redemptions),
    };

    for claim in &claims {
        match claim.status {
            CouponClaimStatus::Pending => summary.pending_claim_count += 1,
            CouponClaimStatus::Claimed => summary.claimed_claim_count += 1,
            CouponClaimStatus::Cancelled => summary.cancelled_claim_count += 1,
            CouponClaimStatus::Expired => summary.expired_claim_count += 1,
            CouponClaimStatus::Rejected => summary.rejected_claim_count += 1,
        }
    }

    Ok(summary)
}

pub async fn find_coupon_redemption<S>(
    store: &S,
    coupon_redemption_id: u64,
) -> Result<Option<CouponRedemptionRecord>>
where
    S: AdminStore + ?Sized,
{
    Ok(store
        .list_coupon_redemption_records()
        .await?
        .into_iter()
        .find(|record| record.coupon_redemption_id == coupon_redemption_id))
}

pub async fn issue_coupon_code<S>(
    store: &S,
    input: IssueCouponCodeInput,
) -> Result<CouponCodeRecord>
where
    S: AdminStore + ?Sized,
{
    let code_lookup_hash = input.code_lookup_hash.trim();
    ensure!(!code_lookup_hash.is_empty(), "code_lookup_hash is required");
    ensure!(
        store
            .find_coupon_code_record_by_lookup_hash(code_lookup_hash)
            .await?
            .is_none(),
        "coupon code lookup hash already exists"
    );

    let batch = load_active_batch(store, input.coupon_code_batch_id).await?;
    let template = load_active_template(store, batch.coupon_template_id).await?;
    ensure_coupon_window(&template, None, input.now_ms)?;
    load_active_campaign_for_window(store, batch.marketing_campaign_id, input.now_ms).await?;

    let code = CouponCodeRecord::new(
        input.coupon_code_id,
        batch.tenant_id,
        batch.organization_id,
        batch.coupon_code_batch_id,
        batch.coupon_template_id,
        batch.marketing_campaign_id,
        code_lookup_hash,
        input.code_kind,
        input.now_ms,
    )
    .with_display_code_prefix(input.display_code_prefix)
    .with_display_code_suffix(input.display_code_suffix)
    .with_expires_at_ms(input.expires_at_ms.or(batch.expires_at_ms))
    .with_updated_at_ms(input.now_ms);
    store.insert_coupon_code_record(&code).await?;

    let updated_batch = batch
        .clone()
        .with_issued_count(batch.issued_count.saturating_add(1))
        .with_updated_at_ms(input.now_ms);
    store
        .insert_coupon_code_batch_record(&updated_batch)
        .await?;

    Ok(code)
}

pub async fn claim_coupon_code<S>(
    store: &S,
    input: ClaimCouponCodeInput,
) -> Result<CouponClaimRecord>
where
    S: AdminStore + ?Sized,
{
    let code_lookup_hash = input.code_lookup_hash.trim();
    ensure!(!code_lookup_hash.is_empty(), "code_lookup_hash is required");
    let (subject_type, subject_id) =
        normalize_subject_identity(&input.subject_type, &input.subject_id)?;

    let stored_code = store
        .find_coupon_code_record_by_lookup_hash(code_lookup_hash)
        .await?
        .ok_or_else(|| anyhow!("coupon code not found"))?;
    let template = load_active_template(store, stored_code.coupon_template_id).await?;
    validate_code_batch_and_campaign(store, &stored_code, input.now_ms).await?;
    ensure_code_usable_for_claim(&stored_code, &template, input.now_ms)?;

    let updated_code = stored_code
        .clone()
        .with_status(CouponCodeStatus::Claimed)
        .with_claim_subject_type(Some(subject_type.clone()))
        .with_claim_subject_id(Some(build_subject_id(
            stored_code.tenant_id,
            stored_code.organization_id,
            &subject_id,
        )))
        .with_claimed_at_ms(Some(input.now_ms))
        .with_updated_at_ms(input.now_ms);
    store.insert_coupon_code_record(&updated_code).await?;

    let batch = load_batch(store, stored_code.coupon_code_batch_id).await?;
    let updated_batch = batch
        .clone()
        .with_claimed_count(batch.claimed_count.saturating_add(1))
        .with_updated_at_ms(input.now_ms);
    store
        .insert_coupon_code_batch_record(&updated_batch)
        .await?;

    let claim = CouponClaimRecord::new(
        input.claim_id,
        stored_code.tenant_id,
        stored_code.organization_id,
        stored_code.coupon_code_id,
        stored_code.coupon_template_id,
        subject_type,
        subject_id,
        input.now_ms,
    )
    .with_status(CouponClaimStatus::Claimed)
    .with_account_id(input.account_id)
    .with_project_id(input.project_id)
    .with_expires_at_ms(updated_code.expires_at_ms)
    .with_updated_at_ms(input.now_ms);

    store.insert_coupon_claim_record(&claim).await
}

pub async fn void_coupon_code<S>(store: &S, input: VoidCouponCodeInput) -> Result<CouponCodeRecord>
where
    S: AdminStore + ?Sized,
{
    let stored_code = store
        .find_coupon_code_record(input.coupon_code_id)
        .await?
        .ok_or_else(|| anyhow!("coupon code not found"))?;

    ensure!(
        stored_code.status != CouponCodeStatus::Redeemed,
        "redeemed coupon code cannot be voided"
    );

    if stored_code.status == CouponCodeStatus::Voided {
        return Ok(stored_code);
    }

    let updated_code = stored_code
        .clone()
        .with_status(CouponCodeStatus::Voided)
        .with_updated_at_ms(input.now_ms);
    store.insert_coupon_code_record(&updated_code).await?;

    let batch = load_batch(store, stored_code.coupon_code_batch_id).await?;
    let updated_batch = batch
        .clone()
        .with_voided_count(batch.voided_count.saturating_add(1))
        .with_updated_at_ms(input.now_ms);
    store
        .insert_coupon_code_batch_record(&updated_batch)
        .await?;

    for claim in store.list_coupon_claim_records().await? {
        if claim.coupon_code_id == stored_code.coupon_code_id {
            let updated_claim = claim
                .with_status(CouponClaimStatus::Cancelled)
                .with_updated_at_ms(input.now_ms);
            store.insert_coupon_claim_record(&updated_claim).await?;
        }
    }

    Ok(updated_code)
}

pub async fn expire_due_coupon_codes<S>(
    store: &S,
    input: ExpireDueCouponCodesInput,
) -> Result<ExpireDueCouponCodesResult>
where
    S: AdminStore + ?Sized,
{
    let mut expired_code_ids = Vec::new();
    for code in store.list_coupon_code_records().await? {
        let is_due = code
            .expires_at_ms
            .map(|expires_at_ms| expires_at_ms < input.now_ms)
            .unwrap_or(false);
        if !is_due {
            continue;
        }

        if !matches!(
            code.status,
            CouponCodeStatus::Issued | CouponCodeStatus::Claimed
        ) {
            continue;
        }

        let updated_code = code
            .clone()
            .with_status(CouponCodeStatus::Expired)
            .with_updated_at_ms(input.now_ms);
        store.insert_coupon_code_record(&updated_code).await?;
        expired_code_ids.push(updated_code.coupon_code_id);
    }

    let mut expired_claim_ids = Vec::new();
    for claim in store.list_coupon_claim_records().await? {
        if !expired_code_ids.contains(&claim.coupon_code_id) {
            continue;
        }
        if !matches!(
            claim.status,
            CouponClaimStatus::Pending | CouponClaimStatus::Claimed
        ) {
            continue;
        }

        let updated_claim = claim
            .with_status(CouponClaimStatus::Expired)
            .with_updated_at_ms(input.now_ms);
        store.insert_coupon_claim_record(&updated_claim).await?;
        expired_claim_ids.push(updated_claim.coupon_claim_id);
    }

    Ok(ExpireDueCouponCodesResult {
        expired_code_ids,
        expired_claim_ids,
    })
}

pub async fn validate_coupon_for_quote<S>(
    store: &S,
    input: ValidateCouponForQuoteInput,
) -> Result<CouponQuoteValidation>
where
    S: AdminStore + ?Sized,
{
    let code_lookup_hash = input.code_lookup_hash.trim();
    ensure!(!code_lookup_hash.is_empty(), "code_lookup_hash is required");
    let (subject_type, subject_id) =
        normalize_subject_identity(&input.subject_type, &input.subject_id)?;

    let stored_code = store
        .find_coupon_code_record_by_lookup_hash(code_lookup_hash)
        .await?
        .ok_or_else(|| anyhow!("coupon code not found"))?;
    let template = load_active_template(store, stored_code.coupon_template_id).await?;
    validate_code_batch_and_campaign(store, &stored_code, input.now_ms).await?;
    ensure_code_usable_for_quote(
        &stored_code,
        &template,
        &subject_type,
        &subject_id,
        input.now_ms,
    )?;
    ensure_no_conflicting_pending_coupon_reservation(
        store,
        stored_code.coupon_code_id,
        input.reservation_idempotency_key.as_deref(),
    )
    .await?;

    let rule = select_quote_benefit_rule(
        store,
        &template,
        input.target_order_kind.as_deref(),
        input.target_product_id.as_deref(),
    )
    .await?;

    Ok(CouponQuoteValidation {
        coupon_code_id: stored_code.coupon_code_id,
        coupon_template_id: stored_code.coupon_template_id,
        coupon_benefit_rule_id: rule.coupon_benefit_rule_id,
        benefit_kind: rule.benefit_kind,
        percentage_off: rule.percentage_off,
        fixed_discount_amount: rule.fixed_discount_amount,
        maximum_subsidy_amount: rule.maximum_subsidy_amount,
        currency_code: rule.currency_code,
    })
}

pub async fn reserve_coupon_redemption<S>(
    store: &S,
    input: ReserveCouponRedemptionInput,
) -> Result<CouponRedemptionRecord>
where
    S: AdminStore + ?Sized,
{
    let code_lookup_hash = input.code_lookup_hash.trim();
    ensure!(!code_lookup_hash.is_empty(), "code_lookup_hash is required");
    let idempotency_key = input.idempotency_key.trim();
    ensure!(!idempotency_key.is_empty(), "idempotency_key is required");
    let (subject_type, subject_id) =
        normalize_subject_identity(&input.subject_type, &input.subject_id)?;

    if let Some(existing) = store
        .find_coupon_redemption_record_by_idempotency_key(idempotency_key)
        .await?
    {
        match existing.status {
            CouponRedemptionStatus::Pending | CouponRedemptionStatus::Fulfilled => {
                return Ok(existing);
            }
            _ => return Err(anyhow!("coupon redemption reservation is not active")),
        }
    }

    let stored_code = store
        .find_coupon_code_record_by_lookup_hash(code_lookup_hash)
        .await?
        .ok_or_else(|| anyhow!("coupon code not found"))?;
    let template = load_active_template(store, stored_code.coupon_template_id).await?;
    validate_code_batch_and_campaign(store, &stored_code, input.now_ms).await?;
    ensure_code_usable_for_quote(
        &stored_code,
        &template,
        &subject_type,
        &subject_id,
        input.now_ms,
    )?;
    ensure_no_conflicting_pending_coupon_reservation(
        store,
        stored_code.coupon_code_id,
        Some(idempotency_key),
    )
    .await?;

    let reservation = CouponRedemptionRecord::new(
        input.redemption_id,
        stored_code.tenant_id,
        stored_code.organization_id,
        stored_code.coupon_code_id,
        stored_code.coupon_template_id,
        stored_code.marketing_campaign_id,
        subject_type,
        subject_id,
        input.now_ms,
    )
    .with_status(CouponRedemptionStatus::Pending)
    .with_account_id(input.account_id)
    .with_project_id(input.project_id)
    .with_order_id(input.order_id)
    .with_idempotency_key(Some(idempotency_key.to_owned()))
    .with_updated_at_ms(input.now_ms);

    store.insert_coupon_redemption_record(&reservation).await
}

pub async fn release_coupon_redemption_reservation<S>(
    store: &S,
    input: ReleaseCouponRedemptionReservationInput,
) -> Result<Option<CouponRedemptionRecord>>
where
    S: AdminStore + ?Sized,
{
    let idempotency_key = input.idempotency_key.trim();
    ensure!(!idempotency_key.is_empty(), "idempotency_key is required");
    ensure!(
        matches!(
            input.status,
            CouponRedemptionStatus::Voided | CouponRedemptionStatus::Failed
        ),
        "reservation release status must be voided or failed"
    );

    let Some(existing) = store
        .find_coupon_redemption_record_by_idempotency_key(idempotency_key)
        .await?
    else {
        return Ok(None);
    };

    if existing.status == input.status {
        return Ok(Some(existing));
    }
    if existing.status != CouponRedemptionStatus::Pending {
        return Ok(Some(existing));
    }

    let released = existing
        .with_status(input.status)
        .with_updated_at_ms(input.now_ms);
    store
        .insert_coupon_redemption_record(&released)
        .await
        .map(Some)
}

pub async fn redeem_coupon_code<S>(
    store: &S,
    input: RedeemCouponCodeInput,
) -> Result<CouponRedemptionRecord>
where
    S: AdminStore + ?Sized,
{
    let code_lookup_hash = input.code_lookup_hash.trim();
    ensure!(!code_lookup_hash.is_empty(), "code_lookup_hash is required");
    let idempotency_key = input.idempotency_key.trim();
    ensure!(!idempotency_key.is_empty(), "idempotency_key is required");
    let (subject_type, subject_id) =
        normalize_subject_identity(&input.subject_type, &input.subject_id)?;

    let existing = store
        .find_coupon_redemption_record_by_idempotency_key(idempotency_key)
        .await?;
    if let Some(existing) = existing.as_ref() {
        match existing.status {
            CouponRedemptionStatus::Fulfilled => return Ok(existing.clone()),
            CouponRedemptionStatus::Pending => {
                ensure!(
                    existing.subject_type == subject_type && existing.subject_id == subject_id,
                    "coupon redemption reservation is owned by a different subject"
                );
            }
            _ => return Err(anyhow!("coupon redemption reservation is not active")),
        }
    }

    let stored_code = store
        .find_coupon_code_record_by_lookup_hash(code_lookup_hash)
        .await?
        .ok_or_else(|| anyhow!("coupon code not found"))?;
    let template = load_active_template(store, stored_code.coupon_template_id).await?;
    validate_code_batch_and_campaign(store, &stored_code, input.now_ms).await?;
    ensure_no_conflicting_pending_coupon_reservation(
        store,
        stored_code.coupon_code_id,
        Some(idempotency_key),
    )
    .await?;
    ensure_code_usable_for_redeem(
        &stored_code,
        &template,
        &subject_type,
        &subject_id,
        input.now_ms,
    )?;

    let updated_code = stored_code
        .clone()
        .with_status(CouponCodeStatus::Redeemed)
        .with_claim_subject_type(Some(subject_type.clone()))
        .with_claim_subject_id(Some(build_subject_id(
            stored_code.tenant_id,
            stored_code.organization_id,
            &subject_id,
        )))
        .with_redeemed_at_ms(Some(input.now_ms))
        .with_updated_at_ms(input.now_ms);
    store.insert_coupon_code_record(&updated_code).await?;

    let batch = load_batch(store, stored_code.coupon_code_batch_id).await?;
    let updated_batch = batch
        .clone()
        .with_redeemed_count(batch.redeemed_count.saturating_add(1))
        .with_updated_at_ms(input.now_ms);
    store
        .insert_coupon_code_batch_record(&updated_batch)
        .await?;

    let redemption = existing.unwrap_or_else(|| {
        CouponRedemptionRecord::new(
            input.redemption_id,
            stored_code.tenant_id,
            stored_code.organization_id,
            stored_code.coupon_code_id,
            stored_code.coupon_template_id,
            stored_code.marketing_campaign_id,
            subject_type.clone(),
            subject_id.clone(),
            input.now_ms,
        )
    });
    let redemption = redemption
        .with_status(CouponRedemptionStatus::Fulfilled)
        .with_account_id(input.account_id)
        .with_project_id(input.project_id)
        .with_order_id(input.order_id)
        .with_payment_order_id(input.payment_order_id)
        .with_idempotency_key(Some(idempotency_key.to_owned()))
        .with_updated_at_ms(input.now_ms);

    store.insert_coupon_redemption_record(&redemption).await
}

fn coupon_redemption_matches_filters(
    record: &CouponRedemptionRecord,
    input: &ListCouponRedemptionsInput,
) -> bool {
    if let Some(subject_type) = input.subject_type.as_deref() {
        if record.subject_type != subject_type {
            return false;
        }
    }
    if let Some(subject_id) = input.subject_id.as_deref() {
        if record.subject_id != subject_id {
            return false;
        }
    }
    if let Some(project_id) = input.project_id.as_deref() {
        if record.project_id.as_deref() != Some(project_id) {
            return false;
        }
    }
    if let Some(order_id) = input.order_id.as_deref() {
        if record.order_id.as_deref() != Some(order_id) {
            return false;
        }
    }
    if let Some(payment_order_id) = input.payment_order_id.as_deref() {
        if record.payment_order_id.as_deref() != Some(payment_order_id) {
            return false;
        }
    }
    if let Some(coupon_template_id) = input.coupon_template_id {
        if record.coupon_template_id != coupon_template_id {
            return false;
        }
    }
    if let Some(coupon_code_id) = input.coupon_code_id {
        if record.coupon_code_id != coupon_code_id {
            return false;
        }
    }
    if let Some(marketing_campaign_id) = input.marketing_campaign_id {
        if record.marketing_campaign_id != Some(marketing_campaign_id) {
            return false;
        }
    }
    if let Some(status) = input.status {
        if record.status != status {
            return false;
        }
    }
    true
}

fn coupon_code_matches_filters(
    record: &CouponCodeRecord,
    input: &ListCouponCodesInput,
    subject_filter: Option<&(String, String)>,
) -> bool {
    if let Some((subject_type, subject_id)) = subject_filter {
        if !coupon_code_belongs_to_subject(record, subject_type, subject_id) {
            return false;
        }
    }
    if let Some(coupon_template_id) = input.coupon_template_id {
        if record.coupon_template_id != coupon_template_id {
            return false;
        }
    }
    if let Some(coupon_code_batch_id) = input.coupon_code_batch_id {
        if record.coupon_code_batch_id != coupon_code_batch_id {
            return false;
        }
    }
    if let Some(marketing_campaign_id) = input.marketing_campaign_id {
        if record.marketing_campaign_id != Some(marketing_campaign_id) {
            return false;
        }
    }
    if let Some(status) = input.status {
        if record.status != status {
            return false;
        }
    }
    true
}

fn coupon_code_belongs_to_subject(
    record: &CouponCodeRecord,
    subject_type: &str,
    subject_id: &str,
) -> bool {
    if record.claim_subject_type.as_deref() != Some(subject_type) {
        return false;
    }
    let Some(claim_subject_id) = record.claim_subject_id.as_deref() else {
        return false;
    };
    claim_subject_id == subject_id || claim_subject_id.rsplit(':').next() == Some(subject_id)
}

fn summarize_coupon_redemption_records(
    redemptions: &[CouponRedemptionRecord],
) -> CouponRedemptionSummary {
    let mut summary = CouponRedemptionSummary {
        total_count: redemptions.len(),
        pending_count: 0,
        fulfilled_count: 0,
        voided_count: 0,
        reversed_count: 0,
        failed_count: 0,
        payment_linked_count: 0,
        subsidized_count: 0,
        total_subsidy_amount: 0.0,
        latest_created_at_ms: None,
    };

    for redemption in redemptions {
        match redemption.status {
            CouponRedemptionStatus::Pending => summary.pending_count += 1,
            CouponRedemptionStatus::Fulfilled => summary.fulfilled_count += 1,
            CouponRedemptionStatus::Voided => summary.voided_count += 1,
            CouponRedemptionStatus::Reversed => summary.reversed_count += 1,
            CouponRedemptionStatus::Failed => summary.failed_count += 1,
        }
        if redemption.payment_order_id.is_some() {
            summary.payment_linked_count += 1;
        }
        if let Some(subsidy_amount) = redemption.subsidy_amount {
            summary.subsidized_count += 1;
            summary.total_subsidy_amount += subsidy_amount;
        }
        summary.latest_created_at_ms = Some(
            summary
                .latest_created_at_ms
                .map(|current| current.max(redemption.created_at_ms))
                .unwrap_or(redemption.created_at_ms),
        );
    }

    summary
}

fn summarize_coupon_code_records(
    codes: &[CouponCodeRecord],
    redemptions: &[CouponRedemptionRecord],
) -> CouponCodeSummary {
    let code_ids = codes
        .iter()
        .map(|code| code.coupon_code_id)
        .collect::<HashSet<_>>();
    let reserved_count = redemptions
        .iter()
        .filter(|redemption| {
            redemption.status == CouponRedemptionStatus::Pending
                && code_ids.contains(&redemption.coupon_code_id)
        })
        .map(|redemption| redemption.coupon_code_id)
        .collect::<HashSet<_>>()
        .len();
    let mut summary = CouponCodeSummary {
        total_count: codes.len(),
        issued_count: 0,
        claimed_count: 0,
        redeemed_count: 0,
        voided_count: 0,
        expired_count: 0,
        blocked_count: 0,
        reserved_count,
        latest_created_at_ms: None,
    };

    for code in codes {
        match code.status {
            CouponCodeStatus::Issued => summary.issued_count += 1,
            CouponCodeStatus::Claimed => summary.claimed_count += 1,
            CouponCodeStatus::Redeemed => summary.redeemed_count += 1,
            CouponCodeStatus::Voided => summary.voided_count += 1,
            CouponCodeStatus::Expired => summary.expired_count += 1,
            CouponCodeStatus::Blocked => summary.blocked_count += 1,
        }
        summary.latest_created_at_ms = Some(
            summary
                .latest_created_at_ms
                .map(|current| current.max(code.created_at_ms))
                .unwrap_or(code.created_at_ms),
        );
    }

    summary
}

async fn ensure_no_conflicting_pending_coupon_reservation<S>(
    store: &S,
    coupon_code_id: u64,
    allowed_idempotency_key: Option<&str>,
) -> Result<()>
where
    S: AdminStore + ?Sized,
{
    let allowed_idempotency_key = allowed_idempotency_key
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let has_conflict = store
        .list_coupon_redemption_records()
        .await?
        .into_iter()
        .any(|record| {
            record.coupon_code_id == coupon_code_id
                && record.status == CouponRedemptionStatus::Pending
                && record.idempotency_key.as_deref() != allowed_idempotency_key
        });
    ensure!(
        !has_conflict,
        "coupon code is already reserved for checkout"
    );
    Ok(())
}

async fn load_active_template<S>(store: &S, coupon_template_id: u64) -> Result<CouponTemplateRecord>
where
    S: AdminStore + ?Sized,
{
    let template = store
        .find_coupon_template_record(coupon_template_id)
        .await?
        .ok_or_else(|| anyhow!("coupon template not found"))?;
    ensure!(
        template.status == CouponTemplateStatus::Active,
        "coupon template is not active"
    );
    Ok(template)
}

async fn load_active_batch<S>(store: &S, coupon_code_batch_id: u64) -> Result<CouponCodeBatchRecord>
where
    S: AdminStore + ?Sized,
{
    let batch = load_batch(store, coupon_code_batch_id).await?;
    ensure!(
        batch.status == CouponCodeBatchStatus::Active,
        "coupon code batch is not active"
    );
    Ok(batch)
}

async fn load_batch<S>(store: &S, coupon_code_batch_id: u64) -> Result<CouponCodeBatchRecord>
where
    S: AdminStore + ?Sized,
{
    store
        .list_coupon_code_batch_records()
        .await?
        .into_iter()
        .find(|candidate| candidate.coupon_code_batch_id == coupon_code_batch_id)
        .ok_or_else(|| anyhow!("coupon code batch not found"))
}

async fn load_active_campaign_for_window<S>(
    store: &S,
    marketing_campaign_id: Option<u64>,
    now_ms: u64,
) -> Result<Option<MarketingCampaignRecord>>
where
    S: AdminStore + ?Sized,
{
    let Some(marketing_campaign_id) = marketing_campaign_id else {
        return Ok(None);
    };

    let campaign = store
        .find_marketing_campaign_record(marketing_campaign_id)
        .await?
        .ok_or_else(|| anyhow!("marketing campaign not found"))?;
    ensure!(
        campaign.status == MarketingCampaignStatus::Active,
        "marketing campaign is not active"
    );
    ensure!(
        campaign
            .starts_at_ms
            .map(|starts_at_ms| starts_at_ms <= now_ms)
            .unwrap_or(true),
        "marketing campaign is not active yet"
    );
    ensure!(
        campaign
            .ends_at_ms
            .map(|ends_at_ms| ends_at_ms >= now_ms)
            .unwrap_or(true),
        "marketing campaign has expired"
    );
    Ok(Some(campaign))
}

async fn validate_code_batch_and_campaign<S>(
    store: &S,
    code: &CouponCodeRecord,
    now_ms: u64,
) -> Result<()>
where
    S: AdminStore + ?Sized,
{
    let batch = load_active_batch(store, code.coupon_code_batch_id).await?;
    ensure!(
        batch.coupon_template_id == code.coupon_template_id,
        "coupon code batch does not match coupon template"
    );

    let marketing_campaign_id = code.marketing_campaign_id.or(batch.marketing_campaign_id);
    load_active_campaign_for_window(store, marketing_campaign_id, now_ms).await?;
    Ok(())
}

fn ensure_code_usable_for_claim(
    code: &CouponCodeRecord,
    template: &CouponTemplateRecord,
    now_ms: u64,
) -> Result<()> {
    ensure_code_not_terminal(code)?;
    ensure!(
        code.status == CouponCodeStatus::Issued,
        "coupon code is not claimable"
    );
    ensure_coupon_window(template, Some(code), now_ms)
}

fn ensure_code_usable_for_redeem(
    code: &CouponCodeRecord,
    template: &CouponTemplateRecord,
    subject_type: &str,
    subject_id: &str,
    now_ms: u64,
) -> Result<()> {
    ensure_code_not_terminal(code)?;
    ensure_coupon_window(template, Some(code), now_ms)?;

    if template.claim_required {
        ensure!(
            code.status == CouponCodeStatus::Claimed,
            "coupon code must be claimed before redemption"
        );
        ensure!(
            code_is_owned_by_subject(code, subject_type, subject_id),
            "coupon code is not owned by the redeeming subject"
        );
    } else {
        ensure!(
            matches!(
                code.status,
                CouponCodeStatus::Issued | CouponCodeStatus::Claimed
            ),
            "coupon code is not redeemable"
        );
    }

    Ok(())
}

fn ensure_code_usable_for_quote(
    code: &CouponCodeRecord,
    template: &CouponTemplateRecord,
    subject_type: &str,
    subject_id: &str,
    now_ms: u64,
) -> Result<()> {
    ensure_code_not_terminal(code)?;
    ensure_coupon_window(template, Some(code), now_ms)?;

    match template.benefit_kind {
        CouponBenefitKind::PercentageDiscount | CouponBenefitKind::FixedAmountDiscount => {}
        _ => return Err(anyhow!("coupon template is not quote-applicable")),
    }

    if template.claim_required {
        ensure!(
            code.status == CouponCodeStatus::Claimed,
            "coupon code must be claimed before quote validation"
        );
        ensure!(
            code_is_owned_by_subject(code, subject_type, subject_id),
            "coupon code is not owned by the quoting subject"
        );
    } else {
        ensure!(
            matches!(
                code.status,
                CouponCodeStatus::Issued | CouponCodeStatus::Claimed
            ),
            "coupon code is not quote-eligible"
        );
    }

    Ok(())
}

async fn select_quote_benefit_rule<S>(
    store: &S,
    template: &CouponTemplateRecord,
    target_order_kind: Option<&str>,
    target_product_id: Option<&str>,
) -> Result<CouponBenefitRuleRecord>
where
    S: AdminStore + ?Sized,
{
    store
        .list_coupon_benefit_rule_records()
        .await?
        .into_iter()
        .find(|rule| {
            rule.coupon_template_id == template.coupon_template_id
                && rule.benefit_kind == template.benefit_kind
                && target_matches(rule.target_order_kind.as_deref(), target_order_kind)
                && target_matches(rule.target_product_id.as_deref(), target_product_id)
        })
        .ok_or_else(|| anyhow!("coupon template has no applicable quote-time benefit rule"))
}

fn target_matches(rule_value: Option<&str>, requested_value: Option<&str>) -> bool {
    match (rule_value, requested_value) {
        (None, _) => true,
        (Some(_), None) => false,
        (Some(rule_value), Some(requested_value)) => {
            rule_value.eq_ignore_ascii_case(requested_value)
        }
    }
}

fn ensure_code_not_terminal(code: &CouponCodeRecord) -> Result<()> {
    match code.status {
        CouponCodeStatus::Redeemed => Err(anyhow!("coupon code has already been redeemed")),
        CouponCodeStatus::Voided => Err(anyhow!("coupon code has been voided")),
        CouponCodeStatus::Expired => Err(anyhow!("coupon code has expired")),
        CouponCodeStatus::Blocked => Err(anyhow!("coupon code is blocked")),
        _ => Ok(()),
    }
}

fn ensure_coupon_window(
    template: &CouponTemplateRecord,
    code: Option<&CouponCodeRecord>,
    now_ms: u64,
) -> Result<()> {
    ensure!(
        template
            .starts_at_ms
            .map(|starts_at_ms| starts_at_ms <= now_ms)
            .unwrap_or(true),
        "coupon template is not active yet"
    );
    ensure!(
        template
            .ends_at_ms
            .map(|ends_at_ms| ends_at_ms >= now_ms)
            .unwrap_or(true),
        "coupon template has expired"
    );
    if let Some(code) = code {
        ensure!(
            code.expires_at_ms
                .map(|expires_at_ms| expires_at_ms >= now_ms)
                .unwrap_or(true),
            "coupon code has expired"
        );
    }
    Ok(())
}

fn normalize_subject_identity(subject_type: &str, subject_id: &str) -> Result<(String, String)> {
    let normalized_subject_type = subject_type.trim().to_ascii_lowercase();
    let normalized_subject_id = subject_id.trim().to_owned();
    ensure!(
        !normalized_subject_type.is_empty(),
        "subject_type is required"
    );
    ensure!(!normalized_subject_id.is_empty(), "subject_id is required");
    Ok((normalized_subject_type, normalized_subject_id))
}

fn code_is_owned_by_subject(code: &CouponCodeRecord, subject_type: &str, subject_id: &str) -> bool {
    let scoped_subject_id = build_subject_id(code.tenant_id, code.organization_id, subject_id);
    code.claim_subject_type.as_deref() == Some(subject_type)
        && code.claim_subject_id.as_deref() == Some(scoped_subject_id.as_str())
}

fn build_subject_id(tenant_id: u64, organization_id: u64, subject_id: &str) -> String {
    format!("{tenant_id}:{organization_id}:{subject_id}")
}
