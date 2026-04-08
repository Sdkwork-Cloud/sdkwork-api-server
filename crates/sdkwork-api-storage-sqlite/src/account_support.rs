use super::*;

pub(crate) fn account_type_as_str(value: AccountType) -> &'static str {
    match value {
        AccountType::Primary => "primary",
        AccountType::Grant => "grant",
        AccountType::Postpaid => "postpaid",
    }
}

pub(crate) fn parse_account_type(value: &str) -> Result<AccountType> {
    match value {
        "primary" => Ok(AccountType::Primary),
        "grant" => Ok(AccountType::Grant),
        "postpaid" => Ok(AccountType::Postpaid),
        other => Err(anyhow::anyhow!("unknown account_type: {other}")),
    }
}

pub(crate) fn account_status_as_str(value: AccountStatus) -> &'static str {
    match value {
        AccountStatus::Active => "active",
        AccountStatus::Suspended => "suspended",
        AccountStatus::Closed => "closed",
    }
}

pub(crate) fn parse_account_status(value: &str) -> Result<AccountStatus> {
    match value {
        "active" => Ok(AccountStatus::Active),
        "suspended" => Ok(AccountStatus::Suspended),
        "closed" => Ok(AccountStatus::Closed),
        other => Err(anyhow::anyhow!("unknown account_status: {other}")),
    }
}

pub(crate) fn account_benefit_type_as_str(value: AccountBenefitType) -> &'static str {
    match value {
        AccountBenefitType::CashCredit => "cash_credit",
        AccountBenefitType::PromoCredit => "promo_credit",
        AccountBenefitType::RequestAllowance => "request_allowance",
        AccountBenefitType::TokenAllowance => "token_allowance",
        AccountBenefitType::ImageAllowance => "image_allowance",
        AccountBenefitType::AudioAllowance => "audio_allowance",
        AccountBenefitType::VideoAllowance => "video_allowance",
        AccountBenefitType::MusicAllowance => "music_allowance",
    }
}

pub(crate) fn parse_account_benefit_type(value: &str) -> Result<AccountBenefitType> {
    match value {
        "cash_credit" => Ok(AccountBenefitType::CashCredit),
        "promo_credit" => Ok(AccountBenefitType::PromoCredit),
        "request_allowance" => Ok(AccountBenefitType::RequestAllowance),
        "token_allowance" => Ok(AccountBenefitType::TokenAllowance),
        "image_allowance" => Ok(AccountBenefitType::ImageAllowance),
        "audio_allowance" => Ok(AccountBenefitType::AudioAllowance),
        "video_allowance" => Ok(AccountBenefitType::VideoAllowance),
        "music_allowance" => Ok(AccountBenefitType::MusicAllowance),
        other => Err(anyhow::anyhow!("unknown account_benefit_type: {other}")),
    }
}

pub(crate) fn account_benefit_source_type_as_str(value: AccountBenefitSourceType) -> &'static str {
    match value {
        AccountBenefitSourceType::Recharge => "recharge",
        AccountBenefitSourceType::Coupon => "coupon",
        AccountBenefitSourceType::Grant => "grant",
        AccountBenefitSourceType::Order => "order",
        AccountBenefitSourceType::ManualAdjustment => "manual_adjustment",
    }
}

pub(crate) fn parse_account_benefit_source_type(value: &str) -> Result<AccountBenefitSourceType> {
    match value {
        "recharge" => Ok(AccountBenefitSourceType::Recharge),
        "coupon" => Ok(AccountBenefitSourceType::Coupon),
        "grant" => Ok(AccountBenefitSourceType::Grant),
        "order" => Ok(AccountBenefitSourceType::Order),
        "manual_adjustment" => Ok(AccountBenefitSourceType::ManualAdjustment),
        other => Err(anyhow::anyhow!(
            "unknown account_benefit_source_type: {other}"
        )),
    }
}

