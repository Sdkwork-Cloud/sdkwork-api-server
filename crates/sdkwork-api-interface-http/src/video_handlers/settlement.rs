use super::*;
use crate::gateway_commercial::{
    begin_gateway_commercial_admission, current_billing_timestamp_ms,
    next_gateway_commercial_record_id,
    record_gateway_usage_for_project_with_route_key_and_tokens_reference_and_media_with_context,
    resolve_gateway_billing_settlement, BillingMediaMetrics, GatewayCommercialAdmission,
    GatewayCommercialAdmissionDecision, GatewayCommercialAdmissionSpec,
};
use axum::http::HeaderMap;
use sdkwork_api_app_billing::CaptureAccountHoldInput;
use sdkwork_api_app_gateway::PlannedExecutionUsageContext;
use sdkwork_api_domain_catalog::ModelPriceRecord;
use sdkwork_api_domain_usage::{RequestMeterFactRecord, RequestStatus, UsageCaptureStatus};

const PENDING_VIDEO_ESTIMATED_SECONDS: f64 = 60.0;
const VIDEO_PROTOCOL_FAMILY: &str = "openai";

#[derive(Debug, Clone, Default)]
pub(crate) struct PendingVideoCommercialPlan {
    pub(crate) admission: Option<GatewayCommercialAdmission>,
    pub(crate) usage_context: Option<PlannedExecutionUsageContext>,
    pub(crate) model_price: Option<ModelPriceRecord>,
}

pub(crate) async fn plan_pending_video_commercial_workflow(
    state: &GatewayApiState,
    request_context: &IdentityGatewayRequestContext,
    tenant_id: &str,
    project_id: &str,
    route_key: &str,
    quota_units: u64,
) -> Result<PendingVideoCommercialPlan, Response> {
    let usage_context = planned_execution_usage_context_for_route(
        state.store.as_ref(),
        tenant_id,
        project_id,
        "videos",
        route_key,
    )
    .await
    .map_err(|_| {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to plan video commercial usage context",
        )
            .into_response()
    })?;
    let model_price =
        find_single_active_video_model_price(state.store.as_ref(), &usage_context.provider_id)
            .await
            .map_err(|_| {
                (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to resolve active video model pricing",
                )
                    .into_response()
            })?;

    let Some(model_price) = model_price else {
        match enforce_project_quota(state.store.as_ref(), project_id, quota_units).await {
            Ok(Some(response)) => return Err(response),
            Ok(None) => {}
            Err(_) => {
                return Err((
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to evaluate quota",
                )
                    .into_response())
            }
        }

        return Ok(PendingVideoCommercialPlan {
            admission: None,
            usage_context: Some(usage_context),
            model_price: None,
        });
    };

    let quoted_amount =
        video_charge_from_model_price(&model_price, PENDING_VIDEO_ESTIMATED_SECONDS).map_err(
            |_| {
                (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to estimate pending video commercial charge",
                )
                    .into_response()
            },
        )?;
    let admission = match begin_gateway_commercial_admission(
        state,
        request_context,
        GatewayCommercialAdmissionSpec { quoted_amount },
    )
    .await
    {
        Ok(GatewayCommercialAdmissionDecision::Canonical(admission)) => Some(admission),
        Ok(GatewayCommercialAdmissionDecision::LegacyQuota) => {
            match enforce_project_quota(state.store.as_ref(), project_id, quota_units).await {
                Ok(Some(response)) => return Err(response),
                Ok(None) => {}
                Err(_) => {
                    return Err((
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        "failed to evaluate quota",
                    )
                        .into_response())
                }
            }
            None
        }
        Err(response) => return Err(response),
    };

    Ok(PendingVideoCommercialPlan {
        admission,
        usage_context: Some(usage_context),
        model_price: Some(model_price),
    })
}

