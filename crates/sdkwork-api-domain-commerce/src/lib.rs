mod membership;
mod order;
mod payment_event;
mod payment_method;
mod refund;
mod webhook;

pub use membership::ProjectMembershipRecord;
pub use order::CommerceOrderRecord;
pub use payment_event::{
    CommercePaymentEventProcessingStatus, CommercePaymentEventRecord,
};
pub use payment_method::{
    CommercePaymentAttemptRecord, PaymentMethodCredentialBindingRecord, PaymentMethodRecord,
};
pub use refund::{
    CommerceReconciliationItemRecord, CommerceReconciliationRunRecord, CommerceRefundRecord,
};
pub use webhook::{
    CommerceWebhookDeliveryAttemptRecord, CommerceWebhookInboxRecord,
};

#[cfg(test)]
mod tests {
    use super::{
        CommerceOrderRecord, CommercePaymentEventProcessingStatus, CommercePaymentEventRecord,
        ProjectMembershipRecord,
    };

    #[test]
    fn commerce_order_keeps_operational_fields() {
        let order = CommerceOrderRecord::new(
            "order_1",
            "project_demo",
            "user_demo",
            "recharge_pack",
            "pack-100k",
            "Boost 100k",
            4_000,
            3_200,
            "$40.00",
            "$32.00",
            100_000,
            0,
            "fulfilled",
            "workspace_seed",
            1_710_000_001,
        )
        .with_applied_coupon_code_option(Some("SPRING20".to_owned()));

        assert_eq!(order.order_id, "order_1");
        assert_eq!(order.project_id, "project_demo");
        assert_eq!(order.user_id, "user_demo");
        assert_eq!(order.target_kind, "recharge_pack");
        assert_eq!(order.payable_price_cents, 3_200);
        assert_eq!(order.granted_units, 100_000);
        assert_eq!(order.currency_code, "USD");
        assert_eq!(order.applied_coupon_code.as_deref(), Some("SPRING20"));
        assert!(order.coupon_reservation_id.is_none());
        assert!(order.coupon_redemption_id.is_none());
        assert!(order.marketing_campaign_id.is_none());
        assert_eq!(order.subsidy_amount_minor, 0);
        assert_eq!(order.settlement_status, "settled");
        assert_eq!(order.refundable_amount_minor, 3_200);
        assert_eq!(order.refunded_amount_minor, 0);
        assert_eq!(order.created_at_ms, 1_710_000_001);
        assert_eq!(order.updated_at_ms, 1_710_000_001);
    }

    #[test]
    fn project_membership_captures_active_plan_entitlements() {
        let membership = ProjectMembershipRecord::new(
            "membership_1",
            "project_demo",
            "user_demo",
            "growth",
            "Growth",
            7_900,
            "$79.00",
            "/month",
            100_000,
            "active",
            "workspace_seed",
            1_710_000_100,
            1_710_000_100,
        );

        assert_eq!(membership.project_id, "project_demo");
        assert_eq!(membership.plan_id, "growth");
        assert_eq!(membership.plan_name, "Growth");
        assert_eq!(membership.included_units, 100_000);
        assert_eq!(membership.status, "active");
    }

    #[test]
    fn commerce_payment_event_keeps_audit_and_processing_fields() {
        let event = CommercePaymentEventRecord::new(
            "payment_event_1",
            "order_1",
            "project_demo",
            "user_demo",
            "stripe",
            "stripe:evt_1",
            "settled",
            "{\"event_type\":\"settled\"}",
            1_710_000_200,
        )
        .with_provider_event_id(Some("evt_1".to_owned()))
        .with_processing_status(CommercePaymentEventProcessingStatus::Processed)
        .with_processed_at_ms(Some(1_710_000_250))
        .with_order_status_after(Some("fulfilled".to_owned()));

        assert_eq!(event.order_id, "order_1");
        assert_eq!(event.provider, "stripe");
        assert_eq!(event.provider_event_id.as_deref(), Some("evt_1"));
        assert_eq!(event.dedupe_key, "stripe:evt_1");
        assert_eq!(
            event.processing_status,
            CommercePaymentEventProcessingStatus::Processed
        );
        assert_eq!(event.order_status_after.as_deref(), Some("fulfilled"));
    }
}
