use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateFineTuningJobRequest {
    pub training_file: String,
    pub model: String,
}

impl CreateFineTuningJobRequest {
    pub fn new(training_file: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            training_file: training_file.into(),
            model: model.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct FineTuningJobObject {
    pub id: String,
    pub object: &'static str,
    pub model: String,
    pub status: &'static str,
}

impl FineTuningJobObject {
    pub fn new(id: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            object: "fine_tuning.job",
            model: model.into(),
            status: "queued",
        }
    }
}
