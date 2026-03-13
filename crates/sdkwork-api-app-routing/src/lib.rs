use anyhow::{ensure, Result};
use sdkwork_api_domain_routing::{select_policy, RoutingDecision, RoutingPolicy};
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

pub fn create_routing_policy(
    policy_id: &str,
    capability: &str,
    model_pattern: &str,
    enabled: bool,
    priority: i32,
    ordered_provider_ids: &[String],
    default_provider_id: Option<&str>,
) -> Result<RoutingPolicy> {
    ensure!(!policy_id.trim().is_empty(), "policy_id must not be empty");
    ensure!(
        !capability.trim().is_empty(),
        "capability must not be empty"
    );
    ensure!(
        !model_pattern.trim().is_empty(),
        "model_pattern must not be empty"
    );

    let policy = RoutingPolicy::new(policy_id, capability, model_pattern)
        .with_enabled(enabled)
        .with_priority(priority)
        .with_ordered_provider_ids(ordered_provider_ids.to_vec());

    Ok(match default_provider_id {
        Some(default_provider_id) if !default_provider_id.trim().is_empty() => {
            policy.with_default_provider_id(default_provider_id)
        }
        _ => policy,
    })
}

pub async fn persist_routing_policy(
    store: &dyn AdminStore,
    policy: &RoutingPolicy,
) -> Result<RoutingPolicy> {
    store.insert_routing_policy(policy).await
}

pub async fn list_routing_policies(store: &dyn AdminStore) -> Result<Vec<RoutingPolicy>> {
    store.list_routing_policies().await
}

pub async fn simulate_route_with_store(
    store: &dyn AdminStore,
    capability: &str,
    model: &str,
) -> Result<RoutingDecision> {
    let mut model_candidate_ids: Vec<String> = store
        .list_models()
        .await?
        .into_iter()
        .filter(|entry| entry.external_name == model)
        .map(|entry| entry.provider_id)
        .collect();

    model_candidate_ids.sort();
    model_candidate_ids.dedup();

    let policies = store.list_routing_policies().await?;
    let matched_policy = select_policy(&policies, capability, model);

    if model_candidate_ids.is_empty() {
        if let Some(policy) = matched_policy {
            let available_provider_ids = store
                .list_providers()
                .await?
                .into_iter()
                .map(|provider| provider.id)
                .collect::<Vec<_>>();
            let candidate_ids = policy
                .declared_provider_ids()
                .into_iter()
                .filter(|provider_id| available_provider_ids.iter().any(|id| id == provider_id))
                .collect::<Vec<_>>();

            if !candidate_ids.is_empty() {
                return Ok(
                    RoutingDecision::new(candidate_ids[0].clone(), candidate_ids)
                        .with_matched_policy_id(policy.policy_id.clone()),
                );
            }
        }

        return simulate_route(capability, model);
    }

    let candidate_ids = match matched_policy {
        Some(policy) => policy.rank_candidates(&model_candidate_ids),
        None => model_candidate_ids,
    };

    let decision = RoutingDecision::new(candidate_ids[0].clone(), candidate_ids);
    Ok(match matched_policy {
        Some(policy) => decision.with_matched_policy_id(policy.policy_id.clone()),
        None => decision,
    })
}
