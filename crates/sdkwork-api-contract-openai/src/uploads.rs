use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUploadRequest {
    pub purpose: String,
    pub filename: String,
    pub bytes: u64,
}

impl CreateUploadRequest {
    pub fn new(purpose: impl Into<String>, filename: impl Into<String>, bytes: u64) -> Self {
        Self {
            purpose: purpose.into(),
            filename: filename.into(),
            bytes,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct UploadObject {
    pub id: String,
    pub object: &'static str,
    pub purpose: String,
    pub filename: String,
    pub status: &'static str,
}

impl UploadObject {
    pub fn new(
        id: impl Into<String>,
        filename: impl Into<String>,
        purpose: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            object: "upload",
            purpose: purpose.into(),
            filename: filename.into(),
            status: "pending",
        }
    }
}
