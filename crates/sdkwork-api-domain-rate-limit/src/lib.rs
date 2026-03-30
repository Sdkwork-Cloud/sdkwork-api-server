use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RateLimitPolicy {
    pub policy_id: String,
    pub project_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub route_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_name: Option<String>,
    pub requests_per_window: u64,
    #[serde(default = "default_window_seconds")]
    pub window_seconds: u64,
    #[serde(default)]
    pub burst_requests: u64,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

impl RateLimitPolicy {
    pub fn new(
        policy_id: impl Into<String>,
        project_id: impl Into<String>,
        requests_per_window: u64,
        window_seconds: u64,
    ) -> Self {
        Self {
            policy_id: policy_id.into(),
            project_id: project_id.into(),
            api_key_hash: None,
            route_key: None,
            model_name: None,
            requests_per_window,
            window_seconds: window_seconds.max(1),
            burst_requests: 0,
            enabled: true,
            notes: None,
            created_at_ms: 0,
            updated_at_ms: 0,
        }
    }

    pub fn with_api_key_hash_option(mut self, api_key_hash: Option<String>) -> Self {
        self.api_key_hash = api_key_hash;
        self
    }

    pub fn with_route_key_option(mut self, route_key: Option<String>) -> Self {
        self.route_key = route_key;
        self
    }

    pub fn with_model_name_option(mut self, model_name: Option<String>) -> Self {
        self.model_name = model_name;
        self
    }

    pub fn with_burst_requests(mut self, burst_requests: u64) -> Self {
        self.burst_requests = burst_requests;
        self
    }

    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn with_notes_option(mut self, notes: Option<String>) -> Self {
        self.notes = notes;
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

    pub fn effective_limit_requests(&self) -> u64 {
        match self.burst_requests {
            0 => self.requests_per_window,
            burst_requests => burst_requests.max(self.requests_per_window),
        }
    }

    pub fn specificity_score(&self) -> u8 {
        self.api_key_hash.iter().count() as u8
            + self.route_key.iter().count() as u8
            + self.model_name.iter().count() as u8
    }

    pub fn matches(
        &self,
        project_id: &str,
        api_key_hash: Option<&str>,
        route_key: &str,
        model_name: Option<&str>,
    ) -> bool {
        self.project_id == project_id
            && self.enabled
            && self
                .api_key_hash
                .as_deref()
                .is_none_or(|value| api_key_hash == Some(value))
            && self
                .route_key
                .as_deref()
                .is_none_or(|value| value == route_key)
            && self
                .model_name
                .as_deref()
                .is_none_or(|value| model_name == Some(value))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RateLimitCheckResult {
    pub allowed: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub policy_id: Option<String>,
    pub requested_requests: u64,
    pub used_requests: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit_requests: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remaining_requests: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window_seconds: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window_start_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window_end_ms: Option<u64>,
}

impl RateLimitCheckResult {
    pub fn allowed_without_policy(requested_requests: u64, used_requests: u64) -> Self {
        Self {
            allowed: true,
            policy_id: None,
            requested_requests,
            used_requests,
            limit_requests: None,
            remaining_requests: None,
            window_seconds: None,
            window_start_ms: None,
            window_end_ms: None,
        }
    }

    pub fn from_policy(
        policy: &RateLimitPolicy,
        used_requests: u64,
        requested_requests: u64,
        window_start_ms: u64,
    ) -> Self {
        let limit_requests = policy.effective_limit_requests();
        let remaining_requests = limit_requests.saturating_sub(used_requests);
        let window_end_ms =
            window_start_ms.saturating_add(policy.window_seconds.saturating_mul(1000));
        Self {
            allowed: used_requests.saturating_add(requested_requests) <= limit_requests,
            policy_id: Some(policy.policy_id.clone()),
            requested_requests,
            used_requests,
            limit_requests: Some(limit_requests),
            remaining_requests: Some(remaining_requests),
            window_seconds: Some(policy.window_seconds),
            window_start_ms: Some(window_start_ms),
            window_end_ms: Some(window_end_ms),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RateLimitWindowSnapshot {
    pub policy_id: String,
    pub project_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub route_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_name: Option<String>,
    pub requests_per_window: u64,
    pub window_seconds: u64,
    pub burst_requests: u64,
    pub limit_requests: u64,
    pub request_count: u64,
    pub remaining_requests: u64,
    pub window_start_ms: u64,
    pub window_end_ms: u64,
    pub updated_at_ms: u64,
    pub enabled: bool,
    pub exceeded: bool,
}

fn default_window_seconds() -> u64 {
    60
}

fn default_enabled() -> bool {
    true
}
