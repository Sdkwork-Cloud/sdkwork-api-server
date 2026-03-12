#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Channel {
    pub id: String,
    pub name: String,
}

impl Channel {
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProxyProvider {
    pub id: String,
    pub channel_id: String,
    pub display_name: String,
}

impl ProxyProvider {
    pub fn new(
        id: impl Into<String>,
        channel_id: impl Into<String>,
        display_name: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            channel_id: channel_id.into(),
            display_name: display_name.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelCatalogEntry {
    pub external_name: String,
    pub provider_id: String,
}

impl ModelCatalogEntry {
    pub fn new(external_name: impl Into<String>, provider_id: impl Into<String>) -> Self {
        Self {
            external_name: external_name.into(),
            provider_id: provider_id.into(),
        }
    }
}
