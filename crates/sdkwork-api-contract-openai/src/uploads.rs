use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUploadRequest {
    pub purpose: String,
    pub filename: String,
    pub mime_type: String,
    pub bytes: u64,
}

impl CreateUploadRequest {
    pub fn new(
        purpose: impl Into<String>,
        filename: impl Into<String>,
        mime_type: impl Into<String>,
        bytes: u64,
    ) -> Self {
        Self {
            purpose: purpose.into(),
            filename: filename.into(),
            mime_type: mime_type.into(),
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
    pub mime_type: String,
    pub bytes: u64,
    pub part_ids: Vec<String>,
    pub status: &'static str,
}

impl UploadObject {
    pub fn new(
        id: impl Into<String>,
        filename: impl Into<String>,
        purpose: impl Into<String>,
    ) -> Self {
        Self::with_details(id, filename, purpose, "application/octet-stream", 0, vec![])
    }

    pub fn with_details(
        id: impl Into<String>,
        filename: impl Into<String>,
        purpose: impl Into<String>,
        mime_type: impl Into<String>,
        bytes: u64,
        part_ids: Vec<String>,
    ) -> Self {
        Self {
            id: id.into(),
            object: "upload",
            purpose: purpose.into(),
            filename: filename.into(),
            mime_type: mime_type.into(),
            bytes,
            part_ids,
            status: "pending",
        }
    }

    pub fn completed(
        id: impl Into<String>,
        filename: impl Into<String>,
        purpose: impl Into<String>,
        mime_type: impl Into<String>,
        bytes: u64,
        part_ids: Vec<String>,
    ) -> Self {
        let mut upload = Self::with_details(id, filename, purpose, mime_type, bytes, part_ids);
        upload.status = "completed";
        upload
    }

    pub fn cancelled(
        id: impl Into<String>,
        filename: impl Into<String>,
        purpose: impl Into<String>,
        mime_type: impl Into<String>,
        bytes: u64,
        part_ids: Vec<String>,
    ) -> Self {
        let mut upload = Self::with_details(id, filename, purpose, mime_type, bytes, part_ids);
        upload.status = "cancelled";
        upload
    }
}

#[derive(Debug, Clone)]
pub struct AddUploadPartRequest {
    pub upload_id: String,
    pub data: Vec<u8>,
    pub filename: Option<String>,
    pub content_type: Option<String>,
}

impl AddUploadPartRequest {
    pub fn new(upload_id: impl Into<String>, data: Vec<u8>) -> Self {
        Self {
            upload_id: upload_id.into(),
            data,
            filename: None,
            content_type: None,
        }
    }

    pub fn with_filename(mut self, filename: impl Into<String>) -> Self {
        self.filename = Some(filename.into());
        self
    }

    pub fn with_content_type(mut self, content_type: impl Into<String>) -> Self {
        self.content_type = Some(content_type.into());
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompleteUploadRequest {
    #[serde(skip)]
    pub upload_id: String,
    pub part_ids: Vec<String>,
}

impl CompleteUploadRequest {
    pub fn new<S, I>(upload_id: impl Into<String>, part_ids: I) -> Self
    where
        S: Into<String>,
        I: IntoIterator<Item = S>,
    {
        Self {
            upload_id: upload_id.into(),
            part_ids: part_ids.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct UploadPartObject {
    pub id: String,
    pub object: &'static str,
    pub upload_id: String,
    pub status: &'static str,
}

impl UploadPartObject {
    pub fn new(id: impl Into<String>, upload_id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            object: "upload.part",
            upload_id: upload_id.into(),
            status: "completed",
        }
    }
}
