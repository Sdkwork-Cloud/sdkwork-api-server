use super::*;
use anyhow::anyhow;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use sdkwork_api_app_billing::{
    create_billing_event, persist_billing_event, CaptureAccountHoldInput, CreateAccountHoldInput,
    CreateBillingEventInput, ReleaseAccountHoldInput,
};
use sdkwork_api_app_gateway::PlannedExecutionUsageContext;
use sdkwork_api_domain_billing::BillingAccountingMode;
use sdkwork_api_policy_billing::{
    builtin_billing_policy_registry, BillingPolicyExecutionInput, BillingPolicyExecutionResult,
    GROUP_DEFAULT_BILLING_POLICY_ID,
};
const GATEWAY_COMMERCIAL_HOLD_TTL_MS: u64 = 5 * 60 * 1000;
const GATEWAY_COMMERCIAL_ID_SEQUENCE_BITS: u32 = 15;
const GATEWAY_COMMERCIAL_ID_SEQUENCE_MASK: u64 = (1_u64 << GATEWAY_COMMERCIAL_ID_SEQUENCE_BITS) - 1;

static GATEWAY_COMMERCIAL_ID_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone)]
pub(crate) struct GatewayCommercialAdmission {
    request_id: u64,
    billing_settlement: BillingPolicyExecutionResult,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct GatewayCommercialAdmissionSpec {
    pub(crate) quoted_amount: f64,
}

pub(crate) enum GatewayCommercialAdmissionDecision {
    Canonical(GatewayCommercialAdmission),
    LegacyQuota,
}

pub(crate) async fn enforce_project_quota<S>(
    store: &S,
    project_id: &str,
    requested_units: u64,
) -> anyhow::Result<Option<Response>>
where
    S: sdkwork_api_app_billing::BillingQuotaStore + ?Sized,
{
    let evaluation = check_quota(store, project_id, requested_units).await?;
    if evaluation.allowed {
        Ok(None)
    } else {
        Ok(Some(quota_exceeded_response(project_id, &evaluation)))
    }
}

fn quota_exceeded_response(project_id: &str, evaluation: &QuotaCheckResult) -> Response {
    let mut error = OpenAiErrorResponse::new(
        quota_exceeded_message(project_id, evaluation),
        "insufficient_quota",
    );
    error.error.code = Some("quota_exceeded".to_owned());
    (StatusCode::TOO_MANY_REQUESTS, Json(error)).into_response()
}

fn bad_gateway_openai_response(message: impl Into<String>) -> Response {
    let mut error = OpenAiErrorResponse::new(message, "server_error");
    error.error.code = Some("bad_gateway".to_owned());
    (StatusCode::BAD_GATEWAY, Json(error)).into_response()
}

fn not_found_openai_response(message: impl Into<String>) -> Response {
    let mut error = OpenAiErrorResponse::new(message, "invalid_request_error");
    error.error.code = Some("not_found".to_owned());
    (StatusCode::NOT_FOUND, Json(error)).into_response()
}

fn invalid_request_openai_response(
    message: impl Into<String>,
    code: impl Into<String>,
) -> Response {
    let mut error = OpenAiErrorResponse::new(message, "invalid_request_error");
    error.error.code = Some(code.into());
    (StatusCode::BAD_REQUEST, Json(error)).into_response()
}

fn local_gateway_error_response(error: anyhow::Error, not_found_message: &'static str) -> Response {
    if error.to_string().to_ascii_lowercase().contains("not found") {
        return not_found_openai_response(not_found_message);
    }

    bad_gateway_openai_response("failed to process local gateway fallback")
}

fn quota_exceeded_message(project_id: &str, evaluation: &QuotaCheckResult) -> String {
    match (evaluation.policy_id.as_deref(), evaluation.limit_units) {
        (Some(policy_id), Some(limit_units)) => format!(
            "Quota exceeded for project {project_id} under policy {policy_id}: requested {} units with {} already used against a limit of {limit_units}.",
            evaluation.requested_units, evaluation.used_units,
        ),
        (_, Some(limit_units)) => format!(
            "Quota exceeded for project {project_id}: requested {} units with {} already used against a limit of {limit_units}.",
            evaluation.requested_units, evaluation.used_units,
        ),
        _ => format!(
            "Quota exceeded for project {project_id}: requested {} units with {} already used.",
            evaluation.requested_units, evaluation.used_units,
        ),
    }
}

fn next_gateway_commercial_record_id(now_ms: u64) -> u64 {
    let sequence = GATEWAY_COMMERCIAL_ID_SEQUENCE.fetch_add(1, Ordering::Relaxed)
        & GATEWAY_COMMERCIAL_ID_SEQUENCE_MASK;
    compose_gateway_commercial_record_id(now_ms, sequence)
}

fn compose_gateway_commercial_record_id(now_ms: u64, sequence: u64) -> u64 {
    (now_ms << GATEWAY_COMMERCIAL_ID_SEQUENCE_BITS)
        | (sequence & GATEWAY_COMMERCIAL_ID_SEQUENCE_MASK)
}

pub(crate) async fn begin_gateway_commercial_admission(
    state: &GatewayApiState,
    request_context: &IdentityGatewayRequestContext,
    spec: GatewayCommercialAdmissionSpec,
) -> Result<GatewayCommercialAdmissionDecision, Response> {
    let Some(commercial_billing) = state.commercial_billing.as_ref() else {
        return Ok(GatewayCommercialAdmissionDecision::LegacyQuota);
    };

    let billing_settlement = resolve_gateway_billing_settlement(
        state.store.as_ref(),
        request_context.api_key_group_id(),
        None,
        spec.quoted_amount,
    )
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "failed to evaluate commercial billing admission",
        )
            .into_response()
    })?;

    if billing_settlement.accounting_mode != BillingAccountingMode::PlatformCredit
        || billing_settlement.customer_charge <= f64::EPSILON
    {
        return Ok(GatewayCommercialAdmissionDecision::LegacyQuota);
    }

    let Some(account) = commercial_billing
        .resolve_payable_account_for_gateway_request_context(request_context)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to resolve payable account",
            )
                .into_response()
        })?
    else {
        return Ok(GatewayCommercialAdmissionDecision::LegacyQuota);
    };

    let now_ms = current_billing_timestamp_ms().map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "failed to allocate commercial billing hold",
        )
            .into_response()
    })?;
    let hold_plan = commercial_billing
        .plan_account_hold(
            account.account_id,
            billing_settlement.customer_charge,
            now_ms,
        )
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to plan commercial billing hold",
            )
                .into_response()
        })?;
    if !hold_plan.sufficient_balance {
        return Err(commercial_balance_exceeded_response(
            request_context.project_id(),
            account.account_id,
            hold_plan.requested_quantity,
            hold_plan.covered_quantity,
            hold_plan.shortfall_quantity,
        ));
    }

    let request_id = next_gateway_commercial_record_id(now_ms);
    let hold_id = next_gateway_commercial_record_id(now_ms);
    let hold_allocation_start_id = next_gateway_commercial_record_id(now_ms);
    commercial_billing
        .create_account_hold(CreateAccountHoldInput {
            hold_id,
            hold_allocation_start_id,
            request_id,
            account_id: account.account_id,
            requested_quantity: billing_settlement.customer_charge,
            expires_at_ms: now_ms + GATEWAY_COMMERCIAL_HOLD_TTL_MS,
            now_ms,
        })
        .await
        .map_err(|error| {
            if looks_like_insufficient_account_balance(&error) {
                commercial_balance_exceeded_response(
                    request_context.project_id(),
                    account.account_id,
                    billing_settlement.customer_charge,
                    0.0,
                    billing_settlement.customer_charge,
                )
            } else {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to create commercial billing hold",
                )
                    .into_response()
            }
        })?;

    Ok(GatewayCommercialAdmissionDecision::Canonical(
        GatewayCommercialAdmission {
            request_id,
            billing_settlement,
        },
    ))
}

