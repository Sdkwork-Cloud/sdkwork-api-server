#![allow(clippy::result_large_err, clippy::too_many_arguments)]

use crate::gateway_commercial::{
    begin_gateway_commercial_admission, capture_gateway_commercial_admission,
    extract_token_usage_metrics as extract_commercial_token_usage_metrics,
    record_gateway_usage_for_project_with_context,
    record_gateway_usage_for_project_with_route_key_and_tokens_and_reference_with_context,
    release_gateway_commercial_admission, GatewayCommercialAdmissionDecision,
    GatewayCommercialAdmissionSpec, TokenUsageMetrics as CommercialTokenUsageMetrics,
};
use crate::gateway_prelude::*;
use crate::GatewayApiState;
use std::sync::OnceLock;

use crate::compat_gemini_handlers::{parse_gemini_compat_tail, GeminiCompatAction};
use sdkwork_api_app_gateway::{
    execute_raw_json_provider_operation_from_planned_execution_context_with_options,
    execute_raw_json_provider_operation_with_runtime,
    execute_raw_stream_provider_operation_from_planned_execution_context_with_options,
    execute_raw_stream_provider_operation_with_runtime, persist_planned_execution_decision_log,
    planned_execution_provider_context_for_route_without_log,
    relay_chat_completion_from_planned_execution_context_with_options,
    relay_chat_completion_from_store_with_execution_context,
    relay_chat_completion_stream_from_planned_execution_context_with_options,
    relay_chat_completion_stream_from_store_with_execution_context,
    relay_count_response_input_tokens_from_planned_execution_context,
    PlannedExecutionProviderContext, PlannedExecutionUsageContext,
};
use sdkwork_api_contract_openai::chat_completions::ChatCompletionResponse;

fn local_chat_completion_gateway_result(
    tenant_id: &str,
    project_id: &str,
    model: &str,
) -> anyhow::Result<ChatCompletionResponse> {
    if model.trim().is_empty() {
        return Err(anyhow::anyhow!("Chat completion model is required."));
    }

    create_chat_completion(tenant_id, project_id, model)
}

fn json_u64(value: Option<&Value>) -> Option<u64> {
    value.and_then(Value::as_u64)
}

fn local_anthropic_count_tokens_response(
    tenant_id: &str,
    project_id: &str,
    model: &str,
) -> Response {
    match count_response_input_tokens(tenant_id, project_id, model) {
        Ok(response) => match serde_json::to_value(response) {
            Ok(value) => Json(openai_count_tokens_to_anthropic(&value)).into_response(),
            Err(_) => anthropic_bad_gateway_response("failed to process local anthropic fallback"),
        },
        Err(error) => {
            let message = error.to_string();
            if local_gateway_error_is_invalid_request(&message) {
                return anthropic_invalid_request_response(message);
            }

            anthropic_bad_gateway_response("failed to process local anthropic fallback")
        }
    }
}

fn local_anthropic_invalid_model_response(error: anyhow::Error) -> Response {
    let message = error.to_string();
    if local_gateway_error_is_invalid_request(&message) {
        return anthropic_invalid_request_response(message);
    }

    anthropic_bad_gateway_response("failed to process local anthropic fallback")
}

fn local_anthropic_chat_completion_result(
    tenant_id: &str,
    project_id: &str,
    model: &str,
) -> std::result::Result<Value, Response> {
    match local_chat_completion_gateway_result(tenant_id, project_id, model) {
        Ok(response) => match serde_json::to_value(response) {
            Ok(value) => Ok(openai_chat_response_to_anthropic(&value)),
            Err(_) => Err(anthropic_bad_gateway_response(
                "failed to process local anthropic fallback",
            )),
        },
        Err(error) => Err(local_anthropic_invalid_model_response(error)),
    }
}

fn local_anthropic_stream_result(
    _tenant_id: &str,
    _project_id: &str,
    _model: &str,
) -> std::result::Result<Response, Response> {
    Err(anthropic_invalid_request_response(
        "Local anthropic message streaming fallback is not supported without an upstream provider.",
    ))
}

pub(crate) async fn anthropic_messages_handler(
    request_context: StatelessGatewayRequest,
    headers: HeaderMap,
    ExtractJson(payload): ExtractJson<Value>,
) -> Response {
    let request = match anthropic_request_to_chat_completion(&payload) {
        Ok(request) => request,
        Err(error) => return anthropic_invalid_request_response(error.to_string()),
    };
    if request.model.trim().is_empty() {
        return anthropic_invalid_request_response("Chat completion model is required.");
    }
    let options = anthropic_request_options(&headers);

    if request.stream.unwrap_or(false) {
        match relay_standard_protocol_stream_request_stateless(
            &request_context,
            StandardProtocolKind::Anthropic,
            "/v1/messages",
            &headers,
            &payload,
        )
        .await
        {
            Ok(Some(StandardProtocolStreamRelayOutcome::Stream(response))) => {
                return upstream_passthrough_response(response);
            }
            Ok(Some(StandardProtocolStreamRelayOutcome::Error(response))) => return response,
            Ok(None) => {}
            Err(_) => {
                return anthropic_bad_gateway_response(
                    "failed to relay upstream anthropic message stream",
                );
            }
        }

        match relay_raw_protocol_stream_request_stateless(
            &request_context,
            StandardProtocolKind::Anthropic,
            "anthropic.messages.create",
            Vec::new(),
            &headers,
            &payload,
        )
        .await
        {
            Ok(Some(response)) => return upstream_passthrough_response(response),
            Ok(None) => {}
            Err(_) => {
                return anthropic_bad_gateway_response(
                    "failed to relay upstream anthropic message stream",
                );
            }
        }

        match relay_stateless_stream_request_with_options(
            &request_context,
            ProviderRequest::ChatCompletionsStream(&request),
            &options,
        )
        .await
        {
            Ok(Some(response)) => {
                upstream_passthrough_response(anthropic_stream_from_openai(response))
            }
            Ok(None) => match local_anthropic_stream_result(
                request_context.tenant_id(),
                request_context.project_id(),
                &request.model,
            ) {
                Ok(response) | Err(response) => response,
            },
            Err(_) => {
                anthropic_bad_gateway_response("failed to relay upstream anthropic message stream")
            }
        }
    } else {
        match relay_standard_protocol_json_request_stateless(
            &request_context,
            StandardProtocolKind::Anthropic,
            "/v1/messages",
            &headers,
            &payload,
        )
        .await
        {
            Ok(Some(StandardProtocolJsonRelayOutcome::Json(response))) => {
                return Json(response).into_response();
            }
            Ok(Some(StandardProtocolJsonRelayOutcome::Error(response))) => return response,
            Ok(None) => {}
            Err(_) => {
                return anthropic_bad_gateway_response(
                    "failed to relay upstream anthropic message",
                );
            }
        }

        match relay_raw_protocol_json_request_stateless(
            &request_context,
            StandardProtocolKind::Anthropic,
            "anthropic.messages.create",
            Vec::new(),
            &headers,
            &payload,
        )
        .await
        {
            Ok(Some(response)) => return Json(response).into_response(),
            Ok(None) => {}
            Err(_) => {
                return anthropic_bad_gateway_response("failed to relay upstream anthropic message")
            }
        }

        match relay_stateless_json_request_with_options(
            &request_context,
            ProviderRequest::ChatCompletions(&request),
            &options,
        )
        .await
        {
            Ok(Some(response)) => {
                Json(openai_chat_response_to_anthropic(&response)).into_response()
            }
            Ok(None) => match local_anthropic_chat_completion_result(
                request_context.tenant_id(),
                request_context.project_id(),
                &request.model,
            ) {
                Ok(response) => Json(response).into_response(),
                Err(response) => response,
            },
            Err(_) => anthropic_bad_gateway_response("failed to relay upstream anthropic message"),
        }
    }
}

pub(crate) async fn anthropic_count_tokens_handler(
    request_context: StatelessGatewayRequest,
    headers: HeaderMap,
    ExtractJson(payload): ExtractJson<Value>,
) -> Response {
    let request = match anthropic_count_tokens_request(&payload) {
        Ok(request) => request,
        Err(error) => return anthropic_invalid_request_response(error.to_string()),
    };
    if request.model.trim().is_empty() {
        return anthropic_invalid_request_response("Response model is required.");
    }

    match relay_standard_protocol_json_request_stateless(
        &request_context,
        StandardProtocolKind::Anthropic,
        "/v1/messages/count_tokens",
        &headers,
        &payload,
    )
    .await
    {
        Ok(Some(StandardProtocolJsonRelayOutcome::Json(response))) => {
            return Json(response).into_response();
        }
        Ok(Some(StandardProtocolJsonRelayOutcome::Error(response))) => return response,
        Ok(None) => {}
        Err(_) => {
            return anthropic_bad_gateway_response(
                "failed to relay upstream anthropic count tokens request",
            );
        }
    }

    match relay_raw_protocol_json_request_stateless(
        &request_context,
        StandardProtocolKind::Anthropic,
        "anthropic.messages.count_tokens",
        Vec::new(),
        &headers,
        &payload,
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return anthropic_bad_gateway_response(
                "failed to relay upstream anthropic count tokens request",
            );
        }
    }

    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ResponsesInputTokens(&request),
    )
    .await
    {
        Ok(Some(response)) => Json(openai_count_tokens_to_anthropic(&response)).into_response(),
        Ok(None) => local_anthropic_count_tokens_response(
            request_context.tenant_id(),
            request_context.project_id(),
            &request.model,
        ),
        Err(_) => anthropic_bad_gateway_response(
            "failed to relay upstream anthropic count tokens request",
        ),
    }
}

