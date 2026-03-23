use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProxyProvider {
    pub id: String,
    pub channel_id: String,
    pub extension_id: String,
    pub adapter_kind: String,
    pub base_url: String,
    pub display_name: String,
    #[serde(default)]
    pub channel_bindings: Vec<ProviderChannelBinding>,
}

impl ProxyProvider {
    pub fn new(
        id: impl Into<String>,
        channel_id: impl Into<String>,
        adapter_kind: impl Into<String>,
        base_url: impl Into<String>,
        display_name: impl Into<String>,
    ) -> Self {
        let id = id.into();
        let channel_id = channel_id.into();
        let adapter_kind = adapter_kind.into();
        Self {
            channel_bindings: vec![ProviderChannelBinding::primary(
                id.clone(),
                channel_id.clone(),
            )],
            id,
            channel_id,
            extension_id: derive_provider_extension_id(&adapter_kind),
            adapter_kind,
            base_url: base_url.into(),
            display_name: display_name.into(),
        }
    }

    pub fn with_extension_id(mut self, extension_id: impl Into<String>) -> Self {
        self.extension_id =
            normalize_provider_extension_id(extension_id.into(), &self.adapter_kind);
        self
    }

    pub fn with_channel_binding(mut self, binding: ProviderChannelBinding) -> Self {
        let mut binding = binding.with_provider_id(self.id.clone());
        binding.is_primary = binding.channel_id == self.channel_id;

        if let Some(existing) = self
            .channel_bindings
            .iter_mut()
            .find(|existing| existing.channel_id == binding.channel_id)
        {
            *existing = binding;
        } else {
            self.channel_bindings.push(binding);
        }

        self.channel_bindings
            .sort_by_key(|binding| (!binding.is_primary, binding.channel_id.clone()));
        self
    }
}

pub fn derive_provider_extension_id(adapter_kind: &str) -> String {
    match adapter_kind {
        "openai" | "openai-compatible" | "custom-openai" => {
            "sdkwork.provider.openai.official".to_owned()
        }
        "openrouter" | "openrouter-compatible" => "sdkwork.provider.openrouter".to_owned(),
        "ollama" | "ollama-compatible" => "sdkwork.provider.ollama".to_owned(),
        _ => format!(
            "sdkwork.provider.{}",
            sanitize_provider_identity_segment(adapter_kind)
        ),
    }
}

pub fn normalize_provider_extension_id(
    extension_id: impl Into<String>,
    adapter_kind: &str,
) -> String {
    let extension_id = extension_id.into();
    if extension_id.trim().is_empty() {
        derive_provider_extension_id(adapter_kind)
    } else {
        extension_id
    }
}