pub(crate) fn account_benefit_lot_status_as_str(value: AccountBenefitLotStatus) -> &'static str {
    match value {
        AccountBenefitLotStatus::Active => "active",
        AccountBenefitLotStatus::Exhausted => "exhausted",
        AccountBenefitLotStatus::Expired => "expired",
        AccountBenefitLotStatus::Disabled => "disabled",
    }
}

pub(crate) fn parse_account_benefit_lot_status(value: &str) -> Result<AccountBenefitLotStatus> {
    match value {
        "active" => Ok(AccountBenefitLotStatus::Active),
        "exhausted" => Ok(AccountBenefitLotStatus::Exhausted),
        "expired" => Ok(AccountBenefitLotStatus::Expired),
        "disabled" => Ok(AccountBenefitLotStatus::Disabled),
        other => Err(anyhow::anyhow!(
            "unknown account_benefit_lot_status: {other}"
        )),
    }
}

pub(crate) fn account_hold_status_as_str(value: AccountHoldStatus) -> &'static str {
    match value {
        AccountHoldStatus::Held => "held",
        AccountHoldStatus::Captured => "captured",
        AccountHoldStatus::PartiallyReleased => "partially_released",
        AccountHoldStatus::Released => "released",
        AccountHoldStatus::Expired => "expired",
        AccountHoldStatus::Failed => "failed",
    }
}

pub(crate) fn parse_account_hold_status(value: &str) -> Result<AccountHoldStatus> {
    match value {
        "held" => Ok(AccountHoldStatus::Held),
        "captured" => Ok(AccountHoldStatus::Captured),
        "partially_released" => Ok(AccountHoldStatus::PartiallyReleased),
        "released" => Ok(AccountHoldStatus::Released),
        "expired" => Ok(AccountHoldStatus::Expired),
        "failed" => Ok(AccountHoldStatus::Failed),
        other => Err(anyhow::anyhow!("unknown account_hold_status: {other}")),
    }
}

pub(crate) fn account_ledger_entry_type_as_str(value: AccountLedgerEntryType) -> &'static str {
    match value {
        AccountLedgerEntryType::HoldCreate => "hold_create",
        AccountLedgerEntryType::HoldRelease => "hold_release",
        AccountLedgerEntryType::SettlementCapture => "settlement_capture",
        AccountLedgerEntryType::GrantIssue => "grant_issue",
        AccountLedgerEntryType::ManualAdjustment => "manual_adjustment",
        AccountLedgerEntryType::Refund => "refund",
    }
}

pub(crate) fn parse_account_ledger_entry_type(value: &str) -> Result<AccountLedgerEntryType> {
    match value {
        "hold_create" => Ok(AccountLedgerEntryType::HoldCreate),
        "hold_release" => Ok(AccountLedgerEntryType::HoldRelease),
        "settlement_capture" => Ok(AccountLedgerEntryType::SettlementCapture),
        "grant_issue" => Ok(AccountLedgerEntryType::GrantIssue),
        "manual_adjustment" => Ok(AccountLedgerEntryType::ManualAdjustment),
        "refund" => Ok(AccountLedgerEntryType::Refund),
        other => Err(anyhow::anyhow!(
            "unknown account_ledger_entry_type: {other}"
        )),
    }
}

pub(crate) fn request_status_as_str(value: RequestStatus) -> &'static str {
    match value {
        RequestStatus::Pending => "pending",
        RequestStatus::Running => "running",
        RequestStatus::Succeeded => "succeeded",
        RequestStatus::Failed => "failed",
        RequestStatus::Cancelled => "cancelled",
    }
}

pub(crate) fn parse_request_status(value: &str) -> Result<RequestStatus> {
    match value {
        "pending" => Ok(RequestStatus::Pending),
        "running" => Ok(RequestStatus::Running),
        "succeeded" => Ok(RequestStatus::Succeeded),
        "failed" => Ok(RequestStatus::Failed),
        "cancelled" => Ok(RequestStatus::Cancelled),
        other => Err(anyhow::anyhow!("unknown request_status: {other}")),
    }
}