pub(crate) async fn persist_pending_video_meter_fact(
    state: &GatewayApiState,
    request_context: &IdentityGatewayRequestContext,
    headers: &HeaderMap,
    response: &Value,
    plan: &PendingVideoCommercialPlan,
) -> Result<(), Response> {
    let (Some(admission), Some(usage_context), Some(model_price)) = (
        plan.admission.as_ref(),
        plan.usage_context.as_ref(),
        plan.model_price.as_ref(),
    ) else {
        return Ok(());
    };
    let Some(reference_id) = video_response_reference_id(response) else {
        return Err(bad_gateway_openai_response(
            "upstream video mutation response missing usage id",
        ));
    };
    let Some(commercial_billing) = state.commercial_billing.as_ref() else {
        return Ok(());
    };
    let Some(account) = commercial_billing
        .resolve_payable_account_for_gateway_request_context(request_context)
        .await
        .map_err(|_| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "failed to resolve payable account",
            )
                .into_response()
        })?
    else {
        return Ok(());
    };

    let now_ms = current_billing_timestamp_ms().map_err(|_| {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to persist pending video meter fact",
        )
            .into_response()
    })?;
    let request_trace_id = headers
        .get("x-request-id")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);
    let model = video_response_model(response).unwrap_or(model_price.model_id.as_str());
    let owner = Some(format!(
        "project:{}:{}",
        request_context.tenant_id(),
        request_context.project_id()
    ));
    let fact = RequestMeterFactRecord::new(
        admission.request_id,
        account.tenant_id,
        account.organization_id,
        account.user_id,
        account.account_id,
        "api_key",
        "videos",
        usage_context.channel_id.as_deref().unwrap_or("openai"),
        model,
        &usage_context.provider_id,
    )
    .with_api_key_id(request_context.canonical_api_key_id)
    .with_api_key_hash(Some(request_context.api_key_hash().to_owned()))
    .with_platform(Some("gateway".to_owned()))
    .with_owner(owner)
    .with_request_trace_id(request_trace_id.clone())
    .with_gateway_request_ref(request_trace_id)
    .with_upstream_request_ref(Some(reference_id.to_owned()))
    .with_protocol_family(VIDEO_PROTOCOL_FAMILY)
    .with_request_status(RequestStatus::Running)
    .with_usage_capture_status(UsageCaptureStatus::Pending)
    .with_estimated_credit_hold(admission.billing_settlement.customer_charge)
    .with_started_at_ms(now_ms)
    .with_created_at_ms(now_ms)
    .with_updated_at_ms(now_ms);

    let Some(account_store) = state.store.account_kernel_store() else {
        return Err((
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "account kernel store is unavailable for pending video settlement",
        )
            .into_response());
    };
    account_store
        .insert_request_meter_fact(&fact)
        .await
        .map_err(|_| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "failed to persist pending video meter fact",
            )
                .into_response()
        })?;
    Ok(())
}

