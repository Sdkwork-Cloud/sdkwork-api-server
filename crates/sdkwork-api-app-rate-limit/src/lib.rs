use anyhow::{ensure, Result};
use async_trait::async_trait;
use sdkwork_api_domain_rate_limit::RateLimitCheckResult;
use sdkwork_api_storage_core::AdminStore;
use std::borrow::ToOwned;
use std::collections::HashMap;
use std::future::Future;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

pub use sdkwork_api_domain_rate_limit::{
    CommercialAdmissionPolicy, CommercialPressureScopeKind, RateLimitCheckResult as RateLimitCheck,
    RateLimitPolicy, TrafficPressureSnapshot,
};

pub fn service_name() -> &'static str {
    "rate-limit-service"
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GatewayTrafficRequestContext {
    pub tenant_id: String,
    pub project_id: String,
    pub api_key_hash: String,
    pub api_key_group_id: Option<String>,
    pub route_key: String,
    pub model_name: Option<String>,
}

impl GatewayTrafficRequestContext {
    pub fn new(
        tenant_id: impl Into<String>,
        project_id: impl Into<String>,
        api_key_hash: impl Into<String>,
        route_key: impl Into<String>,
    ) -> Self {
        Self {
            tenant_id: tenant_id.into(),
            project_id: project_id.into(),
            api_key_hash: api_key_hash.into(),
            api_key_group_id: None,
            route_key: route_key.into(),
            model_name: None,
        }
    }

    pub fn with_api_key_group_id_option(mut self, api_key_group_id: Option<String>) -> Self {
        self.api_key_group_id = api_key_group_id;
        self
    }

    pub fn with_model_name_option(mut self, model_name: Option<String>) -> Self {
        self.model_name = model_name;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GatewayTrafficAdmissionError {
    ProjectConcurrencyExceeded {
        policy_id: String,
        project_id: String,
        current_in_flight: u64,
        limit: u64,
    },
    ApiKeyConcurrencyExceeded {
        policy_id: String,
        project_id: String,
        api_key_hash: String,
        current_in_flight: u64,
        limit: u64,
    },
    ProviderBackpressureExceeded {
        policy_id: String,
        project_id: String,
        provider_id: String,
        current_in_flight: u64,
        limit: u64,
    },
}

impl GatewayTrafficAdmissionError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::ProjectConcurrencyExceeded { .. } | Self::ApiKeyConcurrencyExceeded { .. } => {
                "gateway_concurrency_exceeded"
            }
            Self::ProviderBackpressureExceeded { .. } => "provider_backpressure_exceeded",
        }
    }

    pub fn message(&self) -> String {
        match self {
            Self::ProjectConcurrencyExceeded {
                project_id,
                current_in_flight,
                limit,
                ..
            } => format!(
                "Project {project_id} has {current_in_flight} in-flight gateway requests against a concurrency limit of {limit}."
            ),
            Self::ApiKeyConcurrencyExceeded {
                project_id,
                current_in_flight,
                limit,
                ..
            } => format!(
                "An API key in project {project_id} has {current_in_flight} in-flight gateway requests against a concurrency limit of {limit}."
            ),
            Self::ProviderBackpressureExceeded {
                provider_id,
                current_in_flight,
                limit,
                ..
            } => format!(
                "Provider {provider_id} has {current_in_flight} in-flight executions against a backpressure limit of {limit}."
            ),
        }
    }
}

impl std::fmt::Display for GatewayTrafficAdmissionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message())
    }
}

impl std::error::Error for GatewayTrafficAdmissionError {}

pub trait GatewayTrafficPermitLease: Send + Sync + std::fmt::Debug {}

pub type GatewayTrafficAdmissionPermit = Arc<dyn GatewayTrafficPermitLease>;

#[async_trait]
pub trait GatewayTrafficController: Send + Sync {
    async fn acquire_request_admission(
        &self,
        context: &GatewayTrafficRequestContext,
    ) -> std::result::Result<GatewayTrafficAdmissionPermit, GatewayTrafficAdmissionError>;

    async fn acquire_provider_admission(
        &self,
        context: &GatewayTrafficRequestContext,
        provider_id: &str,
    ) -> std::result::Result<GatewayTrafficAdmissionPermit, GatewayTrafficAdmissionError>;