pub(crate) async fn capture_gateway_commercial_admission(
    state: &GatewayApiState,
    admission: &GatewayCommercialAdmission,
) -> Result<(), Response> {
    let Some(commercial_billing) = state.commercial_billing.as_ref() else {
        return Ok(());
    };

    let settled_at_ms = current_billing_timestamp_ms().map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "failed to finalize commercial billing settlement",
        )
            .into_response()
    })?;
    commercial_billing
        .capture_account_hold(CaptureAccountHoldInput {
            request_settlement_id: next_gateway_commercial_record_id(settled_at_ms),
            request_id: admission.request_id,
            captured_quantity: admission.billing_settlement.customer_charge,
            provider_cost_amount: admission.billing_settlement.upstream_cost,
            retail_charge_amount: admission.billing_settlement.customer_charge,
            settled_at_ms,
        })
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to finalize commercial billing settlement",
            )
                .into_response()
        })?;
    Ok(())
}

pub(crate) async fn release_gateway_commercial_admission(
    state: &GatewayApiState,
    admission: &GatewayCommercialAdmission,
) -> Result<(), Response> {
    let Some(commercial_billing) = state.commercial_billing.as_ref() else {
        return Ok(());
    };

    let released_at_ms = current_billing_timestamp_ms().map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "failed to release commercial billing hold",
        )
            .into_response()
    })?;
    commercial_billing
        .release_account_hold(ReleaseAccountHoldInput {
            request_id: admission.request_id,
            released_at_ms,
        })
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to release commercial billing hold",
            )
                .into_response()
        })?;
    Ok(())
}