fn sanitize_provider_identity_segment(adapter_kind: &str) -> String {
    let mut sanitized = String::with_capacity(adapter_kind.len());
    let mut previous_dash = false;

    for ch in adapter_kind.chars() {
        if ch.is_ascii_alphanumeric() {
            sanitized.push(ch.to_ascii_lowercase());
            previous_dash = false;
        } else if !previous_dash {
            sanitized.push('-');
            previous_dash = true;
        }
    }

    let sanitized = sanitized.trim_matches('-');
    if sanitized.is_empty() {
        "custom".to_owned()
    } else {
        sanitized.to_owned()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderChannelBinding {
    pub provider_id: String,
    pub channel_id: String,
    #[serde(default)]
    pub is_primary: bool,
}

impl ProviderChannelBinding {
    pub fn new(provider_id: impl Into<String>, channel_id: impl Into<String>) -> Self {
        Self {
            provider_id: provider_id.into(),
            channel_id: channel_id.into(),
            is_primary: false,
        }
    }

    pub fn primary(provider_id: impl Into<String>, channel_id: impl Into<String>) -> Self {
        Self {
            is_primary: true,
            ..Self::new(provider_id, channel_id)
        }
    }

    pub fn with_provider_id(mut self, provider_id: impl Into<String>) -> Self {
        self.provider_id = provider_id.into();
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelCapability {
    ChatCompletions,
    Responses,
    Embeddings,
    Completions,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChannelModelRecord {
    pub channel_id: String,
    pub model_id: String,
    pub model_display_name: String,
    #[serde(default)]
    pub capabilities: Vec<ModelCapability>,
    #[serde(default)]
    pub streaming: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context_window: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl ChannelModelRecord {
    pub fn new(
        channel_id: impl Into<String>,
        model_id: impl Into<String>,
        model_display_name: impl Into<String>,
    ) -> Self {
        Self {
            channel_id: channel_id.into(),
            model_id: model_id.into(),
            model_display_name: model_display_name.into(),
            capabilities: Vec::new(),
            streaming: false,
            context_window: None,
            description: None,
        }
    }

    pub fn with_capability(mut self, capability: ModelCapability) -> Self {
        if !self.capabilities.contains(&capability) {
            self.capabilities.push(capability);
        }
        self
    }

    pub fn with_streaming(mut self, streaming: bool) -> Self {
        self.streaming = streaming;
        self
    }

    pub fn with_context_window(mut self, context_window: u64) -> Self {
        self.context_window = Some(context_window);
        self
    }

    pub fn with_context_window_option(mut self, context_window: Option<u64>) -> Self {
        self.context_window = context_window;
        self
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn with_description_option(mut self, description: Option<String>) -> Self {
        self.description = description;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelPriceRecord {
    pub channel_id: String,
    pub model_id: String,
    pub proxy_provider_id: String,
    pub currency_code: String,
    pub price_unit: String,
    pub input_price: f64,
    pub output_price: f64,
    pub cache_read_price: f64,
    pub cache_write_price: f64,
    pub request_price: f64,
    pub is_active: bool,
}

impl ModelPriceRecord {
    pub fn new(
        channel_id: impl Into<String>,
        model_id: impl Into<String>,
        proxy_provider_id: impl Into<String>,
    ) -> Self {
        Self {
            channel_id: channel_id.into(),
            model_id: model_id.into(),
            proxy_provider_id: proxy_provider_id.into(),
            currency_code: "USD".to_owned(),
            price_unit: "per_1m_tokens".to_owned(),
            input_price: 0.0,
            output_price: 0.0,
            cache_read_price: 0.0,
            cache_write_price: 0.0,
            request_price: 0.0,
            is_active: true,
        }
    }

    pub fn with_currency_code(mut self, currency_code: impl Into<String>) -> Self {
        self.currency_code = currency_code.into();
        self
    }

    pub fn with_price_unit(mut self, price_unit: impl Into<String>) -> Self {
        self.price_unit = price_unit.into();
        self
    }

    pub fn with_input_price(mut self, input_price: f64) -> Self {
        self.input_price = input_price;
        self
    }

    pub fn with_output_price(mut self, output_price: f64) -> Self {
        self.output_price = output_price;
        self
    }

    pub fn with_cache_read_price(mut self, cache_read_price: f64) -> Self {
        self.cache_read_price = cache_read_price;
        self
    }

    pub fn with_cache_write_price(mut self, cache_write_price: f64) -> Self {
        self.cache_write_price = cache_write_price;
        self
    }

    pub fn with_request_price(mut self, request_price: f64) -> Self {
        self.request_price = request_price;
        self
    }

    pub fn with_active(mut self, is_active: bool) -> Self {
        self.is_active = is_active;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelCatalogEntry {
    pub external_name: String,
    pub provider_id: String,
    #[serde(default)]
    pub capabilities: Vec<ModelCapability>,
    #[serde(default)]
    pub streaming: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context_window: Option<u64>,
}

impl ModelCatalogEntry {
    pub fn new(external_name: impl Into<String>, provider_id: impl Into<String>) -> Self {
        Self {
            external_name: external_name.into(),
            provider_id: provider_id.into(),
            capabilities: Vec::new(),
            streaming: false,
            context_window: None,
        }
    }

    pub fn with_capability(mut self, capability: ModelCapability) -> Self {
        if !self.capabilities.contains(&capability) {
            self.capabilities.push(capability);
        }
        self
    }

    pub fn with_streaming(mut self, streaming: bool) -> Self {
        self.streaming = streaming;
        self
    }

    pub fn with_context_window(mut self, context_window: u64) -> Self {
        self.context_window = Some(context_window);
        self
    }
}

pub type ModelVariant = ModelCatalogEntry;