    fn list_pressure_snapshots(&self) -> Vec<TrafficPressureSnapshot>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CounterScope {
    Project,
    ApiKey,
    Provider,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CounterRelease {
    scope: CounterScope,
    policy_id: String,
}

#[derive(Debug, Default)]
struct InMemoryGatewayTrafficState {
    policies: Vec<CommercialAdmissionPolicy>,
    project_in_flight: HashMap<String, u64>,
    api_key_in_flight: HashMap<String, u64>,
    provider_in_flight: HashMap<String, u64>,
}

#[derive(Default)]
pub struct InMemoryGatewayTrafficController {
    state: Arc<Mutex<InMemoryGatewayTrafficState>>,
}

impl InMemoryGatewayTrafficController {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn replace_policies(&self, policies: Vec<CommercialAdmissionPolicy>) {
        let mut state = self.state.lock().expect("traffic controller lock poisoned");
        state.policies = policies;
        state.project_in_flight.clear();
        state.api_key_in_flight.clear();
        state.provider_in_flight.clear();
    }

    fn acquire_request_admission_sync(
        &self,
        context: &GatewayTrafficRequestContext,
    ) -> std::result::Result<GatewayTrafficAdmissionPermit, GatewayTrafficAdmissionError> {
        let mut state = self.state.lock().expect("traffic controller lock poisoned");
        let project_policy = select_project_concurrency_policy(&state.policies, context).cloned();
        let api_key_policy = select_api_key_concurrency_policy(&state.policies, context).cloned();

        if let Some(policy) = project_policy.as_ref() {
            let limit = policy.project_concurrency_limit.unwrap_or_default();
            let current = state
                .project_in_flight
                .get(&policy.policy_id)
                .copied()
                .unwrap_or_default();
            if current >= limit {
                return Err(GatewayTrafficAdmissionError::ProjectConcurrencyExceeded {
                    policy_id: policy.policy_id.clone(),
                    project_id: context.project_id.clone(),
                    current_in_flight: current,
                    limit,
                });
            }
        }

        if let Some(policy) = api_key_policy.as_ref() {
            let limit = policy.api_key_concurrency_limit.unwrap_or_default();
            let current = state
                .api_key_in_flight
                .get(&policy.policy_id)
                .copied()
                .unwrap_or_default();
            if current >= limit {
                return Err(GatewayTrafficAdmissionError::ApiKeyConcurrencyExceeded {
                    policy_id: policy.policy_id.clone(),
                    project_id: context.project_id.clone(),
                    api_key_hash: context.api_key_hash.clone(),
                    current_in_flight: current,
                    limit,
                });
            }
        }

        let mut releases = Vec::new();

        if let Some(policy) = project_policy.as_ref() {
            *state
                .project_in_flight
                .entry(policy.policy_id.clone())
                .or_default() += 1;
            releases.push(CounterRelease {
                scope: CounterScope::Project,
                policy_id: policy.policy_id.clone(),
            });
        }

        if let Some(policy) = api_key_policy.as_ref() {
            *state
                .api_key_in_flight
                .entry(policy.policy_id.clone())
                .or_default() += 1;
            releases.push(CounterRelease {
                scope: CounterScope::ApiKey,
                policy_id: policy.policy_id.clone(),
            });
        }

        Ok(Arc::new(InMemoryGatewayTrafficPermit {
            state: Arc::clone(&self.state),
            releases,
        }))
    }

    fn acquire_provider_admission_sync(
        &self,
        context: &GatewayTrafficRequestContext,
        provider_id: &str,
    ) -> std::result::Result<GatewayTrafficAdmissionPermit, GatewayTrafficAdmissionError> {
        let mut state = self.state.lock().expect("traffic controller lock poisoned");
        let policy =
            select_provider_concurrency_policy(&state.policies, context, provider_id).cloned();

        let Some(policy) = policy else {
            return Ok(Arc::new(InMemoryGatewayTrafficPermit {
                state: Arc::clone(&self.state),
                releases: Vec::new(),
            }));
        };

        let limit = policy.provider_concurrency_limit.unwrap_or_default();
        let current = state
            .provider_in_flight
            .get(&policy.policy_id)
            .copied()
            .unwrap_or_default();
        if current >= limit {
            return Err(GatewayTrafficAdmissionError::ProviderBackpressureExceeded {
                policy_id: policy.policy_id.clone(),
                project_id: context.project_id.clone(),
                provider_id: provider_id.to_owned(),
                current_in_flight: current,
                limit,
            });
        }

        *state
            .provider_in_flight
            .entry(policy.policy_id.clone())
            .or_default() += 1;

        Ok(Arc::new(InMemoryGatewayTrafficPermit {
            state: Arc::clone(&self.state),
            releases: vec![CounterRelease {
                scope: CounterScope::Provider,
                policy_id: policy.policy_id.clone(),
            }],
        }))
    }
}

#[derive(Debug)]
struct InMemoryGatewayTrafficPermit {
    state: Arc<Mutex<InMemoryGatewayTrafficState>>,
    releases: Vec<CounterRelease>,
}

impl GatewayTrafficPermitLease for InMemoryGatewayTrafficPermit {}

impl Drop for InMemoryGatewayTrafficPermit {
    fn drop(&mut self) {
        let mut state = self.state.lock().expect("traffic controller lock poisoned");
        for release in self.releases.drain(..) {
            let counter_map = match release.scope {
                CounterScope::Project => &mut state.project_in_flight,
                CounterScope::ApiKey => &mut state.api_key_in_flight,
                CounterScope::Provider => &mut state.provider_in_flight,
            };

            if let Some(entry) = counter_map.get_mut(&release.policy_id) {
                if *entry > 1 {
                    *entry -= 1;
                } else {
                    counter_map.remove(&release.policy_id);
                }
            }
        }
    }
}

#[async_trait]
impl GatewayTrafficController for InMemoryGatewayTrafficController {
    async fn acquire_request_admission(
        &self,
        context: &GatewayTrafficRequestContext,
    ) -> std::result::Result<GatewayTrafficAdmissionPermit, GatewayTrafficAdmissionError> {
        self.acquire_request_admission_sync(context)
    }

    async fn acquire_provider_admission(
        &self,
        context: &GatewayTrafficRequestContext,
        provider_id: &str,
    ) -> std::result::Result<GatewayTrafficAdmissionPermit, GatewayTrafficAdmissionError> {
        self.acquire_provider_admission_sync(context, provider_id)
    }

    fn list_pressure_snapshots(&self) -> Vec<TrafficPressureSnapshot> {
        let state = self.state.lock().expect("traffic controller lock poisoned");
        let now_ms = now_epoch_millis();
        let mut snapshots = Vec::new();

        for policy in &state.policies {
            if !policy.enabled {
                continue;
            }

            if let Some(limit) = policy.project_concurrency_limit {
                let current = state
                    .project_in_flight
                    .get(&policy.policy_id)
                    .copied()
                    .unwrap_or_default();
                snapshots.push(traffic_pressure_snapshot(
                    policy,
                    CommercialPressureScopeKind::Project,
                    policy.project_id.clone(),
                    current,
                    limit,
                    now_ms,
                ));
            }

            if let Some(limit) = policy.api_key_concurrency_limit {
                let current = state
                    .api_key_in_flight
                    .get(&policy.policy_id)
                    .copied()
                    .unwrap_or_default();
                snapshots.push(traffic_pressure_snapshot(
                    policy,
                    CommercialPressureScopeKind::ApiKey,
                    policy
                        .api_key_hash
                        .clone()
                        .unwrap_or_else(|| policy.policy_id.clone()),
                    current,
                    limit,
                    now_ms,
                ));
            }

            if let Some(limit) = policy.provider_concurrency_limit {
                let current = state
                    .provider_in_flight
                    .get(&policy.policy_id)
                    .copied()
                    .unwrap_or_default();
                snapshots.push(traffic_pressure_snapshot(
                    policy,
                    CommercialPressureScopeKind::Provider,
                    policy
                        .provider_id
                        .clone()
                        .unwrap_or_else(|| policy.policy_id.clone()),
                    current,
                    limit,
                    now_ms,
                ));
            }
        }

        snapshots
    }
}

fn traffic_pressure_snapshot(
    policy: &CommercialAdmissionPolicy,
    scope_kind: CommercialPressureScopeKind,
    scope_key: String,
    current_in_flight: u64,
    limit: u64,
    now_ms: u64,
) -> TrafficPressureSnapshot {
    TrafficPressureSnapshot {
        policy_id: policy.policy_id.clone(),
        project_id: policy.project_id.clone(),
        scope_kind,
        scope_key,
        api_key_hash: policy.api_key_hash.clone(),
        api_key_group_id: policy.api_key_group_id.clone(),
        route_key: policy.route_key.clone(),
        model_name: policy.model_name.clone(),
        provider_id: policy.provider_id.clone(),
        current_in_flight,
        limit,
        remaining: limit.saturating_sub(current_in_flight),
        saturated: current_in_flight >= limit,
        updated_at_ms: now_ms,
    }
}

fn select_project_concurrency_policy<'a>(
    policies: &'a [CommercialAdmissionPolicy],
    context: &GatewayTrafficRequestContext,
) -> Option<&'a CommercialAdmissionPolicy> {
    policies
        .iter()
        .filter(|policy| policy.project_concurrency_limit.is_some())
        .filter(|policy| {
            policy.matches_request_scope(
                &context.project_id,
                &context.api_key_hash,
                context.api_key_group_id.as_deref(),
                &context.route_key,
                context.model_name.as_deref(),
            )
        })
        .min_by(|left, right| {
            left.project_concurrency_limit
                .unwrap_or(u64::MAX)
                .cmp(&right.project_concurrency_limit.unwrap_or(u64::MAX))
                .then_with(|| {
                    left.request_specificity_score()
                        .cmp(&right.request_specificity_score())
                        .reverse()
                })
                .then_with(|| left.policy_id.cmp(&right.policy_id))
        })
}