fn commercial_balance_exceeded_response(
    project_id: &str,
    account_id: u64,
    requested_quantity: f64,
    covered_quantity: f64,
    shortfall_quantity: f64,
) -> Response {
    let mut error = OpenAiErrorResponse::new(
        format!(
            "Insufficient balance for project {project_id} on primary account {account_id}: requested {requested_quantity:.4} credits, available {covered_quantity:.4}, shortfall {shortfall_quantity:.4}."
        ),
        "payment_required",
    );
    error.error.code = Some("insufficient_balance".to_owned());
    (StatusCode::PAYMENT_REQUIRED, Json(error)).into_response()
}

#[cfg(test)]
mod gateway_commercial_id_tests {
    use super::compose_gateway_commercial_record_id;

    #[test]
    fn gateway_commercial_record_ids_leave_headroom_for_ledger_suffixes() {
        let future_now_ms = 4_102_444_800_000_u64;
        let max_sequence = 0x0000_7fff_u64;
        let record_id = compose_gateway_commercial_record_id(future_now_ms, max_sequence);
        let derived_ledger_id = record_id.saturating_mul(10).saturating_add(4);

        assert!(
            i64::try_from(derived_ledger_id).is_ok(),
            "commercial record id {record_id} must stay representable after ledger suffix expansion"
        );
    }
}

fn looks_like_insufficient_account_balance(error: &anyhow::Error) -> bool {
    error
        .to_string()
        .to_ascii_lowercase()
        .contains("insufficient available balance")
}

pub(crate) async fn record_gateway_usage_for_project(
    store: &dyn AdminStore,
    tenant_id: &str,
    project_id: &str,
    capability: &str,
    model: &str,
    units: u64,
    amount: f64,
) -> anyhow::Result<()> {
    record_gateway_usage_for_project_with_context(
        store, tenant_id, project_id, capability, model, units, amount, None,
    )
    .await
}

