use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateFileRequest {
    pub purpose: String,
    pub filename: String,
}

impl CreateFileRequest {
    pub fn new(purpose: impl Into<String>, filename: impl Into<String>) -> Self {
        Self {
            purpose: purpose.into(),
            filename: filename.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct FileObject {
    pub id: String,
    pub object: &'static str,
    pub purpose: String,
    pub filename: String,
    pub status: &'static str,
}

impl FileObject {
    pub fn new(
        id: impl Into<String>,
        filename: impl Into<String>,
        purpose: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            object: "file",
            purpose: purpose.into(),
            filename: filename.into(),
            status: "processed",
        }
    }
}
