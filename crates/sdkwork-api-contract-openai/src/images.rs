use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateImageRequest {
    pub model: String,
    pub prompt: String,
}

impl CreateImageRequest {
    pub fn new(model: impl Into<String>, prompt: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            prompt: prompt.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageUpload {
    pub filename: String,
    pub bytes: Vec<u8>,
    pub content_type: Option<String>,
}

impl ImageUpload {
    pub fn new(filename: impl Into<String>, bytes: Vec<u8>) -> Self {
        Self {
            filename: filename.into(),
            bytes,
            content_type: None,
        }
    }

    pub fn with_content_type(mut self, content_type: impl Into<String>) -> Self {
        self.content_type = Some(content_type.into());
        self
    }
}

#[derive(Debug, Clone)]
pub struct CreateImageEditRequest {
    pub model: Option<String>,
    pub prompt: String,
    pub image: ImageUpload,
    pub mask: Option<ImageUpload>,
    pub n: Option<u32>,
    pub quality: Option<String>,
    pub response_format: Option<String>,
    pub size: Option<String>,
    pub user: Option<String>,
}

impl CreateImageEditRequest {
    pub fn new(prompt: impl Into<String>, image: ImageUpload) -> Self {
        Self {
            model: None,
            prompt: prompt.into(),
            image,
            mask: None,
            n: None,
            quality: None,
            response_format: None,
            size: None,
            user: None,
        }
    }

    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    pub fn with_mask(mut self, mask: ImageUpload) -> Self {
        self.mask = Some(mask);
        self
    }

    pub fn model_or_default(&self) -> &str {
        self.model.as_deref().unwrap_or("gpt-image-1")
    }
}

#[derive(Debug, Clone)]
pub struct CreateImageVariationRequest {
    pub model: Option<String>,
    pub image: ImageUpload,
    pub n: Option<u32>,
    pub response_format: Option<String>,
    pub size: Option<String>,
    pub user: Option<String>,
}

impl CreateImageVariationRequest {
    pub fn new(image: ImageUpload) -> Self {
        Self {
            model: None,
            image,
            n: None,
            response_format: None,
            size: None,
            user: None,
        }
    }

    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    pub fn model_or_default(&self) -> &str {
        self.model.as_deref().unwrap_or("gpt-image-1")
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ImageObject {
    pub b64_json: String,
}

impl ImageObject {
    pub fn base64(payload: impl Into<String>) -> Self {
        Self {
            b64_json: payload.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ImagesResponse {
    pub data: Vec<ImageObject>,
}

impl ImagesResponse {
    pub fn new(data: Vec<ImageObject>) -> Self {
        Self { data }
    }
}
