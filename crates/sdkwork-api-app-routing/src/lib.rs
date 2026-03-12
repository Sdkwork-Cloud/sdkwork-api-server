use anyhow::Result;
use sdkwork_api_domain_routing::RoutingDecision;

pub fn service_name() -> &'static str {
    "routing-service"
}

pub fn simulate_route(_capability: &str, _model: &str) -> Result<RoutingDecision> {
    Ok(RoutingDecision::new(
        "provider-openai-official",
        vec!["provider-openai-official".into()],
    ))
}
