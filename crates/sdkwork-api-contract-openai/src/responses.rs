use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ResponseObject {
    pub id: String,
    pub object: &'static str,
    pub model: String,
    pub output: Vec<ResponseOutputItem>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResponseOutputItem {
    pub r#type: &'static str,
}

impl ResponseObject {
    pub fn empty(id: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            object: "response",
            model: model.into(),
            output: Vec::new(),
        }
    }
}
