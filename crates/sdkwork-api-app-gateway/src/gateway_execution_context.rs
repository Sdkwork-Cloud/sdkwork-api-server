use super::*;

const GATEWAY_UPSTREAM_RETRY_MAX_ATTEMPTS_ENV: &str = "SDKWORK_GATEWAY_UPSTREAM_RETRY_MAX_ATTEMPTS";
const GATEWAY_UPSTREAM_RETRY_BASE_DELAY_MS_ENV: &str =
    "SDKWORK_GATEWAY_UPSTREAM_RETRY_BASE_DELAY_MS";
const GATEWAY_UPSTREAM_RETRY_MAX_DELAY_MS_ENV: &str = "SDKWORK_GATEWAY_UPSTREAM_RETRY_MAX_DELAY_MS";
const DEFAULT_GATEWAY_UPSTREAM_RETRY_MAX_ATTEMPTS: usize = 2;
const DEFAULT_GATEWAY_UPSTREAM_RETRY_BASE_DELAY_MS: u64 = 25;
const DEFAULT_GATEWAY_UPSTREAM_RETRY_MAX_DELAY_MS: u64 = 5_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct GatewayUpstreamRetryPolicy {
    pub(crate) max_attempts: usize,
    pub(crate) base_delay_ms: u64,
    pub(crate) max_delay_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct GatewayExecutionPolicy {
    pub(crate) failover_enabled: bool,
    pub(crate) retry_policy: GatewayUpstreamRetryPolicy,
}

impl GatewayUpstreamRetryPolicy {
    fn disabled() -> Self {
        Self {
            max_attempts: 1,
            base_delay_ms: 0,
            max_delay_ms: 0,
        }
    }

    pub(crate) fn enabled(&self) -> bool {
        self.max_attempts > 1
    }

    fn delay_before_next_attempt(&self, failed_attempt: usize) -> Duration {
        if failed_attempt == 0 || self.base_delay_ms == 0 {
            return Duration::from_millis(0);
        }

        let exponent = failed_attempt.saturating_sub(1).min(20) as u32;
        let multiplier = 1u64.checked_shl(exponent).unwrap_or(u64::MAX);
        let max_delay_ms = self.max_delay_ms.max(self.base_delay_ms);
        let delay_ms = self
            .base_delay_ms
            .saturating_mul(multiplier)
            .min(max_delay_ms);
        Duration::from_millis(delay_ms)
    }
}

async fn gateway_routing_policy_for_decision(
    store: &dyn AdminStore,
    decision: &RoutingDecision,
) -> Result<Option<RoutingPolicy>> {
    let Some(policy_id) = decision.matched_policy_id.as_deref() else {
        return Ok(None);
    };
    Ok(store
        .list_routing_policies()
        .await?
        .into_iter()
        .find(|policy| policy.policy_id == policy_id))
}

pub(crate) async fn gateway_execution_policy_for_decision(
    store: &dyn AdminStore,
    decision: &RoutingDecision,
    request: &ProviderRequest<'_>,
) -> Result<GatewayExecutionPolicy> {
    let routing_policy = gateway_routing_policy_for_decision(store, decision).await?;
    Ok(GatewayExecutionPolicy {
        failover_enabled: routing_policy
            .as_ref()
            .map(|policy| policy.execution_failover_enabled)
            .unwrap_or(true),
        retry_policy: gateway_upstream_retry_policy(request, routing_policy.as_ref()),
    })
}

pub(crate) fn gateway_upstream_retry_policy(
    request: &ProviderRequest<'_>,
    routing_policy: Option<&RoutingPolicy>,
) -> GatewayUpstreamRetryPolicy {
    if !gateway_request_supports_retry(request) {
        return GatewayUpstreamRetryPolicy::disabled();
    }

    let base_delay_ms = routing_policy
        .and_then(|policy| policy.upstream_retry_base_delay_ms)
        .unwrap_or_else(|| {
            std::env::var(GATEWAY_UPSTREAM_RETRY_BASE_DELAY_MS_ENV)
                .ok()
                .and_then(|value| value.parse::<u64>().ok())
                .unwrap_or(DEFAULT_GATEWAY_UPSTREAM_RETRY_BASE_DELAY_MS)
        });
    let max_delay_ms = routing_policy
        .and_then(|policy| policy.upstream_retry_max_delay_ms)
        .unwrap_or_else(|| {
            std::env::var(GATEWAY_UPSTREAM_RETRY_MAX_DELAY_MS_ENV)
                .ok()
                .and_then(|value| value.parse::<u64>().ok())
                .unwrap_or(DEFAULT_GATEWAY_UPSTREAM_RETRY_MAX_DELAY_MS)
        })
        .max(base_delay_ms);

    GatewayUpstreamRetryPolicy {
        max_attempts: routing_policy
            .and_then(|policy| policy.upstream_retry_max_attempts)
            .map(|value| value.clamp(1, 4) as usize)
            .or_else(|| {
                std::env::var(GATEWAY_UPSTREAM_RETRY_MAX_ATTEMPTS_ENV)
                    .ok()
                    .and_then(|value| value.parse::<usize>().ok())
                    .map(|value| value.clamp(1, 4))
            })
            .unwrap_or(DEFAULT_GATEWAY_UPSTREAM_RETRY_MAX_ATTEMPTS),
        base_delay_ms,
        max_delay_ms,
    }
}

fn gateway_request_supports_retry(request: &ProviderRequest<'_>) -> bool {
    matches!(
        request,
        ProviderRequest::ChatCompletions(_)
            | ProviderRequest::ChatCompletionsStream(_)
            | ProviderRequest::Responses(_)
            | ProviderRequest::ResponsesStream(_)
    )
}

pub(crate) fn gateway_upstream_error_is_retryable(error: &anyhow::Error) -> bool {
    if let Some(error) = gateway_execution_context_error(error) {
        return matches!(
            error.kind(),
            GatewayExecutionContextErrorKind::RequestTimeout
        );
    }

    if let Some(error) = gateway_provider_http_error(error) {
        return matches!(
            error.status(),
            Some(
                reqwest::StatusCode::REQUEST_TIMEOUT
                    | reqwest::StatusCode::TOO_MANY_REQUESTS
                    | reqwest::StatusCode::INTERNAL_SERVER_ERROR
                    | reqwest::StatusCode::BAD_GATEWAY
                    | reqwest::StatusCode::SERVICE_UNAVAILABLE
                    | reqwest::StatusCode::GATEWAY_TIMEOUT
            )
        );
    }

    let Some(error) = gateway_reqwest_error(error) else {
        return false;
    };

    if error.is_timeout() || error.is_connect() {
        return true;
    }

    matches!(
        error.status(),
        Some(
            reqwest::StatusCode::REQUEST_TIMEOUT
                | reqwest::StatusCode::TOO_MANY_REQUESTS
                | reqwest::StatusCode::INTERNAL_SERVER_ERROR
                | reqwest::StatusCode::BAD_GATEWAY
                | reqwest::StatusCode::SERVICE_UNAVAILABLE
                | reqwest::StatusCode::GATEWAY_TIMEOUT
        )
    )
}

pub(crate) fn gateway_retry_reason_for_status(status: reqwest::StatusCode) -> &'static str {
    match status {
        reqwest::StatusCode::REQUEST_TIMEOUT => "status_408",
        reqwest::StatusCode::TOO_MANY_REQUESTS => "status_429",
        reqwest::StatusCode::INTERNAL_SERVER_ERROR => "status_500",
        reqwest::StatusCode::BAD_GATEWAY => "status_502",
        reqwest::StatusCode::SERVICE_UNAVAILABLE => "status_503",
        reqwest::StatusCode::GATEWAY_TIMEOUT => "status_504",
        _ => "status_other",
    }
}

