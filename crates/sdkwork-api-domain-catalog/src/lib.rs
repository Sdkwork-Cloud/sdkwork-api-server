use serde::{Deserialize, Serialize};
use std::str::FromStr;
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct ProxyProvider {
    pub id: String,
    pub channel_id: String,
    pub extension_id: String,
    pub adapter_kind: String,
    #[serde(default)]
    pub protocol_kind: String,
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
        let protocol_kind = derive_provider_protocol_kind(&adapter_kind).to_owned();
        Self {
            channel_bindings: vec![ProviderChannelBinding::primary(
                id.clone(),
                channel_id.clone(),
            )],
            id,
            channel_id,
            extension_id: derive_provider_extension_id(&adapter_kind),
            adapter_kind,
            protocol_kind,
            base_url: base_url.into(),
            display_name: display_name.into(),
        }
    }

    pub fn with_extension_id(mut self, extension_id: impl Into<String>) -> Self {
        self.extension_id =
            normalize_provider_extension_id(extension_id.into(), &self.adapter_kind);
        self
    }

    pub fn with_protocol_kind(mut self, protocol_kind: impl Into<String>) -> Self {
        self.protocol_kind =
            normalize_provider_protocol_kind(protocol_kind.into(), &self.adapter_kind);
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

    pub fn protocol_kind(&self) -> &str {
        if self.protocol_kind.trim().is_empty() {
            derive_provider_protocol_kind(&self.adapter_kind)
        } else {
            &self.protocol_kind
        }
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

pub fn derive_provider_protocol_kind(adapter_kind: &str) -> &'static str {
    match adapter_kind {
        "openai"
        | "openai-compatible"
        | "custom-openai"
        | "openrouter"
        | "openrouter-compatible"
        | "siliconflow"
        | "siliconflow-compatible" => "openai",
        "anthropic" | "anthropic-compatible" | "claude" | "claude-compatible" => "anthropic",
        "gemini" | "gemini-compatible" => "gemini",
        "ollama" | "ollama-compatible" => "custom",
        _ => "custom",
    }
}

pub fn derive_provider_default_plugin_family(adapter_kind: &str) -> Option<&'static str> {
    match adapter_kind {
        "openrouter" | "openrouter-compatible" => Some("openrouter"),
        "siliconflow" | "siliconflow-compatible" => Some("siliconflow"),
        "ollama" | "ollama-compatible" => Some("ollama"),
        _ => None,
    }
}

pub fn normalize_provider_default_plugin_family(
    default_plugin_family: impl AsRef<str>,
) -> Option<&'static str> {
    match default_plugin_family
        .as_ref()
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "openrouter" | "openrouter-compatible" => Some("openrouter"),
        "siliconflow" | "siliconflow-compatible" => Some("siliconflow"),
        "ollama" | "ollama-compatible" => Some("ollama"),
        _ => None,
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

