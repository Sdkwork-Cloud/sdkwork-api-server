use super::super::types::{CouponTemplateActionDecision, CouponTemplateActionability};
use sdkwork_api_domain_marketing::{
    CouponTemplateApprovalState, CouponTemplateRecord, CouponTemplateStatus,
};

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

pub(in crate::governance::template) fn build_coupon_template_actionability(
    coupon_template: &CouponTemplateRecord,
    now_ms: u64,
) -> CouponTemplateActionability {
    let template_archived = coupon_template.status == CouponTemplateStatus::Archived;
    let template_active = coupon_template.status == CouponTemplateStatus::Active;
    let template_scheduled = coupon_template.status == CouponTemplateStatus::Scheduled;
    let template_draft = coupon_template.status == CouponTemplateStatus::Draft;
    let approval_in_review =
        coupon_template.approval_state == CouponTemplateApprovalState::InReview;
    let approval_approved = coupon_template.approval_state == CouponTemplateApprovalState::Approved;
    let has_future_activation = coupon_template
        .activation_at_ms
        .is_some_and(|value| value > now_ms);

    let clone = allowed_coupon_template_action();
    let submit_for_approval = if !template_draft {
        blocked_coupon_template_action(
            "coupon template must remain draft before approval submission",
        )
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