pub(crate) fn usage_capture_status_as_str(value: UsageCaptureStatus) -> &'static str {
    match value {
        UsageCaptureStatus::Pending => "pending",
        UsageCaptureStatus::Estimated => "estimated",
        UsageCaptureStatus::Captured => "captured",
        UsageCaptureStatus::Reconciled => "reconciled",
        UsageCaptureStatus::Failed => "failed",
    }
}

pub(crate) fn parse_usage_capture_status(value: &str) -> Result<UsageCaptureStatus> {
    match value {
        "pending" => Ok(UsageCaptureStatus::Pending),
        "estimated" => Ok(UsageCaptureStatus::Estimated),
        "captured" => Ok(UsageCaptureStatus::Captured),
        "reconciled" => Ok(UsageCaptureStatus::Reconciled),
        "failed" => Ok(UsageCaptureStatus::Failed),
        other => Err(anyhow::anyhow!("unknown usage_capture_status: {other}")),
    }
}

pub(crate) fn request_settlement_status_as_str(value: RequestSettlementStatus) -> &'static str {
    match value {
        RequestSettlementStatus::Pending => "pending",
        RequestSettlementStatus::Captured => "captured",
        RequestSettlementStatus::PartiallyReleased => "partially_released",
        RequestSettlementStatus::Released => "released",
        RequestSettlementStatus::Refunded => "refunded",
        RequestSettlementStatus::Failed => "failed",
    }
}

pub(crate) fn parse_request_settlement_status(value: &str) -> Result<RequestSettlementStatus> {
    match value {
        "pending" => Ok(RequestSettlementStatus::Pending),
        "captured" => Ok(RequestSettlementStatus::Captured),
        "partially_released" => Ok(RequestSettlementStatus::PartiallyReleased),
        "released" => Ok(RequestSettlementStatus::Released),
        "refunded" => Ok(RequestSettlementStatus::Refunded),
        "failed" => Ok(RequestSettlementStatus::Failed),
        other => Err(anyhow::anyhow!(
            "unknown request_settlement_status: {other}"
        )),
    }
}

pub(crate) fn decode_account_record_row(row: SqliteRow) -> Result<AccountRecord> {
    Ok(AccountRecord::new(
        u64::try_from(row.try_get::<i64, _>("account_id")?)?,
        u64::try_from(row.try_get::<i64, _>("tenant_id")?)?,
        u64::try_from(row.try_get::<i64, _>("organization_id")?)?,
        u64::try_from(row.try_get::<i64, _>("user_id")?)?,
        parse_account_type(&row.try_get::<String, _>("account_type")?)?,
    )
    .with_currency_code(row.try_get::<String, _>("currency_code")?)
    .with_credit_unit_code(row.try_get::<String, _>("credit_unit_code")?)
    .with_status(parse_account_status(&row.try_get::<String, _>("status")?)?)
    .with_allow_overdraft(row.try_get::<i64, _>("allow_overdraft")? != 0)
    .with_overdraft_limit(row.try_get::<f64, _>("overdraft_limit")?)
    .with_created_at_ms(u64::try_from(row.try_get::<i64, _>("created_at_ms")?)?)
    .with_updated_at_ms(u64::try_from(row.try_get::<i64, _>("updated_at_ms")?)?))
}

pub(crate) fn decode_account_commerce_reconciliation_state_row(
    row: SqliteRow,
) -> Result<AccountCommerceReconciliationStateRecord> {
    Ok(AccountCommerceReconciliationStateRecord::new(
        u64::try_from(row.try_get::<i64, _>("tenant_id")?)?,
        u64::try_from(row.try_get::<i64, _>("organization_id")?)?,
        u64::try_from(row.try_get::<i64, _>("account_id")?)?,
        row.try_get::<String, _>("project_id")?,
        row.try_get::<String, _>("last_order_id")?,
    )
    .with_last_order_updated_at_ms(u64::try_from(
        row.try_get::<i64, _>("last_order_updated_at_ms")?,
    )?)
    .with_last_order_created_at_ms(u64::try_from(
        row.try_get::<i64, _>("last_order_created_at_ms")?,
    )?)
    .with_updated_at_ms(u64::try_from(row.try_get::<i64, _>("updated_at_ms")?)?))
}

