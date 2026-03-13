use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEvalRequest {
    pub name: String,
    pub data_source_config: EvalDataSourceConfig,
}

impl CreateEvalRequest {
    pub fn new(name: impl Into<String>, file_id: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            data_source_config: EvalDataSourceConfig::file(file_id),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalDataSourceConfig {
    pub r#type: String,
    pub file_id: String,
}

impl EvalDataSourceConfig {
    pub fn file(file_id: impl Into<String>) -> Self {
        Self {
            r#type: "file".to_owned(),
            file_id: file_id.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct EvalObject {
    pub id: String,
    pub object: &'static str,
    pub name: String,
    pub status: &'static str,
}

impl EvalObject {
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            object: "eval",
            name: name.into(),
            status: "queued",
        }
    }
}
