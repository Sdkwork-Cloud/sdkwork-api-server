use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateVectorStoreRequest {
    pub name: String,
}

impl CreateVectorStoreRequest {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateVectorStoreRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

impl UpdateVectorStoreRequest {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: Some(name.into()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateVectorStoreFileRequest {
    pub file_id: String,
}

impl CreateVectorStoreFileRequest {
    pub fn new(file_id: impl Into<String>) -> Self {
        Self {
            file_id: file_id.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateVectorStoreFileBatchRequest {
    pub file_ids: Vec<String>,
}

impl CreateVectorStoreFileBatchRequest {
    pub fn new(file_ids: Vec<impl Into<String>>) -> Self {
        Self {
            file_ids: file_ids.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct VectorStoreObject {
    pub id: String,
    pub object: &'static str,
    pub name: String,
    pub status: &'static str,
}

impl VectorStoreObject {
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            object: "vector_store",
            name: name.into(),
            status: "completed",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ListVectorStoresResponse {
    pub object: &'static str,
    pub data: Vec<VectorStoreObject>,
}

impl ListVectorStoresResponse {
    pub fn new(data: Vec<VectorStoreObject>) -> Self {
        Self {
            object: "list",
            data,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DeleteVectorStoreResponse {
    pub id: String,
    pub object: &'static str,
    pub deleted: bool,
}

impl DeleteVectorStoreResponse {
    pub fn deleted(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            object: "vector_store.deleted",
            deleted: true,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct VectorStoreFileObject {
    pub id: String,
    pub object: &'static str,
    pub status: &'static str,
}

impl VectorStoreFileObject {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            object: "vector_store.file",
            status: "completed",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ListVectorStoreFilesResponse {
    pub object: &'static str,
    pub data: Vec<VectorStoreFileObject>,
}

impl ListVectorStoreFilesResponse {
    pub fn new(data: Vec<VectorStoreFileObject>) -> Self {
        Self {
            object: "list",
            data,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DeleteVectorStoreFileResponse {
    pub id: String,
    pub object: &'static str,
    pub deleted: bool,
}

impl DeleteVectorStoreFileResponse {
    pub fn deleted(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            object: "vector_store.file.deleted",
            deleted: true,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct VectorStoreFileBatchObject {
    pub id: String,
    pub object: &'static str,
    pub status: &'static str,
}

impl VectorStoreFileBatchObject {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            object: "vector_store.file_batch",
            status: "in_progress",
        }
    }

    pub fn cancelled(id: impl Into<String>) -> Self {
        let mut batch = Self::new(id);
        batch.status = "cancelled";
        batch
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchVectorStoreRequest {
    pub query: String,
}

impl SearchVectorStoreRequest {
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct VectorStoreSearchContent {
    pub r#type: &'static str,
    pub text: String,
}

impl VectorStoreSearchContent {
    pub fn text(text: impl Into<String>) -> Self {
        Self {
            r#type: "text",
            text: text.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct VectorStoreSearchResult {
    pub file_id: String,
    pub filename: String,
    pub score: f64,
    pub attributes: BTreeMap<String, serde_json::Value>,
    pub content: Vec<VectorStoreSearchContent>,
}

impl VectorStoreSearchResult {
    pub fn sample(query: impl Into<String>) -> Self {
        Self {
            file_id: "file_1".to_owned(),
            filename: "kb.txt".to_owned(),
            score: 0.98,
            attributes: BTreeMap::new(),
            content: vec![VectorStoreSearchContent::text(query)],
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SearchVectorStoreResponse {
    pub object: &'static str,
    pub data: Vec<VectorStoreSearchResult>,
    pub has_more: bool,
    pub next_page: Option<String>,
    pub search_query: String,
}

impl SearchVectorStoreResponse {
    pub fn sample(query: impl Into<String>) -> Self {
        let query = query.into();
        Self {
            object: "list",
            data: vec![VectorStoreSearchResult::sample(query.clone())],
            has_more: false,
            next_page: None,
            search_query: query,
        }
    }
}
