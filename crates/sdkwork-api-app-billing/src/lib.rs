use anyhow::{ensure, Result};
use sdkwork_api_domain_billing::{BillingSummary, LedgerEntry, ProjectBillingSummary};
use sdkwork_api_storage_core::AdminStore;
use std::collections::BTreeMap;

pub use sdkwork_api_domain_billing::{QuotaCheckResult, QuotaPolicy};

pub fn service_name() -> &'static str {
    "billing-service"
}

pub fn create_quota_policy(
    policy_id: &str,
    project_id: &str,
    max_units: u64,
    enabled: bool,
) -> Result<QuotaPolicy> {
    ensure!(!policy_id.trim().is_empty(), "policy_id must not be empty");
    ensure!(
        !project_id.trim().is_empty(),
        "project_id must not be empty"
    );
    ensure!(max_units > 0, "max_units must be greater than 0");

    Ok(QuotaPolicy::new(policy_id, project_id, max_units).with_enabled(enabled))
}

pub fn book_usage_cost(project_id: &str, units: u64, amount: f64) -> Result<LedgerEntry> {
    Ok(LedgerEntry::new(project_id, units, amount))
}

pub async fn persist_ledger_entry(
    store: &dyn AdminStore,
    project_id: &str,
    units: u64,
    amount: f64,
) -> Result<LedgerEntry> {
    let entry = book_usage_cost(project_id, units, amount)?;
    store.insert_ledger_entry(&entry).await
}

pub async fn list_ledger_entries(store: &dyn AdminStore) -> Result<Vec<LedgerEntry>> {
    store.list_ledger_entries().await
}

pub async fn persist_quota_policy(
    store: &dyn AdminStore,
    policy: &QuotaPolicy,
) -> Result<QuotaPolicy> {
    store.insert_quota_policy(policy).await
}

pub async fn list_quota_policies(store: &dyn AdminStore) -> Result<Vec<QuotaPolicy>> {
    store.list_quota_policies().await
}

pub async fn check_quota(
    store: &dyn AdminStore,
    project_id: &str,
    requested_units: u64,
) -> Result<QuotaCheckResult> {
    let used_units = store
        .list_ledger_entries()
        .await?
        .into_iter()
        .filter(|entry| entry.project_id == project_id)
        .map(|entry| entry.units)
        .sum();

    let effective_policy = store
        .list_quota_policies()
        .await?
        .into_iter()
        .filter(|policy| policy.enabled && policy.project_id == project_id)
        .min_by(|left, right| {
            left.max_units
                .cmp(&right.max_units)
                .then_with(|| left.policy_id.cmp(&right.policy_id))
        });

    Ok(match effective_policy {
        Some(policy) => QuotaCheckResult::from_policy(&policy, used_units, requested_units),
        None => QuotaCheckResult::allowed_without_policy(requested_units, used_units),
    })
}

pub fn summarize_billing_snapshot(
    entries: &[LedgerEntry],
    policies: &[QuotaPolicy],
) -> BillingSummary {
    if entries.is_empty() && policies.is_empty() {
        return BillingSummary::empty();
    }

    let mut projects = BTreeMap::<String, ProjectBillingSummary>::new();

    for entry in entries {
        let summary = projects
            .entry(entry.project_id.clone())
            .or_insert_with(|| ProjectBillingSummary::new(entry.project_id.clone()));
        summary.entry_count += 1;
        summary.used_units += entry.units;
        summary.booked_amount += entry.amount;
    }

    let active_policies = policies
        .iter()
        .filter(|policy| policy.enabled)
        .collect::<Vec<_>>();

    for policy in &active_policies {
        let summary = projects
            .entry(policy.project_id.clone())
            .or_insert_with(|| ProjectBillingSummary::new(policy.project_id.clone()));
        let replace_policy = match (
            summary.quota_limit_units,
            summary.quota_policy_id.as_deref(),
        ) {
            (None, _) => true,
            (Some(current_limit), Some(current_policy_id)) => {
                policy.max_units < current_limit
                    || (policy.max_units == current_limit
                        && policy.policy_id.as_str() < current_policy_id)
            }
            (Some(_), None) => true,
        };

        if replace_policy {
            summary.quota_policy_id = Some(policy.policy_id.clone());
            summary.quota_limit_units = Some(policy.max_units);
        }
    }

    let total_entries = entries.len() as u64;
    let total_units = entries.iter().map(|entry| entry.units).sum();
    let total_amount = entries.iter().map(|entry| entry.amount).sum();

    let mut project_summaries = projects
        .into_values()
        .map(|mut summary| {
            if let Some(limit_units) = summary.quota_limit_units {
                let remaining_units = limit_units.saturating_sub(summary.used_units);
                summary.remaining_units = Some(remaining_units);
                summary.exhausted = summary.used_units >= limit_units;
            }
            summary
        })
        .collect::<Vec<_>>();

    project_summaries.sort_by(|left, right| {
        right
            .quota_limit_units
            .is_some()
            .cmp(&left.quota_limit_units.is_some())
            .then_with(|| right.exhausted.cmp(&left.exhausted))
            .then_with(|| right.used_units.cmp(&left.used_units))
            .then_with(|| left.project_id.cmp(&right.project_id))
    });

    let exhausted_project_count = project_summaries
        .iter()
        .filter(|summary| summary.exhausted)
        .count() as u64;

    BillingSummary {
        total_entries,
        project_count: project_summaries.len() as u64,
        total_units,
        total_amount,
        active_quota_policy_count: active_policies.len() as u64,
        exhausted_project_count,
        projects: project_summaries,
    }
}

pub async fn summarize_billing_from_store(store: &dyn AdminStore) -> Result<BillingSummary> {
    let entries = list_ledger_entries(store).await?;
    let policies = list_quota_policies(store).await?;
    Ok(summarize_billing_snapshot(&entries, &policies))
}
