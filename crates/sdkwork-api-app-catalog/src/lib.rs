use anyhow::{anyhow, Result};
use sdkwork_api_domain_billing::PricingPlanRecord;
use sdkwork_api_domain_catalog::{
    derive_provider_default_plugin_family, derive_provider_extension_id,
    derive_provider_protocol_kind, normalize_provider_default_plugin_family, ApiProduct,
    ApiProductKind, CatalogPublication, CatalogPublicationKind, CatalogPublicationStatus, Channel,
    ChannelModelRecord, CommercialCatalog, ModelCapability, ModelCatalogEntry, ModelPriceRecord,
    ModelPriceTier, ProductOffer, ProviderAccountRecord, ProviderChannelBinding,
    ProviderModelRecord, ProxyProvider, QuoteKind,
};
use sdkwork_api_storage_core::AdminStore;
use serde::Serialize;
use utoipa::ToSchema;

pub use sdkwork_api_domain_catalog::{
    ApiProduct as CommercialApiProduct, ApiProductKind as CommercialApiProductKind,
    CatalogPublication as CommercialCatalogPublication,
    CatalogPublicationKind as CommercialCatalogPublicationKind,
    CatalogPublicationStatus as CommercialCatalogPublicationStatus,
    CommercialCatalog as CanonicalCommercialCatalog, ProductOffer as CommercialProductOffer,
    QuoteKind as CommercialQuoteKind,
};

pub fn service_name() -> &'static str {
    "catalog-service"
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommercialCatalogSeedProduct {
    pub product_kind: ApiProductKind,
    pub target_id: String,
    pub display_name: String,
    pub source: String,
    pub price_label: Option<String>,
}

impl CommercialCatalogSeedProduct {
    pub fn new(
        product_kind: ApiProductKind,
        target_id: impl Into<String>,
        display_name: impl Into<String>,
        source: impl Into<String>,
    ) -> Self {
        Self {
            product_kind,
            target_id: target_id.into(),
            display_name: display_name.into(),
            source: source.into(),
            price_label: None,
        }
    }

    pub fn with_price_label_option(mut self, price_label: Option<String>) -> Self {
        self.price_label = price_label;
        self
    }
}

pub fn canonical_catalog_product_id(
    product_kind: ApiProductKind,
    target_id: impl AsRef<str>,
) -> String {
    format!("product:{}:{}", product_kind.as_str(), target_id.as_ref())
}

pub fn canonical_catalog_offer_id(
    product_kind: ApiProductKind,
    target_id: impl AsRef<str>,
) -> String {
    format!("offer:{}:{}", product_kind.as_str(), target_id.as_ref())
}

pub fn canonical_catalog_pricing_plan_id(
    product_kind: ApiProductKind,
    target_id: impl AsRef<str>,
) -> String {
    format!(
        "pricing_plan:{}:{}",
        product_kind.as_str(),
        target_id.as_ref()
    )
}

pub fn canonical_catalog_pricing_plan_code(
    product_kind: ApiProductKind,
    target_id: impl AsRef<str>,
) -> String {
    format!("{}:{}", product_kind.as_str(), target_id.as_ref())
}

pub fn normalize_commercial_pricing_plan_code(
    plan_code: impl AsRef<str>,
) -> Result<Option<String>> {
    let plan_code = plan_code.as_ref().trim();
    let Some((product_kind, target_id)) = plan_code.split_once(':') else {
        return Ok(None);
    };
    let Some(product_kind) = normalize_commercial_product_kind_segment(product_kind) else {
        return Ok(None);
    };
    let target_id = target_id.trim();
    if target_id.is_empty() {
        return Err(anyhow!(
            "canonical commercial pricing plan code requires a non-empty target_id"
        ));
    }
    Ok(Some(canonical_catalog_pricing_plan_code(
        product_kind,
        target_id,
    )))
}

pub fn canonical_catalog_pricing_metric_code(product_kind: ApiProductKind) -> &'static str {
    match product_kind {
        ApiProductKind::SubscriptionPlan => "subscription.base",
        ApiProductKind::RechargePack => "credit.prepaid_pack",
        ApiProductKind::CustomRecharge => "credit.prepaid_custom",
    }
}

pub fn canonical_catalog_pricing_rate_id(
    product_kind: ApiProductKind,
    target_id: impl AsRef<str>,
    metric_code: impl AsRef<str>,
) -> String {
    format!(
        "pricing_rate:{}:{}:{}",
        product_kind.as_str(),
        target_id.as_ref(),
        metric_code.as_ref()
    )
}

pub fn canonical_catalog_publication_id(
    publication_kind: CatalogPublicationKind,
    product_kind: ApiProductKind,
    target_id: impl AsRef<str>,
) -> String {
    format!(
        "publication:{}:offer:{}:{}",
        publication_kind.as_str(),
        product_kind.as_str(),
        target_id.as_ref()
    )
}

