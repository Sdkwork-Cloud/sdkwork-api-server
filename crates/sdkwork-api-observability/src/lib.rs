use std::collections::{BTreeMap, BTreeSet};
use std::future::Future;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::extract::{MatchedPath, State};
use axum::http::header::{HeaderName, HeaderValue};
use axum::http::Request;
use axum::middleware::Next;
use axum::response::Response;
use tracing::Instrument;

pub const REQUEST_ID_HEADER: &str = "x-request-id";

const LATENCY_BUCKETS_MS: [u64; 11] = [5, 10, 25, 50, 100, 250, 500, 1000, 2500, 5000, 10000];

static REQUEST_ID_COUNTER: AtomicU64 = AtomicU64::new(1);
static TRACING_INIT: OnceLock<()> = OnceLock::new();

tokio::task_local! {
    static CURRENT_HTTP_METRICS_REGISTRY: Arc<HttpMetricsRegistry>;
}

tokio::task_local! {
    static CURRENT_HTTP_METRIC_DIMENSIONS: Arc<Mutex<HttpMetricDimensions>>;
}

pub fn service_name(name: &str) -> &str {
    name
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestId(String);

impl RequestId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TelemetryCardinalityLimits {
    route_limit: usize,
    tenant_limit: usize,
    model_limit: usize,
    provider_limit: usize,
    billing_mode_limit: usize,
    retry_outcome_limit: usize,
    failover_outcome_limit: usize,
    payment_outcome_limit: usize,
    event_kind_limit: usize,
    result_limit: usize,
}

impl Default for TelemetryCardinalityLimits {
    fn default() -> Self {
        Self {
            route_limit: 128,
            tenant_limit: 128,
            model_limit: 256,
            provider_limit: 128,
            billing_mode_limit: 32,
            retry_outcome_limit: 16,
            failover_outcome_limit: 16,
            payment_outcome_limit: 32,
            event_kind_limit: 32,
            result_limit: 32,
        }
    }
}

impl TelemetryCardinalityLimits {
    pub fn with_route_limit(mut self, limit: usize) -> Self {
        self.route_limit = limit.max(1);
        self
    }

    pub fn with_tenant_limit(mut self, limit: usize) -> Self {
        self.tenant_limit = limit.max(1);
        self
    }

    pub fn with_model_limit(mut self, limit: usize) -> Self {
        self.model_limit = limit.max(1);
        self
    }

    pub fn with_provider_limit(mut self, limit: usize) -> Self {
        self.provider_limit = limit.max(1);
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum DimensionKind {
    Route,
    Tenant,
    Model,
    Provider,
    BillingMode,
    RetryOutcome,
    FailoverOutcome,
    PaymentOutcome,
    EventKind,
    Result,
}

#[derive(Debug, Clone)]
struct MetricCardinalityLimiter {
    limits: TelemetryCardinalityLimits,
    seen: BTreeMap<DimensionKind, BTreeSet<String>>,
}

impl MetricCardinalityLimiter {
    fn new(limits: TelemetryCardinalityLimits) -> Self {
        Self {
            limits,
            seen: BTreeMap::new(),
        }
    }

    fn normalize(
        &mut self,
        kind: DimensionKind,
        value: Option<&str>,
        missing_fallback: &'static str,
    ) -> String {
        let Some(value) = value.and_then(sanitize_label_value) else {
            return missing_fallback.to_owned();
        };

        let limit = match kind {
            DimensionKind::Route => self.limits.route_limit,
            DimensionKind::Tenant => self.limits.tenant_limit,
            DimensionKind::Model => self.limits.model_limit,
            DimensionKind::Provider => self.limits.provider_limit,
            DimensionKind::BillingMode => self.limits.billing_mode_limit,
            DimensionKind::RetryOutcome => self.limits.retry_outcome_limit,
            DimensionKind::FailoverOutcome => self.limits.failover_outcome_limit,
            DimensionKind::PaymentOutcome => self.limits.payment_outcome_limit,
            DimensionKind::EventKind => self.limits.event_kind_limit,
            DimensionKind::Result => self.limits.result_limit,
        };

        let seen = self.seen.entry(kind).or_default();
        if seen.contains(&value) || seen.len() < limit {
            seen.insert(value.clone());
            value
        } else {
            "other".to_owned()
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HttpMetricDimensions {
    pub route: Option<String>,
    pub tenant: Option<String>,
    pub model: Option<String>,
    pub provider: Option<String>,
    pub billing_mode: Option<String>,
    pub retry_outcome: Option<String>,
    pub failover_outcome: Option<String>,
    pub payment_outcome: Option<String>,
}

impl HttpMetricDimensions {
    pub fn with_route(mut self, value: impl Into<String>) -> Self {
        self.route = Some(value.into());
        self
    }

    pub fn with_route_option(mut self, value: Option<String>) -> Self {
        self.route = value;
        self
    }

    pub fn with_tenant(mut self, value: impl Into<String>) -> Self {
        self.tenant = Some(value.into());
        self
    }

    pub fn with_tenant_option(mut self, value: Option<String>) -> Self {
        self.tenant = value;
        self
    }

    pub fn with_model(mut self, value: impl Into<String>) -> Self {
        self.model = Some(value.into());
        self
    }

    pub fn with_model_option(mut self, value: Option<String>) -> Self {
        self.model = value;
        self
    }

    pub fn with_provider(mut self, value: impl Into<String>) -> Self {
        self.provider = Some(value.into());
        self
    }

    pub fn with_provider_option(mut self, value: Option<String>) -> Self {
        self.provider = value;
        self
    }

    pub fn with_billing_mode(mut self, value: impl Into<String>) -> Self {
        self.billing_mode = Some(value.into());
        self
    }

    pub fn with_billing_mode_option(mut self, value: Option<String>) -> Self {
        self.billing_mode = value;
        self
    }

    pub fn with_retry_outcome(mut self, value: impl Into<String>) -> Self {
        self.retry_outcome = Some(value.into());
        self
    }

    pub fn with_retry_outcome_option(mut self, value: Option<String>) -> Self {
        self.retry_outcome = value;
        self
    }

    pub fn with_failover_outcome(mut self, value: impl Into<String>) -> Self {
        self.failover_outcome = Some(value.into());
        self
    }

    pub fn with_failover_outcome_option(mut self, value: Option<String>) -> Self {
        self.failover_outcome = value;
        self
    }

    pub fn with_payment_outcome(mut self, value: impl Into<String>) -> Self {
        self.payment_outcome = Some(value.into());
        self
    }

    pub fn with_payment_outcome_option(mut self, value: Option<String>) -> Self {
        self.payment_outcome = value;
        self
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProviderExecutionMetricDimensions {
    pub route: Option<String>,
    pub tenant: Option<String>,
    pub model: Option<String>,
    pub provider: Option<String>,
    pub billing_mode: Option<String>,
    pub retry_outcome: Option<String>,
    pub failover_outcome: Option<String>,
    pub result: Option<String>,
}

impl ProviderExecutionMetricDimensions {
    pub fn with_route(mut self, value: impl Into<String>) -> Self {
        self.route = Some(value.into());
        self
    }

    pub fn with_tenant(mut self, value: impl Into<String>) -> Self {
        self.tenant = Some(value.into());
        self
    }

    pub fn with_model(mut self, value: impl Into<String>) -> Self {
        self.model = Some(value.into());
        self
    }

    pub fn with_provider(mut self, value: impl Into<String>) -> Self {
        self.provider = Some(value.into());
        self
    }

    pub fn with_billing_mode(mut self, value: impl Into<String>) -> Self {
        self.billing_mode = Some(value.into());
        self
    }

    pub fn with_retry_outcome(mut self, value: impl Into<String>) -> Self {
        self.retry_outcome = Some(value.into());
        self
    }

    pub fn with_failover_outcome(mut self, value: impl Into<String>) -> Self {
        self.failover_outcome = Some(value.into());
        self
    }

    pub fn with_result(mut self, value: impl Into<String>) -> Self {
        self.result = Some(value.into());
        self
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PaymentMetricDimensions {
    pub provider: Option<String>,
    pub tenant: Option<String>,
    pub payment_outcome: Option<String>,
}

impl PaymentMetricDimensions {
    pub fn with_provider(mut self, value: impl Into<String>) -> Self {
        self.provider = Some(value.into());
        self
    }

    pub fn with_tenant(mut self, value: impl Into<String>) -> Self {
        self.tenant = Some(value.into());
        self
    }

    pub fn with_payment_outcome(mut self, value: impl Into<String>) -> Self {
        self.payment_outcome = Some(value.into());
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CommercialEventKind {
    HoldFailure,
    SettlementReplay,
    FailoverActivation,
    CallbackReplay,
    Throttling,
}

impl CommercialEventKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::HoldFailure => "hold_failure",
            Self::SettlementReplay => "settlement_replay",
            Self::FailoverActivation => "failover_activation",
            Self::CallbackReplay => "callback_replay",
            Self::Throttling => "throttling",
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CommercialEventDimensions {
    pub route: Option<String>,
    pub tenant: Option<String>,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub payment_outcome: Option<String>,
    pub result: Option<String>,
}

impl CommercialEventDimensions {
    pub fn with_route(mut self, value: impl Into<String>) -> Self {
        self.route = Some(value.into());
        self
    }

    pub fn with_tenant(mut self, value: impl Into<String>) -> Self {
        self.tenant = Some(value.into());
        self
    }

    pub fn with_provider(mut self, value: impl Into<String>) -> Self {
        self.provider = Some(value.into());
        self
    }

    pub fn with_model(mut self, value: impl Into<String>) -> Self {
        self.model = Some(value.into());
        self
    }

    pub fn with_model_option(mut self, value: Option<String>) -> Self {
        self.model = value;
        self
    }

    pub fn with_payment_outcome(mut self, value: impl Into<String>) -> Self {
        self.payment_outcome = Some(value.into());
        self
    }

    pub fn with_result(mut self, value: impl Into<String>) -> Self {
        self.result = Some(value.into());
        self
    }
}

#[derive(Debug, Clone)]
pub struct HttpMetricsRegistry {
    service: Arc<str>,
    state: Arc<Mutex<TelemetryState>>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct HttpMetricKey {
    method: String,
    route: String,
    status: u16,
    tenant: String,
    model: String,
    provider: String,
    billing_mode: String,
    retry_outcome: String,
    failover_outcome: String,
    payment_outcome: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ProviderExecutionMetricKey {
    route: String,
    tenant: String,
    model: String,
    provider: String,
    billing_mode: String,
    retry_outcome: String,
    failover_outcome: String,
    result: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct PaymentMetricKey {
    provider: String,
    tenant: String,
    payment_outcome: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct CommercialEventKey {
    event_kind: String,
    route: String,
    tenant: String,
    provider: String,
    model: String,
    payment_outcome: String,
    result: String,
}

#[derive(Debug, Clone)]
struct TelemetryState {
    http_metrics: BTreeMap<HttpMetricKey, HistogramMetricValue>,
    provider_metrics: BTreeMap<ProviderExecutionMetricKey, HistogramMetricValue>,
    payment_metrics: BTreeMap<PaymentMetricKey, u64>,
    commercial_events: BTreeMap<CommercialEventKey, u64>,
    limiter: MetricCardinalityLimiter,
}

#[derive(Debug, Clone)]
struct HistogramMetricValue {
    count: u64,
    duration_ms_sum: u64,
    bucket_counts: Vec<u64>,
}

impl Default for HistogramMetricValue {
    fn default() -> Self {
        Self {
            count: 0,
            duration_ms_sum: 0,
            bucket_counts: vec![0; LATENCY_BUCKETS_MS.len() + 1],
        }
    }
}

impl HistogramMetricValue {
    fn observe(&mut self, duration_ms: u64) {
        self.count += 1;
        self.duration_ms_sum += duration_ms;

        for (index, boundary) in LATENCY_BUCKETS_MS.iter().enumerate() {
            if duration_ms <= *boundary {
                self.bucket_counts[index] += 1;
            }
        }
        if let Some(last) = self.bucket_counts.last_mut() {
            *last += 1;
        }
    }
}

impl HttpMetricsRegistry {
    pub fn new(service: impl Into<String>) -> Self {
        Self::with_cardinality_limits(service, TelemetryCardinalityLimits::default())
    }

    pub fn with_cardinality_limits(
        service: impl Into<String>,
        limits: TelemetryCardinalityLimits,
    ) -> Self {
        Self {
            service: service.into().into(),
            state: Arc::new(Mutex::new(TelemetryState {
                http_metrics: BTreeMap::new(),
                provider_metrics: BTreeMap::new(),
                payment_metrics: BTreeMap::new(),
                commercial_events: BTreeMap::new(),
                limiter: MetricCardinalityLimiter::new(limits),
            })),
        }
    }

    pub fn service(&self) -> &str {
        &self.service
    }

    pub fn record(&self, method: &str, route: &str, status: u16, duration_ms: u64) {
        self.record_with_dimensions(
            method,
            route,
            status,
            duration_ms,
            HttpMetricDimensions::default(),
        );
    }

    pub fn record_with_dimensions(
        &self,
        method: &str,
        route: &str,
        status: u16,
        duration_ms: u64,
        dimensions: HttpMetricDimensions,
    ) {
        let dimensions = merge_http_dimensions_with_current_context(dimensions)
            .with_route_option(Some(route.to_owned()));
        let mut state = lock_mutex(&self.state);
        let key = HttpMetricKey {
            method: method.to_owned(),
            route: state.limiter.normalize(
                DimensionKind::Route,
                dimensions.route.as_deref(),
                "unmatched",
            ),
            status,
            tenant: state.limiter.normalize(
                DimensionKind::Tenant,
                dimensions.tenant.as_deref(),
                "none",
            ),
            model: state.limiter.normalize(
                DimensionKind::Model,
                dimensions.model.as_deref(),
                "none",
            ),
            provider: state.limiter.normalize(
                DimensionKind::Provider,
                dimensions.provider.as_deref(),
                "none",
            ),
            billing_mode: state.limiter.normalize(
                DimensionKind::BillingMode,
                dimensions.billing_mode.as_deref(),
                "none",
            ),
            retry_outcome: state.limiter.normalize(
                DimensionKind::RetryOutcome,
                dimensions.retry_outcome.as_deref(),
                "none",
            ),
            failover_outcome: state.limiter.normalize(
                DimensionKind::FailoverOutcome,
                dimensions.failover_outcome.as_deref(),
                "none",
            ),
            payment_outcome: state.limiter.normalize(
                DimensionKind::PaymentOutcome,
                dimensions.payment_outcome.as_deref(),
                "none",
            ),
        };

        state
            .http_metrics
            .entry(key)
            .or_default()
            .observe(duration_ms);
    }

    pub fn record_provider_execution(
        &self,
        duration_ms: u64,
        dimensions: ProviderExecutionMetricDimensions,
    ) {
        let dimensions = merge_provider_dimensions_with_current_context(dimensions);
        let mut state = lock_mutex(&self.state);
        let key = ProviderExecutionMetricKey {
            route: state.limiter.normalize(
                DimensionKind::Route,
                dimensions.route.as_deref(),
                "none",
            ),
            tenant: state.limiter.normalize(
                DimensionKind::Tenant,
                dimensions.tenant.as_deref(),
                "none",
            ),
            model: state.limiter.normalize(
                DimensionKind::Model,
                dimensions.model.as_deref(),
                "none",
            ),
            provider: state.limiter.normalize(
                DimensionKind::Provider,
                dimensions.provider.as_deref(),
                "none",
            ),
            billing_mode: state.limiter.normalize(
                DimensionKind::BillingMode,
                dimensions.billing_mode.as_deref(),
                "none",
            ),
            retry_outcome: state.limiter.normalize(
                DimensionKind::RetryOutcome,
                dimensions.retry_outcome.as_deref(),
                "none",
            ),
            failover_outcome: state.limiter.normalize(
                DimensionKind::FailoverOutcome,
                dimensions.failover_outcome.as_deref(),
                "none",
            ),
            result: state.limiter.normalize(
                DimensionKind::Result,
                dimensions.result.as_deref(),
                "none",
            ),
        };

        state
            .provider_metrics
            .entry(key)
            .or_default()
            .observe(duration_ms);
    }

    pub fn record_payment_callback(&self, dimensions: PaymentMetricDimensions) {
        let dimensions = merge_payment_dimensions_with_current_context(dimensions);
        let mut state = lock_mutex(&self.state);
        let key = PaymentMetricKey {
            provider: state.limiter.normalize(
                DimensionKind::Provider,
                dimensions.provider.as_deref(),
                "none",
            ),
            tenant: state.limiter.normalize(
                DimensionKind::Tenant,
                dimensions.tenant.as_deref(),
                "none",
            ),
            payment_outcome: state.limiter.normalize(
                DimensionKind::PaymentOutcome,
                dimensions.payment_outcome.as_deref(),
                "none",
            ),
        };

        *state.payment_metrics.entry(key).or_default() += 1;
    }

    pub fn record_commercial_event(
        &self,
        kind: CommercialEventKind,
        dimensions: CommercialEventDimensions,
    ) {
        let dimensions = merge_commercial_event_dimensions_with_current_context(dimensions);
        let mut state = lock_mutex(&self.state);
        let key = CommercialEventKey {
            event_kind: state.limiter.normalize(
                DimensionKind::EventKind,
                Some(kind.as_str()),
                "none",
            ),
            route: state.limiter.normalize(
                DimensionKind::Route,
                dimensions.route.as_deref(),
                "none",
            ),
            tenant: state.limiter.normalize(
                DimensionKind::Tenant,
                dimensions.tenant.as_deref(),
                "none",
            ),
            provider: state.limiter.normalize(
                DimensionKind::Provider,
                dimensions.provider.as_deref(),
                "none",
            ),
            model: state.limiter.normalize(
                DimensionKind::Model,
                dimensions.model.as_deref(),
                "none",
            ),
            payment_outcome: state.limiter.normalize(
                DimensionKind::PaymentOutcome,
                dimensions.payment_outcome.as_deref(),
                "none",
            ),
            result: state.limiter.normalize(
                DimensionKind::Result,
                dimensions.result.as_deref(),
                "none",
            ),
        };

        *state.commercial_events.entry(key).or_default() += 1;
    }
}

impl HttpMetricsRegistry {
    pub fn render_prometheus(&self) -> String {
        let state = lock_mutex(&self.state);

        let mut output = String::new();
        output.push_str("# HELP sdkwork_service_info Static service identity metric\n");
        output.push_str("# TYPE sdkwork_service_info gauge\n");
        output.push_str(&format!(
            "sdkwork_service_info{{service=\"{}\"}} 1\n",
            escape_label(self.service())
        ));

        output.push_str("# HELP sdkwork_http_requests_total Total HTTP requests observed\n");
        output.push_str("# TYPE sdkwork_http_requests_total counter\n");
        for (key, value) in &state.http_metrics {
            output.push_str(&format!(
                "sdkwork_http_requests_total{{service=\"{}\",method=\"{}\",route=\"{}\",status=\"{}\",tenant=\"{}\",model=\"{}\",provider=\"{}\",billing_mode=\"{}\",retry_outcome=\"{}\",failover_outcome=\"{}\",payment_outcome=\"{}\"}} {}\n",
                escape_label(self.service()),
                escape_label(&key.method),
                escape_label(&key.route),
                key.status,
                escape_label(&key.tenant),
                escape_label(&key.model),
                escape_label(&key.provider),
                escape_label(&key.billing_mode),
                escape_label(&key.retry_outcome),
                escape_label(&key.failover_outcome),
                escape_label(&key.payment_outcome),
                value.count
            ));
        }

        output.push_str(
            "# HELP sdkwork_http_request_duration_ms Request latency histogram in milliseconds\n",
        );
        output.push_str("# TYPE sdkwork_http_request_duration_ms histogram\n");
        for (key, value) in &state.http_metrics {
            render_histogram(
                &mut output,
                "sdkwork_http_request_duration_ms",
                format!(
                    "service=\"{}\",method=\"{}\",route=\"{}\",status=\"{}\",tenant=\"{}\",model=\"{}\",provider=\"{}\",billing_mode=\"{}\",retry_outcome=\"{}\",failover_outcome=\"{}\",payment_outcome=\"{}\"",
                    escape_label(self.service()),
                    escape_label(&key.method),
                    escape_label(&key.route),
                    key.status,
                    escape_label(&key.tenant),
                    escape_label(&key.model),
                    escape_label(&key.provider),
                    escape_label(&key.billing_mode),
                    escape_label(&key.retry_outcome),
                    escape_label(&key.failover_outcome),
                    escape_label(&key.payment_outcome),
                ),
                value,
            );
        }

        output.push_str(
            "# HELP sdkwork_provider_execution_total Total provider execution attempts observed\n",
        );
        output.push_str("# TYPE sdkwork_provider_execution_total counter\n");
        for (key, value) in &state.provider_metrics {
            output.push_str(&format!(
                "sdkwork_provider_execution_total{{service=\"{}\",route=\"{}\",tenant=\"{}\",model=\"{}\",provider=\"{}\",billing_mode=\"{}\",retry_outcome=\"{}\",failover_outcome=\"{}\",result=\"{}\"}} {}\n",
                escape_label(self.service()),
                escape_label(&key.route),
                escape_label(&key.tenant),
                escape_label(&key.model),
                escape_label(&key.provider),
                escape_label(&key.billing_mode),
                escape_label(&key.retry_outcome),
                escape_label(&key.failover_outcome),
                escape_label(&key.result),
                value.count
            ));
        }

        output.push_str("# HELP sdkwork_provider_execution_duration_ms Provider execution latency histogram in milliseconds\n");
        output.push_str("# TYPE sdkwork_provider_execution_duration_ms histogram\n");
        for (key, value) in &state.provider_metrics {
            render_histogram(
                &mut output,
                "sdkwork_provider_execution_duration_ms",
                format!(
                    "service=\"{}\",route=\"{}\",tenant=\"{}\",model=\"{}\",provider=\"{}\",billing_mode=\"{}\",retry_outcome=\"{}\",failover_outcome=\"{}\",result=\"{}\"",
                    escape_label(self.service()),
                    escape_label(&key.route),
                    escape_label(&key.tenant),
                    escape_label(&key.model),
                    escape_label(&key.provider),
                    escape_label(&key.billing_mode),
                    escape_label(&key.retry_outcome),
                    escape_label(&key.failover_outcome),
                    escape_label(&key.result),
                ),
                value,
            );
        }

        output.push_str(
            "# HELP sdkwork_payment_callbacks_total Total payment callbacks by outcome\n",
        );
        output.push_str("# TYPE sdkwork_payment_callbacks_total counter\n");
        for (key, value) in &state.payment_metrics {
            output.push_str(&format!(
                "sdkwork_payment_callbacks_total{{service=\"{}\",provider=\"{}\",tenant=\"{}\",payment_outcome=\"{}\"}} {}\n",
                escape_label(self.service()),
                escape_label(&key.provider),
                escape_label(&key.tenant),
                escape_label(&key.payment_outcome),
                value
            ));
        }

        output.push_str(
            "# HELP sdkwork_commercial_events_total Total structured commercial events emitted\n",
        );
        output.push_str("# TYPE sdkwork_commercial_events_total counter\n");
        for (key, value) in &state.commercial_events {
            output.push_str(&format!(
                "sdkwork_commercial_events_total{{service=\"{}\",event_kind=\"{}\",route=\"{}\",tenant=\"{}\",provider=\"{}\",model=\"{}\",payment_outcome=\"{}\",result=\"{}\"}} {}\n",
                escape_label(self.service()),
                escape_label(&key.event_kind),
                escape_label(&key.route),
                escape_label(&key.tenant),
                escape_label(&key.provider),
                escape_label(&key.model),
                escape_label(&key.payment_outcome),
                escape_label(&key.result),
                value
            ));
        }

        output
    }
}

pub fn annotate_current_http_metrics<F>(mutator: F)
where
    F: FnOnce(&mut HttpMetricDimensions),
{
    let _ = CURRENT_HTTP_METRIC_DIMENSIONS.try_with(|dimensions| {
        let mut dimensions = lock_mutex(dimensions);
        mutator(&mut dimensions);
    });
}

pub fn current_http_metrics_registry() -> Option<Arc<HttpMetricsRegistry>> {
    CURRENT_HTTP_METRICS_REGISTRY.try_with(Arc::clone).ok()
}

pub fn record_current_provider_execution(
    duration_ms: u64,
    dimensions: ProviderExecutionMetricDimensions,
) {
    if let Some(registry) = current_http_metrics_registry() {
        registry.record_provider_execution(duration_ms, dimensions);
    }
}

pub fn record_current_payment_callback(dimensions: PaymentMetricDimensions) {
    if let Some(registry) = current_http_metrics_registry() {
        registry.record_payment_callback(dimensions);
    }
}

pub fn record_current_commercial_event(
    kind: CommercialEventKind,
    dimensions: CommercialEventDimensions,
) {
    if let Some(registry) = current_http_metrics_registry() {
        registry.record_commercial_event(kind, dimensions);
    }
}

pub async fn observe_http_metrics(
    State(registry): State<Arc<HttpMetricsRegistry>>,
    request: Request<axum::body::Body>,
    next: Next,
) -> Response {
    let method = request.method().as_str().to_owned();
    let route = request
        .extensions()
        .get::<MatchedPath>()
        .map(MatchedPath::as_str)
        .unwrap_or("unmatched")
        .to_owned();
    let started_at = Instant::now();
    let dimensions = Arc::new(Mutex::new(
        HttpMetricDimensions::default().with_route(route.clone()),
    ));
    let response = CURRENT_HTTP_METRICS_REGISTRY
        .scope(
            registry.clone(),
            CURRENT_HTTP_METRIC_DIMENSIONS.scope(dimensions.clone(), next.run(request)),
        )
        .await;
    let duration_ms = started_at.elapsed().as_millis() as u64;
    let status = response.status().as_u16();
    let dimensions = lock_mutex(&dimensions).clone();
    registry.record_with_dimensions(&method, &route, status, duration_ms, dimensions);
    response
}

pub async fn observe_http_tracing(
    State(service): State<Arc<str>>,
    mut request: Request<axum::body::Body>,
    next: Next,
) -> Response {
    let method = request.method().as_str().to_owned();
    let route = request
        .extensions()
        .get::<MatchedPath>()
        .map(MatchedPath::as_str)
        .unwrap_or("unmatched")
        .to_owned();
    let request_id = resolved_request_id(&request);
    request
        .extensions_mut()
        .insert(RequestId::new(request_id.clone()));
    let started_at = Instant::now();
    let span = tracing::info_span!(
        "http_request",
        service = %service,
        request_id = %request_id,
        method = %method,
        route = %route
    );
    let mut response = next.run(request).instrument(span.clone()).await;
    let duration_ms = started_at.elapsed().as_millis() as u64;
    let status = response.status().as_u16();
    if let Ok(value) = HeaderValue::from_str(&request_id) {
        response
            .headers_mut()
            .insert(HeaderName::from_static(REQUEST_ID_HEADER), value);
    }
    tracing::info!(parent: &span, status, duration_ms, "completed request");
    response
}

pub fn init_tracing(service: &str) {
    TRACING_INIT.get_or_init(|| {
        let _ = tracing_subscriber::fmt()
            .compact()
            .with_target(false)
            .with_max_level(tracing::Level::INFO)
            .try_init();
        tracing::info!(service = service, "tracing initialized");
    });
}

fn render_histogram(
    output: &mut String,
    metric_name: &str,
    labels: String,
    value: &HistogramMetricValue,
) {
    for (index, boundary) in LATENCY_BUCKETS_MS.iter().enumerate() {
        output.push_str(&format!(
            "{metric_name}_bucket{{{labels},le=\"{}\"}} {}\n",
            boundary, value.bucket_counts[index]
        ));
    }
    output.push_str(&format!(
        "{metric_name}_bucket{{{labels},le=\"+Inf\"}} {}\n",
        value.bucket_counts.last().copied().unwrap_or_default()
    ));
    output.push_str(&format!(
        "{metric_name}_sum{{{labels}}} {}\n",
        value.duration_ms_sum
    ));
    output.push_str(&format!(
        "{metric_name}_count{{{labels}}} {}\n",
        value.count
    ));
}

fn merge_http_dimensions_with_current_context(
    mut dimensions: HttpMetricDimensions,
) -> HttpMetricDimensions {
    if let Some(current) = current_http_metric_dimensions() {
        if dimensions.route.is_none() {
            dimensions.route = current.route;
        }
        if dimensions.tenant.is_none() {
            dimensions.tenant = current.tenant;
        }
        if dimensions.model.is_none() {
            dimensions.model = current.model;
        }
        if dimensions.provider.is_none() {
            dimensions.provider = current.provider;
        }
        if dimensions.billing_mode.is_none() {
            dimensions.billing_mode = current.billing_mode;
        }
        if dimensions.retry_outcome.is_none() {
            dimensions.retry_outcome = current.retry_outcome;
        }
        if dimensions.failover_outcome.is_none() {
            dimensions.failover_outcome = current.failover_outcome;
        }
        if dimensions.payment_outcome.is_none() {
            dimensions.payment_outcome = current.payment_outcome;
        }
    }
    dimensions
}

fn merge_provider_dimensions_with_current_context(
    mut dimensions: ProviderExecutionMetricDimensions,
) -> ProviderExecutionMetricDimensions {
    if let Some(current) = current_http_metric_dimensions() {
        if dimensions.route.is_none() {
            dimensions.route = current.route;
        }
        if dimensions.tenant.is_none() {
            dimensions.tenant = current.tenant;
        }
        if dimensions.model.is_none() {
            dimensions.model = current.model;
        }
        if dimensions.provider.is_none() {
            dimensions.provider = current.provider;
        }
        if dimensions.billing_mode.is_none() {
            dimensions.billing_mode = current.billing_mode;
        }
        if dimensions.retry_outcome.is_none() {
            dimensions.retry_outcome = current.retry_outcome;
        }
        if dimensions.failover_outcome.is_none() {
            dimensions.failover_outcome = current.failover_outcome;
        }
    }
    dimensions
}

fn merge_payment_dimensions_with_current_context(
    mut dimensions: PaymentMetricDimensions,
) -> PaymentMetricDimensions {
    if let Some(current) = current_http_metric_dimensions() {
        if dimensions.provider.is_none() {
            dimensions.provider = current.provider;
        }
        if dimensions.tenant.is_none() {
            dimensions.tenant = current.tenant;
        }
        if dimensions.payment_outcome.is_none() {
            dimensions.payment_outcome = current.payment_outcome;
        }
    }
    dimensions
}

fn merge_commercial_event_dimensions_with_current_context(
    mut dimensions: CommercialEventDimensions,
) -> CommercialEventDimensions {
    if let Some(current) = current_http_metric_dimensions() {
        if dimensions.route.is_none() {
            dimensions.route = current.route;
        }
        if dimensions.tenant.is_none() {
            dimensions.tenant = current.tenant;
        }
        if dimensions.provider.is_none() {
            dimensions.provider = current.provider;
        }
        if dimensions.model.is_none() {
            dimensions.model = current.model;
        }
        if dimensions.payment_outcome.is_none() {
            dimensions.payment_outcome = current.payment_outcome;
        }
    }
    dimensions
}

fn current_http_metric_dimensions() -> Option<HttpMetricDimensions> {
    CURRENT_HTTP_METRIC_DIMENSIONS
        .try_with(|dimensions| lock_mutex(dimensions).clone())
        .ok()
}

fn sanitize_label_value(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }

    let normalized = trimmed
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.' | '/' | ':') {
                ch
            } else {
                '_'
            }
        })
        .collect::<String>();
    let normalized = normalized.trim_matches('_');
    if normalized.is_empty() {
        None
    } else {
        Some(normalized.chars().take(64).collect())
    }
}

fn lock_mutex<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    match mutex.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

fn escape_label(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn resolved_request_id(request: &Request<axum::body::Body>) -> String {
    request
        .headers()
        .get(REQUEST_ID_HEADER)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(generate_request_id)
}

fn generate_request_id() -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let sequence = REQUEST_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("sdkw-{millis:x}-{sequence:x}")
}

pub async fn with_current_http_metrics_registry<T, F>(
    registry: Arc<HttpMetricsRegistry>,
    future: F,
) -> T
where
    F: Future<Output = T>,
{
    CURRENT_HTTP_METRICS_REGISTRY
        .scope(
            registry,
            CURRENT_HTTP_METRIC_DIMENSIONS.scope(
                Arc::new(Mutex::new(HttpMetricDimensions::default())),
                future,
            ),
        )
        .await
}
