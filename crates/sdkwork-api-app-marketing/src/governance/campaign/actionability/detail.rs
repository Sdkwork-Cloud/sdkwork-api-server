use super::super::types::MarketingCampaignDetail;
use super::decision::build_marketing_campaign_actionability;
use sdkwork_api_domain_marketing::{CouponTemplateRecord, MarketingCampaignRecord};

pub(in crate::governance::campaign) fn build_marketing_campaign_detail(
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