pub fn canonical_catalog_publication_revision_id(
    publication_kind: CatalogPublicationKind,
    product_kind: ApiProductKind,
    target_id: impl AsRef<str>,
    publication_version: u64,
) -> String {
    format!(
        "publication_revision:{}:offer:{}:{}:v{}",
        publication_kind.as_str(),
        product_kind.as_str(),
        target_id.as_ref(),
        publication_version
    )
}

pub fn build_canonical_commercial_catalog(
    seed_products: &[CommercialCatalogSeedProduct],
) -> CommercialCatalog {
    build_canonical_commercial_catalog_with_pricing_plans(seed_products, &[])
}

pub fn build_canonical_commercial_catalog_with_pricing_plans(
    seed_products: &[CommercialCatalogSeedProduct],
    pricing_plans: &[PricingPlanRecord],
) -> CommercialCatalog {
    let mut products = Vec::with_capacity(seed_products.len());
    let mut offers = Vec::with_capacity(seed_products.len());
    let mut publications = Vec::with_capacity(seed_products.len());

    for seed in seed_products {
        let product_id = canonical_catalog_product_id(seed.product_kind, &seed.target_id);
        let offer_id = canonical_catalog_offer_id(seed.product_kind, &seed.target_id);
        let pricing_plan_id = canonical_catalog_pricing_plan_id(seed.product_kind, &seed.target_id);
        let governed_pricing_plan =
            resolve_catalog_pricing_governance(pricing_plans, seed.product_kind, &seed.target_id);
        let pricing_metric_code = canonical_catalog_pricing_metric_code(seed.product_kind);
        let pricing_rate_id = canonical_catalog_pricing_rate_id(
            seed.product_kind,
            &seed.target_id,
            pricing_metric_code,
        );
        let publication_kind = CatalogPublicationKind::PortalCatalog;
        let publication_governance = resolve_catalog_publication_governance(governed_pricing_plan);
        let pricing_plan_version = publication_governance.publication_version;

        products.push(
            ApiProduct::new(
                product_id.clone(),
                seed.product_kind,
                seed.target_id.clone(),
                seed.display_name.clone(),
            )
            .with_source(seed.source.clone()),
        );
        offers.push(
            ProductOffer::new(
                offer_id.clone(),
                product_id.clone(),
                seed.product_kind,
                seed.display_name.clone(),
                QuoteKind::ProductPurchase,
                seed.product_kind,
                seed.target_id.clone(),
            )
            .with_pricing_plan_id_option(Some(pricing_plan_id))
            .with_pricing_plan_version_option(Some(pricing_plan_version))
            .with_pricing_rate_id_option(Some(pricing_rate_id))
            .with_pricing_metric_code_option(Some(pricing_metric_code.to_owned()))
            .with_price_label_option(seed.price_label.clone())
            .with_source(seed.source.clone()),
        );
        publications.push(
            CatalogPublication::new(
                canonical_catalog_publication_id(
                    publication_kind,
                    seed.product_kind,
                    &seed.target_id,
                ),
                product_id,
                offer_id,
                publication_kind,
            )
            .with_publication_revision_id(canonical_catalog_publication_revision_id(
                publication_kind,
                seed.product_kind,
                &seed.target_id,
                publication_governance.publication_version,
            ))
            .with_publication_version(publication_governance.publication_version)
            .with_publication_source_kind(publication_governance.source_kind)
            .with_publication_effective_from_ms_option(
                publication_governance.publication_effective_from_ms,
            )
            .with_status(publication_governance.status)
            .with_source(seed.source.clone()),
        );
    }

    CommercialCatalog::new(products, offers, publications)
}

fn resolve_catalog_pricing_governance<'a>(
    pricing_plans: &'a [PricingPlanRecord],
    product_kind: ApiProductKind,
    target_id: &str,
) -> Option<&'a PricingPlanRecord> {
    let expected_plan_code = canonical_catalog_pricing_plan_code(product_kind, target_id);
    pricing_plans
        .iter()
        .filter(|plan| {
            normalize_commercial_pricing_plan_code(&plan.plan_code)
                .ok()
                .flatten()
                .as_deref()
                == Some(expected_plan_code.as_str())
        })
        .filter(|plan| catalog_pricing_governance_rank(&plan.status).is_some())
        .max_by(|left, right| compare_catalog_pricing_governance(left, right))
}

fn compare_catalog_pricing_governance(
    left: &PricingPlanRecord,
    right: &PricingPlanRecord,
) -> std::cmp::Ordering {
    catalog_pricing_governance_rank(&left.status)
        .cmp(&catalog_pricing_governance_rank(&right.status))
        .then(left.plan_version.cmp(&right.plan_version))
        .then(left.effective_from_ms.cmp(&right.effective_from_ms))
        .then(left.updated_at_ms.cmp(&right.updated_at_ms))
        .then(left.created_at_ms.cmp(&right.created_at_ms))
        .then(left.pricing_plan_id.cmp(&right.pricing_plan_id))
}

fn catalog_pricing_governance_rank(status: &str) -> Option<u8> {
    match status.trim().to_ascii_lowercase().as_str() {
        "active" | "published" => Some(3),
        "planned" | "draft" => Some(2),
        "retired" | "archived" => Some(1),
        _ => None,
    }
}

