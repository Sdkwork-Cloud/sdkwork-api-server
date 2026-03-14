use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UsageRecord {
    pub project_id: String,
    pub model: String,
    pub provider: String,
}

impl UsageRecord {
    pub fn new(
        project_id: impl Into<String>,
        model: impl Into<String>,
        provider: impl Into<String>,
    ) -> Self {
        Self {
            project_id: project_id.into(),
            model: model.into(),
            provider: provider.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UsageProjectSummary {
    pub project_id: String,
    pub request_count: u64,
}

impl UsageProjectSummary {
    pub fn new(project_id: impl Into<String>, request_count: u64) -> Self {
        Self {
            project_id: project_id.into(),
            request_count,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UsageProviderSummary {
    pub provider: String,
    pub request_count: u64,
    pub project_count: u64,
}

impl UsageProviderSummary {
    pub fn new(provider: impl Into<String>, request_count: u64, project_count: u64) -> Self {
        Self {
            provider: provider.into(),
            request_count,
            project_count,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UsageModelSummary {
    pub model: String,
    pub request_count: u64,
    pub provider_count: u64,
}

impl UsageModelSummary {
    pub fn new(model: impl Into<String>, request_count: u64, provider_count: u64) -> Self {
        Self {
            model: model.into(),
            request_count,
            provider_count,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UsageSummary {
    pub total_requests: u64,
    pub project_count: u64,
    pub model_count: u64,
    pub provider_count: u64,
    pub projects: Vec<UsageProjectSummary>,
    pub providers: Vec<UsageProviderSummary>,
    pub models: Vec<UsageModelSummary>,
}

impl UsageSummary {
    pub fn empty() -> Self {
        Self {
            total_requests: 0,
            project_count: 0,
            model_count: 0,
            provider_count: 0,
            projects: Vec::new(),
            providers: Vec::new(),
            models: Vec::new(),
        }
    }
}
