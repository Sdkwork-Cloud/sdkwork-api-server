use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRealtimeSessionRequest {
    pub model: String,
    pub voice: Option<String>,
}

impl CreateRealtimeSessionRequest {
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            voice: None,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RealtimeSessionObject {
    pub id: String,
    pub object: &'static str,
    pub model: String,
    pub client_secret: Option<RealtimeClientSecret>,
}

impl RealtimeSessionObject {
    pub fn new(id: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            object: "realtime.session",
            model: model.into(),
            client_secret: None,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RealtimeClientSecret {
    pub value: String,
    pub expires_at: u64,
}