pub(crate) fn decode_account_benefit_lot_row(row: SqliteRow) -> Result<AccountBenefitLotRecord> {
    Ok(AccountBenefitLotRecord::new(
        u64::try_from(row.try_get::<i64, _>("lot_id")?)?,
        u64::try_from(row.try_get::<i64, _>("tenant_id")?)?,
        u64::try_from(row.try_get::<i64, _>("organization_id")?)?,
        u64::try_from(row.try_get::<i64, _>("account_id")?)?,
        u64::try_from(row.try_get::<i64, _>("user_id")?)?,
        parse_account_benefit_type(&row.try_get::<String, _>("benefit_type")?)?,
    )
    .with_source_type(parse_account_benefit_source_type(
        &row.try_get::<String, _>("source_type")?,
    )?)
    .with_source_id(
        row.try_get::<Option<i64>, _>("source_id")?
            .map(u64::try_from)
            .transpose()?,
    )
    .with_scope_json(row.try_get::<Option<String>, _>("scope_json")?)
    .with_original_quantity(row.try_get::<f64, _>("original_quantity")?)
    .with_remaining_quantity(row.try_get::<f64, _>("remaining_quantity")?)
    .with_held_quantity(row.try_get::<f64, _>("held_quantity")?)
    .with_priority(row.try_get::<i32, _>("priority")?)
    .with_acquired_unit_cost(row.try_get::<Option<f64>, _>("acquired_unit_cost")?)
    .with_issued_at_ms(u64::try_from(row.try_get::<i64, _>("issued_at_ms")?)?)
    .with_expires_at_ms(
        row.try_get::<Option<i64>, _>("expires_at_ms")?
            .map(u64::try_from)
            .transpose()?,
    )
    .with_status(parse_account_benefit_lot_status(
        &row.try_get::<String, _>("status")?,
    )?)
    .with_created_at_ms(u64::try_from(row.try_get::<i64, _>("created_at_ms")?)?)
    .with_updated_at_ms(u64::try_from(row.try_get::<i64, _>("updated_at_ms")?)?))
}

pub(crate) fn decode_account_hold_row(row: SqliteRow) -> Result<AccountHoldRecord> {
    Ok(AccountHoldRecord::new(
        u64::try_from(row.try_get::<i64, _>("hold_id")?)?,
        u64::try_from(row.try_get::<i64, _>("tenant_id")?)?,
        u64::try_from(row.try_get::<i64, _>("organization_id")?)?,
        u64::try_from(row.try_get::<i64, _>("account_id")?)?,
        u64::try_from(row.try_get::<i64, _>("user_id")?)?,
        u64::try_from(row.try_get::<i64, _>("request_id")?)?,
    )
    .with_status(parse_account_hold_status(
        &row.try_get::<String, _>("hold_status")?,
    )?)
    .with_estimated_quantity(row.try_get::<f64, _>("estimated_quantity")?)
    .with_captured_quantity(row.try_get::<f64, _>("captured_quantity")?)
    .with_released_quantity(row.try_get::<f64, _>("released_quantity")?)
    .with_expires_at_ms(u64::try_from(row.try_get::<i64, _>("expires_at_ms")?)?)
    .with_created_at_ms(u64::try_from(row.try_get::<i64, _>("created_at_ms")?)?)
    .with_updated_at_ms(u64::try_from(row.try_get::<i64, _>("updated_at_ms")?)?))
}

pub(crate) fn decode_account_hold_allocation_row(
    row: SqliteRow,
) -> Result<AccountHoldAllocationRecord> {
    Ok(AccountHoldAllocationRecord::new(
        u64::try_from(row.try_get::<i64, _>("hold_allocation_id")?)?,
        u64::try_from(row.try_get::<i64, _>("tenant_id")?)?,
        u64::try_from(row.try_get::<i64, _>("organization_id")?)?,
        u64::try_from(row.try_get::<i64, _>("hold_id")?)?,
        u64::try_from(row.try_get::<i64, _>("lot_id")?)?,
    )
    .with_allocated_quantity(row.try_get::<f64, _>("allocated_quantity")?)
    .with_captured_quantity(row.try_get::<f64, _>("captured_quantity")?)
    .with_released_quantity(row.try_get::<f64, _>("released_quantity")?)
    .with_created_at_ms(u64::try_from(row.try_get::<i64, _>("created_at_ms")?)?)
    .with_updated_at_ms(u64::try_from(row.try_get::<i64, _>("updated_at_ms")?)?))
}