fn select_api_key_concurrency_policy<'a>(
    policies: &'a [CommercialAdmissionPolicy],
    context: &GatewayTrafficRequestContext,
) -> Option<&'a CommercialAdmissionPolicy> {
    policies
        .iter()
        .filter(|policy| policy.api_key_concurrency_limit.is_some())
        .filter(|policy| {
            policy.matches_request_scope(
                &context.project_id,
                &context.api_key_hash,
                context.api_key_group_id.as_deref(),
                &context.route_key,
                context.model_name.as_deref(),
            )
        })
        .min_by(|left, right| {
            left.api_key_concurrency_limit
                .unwrap_or(u64::MAX)
                .cmp(&right.api_key_concurrency_limit.unwrap_or(u64::MAX))
                .then_with(|| {
                    left.request_specificity_score()
                        .cmp(&right.request_specificity_score())
                        .reverse()
                })
                .then_with(|| left.policy_id.cmp(&right.policy_id))
        })
}

fn select_provider_concurrency_policy<'a>(
    policies: &'a [CommercialAdmissionPolicy],
    context: &GatewayTrafficRequestContext,
    provider_id: &str,
) -> Option<&'a CommercialAdmissionPolicy> {
    policies
        .iter()
        .filter(|policy| policy.provider_concurrency_limit.is_some())
        .filter(|policy| {
            policy.matches_provider_scope(
                &context.project_id,
                &context.api_key_hash,
                context.api_key_group_id.as_deref(),
                &context.route_key,
                context.model_name.as_deref(),
                provider_id,
            )
        })
        .min_by(|left, right| {
            left.provider_concurrency_limit
                .unwrap_or(u64::MAX)
                .cmp(&right.provider_concurrency_limit.unwrap_or(u64::MAX))
                .then_with(|| {
                    left.provider_specificity_score()
                        .cmp(&right.provider_specificity_score())
                        .reverse()
                })
                .then_with(|| left.policy_id.cmp(&right.policy_id))
        })
}