pub(crate) async fn reconcile_pending_video_retrieval(
    state: &GatewayApiState,
    request_context: &IdentityGatewayRequestContext,
    tenant_id: &str,
    project_id: &str,
    response: &Value,
) -> Result<bool, Response> {
    let Some(reference_id) = video_response_reference_id(response) else {
        return Ok(false);
    };
    let pending_fact =
        load_latest_video_meter_fact(state, tenant_id, project_id, reference_id).await?;

    let Some(fact) = pending_fact else {
        return Ok(false);
    };
    if fact.usage_capture_status != UsageCaptureStatus::Pending
        || fact.request_status != RequestStatus::Running
    {
        return Ok(true);
    }
    if !video_response_is_completed(response) {
        return Ok(true);
    }

    let duration_seconds =
        video_response_duration_seconds(response).unwrap_or(PENDING_VIDEO_ESTIMATED_SECONDS);
    let model = video_response_model(response).unwrap_or(fact.model_code.as_str());
    let model_price = find_video_model_price(state.store.as_ref(), model, &fact.provider_code)
        .await
        .map_err(|_| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "failed to resolve pending video pricing",
            )
                .into_response()
        })?
        .ok_or_else(|| bad_gateway_openai_response("failed to resolve pending video pricing"))?;
    let raw_amount = video_charge_from_model_price(&model_price, duration_seconds)
        .map_err(|_| bad_gateway_openai_response("failed to resolve pending video pricing"))?;
    let billing_settlement = resolve_gateway_billing_settlement(
        state.store.as_ref(),
        request_context.api_key_group_id(),
        None,
        raw_amount,
    )
    .await
    .map_err(|_| {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to evaluate pending video billing settlement",
        )
            .into_response()
    })?;
    let settled_at_ms = current_billing_timestamp_ms().map_err(|_| {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to finalize pending video settlement",
        )
            .into_response()
    })?;

    if billing_settlement.customer_charge > f64::EPSILON {
        let Some(commercial_billing) = state.commercial_billing.as_ref() else {
            return Err((
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "failed to finalize pending video settlement",
            )
                .into_response());
        };
        commercial_billing
            .capture_account_hold(CaptureAccountHoldInput {
                request_settlement_id: next_gateway_commercial_record_id(settled_at_ms),
                request_id: fact.request_id,
                captured_quantity: billing_settlement.customer_charge,
                provider_cost_amount: billing_settlement.upstream_cost,
                retail_charge_amount: billing_settlement.customer_charge,
                settled_at_ms,
            })
            .await
            .map_err(|_| {
                (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to finalize pending video settlement",
                )
                    .into_response()
            })?;
    }

    let mut updated_fact = fact.clone();
    updated_fact.model_code = model.to_owned();
    updated_fact.request_status = RequestStatus::Succeeded;
    updated_fact.usage_capture_status = UsageCaptureStatus::Reconciled;
    updated_fact.actual_credit_charge = Some(billing_settlement.customer_charge);
    updated_fact.actual_provider_cost = Some(billing_settlement.upstream_cost);
    updated_fact.finished_at_ms = Some(settled_at_ms);
    updated_fact.updated_at_ms = settled_at_ms;
    let Some(account_store) = state.store.account_kernel_store() else {
        return Err((
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "account kernel store is unavailable for pending video settlement",
        )
            .into_response());
    };
    account_store
        .insert_request_meter_fact(&updated_fact)
        .await
        .map_err(|_| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "failed to update pending video meter fact",
            )
                .into_response()
        })?;

    let usage_context = PlannedExecutionUsageContext {
        provider_id: fact.provider_code.clone(),
        channel_id: optional_non_empty(&fact.channel_code),
        api_key_group_id: request_context.api_key_group_id().map(ToOwned::to_owned),
        applied_routing_profile_id: None,
        compiled_routing_snapshot_id: None,
        fallback_reason: None,
        latency_ms: None,
        reference_amount: None,
    };
    record_gateway_usage_for_project_with_route_key_and_tokens_reference_and_media_with_context(
        state.store.as_ref(),
        tenant_id,
        project_id,
        "videos",
        model,
        model,
        duration_seconds.max(1.0).ceil() as u64,
        raw_amount,
        None,
        Some(reference_id),
        BillingMediaMetrics {
            video_seconds: duration_seconds,
            ..BillingMediaMetrics::default()
        },
        Some(&usage_context),
    )
    .await
    .map_err(|_| {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response()
    })?;

    Ok(true)
}

pub(crate) async fn pending_video_provider_id(
    state: &GatewayApiState,
    tenant_id: &str,
    project_id: &str,
    reference_id: &str,
) -> Result<Option<String>, Response> {
    Ok(
        load_latest_video_meter_fact(state, tenant_id, project_id, reference_id)
            .await?
            .map(|fact| fact.provider_code),
    )
}

pub(crate) fn video_response_is_processing(response: &Value) -> bool {
    video_response_status(response).is_some_and(|status| status.eq_ignore_ascii_case("processing"))
}

