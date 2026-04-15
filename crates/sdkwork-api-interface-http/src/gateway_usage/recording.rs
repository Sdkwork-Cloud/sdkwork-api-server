#![allow(clippy::too_many_arguments)]

use super::*;

pub(crate) async fn record_gateway_usage_for_project(
    store: &dyn AdminStore,
    tenant_id: &str,
    project_id: &str,
    capability: &str,
    model: &str,
    units: u64,
    amount: f64,
) -> anyhow::Result<()> {
    record_gateway_usage_for_project_with_route_key(
        store, tenant_id, project_id, capability, model, model, units, amount,
    )
    .await
}

pub(crate) async fn record_gateway_usage_for_project_with_route_key(
    store: &dyn AdminStore,
    tenant_id: &str,
    project_id: &str,
    capability: &str,
    route_key: &str,
    usage_model: &str,
    units: u64,
    amount: f64,
) -> anyhow::Result<()> {
    record_gateway_usage_for_project_with_route_key_and_tokens(
        store,
        tenant_id,
        project_id,
        capability,
        route_key,
        usage_model,
        units,
        amount,
        None,
    )
    .await
}

pub(crate) async fn record_gateway_usage_for_project_with_route_key_and_tokens(
    store: &dyn AdminStore,
    tenant_id: &str,
    project_id: &str,
    capability: &str,
    route_key: &str,
    usage_model: &str,
    units: u64,
    amount: f64,
    token_usage: Option<TokenUsageMetrics>,
) -> anyhow::Result<()> {
    let usage_context = planned_execution_usage_context_for_route(
        store, tenant_id, project_id, capability, route_key,
    )
    .await?;
    let token_usage = token_usage.unwrap_or_default();
    let api_key_hash =
        current_gateway_request_context().map(|context| context.api_key_hash().to_owned());
    persist_usage_record_with_tokens_and_facts(
        store,
        project_id,
        usage_model,
        &usage_context.provider_id,
        units,
        amount,
        token_usage.input_tokens,
        token_usage.output_tokens,
        token_usage.total_tokens,
        api_key_hash.as_deref(),
        usage_context.channel_id.as_deref(),
        current_gateway_request_latency_ms().or(usage_context.latency_ms),
        usage_context.reference_amount,
    )
    .await?;
    persist_ledger_entry(store, project_id, units, amount).await?;
    Ok(())
}
