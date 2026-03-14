use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LedgerEntry {
    pub project_id: String,
    pub units: u64,
    pub amount: f64,
}

impl LedgerEntry {
    pub fn new(project_id: impl Into<String>, units: u64, amount: f64) -> Self {
        Self {
            project_id: project_id.into(),
            units,
            amount,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuotaPolicy {
    pub policy_id: String,
    pub project_id: String,
    pub max_units: u64,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

impl QuotaPolicy {
    pub fn new(
        policy_id: impl Into<String>,
        project_id: impl Into<String>,
        max_units: u64,
    ) -> Self {
        Self {
            policy_id: policy_id.into(),
            project_id: project_id.into(),
            max_units,
            enabled: true,
        }
    }

    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuotaCheckResult {
    pub allowed: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub policy_id: Option<String>,
    pub requested_units: u64,
    pub used_units: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit_units: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remaining_units: Option<u64>,
}

impl QuotaCheckResult {
    pub fn allowed_without_policy(requested_units: u64, used_units: u64) -> Self {
        Self {
            allowed: true,
            policy_id: None,
            requested_units,
            used_units,
            limit_units: None,
            remaining_units: None,
        }
    }

    pub fn from_policy(policy: &QuotaPolicy, used_units: u64, requested_units: u64) -> Self {
        let remaining_units = policy.max_units.saturating_sub(used_units);
        Self {
            allowed: used_units.saturating_add(requested_units) <= policy.max_units,
            policy_id: Some(policy.policy_id.clone()),
            requested_units,
            used_units,
            limit_units: Some(policy.max_units),
            remaining_units: Some(remaining_units),
        }
    }
}

fn default_enabled() -> bool {
    true
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectBillingSummary {
    pub project_id: String,
    pub entry_count: u64,
    pub used_units: u64,
    pub booked_amount: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quota_policy_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quota_limit_units: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remaining_units: Option<u64>,
    #[serde(default)]
    pub exhausted: bool,
}

impl ProjectBillingSummary {
    pub fn new(project_id: impl Into<String>) -> Self {
        Self {
            project_id: project_id.into(),
            entry_count: 0,
            used_units: 0,
            booked_amount: 0.0,
            quota_policy_id: None,
            quota_limit_units: None,
            remaining_units: None,
            exhausted: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BillingSummary {
    pub total_entries: u64,
    pub project_count: u64,
    pub total_units: u64,
    pub total_amount: f64,
    pub active_quota_policy_count: u64,
    pub exhausted_project_count: u64,
    pub projects: Vec<ProjectBillingSummary>,
}

impl BillingSummary {
    pub fn empty() -> Self {
        Self {
            total_entries: 0,
            project_count: 0,
            total_units: 0,
            total_amount: 0.0,
            active_quota_policy_count: 0,
            exhausted_project_count: 0,
            projects: Vec::new(),
        }
    }
}
