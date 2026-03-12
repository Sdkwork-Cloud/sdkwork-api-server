#[derive(Debug, Clone, PartialEq, Eq)]
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