pub(crate) fn decode_account_ledger_entry_row(row: SqliteRow) -> Result<AccountLedgerEntryRecord> {
    Ok(AccountLedgerEntryRecord::new(
        u64::try_from(row.try_get::<i64, _>("ledger_entry_id")?)?,
        u64::try_from(row.try_get::<i64, _>("tenant_id")?)?,
        u64::try_from(row.try_get::<i64, _>("organization_id")?)?,
        u64::try_from(row.try_get::<i64, _>("account_id")?)?,
        u64::try_from(row.try_get::<i64, _>("user_id")?)?,
        parse_account_ledger_entry_type(&row.try_get::<String, _>("entry_type")?)?,
    )
    .with_request_id(
        row.try_get::<Option<i64>, _>("request_id")?
            .map(u64::try_from)
            .transpose()?,
    )
    .with_hold_id(
        row.try_get::<Option<i64>, _>("hold_id")?
            .map(u64::try_from)
            .transpose()?,
    )
    .with_benefit_type(row.try_get::<Option<String>, _>("benefit_type")?)
    .with_quantity(row.try_get::<f64, _>("quantity")?)
    .with_amount(row.try_get::<f64, _>("amount")?)
    .with_created_at_ms(u64::try_from(row.try_get::<i64, _>("created_at_ms")?)?))
}

pub(crate) fn decode_account_ledger_allocation_row(
    row: SqliteRow,
) -> Result<AccountLedgerAllocationRecord> {
    Ok(AccountLedgerAllocationRecord::new(
        u64::try_from(row.try_get::<i64, _>("ledger_allocation_id")?)?,
        u64::try_from(row.try_get::<i64, _>("tenant_id")?)?,
        u64::try_from(row.try_get::<i64, _>("organization_id")?)?,
        u64::try_from(row.try_get::<i64, _>("ledger_entry_id")?)?,
        u64::try_from(row.try_get::<i64, _>("lot_id")?)?,
    )
    .with_quantity_delta(row.try_get::<f64, _>("quantity_delta")?)
    .with_created_at_ms(u64::try_from(row.try_get::<i64, _>("created_at_ms")?)?))
}

