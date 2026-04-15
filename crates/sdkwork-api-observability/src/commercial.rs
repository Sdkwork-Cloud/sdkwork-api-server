use std::collections::{BTreeMap, BTreeSet};
use std::future::Future;
use std::sync::{Arc, Mutex, MutexGuard};

const HISTOGRAM_BUCKETS_MS: [u64; 10] = [5, 10, 25, 50, 100, 250, 500, 1000, 2500, 5000];

tokio::task_local! {
    static CURRENT_HTTP_METRICS_CONTEXT: Arc<CurrentHttpMetricsContext>;
}

#[derive(Debug, Clone, Default)]
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
    pub fn with_tenant(mut self, tenant: impl Into<String>) -> Self {
        self.tenant = Some(tenant.into());
        self
    }

    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    pub fn with_provider(mut self, provider: impl Into<String>) -> Self {
        self.provider = Some(provider.into());
        self
    }

    pub fn with_billing_mode(mut self, billing_mode: impl Into<String>) -> Self {
        self.billing_mode = Some(billing_mode.into());
        self
    }

    pub fn with_retry_outcome(mut self, retry_outcome: impl Into<String>) -> Self {
        self.retry_outcome = Some(retry_outcome.into());
        self
    }

    pub fn with_failover_outcome(mut self, failover_outcome: impl Into<String>) -> Self {
        self.failover_outcome = Some(failover_outcome.into());
        self
    }

    pub fn with_payment_outcome(mut self, payment_outcome: impl Into<String>) -> Self {
        self.payment_outcome = Some(payment_outcome.into());
        self
    }
}

#[derive(Debug, Clone, Default)]
pub struct ProviderExecutionMetricDimensions {
    route: Option<String>,
    tenant: Option<String>,
    model: Option<String>,
    provider: Option<String>,
    billing_mode: Option<String>,
    retry_outcome: Option<String>,
    failover_outcome: Option<String>,
    result: Option<String>,
}

impl ProviderExecutionMetricDimensions {
    pub fn with_route(mut self, route: impl Into<String>) -> Self {
        self.route = Some(route.into());
        self
    }

    pub fn with_tenant(mut self, tenant: impl Into<String>) -> Self {
        self.tenant = Some(tenant.into());
        self
    }

    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    pub fn with_provider(mut self, provider: impl Into<String>) -> Self {
        self.provider = Some(provider.into());
        self
    }

    pub fn with_billing_mode(mut self, billing_mode: impl Into<String>) -> Self {
        self.billing_mode = Some(billing_mode.into());
        self
    }

    pub fn with_retry_outcome(mut self, retry_outcome: impl Into<String>) -> Self {
        self.retry_outcome = Some(retry_outcome.into());
        self
    }

    pub fn with_failover_outcome(mut self, failover_outcome: impl Into<String>) -> Self {
        self.failover_outcome = Some(failover_outcome.into());
        self
    }

    pub fn with_result(mut self, result: impl Into<String>) -> Self {
        self.result = Some(result.into());
        self
    }
}

#[derive(Debug, Clone, Default)]
pub struct PaymentMetricDimensions {
    provider: Option<String>,
    tenant: Option<String>,
    payment_outcome: Option<String>,
}

impl PaymentMetricDimensions {
    pub fn with_provider(mut self, provider: impl Into<String>) -> Self {
        self.provider = Some(provider.into());
        self
    }

    pub fn with_tenant(mut self, tenant: impl Into<String>) -> Self {
        self.tenant = Some(tenant.into());
        self
    }

    pub fn with_payment_outcome(mut self, payment_outcome: impl Into<String>) -> Self {
        self.payment_outcome = Some(payment_outcome.into());
        self
    }
}

#[derive(Debug, Clone, Default)]
pub struct CommercialEventDimensions {
    route: Option<String>,
    tenant: Option<String>,
    provider: Option<String>,
    model: Option<String>,
    payment_outcome: Option<String>,
    result: Option<String>,
}

impl CommercialEventDimensions {
    pub fn with_route(mut self, route: impl Into<String>) -> Self {
        self.route = Some(route.into());
        self
    }

    pub fn with_tenant(mut self, tenant: impl Into<String>) -> Self {
        self.tenant = Some(tenant.into());
        self
    }

    pub fn with_provider(mut self, provider: impl Into<String>) -> Self {
        self.provider = Some(provider.into());
        self
    }

    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    pub fn with_payment_outcome(mut self, payment_outcome: impl Into<String>) -> Self {
        self.payment_outcome = Some(payment_outcome.into());
        self
    }

