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

#[derive(Debug, Clone, Serialize)]
pub struct ListEvalsResponse {
    pub object: &'static str,
    pub data: Vec<EvalObject>,
}

impl ListEvalsResponse {
    pub fn new(data: Vec<EvalObject>) -> Self {
        Self {
            object: "list",
            data,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateEvalRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

impl UpdateEvalRequest {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: Some(name.into()),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DeleteEvalResponse {
    pub id: String,
    pub object: &'static str,
    pub deleted: bool,
}

impl DeleteEvalResponse {
    pub fn deleted(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            object: "eval.deleted",
            deleted: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEvalRunRequest {
    pub name: String,
}

impl CreateEvalRunRequest {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct EvalRunObject {
    pub id: String,
    pub object: &'static str,
    pub status: &'static str,
}

impl EvalRunObject {
    pub fn queued(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            object: "eval.run",
            status: "queued",
        }
    }

    pub fn completed(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            object: "eval.run",
            status: "completed",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ListEvalRunsResponse {
    pub object: &'static str,
    pub data: Vec<EvalRunObject>,
}

impl ListEvalRunsResponse {
    pub fn new(data: Vec<EvalRunObject>) -> Self {
        Self {
            object: "list",
            data,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DeleteEvalRunResponse {
    pub id: String,
    pub object: &'static str,
    pub deleted: bool,
}

impl DeleteEvalRunResponse {
    pub fn deleted(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            object: "eval.run.deleted",
            deleted: true,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct EvalRunOutputItemObject {
    pub id: String,
    pub object: &'static str,
    pub status: &'static str,
}

impl EvalRunOutputItemObject {
    pub fn passed(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            object: "eval.run.output_item",
            status: "pass",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ListEvalRunOutputItemsResponse {
    pub object: &'static str,
    pub data: Vec<EvalRunOutputItemObject>,
}

impl ListEvalRunOutputItemsResponse {
    pub fn new(data: Vec<EvalRunOutputItemObject>) -> Self {
        Self {
            object: "list",
            data,
        }
    }
}