fn local_gemini_invalid_model_response(error: anyhow::Error) -> Response {
    let message = error.to_string();
    if local_gateway_error_is_invalid_request(&message) {
        return gemini_invalid_request_response(message);
    }

    gemini_bad_gateway_response("failed to process local gemini fallback")
}

fn local_gemini_chat_completion_result(
    tenant_id: &str,
    project_id: &str,
    model: &str,
) -> std::result::Result<Value, Response> {
    match local_chat_completion_gateway_result(tenant_id, project_id, model) {
        Ok(response) => match serde_json::to_value(response) {
            Ok(value) => Ok(openai_chat_response_to_gemini(&value)),
            Err(_) => Err(gemini_bad_gateway_response(
                "failed to process local gemini fallback",
            )),
        },
        Err(error) => Err(local_gemini_invalid_model_response(error)),
    }
}

fn local_gemini_stream_result(
    _tenant_id: &str,
    _project_id: &str,
    _model: &str,
) -> std::result::Result<Response, Response> {
    Err(gemini_invalid_request_response(
        "Local Gemini streamGenerateContent fallback is not supported without an upstream provider.",
    ))
}

fn local_gemini_count_tokens_result(
    tenant_id: &str,
    project_id: &str,
    model: &str,
) -> std::result::Result<Value, Response> {
    match count_response_input_tokens(tenant_id, project_id, model) {
        Ok(response) => match serde_json::to_value(response) {
            Ok(value) => Ok(openai_count_tokens_to_gemini(&value)),
            Err(_) => Err(gemini_bad_gateway_response(
                "failed to process local gemini fallback",
            )),
        },
        Err(error) => {
            let message = error.to_string();
            if local_gateway_error_is_invalid_request(&message) {
                return Err(gemini_invalid_request_response(message));
            }

            Err(gemini_bad_gateway_response(
                "failed to process local gemini fallback",
            ))
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StandardProtocolKind {
    Anthropic,
    Gemini,
}

impl StandardProtocolKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Anthropic => "anthropic",
            Self::Gemini => "gemini",
        }
    }

    fn auth_header_name(self) -> &'static str {
        match self {
            Self::Anthropic => "x-api-key",
            Self::Gemini => "x-goog-api-key",
        }
    }

    fn forwarded_header_names(self) -> &'static [&'static str] {
        match self {
            Self::Anthropic => &["anthropic-version", "anthropic-beta"],
            Self::Gemini => &[],
        }
    }
}

enum StandardProtocolJsonRelayOutcome {
    Json(Value),
    Error(Response),
}

enum StandardProtocolStreamRelayOutcome {
    Stream(ProviderStreamOutput),
    Error(Response),
}

fn build_standard_protocol_upstream_url(base_url: &str, path_and_query: &str) -> String {
    format!("{}{}", base_url.trim_end_matches('/'), path_and_query)
}

fn standard_protocol_status(status: reqwest::StatusCode) -> StatusCode {
    StatusCode::from_u16(status.as_u16()).unwrap_or(StatusCode::BAD_GATEWAY)
}

fn standard_protocol_http_client() -> &'static reqwest::Client {
    static STANDARD_PROTOCOL_HTTP_CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    STANDARD_PROTOCOL_HTTP_CLIENT.get_or_init(reqwest::Client::new)
}

async fn standard_protocol_error_response(response: reqwest::Response) -> Response {
    let status = standard_protocol_status(response.status());
    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);
    let body = response.bytes().await.unwrap_or_default();

    let mut builder = Response::builder().status(status);
    if let Some(content_type) = content_type {
        builder = builder.header(header::CONTENT_TYPE, content_type);
    }
    match builder.body(Body::from(body)) {
        Ok(response) => response,
        Err(_) => bad_gateway_openai_response("failed to process upstream standard protocol error"),
    }
}

async fn send_standard_protocol_request(
    protocol: StandardProtocolKind,
    base_url: &str,
    api_key: &str,
    path_and_query: &str,
    headers: &HeaderMap,
    payload: &Value,
) -> anyhow::Result<reqwest::Response> {
    let mut request = standard_protocol_http_client()
        .post(build_standard_protocol_upstream_url(
            base_url,
            path_and_query,
        ))
        .header(protocol.auth_header_name(), api_key)
        .json(payload);

    for header_name in protocol.forwarded_header_names() {
        if let Some(value) = headers
            .get(*header_name)
            .and_then(|value| value.to_str().ok())
        {
            request = request.header(*header_name, value);
        }
    }

    Ok(request.send().await?)
}

async fn relay_standard_protocol_json_request(
    protocol: StandardProtocolKind,
    base_url: &str,
    api_key: &str,
    path_and_query: &str,
    headers: &HeaderMap,
    payload: &Value,
) -> anyhow::Result<StandardProtocolJsonRelayOutcome> {
    let response = send_standard_protocol_request(
        protocol,
        base_url,
        api_key,
        path_and_query,
        headers,
        payload,
    )
    .await?;
    if !response.status().is_success() {
        return Ok(StandardProtocolJsonRelayOutcome::Error(
            standard_protocol_error_response(response).await,
        ));
    }

    Ok(StandardProtocolJsonRelayOutcome::Json(
        response.json().await?,
    ))
}

async fn relay_standard_protocol_stream_request(
    protocol: StandardProtocolKind,
    base_url: &str,
    api_key: &str,
    path_and_query: &str,
    headers: &HeaderMap,
    payload: &Value,
) -> anyhow::Result<StandardProtocolStreamRelayOutcome> {
    let response = send_standard_protocol_request(
        protocol,
        base_url,
        api_key,
        path_and_query,
        headers,
        payload,
    )
    .await?;
    if !response.status().is_success() {
        return Ok(StandardProtocolStreamRelayOutcome::Error(
            standard_protocol_error_response(response).await,
        ));
    }

    Ok(StandardProtocolStreamRelayOutcome::Stream(
        ProviderStreamOutput::from_reqwest_response(response),
    ))
}

async fn relay_standard_protocol_json_request_stateless(
    request_context: &StatelessGatewayRequest,
    protocol: StandardProtocolKind,
    path_and_query: &str,
    headers: &HeaderMap,
    payload: &Value,
) -> anyhow::Result<Option<StandardProtocolJsonRelayOutcome>> {
    let Some(upstream) = request_context.upstream() else {
        return Ok(None);
    };
    if upstream.protocol_kind() != protocol.as_str() {
        return Ok(None);
    }

    Ok(Some(
        relay_standard_protocol_json_request(
            protocol,
            upstream.base_url(),
            upstream.api_key(),
            path_and_query,
            headers,
            payload,
        )
        .await?,
    ))
}

async fn relay_standard_protocol_stream_request_stateless(
    request_context: &StatelessGatewayRequest,
    protocol: StandardProtocolKind,
    path_and_query: &str,
    headers: &HeaderMap,
    payload: &Value,
) -> anyhow::Result<Option<StandardProtocolStreamRelayOutcome>> {
    let Some(upstream) = request_context.upstream() else {
        return Ok(None);
    };
    if upstream.protocol_kind() != protocol.as_str() {
        return Ok(None);
    }

    Ok(Some(
        relay_standard_protocol_stream_request(
            protocol,
            upstream.base_url(),
            upstream.api_key(),
            path_and_query,
            headers,
            payload,
        )
        .await?,
    ))
}

async fn planned_stateful_provider_context(
    state: &GatewayApiState,
    request_context: &CompatAuthenticatedGatewayRequest,
    capability: &str,
    route_key: &str,
) -> anyhow::Result<Option<PlannedExecutionProviderContext>> {
    planned_execution_provider_context_for_route_without_log(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        capability,
        route_key,
    )
    .await
}

async fn relay_standard_protocol_json_request_from_planned_context(
    state: &GatewayApiState,
    request_context: &CompatAuthenticatedGatewayRequest,
    capability: &str,
    route_key: &str,
    protocol: StandardProtocolKind,
    path_and_query: &str,
    headers: &HeaderMap,
    payload: &Value,
    planned_context: Option<&PlannedExecutionProviderContext>,
) -> anyhow::Result<
    Option<(
        StandardProtocolJsonRelayOutcome,
        PlannedExecutionUsageContext,
    )>,