pub(crate) fn decode_request_meter_fact_row(row: SqliteRow) -> Result<RequestMeterFactRecord> {
    Ok(RequestMeterFactRecord::new(
        u64::try_from(row.try_get::<i64, _>("request_id")?)?,
        u64::try_from(row.try_get::<i64, _>("tenant_id")?)?,
        u64::try_from(row.try_get::<i64, _>("organization_id")?)?,
        u64::try_from(row.try_get::<i64, _>("user_id")?)?,
        u64::try_from(row.try_get::<i64, _>("account_id")?)?,
        row.try_get::<String, _>("auth_type")?,
        row.try_get::<String, _>("capability_code")?,
        row.try_get::<String, _>("channel_code")?,
        row.try_get::<String, _>("model_code")?,
        row.try_get::<String, _>("provider_code")?,
    )
    .with_api_key_id(
        row.try_get::<Option<i64>, _>("api_key_id")?
            .map(u64::try_from)
            .transpose()?,
    )
    .with_api_key_hash(row.try_get::<Option<String>, _>("api_key_hash")?)
    .with_jwt_subject(row.try_get::<Option<String>, _>("jwt_subject")?)
    .with_platform(row.try_get::<Option<String>, _>("platform")?)
    .with_owner(row.try_get::<Option<String>, _>("owner")?)
    .with_request_trace_id(row.try_get::<Option<String>, _>("request_trace_id")?)
    .with_gateway_request_ref(row.try_get::<Option<String>, _>("gateway_request_ref")?)
    .with_upstream_request_ref(row.try_get::<Option<String>, _>("upstream_request_ref")?)
    .with_protocol_family(row.try_get::<String, _>("protocol_family")?)
    .with_request_status(parse_request_status(
        &row.try_get::<String, _>("request_status")?,
    )?)
    .with_usage_capture_status(parse_usage_capture_status(
        &row.try_get::<String, _>("usage_capture_status")?,
    )?)
    .with_cost_pricing_plan_id(
        row.try_get::<Option<i64>, _>("cost_pricing_plan_id")?
            .map(u64::try_from)
            .transpose()?,
    )
    .with_retail_pricing_plan_id(
        row.try_get::<Option<i64>, _>("retail_pricing_plan_id")?
            .map(u64::try_from)
            .transpose()?,
    )
    .with_estimated_credit_hold(row.try_get::<f64, _>("estimated_credit_hold")?)
    .with_actual_credit_charge(row.try_get::<Option<f64>, _>("actual_credit_charge")?)
    .with_actual_provider_cost(row.try_get::<Option<f64>, _>("actual_provider_cost")?)
    .with_started_at_ms(u64::try_from(row.try_get::<i64, _>("started_at_ms")?)?)
    .with_finished_at_ms(
        row.try_get::<Option<i64>, _>("finished_at_ms")?
            .map(u64::try_from)
            .transpose()?,
    )
    .with_created_at_ms(u64::try_from(row.try_get::<i64, _>("created_at_ms")?)?)
    .with_updated_at_ms(u64::try_from(row.try_get::<i64, _>("updated_at_ms")?)?))
}

pub(crate) fn decode_request_meter_metric_row(row: SqliteRow) -> Result<RequestMeterMetricRecord> {
    Ok(RequestMeterMetricRecord::new(
        u64::try_from(row.try_get::<i64, _>("request_metric_id")?)?,
        u64::try_from(row.try_get::<i64, _>("tenant_id")?)?,
        u64::try_from(row.try_get::<i64, _>("organization_id")?)?,
        u64::try_from(row.try_get::<i64, _>("request_id")?)?,
        row.try_get::<String, _>("metric_code")?,
        row.try_get::<f64, _>("quantity")?,
    )
    .with_provider_field(row.try_get::<Option<String>, _>("provider_field")?)
    .with_source_kind(row.try_get::<String, _>("source_kind")?)
    .with_capture_stage(row.try_get::<String, _>("capture_stage")?)
    .with_is_billable(row.try_get::<i64, _>("is_billable")? != 0)
    .with_captured_at_ms(u64::try_from(row.try_get::<i64, _>("captured_at_ms")?)?))
}

pub(crate) fn decode_pricing_plan_row(row: SqliteRow) -> Result<PricingPlanRecord> {
    Ok(PricingPlanRecord::new(
        u64::try_from(row.try_get::<i64, _>("pricing_plan_id")?)?,
        u64::try_from(row.try_get::<i64, _>("tenant_id")?)?,
        u64::try_from(row.try_get::<i64, _>("organization_id")?)?,
        row.try_get::<String, _>("plan_code")?,
        u64::try_from(row.try_get::<i64, _>("plan_version")?)?,
    )
    .with_display_name(row.try_get::<String, _>("display_name")?)
    .with_currency_code(row.try_get::<String, _>("currency_code")?)
    .with_credit_unit_code(row.try_get::<String, _>("credit_unit_code")?)
    .with_status(row.try_get::<String, _>("status")?)
    .with_effective_from_ms(u64::try_from(row.try_get::<i64, _>("effective_from_ms")?)?)
    .with_effective_to_ms(
        row.try_get::<Option<i64>, _>("effective_to_ms")?
            .map(u64::try_from)
            .transpose()?,
    )
    .with_created_at_ms(u64::try_from(row.try_get::<i64, _>("created_at_ms")?)?)
    .with_updated_at_ms(u64::try_from(row.try_get::<i64, _>("updated_at_ms")?)?))
}