pub(crate) async fn record_gateway_usage_for_project_with_context(
    store: &dyn AdminStore,
    tenant_id: &str,
    project_id: &str,
    capability: &str,
    model: &str,
    units: u64,
    amount: f64,
    usage_context_override: Option<&PlannedExecutionUsageContext>,
) -> anyhow::Result<()> {
    record_gateway_usage_for_project_with_route_key_and_reference_id_with_context(
        store,
        tenant_id,
        project_id,
        capability,
        model,
        model,
        units,
        amount,
        None,
        usage_context_override,
    )
    .await
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct TokenUsageMetrics {
    input_tokens: u64,
    output_tokens: u64,
    total_tokens: u64,
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct BillingMediaMetrics {
    pub(crate) image_count: u64,
    pub(crate) audio_seconds: f64,
    pub(crate) video_seconds: f64,
    pub(crate) music_seconds: f64,
}

fn json_u64(value: Option<&Value>) -> Option<u64> {
    value.and_then(|value| value.as_u64())
}

pub(crate) fn extract_token_usage_metrics(response: &Value) -> Option<TokenUsageMetrics> {
    if let Some(usage) = response.get("usage") {
        let input_tokens = json_u64(usage.get("prompt_tokens"))
            .or_else(|| json_u64(usage.get("input_tokens")))
            .unwrap_or(0);
        let output_tokens = json_u64(usage.get("completion_tokens"))
            .or_else(|| json_u64(usage.get("output_tokens")))
            .unwrap_or(0);
        let total_tokens = json_u64(usage.get("total_tokens"))
            .unwrap_or_else(|| input_tokens.saturating_add(output_tokens));

        if input_tokens > 0 || output_tokens > 0 || total_tokens > 0 {
            return Some(TokenUsageMetrics {
                input_tokens,
                output_tokens,
                total_tokens,
            });
        }
    }

    let input_tokens = json_u64(response.get("input_tokens")).unwrap_or(0);
    let output_tokens = json_u64(response.get("output_tokens")).unwrap_or(0);
    let total_tokens = json_u64(response.get("total_tokens"))
        .unwrap_or_else(|| input_tokens.saturating_add(output_tokens));

    if input_tokens > 0 || output_tokens > 0 || total_tokens > 0 {
        return Some(TokenUsageMetrics {
            input_tokens,
            output_tokens,
            total_tokens,
        });
    }

    None
}

pub(crate) fn response_usage_id_or_single_data_item_id(response: &Value) -> Option<&str> {
    response.get("id").and_then(Value::as_str).or_else(|| {
        match response
            .get("data")
            .and_then(Value::as_array)
            .map(Vec::as_slice)
        {
            Some([item]) => item.get("id").and_then(Value::as_str),
            _ => None,
        }
    })
}

fn image_count_from_response(response: &Value) -> u64 {
    response
        .get("data")
        .and_then(Value::as_array)
        .and_then(|data| u64::try_from(data.len()).ok())
        .unwrap_or(0)
}

pub(crate) fn music_seconds_from_response(response: &Value) -> f64 {
    response
        .get("duration_seconds")
        .and_then(Value::as_f64)
        .or_else(|| {
            response
                .get("data")
                .and_then(Value::as_array)
                .and_then(|data| match data.as_slice() {
                    [item] => item.get("duration_seconds").and_then(Value::as_f64),
                    _ => None,
                })
        })
        .unwrap_or(0.0)
}

pub(crate) fn music_billing_units(music_seconds: f64) -> u64 {
    music_seconds.max(1.0).ceil() as u64
}

pub(crate) fn music_billing_amount(music_seconds: f64) -> f64 {
    music_seconds.max(1.0) * 0.001
}

fn current_billing_timestamp_ms() -> anyhow::Result<u64> {
    Ok(SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as u64)
}

fn build_gateway_billing_event_id(
    project_id: &str,
    capability: &str,
    route_key: &str,
    provider_id: &str,
    reference_id: Option<&str>,
    created_at_ms: u64,
) -> String {
    format!(
        "bill_evt:{project_id}:{capability}:{route_key}:{provider_id}:{}:{created_at_ms}",
        reference_id.unwrap_or("none")
    )
}

fn billing_modality_for_capability(capability: &str) -> &'static str {
    match capability {
        "responses" => "multimodal",
        "images" | "image_edits" | "image_variations" => "image",
        "audio" | "speech" | "transcriptions" | "translations" => "audio",
        "videos" => "video",
        "music" => "music",
        _ => "text",
    }
}

pub(crate) async fn record_gateway_usage_for_project_with_route_key_and_reference_id(
    store: &dyn AdminStore,
    tenant_id: &str,
    project_id: &str,
    capability: &str,
    route_key: &str,
    usage_model: &str,
    units: u64,
    amount: f64,
    reference_id: Option<&str>,
) -> anyhow::Result<()> {
    record_gateway_usage_for_project_with_route_key_and_reference_id_with_context(
        store,
        tenant_id,
        project_id,
        capability,
        route_key,
        usage_model,
        units,
        amount,
        reference_id,
        None,
    )
    .await
}

async fn record_gateway_usage_for_project_with_route_key_and_reference_id_with_context(
    store: &dyn AdminStore,
    tenant_id: &str,
    project_id: &str,
    capability: &str,
    route_key: &str,
    usage_model: &str,
    units: u64,
    amount: f64,
    reference_id: Option<&str>,
    usage_context_override: Option<&PlannedExecutionUsageContext>,
) -> anyhow::Result<()> {
    record_gateway_usage_for_project_with_route_key_and_tokens_and_reference_with_context(
        store,
        tenant_id,
        project_id,
        capability,
        route_key,
        usage_model,
        units,
        amount,
        None,
        reference_id,
        usage_context_override,
    )
    .await
}

pub(crate) async fn record_gateway_usage_for_project_with_media_and_reference_id(
    store: &dyn AdminStore,
    tenant_id: &str,
    project_id: &str,
    capability: &str,
    model: &str,
    units: u64,
    amount: f64,
    media_metrics: BillingMediaMetrics,
    reference_id: Option<&str>,
) -> anyhow::Result<()> {
    record_gateway_usage_for_project_with_route_key_and_tokens_reference_and_media_with_context(
        store,
        tenant_id,
        project_id,
        capability,
        model,
        model,
        units,
        amount,
        None,
        reference_id,
        media_metrics,
        None,
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
    record_gateway_usage_for_project_with_route_key_and_tokens_and_reference_with_context(
        store,
        tenant_id,
        project_id,
        capability,
        route_key,
        usage_model,
        units,
        amount,
        None,
        None,
        None,
    )
    .await
}

pub(crate) async fn record_gateway_usage_for_project_with_route_key_and_tokens_and_reference(
    store: &dyn AdminStore,
    tenant_id: &str,
    project_id: &str,
    capability: &str,
    route_key: &str,
    usage_model: &str,
    units: u64,
    amount: f64,
    token_usage: Option<TokenUsageMetrics>,
    reference_id: Option<&str>,
) -> anyhow::Result<()> {
    record_gateway_usage_for_project_with_route_key_and_tokens_and_reference_with_context(
        store,
        tenant_id,
        project_id,
        capability,
        route_key,
        usage_model,
        units,
        amount,
        token_usage,
        reference_id,
        None,
    )
    .await
}

pub(crate) async fn record_gateway_usage_for_project_with_route_key_and_tokens_and_reference_with_context(
    store: &dyn AdminStore,
    tenant_id: &str,
    project_id: &str,
    capability: &str,
    route_key: &str,
    usage_model: &str,
    units: u64,
    amount: f64,
    token_usage: Option<TokenUsageMetrics>,
    reference_id: Option<&str>,
    usage_context_override: Option<&PlannedExecutionUsageContext>,
) -> anyhow::Result<()> {
    record_gateway_usage_for_project_with_route_key_and_tokens_reference_and_media_with_context(
        store,
        tenant_id,
        project_id,
        capability,
        route_key,
        usage_model,
        units,
        amount,
        token_usage,
        reference_id,
        BillingMediaMetrics::default(),
        usage_context_override,
    )
    .await
}

async fn record_gateway_usage_for_project_with_route_key_and_tokens_reference_and_media_with_context(
    store: &dyn AdminStore,
    tenant_id: &str,
    project_id: &str,
    capability: &str,
    route_key: &str,
    usage_model: &str,
    units: u64,
    amount: f64,
    token_usage: Option<TokenUsageMetrics>,
    reference_id: Option<&str>,
    media_metrics: BillingMediaMetrics,
    usage_context_override: Option<&PlannedExecutionUsageContext>,
) -> anyhow::Result<()> {
    let usage_context = match usage_context_override {
        Some(context) => (*context).clone(),
        None => {
            planned_execution_usage_context_for_route(
                store, tenant_id, project_id, capability, route_key,
            )
            .await?
        }
    };
    let token_usage = token_usage.unwrap_or_default();
    let request_context = current_gateway_request_context();
    let api_key_hash = request_context
        .as_ref()
        .map(|context| context.api_key_hash().to_owned());
    let billing_settlement = resolve_gateway_billing_settlement(
        store,
        request_context
            .as_ref()
            .and_then(|context| context.api_key_group_id()),
        usage_context.reference_amount,
        amount,
    )
    .await?;
    let latency_ms = current_gateway_request_latency_ms().or(usage_context.latency_ms);
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
        latency_ms,
        usage_context.reference_amount,
    )
    .await?;
    let created_at_ms = current_billing_timestamp_ms()?;
    let billing_event = create_billing_event(CreateBillingEventInput {
        event_id: &build_gateway_billing_event_id(
            project_id,
            capability,
            route_key,
            &usage_context.provider_id,
            reference_id,
            created_at_ms,
        ),
        tenant_id,
        project_id,
        api_key_group_id: usage_context.api_key_group_id.as_deref(),
        capability,
        route_key,
        usage_model,
        provider_id: &usage_context.provider_id,
        accounting_mode: billing_settlement.accounting_mode,
        operation_kind: capability,
        modality: billing_modality_for_capability(capability),
        api_key_hash: api_key_hash.as_deref(),
        channel_id: usage_context.channel_id.as_deref(),
        reference_id,
        latency_ms,
        units,
        request_count: 1,
        input_tokens: token_usage.input_tokens,
        output_tokens: token_usage.output_tokens,
        total_tokens: token_usage.total_tokens,
        cache_read_tokens: 0,
        cache_write_tokens: 0,
        image_count: media_metrics.image_count,
        audio_seconds: media_metrics.audio_seconds,
        video_seconds: media_metrics.video_seconds,
        music_seconds: media_metrics.music_seconds,
        upstream_cost: billing_settlement.upstream_cost,
        customer_charge: billing_settlement.customer_charge,
        applied_routing_profile_id: usage_context.applied_routing_profile_id.as_deref(),
        compiled_routing_snapshot_id: usage_context.compiled_routing_snapshot_id.as_deref(),
        fallback_reason: usage_context.fallback_reason.as_deref(),
        created_at_ms,
    })?;
    persist_billing_event(store, &billing_event).await?;
    persist_ledger_entry(store, project_id, units, amount).await?;
    Ok(())
}

async fn resolve_gateway_billing_settlement(
    store: &dyn AdminStore,
    api_key_group_id: Option<&str>,
    upstream_cost: Option<f64>,
    customer_charge: f64,
) -> anyhow::Result<BillingPolicyExecutionResult> {
    let group_default_accounting_mode =
        load_api_key_group_default_accounting_mode(store, api_key_group_id).await?;
    let registry = builtin_billing_policy_registry();
    let plugin = registry
        .resolve(GROUP_DEFAULT_BILLING_POLICY_ID)
        .ok_or_else(|| anyhow!("missing builtin group-default billing policy plugin"))?;

    plugin.execute(BillingPolicyExecutionInput {
        api_key_group_default_accounting_mode: group_default_accounting_mode.as_deref(),
        default_accounting_mode: BillingAccountingMode::PlatformCredit,
        upstream_cost,
        customer_charge,
    })
}

async fn load_api_key_group_default_accounting_mode(
    store: &dyn AdminStore,
    api_key_group_id: Option<&str>,
) -> anyhow::Result<Option<String>> {
    let Some(api_key_group_id) = api_key_group_id
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return Ok(None);
    };

    Ok(store
        .find_api_key_group(api_key_group_id)
        .await?
        .and_then(|group| group.default_accounting_mode))
}
