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
