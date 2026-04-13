use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommercialPressureScopeKind {
    Project,
    ApiKey,
    Provider,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommercialAdmissionPolicy {
    pub policy_id: String,
    pub project_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key_group_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub route_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requests_per_window: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window_seconds: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub burst_requests: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_concurrency_limit: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key_concurrency_limit: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_concurrency_limit: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token_budget: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub media_job_queue_limit: Option<u64>,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

impl CommercialAdmissionPolicy {
    pub fn new(policy_id: impl Into<String>, project_id: impl Into<String>) -> Self {
        Self {
            policy_id: policy_id.into(),
            project_id: project_id.into(),
            api_key_hash: None,
            api_key_group_id: None,
            route_key: None,
            model_name: None,
            provider_id: None,
            requests_per_window: None,
            window_seconds: None,
            burst_requests: None,
            project_concurrency_limit: None,
            api_key_concurrency_limit: None,
            provider_concurrency_limit: None,
            token_budget: None,
            media_job_queue_limit: None,
            enabled: true,
            notes: None,
            created_at_ms: 0,
            updated_at_ms: 0,
        }
    }

    pub fn with_api_key_hash(mut self, api_key_hash: impl Into<String>) -> Self {
        self.api_key_hash = Some(api_key_hash.into());
        self
    }

    pub fn with_api_key_hash_option(mut self, api_key_hash: Option<String>) -> Self {
        self.api_key_hash = api_key_hash;
        self
    }

    pub fn with_api_key_group_id(mut self, api_key_group_id: impl Into<String>) -> Self {
        self.api_key_group_id = Some(api_key_group_id.into());
        self
    }

    pub fn with_api_key_group_id_option(mut self, api_key_group_id: Option<String>) -> Self {
        self.api_key_group_id = api_key_group_id;
        self
    }

    pub fn with_route_key(mut self, route_key: impl Into<String>) -> Self {
        self.route_key = Some(route_key.into());
        self
    }

    pub fn with_route_key_option(mut self, route_key: Option<String>) -> Self {
        self.route_key = route_key;
        self
    }

    pub fn with_model_name(mut self, model_name: impl Into<String>) -> Self {
        self.model_name = Some(model_name.into());
        self
    }

    pub fn with_model_name_option(mut self, model_name: Option<String>) -> Self {
        self.model_name = model_name;
        self
    }

    pub fn with_provider_id(mut self, provider_id: impl Into<String>) -> Self {
        self.provider_id = Some(provider_id.into());
        self
    }

    pub fn with_provider_id_option(mut self, provider_id: Option<String>) -> Self {
        self.provider_id = provider_id;
        self
    }

    pub fn with_request_window(mut self, requests_per_window: u64, window_seconds: u64) -> Self {
        self.requests_per_window = Some(requests_per_window.max(1));
        self.window_seconds = Some(window_seconds.max(1));
        self
    }

    pub fn with_burst_requests_limit(mut self, burst_requests: u64) -> Self {
        self.burst_requests = Some(burst_requests);
        self
    }

    pub fn with_project_concurrency_limit(mut self, limit: u64) -> Self {
        self.project_concurrency_limit = Some(limit);
        self
    }

    pub fn with_api_key_concurrency_limit(mut self, limit: u64) -> Self {
        self.api_key_concurrency_limit = Some(limit);
        self
    }

    pub fn with_provider_concurrency_limit(mut self, limit: u64) -> Self {
        self.provider_concurrency_limit = Some(limit);
        self
    }

    pub fn with_token_budget(mut self, token_budget: u64) -> Self {
        self.token_budget = Some(token_budget);
        self
    }

    pub fn with_media_job_queue_limit(mut self, media_job_queue_limit: u64) -> Self {
        self.media_job_queue_limit = Some(media_job_queue_limit);
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

    pub fn matches_request_scope(
        &self,
        project_id: &str,
        api_key_hash: &str,
        api_key_group_id: Option<&str>,
        route_key: &str,
        model_name: Option<&str>,
    ) -> bool {
        self.project_id == project_id
            && self.enabled
            && self
                .api_key_hash
                .as_deref()
                .is_none_or(|value| value == api_key_hash)
            && self
                .api_key_group_id
                .as_deref()
                .is_none_or(|value| api_key_group_id == Some(value))
            && self
                .route_key
                .as_deref()
                .is_none_or(|value| value == route_key)
            && self
                .model_name
                .as_deref()
                .is_none_or(|value| model_name == Some(value))
    }

    pub fn matches_provider_scope(
        &self,
        project_id: &str,
        api_key_hash: &str,
        api_key_group_id: Option<&str>,
        route_key: &str,
        model_name: Option<&str>,
        provider_id: &str,
    ) -> bool {
        self.matches_request_scope(
            project_id,
            api_key_hash,
            api_key_group_id,
            route_key,
            model_name,
        ) && self
            .provider_id
            .as_deref()
            .is_none_or(|value| value == provider_id)
    }

    pub fn request_specificity_score(&self) -> u8 {
        self.api_key_hash.iter().count() as u8
            + self.api_key_group_id.iter().count() as u8
            + self.route_key.iter().count() as u8
            + self.model_name.iter().count() as u8
    }

    pub fn provider_specificity_score(&self) -> u8 {
        self.request_specificity_score() + self.provider_id.iter().count() as u8
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrafficPressureSnapshot {
    pub policy_id: String,
    pub project_id: String,
    pub scope_kind: CommercialPressureScopeKind,
    pub scope_key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key_group_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub route_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_id: Option<String>,
    pub current_in_flight: u64,
    pub limit: u64,
    pub remaining: u64,
    pub saturated: bool,
    pub updated_at_ms: u64,
}

fn default_window_seconds() -> u64 {
    60
}

fn default_enabled() -> bool {
    true
}
