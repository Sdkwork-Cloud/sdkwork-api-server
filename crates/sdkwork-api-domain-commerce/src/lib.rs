use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommerceOrderRecord {
    pub order_id: String,
    pub project_id: String,
    pub user_id: String,
    pub target_kind: String,
    pub target_id: String,
    pub target_name: String,
    pub list_price_cents: u64,
    pub payable_price_cents: u64,
    pub list_price_label: String,
    pub payable_price_label: String,
    pub granted_units: u64,
    pub bonus_units: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub applied_coupon_code: Option<String>,
    pub status: String,
    pub source: String,
    pub created_at_ms: u64,
}

impl CommerceOrderRecord {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        order_id: impl Into<String>,
        project_id: impl Into<String>,
        user_id: impl Into<String>,
        target_kind: impl Into<String>,
        target_id: impl Into<String>,
        target_name: impl Into<String>,
        list_price_cents: u64,
        payable_price_cents: u64,
        list_price_label: impl Into<String>,
        payable_price_label: impl Into<String>,
        granted_units: u64,
        bonus_units: u64,
        status: impl Into<String>,
        source: impl Into<String>,
        created_at_ms: u64,
    ) -> Self {
        Self {
            order_id: order_id.into(),
            project_id: project_id.into(),
            user_id: user_id.into(),
            target_kind: target_kind.into(),
            target_id: target_id.into(),
            target_name: target_name.into(),
            list_price_cents,
            payable_price_cents,
            list_price_label: list_price_label.into(),
            payable_price_label: payable_price_label.into(),
            granted_units,
            bonus_units,
            applied_coupon_code: None,
            status: status.into(),
            source: source.into(),
            created_at_ms,
        }
    }

    pub fn with_applied_coupon_code_option(mut self, applied_coupon_code: Option<String>) -> Self {
        self.applied_coupon_code = applied_coupon_code;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectMembershipRecord {
    pub membership_id: String,
    pub project_id: String,
    pub user_id: String,
    pub plan_id: String,
    pub plan_name: String,
    pub price_cents: u64,
    pub price_label: String,
    pub cadence: String,
    pub included_units: u64,
    pub status: String,
    pub source: String,
    pub activated_at_ms: u64,
    pub updated_at_ms: u64,
}

impl ProjectMembershipRecord {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        membership_id: impl Into<String>,
        project_id: impl Into<String>,
        user_id: impl Into<String>,
        plan_id: impl Into<String>,
        plan_name: impl Into<String>,
        price_cents: u64,
        price_label: impl Into<String>,
        cadence: impl Into<String>,
        included_units: u64,
        status: impl Into<String>,
        source: impl Into<String>,
        activated_at_ms: u64,
        updated_at_ms: u64,
    ) -> Self {
        Self {
            membership_id: membership_id.into(),
            project_id: project_id.into(),
            user_id: user_id.into(),
            plan_id: plan_id.into(),
            plan_name: plan_name.into(),
            price_cents,
            price_label: price_label.into(),
            cadence: cadence.into(),
            included_units,
            status: status.into(),
            source: source.into(),
            activated_at_ms,
            updated_at_ms,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{CommerceOrderRecord, ProjectMembershipRecord};

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
        assert_eq!(order.applied_coupon_code.as_deref(), Some("SPRING20"));
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
}