    pub fn with_result(mut self, result: impl Into<String>) -> Self {
        self.result = Some(result.into());
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CommercialEventKind {
    CallbackReplay,
    FailoverActivation,
}

impl CommercialEventKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::CallbackReplay => "callback_replay",
            Self::FailoverActivation => "failover_activation",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TelemetryCardinalityLimits {
    tenant_limit: usize,
    model_limit: usize,
    provider_limit: usize,
}

impl Default for TelemetryCardinalityLimits {
    fn default() -> Self {
        Self {
            tenant_limit: usize::MAX,
            model_limit: usize::MAX,
            provider_limit: usize::MAX,
        }
    }
}

impl TelemetryCardinalityLimits {
    pub fn with_tenant_limit(mut self, tenant_limit: usize) -> Self {
        self.tenant_limit = tenant_limit;
        self
    }

    pub fn with_model_limit(mut self, model_limit: usize) -> Self {
        self.model_limit = model_limit;
        self
    }

    pub fn with_provider_limit(mut self, provider_limit: usize) -> Self {
        self.provider_limit = provider_limit;
        self
    }
}

#[derive(Debug)]
pub(crate) struct CommercialMetricsState {
    limits: TelemetryCardinalityLimits,
    cardinality: Mutex<CardinalityState>,
    http_metrics: Mutex<BTreeMap<HttpMetricKey, HistogramMetricValue>>,
    provider_execution_metrics: Mutex<BTreeMap<ProviderExecutionMetricKey, HistogramMetricValue>>,
    payment_callback_metrics: Mutex<BTreeMap<PaymentMetricKey, u64>>,
    commercial_event_metrics: Mutex<BTreeMap<CommercialEventMetricKey, u64>>,
}

impl CommercialMetricsState {
    pub(crate) fn new(limits: TelemetryCardinalityLimits) -> Self {
        Self {
            limits,
            cardinality: Mutex::new(CardinalityState::default()),
            http_metrics: Mutex::new(BTreeMap::new()),
            provider_execution_metrics: Mutex::new(BTreeMap::new()),
            payment_callback_metrics: Mutex::new(BTreeMap::new()),
            commercial_event_metrics: Mutex::new(BTreeMap::new()),
        }
    }

    pub(crate) fn record_http(
        &self,
        method: &str,
        route: &str,
        status: u16,
        duration_ms: u64,
        dimensions: HttpMetricDimensions,
    ) {
        let labels = self.http_labels(&dimensions);
        let key = HttpMetricKey {
            method: method.to_owned(),
            route: route.to_owned(),
            status,
            labels,
        };
        let mut metrics = lock_or_recover(&self.http_metrics);
        metrics.entry(key).or_default().record(duration_ms);
    }

    pub(crate) fn record_provider_execution(
        &self,
        duration_ms: u64,
        dimensions: ProviderExecutionMetricDimensions,
    ) {
        let key = self.provider_execution_key(&dimensions);
        let mut metrics = lock_or_recover(&self.provider_execution_metrics);
        metrics.entry(key).or_default().record(duration_ms);
    }

    pub(crate) fn record_payment_callback(&self, dimensions: PaymentMetricDimensions) {
        let key = self.payment_key(&dimensions);
        let mut metrics = lock_or_recover(&self.payment_callback_metrics);
        *metrics.entry(key).or_default() += 1;
    }

    pub(crate) fn record_commercial_event(
        &self,
        event_kind: CommercialEventKind,
        dimensions: CommercialEventDimensions,
    ) {
        let key = self.commercial_event_key(event_kind, &dimensions);
        let mut metrics = lock_or_recover(&self.commercial_event_metrics);
        *metrics.entry(key).or_default() += 1;
    }

    pub(crate) fn record_failover_success(
        &self,
        service: &str,
        from_provider: &str,
        to_provider: &str,
    ) {
        let Some(current) = current_http_metric_dimensions_for(service) else {
            return;
        };
        let route = normalize_optional_dimension(current.route.as_deref());
        let tenant = normalize_optional_dimension(current.tenant.as_deref());
        let model = normalize_optional_dimension(current.model.as_deref());
        let billing_mode = normalize_optional_dimension(current.billing_mode.as_deref());
        self.record_provider_execution(
            0,
            ProviderExecutionMetricDimensions::default()
                .with_route(route.clone())
                .with_tenant(tenant.clone())
                .with_model(model.clone())
                .with_provider(from_provider)
                .with_billing_mode(billing_mode.clone())
                .with_retry_outcome("will_failover")
                .with_failover_outcome("activated")
                .with_result("retryable_failure"),
        );
        self.record_provider_execution(
            0,
            ProviderExecutionMetricDimensions::default()
                .with_route(route.clone())
                .with_tenant(tenant.clone())
                .with_model(model.clone())
                .with_provider(to_provider)
                .with_billing_mode(billing_mode)
                .with_retry_outcome("none")
                .with_failover_outcome("activated")
                .with_result("succeeded"),
        );
        self.record_commercial_event(
            CommercialEventKind::FailoverActivation,
            CommercialEventDimensions::default()
                .with_route(route)
                .with_tenant(tenant)
                .with_provider(from_provider)
                .with_model(model)
                .with_payment_outcome("none")
                .with_result("activated"),
        );
    }

    pub(crate) fn render_prometheus(&self, service: &str, output: &mut String) {
        let http_metrics = lock_or_recover(&self.http_metrics).clone();
        let provider_execution_metrics = lock_or_recover(&self.provider_execution_metrics).clone();
        let payment_callback_metrics = lock_or_recover(&self.payment_callback_metrics).clone();
        let commercial_event_metrics = lock_or_recover(&self.commercial_event_metrics).clone();

        output.push_str("# HELP sdkwork_http_request_duration_ms_bucket HTTP request duration histogram buckets\n");
        output.push_str("# TYPE sdkwork_http_request_duration_ms_bucket counter\n");
        for (key, value) in &http_metrics {
            write_http_lines(service, key, value, output);
        }

        for (key, value) in &http_metrics {
            output.push_str(&format!(
                "sdkwork_http_requests_total{{service=\"{}\",method=\"{}\",route=\"{}\",status=\"{}\",tenant=\"{}\",model=\"{}\",provider=\"{}\",billing_mode=\"{}\",retry_outcome=\"{}\",failover_outcome=\"{}\",payment_outcome=\"{}\"}} {}\n",
                escape_label(service),
                escape_label(&key.method),
                escape_label(&key.route),
                key.status,
                escape_label(&key.labels.tenant),
                escape_label(&key.labels.model),
                escape_label(&key.labels.provider),
                escape_label(&key.labels.billing_mode),
                escape_label(&key.labels.retry_outcome),
                escape_label(&key.labels.failover_outcome),
                escape_label(&key.labels.payment_outcome),
                value.count
            ));
            output.push_str(&format!(
                "sdkwork_http_request_duration_ms_sum{{service=\"{}\",method=\"{}\",route=\"{}\",status=\"{}\",tenant=\"{}\",model=\"{}\",provider=\"{}\",billing_mode=\"{}\",retry_outcome=\"{}\",failover_outcome=\"{}\",payment_outcome=\"{}\"}} {}\n",
                escape_label(service),
                escape_label(&key.method),
                escape_label(&key.route),
                key.status,
                escape_label(&key.labels.tenant),
                escape_label(&key.labels.model),
                escape_label(&key.labels.provider),
                escape_label(&key.labels.billing_mode),
                escape_label(&key.labels.retry_outcome),
                escape_label(&key.labels.failover_outcome),
                escape_label(&key.labels.payment_outcome),
                value.duration_ms_sum
            ));
            output.push_str(&format!(
                "sdkwork_http_request_duration_ms_count{{service=\"{}\",method=\"{}\",route=\"{}\",status=\"{}\",tenant=\"{}\",model=\"{}\",provider=\"{}\",billing_mode=\"{}\",retry_outcome=\"{}\",failover_outcome=\"{}\",payment_outcome=\"{}\"}} {}\n",
                escape_label(service),
                escape_label(&key.method),
                escape_label(&key.route),
                key.status,
                escape_label(&key.labels.tenant),
                escape_label(&key.labels.model),
                escape_label(&key.labels.provider),
                escape_label(&key.labels.billing_mode),
                escape_label(&key.labels.retry_outcome),
                escape_label(&key.labels.failover_outcome),
                escape_label(&key.labels.payment_outcome),
                value.count
            ));
        }

        output.push_str(
            "# HELP sdkwork_provider_execution_total Total provider executions observed\n",
        );
        output.push_str("# TYPE sdkwork_provider_execution_total counter\n");
        output.push_str("# HELP sdkwork_provider_execution_duration_ms_bucket Provider execution duration histogram buckets\n");
        output.push_str("# TYPE sdkwork_provider_execution_duration_ms_bucket counter\n");
        output.push_str("# HELP sdkwork_provider_execution_duration_ms_sum Cumulative provider execution duration in milliseconds\n");
        output.push_str("# TYPE sdkwork_provider_execution_duration_ms_sum counter\n");
        output.push_str("# HELP sdkwork_provider_execution_duration_ms_count Provider execution count paired with duration summaries\n");
        output.push_str("# TYPE sdkwork_provider_execution_duration_ms_count counter\n");
        for (key, value) in &provider_execution_metrics {
            output.push_str(&format!(
                "sdkwork_provider_execution_total{{service=\"{}\",route=\"{}\",tenant=\"{}\",model=\"{}\",provider=\"{}\",billing_mode=\"{}\",retry_outcome=\"{}\",failover_outcome=\"{}\",result=\"{}\"}} {}\n",
                escape_label(service),
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
            for (index, bucket) in HISTOGRAM_BUCKETS_MS.iter().enumerate() {
                output.push_str(&format!(
                    "sdkwork_provider_execution_duration_ms_bucket{{service=\"{}\",route=\"{}\",tenant=\"{}\",model=\"{}\",provider=\"{}\",billing_mode=\"{}\",retry_outcome=\"{}\",failover_outcome=\"{}\",result=\"{}\",le=\"{}\"}} {}\n",
                    escape_label(service),
                    escape_label(&key.route),
                    escape_label(&key.tenant),
                    escape_label(&key.model),
                    escape_label(&key.provider),
                    escape_label(&key.billing_mode),
                    escape_label(&key.retry_outcome),
                    escape_label(&key.failover_outcome),
                    escape_label(&key.result),
                    bucket,
                    value.bucket_counts[index]
                ));
            }
            output.push_str(&format!(
                "sdkwork_provider_execution_duration_ms_bucket{{service=\"{}\",route=\"{}\",tenant=\"{}\",model=\"{}\",provider=\"{}\",billing_mode=\"{}\",retry_outcome=\"{}\",failover_outcome=\"{}\",result=\"{}\",le=\"+Inf\"}} {}\n",
                escape_label(service),
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
            output.push_str(&format!(
                "sdkwork_provider_execution_duration_ms_sum{{service=\"{}\",route=\"{}\",tenant=\"{}\",model=\"{}\",provider=\"{}\",billing_mode=\"{}\",retry_outcome=\"{}\",failover_outcome=\"{}\",result=\"{}\"}} {}\n",
                escape_label(service),
                escape_label(&key.route),
                escape_label(&key.tenant),
                escape_label(&key.model),
                escape_label(&key.provider),
                escape_label(&key.billing_mode),
                escape_label(&key.retry_outcome),
                escape_label(&key.failover_outcome),
                escape_label(&key.result),
                value.duration_ms_sum
            ));
            output.push_str(&format!(
                "sdkwork_provider_execution_duration_ms_count{{service=\"{}\",route=\"{}\",tenant=\"{}\",model=\"{}\",provider=\"{}\",billing_mode=\"{}\",retry_outcome=\"{}\",failover_outcome=\"{}\",result=\"{}\"}} {}\n",
                escape_label(service),
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

        output.push_str(
            "# HELP sdkwork_payment_callbacks_total Total payment callback outcomes observed\n",
        );
        output.push_str("# TYPE sdkwork_payment_callbacks_total counter\n");
        for (key, value) in &payment_callback_metrics {
            output.push_str(&format!(
                "sdkwork_payment_callbacks_total{{service=\"{}\",provider=\"{}\",tenant=\"{}\",payment_outcome=\"{}\"}} {}\n",
                escape_label(service),
                escape_label(&key.provider),
                escape_label(&key.tenant),
                escape_label(&key.payment_outcome),
                value
            ));
        }

        output.push_str("# HELP sdkwork_commercial_events_total Total commercial control-plane events observed\n");
        output.push_str("# TYPE sdkwork_commercial_events_total counter\n");
        for (key, value) in &commercial_event_metrics {
            output.push_str(&format!(
                "sdkwork_commercial_events_total{{service=\"{}\",event_kind=\"{}\",route=\"{}\",tenant=\"{}\",provider=\"{}\",model=\"{}\",payment_outcome=\"{}\",result=\"{}\"}} {}\n",
                escape_label(service),
                key.event_kind.as_str(),
                escape_label(&key.route),
                escape_label(&key.tenant),
                escape_label(&key.provider),
                escape_label(&key.model),
                escape_label(&key.payment_outcome),
                escape_label(&key.result),
                value
            ));
        }
    }

    fn http_labels(&self, dimensions: &HttpMetricDimensions) -> HttpMetricLabels {
        let mut state = lock_or_recover(&self.cardinality);
        HttpMetricLabels {
            tenant: canonicalize_limited_dimension(
                dimensions.tenant.as_deref(),
                &mut state.tenants,
                self.limits.tenant_limit,
            ),
            model: canonicalize_limited_dimension(
                dimensions.model.as_deref(),
                &mut state.models,
                self.limits.model_limit,
            ),
            provider: canonicalize_limited_dimension(
                dimensions.provider.as_deref(),
                &mut state.providers,
                self.limits.provider_limit,
            ),
            billing_mode: normalize_optional_dimension(dimensions.billing_mode.as_deref()),
            retry_outcome: normalize_optional_dimension(dimensions.retry_outcome.as_deref()),
            failover_outcome: normalize_optional_dimension(dimensions.failover_outcome.as_deref()),
            payment_outcome: normalize_optional_dimension(dimensions.payment_outcome.as_deref()),
        }
    }

    fn provider_execution_key(
        &self,
        dimensions: &ProviderExecutionMetricDimensions,
    ) -> ProviderExecutionMetricKey {
        let mut state = lock_or_recover(&self.cardinality);
        ProviderExecutionMetricKey {
            route: normalize_optional_dimension(dimensions.route.as_deref()),
            tenant: canonicalize_limited_dimension(
                dimensions.tenant.as_deref(),
                &mut state.tenants,
                self.limits.tenant_limit,
            ),
            model: canonicalize_limited_dimension(
                dimensions.model.as_deref(),
                &mut state.models,
                self.limits.model_limit,
            ),
            provider: canonicalize_limited_dimension(
                dimensions.provider.as_deref(),
                &mut state.providers,
                self.limits.provider_limit,
            ),
            billing_mode: normalize_optional_dimension(dimensions.billing_mode.as_deref()),
            retry_outcome: normalize_optional_dimension(dimensions.retry_outcome.as_deref()),
            failover_outcome: normalize_optional_dimension(dimensions.failover_outcome.as_deref()),
            result: normalize_optional_dimension(dimensions.result.as_deref()),
        }
    }

    fn payment_key(&self, dimensions: &PaymentMetricDimensions) -> PaymentMetricKey {
        let mut state = lock_or_recover(&self.cardinality);
        PaymentMetricKey {
            provider: canonicalize_limited_dimension(
                dimensions.provider.as_deref(),
                &mut state.providers,
                self.limits.provider_limit,
            ),
            tenant: canonicalize_limited_dimension(
                dimensions.tenant.as_deref(),
                &mut state.tenants,
                self.limits.tenant_limit,
            ),
            payment_outcome: normalize_optional_dimension(dimensions.payment_outcome.as_deref()),
        }
    }

    fn commercial_event_key(
        &self,
        event_kind: CommercialEventKind,
        dimensions: &CommercialEventDimensions,
    ) -> CommercialEventMetricKey {
        let mut state = lock_or_recover(&self.cardinality);
        CommercialEventMetricKey {
            event_kind,
            route: normalize_optional_dimension(dimensions.route.as_deref()),
            tenant: canonicalize_limited_dimension(
                dimensions.tenant.as_deref(),
                &mut state.tenants,
                self.limits.tenant_limit,
            ),
            provider: canonicalize_limited_dimension(
                dimensions.provider.as_deref(),
                &mut state.providers,
                self.limits.provider_limit,
            ),
            model: canonicalize_limited_dimension(
                dimensions.model.as_deref(),
                &mut state.models,
                self.limits.model_limit,
            ),
            payment_outcome: normalize_optional_dimension(dimensions.payment_outcome.as_deref()),
            result: normalize_optional_dimension(dimensions.result.as_deref()),
        }
    }
}

pub(crate) async fn with_current_http_metrics_service<F, T>(service: &str, future: F) -> T
where
    F: Future<Output = T>,
{
    let context = Arc::new(CurrentHttpMetricsContext {
        service: service.to_owned(),
        dimensions: Mutex::new(HttpMetricDimensions::default()),
    });
    CURRENT_HTTP_METRICS_CONTEXT.scope(context, future).await
}

pub(crate) fn current_http_metric_dimensions_for(service: &str) -> Option<HttpMetricDimensions> {
    CURRENT_HTTP_METRICS_CONTEXT
        .try_with(|context| {
            if context.service != service {
                return None;
            }
            Some(lock_or_recover(&context.dimensions).clone())
        })
        .ok()
        .flatten()
}

pub fn annotate_current_http_metrics(annotate: impl FnOnce(&mut HttpMetricDimensions)) {
    let _ = CURRENT_HTTP_METRICS_CONTEXT.try_with(|context| {
        let mut dimensions = lock_or_recover(&context.dimensions);
        annotate(&mut dimensions);
    });
}

#[derive(Debug)]
struct CurrentHttpMetricsContext {
    service: String,
    dimensions: Mutex<HttpMetricDimensions>,
}

#[derive(Debug, Clone, Default)]
struct HistogramMetricValue {
    count: u64,
    duration_ms_sum: u64,
    bucket_counts: [u64; HISTOGRAM_BUCKETS_MS.len()],
}

impl HistogramMetricValue {
    fn record(&mut self, duration_ms: u64) {
        self.count += 1;
        self.duration_ms_sum += duration_ms;
        for (index, bucket) in HISTOGRAM_BUCKETS_MS.iter().enumerate() {
            if duration_ms <= *bucket {
                self.bucket_counts[index] += 1;
            }
        }
    }
}

#[derive(Debug, Default)]
struct CardinalityState {
    tenants: BTreeSet<String>,
    models: BTreeSet<String>,
    providers: BTreeSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct HttpMetricLabels {
    tenant: String,
    model: String,
    provider: String,
    billing_mode: String,
    retry_outcome: String,
    failover_outcome: String,
    payment_outcome: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct HttpMetricKey {
    method: String,
    route: String,
    status: u16,
    labels: HttpMetricLabels,
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
struct CommercialEventMetricKey {
    event_kind: CommercialEventKind,
    route: String,
    tenant: String,
    provider: String,
    model: String,
    payment_outcome: String,
    result: String,
}

fn write_http_lines(
    service: &str,
    key: &HttpMetricKey,
    value: &HistogramMetricValue,
    output: &mut String,
) {
    for (index, bucket) in HISTOGRAM_BUCKETS_MS.iter().enumerate() {
        output.push_str(&format!(
            "sdkwork_http_request_duration_ms_bucket{{service=\"{}\",method=\"{}\",route=\"{}\",status=\"{}\",tenant=\"{}\",model=\"{}\",provider=\"{}\",billing_mode=\"{}\",retry_outcome=\"{}\",failover_outcome=\"{}\",payment_outcome=\"{}\",le=\"{}\"}} {}\n",
            escape_label(service),
            escape_label(&key.method),
            escape_label(&key.route),
            key.status,
            escape_label(&key.labels.tenant),
            escape_label(&key.labels.model),
            escape_label(&key.labels.provider),
            escape_label(&key.labels.billing_mode),
            escape_label(&key.labels.retry_outcome),
            escape_label(&key.labels.failover_outcome),
            escape_label(&key.labels.payment_outcome),
            bucket,
            value.bucket_counts[index]
        ));
    }
    output.push_str(&format!(
        "sdkwork_http_request_duration_ms_bucket{{service=\"{}\",method=\"{}\",route=\"{}\",status=\"{}\",tenant=\"{}\",model=\"{}\",provider=\"{}\",billing_mode=\"{}\",retry_outcome=\"{}\",failover_outcome=\"{}\",payment_outcome=\"{}\",le=\"+Inf\"}} {}\n",
        escape_label(service),
        escape_label(&key.method),
        escape_label(&key.route),
        key.status,
        escape_label(&key.labels.tenant),
        escape_label(&key.labels.model),
        escape_label(&key.labels.provider),
        escape_label(&key.labels.billing_mode),
        escape_label(&key.labels.retry_outcome),
        escape_label(&key.labels.failover_outcome),
        escape_label(&key.labels.payment_outcome),
        value.count
    ));
}

fn canonicalize_limited_dimension(
    value: Option<&str>,
    seen: &mut BTreeSet<String>,
    limit: usize,
) -> String {
    let normalized = normalize_optional_dimension(value);
    if normalized == "none" {
        return normalized;
    }
    if seen.contains(&normalized) {
        return normalized;
    }
    if limit == usize::MAX || seen.len() < limit {
        seen.insert(normalized.clone());
        return normalized;
    }
    "other".to_owned()
}

fn normalize_optional_dimension(value: Option<&str>) -> String {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("none")
        .to_owned()
}

fn lock_or_recover<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    match mutex.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

fn escape_label(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}