tokio::task_local! {
    static CURRENT_GATEWAY_TRAFFIC_CONTROLLER: Arc<dyn GatewayTrafficController>;
}

tokio::task_local! {
    static CURRENT_GATEWAY_TRAFFIC_CONTEXT: GatewayTrafficRequestContext;
}

pub async fn with_gateway_traffic_context<F, T>(
    controller: Arc<dyn GatewayTrafficController>,
    context: GatewayTrafficRequestContext,
    future: F,
) -> T
where
    F: Future<Output = T>,
{
    CURRENT_GATEWAY_TRAFFIC_CONTROLLER
        .scope(
            controller,
            CURRENT_GATEWAY_TRAFFIC_CONTEXT.scope(context, future),
        )
        .await
}

pub fn current_gateway_traffic_context() -> Option<GatewayTrafficRequestContext> {
    CURRENT_GATEWAY_TRAFFIC_CONTEXT.try_with(Clone::clone).ok()
}

pub fn current_gateway_traffic_controller() -> Option<Arc<dyn GatewayTrafficController>> {
    CURRENT_GATEWAY_TRAFFIC_CONTROLLER.try_with(Arc::clone).ok()
}

pub async fn acquire_provider_admission_for_current_request(
    provider_id: &str,
) -> std::result::Result<Option<GatewayTrafficAdmissionPermit>, GatewayTrafficAdmissionError> {
    let Some(controller) = current_gateway_traffic_controller() else {
        return Ok(None);
    };
    let Some(context) = current_gateway_traffic_context() else {
        return Ok(None);
    };

    controller
        .acquire_provider_admission(&context, provider_id)
        .await
        .map(Some)
}

