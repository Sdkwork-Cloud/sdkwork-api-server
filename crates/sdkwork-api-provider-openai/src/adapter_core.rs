use super::*;

pub fn map_model_object(model: &str) -> ModelCatalogEntry {
    ModelCatalogEntry::new(model, "provider-openai-official")
}

#[derive(Debug, Clone)]
pub struct OpenAiProviderAdapter {
    pub(crate) base_url: String,
    pub(crate) client: Client,
    pub(crate) request_headers: HashMap<String, String>,
}

impl OpenAiProviderAdapter {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into().trim_end_matches('/').to_owned(),
            client: Client::new(),
            request_headers: HashMap::new(),
        }
    }

    pub fn with_request_headers(mut self, headers: &HashMap<String, String>) -> Self {
        self.request_headers = headers.clone();
        self
    }
}
