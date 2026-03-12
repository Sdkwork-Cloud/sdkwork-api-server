#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoutingDecision {
    pub selected_provider_id: String,
    pub candidate_ids: Vec<String>,
}

impl RoutingDecision {
    pub fn new(selected_provider_id: impl Into<String>, candidate_ids: Vec<String>) -> Self {
        Self {
            selected_provider_id: selected_provider_id.into(),
            candidate_ids,
        }
    }
}
