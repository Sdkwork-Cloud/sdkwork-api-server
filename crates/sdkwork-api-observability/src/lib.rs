mod commercial;

use std::collections::BTreeMap;
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

pub use commercial::{
    annotate_current_http_metrics, CommercialEventDimensions, CommercialEventKind,
    HttpMetricDimensions, PaymentMetricDimensions, ProviderExecutionMetricDimensions,
    TelemetryCardinalityLimits,
};

pub const REQUEST_ID_HEADER: &str = "x-request-id";

static REQUEST_ID_COUNTER: AtomicU64 = AtomicU64::new(1);
static TRACING_INIT: OnceLock<()> = OnceLock::new();
static SHARED_SERVICE_METRICS: OnceLock<Mutex<BTreeMap<String, Arc<ServiceMetricsState>>>> =
    OnceLock::new();

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

#[derive(Debug, Clone)]
pub struct HttpMetricsRegistry {
    service: Arc<str>,
    state: Arc<ServiceMetricsState>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct HttpMetricKey {
    method: String,
    route: String,
    status: u16,
}

#[derive(Debug, Clone, Default)]
struct HttpMetricValue {
    count: u64,
    duration_ms_sum: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct UpstreamMetricKey {
    capability: String,
    provider: String,
    outcome: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct UpstreamRetryMetricKey {
    capability: String,
    provider: String,
    outcome: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct UpstreamRetryReasonMetricKey {
    capability: String,
    provider: String,
    outcome: String,
    reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct UpstreamRetryDelayMetricKey {
    capability: String,
    provider: String,
    source: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct GatewayFailoverMetricKey {
    capability: String,
    from_provider: String,
    to_provider: String,
    outcome: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ProviderHealthMetricKey {
    provider: String,
    runtime: String,
}

#[derive(Debug, Clone, Default)]
struct ProviderHealthMetricValue {
    healthy: u64,
    observed_at_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ProviderHealthPersistFailureMetricKey {
    provider: String,
    runtime: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ProviderHealthRecoveryProbeMetricKey {
    provider: String,
    outcome: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct GatewayExecutionContextFailureMetricKey {
    capability: String,
    provider: String,
    reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct CommerceReconciliationAttemptMetricKey {
    outcome: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct MarketingRecoveryAttemptMetricKey {
    outcome: String,
}

#[derive(Debug, Clone, Default)]
struct CommerceReconciliationMetricValue {
    backlog_orders: u64,
    checkpoint_lag_ms: u64,
    processed_orders_total: u64,
    last_success_at_ms: u64,
    last_failure_at_ms: u64,
}

#[derive(Debug, Clone, Default)]
struct MarketingRecoveryMetricValue {
    scanned_reservations_total: u64,
    expired_reservations_total: u64,
    released_codes_total: u64,
    released_budget_minor_total: u64,
    outbox_events_total: u64,
    last_success_at_ms: u64,
    last_failure_at_ms: u64,
}

#[derive(Debug)]
struct ServiceMetricsState {
    commercial: commercial::CommercialMetricsState,
    http_metrics: Mutex<BTreeMap<HttpMetricKey, HttpMetricValue>>,
    upstream_metrics: Mutex<BTreeMap<UpstreamMetricKey, u64>>,
    upstream_retry_metrics: Mutex<BTreeMap<UpstreamRetryMetricKey, u64>>,
    upstream_retry_reason_metrics: Mutex<BTreeMap<UpstreamRetryReasonMetricKey, u64>>,
    upstream_retry_delay_metrics: Mutex<BTreeMap<UpstreamRetryDelayMetricKey, u64>>,
    gateway_failover_metrics: Mutex<BTreeMap<GatewayFailoverMetricKey, u64>>,
    provider_health_metrics: Mutex<BTreeMap<ProviderHealthMetricKey, ProviderHealthMetricValue>>,
    provider_health_persist_failure_metrics:
        Mutex<BTreeMap<ProviderHealthPersistFailureMetricKey, u64>>,
    provider_health_recovery_probe_metrics:
        Mutex<BTreeMap<ProviderHealthRecoveryProbeMetricKey, u64>>,
    gateway_execution_context_failure_metrics:
        Mutex<BTreeMap<GatewayExecutionContextFailureMetricKey, u64>>,
    commerce_reconciliation_attempt_metrics:
        Mutex<BTreeMap<CommerceReconciliationAttemptMetricKey, u64>>,
    commerce_reconciliation_metrics: Mutex<CommerceReconciliationMetricValue>,
    marketing_recovery_attempt_metrics: Mutex<BTreeMap<MarketingRecoveryAttemptMetricKey, u64>>,
    marketing_recovery_metrics: Mutex<MarketingRecoveryMetricValue>,
}

impl ServiceMetricsState {
    fn new(cardinality_limits: TelemetryCardinalityLimits) -> Self {
        Self {
            commercial: commercial::CommercialMetricsState::new(cardinality_limits),
            http_metrics: Mutex::new(BTreeMap::new()),
            upstream_metrics: Mutex::new(BTreeMap::new()),
            upstream_retry_metrics: Mutex::new(BTreeMap::new()),
            upstream_retry_reason_metrics: Mutex::new(BTreeMap::new()),
            upstream_retry_delay_metrics: Mutex::new(BTreeMap::new()),
            gateway_failover_metrics: Mutex::new(BTreeMap::new()),
            provider_health_metrics: Mutex::new(BTreeMap::new()),
            provider_health_persist_failure_metrics: Mutex::new(BTreeMap::new()),
            provider_health_recovery_probe_metrics: Mutex::new(BTreeMap::new()),
            gateway_execution_context_failure_metrics: Mutex::new(BTreeMap::new()),
            commerce_reconciliation_attempt_metrics: Mutex::new(BTreeMap::new()),
            commerce_reconciliation_metrics: Mutex::new(
                CommerceReconciliationMetricValue::default(),
            ),
            marketing_recovery_attempt_metrics: Mutex::new(BTreeMap::new()),
            marketing_recovery_metrics: Mutex::new(MarketingRecoveryMetricValue::default()),
        }
    }
}

impl HttpMetricsRegistry {
    pub fn new(service: impl Into<String>) -> Self {
        let service = service.into();
        Self {
            service: service.clone().into(),
            state: shared_service_metrics(&service, TelemetryCardinalityLimits::default()),
        }
    }

    pub fn with_cardinality_limits(
        service: impl Into<String>,
        limits: TelemetryCardinalityLimits,
    ) -> Self {
        let service = service.into();
        Self {
            service: service.clone().into(),
            state: Arc::new(ServiceMetricsState::new(limits)),
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
        let key = HttpMetricKey {
            method: method.to_owned(),
            route: route.to_owned(),
            status,
        };

        let mut metrics = match self.state.http_metrics.lock() {
            Ok(metrics) => metrics,
            Err(poisoned) => poisoned.into_inner(),
        };
        let entry = metrics.entry(key).or_default();
        entry.count += 1;
        entry.duration_ms_sum += duration_ms;
        drop(metrics);

        self.state
            .commercial
            .record_http(method, route, status, duration_ms, dimensions);
    }

    pub fn record_provider_execution(
        &self,
        duration_ms: u64,
        dimensions: ProviderExecutionMetricDimensions,
    ) {
        self.state
            .commercial
            .record_provider_execution(duration_ms, dimensions);
    }

    pub fn record_payment_callback(&self, dimensions: PaymentMetricDimensions) {
        self.state.commercial.record_payment_callback(dimensions);
    }

    pub fn record_commercial_event(
        &self,
        event_kind: CommercialEventKind,
        dimensions: CommercialEventDimensions,
    ) {
        self.state
            .commercial
            .record_commercial_event(event_kind, dimensions);
    }

    pub fn record_upstream_outcome(&self, capability: &str, provider: &str, outcome: &str) {
        let key = UpstreamMetricKey {
            capability: capability.to_owned(),
            provider: provider.to_owned(),
            outcome: outcome.to_owned(),
        };

        let mut metrics = match self.state.upstream_metrics.lock() {
            Ok(metrics) => metrics,
            Err(poisoned) => poisoned.into_inner(),
        };
        let entry = metrics.entry(key).or_default();
        *entry += 1;
    }

    pub fn record_upstream_retry(&self, capability: &str, provider: &str, outcome: &str) {
        let key = UpstreamRetryMetricKey {
            capability: capability.to_owned(),
            provider: provider.to_owned(),
            outcome: outcome.to_owned(),
        };

        let mut metrics = match self.state.upstream_retry_metrics.lock() {
            Ok(metrics) => metrics,
            Err(poisoned) => poisoned.into_inner(),
        };
        let entry = metrics.entry(key).or_default();
        *entry += 1;
    }

    pub fn record_upstream_retry_reason(
        &self,
        capability: &str,
        provider: &str,
        outcome: &str,
        reason: &str,
    ) {
        let key = UpstreamRetryReasonMetricKey {
            capability: capability.to_owned(),
            provider: provider.to_owned(),
            outcome: outcome.to_owned(),
            reason: reason.to_owned(),
        };

        let mut metrics = match self.state.upstream_retry_reason_metrics.lock() {
            Ok(metrics) => metrics,
            Err(poisoned) => poisoned.into_inner(),
        };
        let entry = metrics.entry(key).or_default();
        *entry += 1;
    }

    pub fn record_upstream_retry_delay(
        &self,
        capability: &str,
        provider: &str,
        source: &str,
        delay_ms: u64,
    ) {
        let key = UpstreamRetryDelayMetricKey {
            capability: capability.to_owned(),
            provider: provider.to_owned(),
            source: source.to_owned(),
        };

        let mut metrics = match self.state.upstream_retry_delay_metrics.lock() {
            Ok(metrics) => metrics,
            Err(poisoned) => poisoned.into_inner(),
        };
        let entry = metrics.entry(key).or_default();
        *entry += delay_ms;
    }

    pub fn record_gateway_failover(
        &self,
        capability: &str,
        from_provider: &str,
        to_provider: &str,
        outcome: &str,
    ) {
        let key = GatewayFailoverMetricKey {
            capability: capability.to_owned(),
            from_provider: from_provider.to_owned(),
            to_provider: to_provider.to_owned(),
            outcome: outcome.to_owned(),
        };

        let mut metrics = match self.state.gateway_failover_metrics.lock() {
            Ok(metrics) => metrics,
            Err(poisoned) => poisoned.into_inner(),
        };
        let entry = metrics.entry(key).or_default();
        *entry += 1;
        drop(metrics);

        if outcome == "success" {
            self.state.commercial.record_failover_success(
                self.service(),
                from_provider,
                to_provider,
            );
        }
    }

    pub fn record_provider_health(
        &self,
        provider: &str,
        runtime: &str,
        healthy: bool,
        observed_at_ms: u64,
    ) {
        let key = ProviderHealthMetricKey {
            provider: provider.to_owned(),
            runtime: runtime.to_owned(),
        };

        let mut metrics = match self.state.provider_health_metrics.lock() {
            Ok(metrics) => metrics,
            Err(poisoned) => poisoned.into_inner(),
        };
        metrics.insert(
            key,
            ProviderHealthMetricValue {
                healthy: healthy.into(),
                observed_at_ms,
            },
        );
    }

    pub fn record_provider_health_persist_failure(&self, provider: &str, runtime: &str) {
        let key = ProviderHealthPersistFailureMetricKey {
            provider: provider.to_owned(),
            runtime: runtime.to_owned(),
        };

        let mut metrics = match self.state.provider_health_persist_failure_metrics.lock() {
            Ok(metrics) => metrics,
            Err(poisoned) => poisoned.into_inner(),
        };
        let entry = metrics.entry(key).or_default();
        *entry += 1;
    }

    pub fn record_provider_health_recovery_probe(&self, provider: &str, outcome: &str) {
        let key = ProviderHealthRecoveryProbeMetricKey {
            provider: provider.to_owned(),
            outcome: outcome.to_owned(),
        };

        let mut metrics = match self.state.provider_health_recovery_probe_metrics.lock() {
            Ok(metrics) => metrics,
            Err(poisoned) => poisoned.into_inner(),
        };
        let entry = metrics.entry(key).or_default();
        *entry += 1;
    }

    pub fn record_gateway_execution_context_failure(
        &self,
        capability: &str,
        provider: &str,
        reason: &str,
    ) {
        let key = GatewayExecutionContextFailureMetricKey {
            capability: capability.to_owned(),
            provider: provider.to_owned(),
            reason: reason.to_owned(),
        };

        let mut metrics = match self.state.gateway_execution_context_failure_metrics.lock() {
            Ok(metrics) => metrics,
            Err(poisoned) => poisoned.into_inner(),
        };
        let entry = metrics.entry(key).or_default();
        *entry += 1;
    }

    pub fn record_commerce_reconciliation_success(
        &self,
        backlog_orders: u64,
        checkpoint_lag_ms: u64,
        processed_orders: u64,
        observed_at_ms: u64,
    ) {
        self.record_commerce_reconciliation_attempt("success");
        let mut metrics = match self.state.commerce_reconciliation_metrics.lock() {
            Ok(metrics) => metrics,
            Err(poisoned) => poisoned.into_inner(),
        };
        metrics.backlog_orders = backlog_orders;
        metrics.checkpoint_lag_ms = checkpoint_lag_ms;
        metrics.processed_orders_total += processed_orders;
        metrics.last_success_at_ms = observed_at_ms;
    }

    pub fn record_commerce_reconciliation_failure(
        &self,
        backlog_orders: u64,
        checkpoint_lag_ms: u64,
        observed_at_ms: u64,
    ) {
        self.record_commerce_reconciliation_attempt("failure");
        let mut metrics = match self.state.commerce_reconciliation_metrics.lock() {
            Ok(metrics) => metrics,
            Err(poisoned) => poisoned.into_inner(),
        };
        metrics.backlog_orders = backlog_orders;
        metrics.checkpoint_lag_ms = checkpoint_lag_ms;
        metrics.last_failure_at_ms = observed_at_ms;
    }

    fn record_commerce_reconciliation_attempt(&self, outcome: &str) {
        let key = CommerceReconciliationAttemptMetricKey {
            outcome: outcome.to_owned(),
        };
        let mut metrics = match self.state.commerce_reconciliation_attempt_metrics.lock() {
            Ok(metrics) => metrics,
            Err(poisoned) => poisoned.into_inner(),
        };
        let entry = metrics.entry(key).or_default();
        *entry += 1;
    }

    pub fn record_marketing_recovery_success(
        &self,
        scanned_reservations: u64,
        expired_reservations: u64,
        released_codes: u64,
        released_budget_minor: u64,
        outbox_events: u64,
        observed_at_ms: u64,
    ) {
        self.record_marketing_recovery_attempt("success");
        let mut metrics = match self.state.marketing_recovery_metrics.lock() {
            Ok(metrics) => metrics,
            Err(poisoned) => poisoned.into_inner(),
        };
        metrics.scanned_reservations_total += scanned_reservations;
        metrics.expired_reservations_total += expired_reservations;
        metrics.released_codes_total += released_codes;
        metrics.released_budget_minor_total += released_budget_minor;
        metrics.outbox_events_total += outbox_events;
        metrics.last_success_at_ms = observed_at_ms;
    }

    pub fn record_marketing_recovery_failure(&self, observed_at_ms: u64) {
        self.record_marketing_recovery_attempt("failure");
        let mut metrics = match self.state.marketing_recovery_metrics.lock() {
            Ok(metrics) => metrics,
            Err(poisoned) => poisoned.into_inner(),
        };
        metrics.last_failure_at_ms = observed_at_ms;
    }

    fn record_marketing_recovery_attempt(&self, outcome: &str) {
        let key = MarketingRecoveryAttemptMetricKey {
            outcome: outcome.to_owned(),
        };
        let mut metrics = match self.state.marketing_recovery_attempt_metrics.lock() {
            Ok(metrics) => metrics,
            Err(poisoned) => poisoned.into_inner(),
        };
        let entry = metrics.entry(key).or_default();
        *entry += 1;
    }

    pub fn render_prometheus(&self) -> String {
        let metrics = match self.state.http_metrics.lock() {
            Ok(metrics) => metrics,
            Err(poisoned) => poisoned.into_inner(),
        };
        let upstream_metrics = match self.state.upstream_metrics.lock() {
            Ok(metrics) => metrics,
            Err(poisoned) => poisoned.into_inner(),
        };
        let upstream_retry_metrics = match self.state.upstream_retry_metrics.lock() {
            Ok(metrics) => metrics,
            Err(poisoned) => poisoned.into_inner(),
        };
        let upstream_retry_reason_metrics = match self.state.upstream_retry_reason_metrics.lock() {
            Ok(metrics) => metrics,
            Err(poisoned) => poisoned.into_inner(),
        };
        let upstream_retry_delay_metrics = match self.state.upstream_retry_delay_metrics.lock() {
            Ok(metrics) => metrics,
            Err(poisoned) => poisoned.into_inner(),
        };
        let gateway_failover_metrics = match self.state.gateway_failover_metrics.lock() {
            Ok(metrics) => metrics,
            Err(poisoned) => poisoned.into_inner(),
        };
        let provider_health_metrics = match self.state.provider_health_metrics.lock() {
            Ok(metrics) => metrics,
            Err(poisoned) => poisoned.into_inner(),
        };
        let provider_health_persist_failure_metrics =
            match self.state.provider_health_persist_failure_metrics.lock() {
                Ok(metrics) => metrics,
                Err(poisoned) => poisoned.into_inner(),
            };
        let provider_health_recovery_probe_metrics =
            match self.state.provider_health_recovery_probe_metrics.lock() {
                Ok(metrics) => metrics,
                Err(poisoned) => poisoned.into_inner(),
            };
        let gateway_execution_context_failure_metrics =
            match self.state.gateway_execution_context_failure_metrics.lock() {
                Ok(metrics) => metrics,
                Err(poisoned) => poisoned.into_inner(),
            };
        let commerce_reconciliation_attempt_metrics =
            match self.state.commerce_reconciliation_attempt_metrics.lock() {
                Ok(metrics) => metrics,
                Err(poisoned) => poisoned.into_inner(),
            };
        let commerce_reconciliation_metrics =
            match self.state.commerce_reconciliation_metrics.lock() {
                Ok(metrics) => metrics,
                Err(poisoned) => poisoned.into_inner(),
            };
        let marketing_recovery_attempt_metrics =
            match self.state.marketing_recovery_attempt_metrics.lock() {
                Ok(metrics) => metrics,
                Err(poisoned) => poisoned.into_inner(),
            };
        let marketing_recovery_metrics = match self.state.marketing_recovery_metrics.lock() {
            Ok(metrics) => metrics,
            Err(poisoned) => poisoned.into_inner(),
        };

        let mut output = String::new();
        output.push_str("# HELP sdkwork_service_info Static service identity metric\n");
        output.push_str("# TYPE sdkwork_service_info gauge\n");
        output.push_str(&format!(
            "sdkwork_service_info{{service=\"{}\"}} 1\n",
            escape_label(self.service())
        ));

        output.push_str("# HELP sdkwork_http_requests_total Total HTTP requests observed\n");
        output.push_str("# TYPE sdkwork_http_requests_total counter\n");
        for (key, value) in metrics.iter() {
            output.push_str(&format!(
                "sdkwork_http_requests_total{{service=\"{}\",method=\"{}\",route=\"{}\",status=\"{}\"}} {}\n",
                escape_label(self.service()),
                escape_label(&key.method),
                escape_label(&key.route),
                key.status,
                value.count
            ));
        }

        output.push_str(
            "# HELP sdkwork_http_request_duration_ms_sum Cumulative request duration in milliseconds\n",
        );
        output.push_str("# TYPE sdkwork_http_request_duration_ms_sum counter\n");
        for (key, value) in metrics.iter() {
            output.push_str(&format!(
                "sdkwork_http_request_duration_ms_sum{{service=\"{}\",method=\"{}\",route=\"{}\",status=\"{}\"}} {}\n",
                escape_label(self.service()),
                escape_label(&key.method),
                escape_label(&key.route),
                key.status,
                value.duration_ms_sum
            ));
        }

        output.push_str(
            "# HELP sdkwork_http_request_duration_ms_count Request count paired with duration summaries\n",
        );
        output.push_str("# TYPE sdkwork_http_request_duration_ms_count counter\n");
        for (key, value) in metrics.iter() {
            output.push_str(&format!(
                "sdkwork_http_request_duration_ms_count{{service=\"{}\",method=\"{}\",route=\"{}\",status=\"{}\"}} {}\n",
                escape_label(self.service()),
                escape_label(&key.method),
                escape_label(&key.route),
                key.status,
                value.count
            ));
        }

        self.state
            .commercial
            .render_prometheus(self.service(), &mut output);

        output.push_str(
            "# HELP sdkwork_upstream_requests_total Total upstream execution outcomes observed\n",
        );
        output.push_str("# TYPE sdkwork_upstream_requests_total counter\n");
        for (key, value) in upstream_metrics.iter() {
            output.push_str(&format!(
                "sdkwork_upstream_requests_total{{service=\"{}\",capability=\"{}\",provider=\"{}\",outcome=\"{}\"}} {}\n",
                escape_label(self.service()),
                escape_label(&key.capability),
                escape_label(&key.provider),
                escape_label(&key.outcome),
                value
            ));
        }

        output.push_str("# HELP sdkwork_upstream_retries_total Total upstream retry control decisions observed\n");
        output.push_str("# TYPE sdkwork_upstream_retries_total counter\n");
        for (key, value) in upstream_retry_metrics.iter() {
            output.push_str(&format!(
                "sdkwork_upstream_retries_total{{service=\"{}\",capability=\"{}\",provider=\"{}\",outcome=\"{}\"}} {}\n",
                escape_label(self.service()),
                escape_label(&key.capability),
                escape_label(&key.provider),
                escape_label(&key.outcome),
                value
            ));
        }

        output.push_str(
            "# HELP sdkwork_upstream_retry_reasons_total Total upstream retry reasons observed\n",
        );
        output.push_str("# TYPE sdkwork_upstream_retry_reasons_total counter\n");
        for (key, value) in upstream_retry_reason_metrics.iter() {
            output.push_str(&format!(
                "sdkwork_upstream_retry_reasons_total{{service=\"{}\",capability=\"{}\",provider=\"{}\",outcome=\"{}\",reason=\"{}\"}} {}\n",
                escape_label(self.service()),
                escape_label(&key.capability),
                escape_label(&key.provider),
                escape_label(&key.outcome),
                escape_label(&key.reason),
                value
            ));
        }

        output.push_str(
            "# HELP sdkwork_upstream_retry_delay_ms_total Cumulative upstream retry delay in milliseconds\n",
        );
        output.push_str("# TYPE sdkwork_upstream_retry_delay_ms_total counter\n");
        for (key, value) in upstream_retry_delay_metrics.iter() {
            output.push_str(&format!(
                "sdkwork_upstream_retry_delay_ms_total{{service=\"{}\",capability=\"{}\",provider=\"{}\",source=\"{}\"}} {}\n",
                escape_label(self.service()),
                escape_label(&key.capability),
                escape_label(&key.provider),
                escape_label(&key.source),
                value
            ));
        }

        output.push_str(
            "# HELP sdkwork_gateway_failovers_total Total gateway failover outcomes observed\n",
        );
        output.push_str("# TYPE sdkwork_gateway_failovers_total counter\n");
        for (key, value) in gateway_failover_metrics.iter() {
            output.push_str(&format!(
                "sdkwork_gateway_failovers_total{{service=\"{}\",capability=\"{}\",from_provider=\"{}\",to_provider=\"{}\",outcome=\"{}\"}} {}\n",
                escape_label(self.service()),
                escape_label(&key.capability),
                escape_label(&key.from_provider),
                escape_label(&key.to_provider),
                escape_label(&key.outcome),
                value
            ));
        }

        output.push_str(
            "# HELP sdkwork_provider_health_status Latest observed provider health state where 1 is healthy and 0 is unhealthy\n",
        );
        output.push_str("# TYPE sdkwork_provider_health_status gauge\n");
        for (key, value) in provider_health_metrics.iter() {
            output.push_str(&format!(
                "sdkwork_provider_health_status{{service=\"{}\",provider=\"{}\",runtime=\"{}\"}} {}\n",
                escape_label(self.service()),
                escape_label(&key.provider),
                escape_label(&key.runtime),
                value.healthy
            ));
        }

        output.push_str(
            "# HELP sdkwork_provider_health_observed_at_ms Latest observed provider health timestamp in unix milliseconds\n",
        );
        output.push_str("# TYPE sdkwork_provider_health_observed_at_ms gauge\n");
        for (key, value) in provider_health_metrics.iter() {
            output.push_str(&format!(
                "sdkwork_provider_health_observed_at_ms{{service=\"{}\",provider=\"{}\",runtime=\"{}\"}} {}\n",
                escape_label(self.service()),
                escape_label(&key.provider),
                escape_label(&key.runtime),
                value.observed_at_ms
            ));
        }

        output.push_str(
            "# HELP sdkwork_provider_health_persist_failures_total Total provider health snapshot persistence failures observed\n",
        );
        output.push_str("# TYPE sdkwork_provider_health_persist_failures_total counter\n");
        for (key, value) in provider_health_persist_failure_metrics.iter() {
            output.push_str(&format!(
                "sdkwork_provider_health_persist_failures_total{{service=\"{}\",provider=\"{}\",runtime=\"{}\"}} {}\n",
                escape_label(self.service()),
                escape_label(&key.provider),
                escape_label(&key.runtime),
                value
            ));
        }

        output.push_str(
            "# HELP sdkwork_provider_health_recovery_probes_total Total provider health recovery probe outcomes observed\n",
        );
        output.push_str("# TYPE sdkwork_provider_health_recovery_probes_total counter\n");
        for (key, value) in provider_health_recovery_probe_metrics.iter() {
            output.push_str(&format!(
                "sdkwork_provider_health_recovery_probes_total{{service=\"{}\",provider=\"{}\",outcome=\"{}\"}} {}\n",
                escape_label(self.service()),
                escape_label(&key.provider),
                escape_label(&key.outcome),
                value
            ));
        }

        output.push_str(
            "# HELP sdkwork_gateway_execution_context_failures_total Total gateway-local execution context failures observed\n",
        );
        output.push_str("# TYPE sdkwork_gateway_execution_context_failures_total counter\n");
        for (key, value) in gateway_execution_context_failure_metrics.iter() {
            output.push_str(&format!(
                "sdkwork_gateway_execution_context_failures_total{{service=\"{}\",capability=\"{}\",provider=\"{}\",reason=\"{}\"}} {}\n",
                escape_label(self.service()),
                escape_label(&key.capability),
                escape_label(&key.provider),
                escape_label(&key.reason),
                value
            ));
        }

        output.push_str(
            "# HELP sdkwork_commerce_reconciliation_attempts_total Total commerce reconciliation outcomes observed\n",
        );
        output.push_str("# TYPE sdkwork_commerce_reconciliation_attempts_total counter\n");
        for (key, value) in commerce_reconciliation_attempt_metrics.iter() {
            output.push_str(&format!(
                "sdkwork_commerce_reconciliation_attempts_total{{service=\"{}\",outcome=\"{}\"}} {}\n",
                escape_label(self.service()),
                escape_label(&key.outcome),
                value
            ));
        }

        output.push_str(
            "# HELP sdkwork_commerce_reconciliation_processed_orders_total Total commerce orders reconciled into canonical accounts\n",
        );
        output.push_str("# TYPE sdkwork_commerce_reconciliation_processed_orders_total counter\n");
        output.push_str(&format!(
            "sdkwork_commerce_reconciliation_processed_orders_total{{service=\"{}\"}} {}\n",
            escape_label(self.service()),
            commerce_reconciliation_metrics.processed_orders_total
        ));

        output.push_str(
            "# HELP sdkwork_commerce_reconciliation_backlog_orders Latest observed unreconciled commerce order backlog\n",
        );
        output.push_str("# TYPE sdkwork_commerce_reconciliation_backlog_orders gauge\n");
        output.push_str(&format!(
            "sdkwork_commerce_reconciliation_backlog_orders{{service=\"{}\"}} {}\n",
            escape_label(self.service()),
            commerce_reconciliation_metrics.backlog_orders
        ));

        output.push_str(
            "# HELP sdkwork_commerce_reconciliation_checkpoint_lag_ms Latest observed lag between checkpoint and newest commerce order progress\n",
        );
        output.push_str("# TYPE sdkwork_commerce_reconciliation_checkpoint_lag_ms gauge\n");
        output.push_str(&format!(
            "sdkwork_commerce_reconciliation_checkpoint_lag_ms{{service=\"{}\"}} {}\n",
            escape_label(self.service()),
            commerce_reconciliation_metrics.checkpoint_lag_ms
        ));

        output.push_str(
            "# HELP sdkwork_commerce_reconciliation_last_success_at_ms Unix timestamp in milliseconds for the latest successful commerce reconciliation run\n",
        );
        output.push_str("# TYPE sdkwork_commerce_reconciliation_last_success_at_ms gauge\n");
        output.push_str(&format!(
            "sdkwork_commerce_reconciliation_last_success_at_ms{{service=\"{}\"}} {}\n",
            escape_label(self.service()),
            commerce_reconciliation_metrics.last_success_at_ms
        ));

        output.push_str(
            "# HELP sdkwork_commerce_reconciliation_last_failure_at_ms Unix timestamp in milliseconds for the latest failed commerce reconciliation run\n",
        );
        output.push_str("# TYPE sdkwork_commerce_reconciliation_last_failure_at_ms gauge\n");
        output.push_str(&format!(
            "sdkwork_commerce_reconciliation_last_failure_at_ms{{service=\"{}\"}} {}\n",
            escape_label(self.service()),
            commerce_reconciliation_metrics.last_failure_at_ms
        ));

        output.push_str(
            "# HELP sdkwork_marketing_recovery_attempts_total Total marketing recovery job outcomes observed\n",
        );
        output.push_str("# TYPE sdkwork_marketing_recovery_attempts_total counter\n");
        for (key, value) in marketing_recovery_attempt_metrics.iter() {
            output.push_str(&format!(
                "sdkwork_marketing_recovery_attempts_total{{service=\"{}\",outcome=\"{}\"}} {}\n",
                escape_label(self.service()),
                escape_label(&key.outcome),
                value
            ));
        }

        output.push_str(
            "# HELP sdkwork_marketing_recovery_scanned_reservations_total Total reservations scanned by marketing recovery jobs\n",
        );
        output.push_str("# TYPE sdkwork_marketing_recovery_scanned_reservations_total counter\n");
        output.push_str(&format!(
            "sdkwork_marketing_recovery_scanned_reservations_total{{service=\"{}\"}} {}\n",
            escape_label(self.service()),
            marketing_recovery_metrics.scanned_reservations_total
        ));

        output.push_str(
            "# HELP sdkwork_marketing_expired_reservations_total Total stale coupon reservations expired by recovery jobs\n",
        );
        output.push_str("# TYPE sdkwork_marketing_expired_reservations_total counter\n");
        output.push_str(&format!(
            "sdkwork_marketing_expired_reservations_total{{service=\"{}\"}} {}\n",
            escape_label(self.service()),
            marketing_recovery_metrics.expired_reservations_total
        ));

        output.push_str(
            "# HELP sdkwork_marketing_released_codes_total Total coupon codes released by recovery jobs\n",
        );
        output.push_str("# TYPE sdkwork_marketing_released_codes_total counter\n");
        output.push_str(&format!(
            "sdkwork_marketing_released_codes_total{{service=\"{}\"}} {}\n",
            escape_label(self.service()),
            marketing_recovery_metrics.released_codes_total
        ));

        output.push_str(
            "# HELP sdkwork_marketing_released_budget_minor_total Total marketing budget minor units released by recovery jobs\n",
        );
        output.push_str("# TYPE sdkwork_marketing_released_budget_minor_total counter\n");
        output.push_str(&format!(
            "sdkwork_marketing_released_budget_minor_total{{service=\"{}\"}} {}\n",
            escape_label(self.service()),
            marketing_recovery_metrics.released_budget_minor_total
        ));

        output.push_str(
            "# HELP sdkwork_marketing_recovery_outbox_events_total Total marketing recovery outbox events emitted\n",
        );
        output.push_str("# TYPE sdkwork_marketing_recovery_outbox_events_total counter\n");
        output.push_str(&format!(
            "sdkwork_marketing_recovery_outbox_events_total{{service=\"{}\"}} {}\n",
            escape_label(self.service()),
            marketing_recovery_metrics.outbox_events_total
        ));

        output.push_str(
            "# HELP sdkwork_marketing_recovery_last_success_at_ms Unix timestamp in milliseconds for the latest successful marketing recovery run\n",
        );
        output.push_str("# TYPE sdkwork_marketing_recovery_last_success_at_ms gauge\n");
        output.push_str(&format!(
            "sdkwork_marketing_recovery_last_success_at_ms{{service=\"{}\"}} {}\n",
            escape_label(self.service()),
            marketing_recovery_metrics.last_success_at_ms
        ));

        output.push_str(
            "# HELP sdkwork_marketing_recovery_last_failure_at_ms Unix timestamp in milliseconds for the latest failed marketing recovery run\n",
        );
        output.push_str("# TYPE sdkwork_marketing_recovery_last_failure_at_ms gauge\n");
        output.push_str(&format!(
            "sdkwork_marketing_recovery_last_failure_at_ms{{service=\"{}\"}} {}\n",
            escape_label(self.service()),
            marketing_recovery_metrics.last_failure_at_ms
        ));

        output
    }
}

fn shared_service_metrics(
    service: &str,
    limits: TelemetryCardinalityLimits,
) -> Arc<ServiceMetricsState> {
    let registry = SHARED_SERVICE_METRICS.get_or_init(|| Mutex::new(BTreeMap::new()));
    let mut registry = match registry.lock() {
        Ok(registry) => registry,
        Err(poisoned) => poisoned.into_inner(),
    };
    registry
        .entry(service.to_owned())
        .or_insert_with(|| Arc::new(ServiceMetricsState::new(limits)))
        .clone()
}

pub async fn with_current_http_metrics_registry<F, T>(
    registry: Arc<HttpMetricsRegistry>,
    future: F,
) -> T
where
    F: std::future::Future<Output = T>,
{
    commercial::with_current_http_metrics_service(registry.service(), future).await
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
    let response = next.run(request).await;
    let duration_ms = started_at.elapsed().as_millis() as u64;
    let status = response.status().as_u16();
    registry.record_with_dimensions(
        &method,
        &route,
        status,
        duration_ms,
        commercial::current_http_metric_dimensions_for(registry.service()).unwrap_or_default(),
    );
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
