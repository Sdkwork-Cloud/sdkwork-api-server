use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct EmbeddingObject {
    pub object: &'static str,
    pub embedding: Vec<f32>,
    pub index: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateEmbeddingResponse {
    pub object: &'static str,
    pub data: Vec<EmbeddingObject>,
    pub model: String,
}

impl CreateEmbeddingResponse {
    pub fn empty(model: impl Into<String>) -> Self {
        Self {
            object: "list",
            data: Vec::new(),
            model: model.into(),
        }
    }
}