fn catalog_publication_status_from_pricing_status(
    status: &str,
) -> Option<CatalogPublicationStatus> {
    match status.trim().to_ascii_lowercase().as_str() {
        "active" | "published" => Some(CatalogPublicationStatus::Published),
        "planned" | "draft" => Some(CatalogPublicationStatus::Draft),
        "retired" | "archived" => Some(CatalogPublicationStatus::Archived),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CatalogPublicationGovernance {
    publication_version: u64,
    status: CatalogPublicationStatus,
    source_kind: &'static str,
    publication_effective_from_ms: Option<u64>,
}

fn resolve_catalog_publication_governance(
    governed_pricing_plan: Option<&PricingPlanRecord>,
) -> CatalogPublicationGovernance {
    match governed_pricing_plan {
        Some(plan) => CatalogPublicationGovernance {
            publication_version: plan.plan_version,
            status: catalog_publication_status_from_pricing_status(&plan.status)
                .unwrap_or(CatalogPublicationStatus::Published),
            source_kind: "pricing_plan",
            publication_effective_from_ms: Some(plan.effective_from_ms),
        },
        None => CatalogPublicationGovernance {
            publication_version: 1,
            status: CatalogPublicationStatus::Published,
            source_kind: "catalog_seed",
            publication_effective_from_ms: None,
        },
    }
}

pub fn create_channel(id: &str, name: &str) -> Result<Channel> {
    Ok(Channel::new(id, name))
}

pub fn create_provider(id: &str, channel_id: &str, display_name: &str) -> Result<ProxyProvider> {
    Ok(ProxyProvider::new(
        id,
        channel_id,
        channel_id,
        "http://localhost",
        display_name,
    ))
}

pub async fn persist_channel(store: &dyn AdminStore, id: &str, name: &str) -> Result<Channel> {
    let channel = create_channel(id, name)?;
    store.insert_channel(&channel).await
}

pub async fn list_channels(store: &dyn AdminStore) -> Result<Vec<Channel>> {
    store.list_channels().await
}

pub async fn delete_channel(store: &dyn AdminStore, channel_id: &str) -> Result<bool> {
    let channel_id = channel_id.trim();
    if channel_id.is_empty() {
        return Err(anyhow!("channel id is required"));
    }
    store.delete_channel(channel_id).await
}

pub fn create_provider_with_config(
    id: &str,
    channel_id: &str,
    adapter_kind: &str,
    base_url: &str,
    display_name: &str,
) -> Result<ProxyProvider> {
    create_provider_with_extension_id(id, channel_id, adapter_kind, None, base_url, display_name)
}

pub fn create_provider_with_protocol_kind(
    id: &str,
    channel_id: &str,
    adapter_kind: &str,
    protocol_kind: &str,
    base_url: &str,
    display_name: &str,
) -> Result<ProxyProvider> {
    create_provider_with_protocol_kind_and_extension_id(
        id,
        channel_id,
        adapter_kind,
        Some(protocol_kind),
        None,
        base_url,
        display_name,
    )
}

pub fn create_provider_with_default_plugin_family(
    id: &str,
    channel_id: &str,
    default_plugin_family: &str,
    base_url: &str,
    display_name: &str,
) -> Result<ProxyProvider> {
    create_provider_with_default_plugin_family_and_bindings(
        id,
        channel_id,
        default_plugin_family,
        base_url,
        display_name,
        &[],
    )
}

pub fn create_provider_with_default_plugin_family_and_bindings(
    id: &str,
    channel_id: &str,
    default_plugin_family: &str,
    base_url: &str,
    display_name: &str,
    channel_bindings: &[ProviderChannelBinding],
) -> Result<ProxyProvider> {
    let normalized = normalize_provider_integration(None, None, None, Some(default_plugin_family))?;
    create_provider_with_bindings_and_extension_id(
        id,
        channel_id,
        &normalized.adapter_kind,
        normalized.protocol_kind.as_deref(),
        normalized.extension_id.as_deref(),
        base_url,
        display_name,
        channel_bindings,
    )
}

pub fn create_provider_with_extension_id(
    id: &str,
    channel_id: &str,
    adapter_kind: &str,
    extension_id: Option<&str>,
    base_url: &str,
    display_name: &str,
) -> Result<ProxyProvider> {
    create_provider_with_protocol_kind_and_extension_id(
        id,
        channel_id,
        adapter_kind,
        None,
        extension_id,
        base_url,
        display_name,
    )
}

fn create_provider_with_protocol_kind_and_extension_id(
    id: &str,
    channel_id: &str,
    adapter_kind: &str,
    protocol_kind: Option<&str>,
    extension_id: Option<&str>,
    base_url: &str,
    display_name: &str,
) -> Result<ProxyProvider> {
    let provider = ProxyProvider::new(id, channel_id, adapter_kind, base_url, display_name);
    let provider = match protocol_kind {
        Some(protocol_kind) => provider.with_protocol_kind(protocol_kind),
        None => provider,
    };
    Ok(match extension_id {
        Some(extension_id) => provider.with_extension_id(extension_id),
        None => provider,
    })
}

pub fn create_provider_with_bindings(
    id: &str,
    channel_id: &str,
    adapter_kind: &str,
    base_url: &str,
    display_name: &str,
    channel_bindings: &[ProviderChannelBinding],
) -> Result<ProxyProvider> {
    create_provider_with_bindings_and_extension_id(
        id,
        channel_id,
        adapter_kind,
        None,
        None,
        base_url,
        display_name,
        channel_bindings,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn create_provider_with_bindings_and_extension_id(
    id: &str,
    channel_id: &str,
    adapter_kind: &str,
    protocol_kind: Option<&str>,
    extension_id: Option<&str>,
    base_url: &str,
    display_name: &str,
    channel_bindings: &[ProviderChannelBinding],
) -> Result<ProxyProvider> {
    let mut provider = create_provider_with_protocol_kind_and_extension_id(
        id,
        channel_id,
        adapter_kind,
        protocol_kind,
        extension_id,
        base_url,
        display_name,
    )?;
    for binding in channel_bindings {
        provider = provider.with_channel_binding(binding.clone());
    }
    Ok(provider)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
pub struct ProviderIntegrationView {
    pub mode: ProviderIntegrationMode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_plugin_family: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ProviderIntegrationMode {
    StandardPassthrough,
    DefaultPlugin,
    CustomPlugin,
}

pub fn provider_integration_view(provider: &ProxyProvider) -> ProviderIntegrationView {
    let derived_extension_id = derive_provider_extension_id(&provider.adapter_kind);
    if provider.extension_id != derived_extension_id {
        return ProviderIntegrationView {
            mode: ProviderIntegrationMode::CustomPlugin,
            default_plugin_family: None,
        };
    }

    if let Some(default_plugin_family) =
        derive_provider_default_plugin_family(&provider.adapter_kind)
    {
        return ProviderIntegrationView {
            mode: ProviderIntegrationMode::DefaultPlugin,
            default_plugin_family: Some(default_plugin_family.to_owned()),
        };
    }

    if matches!(provider.protocol_kind(), "openai" | "anthropic" | "gemini") {
        return ProviderIntegrationView {
            mode: ProviderIntegrationMode::StandardPassthrough,
            default_plugin_family: None,
        };
    }

    ProviderIntegrationView {
        mode: ProviderIntegrationMode::CustomPlugin,
        default_plugin_family: None,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedProviderIntegration {
    pub adapter_kind: String,
    pub protocol_kind: Option<String>,
    pub extension_id: Option<String>,
}

pub fn normalize_provider_integration(
    adapter_kind: Option<&str>,
    protocol_kind: Option<&str>,
    extension_id: Option<&str>,
    default_plugin_family: Option<&str>,
) -> Result<NormalizedProviderIntegration> {
    let normalized_adapter_kind = normalize_optional_identity_segment(adapter_kind);

    if let Some(default_plugin_family) = default_plugin_family {
        let default_plugin_family = normalize_provider_default_plugin_family(default_plugin_family)
            .ok_or_else(|| anyhow!("unsupported default_plugin_family"))?;

        if let Some(adapter_kind) = normalized_adapter_kind.as_deref() {
            if adapter_kind != default_plugin_family {
                return Err(anyhow!(
                    "default_plugin_family must match adapter_kind when both are provided"
                ));
            }
        }

        let derived_protocol_kind = derive_provider_protocol_kind(default_plugin_family);
        let derived_extension_id = derive_provider_extension_id(default_plugin_family);

        if let Some(protocol_kind) = normalize_optional_identity_segment(protocol_kind) {
            if protocol_kind != derived_protocol_kind {
                return Err(anyhow!(
                    "default_plugin_family cannot override protocol_kind"
                ));
            }
        }

        if let Some(extension_id) = normalize_optional_field(extension_id) {
            if extension_id != derived_extension_id {
                return Err(anyhow!(
                    "default_plugin_family cannot override extension_id"
                ));
            }
        }

        return Ok(NormalizedProviderIntegration {
            adapter_kind: default_plugin_family.to_owned(),
            protocol_kind: Some(derived_protocol_kind.to_owned()),
            extension_id: Some(derived_extension_id),
        });
    }

    Ok(NormalizedProviderIntegration {
        adapter_kind: normalized_adapter_kind.ok_or_else(|| anyhow!("adapter_kind is required"))?,
        protocol_kind: normalize_optional_identity_segment(protocol_kind),
        extension_id: normalize_optional_field(extension_id),
    })
}

#[derive(Debug, Clone, Copy)]
pub struct PersistProviderWithBindingsRequest<'a> {
    pub id: &'a str,
    pub channel_id: &'a str,
    pub adapter_kind: &'a str,
    pub protocol_kind: Option<&'a str>,
    pub extension_id: Option<&'a str>,
    pub base_url: &'a str,
    pub display_name: &'a str,
    pub channel_bindings: &'a [ProviderChannelBinding],
}

pub async fn persist_provider(
    store: &dyn AdminStore,
    id: &str,
    channel_id: &str,
    adapter_kind: &str,
    base_url: &str,
    display_name: &str,
) -> Result<ProxyProvider> {
    let provider = create_provider_with_extension_id(
        id,
        channel_id,
        adapter_kind,
        None,
        base_url,
        display_name,
    )?;
    store.insert_provider(&provider).await
}

pub async fn persist_provider_with_bindings(
    store: &dyn AdminStore,
    id: &str,
    channel_id: &str,
    adapter_kind: &str,
    base_url: &str,
    display_name: &str,
    channel_bindings: &[ProviderChannelBinding],
) -> Result<ProxyProvider> {
    persist_provider_with_bindings_and_extension_id(
        store,
        PersistProviderWithBindingsRequest {
            id,
            channel_id,
            adapter_kind,
            protocol_kind: None,
            extension_id: None,
            base_url,
            display_name,
            channel_bindings,
        },
    )
    .await
}

pub async fn persist_provider_with_bindings_and_extension_id(
    store: &dyn AdminStore,
    request: PersistProviderWithBindingsRequest<'_>,
) -> Result<ProxyProvider> {
    let provider = create_provider_with_bindings_and_extension_id(
        request.id,
        request.channel_id,
        request.adapter_kind,
        request.protocol_kind,
        request.extension_id,
        request.base_url,
        request.display_name,
        request.channel_bindings,
    )?;
    store.insert_provider(&provider).await
}

pub async fn list_providers(store: &dyn AdminStore) -> Result<Vec<ProxyProvider>> {
    store.list_providers().await
}

pub async fn delete_provider(store: &dyn AdminStore, provider_id: &str) -> Result<bool> {
    let provider_id = provider_id.trim();
    if provider_id.is_empty() {
        return Err(anyhow!("provider id is required"));
    }
    store.delete_provider(provider_id).await
}

pub fn create_model(external_name: &str, provider_id: &str) -> Result<ModelCatalogEntry> {
    Ok(ModelCatalogEntry::new(external_name, provider_id))
}

pub fn create_channel_model(
    channel_id: &str,
    model_id: &str,
    model_display_name: &str,
) -> Result<ChannelModelRecord> {
    Ok(ChannelModelRecord::new(
        channel_id,
        model_id,
        model_display_name,
    ))
}

#[allow(clippy::too_many_arguments)]
pub fn create_channel_model_with_metadata(
    channel_id: &str,
    model_id: &str,
    model_display_name: &str,
    capabilities: &[ModelCapability],
    streaming: bool,
    context_window: Option<u64>,
    description: Option<&str>,
) -> Result<ChannelModelRecord> {
    let mut model = create_channel_model(channel_id, model_id, model_display_name)?
        .with_streaming(streaming)
        .with_context_window_option(context_window)
        .with_description_option(description.map(str::to_owned));
    for capability in capabilities {
        model = model.with_capability(capability.clone());
    }
    Ok(model)
}

pub fn create_model_price(
    channel_id: &str,
    model_id: &str,
    proxy_provider_id: &str,
) -> Result<ModelPriceRecord> {
    Ok(ModelPriceRecord::new(
        channel_id,
        model_id,
        proxy_provider_id,
    ))
}

#[allow(clippy::too_many_arguments)]
pub fn create_model_price_with_rates(
    channel_id: &str,
    model_id: &str,
    proxy_provider_id: &str,
    currency_code: &str,
    price_unit: &str,
    input_price: f64,
    output_price: f64,
    cache_read_price: f64,
    cache_write_price: f64,
    request_price: f64,
    is_active: bool,
) -> Result<ModelPriceRecord> {
    Ok(create_model_price(channel_id, model_id, proxy_provider_id)?
        .with_currency_code(currency_code)
        .with_price_unit(price_unit)
        .with_input_price(input_price)
        .with_output_price(output_price)
        .with_cache_read_price(cache_read_price)
        .with_cache_write_price(cache_write_price)
        .with_request_price(request_price)
        .with_active(is_active))
}

#[allow(clippy::too_many_arguments)]
pub fn create_model_price_with_rates_and_metadata(
    channel_id: &str,
    model_id: &str,
    proxy_provider_id: &str,
    currency_code: &str,
    price_unit: &str,
    input_price: f64,
    output_price: f64,
    cache_read_price: f64,
    cache_write_price: f64,
    request_price: f64,
    price_source_kind: &str,
    billing_notes: Option<&str>,
    pricing_tiers: Vec<ModelPriceTier>,
    is_active: bool,
) -> Result<ModelPriceRecord> {
    Ok(create_model_price_with_rates(
        channel_id,
        model_id,
        proxy_provider_id,
        currency_code,
        price_unit,
        input_price,
        output_price,
        cache_read_price,
        cache_write_price,
        request_price,
        is_active,
    )?
    .with_price_source_kind(price_source_kind)
    .with_billing_notes_option(billing_notes.map(str::to_owned))
    .with_pricing_tiers(pricing_tiers))
}

#[allow(clippy::too_many_arguments)]
pub fn create_provider_model_with_metadata(
    proxy_provider_id: &str,
    channel_id: &str,
    model_id: &str,
    provider_model_id: &str,
    provider_model_family: Option<&str>,
    capabilities: &[ModelCapability],
    streaming: bool,
    context_window: Option<u64>,
    max_output_tokens: Option<u64>,
    supports_prompt_caching: bool,
    supports_reasoning_usage: bool,
    supports_tool_usage_metrics: bool,
    is_default_route: bool,
    is_active: bool,
) -> Result<ProviderModelRecord> {
    let mut record = ProviderModelRecord::new(proxy_provider_id, channel_id, model_id)
        .with_provider_model_id(provider_model_id)
        .with_provider_model_family_option(provider_model_family.map(str::to_owned))
        .with_streaming(streaming)
        .with_context_window_option(context_window)
        .with_max_output_tokens_option(max_output_tokens)
        .with_supports_prompt_caching(supports_prompt_caching)
        .with_supports_reasoning_usage(supports_reasoning_usage)
        .with_supports_tool_usage_metrics(supports_tool_usage_metrics)
        .with_default_route(is_default_route)
        .with_active(is_active);
    for capability in capabilities {
        record = record.with_capability(capability.clone());
    }
    Ok(record)
}

pub fn create_model_with_metadata(
    external_name: &str,
    provider_id: &str,
    capabilities: &[ModelCapability],
    streaming: bool,
    context_window: Option<u64>,
) -> Result<ModelCatalogEntry> {
    let mut model = create_model(external_name, provider_id)?.with_streaming(streaming);
    for capability in capabilities {
        model = model.with_capability(capability.clone());
    }
    if let Some(context_window) = context_window {
        model = model.with_context_window(context_window);
    }
    Ok(model)
}

pub async fn persist_model(
    store: &dyn AdminStore,
    external_name: &str,
    provider_id: &str,
) -> Result<ModelCatalogEntry> {
    persist_model_with_metadata(store, external_name, provider_id, &[], false, None).await
}

pub async fn persist_model_with_metadata(
    store: &dyn AdminStore,
    external_name: &str,
    provider_id: &str,
    capabilities: &[ModelCapability],
    streaming: bool,
    context_window: Option<u64>,
) -> Result<ModelCatalogEntry> {
    let model = create_model_with_metadata(
        external_name,
        provider_id,
        capabilities,
        streaming,
        context_window,
    )?;
    store.insert_model(&model).await
}

pub async fn list_model_entries(store: &dyn AdminStore) -> Result<Vec<ModelCatalogEntry>> {
    store.list_models().await
}

#[allow(clippy::too_many_arguments)]
pub async fn persist_channel_model_with_metadata(
    store: &dyn AdminStore,
    channel_id: &str,
    model_id: &str,
    model_display_name: &str,
    capabilities: &[ModelCapability],
    streaming: bool,
    context_window: Option<u64>,
    description: Option<&str>,
) -> Result<ChannelModelRecord> {
    let record = create_channel_model_with_metadata(
        channel_id,
        model_id,
        model_display_name,
        capabilities,
        streaming,
        context_window,
        description,
    )?;
    store.insert_channel_model(&record).await
}

pub async fn list_channel_models(store: &dyn AdminStore) -> Result<Vec<ChannelModelRecord>> {
    store.list_channel_models().await
}

#[allow(clippy::too_many_arguments)]
pub async fn persist_provider_model_with_metadata(
    store: &dyn AdminStore,
    proxy_provider_id: &str,
    channel_id: &str,
    model_id: &str,
    provider_model_id: Option<&str>,
    provider_model_family: Option<&str>,
    capabilities: Option<&[ModelCapability]>,
    streaming: Option<bool>,
    context_window: Option<u64>,
    max_output_tokens: Option<u64>,
    supports_prompt_caching: bool,
    supports_reasoning_usage: bool,
    supports_tool_usage_metrics: bool,
    is_default_route: bool,
    is_active: bool,
) -> Result<ProviderModelRecord> {
    let (provider, channel_model) =
        load_provider_and_channel_model(store, proxy_provider_id, channel_id, model_id).await?;
    if !provider
        .channel_bindings
        .iter()
        .any(|binding| binding.channel_id == channel_id)
        && provider.channel_id != channel_id
    {
        return Err(anyhow!("provider is not bound to the selected channel"));
    }
    let record = create_provider_model_with_metadata(
        proxy_provider_id,
        channel_id,
        model_id,
        provider_model_id.unwrap_or(model_id),
        provider_model_family,
        capabilities.unwrap_or(&channel_model.capabilities),
        streaming.unwrap_or(channel_model.streaming),
        context_window.or(channel_model.context_window),
        max_output_tokens,
        supports_prompt_caching,
        supports_reasoning_usage,
        supports_tool_usage_metrics,
        is_default_route,
        is_active,
    )?;
    store.upsert_provider_model(&record).await
}

pub async fn list_provider_models(store: &dyn AdminStore) -> Result<Vec<ProviderModelRecord>> {
    store.list_provider_models().await
}

#[allow(clippy::too_many_arguments)]
pub async fn persist_provider_account(
    store: &dyn AdminStore,
    provider_account_id: &str,
    provider_id: &str,
    display_name: &str,
    account_kind: &str,
    owner_scope: &str,
    owner_tenant_id: Option<&str>,
    execution_instance_id: &str,
    base_url_override: Option<&str>,
    region: Option<&str>,
    priority: i32,
    weight: u32,
    enabled: bool,
    routing_tags: &[String],
    health_score_hint: Option<f64>,
    latency_ms_hint: Option<u64>,
    cost_hint: Option<f64>,
    success_rate_hint: Option<f64>,
    throughput_hint: Option<f64>,
    max_concurrency: Option<u32>,
    daily_budget: Option<f64>,
    notes: Option<&str>,
) -> Result<ProviderAccountRecord> {
    let provider_account_id = provider_account_id.trim();
    let provider_id = provider_id.trim();
    let display_name = display_name.trim();
    let account_kind = account_kind.trim();
    let owner_scope = owner_scope.trim();
    let execution_instance_id = execution_instance_id.trim();
    if provider_account_id.is_empty()
        || provider_id.is_empty()
        || display_name.is_empty()
        || account_kind.is_empty()
        || owner_scope.is_empty()
        || execution_instance_id.is_empty()
    {
        return Err(anyhow!(
            "provider_account_id, provider_id, display_name, account_kind, owner_scope, and execution_instance_id are required"
        ));
    }
    if store.find_provider(provider_id).await?.is_none() {
        return Err(anyhow!(
            "provider must exist before provider-account can be saved"
        ));
    }
    if !store
        .list_extension_instances()
        .await?
        .into_iter()
        .any(|instance| instance.instance_id == execution_instance_id)
    {
        return Err(anyhow!(
            "execution_instance_id must reference a registered extension instance"
        ));
    }

    let record = ProviderAccountRecord::new(
        provider_account_id,
        provider_id,
        display_name,
        account_kind,
        execution_instance_id,
    )
    .with_owner_scope(owner_scope)
    .with_owner_tenant_id_option(owner_tenant_id.map(ToOwned::to_owned))
    .with_base_url_override_option(base_url_override.map(ToOwned::to_owned))
    .with_region_option(region.map(ToOwned::to_owned))
    .with_priority(priority)
    .with_weight(weight)
    .with_enabled(enabled)
    .with_routing_tags(routing_tags.to_vec())
    .with_health_score_hint_option(health_score_hint)
    .with_latency_ms_hint_option(latency_ms_hint)
    .with_cost_hint_option(cost_hint)
    .with_success_rate_hint_option(success_rate_hint)
    .with_throughput_hint_option(throughput_hint)
    .with_max_concurrency_option(max_concurrency)
    .with_daily_budget_option(daily_budget)
    .with_notes_option(notes.map(ToOwned::to_owned));
    store.upsert_provider_account(&record).await
}

pub async fn list_provider_accounts(store: &dyn AdminStore) -> Result<Vec<ProviderAccountRecord>> {
    store.list_provider_accounts().await
}

pub async fn delete_provider_account(
    store: &dyn AdminStore,
    provider_account_id: &str,
) -> Result<bool> {
    let provider_account_id = provider_account_id.trim();
    if provider_account_id.is_empty() {
        return Err(anyhow!("provider_account_id is required"));
    }
    store.delete_provider_account(provider_account_id).await
}

pub async fn delete_provider_model(
    store: &dyn AdminStore,
    proxy_provider_id: &str,
    channel_id: &str,
    model_id: &str,
) -> Result<bool> {
    let proxy_provider_id = proxy_provider_id.trim();
    let channel_id = channel_id.trim();
    let model_id = model_id.trim();
    if proxy_provider_id.is_empty() || channel_id.is_empty() || model_id.is_empty() {
        return Err(anyhow!(
            "proxy_provider_id, channel_id, and model_id are required"
        ));
    }
    store
        .delete_provider_model(proxy_provider_id, channel_id, model_id)
        .await
}

pub async fn delete_channel_model(
    store: &dyn AdminStore,
    channel_id: &str,
    model_id: &str,
) -> Result<bool> {
    let channel_id = channel_id.trim();
    let model_id = model_id.trim();
    if channel_id.is_empty() || model_id.is_empty() {
        return Err(anyhow!("channel_id and model_id are required"));
    }
    store.delete_channel_model(channel_id, model_id).await
}

#[allow(clippy::too_many_arguments)]
pub async fn persist_model_price_with_rates(
    store: &dyn AdminStore,
    channel_id: &str,
    model_id: &str,
    proxy_provider_id: &str,
    currency_code: &str,
    price_unit: &str,
    input_price: f64,
    output_price: f64,
    cache_read_price: f64,
    cache_write_price: f64,
    request_price: f64,
    is_active: bool,
) -> Result<ModelPriceRecord> {
    ensure_provider_model_support_for_pricing(store, proxy_provider_id, channel_id, model_id)
        .await?;
    let record = create_model_price_with_rates(
        channel_id,
        model_id,
        proxy_provider_id,
        currency_code,
        price_unit,
        input_price,
        output_price,
        cache_read_price,
        cache_write_price,
        request_price,
        is_active,
    )?;
    store.insert_model_price(&record).await
}

#[allow(clippy::too_many_arguments)]
pub async fn persist_model_price_with_rates_and_metadata(
    store: &dyn AdminStore,
    channel_id: &str,
    model_id: &str,
    proxy_provider_id: &str,
    currency_code: &str,
    price_unit: &str,
    input_price: f64,
    output_price: f64,
    cache_read_price: f64,
    cache_write_price: f64,
    request_price: f64,
    price_source_kind: &str,
    billing_notes: Option<&str>,
    pricing_tiers: Vec<ModelPriceTier>,
    is_active: bool,
) -> Result<ModelPriceRecord> {
    ensure_provider_model_support_for_pricing(store, proxy_provider_id, channel_id, model_id)
        .await?;
    let record = create_model_price_with_rates_and_metadata(
        channel_id,
        model_id,
        proxy_provider_id,
        currency_code,
        price_unit,
        input_price,
        output_price,
        cache_read_price,
        cache_write_price,
        request_price,
        price_source_kind,
        billing_notes,
        pricing_tiers,
        is_active,
    )?;
    store.insert_model_price(&record).await
}

pub async fn list_model_prices(store: &dyn AdminStore) -> Result<Vec<ModelPriceRecord>> {
    store.list_model_prices().await
}

pub async fn delete_model_price(
    store: &dyn AdminStore,
    channel_id: &str,
    model_id: &str,
    proxy_provider_id: &str,
) -> Result<bool> {
    let channel_id = channel_id.trim();
    let model_id = model_id.trim();
    let proxy_provider_id = proxy_provider_id.trim();
    if channel_id.is_empty() || model_id.is_empty() || proxy_provider_id.is_empty() {
        return Err(anyhow!(
            "channel_id, model_id, and proxy_provider_id are required"
        ));
    }
    store
        .delete_model_price(channel_id, model_id, proxy_provider_id)
        .await
}

pub async fn delete_model_variant(
    store: &dyn AdminStore,
    external_name: &str,
    provider_id: &str,
) -> Result<bool> {
    let external_name = external_name.trim();
    if external_name.is_empty() {
        return Err(anyhow!("external_name is required"));
    }
    let provider_id = provider_id.trim();
    if provider_id.is_empty() {
        return Err(anyhow!("provider_id is required"));
    }
    store.delete_model_variant(external_name, provider_id).await
}

fn normalize_optional_identity_segment(value: Option<&str>) -> Option<String> {
    value.and_then(|value| {
        let normalized = value.trim().to_ascii_lowercase();
        (!normalized.is_empty()).then_some(normalized)
    })
}

fn normalize_commercial_product_kind_segment(value: &str) -> Option<ApiProductKind> {
    let mut normalized = String::with_capacity(value.len());
    let mut previous_separator = false;

    for ch in value.trim().chars() {
        if ch.is_ascii_alphanumeric() {
            normalized.push(ch.to_ascii_lowercase());
            previous_separator = false;
        } else if !previous_separator {
            normalized.push('_');
            previous_separator = true;
        }
    }

    match normalized.trim_matches('_') {
        "subscription_plan" | "subscriptionplan" => Some(ApiProductKind::SubscriptionPlan),
        "recharge_pack" | "rechargepack" => Some(ApiProductKind::RechargePack),
        "custom_recharge" | "customrecharge" => Some(ApiProductKind::CustomRecharge),
        _ => None,
    }
}

fn normalize_optional_field(value: Option<&str>) -> Option<String> {
    value.and_then(|value| {
        let normalized = value.trim();
        (!normalized.is_empty()).then_some(normalized.to_owned())
    })
}

async fn load_provider_and_channel_model(
    store: &dyn AdminStore,
    proxy_provider_id: &str,
    channel_id: &str,
    model_id: &str,
) -> Result<(ProxyProvider, ChannelModelRecord)> {
    let provider = store
        .find_provider(proxy_provider_id)
        .await?
        .ok_or_else(|| anyhow!("proxy_provider_id is not registered"))?;
    let channel_model = store
        .list_channel_models()
        .await?
        .into_iter()
        .find(|record| record.channel_id == channel_id && record.model_id == model_id)
        .ok_or_else(|| anyhow!("channel_id/model_id is not registered"))?;
    Ok((provider, channel_model))
}

async fn ensure_provider_model_support_for_pricing(
    store: &dyn AdminStore,
    proxy_provider_id: &str,
    channel_id: &str,
    model_id: &str,
) -> Result<()> {
    let _ = load_provider_and_channel_model(store, proxy_provider_id, channel_id, model_id).await?;
    let provider_model_exists = store
        .list_provider_models_for_channel_model(channel_id, model_id)
        .await?
        .into_iter()
        .any(|record| record.proxy_provider_id == proxy_provider_id);
    if !provider_model_exists {
        return Err(anyhow!(
            "provider-model must exist before pricing can be saved"
        ));
    }
    Ok(())
}