pub fn create_rate_limit_policy(
    policy_id: &str,
    project_id: &str,
    requests_per_window: u64,
    window_seconds: u64,
    burst_requests: u64,
    enabled: bool,
    route_key: Option<&str>,
    api_key_hash: Option<&str>,
    model_name: Option<&str>,
    notes: Option<&str>,
) -> Result<RateLimitPolicy> {
    ensure!(!policy_id.trim().is_empty(), "policy_id must not be empty");
    ensure!(
        !project_id.trim().is_empty(),
        "project_id must not be empty"
    );
    ensure!(
        requests_per_window > 0,
        "requests_per_window must be greater than 0"
    );
    ensure!(window_seconds > 0, "window_seconds must be greater than 0");

    Ok(
        RateLimitPolicy::new(policy_id, project_id, requests_per_window, window_seconds)
            .with_burst_requests(burst_requests)
            .with_enabled(enabled)
            .with_route_key_option(route_key.map(ToOwned::to_owned))
            .with_api_key_hash_option(api_key_hash.map(ToOwned::to_owned))
            .with_model_name_option(model_name.map(ToOwned::to_owned))
            .with_notes_option(notes.map(ToOwned::to_owned)),
    )
}

pub async fn persist_rate_limit_policy(
    store: &dyn AdminStore,
    policy: &RateLimitPolicy,
) -> Result<RateLimitPolicy> {
    store.insert_rate_limit_policy(policy).await
}

pub async fn list_rate_limit_policies(store: &dyn AdminStore) -> Result<Vec<RateLimitPolicy>> {
    store.list_rate_limit_policies().await
}

#[async_trait]
pub trait RateLimitPolicyStore: Send + Sync {
    async fn list_rate_limit_policies_for_project(
        &self,
        project_id: &str,
    ) -> Result<Vec<RateLimitPolicy>>;
    async fn check_and_consume_rate_limit(
        &self,
        policy_id: &str,
        requested_requests: u64,
        limit_requests: u64,
        window_seconds: u64,
        now_ms: u64,
    ) -> Result<RateLimitCheckResult>;
}

#[async_trait]
impl<T> RateLimitPolicyStore for T
where
    T: AdminStore + ?Sized,
{
    async fn list_rate_limit_policies_for_project(
        &self,
        project_id: &str,
    ) -> Result<Vec<RateLimitPolicy>> {
        AdminStore::list_rate_limit_policies_for_project(self, project_id).await
    }

    async fn check_and_consume_rate_limit(
        &self,
        policy_id: &str,
        requested_requests: u64,
        limit_requests: u64,
        window_seconds: u64,
        now_ms: u64,
    ) -> Result<RateLimitCheckResult> {
        AdminStore::check_and_consume_rate_limit(
            self,
            policy_id,
            requested_requests,
            limit_requests,
            window_seconds,
            now_ms,
        )
        .await
    }
}

pub async fn check_rate_limit<S>(
    store: &S,
    project_id: &str,
    api_key_hash: Option<&str>,
    route_key: &str,
    model_name: Option<&str>,
    requested_requests: u64,
) -> Result<RateLimitCheckResult>
where
    S: RateLimitPolicyStore + ?Sized,
{
    let effective_policy = store
        .list_rate_limit_policies_for_project(project_id)
        .await?
        .into_iter()
        .filter(|policy| policy.matches(project_id, api_key_hash, route_key, model_name))
        .min_by(|left, right| {
            left.effective_limit_requests()
                .cmp(&right.effective_limit_requests())
                .then_with(|| {
                    left.specificity_score()
                        .cmp(&right.specificity_score())
                        .reverse()
                })
                .then_with(|| left.policy_id.cmp(&right.policy_id))
        });

    let Some(policy) = effective_policy else {
        return Ok(RateLimitCheckResult::allowed_without_policy(
            requested_requests,
            0,
        ));
    };

    let now_ms = now_epoch_millis();
    let result = store
        .check_and_consume_rate_limit(
            &policy.policy_id,
            requested_requests,
            policy.effective_limit_requests(),
            policy.window_seconds,
            now_ms,
        )
        .await?;

    Ok(result)
}

fn now_epoch_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}