pub(crate) fn gateway_retry_reason_for_error(error: &anyhow::Error) -> &'static str {
    if let Some(error) = gateway_execution_context_error(error) {
        return match error.kind() {
            GatewayExecutionContextErrorKind::RequestTimeout => "execution_timeout",
            GatewayExecutionContextErrorKind::DeadlineExceeded => "deadline_exceeded",
            GatewayExecutionContextErrorKind::ProviderOverloaded => "provider_overloaded",
        };
    }

    if let Some(error) = gateway_provider_http_error(error) {
        if let Some(status) = error.status() {
            return gateway_retry_reason_for_status(status);
        }
    }

    if let Some(error) = gateway_reqwest_error(error) {
        if error.is_timeout() {
            return "reqwest_timeout";
        }
        if error.is_connect() {
            return "reqwest_connect";
        }
        if let Some(status) = error.status() {
            return gateway_retry_reason_for_status(status);
        }
    }

    "unknown"
}

pub(crate) fn gateway_execution_context_metric_reason(
    error: &GatewayExecutionContextError,
) -> &'static str {
    match error.kind() {
        GatewayExecutionContextErrorKind::RequestTimeout => "request_timeout",
        GatewayExecutionContextErrorKind::DeadlineExceeded => "deadline_exceeded",
        GatewayExecutionContextErrorKind::ProviderOverloaded => "provider_overloaded",
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GatewayExecutionContextErrorKind {
    RequestTimeout,
    DeadlineExceeded,
    ProviderOverloaded,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GatewayExecutionContextError {
    kind: GatewayExecutionContextErrorKind,
    timeout_ms: Option<u64>,
    deadline_at_ms: Option<u64>,
    provider_id: Option<String>,
    max_in_flight: Option<usize>,
}

impl GatewayExecutionContextError {
    fn request_timeout(timeout_ms: u64, deadline_at_ms: Option<u64>) -> Self {
        Self {
            kind: GatewayExecutionContextErrorKind::RequestTimeout,
            timeout_ms: Some(timeout_ms),
            deadline_at_ms,
            provider_id: None,
            max_in_flight: None,
        }
    }

    fn deadline_exceeded(deadline_at_ms: Option<u64>) -> Self {
        Self {
            kind: GatewayExecutionContextErrorKind::DeadlineExceeded,
            timeout_ms: None,
            deadline_at_ms,
            provider_id: None,
            max_in_flight: None,
        }
    }

    fn provider_overloaded(provider_id: impl Into<String>, max_in_flight: usize) -> Self {
        Self {
            kind: GatewayExecutionContextErrorKind::ProviderOverloaded,
            timeout_ms: None,
            deadline_at_ms: None,
            provider_id: Some(provider_id.into()),
            max_in_flight: Some(max_in_flight),
        }
    }

    pub(crate) fn kind(&self) -> GatewayExecutionContextErrorKind {
        self.kind
    }
}

impl fmt::Display for GatewayExecutionContextError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self.kind, self.timeout_ms, self.deadline_at_ms) {
            (
                GatewayExecutionContextErrorKind::RequestTimeout,
                Some(timeout_ms),
                Some(deadline),
            ) => {
                write!(
                    f,
                    "gateway upstream request timed out after {timeout_ms}ms before deadline {deadline}"
                )
            }
            (GatewayExecutionContextErrorKind::RequestTimeout, Some(timeout_ms), None) => {
                write!(f, "gateway upstream request timed out after {timeout_ms}ms")
            }
            (GatewayExecutionContextErrorKind::DeadlineExceeded, _, Some(deadline)) => {
                write!(
                    f,
                    "gateway upstream deadline {deadline} has already expired"
                )
            }
            (GatewayExecutionContextErrorKind::DeadlineExceeded, _, None) => {
                write!(f, "gateway upstream deadline has already expired")
            }
            (GatewayExecutionContextErrorKind::RequestTimeout, None, _) => {
                write!(f, "gateway upstream request timed out")
            }
            (GatewayExecutionContextErrorKind::ProviderOverloaded, _, _) => {
                match (self.provider_id.as_deref(), self.max_in_flight) {
                    (Some(provider_id), Some(max_in_flight)) => write!(
                        f,
                        "gateway provider {provider_id} is locally overloaded because in-flight requests reached {max_in_flight}"
                    ),
                    (Some(provider_id), None) => {
                        write!(f, "gateway provider {provider_id} is locally overloaded")
                    }
                    (None, Some(max_in_flight)) => write!(
                        f,
                        "gateway provider is locally overloaded because in-flight requests reached {max_in_flight}"
                    ),
                    (None, None) => write!(f, "gateway provider is locally overloaded"),
                }
            }
        }
    }
}

