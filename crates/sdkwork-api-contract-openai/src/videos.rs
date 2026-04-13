use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateVideoRequest {
    pub model: String,
    pub prompt: String,
}

impl CreateVideoRequest {
    pub fn new(model: impl Into<String>, prompt: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            prompt: prompt.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemixVideoRequest {
    pub prompt: String,
}

impl RemixVideoRequest {
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            prompt: prompt.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtendVideoRequest {
    pub prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub video_id: Option<String>,
}

impl ExtendVideoRequest {
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            prompt: prompt.into(),
            video_id: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateVideoCharacterRequest {
    pub name: String,
    pub video_id: String,
}

impl CreateVideoCharacterRequest {
    pub fn new(name: impl Into<String>, video_id: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            video_id: video_id.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditVideoRequest {
    pub prompt: String,
    pub video_id: String,
}

impl EditVideoRequest {
    pub fn new(prompt: impl Into<String>, video_id: impl Into<String>) -> Self {
        Self {
            prompt: prompt.into(),
            video_id: video_id.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateVideoCharacterRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
}

impl UpdateVideoCharacterRequest {
    pub fn new(name: impl Into<String>, prompt: impl Into<String>) -> Self {
        Self {
            name: Some(name.into()),
            prompt: Some(prompt.into()),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct VideoObject {
    pub id: String,
    pub object: &'static str,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_seconds: Option<f64>,
}

impl VideoObject {
    pub fn new(id: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            object: "video",
            url: url.into(),
            status: None,
            duration_seconds: None,
        }
    }

    pub fn with_status(mut self, status: impl Into<String>) -> Self {
        self.status = Some(status.into());
        self
    }

    pub fn with_duration_seconds(mut self, duration_seconds: f64) -> Self {
        self.duration_seconds = Some(duration_seconds);
        self
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct VideosResponse {
    pub object: &'static str,
    pub data: Vec<VideoObject>,
}

impl VideosResponse {
    pub fn new(data: Vec<VideoObject>) -> Self {
        Self {
            object: "list",
            data,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct VideoCharacterObject {
    pub id: String,
    pub object: &'static str,
    pub name: String,
}

impl VideoCharacterObject {
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            object: "video.character",
            name: name.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct VideoCharactersResponse {
    pub object: &'static str,
    pub data: Vec<VideoCharacterObject>,
}

impl VideoCharactersResponse {
    pub fn new(data: Vec<VideoCharacterObject>) -> Self {
        Self {
            object: "list",
            data,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DeleteVideoResponse {
    pub id: String,
    pub object: &'static str,
    pub deleted: bool,
}

impl DeleteVideoResponse {
    pub fn deleted(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            object: "video.deleted",
            deleted: true,
        }
    }
}
