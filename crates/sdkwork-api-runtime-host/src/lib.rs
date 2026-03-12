use anyhow::Result;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmbeddedRuntime {
    base_url: String,
}

impl EmbeddedRuntime {
    pub async fn start_ephemeral() -> Result<Self> {
        Ok(Self {
            base_url: "http://127.0.0.1:3001".to_owned(),
        })
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}
