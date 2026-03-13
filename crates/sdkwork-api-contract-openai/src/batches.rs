use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateBatchRequest {
    pub input_file_id: String,
    pub endpoint: String,
    pub completion_window: String,
}

impl CreateBatchRequest {
    pub fn new(
        input_file_id: impl Into<String>,
        endpoint: impl Into<String>,
        completion_window: impl Into<String>,
    ) -> Self {
        Self {
            input_file_id: input_file_id.into(),
            endpoint: endpoint.into(),
            completion_window: completion_window.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct BatchObject {
    pub id: String,
    pub object: &'static str,
    pub endpoint: String,
    pub input_file_id: String,
    pub status: &'static str,
}

impl BatchObject {
    pub fn new(
        id: impl Into<String>,
        endpoint: impl Into<String>,
        input_file_id: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            object: "batch",
            endpoint: endpoint.into(),
            input_file_id: input_file_id.into(),
            status: "validating",
        }
    }
}