> {
    let Some(planned_context) = planned_context else {
        return Ok(None);
    };
    if planned_context.provider.protocol_kind != protocol.as_str() {
        return Ok(None);
    }

    persist_planned_execution_decision_log(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        capability,
        route_key,
        &planned_context.decision,
    )
    .await?;

    Ok(Some((
        relay_standard_protocol_json_request(
            protocol,
            &planned_context.provider.base_url,
            &planned_context.api_key,
            path_and_query,
            headers,
            payload,
        )
        .await?,
        planned_context.usage_context.clone(),
    )))
}

async fn relay_standard_protocol_stream_request_from_planned_context(
    state: &GatewayApiState,
    request_context: &CompatAuthenticatedGatewayRequest,
    capability: &str,
    route_key: &str,
    protocol: StandardProtocolKind,
    path_and_query: &str,
    headers: &HeaderMap,
    payload: &Value,
    planned_context: Option<&PlannedExecutionProviderContext>,
) -> anyhow::Result<
    Option<(
        StandardProtocolStreamRelayOutcome,
        PlannedExecutionUsageContext,
    )>,
> {
    let Some(planned_context) = planned_context else {
        return Ok(None);
    };
    if planned_context.provider.protocol_kind != protocol.as_str() {
        return Ok(None);
    }

    persist_planned_execution_decision_log(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        capability,
        route_key,
        &planned_context.decision,
    )
    .await?;

    Ok(Some((
        relay_standard_protocol_stream_request(
            protocol,
            &planned_context.provider.base_url,
            &planned_context.api_key,
            path_and_query,
            headers,
            payload,
        )
        .await?,
        planned_context.usage_context.clone(),
    )))
}

fn compat_protocol_headers(
    protocol: StandardProtocolKind,
    headers: &HeaderMap,
) -> std::collections::HashMap<String, String> {
    protocol
        .forwarded_header_names()
        .iter()
        .filter_map(|header_name| {
            headers
                .get(*header_name)
                .and_then(|value| value.to_str().ok())
                .map(|value| ((*header_name).to_owned(), value.to_owned()))
        })
        .collect()
}

async fn relay_raw_protocol_json_request_stateless(
    request_context: &StatelessGatewayRequest,
    protocol: StandardProtocolKind,
    operation: &str,
    path_params: Vec<String>,
    headers: &HeaderMap,
    payload: &Value,
) -> anyhow::Result<Option<Value>> {
    let Some(upstream) = request_context.upstream() else {
        return Ok(None);
    };
    if upstream.protocol_kind() == protocol.as_str() {
        return Ok(None);
    }

    execute_raw_json_provider_operation_with_runtime(
        upstream.runtime_key(),
        upstream.base_url(),
        upstream.api_key(),
        operation,
        path_params,
        payload.clone(),
        compat_protocol_headers(protocol, headers),
    )
}

async fn relay_raw_protocol_stream_request_stateless(
    request_context: &StatelessGatewayRequest,
    protocol: StandardProtocolKind,
    operation: &str,
    path_params: Vec<String>,
    headers: &HeaderMap,
    payload: &Value,
) -> anyhow::Result<Option<ProviderStreamOutput>> {
    let Some(upstream) = request_context.upstream() else {
        return Ok(None);
    };
    if upstream.protocol_kind() == protocol.as_str() {
        return Ok(None);
    }

    execute_raw_stream_provider_operation_with_runtime(
        upstream.runtime_key(),
        upstream.base_url(),
        upstream.api_key(),
        operation,
        path_params,
        payload.clone(),
        compat_protocol_headers(protocol, headers),
    )
    .await
}

async fn relay_raw_protocol_json_request_from_planned_context(
    state: &GatewayApiState,
    request_context: &CompatAuthenticatedGatewayRequest,
    capability: &str,
    route_key: &str,
    protocol: StandardProtocolKind,
    operation: &str,
    path_params: Vec<String>,
    headers: &HeaderMap,
    payload: &Value,
    planned_context: Option<&PlannedExecutionProviderContext>,
) -> anyhow::Result<Option<(Value, PlannedExecutionUsageContext)>> {
    let Some(planned_context) = planned_context else {
        return Ok(None);
    };
    if planned_context.provider.protocol_kind() == protocol.as_str() {
        return Ok(None);
    }
    if planned_context.execution.local_fallback {
        persist_planned_execution_decision_log(
            state.store.as_ref(),
            request_context.tenant_id(),
            request_context.project_id(),
            capability,
            route_key,
            &planned_context.decision,
        )
        .await?;
        return Err(anyhow::anyhow!(
            "explicit runtime binding is unavailable for selected provider"
        ));
    }

    match execute_raw_json_provider_operation_from_planned_execution_context_with_options(
        state.store.as_ref(),
        planned_context,
        capability,
        operation,
        path_params,
        payload.clone(),
        compat_protocol_headers(protocol, headers),
        &ProviderRequestOptions::default(),
    )
    .await
    {
        Ok(Some(response)) => {
            persist_planned_execution_decision_log(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                capability,
                route_key,
                &planned_context.decision,
            )
            .await?;
            Ok(Some((response, planned_context.usage_context.clone())))
        }
        Ok(None) => Ok(None),
        Err(error) => {
            persist_planned_execution_decision_log(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                capability,
                route_key,
                &planned_context.decision,
            )
            .await?;
            Err(error)
        }
    }
}

async fn relay_raw_protocol_stream_request_from_planned_context(
    state: &GatewayApiState,
    request_context: &CompatAuthenticatedGatewayRequest,
    capability: &str,
    route_key: &str,
    protocol: StandardProtocolKind,
    operation: &str,
    path_params: Vec<String>,
    headers: &HeaderMap,
    payload: &Value,
    planned_context: Option<&PlannedExecutionProviderContext>,
) -> anyhow::Result<Option<(ProviderStreamOutput, PlannedExecutionUsageContext)>> {
    let Some(planned_context) = planned_context else {
        return Ok(None);
    };
    if planned_context.provider.protocol_kind() == protocol.as_str() {
        return Ok(None);
    }
    if planned_context.execution.local_fallback {
        persist_planned_execution_decision_log(
            state.store.as_ref(),
            request_context.tenant_id(),
            request_context.project_id(),
            capability,
            route_key,
            &planned_context.decision,
        )
        .await?;
        return Err(anyhow::anyhow!(
            "explicit runtime binding is unavailable for selected provider"
        ));
    }

    match execute_raw_stream_provider_operation_from_planned_execution_context_with_options(
        state.store.as_ref(),
        planned_context,
        capability,
        operation,
        path_params,
        payload.clone(),
        compat_protocol_headers(protocol, headers),
        &ProviderRequestOptions::default(),
    )
    .await
    {
        Ok(Some(response)) => {
            persist_planned_execution_decision_log(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                capability,
                route_key,
                &planned_context.decision,
            )
            .await?;
            Ok(Some((response, planned_context.usage_context.clone())))
        }
        Ok(None) => Ok(None),
        Err(error) => {
            persist_planned_execution_decision_log(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                capability,
                route_key,
                &planned_context.decision,
            )
            .await?;
            Err(error)
        }
    }
}

fn extract_anthropic_token_usage_metrics(response: &Value) -> Option<CommercialTokenUsageMetrics> {
    extract_commercial_token_usage_metrics(response)
}

fn extract_gemini_token_usage_metrics(response: &Value) -> Option<CommercialTokenUsageMetrics> {
    let usage = response.get("usageMetadata")?;
    let input_tokens = json_u64(usage.get("promptTokenCount")).unwrap_or(0);
    let output_tokens = json_u64(usage.get("candidatesTokenCount")).unwrap_or(0);
    let total_tokens = json_u64(usage.get("totalTokenCount"))
        .unwrap_or_else(|| input_tokens.saturating_add(output_tokens));
    if input_tokens == 0 && output_tokens == 0 && total_tokens == 0 {
        return None;
    }

    let normalized = serde_json::json!({
        "usage": {
            "prompt_tokens": input_tokens,
            "completion_tokens": output_tokens,
            "total_tokens": total_tokens
        }
    });
    extract_commercial_token_usage_metrics(&normalized)
}

