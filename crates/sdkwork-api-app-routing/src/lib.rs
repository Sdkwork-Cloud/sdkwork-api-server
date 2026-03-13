use anyhow::Result;
use sdkwork_api_domain_routing::RoutingDecision;
use sdkwork_api_storage_core::AdminStore;

pub fn service_name() -> &'static str {
    "routing-service"
}

pub fn simulate_route(_capability: &str, _model: &str) -> Result<RoutingDecision> {
    Ok(RoutingDecision::new(
        "provider-openai-official",
        vec!["provider-openai-official".into()],
    ))
}

pub async fn simulate_route_with_store(
    store: &dyn AdminStore,
    capability: &str,
    model: &str,
) -> Result<RoutingDecision> {
    let mut candidate_ids: Vec<String> = store
        .list_models()
        .await?
        .into_iter()
        .filter(|entry| entry.external_name == model)
        .map(|entry| entry.provider_id)
        .collect();

    candidate_ids.sort();
    candidate_ids.dedup();

    if candidate_ids.is_empty() {
        return simulate_route(capability, model);
    }

    Ok(RoutingDecision::new(
        candidate_ids[0].clone(),
        candidate_ids,
    ))
}