pub(crate) fn video_response_is_completed(response: &Value) -> bool {
    video_response_status(response).is_some_and(|status| status.eq_ignore_ascii_case("completed"))
}

pub(crate) fn video_response_reference_id(response: &Value) -> Option<&str> {
    response.get("id").and_then(Value::as_str).or_else(|| {
        video_response_item(response)?
            .get("id")
            .and_then(Value::as_str)
    })
}

pub(crate) fn video_response_model(response: &Value) -> Option<&str> {
    response.get("model").and_then(Value::as_str).or_else(|| {
        video_response_item(response)?
            .get("model")
            .and_then(Value::as_str)
    })
}

pub(crate) fn video_response_duration_seconds(response: &Value) -> Option<f64> {
    response
        .get("duration_seconds")
        .and_then(Value::as_f64)
        .or_else(|| {
            video_response_item(response)?
                .get("duration_seconds")
                .and_then(Value::as_f64)
        })
}

fn video_response_status(response: &Value) -> Option<&str> {
    response.get("status").and_then(Value::as_str).or_else(|| {
        video_response_item(response)?
            .get("status")
            .and_then(Value::as_str)
    })
}

fn video_response_item(response: &Value) -> Option<&Value> {
    match response
        .get("data")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
    {
        Some([item]) => Some(item),
        _ => None,
    }
}

fn optional_non_empty(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_owned())
    }
}

fn video_charge_from_model_price(
    price: &ModelPriceRecord,
    video_seconds: f64,
) -> anyhow::Result<f64> {
    let seconds = video_seconds.max(0.0);
    let amount = match price.price_unit.as_str() {
        "per_minute_video" => price.request_price + ((seconds / 60.0) * price.input_price),
        "per_second_video" => price.request_price + (seconds * price.input_price),
        _ => anyhow::bail!("unsupported video price unit {}", price.price_unit),
    };
    Ok(amount.max(0.0))
}

async fn find_single_active_video_model_price(
    store: &dyn AdminStore,
    provider_id: &str,
) -> anyhow::Result<Option<ModelPriceRecord>> {
    let mut prices = store
        .list_model_prices()
        .await?
        .into_iter()
        .filter(|price| price.proxy_provider_id == provider_id && price.is_active)
        .filter(|price| {
            matches!(
                price.price_unit.as_str(),
                "per_minute_video" | "per_second_video"
            )
        })
        .collect::<Vec<_>>();
    prices.sort_by(|left, right| left.model_id.cmp(&right.model_id));
    match prices.len() {
        0 => Ok(None),
        1 => Ok(prices.pop()),
        _ => Ok(None),
    }
}

async fn find_video_model_price(
    store: &dyn AdminStore,
    model_id: &str,
    provider_id: &str,
) -> anyhow::Result<Option<ModelPriceRecord>> {
    Ok(store.list_model_prices().await?.into_iter().find(|price| {
        price.is_active
            && price.model_id == model_id
            && price.proxy_provider_id == provider_id
            && matches!(
                price.price_unit.as_str(),
                "per_minute_video" | "per_second_video"
            )
    }))
}

async fn load_latest_video_meter_fact(
    state: &GatewayApiState,
    tenant_id: &str,
    project_id: &str,
    reference_id: &str,
) -> Result<Option<RequestMeterFactRecord>, Response> {
    let expected_owner = format!("project:{tenant_id}:{project_id}");
    let Some(account_store) = state.store.account_kernel_store() else {
        return Err((
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "account kernel store is unavailable for pending video settlement",
        )
            .into_response());
    };
    account_store
        .list_request_meter_facts()
        .await
        .map_err(|_| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "failed to load pending video meter facts",
            )
                .into_response()
        })
        .map(|facts| {
            facts
                .into_iter()
                .filter(|fact| fact.upstream_request_ref.as_deref() == Some(reference_id))
                .filter(|fact| fact.owner.as_deref() == Some(expected_owner.as_str()))
                .max_by_key(|fact| fact.updated_at_ms)
        })
}