pub(crate) async fn anthropic_messages_with_state_handler(
    request_context: CompatAuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    headers: HeaderMap,
    ExtractJson(payload): ExtractJson<Value>,
) -> Response {
    let request = match anthropic_request_to_chat_completion(&payload) {
        Ok(request) => request,
        Err(error) => return anthropic_invalid_request_response(error.to_string()),
    };
    if request.model.trim().is_empty() {
        return anthropic_invalid_request_response("Chat completion model is required.");
    }
    let options = anthropic_request_options(&headers);

    let commercial_admission = match begin_gateway_commercial_admission(
        &state,
        request_context.context(),
        GatewayCommercialAdmissionSpec {
            quoted_amount: 0.10,
        },
    )
    .await
    {
        Ok(GatewayCommercialAdmissionDecision::Canonical(admission)) => Some(admission),
        Ok(GatewayCommercialAdmissionDecision::LegacyQuota) => {
            match enforce_project_quota(state.store.as_ref(), request_context.project_id(), 100)
                .await
            {
                Ok(Some(response)) => return response,
                Ok(None) => {}
                Err(_) => {
                    return (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        "failed to evaluate quota",
                    )
                        .into_response();
                }
            }
            None
        }
        Err(response) => return response,
    };

    if request.stream.unwrap_or(false) {
        let planned_context = match planned_stateful_provider_context(
            &state,
            &request_context,
            "chat_completion",
            &request.model,
        )
        .await
        {
            Ok(planned_context) => planned_context,
            Err(_) => {
                if let Some(admission) = commercial_admission.as_ref() {
                    if let Err(response) =
                        release_gateway_commercial_admission(&state, admission).await
                    {
                        return response;
                    }
                }
                return anthropic_bad_gateway_response(
                    "failed to plan upstream anthropic message stream",
                );
            }
        };

        match relay_standard_protocol_stream_request_from_planned_context(
            &state,
            &request_context,
            "chat_completion",
            &request.model,
            StandardProtocolKind::Anthropic,
            "/v1/messages",
            &headers,
            &payload,
            planned_context.as_ref(),
        )
        .await
        {
            Ok(Some((StandardProtocolStreamRelayOutcome::Stream(response), usage_context))) => {
                if let Some(admission) = commercial_admission.as_ref() {
                    if let Err(response) =
                        capture_gateway_commercial_admission(&state, admission).await
                    {
                        return response;
                    }
                }
                if record_gateway_usage_for_project_with_context(
                    state.store.as_ref(),
                    request_context.tenant_id(),
                    request_context.project_id(),
                    "chat_completion",
                    &request.model,
                    100,
                    0.10,
                    Some(&usage_context),
                )
                .await
                .is_err()
                {
                    return (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        "failed to record usage",
                    )
                        .into_response();
                }

                return upstream_passthrough_response(response);
            }
            Ok(Some((StandardProtocolStreamRelayOutcome::Error(response), _))) => {
                if let Some(admission) = commercial_admission.as_ref() {
                    if let Err(release_response) =
                        release_gateway_commercial_admission(&state, admission).await
                    {
                        return release_response;
                    }
                }
                return response;
            }
            Ok(None) => {}
            Err(_) => {
                if let Some(admission) = commercial_admission.as_ref() {
                    if let Err(response) =
                        release_gateway_commercial_admission(&state, admission).await
                    {
                        return response;
                    }
                }
                return anthropic_bad_gateway_response(
                    "failed to relay upstream anthropic message stream",
                );
            }
        }

        match relay_raw_protocol_stream_request_from_planned_context(
            &state,
            &request_context,
            "chat_completion",
            &request.model,
            StandardProtocolKind::Anthropic,
            "anthropic.messages.create",
            Vec::new(),
            &headers,
            &payload,
            planned_context.as_ref(),
        )
        .await
        {
            Ok(Some((response, usage_context))) => {
                if let Some(admission) = commercial_admission.as_ref() {
                    if let Err(response) =
                        capture_gateway_commercial_admission(&state, admission).await
                    {
                        return response;
                    }
                }
                if record_gateway_usage_for_project_with_context(
                    state.store.as_ref(),
                    request_context.tenant_id(),
                    request_context.project_id(),
                    "chat_completion",
                    &request.model,
                    100,
                    0.10,
                    Some(&usage_context),
                )
                .await
                .is_err()
                {
                    return (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        "failed to record usage",
                    )
                        .into_response();
                }

                return upstream_passthrough_response(response);
            }
            Ok(None) => {}
            Err(_) => {
                if let Some(admission) = commercial_admission.as_ref() {
                    if let Err(response) =
                        release_gateway_commercial_admission(&state, admission).await
                    {
                        return response;
                    }
                }
                return anthropic_bad_gateway_response(
                    "failed to relay upstream anthropic message stream",
                );
            }
        }

        if let Some(planned_context) = planned_context.as_ref() {
            match relay_chat_completion_stream_from_planned_execution_context_with_options(
                state.store.as_ref(),
                &state.secret_manager,
                request_context.tenant_id(),
                request_context.project_id(),
                &request,
                &options,
                planned_context,
            )
            .await
            {
                Ok(execution) => {
                    let usage_context = execution.usage_context;
                    if let Some(response) = execution.response {
                        if let Some(admission) = commercial_admission.as_ref() {
                            if let Err(response) =
                                capture_gateway_commercial_admission(&state, admission).await
                            {
                                return response;
                            }
                        }
                        if record_gateway_usage_for_project_with_context(
                            state.store.as_ref(),
                            request_context.tenant_id(),
                            request_context.project_id(),
                            "chat_completion",
                            &request.model,
                            100,
                            0.10,
                            usage_context.as_ref(),
                        )
                        .await
                        .is_err()
                        {
                            return (
                                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                                "failed to record usage",
                            )
                                .into_response();
                        }

                        return upstream_passthrough_response(anthropic_stream_from_openai(
                            response,
                        ));
                    }
                }
                Err(_) => {
                    if let Some(admission) = commercial_admission.as_ref() {
                        if let Err(response) =
                            release_gateway_commercial_admission(&state, admission).await
                        {
                            return response;
                        }
                    }
                    return anthropic_bad_gateway_response(
                        "failed to relay upstream anthropic message stream",
                    );
                }
            }
        } else {
            match relay_chat_completion_stream_from_store_with_execution_context(
                state.store.as_ref(),
                &state.secret_manager,
                request_context.tenant_id(),
                request_context.project_id(),
                &request,
                &options,
            )
            .await
            {
                Ok(execution) => {
                    let usage_context = execution.usage_context;
                    if let Some(response) = execution.response {
                        if let Some(admission) = commercial_admission.as_ref() {
                            if let Err(response) =
                                capture_gateway_commercial_admission(&state, admission).await
                            {
                                return response;
                            }
                        }
                        if record_gateway_usage_for_project_with_context(
                            state.store.as_ref(),
                            request_context.tenant_id(),
                            request_context.project_id(),
                            "chat_completion",
                            &request.model,
                            100,
                            0.10,
                            usage_context.as_ref(),
                        )
                        .await
                        .is_err()
                        {
                            return (
                                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                                "failed to record usage",
                            )
                                .into_response();
                        }

                        return upstream_passthrough_response(anthropic_stream_from_openai(
                            response,
                        ));
                    }
                }
                Err(_) => {
                    if let Some(admission) = commercial_admission.as_ref() {
                        if let Err(response) =
                            release_gateway_commercial_admission(&state, admission).await
                        {
                            return response;
                        }
                    }
                    return anthropic_bad_gateway_response(
                        "failed to relay upstream anthropic message stream",
                    );
                }
            }
        }

        let local_response = match local_anthropic_stream_result(
            request_context.tenant_id(),
            request_context.project_id(),
            &request.model,
        ) {
            Ok(response) => response,
            Err(response) => {
                if let Some(admission) = commercial_admission.as_ref() {
                    if let Err(release_response) =
                        release_gateway_commercial_admission(&state, admission).await
                    {
                        return release_response;
                    }
                }
                return response;
            }
        };

        if let Some(admission) = commercial_admission.as_ref() {
            if let Err(release_response) =
                release_gateway_commercial_admission(&state, admission).await
            {
                return release_response;
            }
        }

        return local_response;
    }

    let planned_context = match planned_stateful_provider_context(
        &state,
        &request_context,
        "chat_completion",
        &request.model,
    )
    .await
    {
        Ok(planned_context) => planned_context,
        Err(_) => {
            if let Some(admission) = commercial_admission.as_ref() {
                if let Err(response) = release_gateway_commercial_admission(&state, admission).await
                {
                    return response;
                }
            }
            return anthropic_bad_gateway_response("failed to plan upstream anthropic message");
        }
    };

    match relay_standard_protocol_json_request_from_planned_context(
        &state,
        &request_context,
        "chat_completion",
        &request.model,
        StandardProtocolKind::Anthropic,
        "/v1/messages",
        &headers,
        &payload,
        planned_context.as_ref(),
    )
    .await
    {
        Ok(Some((StandardProtocolJsonRelayOutcome::Json(response), usage_context))) => {
            if let Some(admission) = commercial_admission.as_ref() {
                if let Err(response) = capture_gateway_commercial_admission(&state, admission).await
                {
                    return response;
                }
            }
            if record_gateway_usage_for_project_with_route_key_and_tokens_and_reference_with_context(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "chat_completion",
                &request.model,
                &request.model,
                100,
                0.10,
                extract_anthropic_token_usage_metrics(&response),
                response.get("id").and_then(Value::as_str),
                Some(&usage_context),
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(Some((StandardProtocolJsonRelayOutcome::Error(response), _))) => {
            if let Some(admission) = commercial_admission.as_ref() {
                if let Err(release_response) =
                    release_gateway_commercial_admission(&state, admission).await
                {
                    return release_response;
                }
            }
            return response;
        }
        Ok(None) => {}
        Err(_) => {
            if let Some(admission) = commercial_admission.as_ref() {
                if let Err(response) = release_gateway_commercial_admission(&state, admission).await
                {
                    return response;
                }
            }
            return anthropic_bad_gateway_response("failed to relay upstream anthropic message");
        }
    }

    match relay_raw_protocol_json_request_from_planned_context(
        &state,
        &request_context,
        "chat_completion",
        &request.model,
        StandardProtocolKind::Anthropic,
        "anthropic.messages.create",
        Vec::new(),
        &headers,
        &payload,
        planned_context.as_ref(),
    )
    .await
    {
        Ok(Some((response, usage_context))) => {
            if let Some(admission) = commercial_admission.as_ref() {
                if let Err(response) = capture_gateway_commercial_admission(&state, admission).await
                {
                    return response;
                }
            }
            if record_gateway_usage_for_project_with_route_key_and_tokens_and_reference_with_context(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "chat_completion",
                &request.model,
                &request.model,
                100,
                0.10,
                extract_anthropic_token_usage_metrics(&response),
                response.get("id").and_then(Value::as_str),
                Some(&usage_context),
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            if let Some(admission) = commercial_admission.as_ref() {
                if let Err(response) = release_gateway_commercial_admission(&state, admission).await
                {
                    return response;
                }
            }
            return anthropic_bad_gateway_response("failed to relay upstream anthropic message");
        }
    }

    if let Some(planned_context) = planned_context.as_ref() {
        match relay_chat_completion_from_planned_execution_context_with_options(
            state.store.as_ref(),
            &state.secret_manager,
            request_context.tenant_id(),
            request_context.project_id(),
            &request,
            &options,
            planned_context,
        )
        .await
        {
            Ok(execution) => {
                let usage_context = execution.usage_context;
                if let Some(response) = execution.response {
                    if let Some(admission) = commercial_admission.as_ref() {
                        if let Err(response) =
                            capture_gateway_commercial_admission(&state, admission).await
                        {
                            return response;
                        }
                    }
                    let token_usage = extract_commercial_token_usage_metrics(&response);
                    if record_gateway_usage_for_project_with_route_key_and_tokens_and_reference_with_context(
                        state.store.as_ref(),
                        request_context.tenant_id(),
                        request_context.project_id(),
                        "chat_completion",
                        &request.model,
                        &request.model,
                        100,
                        0.10,
                        token_usage,
                        response_usage_id_or_single_data_item_id(&response),
                        usage_context.as_ref(),
                    )
                    .await
                    .is_err()
                    {
                        return (
                            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                            "failed to record usage",
                        )
                            .into_response();
                    }

                    return Json(openai_chat_response_to_anthropic(&response)).into_response();
                }
            }
            Err(_) => {
                if let Some(admission) = commercial_admission.as_ref() {
                    if let Err(response) =
                        release_gateway_commercial_admission(&state, admission).await
                    {
                        return response;
                    }
                }
                return anthropic_bad_gateway_response(
                    "failed to relay upstream anthropic message",
                );
            }
        }
    } else {
        match relay_chat_completion_from_store_with_execution_context(
            state.store.as_ref(),
            &state.secret_manager,
            request_context.tenant_id(),
            request_context.project_id(),
            &request,
            &options,
        )
        .await
        {
            Ok(execution) => {
                let usage_context = execution.usage_context;
                if let Some(response) = execution.response {
                    if let Some(admission) = commercial_admission.as_ref() {
                        if let Err(response) =
                            capture_gateway_commercial_admission(&state, admission).await
                        {
                            return response;
                        }
                    }
                    let token_usage = extract_commercial_token_usage_metrics(&response);
                    if record_gateway_usage_for_project_with_route_key_and_tokens_and_reference_with_context(
                        state.store.as_ref(),
                        request_context.tenant_id(),
                        request_context.project_id(),
                        "chat_completion",
                        &request.model,
                        &request.model,
                        100,
                        0.10,
                        token_usage,
                        response_usage_id_or_single_data_item_id(&response),
                        usage_context.as_ref(),
                    )
                    .await
                    .is_err()
                    {
                        return (
                            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                            "failed to record usage",
                        )
                            .into_response();
                    }

                    return Json(openai_chat_response_to_anthropic(&response)).into_response();
                }
            }
            Err(_) => {
                if let Some(admission) = commercial_admission.as_ref() {
                    if let Err(response) =
                        release_gateway_commercial_admission(&state, admission).await
                    {
                        return response;
                    }
                }
                return anthropic_bad_gateway_response(
                    "failed to relay upstream anthropic message",
                );
            }
        }
    }

    let local_response = match local_anthropic_chat_completion_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &request.model,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if let Some(admission) = commercial_admission.as_ref() {
        if let Err(response) = capture_gateway_commercial_admission(&state, admission).await {
            return response;
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "chat_completion",
        &request.model,
        100,
        0.10,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(local_response).into_response()
}

pub(crate) async fn anthropic_count_tokens_with_state_handler(
    request_context: CompatAuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    headers: HeaderMap,
    ExtractJson(payload): ExtractJson<Value>,
) -> Response {
    let request = match anthropic_count_tokens_request(&payload) {
        Ok(request) => request,
        Err(error) => return anthropic_invalid_request_response(error.to_string()),
    };
    if request.model.trim().is_empty() {
        return anthropic_invalid_request_response("Response model is required.");
    }

    let planned_context = match planned_stateful_provider_context(
        &state,
        &request_context,
        "responses",
        &request.model,
    )
    .await
    {
        Ok(planned_context) => planned_context,
        Err(_) => {
            return anthropic_bad_gateway_response(
                "failed to plan upstream anthropic count tokens request",
            );
        }
    };

    match relay_standard_protocol_json_request_from_planned_context(
        &state,
        &request_context,
        "responses",
        &request.model,
        StandardProtocolKind::Anthropic,
        "/v1/messages/count_tokens",
        &headers,
        &payload,
        planned_context.as_ref(),
    )
    .await
    {
        Ok(Some((StandardProtocolJsonRelayOutcome::Json(response), _))) => {
            return Json(response).into_response();
        }
        Ok(Some((StandardProtocolJsonRelayOutcome::Error(response), _))) => return response,
        Ok(None) => {}
        Err(_) => {
            return anthropic_bad_gateway_response(
                "failed to relay upstream anthropic count tokens request",
            );
        }
    }

    match relay_raw_protocol_json_request_from_planned_context(
        &state,
        &request_context,
        "responses",
        &request.model,
        StandardProtocolKind::Anthropic,
        "anthropic.messages.count_tokens",
        Vec::new(),
        &headers,
        &payload,
        planned_context.as_ref(),
    )
    .await
    {
        Ok(Some((response, _))) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return anthropic_bad_gateway_response(
                "failed to relay upstream anthropic count tokens request",
            );
        }
    }

    if let Some(planned_context) = planned_context.as_ref() {
        match relay_count_response_input_tokens_from_planned_execution_context(
            state.store.as_ref(),
            request_context.tenant_id(),
            request_context.project_id(),
            &request,
            planned_context,
        )
        .await
        {
            Ok(Some(response)) => Json(openai_count_tokens_to_anthropic(&response)).into_response(),
            Ok(None) => local_anthropic_count_tokens_response(
                request_context.tenant_id(),
                request_context.project_id(),
                &request.model,
            ),
            Err(_) => anthropic_bad_gateway_response(
                "failed to relay upstream anthropic count tokens request",
            ),
        }
    } else {
        match relay_count_response_input_tokens_from_store(
            state.store.as_ref(),
            &state.secret_manager,
            request_context.tenant_id(),
            request_context.project_id(),
            &request,
        )
        .await
        {
            Ok(Some(response)) => Json(openai_count_tokens_to_anthropic(&response)).into_response(),
            Ok(None) => local_anthropic_count_tokens_response(
                request_context.tenant_id(),
                request_context.project_id(),
                &request.model,
            ),
            Err(_) => anthropic_bad_gateway_response(
                "failed to relay upstream anthropic count tokens request",
            ),
        }
    }
}

pub(crate) async fn gemini_models_compat_handler(
    request_context: StatelessGatewayRequest,
    Path(tail): Path<String>,
    ExtractJson(payload): ExtractJson<Value>,
) -> Response {
    let Some((model, action)) = parse_gemini_compat_tail(&tail) else {
        return gemini_invalid_request_response("unsupported gemini compatibility route");
    };

    match action {
        GeminiCompatAction::GenerateContent => {
            let request = match gemini_request_to_chat_completion(&model, &payload) {
                Ok(request) => request,
                Err(error) => return gemini_invalid_request_response(error.to_string()),
            };
            if request.model.trim().is_empty() {
                return gemini_invalid_request_response("Chat completion model is required.");
            }

            let headers = HeaderMap::new();
            let path = format!("/v1beta/models/{model}:generateContent");
            match relay_standard_protocol_json_request_stateless(
                &request_context,
                StandardProtocolKind::Gemini,
                &path,
                &headers,
                &payload,
            )
            .await
            {
                Ok(Some(StandardProtocolJsonRelayOutcome::Json(response))) => {
                    return Json(response).into_response();
                }
                Ok(Some(StandardProtocolJsonRelayOutcome::Error(response))) => return response,
                Ok(None) => {}
                Err(_) => {
                    return gemini_bad_gateway_response(
                        "failed to relay upstream gemini generateContent request",
                    );
                }
            }

            match relay_raw_protocol_json_request_stateless(
                &request_context,
                StandardProtocolKind::Gemini,
                "gemini.generate_content",
                vec![model.clone()],
                &headers,
                &payload,
            )
            .await
            {
                Ok(Some(response)) => return Json(response).into_response(),
                Ok(None) => {}
                Err(_) => {
                    return gemini_bad_gateway_response(
                        "failed to relay upstream gemini generateContent request",
                    );
                }
            }

            match relay_stateless_json_request(
                &request_context,
                ProviderRequest::ChatCompletions(&request),
            )
            .await
            {
                Ok(Some(response)) => {
                    Json(openai_chat_response_to_gemini(&response)).into_response()
                }
                Ok(None) => match local_gemini_chat_completion_result(
                    request_context.tenant_id(),
                    request_context.project_id(),
                    &request.model,
                ) {
                    Ok(response) => Json(response).into_response(),
                    Err(response) => response,
                },
                Err(_) => gemini_bad_gateway_response(
                    "failed to relay upstream gemini generateContent request",
                ),
            }
        }
        GeminiCompatAction::StreamGenerateContent => {
            let mut request = match gemini_request_to_chat_completion(&model, &payload) {
                Ok(request) => request,
                Err(error) => return gemini_invalid_request_response(error.to_string()),
            };
            if request.model.trim().is_empty() {
                return gemini_invalid_request_response("Chat completion model is required.");
            }
            request.stream = Some(true);

            let headers = HeaderMap::new();
            let path = format!("/v1beta/models/{model}:streamGenerateContent?alt=sse");
            match relay_standard_protocol_stream_request_stateless(
                &request_context,
                StandardProtocolKind::Gemini,
                &path,
                &headers,
                &payload,
            )
            .await
            {
                Ok(Some(StandardProtocolStreamRelayOutcome::Stream(response))) => {
                    return upstream_passthrough_response(response);
                }
                Ok(Some(StandardProtocolStreamRelayOutcome::Error(response))) => return response,
                Ok(None) => {}
                Err(_) => {
                    return gemini_bad_gateway_response(
                        "failed to relay upstream gemini streamGenerateContent request",
                    );
                }
            }

            match relay_raw_protocol_stream_request_stateless(
                &request_context,
                StandardProtocolKind::Gemini,
                "gemini.stream_generate_content",
                vec![model.clone()],
                &headers,
                &payload,
            )
            .await
            {
                Ok(Some(response)) => return upstream_passthrough_response(response),
                Ok(None) => {}
                Err(_) => {
                    return gemini_bad_gateway_response(
                        "failed to relay upstream gemini streamGenerateContent request",
                    );
                }
            }

            match relay_stateless_stream_request(
                &request_context,
                ProviderRequest::ChatCompletionsStream(&request),
            )
            .await
            {
                Ok(Some(response)) => {
                    upstream_passthrough_response(gemini_stream_from_openai(response))
                }
                Ok(None) => match local_gemini_stream_result(
                    request_context.tenant_id(),
                    request_context.project_id(),
                    &request.model,
                ) {
                    Ok(response) | Err(response) => response,
                },
                Err(_) => gemini_bad_gateway_response(
                    "failed to relay upstream gemini streamGenerateContent request",
                ),
            }
        }
        GeminiCompatAction::CountTokens => {
            let request = gemini_count_tokens_request(&model, &payload);
            if request.model.trim().is_empty() {
                return gemini_invalid_request_response("Response model is required.");
            }
            let headers = HeaderMap::new();
            let path = format!("/v1beta/models/{model}:countTokens");
            match relay_standard_protocol_json_request_stateless(
                &request_context,
                StandardProtocolKind::Gemini,
                &path,
                &headers,
                &payload,
            )
            .await
            {
                Ok(Some(StandardProtocolJsonRelayOutcome::Json(response))) => {
                    return Json(response).into_response();
                }
                Ok(Some(StandardProtocolJsonRelayOutcome::Error(response))) => return response,
                Ok(None) => {}
                Err(_) => {
                    return gemini_bad_gateway_response(
                        "failed to relay upstream gemini countTokens request",
                    );
                }
            }

            match relay_raw_protocol_json_request_stateless(
                &request_context,
                StandardProtocolKind::Gemini,
                "gemini.count_tokens",
                vec![model.clone()],
                &headers,
                &payload,
            )
            .await
            {
                Ok(Some(response)) => return Json(response).into_response(),
                Ok(None) => {}
                Err(_) => {
                    return gemini_bad_gateway_response(
                        "failed to relay upstream gemini countTokens request",
                    );
                }
            }
            match relay_stateless_json_request(
                &request_context,
                ProviderRequest::ResponsesInputTokens(&request),
            )
            .await
            {
                Ok(Some(response)) => {
                    Json(openai_count_tokens_to_gemini(&response)).into_response()
                }
                Ok(None) => match local_gemini_count_tokens_result(
                    request_context.tenant_id(),
                    request_context.project_id(),
                    &request.model,
                ) {
                    Ok(response) => Json(response).into_response(),
                    Err(response) => response,
                },
                Err(_) => gemini_bad_gateway_response(
                    "failed to relay upstream gemini countTokens request",
                ),
            }
        }
    }
}

pub(crate) async fn gemini_models_compat_with_state_handler(
    request_context: CompatAuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(tail): Path<String>,
    ExtractJson(payload): ExtractJson<Value>,
) -> Response {
    let Some((model, action)) = parse_gemini_compat_tail(&tail) else {
        return gemini_invalid_request_response("unsupported gemini compatibility route");
    };

    match action {
        GeminiCompatAction::GenerateContent => {
            let request = match gemini_request_to_chat_completion(&model, &payload) {
                Ok(request) => request,
                Err(error) => return gemini_invalid_request_response(error.to_string()),
            };
            if request.model.trim().is_empty() {
                return gemini_invalid_request_response("Chat completion model is required.");
            }

            let commercial_admission = match begin_gateway_commercial_admission(
                &state,
                request_context.context(),
                GatewayCommercialAdmissionSpec {
                    quoted_amount: 0.10,
                },
            )
            .await
            {
                Ok(GatewayCommercialAdmissionDecision::Canonical(admission)) => Some(admission),
                Ok(GatewayCommercialAdmissionDecision::LegacyQuota) => {
                    match enforce_project_quota(
                        state.store.as_ref(),
                        request_context.project_id(),
                        100,
                    )
                    .await
                    {
                        Ok(Some(response)) => return response,
                        Ok(None) => {}
                        Err(_) => {
                            return (
                                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                                "failed to evaluate quota",
                            )
                                .into_response();
                        }
                    }
                    None
                }
                Err(response) => return response,
            };

            let headers = HeaderMap::new();
            let path = format!("/v1beta/models/{model}:generateContent");
            let planned_context = match planned_stateful_provider_context(
                &state,
                &request_context,
                "chat_completion",
                &request.model,
            )
            .await
            {
                Ok(planned_context) => planned_context,
                Err(_) => {
                    if let Some(admission) = commercial_admission.as_ref() {
                        if let Err(response) =
                            release_gateway_commercial_admission(&state, admission).await
                        {
                            return response;
                        }
                    }
                    return gemini_bad_gateway_response(
                        "failed to plan upstream gemini generateContent request",
                    );
                }
            };

            match relay_standard_protocol_json_request_from_planned_context(
                &state,
                &request_context,
                "chat_completion",
                &request.model,
                StandardProtocolKind::Gemini,
                &path,
                &headers,
                &payload,
                planned_context.as_ref(),
            )
            .await
            {
                Ok(Some((StandardProtocolJsonRelayOutcome::Json(response), usage_context))) => {
                    if let Some(admission) = commercial_admission.as_ref() {
                        if let Err(response) =
                            capture_gateway_commercial_admission(&state, admission).await
                        {
                            return response;
                        }
                    }
                    if record_gateway_usage_for_project_with_route_key_and_tokens_and_reference_with_context(
                        state.store.as_ref(),
                        request_context.tenant_id(),
                        request_context.project_id(),
                        "chat_completion",
                        &request.model,
                        &request.model,
                        100,
                        0.10,
                        extract_gemini_token_usage_metrics(&response),
                        response.get("id").and_then(Value::as_str),
                        Some(&usage_context),
                    )
                    .await
                    .is_err()
                    {
                        return (
                            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                            "failed to record usage",
                        )
                            .into_response();
                    }

                    return Json(response).into_response();
                }
                Ok(Some((StandardProtocolJsonRelayOutcome::Error(response), _))) => {
                    if let Some(admission) = commercial_admission.as_ref() {
                        if let Err(release_response) =
                            release_gateway_commercial_admission(&state, admission).await
                        {
                            return release_response;
                        }
                    }
                    return response;
                }
                Ok(None) => {}
                Err(_) => {
                    if let Some(admission) = commercial_admission.as_ref() {
                        if let Err(response) =
                            release_gateway_commercial_admission(&state, admission).await
                        {
                            return response;
                        }
                    }
                    return gemini_bad_gateway_response(
                        "failed to relay upstream gemini generateContent request",
                    );
                }
            }

            match relay_raw_protocol_json_request_from_planned_context(
                &state,
                &request_context,
                "chat_completion",
                &request.model,
                StandardProtocolKind::Gemini,
                "gemini.generate_content",
                vec![model.clone()],
                &headers,
                &payload,
                planned_context.as_ref(),
            )
            .await
            {
                Ok(Some((response, usage_context))) => {
                    if let Some(admission) = commercial_admission.as_ref() {
                        if let Err(response) =
                            capture_gateway_commercial_admission(&state, admission).await
                        {
                            return response;
                        }
                    }
                    if record_gateway_usage_for_project_with_route_key_and_tokens_and_reference_with_context(
                        state.store.as_ref(),
                        request_context.tenant_id(),
                        request_context.project_id(),
                        "chat_completion",
                        &request.model,
                        &request.model,
                        100,
                        0.10,
                        extract_gemini_token_usage_metrics(&response),
                        response.get("id").and_then(Value::as_str),
                        Some(&usage_context),
                    )
                    .await
                    .is_err()
                    {
                        return (
                            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                            "failed to record usage",
                        )
                            .into_response();
                    }

                    return Json(response).into_response();
                }
                Ok(None) => {}
                Err(_) => {
                    if let Some(admission) = commercial_admission.as_ref() {
                        if let Err(response) =
                            release_gateway_commercial_admission(&state, admission).await
                        {
                            return response;
                        }
                    }
                    return gemini_bad_gateway_response(
                        "failed to relay upstream gemini generateContent request",
                    );
                }
            }

            if let Some(planned_context) = planned_context.as_ref() {
                match relay_chat_completion_from_planned_execution_context_with_options(
                    state.store.as_ref(),
                    &state.secret_manager,
                    request_context.tenant_id(),
                    request_context.project_id(),
                    &request,
                    &ProviderRequestOptions::default(),
                    planned_context,
                )
                .await
                {
                    Ok(execution) => {
                        let usage_context = execution.usage_context;
                        if let Some(response) = execution.response {
                            if let Some(admission) = commercial_admission.as_ref() {
                                if let Err(response) =
                                    capture_gateway_commercial_admission(&state, admission).await
                                {
                                    return response;
                                }
                            }
                            let token_usage = extract_commercial_token_usage_metrics(&response);
                            if record_gateway_usage_for_project_with_route_key_and_tokens_and_reference_with_context(
                                state.store.as_ref(),
                                request_context.tenant_id(),
                                request_context.project_id(),
                                "chat_completion",
                                &request.model,
                                &request.model,
                                100,
                                0.10,
                                token_usage,
                                response_usage_id_or_single_data_item_id(&response),
                                usage_context.as_ref(),
                            )
                            .await
                            .is_err()
                            {
                                return (
                                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                                    "failed to record usage",
                                )
                                    .into_response();
                            }

                            return Json(openai_chat_response_to_gemini(&response)).into_response();
                        }
                    }
                    Err(_) => {
                        if let Some(admission) = commercial_admission.as_ref() {
                            if let Err(response) =
                                release_gateway_commercial_admission(&state, admission).await
                            {
                                return response;
                            }
                        }
                        return gemini_bad_gateway_response(
                            "failed to relay upstream gemini generateContent request",
                        );
                    }
                }
            } else {
                match relay_chat_completion_from_store_with_execution_context(
                    state.store.as_ref(),
                    &state.secret_manager,
                    request_context.tenant_id(),
                    request_context.project_id(),
                    &request,
                    &ProviderRequestOptions::default(),
                )
                .await
                {
                    Ok(execution) => {
                        let usage_context = execution.usage_context;
                        if let Some(response) = execution.response {
                            if let Some(admission) = commercial_admission.as_ref() {
                                if let Err(response) =
                                    capture_gateway_commercial_admission(&state, admission).await
                                {
                                    return response;
                                }
                            }
                            let token_usage = extract_commercial_token_usage_metrics(&response);
                            if record_gateway_usage_for_project_with_route_key_and_tokens_and_reference_with_context(
                                state.store.as_ref(),
                                request_context.tenant_id(),
                                request_context.project_id(),
                                "chat_completion",
                                &request.model,
                                &request.model,
                                100,
                                0.10,
                                token_usage,
                                response_usage_id_or_single_data_item_id(&response),
                                usage_context.as_ref(),
                            )
                            .await
                            .is_err()
                            {
                                return (
                                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                                    "failed to record usage",
                                )
                                    .into_response();
                            }

                            return Json(openai_chat_response_to_gemini(&response)).into_response();
                        }
                    }
                    Err(_) => {
                        if let Some(admission) = commercial_admission.as_ref() {
                            if let Err(response) =
                                release_gateway_commercial_admission(&state, admission).await
                            {
                                return response;
                            }
                        }
                        return gemini_bad_gateway_response(
                            "failed to relay upstream gemini generateContent request",
                        );
                    }
                }
            }

            let local_response = match local_gemini_chat_completion_result(
                request_context.tenant_id(),
                request_context.project_id(),
                &request.model,
            ) {
                Ok(response) => response,
                Err(response) => {
                    if let Some(admission) = commercial_admission.as_ref() {
                        if let Err(release_response) =
                            release_gateway_commercial_admission(&state, admission).await
                        {
                            return release_response;
                        }
                    }
                    return response;
                }
            };

            if let Some(admission) = commercial_admission.as_ref() {
                if let Err(response) = capture_gateway_commercial_admission(&state, admission).await
                {
                    return response;
                }
            }

            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "chat_completion",
                &request.model,
                100,
                0.10,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            Json(local_response).into_response()
        }
        GeminiCompatAction::StreamGenerateContent => {
            let mut request = match gemini_request_to_chat_completion(&model, &payload) {
                Ok(request) => request,
                Err(error) => return gemini_invalid_request_response(error.to_string()),
            };
            if request.model.trim().is_empty() {
                return gemini_invalid_request_response("Chat completion model is required.");
            }
            request.stream = Some(true);

            let commercial_admission = match begin_gateway_commercial_admission(
                &state,
                request_context.context(),
                GatewayCommercialAdmissionSpec {
                    quoted_amount: 0.10,
                },
            )
            .await
            {
                Ok(GatewayCommercialAdmissionDecision::Canonical(admission)) => Some(admission),
                Ok(GatewayCommercialAdmissionDecision::LegacyQuota) => {
                    match enforce_project_quota(
                        state.store.as_ref(),
                        request_context.project_id(),
                        100,
                    )
                    .await
                    {
                        Ok(Some(response)) => return response,
                        Ok(None) => {}
                        Err(_) => {
                            return (
                                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                                "failed to evaluate quota",
                            )
                                .into_response();
                        }
                    }
                    None
                }
                Err(response) => return response,
            };

            let headers = HeaderMap::new();
            let path = format!("/v1beta/models/{model}:streamGenerateContent?alt=sse");
            let planned_context = match planned_stateful_provider_context(
                &state,
                &request_context,
                "chat_completion",
                &request.model,
            )
            .await
            {
                Ok(planned_context) => planned_context,
                Err(_) => {
                    if let Some(admission) = commercial_admission.as_ref() {
                        if let Err(response) =
                            release_gateway_commercial_admission(&state, admission).await
                        {
                            return response;
                        }
                    }
                    return gemini_bad_gateway_response(
                        "failed to plan upstream gemini streamGenerateContent request",
                    );
                }
            };

            match relay_standard_protocol_stream_request_from_planned_context(
                &state,
                &request_context,
                "chat_completion",
                &request.model,
                StandardProtocolKind::Gemini,
                &path,
                &headers,
                &payload,
                planned_context.as_ref(),
            )
            .await
            {
                Ok(Some((StandardProtocolStreamRelayOutcome::Stream(response), usage_context))) => {
                    if let Some(admission) = commercial_admission.as_ref() {
                        if let Err(response) =
                            capture_gateway_commercial_admission(&state, admission).await
                        {
                            return response;
                        }
                    }
                    if record_gateway_usage_for_project_with_context(
                        state.store.as_ref(),
                        request_context.tenant_id(),
                        request_context.project_id(),
                        "chat_completion",
                        &request.model,
                        100,
                        0.10,
                        Some(&usage_context),
                    )
                    .await
                    .is_err()
                    {
                        return (
                            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                            "failed to record usage",
                        )
                            .into_response();
                    }

                    return upstream_passthrough_response(response);
                }
                Ok(Some((StandardProtocolStreamRelayOutcome::Error(response), _))) => {
                    if let Some(admission) = commercial_admission.as_ref() {
                        if let Err(release_response) =
                            release_gateway_commercial_admission(&state, admission).await
                        {
                            return release_response;
                        }
                    }
                    return response;
                }
                Ok(None) => {}
                Err(_) => {
                    if let Some(admission) = commercial_admission.as_ref() {
                        if let Err(response) =
                            release_gateway_commercial_admission(&state, admission).await
                        {
                            return response;
                        }
                    }
                    return gemini_bad_gateway_response(
                        "failed to relay upstream gemini streamGenerateContent request",
                    );
                }
            }

            match relay_raw_protocol_stream_request_from_planned_context(
                &state,
                &request_context,
                "chat_completion",
                &request.model,
                StandardProtocolKind::Gemini,
                "gemini.stream_generate_content",
                vec![model.clone()],
                &headers,
                &payload,
                planned_context.as_ref(),
            )
            .await
            {
                Ok(Some((response, usage_context))) => {
                    if let Some(admission) = commercial_admission.as_ref() {
                        if let Err(response) =
                            capture_gateway_commercial_admission(&state, admission).await
                        {
                            return response;
                        }
                    }
                    if record_gateway_usage_for_project_with_context(
                        state.store.as_ref(),
                        request_context.tenant_id(),
                        request_context.project_id(),
                        "chat_completion",
                        &request.model,
                        100,
                        0.10,
                        Some(&usage_context),
                    )
                    .await
                    .is_err()
                    {
                        return (
                            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                            "failed to record usage",
                        )
                            .into_response();
                    }

                    return upstream_passthrough_response(response);
                }
                Ok(None) => {}
                Err(_) => {
                    if let Some(admission) = commercial_admission.as_ref() {
                        if let Err(response) =
                            release_gateway_commercial_admission(&state, admission).await
                        {
                            return response;
                        }
                    }
                    return gemini_bad_gateway_response(
                        "failed to relay upstream gemini streamGenerateContent request",
                    );
                }
            }

            if let Some(planned_context) = planned_context.as_ref() {
                match relay_chat_completion_stream_from_planned_execution_context_with_options(
                    state.store.as_ref(),
                    &state.secret_manager,
                    request_context.tenant_id(),
                    request_context.project_id(),
                    &request,
                    &ProviderRequestOptions::default(),
                    planned_context,
                )
                .await
                {
                    Ok(execution) => {
                        let usage_context = execution.usage_context;
                        if let Some(response) = execution.response {
                            if let Some(admission) = commercial_admission.as_ref() {
                                if let Err(response) =
                                    capture_gateway_commercial_admission(&state, admission).await
                                {
                                    return response;
                                }
                            }
                            if record_gateway_usage_for_project_with_context(
                                state.store.as_ref(),
                                request_context.tenant_id(),
                                request_context.project_id(),
                                "chat_completion",
                                &request.model,
                                100,
                                0.10,
                                usage_context.as_ref(),
                            )
                            .await
                            .is_err()
                            {
                                return (
                                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                                    "failed to record usage",
                                )
                                    .into_response();
                            }

                            return upstream_passthrough_response(gemini_stream_from_openai(
                                response,
                            ));
                        }
                    }
                    Err(_) => {
                        if let Some(admission) = commercial_admission.as_ref() {
                            if let Err(response) =
                                release_gateway_commercial_admission(&state, admission).await
                            {
                                return response;
                            }
                        }
                        return gemini_bad_gateway_response(
                            "failed to relay upstream gemini streamGenerateContent request",
                        );
                    }
                }
            } else {
                match relay_chat_completion_stream_from_store_with_execution_context(
                    state.store.as_ref(),
                    &state.secret_manager,
                    request_context.tenant_id(),
                    request_context.project_id(),
                    &request,
                    &ProviderRequestOptions::default(),
                )
                .await
                {
                    Ok(execution) => {
                        let usage_context = execution.usage_context;
                        if let Some(response) = execution.response {
                            if let Some(admission) = commercial_admission.as_ref() {
                                if let Err(response) =
                                    capture_gateway_commercial_admission(&state, admission).await
                                {
                                    return response;
                                }
                            }
                            if record_gateway_usage_for_project_with_context(
                                state.store.as_ref(),
                                request_context.tenant_id(),
                                request_context.project_id(),
                                "chat_completion",
                                &request.model,
                                100,
                                0.10,
                                usage_context.as_ref(),
                            )
                            .await
                            .is_err()
                            {
                                return (
                                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                                    "failed to record usage",
                                )
                                    .into_response();
                            }

                            return upstream_passthrough_response(gemini_stream_from_openai(
                                response,
                            ));
                        }
                    }
                    Err(_) => {
                        if let Some(admission) = commercial_admission.as_ref() {
                            if let Err(response) =
                                release_gateway_commercial_admission(&state, admission).await
                            {
                                return response;
                            }
                        }
                        return gemini_bad_gateway_response(
                            "failed to relay upstream gemini streamGenerateContent request",
                        );
                    }
                }
            }

            let local_response = match local_gemini_stream_result(
                request_context.tenant_id(),
                request_context.project_id(),
                &request.model,
            ) {
                Ok(response) => response,
                Err(response) => {
                    if let Some(admission) = commercial_admission.as_ref() {
                        if let Err(release_response) =
                            release_gateway_commercial_admission(&state, admission).await
                        {
                            return release_response;
                        }
                    }
                    return response;
                }
            };

            if let Some(admission) = commercial_admission.as_ref() {
                if let Err(release_response) =
                    release_gateway_commercial_admission(&state, admission).await
                {
                    return release_response;
                }
            }

            local_response
        }
        GeminiCompatAction::CountTokens => {
            let request = gemini_count_tokens_request(&model, &payload);
            if request.model.trim().is_empty() {
                return gemini_invalid_request_response("Response model is required.");
            }

            let headers = HeaderMap::new();
            let path = format!("/v1beta/models/{model}:countTokens");
            let planned_context = match planned_stateful_provider_context(
                &state,
                &request_context,
                "responses",
                &request.model,
            )
            .await
            {
                Ok(planned_context) => planned_context,
                Err(_) => {
                    return gemini_bad_gateway_response(
                        "failed to plan upstream gemini countTokens request",
                    );
                }
            };

            match relay_standard_protocol_json_request_from_planned_context(
                &state,
                &request_context,
                "responses",
                &request.model,
                StandardProtocolKind::Gemini,
                &path,
                &headers,
                &payload,
                planned_context.as_ref(),
            )
            .await
            {
                Ok(Some((StandardProtocolJsonRelayOutcome::Json(response), _))) => {
                    return Json(response).into_response();
                }
                Ok(Some((StandardProtocolJsonRelayOutcome::Error(response), _))) => {
                    return response;
                }
                Ok(None) => {}
                Err(_) => {
                    return gemini_bad_gateway_response(
                        "failed to relay upstream gemini countTokens request",
                    );
                }
            }

            match relay_raw_protocol_json_request_from_planned_context(
                &state,
                &request_context,
                "responses",
                &request.model,
                StandardProtocolKind::Gemini,
                "gemini.count_tokens",
                vec![model.clone()],
                &headers,
                &payload,
                planned_context.as_ref(),
            )
            .await
            {
                Ok(Some((response, _))) => return Json(response).into_response(),
                Ok(None) => {}
                Err(_) => {
                    return gemini_bad_gateway_response(
                        "failed to relay upstream gemini countTokens request",
                    );
                }
            }

            if let Some(planned_context) = planned_context.as_ref() {
                match relay_count_response_input_tokens_from_planned_execution_context(
                    state.store.as_ref(),
                    request_context.tenant_id(),
                    request_context.project_id(),
                    &request,
                    planned_context,
                )
                .await
                {
                    Ok(Some(response)) => {
                        Json(openai_count_tokens_to_gemini(&response)).into_response()
                    }
                    Ok(None) => match local_gemini_count_tokens_result(
                        request_context.tenant_id(),
                        request_context.project_id(),
                        &request.model,
                    ) {
                        Ok(response) => Json(response).into_response(),
                        Err(response) => response,
                    },
                    Err(_) => gemini_bad_gateway_response(
                        "failed to relay upstream gemini countTokens request",
                    ),
                }
            } else {
                match relay_count_response_input_tokens_from_store(
                    state.store.as_ref(),
                    &state.secret_manager,
                    request_context.tenant_id(),
                    request_context.project_id(),
                    &request,
                )
                .await
                {
                    Ok(Some(response)) => {
                        Json(openai_count_tokens_to_gemini(&response)).into_response()
                    }
                    Ok(None) => match local_gemini_count_tokens_result(
                        request_context.tenant_id(),
                        request_context.project_id(),
                        &request.model,
                    ) {
                        Ok(response) => Json(response).into_response(),
                        Err(response) => response,
                    },
                    Err(_) => gemini_bad_gateway_response(
                        "failed to relay upstream gemini countTokens request",
                    ),
                }
            }
        }
    }
}