pub(crate) fn decode_pricing_rate_row(row: SqliteRow) -> Result<PricingRateRecord> {
    Ok(PricingRateRecord::new(
        u64::try_from(row.try_get::<i64, _>("pricing_rate_id")?)?,
        u64::try_from(row.try_get::<i64, _>("tenant_id")?)?,
        u64::try_from(row.try_get::<i64, _>("organization_id")?)?,
        u64::try_from(row.try_get::<i64, _>("pricing_plan_id")?)?,
        row.try_get::<String, _>("metric_code")?,
    )
    .with_capability_code(row.try_get::<Option<String>, _>("capability_code")?)
    .with_model_code(row.try_get::<Option<String>, _>("model_code")?)
    .with_provider_code(row.try_get::<Option<String>, _>("provider_code")?)
    .with_charge_unit(row.try_get::<String, _>("charge_unit")?)
    .with_pricing_method(row.try_get::<String, _>("pricing_method")?)
    .with_quantity_step(row.try_get::<f64, _>("quantity_step")?)
    .with_unit_price(row.try_get::<f64, _>("unit_price")?)
    .with_display_price_unit(row.try_get::<String, _>("display_price_unit")?)
    .with_minimum_billable_quantity(row.try_get::<f64, _>("minimum_billable_quantity")?)
    .with_minimum_charge(row.try_get::<f64, _>("minimum_charge")?)
    .with_rounding_increment(row.try_get::<f64, _>("rounding_increment")?)
    .with_rounding_mode(row.try_get::<String, _>("rounding_mode")?)
    .with_included_quantity(row.try_get::<f64, _>("included_quantity")?)
    .with_priority(u64::try_from(row.try_get::<i64, _>("priority")?)?)
    .with_notes(row.try_get::<Option<String>, _>("notes")?)
    .with_status(row.try_get::<String, _>("status")?)
    .with_created_at_ms(u64::try_from(row.try_get::<i64, _>("created_at_ms")?)?)
    .with_updated_at_ms(u64::try_from(row.try_get::<i64, _>("updated_at_ms")?)?))
}

pub(crate) fn decode_request_settlement_row(row: SqliteRow) -> Result<RequestSettlementRecord> {
    Ok(RequestSettlementRecord::new(
        u64::try_from(row.try_get::<i64, _>("request_settlement_id")?)?,
        u64::try_from(row.try_get::<i64, _>("tenant_id")?)?,
        u64::try_from(row.try_get::<i64, _>("organization_id")?)?,
        u64::try_from(row.try_get::<i64, _>("request_id")?)?,
        u64::try_from(row.try_get::<i64, _>("account_id")?)?,
        u64::try_from(row.try_get::<i64, _>("user_id")?)?,
    )
    .with_hold_id(
        row.try_get::<Option<i64>, _>("hold_id")?
            .map(u64::try_from)
            .transpose()?,
    )
    .with_status(parse_request_settlement_status(
        &row.try_get::<String, _>("settlement_status")?,
    )?)
    .with_estimated_credit_hold(row.try_get::<f64, _>("estimated_credit_hold")?)
    .with_released_credit_amount(row.try_get::<f64, _>("released_credit_amount")?)
    .with_captured_credit_amount(row.try_get::<f64, _>("captured_credit_amount")?)
    .with_provider_cost_amount(row.try_get::<f64, _>("provider_cost_amount")?)
    .with_retail_charge_amount(row.try_get::<f64, _>("retail_charge_amount")?)
    .with_shortfall_amount(row.try_get::<f64, _>("shortfall_amount")?)
    .with_refunded_amount(row.try_get::<f64, _>("refunded_amount")?)
    .with_settled_at_ms(u64::try_from(row.try_get::<i64, _>("settled_at_ms")?)?)
    .with_created_at_ms(u64::try_from(row.try_get::<i64, _>("created_at_ms")?)?)
    .with_updated_at_ms(u64::try_from(row.try_get::<i64, _>("updated_at_ms")?)?))
}
