use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
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
