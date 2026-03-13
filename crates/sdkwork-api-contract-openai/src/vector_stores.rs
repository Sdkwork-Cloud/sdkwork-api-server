use serde::{Deserialize, Serialize};

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
