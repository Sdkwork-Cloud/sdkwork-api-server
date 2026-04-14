use super::super::types::{MarketingCampaignActionDecision, MarketingCampaignActionability};
use sdkwork_api_domain_marketing::{
    CouponTemplateRecord, CouponTemplateStatus, MarketingCampaignApprovalState,
    MarketingCampaignRecord, MarketingCampaignStatus,
};

fn allowed_marketing_campaign_action() -> MarketingCampaignActionDecision {
    MarketingCampaignActionDecision {
        allowed: true,
        reasons: Vec::new(),
    }
}

fn blocked_marketing_campaign_action(reason: impl Into<String>) -> MarketingCampaignActionDecision {
    MarketingCampaignActionDecision {
        allowed: false,
        reasons: vec![reason.into()],
    }
}

pub(in crate::governance::campaign) fn build_marketing_campaign_actionability(
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
    let approval_in_review = campaign.approval_state == MarketingCampaignApprovalState::InReview;
    let approval_approved = campaign.approval_state == MarketingCampaignApprovalState::Approved;
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