impl std::error::Error for GatewayExecutionContextError {}

struct GatewayProviderInFlightPermit {
    counter: Arc<AtomicUsize>,
}

impl Drop for GatewayProviderInFlightPermit {
    fn drop(&mut self) {
        self.counter.fetch_sub(1, Ordering::AcqRel);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct GatewayRetryDelayDecision {
    pub(crate) delay: Duration,
    pub(crate) source: &'static str,
}

pub(crate) fn gateway_retry_delay_for_error(
    retry_policy: GatewayUpstreamRetryPolicy,
    failed_attempt: usize,
    error: &anyhow::Error,
) -> GatewayRetryDelayDecision {
    let base_delay = retry_policy.delay_before_next_attempt(failed_attempt);
    let retry_after = gateway_provider_http_error(error).and_then(|error| {
        Some((
            Duration::from_secs(error.retry_after_secs()?),
            error.retry_after_source(),
        ))
    });
    let retry_after_delay = retry_after
        .map(|(delay, _)| delay)
        .unwrap_or(Duration::from_millis(0));
    let capped_retry_after = if retry_after_delay.is_zero() {
        retry_after_delay
    } else {
        retry_after_delay.min(Duration::from_millis(retry_policy.max_delay_ms))
    };
    if !capped_retry_after.is_zero() && capped_retry_after > base_delay {
        let source = match retry_after.and_then(|(_, source)| source) {
            Some(ProviderRetryAfterSource::Seconds) => "retry_after_seconds",
            Some(ProviderRetryAfterSource::HttpDate) => "retry_after_http_date",
            None => "retry_after",
        };
        return GatewayRetryDelayDecision {
            delay: capped_retry_after,
            source,
        };
    }

    GatewayRetryDelayDecision {
        delay: base_delay.max(capped_retry_after),
        source: "backoff",
    }
}

pub(crate) fn gateway_provider_http_error(error: &anyhow::Error) -> Option<&ProviderHttpError> {
    error
        .chain()
        .find_map(|cause| cause.downcast_ref::<ProviderHttpError>())
}

pub(crate) fn gateway_execution_context_error(
    error: &anyhow::Error,
) -> Option<&GatewayExecutionContextError> {
    error
        .chain()
        .find_map(|cause| cause.downcast_ref::<GatewayExecutionContextError>())
}

pub(crate) fn gateway_execution_context_error_impacts_provider_health(
    error: &GatewayExecutionContextError,
) -> bool {
    matches!(
        error.kind(),
        GatewayExecutionContextErrorKind::RequestTimeout
    )
}

pub(crate) fn gateway_error_impacts_provider_health(error: &anyhow::Error) -> bool {
    if let Some(error) = gateway_execution_context_error(error) {
        return gateway_execution_context_error_impacts_provider_health(error);
    }
    true
}

pub(crate) fn gateway_reqwest_error(error: &anyhow::Error) -> Option<&reqwest::Error> {
    error
        .chain()
        .find_map(|cause| cause.downcast_ref::<reqwest::Error>())
}

fn try_acquire_gateway_provider_in_flight_permit(
    provider_id: Option<&str>,
) -> Result<Option<GatewayProviderInFlightPermit>> {
    let (Some(provider_id), Some(max_in_flight)) =
        (provider_id, gateway_provider_max_in_flight_limit())
    else {
        return Ok(None);
    };

    let counter = gateway_provider_in_flight_counter(provider_id);
    loop {
        let current = counter.load(Ordering::Acquire);
        if current >= max_in_flight {
            return Err(anyhow::Error::new(
                GatewayExecutionContextError::provider_overloaded(provider_id, max_in_flight),
            ));
        }
        if counter
            .compare_exchange(current, current + 1, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
        {
            return Ok(Some(GatewayProviderInFlightPermit { counter }));
        }
    }
}

pub(crate) async fn execute_provider_request_with_execution_context(
    adapter: &dyn ProviderExecutionAdapter,
    provider_id: Option<&str>,
    api_key: &str,
    request: ProviderRequest<'_>,
    options: &ProviderRequestOptions,
) -> Result<ProviderOutput> {
    let now_ms = gateway_execution_observed_at_ms();
    if options.deadline_expired(now_ms) {
        return Err(anyhow::Error::new(
            GatewayExecutionContextError::deadline_exceeded(options.deadline_at_ms()),
        ));
    }

    let _in_flight_permit = try_acquire_gateway_provider_in_flight_permit(provider_id)?;

    if let Some(timeout_ms) = options.effective_timeout_ms(now_ms) {
        return match tokio::time::timeout(
            Duration::from_millis(timeout_ms),
            adapter.execute_with_options(api_key, request, options),
        )
        .await
        {
            Ok(result) => result,
            Err(_) => Err(anyhow::Error::new(
                GatewayExecutionContextError::request_timeout(timeout_ms, options.deadline_at_ms()),
            )),
        };
    }

    adapter
        .execute_with_options(api_key, request, options)
        .await
}
