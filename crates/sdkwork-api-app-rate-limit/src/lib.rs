use anyhow::{ensure, Result};
use async_trait::async_trait;
use sdkwork_api_domain_rate_limit::RateLimitCheckResult;
use sdkwork_api_storage_core::AdminStore;
use std::borrow::ToOwned;
use std::time::{SystemTime, UNIX_EPOCH};

pub use sdkwork_api_domain_rate_limit::{RateLimitCheckResult as RateLimitCheck, RateLimitPolicy};

pub fn service_name() -> &'static str {
    "rate-limit-service"
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