pub fn normalize_provider_protocol_kind(
    protocol_kind: impl Into<String>,
    adapter_kind: &str,
) -> String {
    let protocol_kind = protocol_kind.into();
    let protocol_kind = protocol_kind.trim().to_ascii_lowercase();
    if protocol_kind.is_empty() {
        derive_provider_protocol_kind(adapter_kind).to_owned()
    } else {
        protocol_kind
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ModelCapability {
    ChatCompletions,
    Responses,
    Embeddings,
    Completions,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct ProviderModelRecord {
    pub proxy_provider_id: String,
    pub channel_id: String,
    pub model_id: String,
    pub provider_model_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_model_family: Option<String>,
    #[serde(default)]
    pub capabilities: Vec<ModelCapability>,
    #[serde(default)]
    pub streaming: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context_window: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<u64>,
    #[serde(default)]
    pub supports_prompt_caching: bool,
    #[serde(default)]
    pub supports_reasoning_usage: bool,
    #[serde(default)]
    pub supports_tool_usage_metrics: bool,
    #[serde(default)]
    pub is_default_route: bool,
    #[serde(default = "provider_model_default_active")]
    pub is_active: bool,
}

impl ProviderModelRecord {
    pub fn new(
        proxy_provider_id: impl Into<String>,
        channel_id: impl Into<String>,
        model_id: impl Into<String>,
    ) -> Self {
        let model_id = model_id.into();
        Self {
            proxy_provider_id: proxy_provider_id.into(),
            channel_id: channel_id.into(),
            provider_model_id: model_id.clone(),
            model_id,
            provider_model_family: None,
            capabilities: Vec::new(),
            streaming: false,
            context_window: None,
            max_output_tokens: None,
            supports_prompt_caching: false,
            supports_reasoning_usage: false,
            supports_tool_usage_metrics: false,
            is_default_route: false,
            is_active: true,
        }
    }

    pub fn with_provider_model_id(mut self, provider_model_id: impl Into<String>) -> Self {
        self.provider_model_id = provider_model_id.into();
        self
    }

    pub fn with_provider_model_family(mut self, provider_model_family: impl Into<String>) -> Self {
        self.provider_model_family = Some(provider_model_family.into());
        self
    }

    pub fn with_provider_model_family_option(
        mut self,
        provider_model_family: Option<String>,
    ) -> Self {
        self.provider_model_family = provider_model_family;
        self
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

    pub fn with_max_output_tokens(mut self, max_output_tokens: u64) -> Self {
        self.max_output_tokens = Some(max_output_tokens);
        self
    }

    pub fn with_max_output_tokens_option(mut self, max_output_tokens: Option<u64>) -> Self {
        self.max_output_tokens = max_output_tokens;
        self
    }

    pub fn with_supports_prompt_caching(mut self, supports_prompt_caching: bool) -> Self {
        self.supports_prompt_caching = supports_prompt_caching;
        self
    }

    pub fn with_supports_reasoning_usage(mut self, supports_reasoning_usage: bool) -> Self {
        self.supports_reasoning_usage = supports_reasoning_usage;
        self
    }

    pub fn with_supports_tool_usage_metrics(mut self, supports_tool_usage_metrics: bool) -> Self {
        self.supports_tool_usage_metrics = supports_tool_usage_metrics;
        self
    }

    pub fn with_default_route(mut self, is_default_route: bool) -> Self {
        self.is_default_route = is_default_route;
        self
    }

    pub fn with_active(mut self, is_active: bool) -> Self {
        self.is_active = is_active;
        self
    }
}

fn provider_model_default_active() -> bool {
    true
}

fn provider_account_default_owner_scope() -> String {
    "platform".to_owned()
}

fn provider_account_default_weight() -> u32 {
    1
}

fn provider_account_default_enabled() -> bool {
    true
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct ProviderAccountRecord {
    pub provider_account_id: String,
    pub provider_id: String,
    pub display_name: String,
    pub account_kind: String,
    #[serde(default = "provider_account_default_owner_scope")]
    pub owner_scope: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner_tenant_id: Option<String>,
    pub execution_instance_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_url_override: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
    #[serde(default)]
    pub priority: i32,
    #[serde(default = "provider_account_default_weight")]
    pub weight: u32,
    #[serde(default = "provider_account_default_enabled")]
    pub enabled: bool,
    #[serde(default)]
    pub routing_tags: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub health_score_hint: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latency_ms_hint: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cost_hint: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub success_rate_hint: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub throughput_hint: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_concurrency: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub daily_budget: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

impl ProviderAccountRecord {
    pub fn new(
        provider_account_id: impl Into<String>,
        provider_id: impl Into<String>,
        display_name: impl Into<String>,
        account_kind: impl Into<String>,
        execution_instance_id: impl Into<String>,
    ) -> Self {
        Self {
            provider_account_id: provider_account_id.into(),
            provider_id: provider_id.into(),
            display_name: display_name.into(),
            account_kind: account_kind.into(),
            owner_scope: provider_account_default_owner_scope(),
            owner_tenant_id: None,
            execution_instance_id: execution_instance_id.into(),
            base_url_override: None,
            region: None,
            priority: 0,
            weight: provider_account_default_weight(),
            enabled: provider_account_default_enabled(),
            routing_tags: Vec::new(),
            health_score_hint: None,
            latency_ms_hint: None,
            cost_hint: None,
            success_rate_hint: None,
            throughput_hint: None,
            max_concurrency: None,
            daily_budget: None,
            notes: None,
        }
    }

    pub fn with_owner_scope(mut self, owner_scope: impl Into<String>) -> Self {
        self.owner_scope = owner_scope.into();
        self
    }

    pub fn with_owner_tenant_id(mut self, owner_tenant_id: impl Into<String>) -> Self {
        self.owner_tenant_id = Some(owner_tenant_id.into());
        self
    }

    pub fn with_owner_tenant_id_option(mut self, owner_tenant_id: Option<String>) -> Self {
        self.owner_tenant_id = owner_tenant_id;
        self
    }

    pub fn with_base_url_override(mut self, base_url_override: impl Into<String>) -> Self {
        self.base_url_override = Some(base_url_override.into());
        self
    }

    pub fn with_base_url_override_option(mut self, base_url_override: Option<String>) -> Self {
        self.base_url_override = base_url_override;
        self
    }

    pub fn with_region(mut self, region: impl Into<String>) -> Self {
        self.region = Some(region.into());
        self
    }

    pub fn with_region_option(mut self, region: Option<String>) -> Self {
        self.region = region;
        self
    }

    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_weight(mut self, weight: u32) -> Self {
        self.weight = weight;
        self
    }

    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn with_routing_tag(mut self, routing_tag: impl Into<String>) -> Self {
        self.routing_tags.push(routing_tag.into());
        self
    }

    pub fn with_routing_tags(mut self, routing_tags: Vec<String>) -> Self {
        self.routing_tags = routing_tags;
        self
    }

    pub fn with_health_score_hint(mut self, health_score_hint: f64) -> Self {
        self.health_score_hint = Some(health_score_hint);
        self
    }

    pub fn with_health_score_hint_option(mut self, health_score_hint: Option<f64>) -> Self {
        self.health_score_hint = health_score_hint;
        self
    }

    pub fn with_latency_ms_hint(mut self, latency_ms_hint: u64) -> Self {
        self.latency_ms_hint = Some(latency_ms_hint);
        self
    }

    pub fn with_latency_ms_hint_option(mut self, latency_ms_hint: Option<u64>) -> Self {
        self.latency_ms_hint = latency_ms_hint;
        self
    }

    pub fn with_cost_hint(mut self, cost_hint: f64) -> Self {
        self.cost_hint = Some(cost_hint);
        self
    }

    pub fn with_cost_hint_option(mut self, cost_hint: Option<f64>) -> Self {
        self.cost_hint = cost_hint;
        self
    }

    pub fn with_success_rate_hint(mut self, success_rate_hint: f64) -> Self {
        self.success_rate_hint = Some(success_rate_hint);
        self
    }

    pub fn with_success_rate_hint_option(mut self, success_rate_hint: Option<f64>) -> Self {
        self.success_rate_hint = success_rate_hint;
        self
    }

    pub fn with_throughput_hint(mut self, throughput_hint: f64) -> Self {
        self.throughput_hint = Some(throughput_hint);
        self
    }

    pub fn with_throughput_hint_option(mut self, throughput_hint: Option<f64>) -> Self {
        self.throughput_hint = throughput_hint;
        self
    }

    pub fn with_max_concurrency(mut self, max_concurrency: u32) -> Self {
        self.max_concurrency = Some(max_concurrency);
        self
    }

    pub fn with_max_concurrency_option(mut self, max_concurrency: Option<u32>) -> Self {
        self.max_concurrency = max_concurrency;
        self
    }

    pub fn with_daily_budget(mut self, daily_budget: f64) -> Self {
        self.daily_budget = Some(daily_budget);
        self
    }

    pub fn with_daily_budget_option(mut self, daily_budget: Option<f64>) -> Self {
        self.daily_budget = daily_budget;
        self
    }

    pub fn with_notes(mut self, notes: impl Into<String>) -> Self {
        self.notes = Some(notes.into());
        self
    }

    pub fn with_notes_option(mut self, notes: Option<String>) -> Self {
        self.notes = notes;
        self
    }
}

fn model_price_default_currency() -> String {
    "USD".to_owned()
}

fn model_price_default_unit() -> String {
    "per_1m_tokens".to_owned()
}

fn model_price_default_source_kind() -> String {
    "reference".to_owned()
}

fn model_price_default_condition_kind() -> String {
    "default".to_owned()
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct ModelPriceTier {
    pub tier_id: String,
    #[serde(default)]
    pub display_name: String,
    #[serde(default = "model_price_default_condition_kind")]
    pub condition_kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_input_tokens: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_input_tokens: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub modality: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache_ttl: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(default = "model_price_default_currency")]
    pub currency_code: String,
    #[serde(default = "model_price_default_unit")]
    pub price_unit: String,
    #[serde(default)]
    pub input_price: f64,
    #[serde(default)]
    pub output_price: f64,
    #[serde(default)]
    pub cache_read_price: f64,
    #[serde(default)]
    pub cache_write_price: f64,
    #[serde(default)]
    pub request_price: f64,
}

impl ModelPriceTier {
    pub fn new(tier_id: impl Into<String>, display_name: impl Into<String>) -> Self {
        Self {
            tier_id: tier_id.into(),
            display_name: display_name.into(),
            condition_kind: model_price_default_condition_kind(),
            min_input_tokens: None,
            max_input_tokens: None,
            modality: None,
            cache_ttl: None,
            notes: None,
            currency_code: model_price_default_currency(),
            price_unit: model_price_default_unit(),
            input_price: 0.0,
            output_price: 0.0,
            cache_read_price: 0.0,
            cache_write_price: 0.0,
            request_price: 0.0,
        }
    }

    pub fn with_condition_kind(mut self, condition_kind: impl Into<String>) -> Self {
        self.condition_kind = condition_kind.into();
        self
    }

    pub fn with_min_input_tokens_option(mut self, min_input_tokens: Option<u64>) -> Self {
        self.min_input_tokens = min_input_tokens;
        self
    }

    pub fn with_max_input_tokens_option(mut self, max_input_tokens: Option<u64>) -> Self {
        self.max_input_tokens = max_input_tokens;
        self
    }

    pub fn with_modality_option(mut self, modality: Option<String>) -> Self {
        self.modality = modality;
        self
    }

    pub fn with_cache_ttl_option(mut self, cache_ttl: Option<String>) -> Self {
        self.cache_ttl = cache_ttl;
        self
    }

    pub fn with_notes_option(mut self, notes: Option<String>) -> Self {
        self.notes = notes;
        self
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
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
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
    #[serde(default = "model_price_default_source_kind")]
    pub price_source_kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub billing_notes: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pricing_tiers: Vec<ModelPriceTier>,
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
            currency_code: model_price_default_currency(),
            price_unit: model_price_default_unit(),
            input_price: 0.0,
            output_price: 0.0,
            cache_read_price: 0.0,
            cache_write_price: 0.0,
            request_price: 0.0,
            price_source_kind: model_price_default_source_kind(),
            billing_notes: None,
            pricing_tiers: Vec::new(),
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

    pub fn with_price_source_kind(mut self, price_source_kind: impl Into<String>) -> Self {
        self.price_source_kind = price_source_kind.into();
        self
    }

    pub fn with_billing_notes_option(mut self, billing_notes: Option<String>) -> Self {
        self.billing_notes = billing_notes;
        self
    }

    pub fn with_pricing_tiers(mut self, pricing_tiers: Vec<ModelPriceTier>) -> Self {
        self.pricing_tiers = pricing_tiers;
        self
    }

    pub fn with_active(mut self, is_active: bool) -> Self {
        self.is_active = is_active;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ApiProductKind {
    SubscriptionPlan,
    RechargePack,
    CustomRecharge,
}

impl ApiProductKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SubscriptionPlan => "subscription_plan",
            Self::RechargePack => "recharge_pack",
            Self::CustomRecharge => "custom_recharge",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum QuoteKind {
    ProductPurchase,
    CouponRedemption,
}

impl QuoteKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ProductPurchase => "product_purchase",
            Self::CouponRedemption => "coupon_redemption",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum CatalogPublicationKind {
    PortalCatalog,
    PublicApi,
}

impl CatalogPublicationKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PortalCatalog => "portal_catalog",
            Self::PublicApi => "public_api",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum CatalogPublicationStatus {
    Draft,
    Published,
    Archived,
}

impl CatalogPublicationStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Published => "published",
            Self::Archived => "archived",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum CatalogPublicationLifecycleAction {
    Publish,
    Schedule,
    Retire,
}

impl CatalogPublicationLifecycleAction {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Publish => "publish",
            Self::Schedule => "schedule",
            Self::Retire => "retire",
        }
    }
}

impl FromStr for CatalogPublicationLifecycleAction {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "publish" => Ok(Self::Publish),
            "schedule" => Ok(Self::Schedule),
            "retire" => Ok(Self::Retire),
            other => Err(format!(
                "unknown catalog publication lifecycle action: {other}"
            )),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum CatalogPublicationLifecycleAuditOutcome {
    Applied,
    Rejected,
}

impl CatalogPublicationLifecycleAuditOutcome {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Applied => "applied",
            Self::Rejected => "rejected",
        }
    }
}

impl FromStr for CatalogPublicationLifecycleAuditOutcome {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "applied" => Ok(Self::Applied),
            "rejected" => Ok(Self::Rejected),
            other => Err(format!(
                "unknown catalog publication lifecycle audit outcome: {other}"
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct ApiProduct {
    pub product_id: String,
    pub product_kind: ApiProductKind,
    pub target_id: String,
    pub display_name: String,
    pub source: String,
}

impl ApiProduct {
    pub fn new(
        product_id: impl Into<String>,
        product_kind: ApiProductKind,
        target_id: impl Into<String>,
        display_name: impl Into<String>,
    ) -> Self {
        Self {
            product_id: product_id.into(),
            product_kind,
            target_id: target_id.into(),
            display_name: display_name.into(),
            source: String::new(),
        }
    }

    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = source.into();
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct ProductOffer {
    pub offer_id: String,
    pub product_id: String,
    pub product_kind: ApiProductKind,
    pub display_name: String,
    pub quote_kind: QuoteKind,
    pub quote_target_kind: ApiProductKind,
    pub quote_target_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pricing_plan_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pricing_plan_version: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pricing_rate_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pricing_metric_code: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub price_label: Option<String>,
    pub source: String,
}

impl ProductOffer {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        offer_id: impl Into<String>,
        product_id: impl Into<String>,
        product_kind: ApiProductKind,
        display_name: impl Into<String>,
        quote_kind: QuoteKind,
        quote_target_kind: ApiProductKind,
        quote_target_id: impl Into<String>,
    ) -> Self {
        Self {
            offer_id: offer_id.into(),
            product_id: product_id.into(),
            product_kind,
            display_name: display_name.into(),
            quote_kind,
            quote_target_kind,
            quote_target_id: quote_target_id.into(),
            pricing_plan_id: None,
            pricing_plan_version: None,
            pricing_rate_id: None,
            pricing_metric_code: None,
            price_label: None,
            source: String::new(),
        }
    }

    pub fn with_pricing_plan_id_option(mut self, pricing_plan_id: Option<String>) -> Self {
        self.pricing_plan_id = pricing_plan_id;
        self
    }

    pub fn with_pricing_plan_version_option(mut self, pricing_plan_version: Option<u64>) -> Self {
        self.pricing_plan_version = pricing_plan_version;
        self
    }

    pub fn with_pricing_rate_id_option(mut self, pricing_rate_id: Option<String>) -> Self {
        self.pricing_rate_id = pricing_rate_id;
        self
    }

    pub fn with_pricing_metric_code_option(mut self, pricing_metric_code: Option<String>) -> Self {
        self.pricing_metric_code = pricing_metric_code;
        self
    }

    pub fn with_price_label_option(mut self, price_label: Option<String>) -> Self {
        self.price_label = price_label;
        self
    }

    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = source.into();
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct CatalogPublication {
    pub publication_id: String,
    pub publication_revision_id: String,
    pub publication_version: u64,
    pub publication_source_kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub publication_effective_from_ms: Option<u64>,
    pub product_id: String,
    pub offer_id: String,
    pub publication_kind: CatalogPublicationKind,
    pub status: CatalogPublicationStatus,
    pub source: String,
}

impl CatalogPublication {
    pub fn new(
        publication_id: impl Into<String>,
        product_id: impl Into<String>,
        offer_id: impl Into<String>,
        publication_kind: CatalogPublicationKind,
    ) -> Self {
        let publication_id = publication_id.into();
        Self {
            publication_revision_id: publication_id.clone(),
            publication_version: 1,
            publication_source_kind: "catalog_seed".to_owned(),
            publication_effective_from_ms: None,
            publication_id,
            product_id: product_id.into(),
            offer_id: offer_id.into(),
            publication_kind,
            status: CatalogPublicationStatus::Draft,
            source: String::new(),
        }
    }

    pub fn with_status(mut self, status: CatalogPublicationStatus) -> Self {
        self.status = status;
        self
    }

    pub fn with_publication_revision_id(
        mut self,
        publication_revision_id: impl Into<String>,
    ) -> Self {
        self.publication_revision_id = publication_revision_id.into();
        self
    }

    pub fn with_publication_version(mut self, publication_version: u64) -> Self {
        self.publication_version = publication_version;
        self
    }

    pub fn with_publication_source_kind(
        mut self,
        publication_source_kind: impl Into<String>,
    ) -> Self {
        self.publication_source_kind = publication_source_kind.into();
        self
    }

    pub fn with_publication_effective_from_ms_option(
        mut self,
        publication_effective_from_ms: Option<u64>,
    ) -> Self {
        self.publication_effective_from_ms = publication_effective_from_ms;
        self
    }

    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = source.into();
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct CatalogPublicationLifecycleAuditRecord {
    pub audit_id: String,
    pub publication_id: String,
    pub publication_revision_id: String,
    pub publication_version: u64,
    pub publication_source_kind: String,
    pub action: CatalogPublicationLifecycleAction,
    pub outcome: CatalogPublicationLifecycleAuditOutcome,
    pub operator_id: String,
    pub request_id: String,
    pub operator_reason: String,
    pub publication_status_before: String,
    pub publication_status_after: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub governed_pricing_plan_id: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub governed_pricing_status_before: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub governed_pricing_status_after: Option<String>,
    #[serde(default)]
    pub decision_reasons: Vec<String>,
    pub recorded_at_ms: u64,
}

impl CatalogPublicationLifecycleAuditRecord {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        audit_id: impl Into<String>,
        publication_id: impl Into<String>,
        publication_revision_id: impl Into<String>,
        publication_version: u64,
        publication_source_kind: impl Into<String>,
        action: CatalogPublicationLifecycleAction,
        outcome: CatalogPublicationLifecycleAuditOutcome,
        operator_id: impl Into<String>,
        request_id: impl Into<String>,
        operator_reason: impl Into<String>,
        publication_status_before: impl Into<String>,
        publication_status_after: impl Into<String>,
        recorded_at_ms: u64,
    ) -> Self {
        Self {
            audit_id: audit_id.into(),
            publication_id: publication_id.into(),
            publication_revision_id: publication_revision_id.into(),
            publication_version,
            publication_source_kind: publication_source_kind.into(),
            action,
            outcome,
            operator_id: operator_id.into(),
            request_id: request_id.into(),
            operator_reason: operator_reason.into(),
            publication_status_before: publication_status_before.into(),
            publication_status_after: publication_status_after.into(),
            governed_pricing_plan_id: None,
            governed_pricing_status_before: None,
            governed_pricing_status_after: None,
            decision_reasons: Vec::new(),
            recorded_at_ms,
        }
    }

    pub fn with_governed_pricing_plan_id(mut self, governed_pricing_plan_id: Option<u64>) -> Self {
        self.governed_pricing_plan_id = governed_pricing_plan_id;
        self
    }

    pub fn with_governed_pricing_status_before_option(
        mut self,
        governed_pricing_status_before: Option<String>,
    ) -> Self {
        self.governed_pricing_status_before = governed_pricing_status_before;
        self
    }

    pub fn with_governed_pricing_status_after_option(
        mut self,
        governed_pricing_status_after: Option<String>,
    ) -> Self {
        self.governed_pricing_status_after = governed_pricing_status_after;
        self
    }

    pub fn with_decision_reasons(mut self, decision_reasons: Vec<String>) -> Self {
        self.decision_reasons = decision_reasons;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, Default)]
pub struct CommercialCatalog {
    #[serde(default)]
    pub products: Vec<ApiProduct>,
    #[serde(default)]
    pub offers: Vec<ProductOffer>,
    #[serde(default)]
    pub publications: Vec<CatalogPublication>,
}

impl CommercialCatalog {
    pub fn new(
        products: Vec<ApiProduct>,
        offers: Vec<ProductOffer>,
        publications: Vec<CatalogPublication>,
    ) -> Self {
        Self {
            products,
            offers,
            publications,
        }
    }
}
