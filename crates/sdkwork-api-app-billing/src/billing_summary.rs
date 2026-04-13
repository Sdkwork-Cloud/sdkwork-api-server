use super::*;

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

pub fn summarize_billing_events(events: &[BillingEventRecord]) -> BillingEventSummary {
    if events.is_empty() {
        return BillingEventSummary::empty();
    }

    #[derive(Default)]
    struct ProjectAccumulator {
        event_count: u64,
        request_count: u64,
        total_units: u64,
        total_input_tokens: u64,
        total_output_tokens: u64,
        total_tokens: u64,
        total_image_count: u64,
        total_audio_seconds: f64,
        total_video_seconds: f64,
        total_music_seconds: f64,
        total_upstream_cost: f64,
        total_customer_charge: f64,
    }

    #[derive(Default)]
    struct GroupAccumulator {
        project_ids: BTreeSet<String>,
        event_count: u64,
        request_count: u64,
        total_upstream_cost: f64,
        total_customer_charge: f64,
    }

    #[derive(Default)]
    struct CapabilityAccumulator {
        event_count: u64,
        request_count: u64,
        total_tokens: u64,
        image_count: u64,
        audio_seconds: f64,
        video_seconds: f64,
        music_seconds: f64,
        total_upstream_cost: f64,
        total_customer_charge: f64,
    }

    #[derive(Default)]
    struct AccountingModeAccumulator {
        event_count: u64,
        request_count: u64,
        total_upstream_cost: f64,
        total_customer_charge: f64,
    }

    let mut projects = BTreeMap::<String, ProjectAccumulator>::new();
    let mut groups = BTreeMap::<Option<String>, GroupAccumulator>::new();
    let mut capabilities = BTreeMap::<String, CapabilityAccumulator>::new();
    let mut accounting_modes = BTreeMap::<BillingAccountingMode, AccountingModeAccumulator>::new();

    for event in events {
        let project = projects.entry(event.project_id.clone()).or_default();
        project.event_count += 1;
        project.request_count += event.request_count;
        project.total_units += event.units;
        project.total_input_tokens += event.input_tokens;
        project.total_output_tokens += event.output_tokens;
        project.total_tokens += event.total_tokens;
        project.total_image_count += event.image_count;
        project.total_audio_seconds += event.audio_seconds;
        project.total_video_seconds += event.video_seconds;
        project.total_music_seconds += event.music_seconds;
        project.total_upstream_cost += event.upstream_cost;
        project.total_customer_charge += event.customer_charge;

        let group = groups.entry(event.api_key_group_id.clone()).or_default();
        group.project_ids.insert(event.project_id.clone());
        group.event_count += 1;
        group.request_count += event.request_count;
        group.total_upstream_cost += event.upstream_cost;
        group.total_customer_charge += event.customer_charge;

        let capability = capabilities.entry(event.capability.clone()).or_default();
        capability.event_count += 1;
        capability.request_count += event.request_count;
        capability.total_tokens += event.total_tokens;
        capability.image_count += event.image_count;
        capability.audio_seconds += event.audio_seconds;
        capability.video_seconds += event.video_seconds;
        capability.music_seconds += event.music_seconds;
        capability.total_upstream_cost += event.upstream_cost;
        capability.total_customer_charge += event.customer_charge;

        let mode = accounting_modes.entry(event.accounting_mode).or_default();
        mode.event_count += 1;
        mode.request_count += event.request_count;
        mode.total_upstream_cost += event.upstream_cost;
        mode.total_customer_charge += event.customer_charge;
    }

    let mut project_summaries = projects
        .into_iter()
        .map(|(project_id, summary)| BillingEventProjectSummary {
            project_id,
            event_count: summary.event_count,
            request_count: summary.request_count,
            total_units: summary.total_units,
            total_input_tokens: summary.total_input_tokens,
            total_output_tokens: summary.total_output_tokens,
            total_tokens: summary.total_tokens,
            total_image_count: summary.total_image_count,
            total_audio_seconds: summary.total_audio_seconds,
            total_video_seconds: summary.total_video_seconds,
            total_music_seconds: summary.total_music_seconds,
            total_upstream_cost: summary.total_upstream_cost,
            total_customer_charge: summary.total_customer_charge,
        })
        .collect::<Vec<_>>();
    project_summaries.sort_by(|left, right| {
        right
            .total_customer_charge
            .total_cmp(&left.total_customer_charge)
            .then_with(|| right.request_count.cmp(&left.request_count))
            .then_with(|| left.project_id.cmp(&right.project_id))
    });

    let mut group_summaries = groups
        .into_iter()
        .map(|(api_key_group_id, summary)| BillingEventGroupSummary {
            api_key_group_id,
            project_count: summary.project_ids.len() as u64,
            event_count: summary.event_count,
            request_count: summary.request_count,
            total_upstream_cost: summary.total_upstream_cost,
            total_customer_charge: summary.total_customer_charge,
        })
        .collect::<Vec<_>>();
    group_summaries.sort_by(|left, right| {
        right
            .total_customer_charge
            .total_cmp(&left.total_customer_charge)
            .then_with(|| right.request_count.cmp(&left.request_count))
            .then_with(|| left.api_key_group_id.cmp(&right.api_key_group_id))
    });

    let mut capability_summaries = capabilities
        .into_iter()
        .map(|(capability, summary)| BillingEventCapabilitySummary {
            capability,
            event_count: summary.event_count,
            request_count: summary.request_count,
            total_tokens: summary.total_tokens,
            image_count: summary.image_count,
            audio_seconds: summary.audio_seconds,
            video_seconds: summary.video_seconds,
            music_seconds: summary.music_seconds,
            total_upstream_cost: summary.total_upstream_cost,
            total_customer_charge: summary.total_customer_charge,
        })
        .collect::<Vec<_>>();
    capability_summaries.sort_by(|left, right| {
        right
            .request_count
            .cmp(&left.request_count)
            .then_with(|| left.capability.cmp(&right.capability))
    });

    let mut accounting_mode_summaries = accounting_modes
        .into_iter()
        .map(
            |(accounting_mode, summary)| BillingEventAccountingModeSummary {
                accounting_mode,
                event_count: summary.event_count,
                request_count: summary.request_count,
                total_upstream_cost: summary.total_upstream_cost,
                total_customer_charge: summary.total_customer_charge,
            },
        )
        .collect::<Vec<_>>();
    accounting_mode_summaries.sort_by(|left, right| {
        right
            .total_customer_charge
            .total_cmp(&left.total_customer_charge)
            .then_with(|| right.event_count.cmp(&left.event_count))
            .then_with(|| left.accounting_mode.cmp(&right.accounting_mode))
    });

    BillingEventSummary {
        total_events: events.len() as u64,
        project_count: project_summaries.len() as u64,
        group_count: group_summaries.len() as u64,
        capability_count: capability_summaries.len() as u64,
        total_request_count: events.iter().map(|event| event.request_count).sum(),
        total_units: events.iter().map(|event| event.units).sum(),
        total_input_tokens: events.iter().map(|event| event.input_tokens).sum(),
        total_output_tokens: events.iter().map(|event| event.output_tokens).sum(),
        total_tokens: events.iter().map(|event| event.total_tokens).sum(),
        total_image_count: events.iter().map(|event| event.image_count).sum(),
        total_audio_seconds: events.iter().map(|event| event.audio_seconds).sum(),
        total_video_seconds: events.iter().map(|event| event.video_seconds).sum(),
        total_music_seconds: events.iter().map(|event| event.music_seconds).sum(),
        total_upstream_cost: events.iter().map(|event| event.upstream_cost).sum(),
        total_customer_charge: events.iter().map(|event| event.customer_charge).sum(),
        projects: project_summaries,
        groups: group_summaries,
        capabilities: capability_summaries,
        accounting_modes: accounting_mode_summaries,
    }
}

pub async fn summarize_billing_from_store(store: &dyn AdminStore) -> Result<BillingSummary> {
    let entries = list_ledger_entries(store).await?;
    let policies = list_quota_policies(store).await?;
    Ok(summarize_billing_snapshot(&entries, &policies))
}

pub async fn summarize_billing_events_from_store(
    store: &dyn AdminStore,
) -> Result<BillingEventSummary> {
    let events = list_billing_events(store).await?;
    Ok(summarize_billing_events(&events))
}
