use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateModerationRequest {
    pub model: String,
    pub input: Value,
}

impl CreateModerationRequest {
    pub fn new(model: impl Into<String>, input: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            input: Value::String(input.into()),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ModerationCategoryScores {
    pub violence: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModerationResult {
    pub flagged: bool,
    pub category_scores: ModerationCategoryScores,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModerationResponse {
    pub id: String,
    pub model: String,
    pub results: Vec<ModerationResult>,
}

impl ModerationResponse {
    pub fn flagged(model: impl Into<String>) -> Self {
        Self {
            id: "modr_1".to_owned(),
            model: model.into(),
            results: vec![ModerationResult {
                flagged: true,
                category_scores: ModerationCategoryScores { violence: 1.0 },
            }],
        }
    }
}
