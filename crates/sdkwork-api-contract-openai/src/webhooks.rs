use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWebhookRequest {
    pub url: String,
    pub events: Vec<String>,
}

impl CreateWebhookRequest {
    pub fn new<I, S>(url: impl Into<String>, events: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            url: url.into(),
            events: events.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct WebhookObject {
    pub id: String,
    pub object: &'static str,
    pub url: String,
    pub status: &'static str,
}

impl WebhookObject {
    pub fn new(id: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            object: "webhook_endpoint",
            url: url.into(),
            status: "enabled",
        }
    }
}
