use super::*;

fn pricing_status_is(value: &str, expected: &str) -> bool {
    value.trim().eq_ignore_ascii_case(expected)
}

fn is_pricing_plan_active(plan: &PricingPlanRecord) -> bool {
    pricing_status_is(&plan.status, "active")
}

fn is_due_planned_pricing_plan(plan: &PricingPlanRecord, now_ms: u64) -> bool {
    pricing_status_is(&plan.status, "planned")
        && plan.effective_from_ms <= now_ms
        && plan
            .effective_to_ms
            .map_or(true, |effective_to_ms| effective_to_ms >= now_ms)
}

fn compare_due_planned_pricing_candidates(
    left: &PricingPlanRecord,
    right: &PricingPlanRecord,
) -> Ordering {
    left.effective_from_ms
        .cmp(&right.effective_from_ms)
        .then(left.plan_version.cmp(&right.plan_version))
        .then(left.updated_at_ms.cmp(&right.updated_at_ms))
        .then(left.created_at_ms.cmp(&right.created_at_ms))
        .then(left.pricing_plan_id.cmp(&right.pricing_plan_id))
}

fn build_pricing_plan_with_status(
    plan: &PricingPlanRecord,
    status: &str,
    updated_at_ms: u64,
) -> PricingPlanRecord {
    PricingPlanRecord::new(
        plan.pricing_plan_id,
        plan.tenant_id,
        plan.organization_id,
        plan.plan_code.clone(),
        plan.plan_version,
    )
    .with_display_name(plan.display_name.clone())
    .with_currency_code(plan.currency_code.clone())
    .with_credit_unit_code(plan.credit_unit_code.clone())
    .with_status(status.to_owned())
    .with_effective_from_ms(plan.effective_from_ms)
    .with_effective_to_ms(plan.effective_to_ms)
    .with_created_at_ms(plan.created_at_ms)
    .with_updated_at_ms(updated_at_ms)
}

fn build_pricing_rate_with_status(
    rate: &PricingRateRecord,
    status: &str,
    updated_at_ms: u64,
) -> PricingRateRecord {
    PricingRateRecord::new(
        rate.pricing_rate_id,
        rate.tenant_id,
        rate.organization_id,
        rate.pricing_plan_id,
        rate.metric_code.clone(),
    )
    .with_capability_code(rate.capability_code.clone())
    .with_model_code(rate.model_code.clone())
    .with_provider_code(rate.provider_code.clone())
    .with_charge_unit(rate.charge_unit.clone())
    .with_pricing_method(rate.pricing_method.clone())
    .with_quantity_step(rate.quantity_step)
    .with_unit_price(rate.unit_price)
    .with_display_price_unit(rate.display_price_unit.clone())
    .with_minimum_billable_quantity(rate.minimum_billable_quantity)
    .with_minimum_charge(rate.minimum_charge)
    .with_rounding_increment(rate.rounding_increment)
    .with_rounding_mode(rate.rounding_mode.clone())
    .with_included_quantity(rate.included_quantity)
    .with_priority(rate.priority)
    .with_notes(rate.notes.clone())
    .with_status(status.to_owned())
    .with_created_at_ms(rate.created_at_ms)
    .with_updated_at_ms(updated_at_ms)
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct PricingLifecycleSynchronizationReport {
    pub changed: bool,
    pub due_group_count: u64,
    pub activated_plan_count: u64,
    pub archived_plan_count: u64,
    pub activated_rate_count: u64,
    pub archived_rate_count: u64,
    pub skipped_plan_count: u64,
    pub synchronized_at_ms: u64,
}

pub async fn synchronize_due_pricing_plan_lifecycle_with_report<K>(
    kernel: &K,
    now_ms: u64,
) -> Result<PricingLifecycleSynchronizationReport>
where
    K: CommercialBillingAdminKernel + ?Sized,
{
    let plans = kernel.list_pricing_plan_records().await?;
    let rates = kernel.list_pricing_rate_records().await?;
    let mut due_group_keys = BTreeSet::new();

    for plan in &plans {
        if is_due_planned_pricing_plan(plan, now_ms) {
            due_group_keys.insert((plan.tenant_id, plan.organization_id, plan.plan_code.clone()));
        }
    }

    let mut report = PricingLifecycleSynchronizationReport {
        due_group_count: due_group_keys.len() as u64,
        synchronized_at_ms: now_ms,
        ..PricingLifecycleSynchronizationReport::default()
    };
    if due_group_keys.is_empty() {
        return Ok(report);
    }

    let mut changed = false;

    for (tenant_id, organization_id, plan_code) in due_group_keys {
        let mut due_candidates = plans
            .iter()
            .filter(|plan| {
                plan.tenant_id == tenant_id
                    && plan.organization_id == organization_id
                    && plan.plan_code == plan_code
                    && is_due_planned_pricing_plan(plan, now_ms)
            })
            .collect::<Vec<_>>();
        due_candidates.sort_by(|left, right| compare_due_planned_pricing_candidates(left, right));
        let Some(winner) = due_candidates.last().copied() else {
            continue;
        };

        if !rates
            .iter()
            .any(|rate| rate.pricing_plan_id == winner.pricing_plan_id)
        {
            report.skipped_plan_count += 1;
            continue;
        }

        if !is_pricing_plan_active(winner) {
            kernel
                .insert_pricing_plan_record(&build_pricing_plan_with_status(
                    winner, "active", now_ms,
                ))
                .await?;
            report.activated_plan_count += 1;
            changed = true;
        }

        let archived_plan_ids = plans
            .iter()
            .filter(|plan| {
                plan.pricing_plan_id != winner.pricing_plan_id
                    && plan.tenant_id == tenant_id
                    && plan.organization_id == organization_id
                    && plan.plan_code == plan_code
                    && (is_pricing_plan_active(plan) || is_due_planned_pricing_plan(plan, now_ms))
            })
            .map(|plan| plan.pricing_plan_id)
            .collect::<BTreeSet<_>>();

        for plan in plans
            .iter()
            .filter(|plan| archived_plan_ids.contains(&plan.pricing_plan_id))
        {
            if !pricing_status_is(&plan.status, "archived") {
                kernel
                    .insert_pricing_plan_record(&build_pricing_plan_with_status(
                        plan, "archived", now_ms,
                    ))
                    .await?;
                report.archived_plan_count += 1;
                changed = true;
            }
        }

        for rate in rates
            .iter()
            .filter(|rate| rate.pricing_plan_id == winner.pricing_plan_id)
        {
            if !pricing_status_is(&rate.status, "active") {
                kernel
                    .insert_pricing_rate_record(&build_pricing_rate_with_status(
                        rate, "active", now_ms,
                    ))
                    .await?;
                report.activated_rate_count += 1;
                changed = true;
            }
        }

        for rate in rates
            .iter()
            .filter(|rate| archived_plan_ids.contains(&rate.pricing_plan_id))
        {
            if !pricing_status_is(&rate.status, "archived") {
                kernel
                    .insert_pricing_rate_record(&build_pricing_rate_with_status(
                        rate, "archived", now_ms,
                    ))
                    .await?;
                report.archived_rate_count += 1;
                changed = true;
            }
        }
    }

    report.changed = changed;
    Ok(report)
}

pub async fn synchronize_due_pricing_plan_lifecycle<K>(kernel: &K, now_ms: u64) -> Result<bool>
where
    K: CommercialBillingAdminKernel + ?Sized,
{
    Ok(
        synchronize_due_pricing_plan_lifecycle_with_report(kernel, now_ms)
            .await?
            .changed,
    )
}

