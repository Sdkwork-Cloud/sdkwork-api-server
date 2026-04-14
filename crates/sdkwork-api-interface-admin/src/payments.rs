use super::*;
use sdkwork_api_storage_core::PaymentKernelStore;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ResolveReconciliationLineRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) resolved_at_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct PaymentReconciliationReasonBreakdownItem {
    pub(crate) reason_code: String,
    pub(crate) count: usize,
    pub(crate) latest_updated_at_ms: u64,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct PaymentReconciliationSummaryResponse {
    pub(crate) total_count: usize,
    pub(crate) active_count: usize,
    pub(crate) resolved_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) latest_updated_at_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) oldest_active_created_at_ms: Option<u64>,
    pub(crate) active_reason_breakdown: Vec<PaymentReconciliationReasonBreakdownItem>,
}

#[derive(Debug, Deserialize, Default)]
pub(crate) struct PaymentReconciliationListQuery {
    #[serde(default)]
    pub(crate) lifecycle: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub(crate) struct PaymentGatewayAccountListQuery {
    #[serde(default)]
    pub(crate) provider_code: Option<String>,
    #[serde(default)]
    pub(crate) status: Option<String>,
    #[serde(default)]
    pub(crate) tenant_id: Option<u64>,
    #[serde(default)]
    pub(crate) organization_id: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct UpsertPaymentGatewayAccountRequest {
    pub(crate) gateway_account_id: String,
    pub(crate) tenant_id: u64,
    pub(crate) organization_id: u64,
    pub(crate) provider_code: String,
    pub(crate) environment: String,
    pub(crate) merchant_id: String,
    pub(crate) app_id: String,
    pub(crate) status: String,
    pub(crate) priority: i32,
    #[serde(default)]
    pub(crate) created_at_ms: Option<u64>,
    #[serde(default)]
    pub(crate) updated_at_ms: Option<u64>,
}

#[derive(Debug, Deserialize, Default)]
pub(crate) struct PaymentChannelPolicyListQuery {
    #[serde(default)]
    pub(crate) provider_code: Option<String>,
    #[serde(default)]
    pub(crate) status: Option<String>,
    #[serde(default)]
    pub(crate) tenant_id: Option<u64>,
    #[serde(default)]
    pub(crate) organization_id: Option<u64>,
    #[serde(default)]
    pub(crate) scene_code: Option<String>,
    #[serde(default)]
    pub(crate) currency_code: Option<String>,
    #[serde(default)]
    pub(crate) client_kind: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct UpsertPaymentChannelPolicyRequest {
    pub(crate) channel_policy_id: String,
    pub(crate) tenant_id: u64,
    pub(crate) organization_id: u64,
    pub(crate) scene_code: String,
    pub(crate) country_code: String,
    pub(crate) currency_code: String,
    pub(crate) client_kind: String,
    pub(crate) provider_code: String,
    pub(crate) method_code: String,
    pub(crate) priority: i32,
    pub(crate) status: String,
    #[serde(default)]
    pub(crate) created_at_ms: Option<u64>,
    #[serde(default)]
    pub(crate) updated_at_ms: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ApproveRefundOrderRequest {
    #[serde(default)]
    pub(crate) approved_amount_minor: Option<u64>,
    pub(crate) approved_at_ms: u64,
}

#[derive(Debug, Deserialize)]
pub(crate) struct CancelRefundOrderRequest {
    pub(crate) canceled_at_ms: u64,
}

#[derive(Debug, Deserialize)]
pub(crate) struct StartRefundOrderRequest {
    pub(crate) started_at_ms: u64,
}

#[derive(Debug, Deserialize, Default)]
pub(crate) struct RefundOrderListQuery {
    #[serde(default)]
    pub(crate) refund_status: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PaymentReconciliationLifecycle {
    All,
    Active,
    Resolved,
}

impl PaymentReconciliationLifecycle {
    fn parse(raw: Option<&str>) -> anyhow::Result<Self> {
        match raw.unwrap_or("all") {
            "all" => Ok(Self::All),
            "active" => Ok(Self::Active),
            "resolved" => Ok(Self::Resolved),
            other => Err(anyhow::anyhow!(
                "unsupported reconciliation lifecycle filter: {other}"
            )),
        }
    }

    fn matches(self, line: &ReconciliationMatchSummaryRecord) -> bool {
        match self {
            Self::All => true,
            Self::Active => !matches!(line.match_status, ReconciliationMatchStatus::Resolved),
            Self::Resolved => matches!(line.match_status, ReconciliationMatchStatus::Resolved),
        }
    }
}

fn payment_store_kernel(
    state: &AdminApiState,
) -> Result<&Arc<dyn CommercialKernelStore>, (StatusCode, Json<ErrorResponse>)> {
    state.payment_store.as_ref().ok_or_else(|| {
        error_response(
            StatusCode::NOT_IMPLEMENTED,
            "payment control plane is unavailable for the current storage runtime",
        )
    })
}

pub(crate) async fn list_payment_orders_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<PaymentOrderRecord>>, StatusCode> {
    let payment_store = payment_store_kernel(&state).map_err(|(status, _)| status)?;
    load_admin_payment_orders(payment_store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub(crate) async fn get_payment_order_dossier_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Path(payment_order_id): Path<String>,
) -> Result<Json<AdminPaymentOrderDossier>, StatusCode> {
    let payment_store = payment_store_kernel(&state).map_err(|(status, _)| status)?;
    load_admin_payment_order_dossier(payment_store.as_ref(), &payment_order_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

pub(crate) async fn list_refund_orders_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Query(query): Query<RefundOrderListQuery>,
) -> Result<Json<Vec<RefundOrderRecord>>, StatusCode> {
    let payment_store = payment_store_kernel(&state).map_err(|(status, _)| status)?;
    let refund_status_filter = query
        .refund_status
        .as_deref()
        .map(RefundOrderStatus::from_str)
        .transpose()
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    load_admin_refund_orders(payment_store.as_ref(), refund_status_filter)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub(crate) async fn approve_refund_order_handler(
    claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Path(refund_order_id): Path<String>,
    Json(request): Json<ApproveRefundOrderRequest>,
) -> Result<Json<RefundOrderRecord>, StatusCode> {
    let payment_store = payment_store_kernel(&state).map_err(|(status, _)| status)?;
    let refund = approve_refund_order_request(
        payment_store.as_ref(),
        &refund_order_id,
        request.approved_amount_minor,
        request.approved_at_ms,
    )
    .await
    .map_err(map_refund_request_action_error)?;
    audit::record_admin_audit_event(
        &state,
        &claims,
        "payment_refund.approve",
        "refund_order",
        refund.refund_order_id.clone(),
        audit::APPROVAL_SCOPE_FINANCE_CONTROL,
    )
    .await?;
    Ok(Json(refund))
}

pub(crate) async fn cancel_refund_order_handler(
    claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Path(refund_order_id): Path<String>,
    Json(request): Json<CancelRefundOrderRequest>,
) -> Result<Json<RefundOrderRecord>, StatusCode> {
    let payment_store = payment_store_kernel(&state).map_err(|(status, _)| status)?;
    let refund = cancel_refund_order_request(
        payment_store.as_ref(),
        &refund_order_id,
        request.canceled_at_ms,
    )
    .await
    .map_err(map_refund_request_action_error)?;
    audit::record_admin_audit_event(
        &state,
        &claims,
        "payment_refund.cancel",
        "refund_order",
        refund.refund_order_id.clone(),
        audit::APPROVAL_SCOPE_FINANCE_CONTROL,
    )
    .await?;
    Ok(Json(refund))
}

pub(crate) async fn start_refund_order_handler(
    claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Path(refund_order_id): Path<String>,
    Json(request): Json<StartRefundOrderRequest>,
) -> Result<Json<RefundOrderRecord>, StatusCode> {
    let payment_store = payment_store_kernel(&state).map_err(|(status, _)| status)?;
    let refund = start_refund_order_execution(
        payment_store.as_ref(),
        &refund_order_id,
        request.started_at_ms,
    )
    .await
    .map_err(map_refund_request_action_error)?;
    audit::record_admin_audit_event(
        &state,
        &claims,
        "payment_refund.start",
        "refund_order",
        refund.refund_order_id.clone(),
        audit::APPROVAL_SCOPE_FINANCE_CONTROL,
    )
    .await?;
    Ok(Json(refund))
}

pub(crate) async fn list_payment_gateway_accounts_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Query(query): Query<PaymentGatewayAccountListQuery>,
) -> Result<Json<Vec<PaymentGatewayAccountRecord>>, StatusCode> {
    let payment_store = payment_store_kernel(&state).map_err(|(status, _)| status)?;
    load_admin_payment_gateway_accounts(payment_store.as_ref(), &query)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub(crate) async fn upsert_payment_gateway_account_handler(
    claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(request): Json<UpsertPaymentGatewayAccountRequest>,
) -> Result<(StatusCode, Json<PaymentGatewayAccountRecord>), StatusCode> {
    let payment_store = payment_store_kernel(&state).map_err(|(status, _)| status)?;
    let record = payment_gateway_account_record_from_request(request)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let record = payment_store
        .insert_payment_gateway_account_record(&record)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    audit::record_admin_audit_event(
        &state,
        &claims,
        "payment_gateway_account.upsert",
        "payment_gateway_account",
        record.gateway_account_id.clone(),
        audit::APPROVAL_SCOPE_FINANCE_CONTROL,
    )
    .await?;
    Ok((StatusCode::CREATED, Json(record)))
}

pub(crate) async fn list_payment_channel_policies_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Query(query): Query<PaymentChannelPolicyListQuery>,
) -> Result<Json<Vec<PaymentChannelPolicyRecord>>, StatusCode> {
    let payment_store = payment_store_kernel(&state).map_err(|(status, _)| status)?;
    load_admin_payment_channel_policies(payment_store.as_ref(), &query)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub(crate) async fn upsert_payment_channel_policy_handler(
    claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(request): Json<UpsertPaymentChannelPolicyRequest>,
) -> Result<(StatusCode, Json<PaymentChannelPolicyRecord>), StatusCode> {
    let payment_store = payment_store_kernel(&state).map_err(|(status, _)| status)?;
    let record =
        payment_channel_policy_record_from_request(request).map_err(|_| StatusCode::BAD_REQUEST)?;
    let record = payment_store
        .insert_payment_channel_policy_record(&record)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    audit::record_admin_audit_event(
        &state,
        &claims,
        "payment_channel_policy.upsert",
        "payment_channel_policy",
        record.channel_policy_id.clone(),
        audit::APPROVAL_SCOPE_FINANCE_CONTROL,
    )
    .await?;
    Ok((StatusCode::CREATED, Json(record)))
}

pub(crate) async fn list_payment_reconciliation_lines_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Query(query): Query<PaymentReconciliationListQuery>,
) -> Result<Json<Vec<ReconciliationMatchSummaryRecord>>, StatusCode> {
    let payment_store = payment_store_kernel(&state).map_err(|(status, _)| status)?;
    let lifecycle = PaymentReconciliationLifecycle::parse(query.lifecycle.as_deref())
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    load_admin_payment_reconciliation_lines(payment_store.as_ref(), lifecycle)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub(crate) async fn payment_reconciliation_summary_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<PaymentReconciliationSummaryResponse>, StatusCode> {
    let payment_store = payment_store_kernel(&state).map_err(|(status, _)| status)?;
    summarize_admin_payment_reconciliation(payment_store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub(crate) async fn resolve_payment_reconciliation_line_handler(
    claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Path(reconciliation_line_id): Path<String>,
    Json(request): Json<ResolveReconciliationLineRequest>,
) -> Result<Json<ReconciliationMatchSummaryRecord>, StatusCode> {
    let payment_store = payment_store_kernel(&state).map_err(|(status, _)| status)?;
    let line = resolve_admin_payment_reconciliation_line(
        payment_store.as_ref(),
        &reconciliation_line_id,
        request.resolved_at_ms.unwrap_or_else(unix_timestamp_ms),
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;
    audit::record_admin_audit_event(
        &state,
        &claims,
        "payment_reconciliation.resolve",
        "payment_reconciliation_line",
        line.reconciliation_line_id.clone(),
        audit::APPROVAL_SCOPE_FINANCE_CONTROL,
    )
    .await?;
    Ok(Json(line))
}

pub(crate) async fn render_admin_metrics_payload(
    metrics: &HttpMetricsRegistry,
    state: &AdminApiState,
) -> String {
    let mut output = metrics.render_prometheus();
    let Some(payment_store) = state.payment_store.as_ref() else {
        output.push_str(
            "# HELP sdkwork_payment_reconciliation_metrics_scrape_error Whether reconciliation metric aggregation failed\n",
        );
        output.push_str("# TYPE sdkwork_payment_reconciliation_metrics_scrape_error gauge\n");
        output.push_str(&format!(
            "sdkwork_payment_reconciliation_metrics_scrape_error{{service=\"{}\"}} 1\n",
            escape_prometheus_label_value(metrics.service())
        ));
        return output;
    };

    match summarize_admin_payment_reconciliation(payment_store.as_ref()).await {
        Ok(summary) => {
            output.push_str(&render_payment_reconciliation_prometheus(
                metrics.service(),
                &summary,
            ));
            output.push_str(
                "# HELP sdkwork_payment_reconciliation_metrics_scrape_error Whether reconciliation metric aggregation failed\n",
            );
            output.push_str("# TYPE sdkwork_payment_reconciliation_metrics_scrape_error gauge\n");
            output.push_str(&format!(
                "sdkwork_payment_reconciliation_metrics_scrape_error{{service=\"{}\"}} 0\n",
                escape_prometheus_label_value(metrics.service())
            ));
        }
        Err(_) => {
            output.push_str(
                "# HELP sdkwork_payment_reconciliation_metrics_scrape_error Whether reconciliation metric aggregation failed\n",
            );
            output.push_str("# TYPE sdkwork_payment_reconciliation_metrics_scrape_error gauge\n");
            output.push_str(&format!(
                "sdkwork_payment_reconciliation_metrics_scrape_error{{service=\"{}\"}} 1\n",
                escape_prometheus_label_value(metrics.service())
            ));
        }
    }

    output
}

fn payment_gateway_account_record_from_request(
    request: UpsertPaymentGatewayAccountRequest,
) -> anyhow::Result<PaymentGatewayAccountRecord> {
    let provider_code = parse_admin_payment_provider_code(&request.provider_code)?;
    ensure!(
        !matches!(provider_code, PaymentProviderCode::Unspecified),
        "provider_code must be a concrete payment provider"
    );
    ensure!(
        !request.gateway_account_id.trim().is_empty(),
        "gateway_account_id must not be empty"
    );
    ensure!(
        !request.environment.trim().is_empty(),
        "environment must not be empty"
    );
    ensure!(
        !request.merchant_id.trim().is_empty(),
        "merchant_id must not be empty"
    );
    let status = normalize_admin_payment_route_status(&request.status)?;
    let updated_at_ms = request.updated_at_ms.unwrap_or_else(unix_timestamp_ms);
    let created_at_ms = request.created_at_ms.unwrap_or(updated_at_ms);

    Ok(PaymentGatewayAccountRecord::new(
        request.gateway_account_id,
        request.tenant_id,
        request.organization_id,
        provider_code,
    )
    .with_environment(request.environment.trim())
    .with_merchant_id(request.merchant_id.trim())
    .with_app_id(request.app_id.trim())
    .with_status(status)
    .with_priority(request.priority)
    .with_created_at_ms(created_at_ms)
    .with_updated_at_ms(updated_at_ms))
}

fn payment_channel_policy_record_from_request(
    request: UpsertPaymentChannelPolicyRequest,
) -> anyhow::Result<PaymentChannelPolicyRecord> {
    let provider_code = parse_admin_payment_provider_code(&request.provider_code)?;
    ensure!(
        !matches!(provider_code, PaymentProviderCode::Unspecified),
        "provider_code must be a concrete payment provider"
    );
    ensure!(
        !request.channel_policy_id.trim().is_empty(),
        "channel_policy_id must not be empty"
    );
    ensure!(
        !request.method_code.trim().is_empty(),
        "method_code must not be empty"
    );
    let status = normalize_admin_payment_route_status(&request.status)?;
    let updated_at_ms = request.updated_at_ms.unwrap_or_else(unix_timestamp_ms);
    let created_at_ms = request.created_at_ms.unwrap_or(updated_at_ms);

    Ok(PaymentChannelPolicyRecord::new(
        request.channel_policy_id,
        request.tenant_id,
        request.organization_id,
        provider_code,
        request.method_code.trim(),
    )
    .with_scene_code(request.scene_code.trim())
    .with_country_code(request.country_code.trim())
    .with_currency_code(request.currency_code.trim())
    .with_client_kind(request.client_kind.trim())
    .with_priority(request.priority)
    .with_status(status)
    .with_created_at_ms(created_at_ms)
    .with_updated_at_ms(updated_at_ms))
}

fn parse_admin_payment_provider_code(raw: &str) -> anyhow::Result<PaymentProviderCode> {
    PaymentProviderCode::from_str(raw.trim()).map_err(anyhow::Error::msg)
}

fn normalize_admin_payment_route_status(raw: &str) -> anyhow::Result<String> {
    let normalized = raw.trim().to_ascii_lowercase();
    ensure!(
        matches!(normalized.as_str(), "active" | "inactive"),
        "unsupported payment route status"
    );
    Ok(normalized)
}

async fn load_admin_payment_orders(
    store: &dyn PaymentKernelStore,
) -> anyhow::Result<Vec<PaymentOrderRecord>> {
    let mut orders = store.list_payment_order_records().await?;
    orders.sort_by(|left, right| {
        right
            .created_at_ms
            .cmp(&left.created_at_ms)
            .then_with(|| right.payment_order_id.cmp(&left.payment_order_id))
    });
    Ok(orders)
}

async fn load_admin_refund_orders(
    store: &dyn PaymentKernelStore,
    refund_status_filter: Option<RefundOrderStatus>,
) -> anyhow::Result<Vec<RefundOrderRecord>> {
    let payment_orders = load_admin_payment_orders(store).await?;
    let mut refunds = Vec::new();
    for payment_order in &payment_orders {
        let mut order_refunds = store
            .list_refund_order_records_for_payment_order(&payment_order.payment_order_id)
            .await?;
        refunds.append(&mut order_refunds);
    }
    refunds.sort_by(|left, right| {
        right
            .created_at_ms
            .cmp(&left.created_at_ms)
            .then_with(|| right.refund_order_id.cmp(&left.refund_order_id))
    });
    if let Some(refund_status_filter) = refund_status_filter {
        refunds.retain(|refund| refund.refund_status == refund_status_filter);
    }
    Ok(refunds)
}

async fn load_admin_payment_gateway_accounts(
    store: &dyn PaymentKernelStore,
    query: &PaymentGatewayAccountListQuery,
) -> anyhow::Result<Vec<PaymentGatewayAccountRecord>> {
    let mut records = store.list_payment_gateway_account_records().await?;
    records.retain(|record| payment_gateway_account_matches_query(record, query));
    records.sort_by(compare_payment_gateway_accounts);
    Ok(records)
}

fn payment_gateway_account_matches_query(
    record: &PaymentGatewayAccountRecord,
    query: &PaymentGatewayAccountListQuery,
) -> bool {
    optional_string_filter_matches(
        record.provider_code.as_str(),
        query.provider_code.as_deref(),
    ) && optional_string_filter_matches(&record.status, query.status.as_deref())
        && optional_u64_filter_matches(record.tenant_id, query.tenant_id)
        && optional_u64_filter_matches(record.organization_id, query.organization_id)
}

fn compare_payment_gateway_accounts(
    left: &PaymentGatewayAccountRecord,
    right: &PaymentGatewayAccountRecord,
) -> std::cmp::Ordering {
    right
        .priority
        .cmp(&left.priority)
        .then_with(|| right.updated_at_ms.cmp(&left.updated_at_ms))
        .then_with(|| left.gateway_account_id.cmp(&right.gateway_account_id))
}

async fn load_admin_payment_channel_policies(
    store: &dyn PaymentKernelStore,
    query: &PaymentChannelPolicyListQuery,
) -> anyhow::Result<Vec<PaymentChannelPolicyRecord>> {
    let mut records = store.list_payment_channel_policy_records().await?;
    records.retain(|record| payment_channel_policy_matches_query(record, query));
    records.sort_by(compare_payment_channel_policies);
    Ok(records)
}

fn payment_channel_policy_matches_query(
    record: &PaymentChannelPolicyRecord,
    query: &PaymentChannelPolicyListQuery,
) -> bool {
    optional_string_filter_matches(
        record.provider_code.as_str(),
        query.provider_code.as_deref(),
    ) && optional_string_filter_matches(&record.status, query.status.as_deref())
        && optional_u64_filter_matches(record.tenant_id, query.tenant_id)
        && optional_u64_filter_matches(record.organization_id, query.organization_id)
        && optional_string_filter_matches(&record.scene_code, query.scene_code.as_deref())
        && optional_string_filter_matches(&record.currency_code, query.currency_code.as_deref())
        && optional_string_filter_matches(&record.client_kind, query.client_kind.as_deref())
}

fn compare_payment_channel_policies(
    left: &PaymentChannelPolicyRecord,
    right: &PaymentChannelPolicyRecord,
) -> std::cmp::Ordering {
    right
        .priority
        .cmp(&left.priority)
        .then_with(|| right.updated_at_ms.cmp(&left.updated_at_ms))
        .then_with(|| left.channel_policy_id.cmp(&right.channel_policy_id))
}

fn optional_string_filter_matches(value: &str, expected: Option<&str>) -> bool {
    expected
        .map(|expected| value.eq_ignore_ascii_case(expected.trim()))
        .unwrap_or(true)
}

fn optional_u64_filter_matches(value: u64, expected: Option<u64>) -> bool {
    expected.map(|expected| value == expected).unwrap_or(true)
}

async fn load_admin_payment_reconciliation_lines(
    store: &dyn PaymentKernelStore,
    lifecycle: PaymentReconciliationLifecycle,
) -> anyhow::Result<Vec<ReconciliationMatchSummaryRecord>> {
    let mut lines = store
        .list_all_reconciliation_match_summary_records()
        .await?;
    apply_payment_reconciliation_queue_view(&mut lines, lifecycle);
    Ok(lines)
}

fn apply_payment_reconciliation_queue_view(
    lines: &mut Vec<ReconciliationMatchSummaryRecord>,
    lifecycle: PaymentReconciliationLifecycle,
) {
    lines.retain(|line| lifecycle.matches(line));
    lines.sort_by(compare_payment_reconciliation_lines);
}

fn compare_payment_reconciliation_lines(
    left: &ReconciliationMatchSummaryRecord,
    right: &ReconciliationMatchSummaryRecord,
) -> std::cmp::Ordering {
    matches!(left.match_status, ReconciliationMatchStatus::Resolved)
        .cmp(&matches!(
            right.match_status,
            ReconciliationMatchStatus::Resolved
        ))
        .then_with(|| right.updated_at_ms.cmp(&left.updated_at_ms))
        .then_with(|| right.created_at_ms.cmp(&left.created_at_ms))
        .then_with(|| {
            right
                .reconciliation_line_id
                .cmp(&left.reconciliation_line_id)
        })
}

async fn summarize_admin_payment_reconciliation(
    store: &dyn PaymentKernelStore,
) -> anyhow::Result<PaymentReconciliationSummaryResponse> {
    let lines =
        load_admin_payment_reconciliation_lines(store, PaymentReconciliationLifecycle::All).await?;
    let total_count = lines.len();
    let latest_updated_at_ms = lines.iter().map(|line| line.updated_at_ms).max();
    let active_lines = lines
        .iter()
        .filter(|line| !matches!(line.match_status, ReconciliationMatchStatus::Resolved))
        .collect::<Vec<_>>();
    let active_count = active_lines.len();
    let resolved_count = total_count.saturating_sub(active_count);
    let oldest_active_created_at_ms = active_lines.iter().map(|line| line.created_at_ms).min();

    let mut breakdown = BTreeMap::<String, (usize, u64)>::new();
    for line in active_lines {
        let reason_code = line
            .reason_code
            .clone()
            .unwrap_or_else(|| "unknown".to_owned());
        let entry = breakdown
            .entry(reason_code)
            .or_insert((0usize, line.updated_at_ms));
        entry.0 += 1;
        entry.1 = entry.1.max(line.updated_at_ms);
    }

    let mut active_reason_breakdown = breakdown
        .into_iter()
        .map(|(reason_code, (count, latest_updated_at_ms))| {
            PaymentReconciliationReasonBreakdownItem {
                reason_code,
                count,
                latest_updated_at_ms,
            }
        })
        .collect::<Vec<_>>();
    active_reason_breakdown.sort_by(|left, right| {
        right
            .count
            .cmp(&left.count)
            .then_with(|| right.latest_updated_at_ms.cmp(&left.latest_updated_at_ms))
            .then_with(|| left.reason_code.cmp(&right.reason_code))
    });

    Ok(PaymentReconciliationSummaryResponse {
        total_count,
        active_count,
        resolved_count,
        latest_updated_at_ms,
        oldest_active_created_at_ms,
        active_reason_breakdown,
    })
}

fn render_payment_reconciliation_prometheus(
    service: &str,
    summary: &PaymentReconciliationSummaryResponse,
) -> String {
    let mut output = String::new();
    let service = escape_prometheus_label_value(service);

    output.push_str(
        "# HELP sdkwork_payment_reconciliation_total Total reconciliation lines observed\n",
    );
    output.push_str("# TYPE sdkwork_payment_reconciliation_total gauge\n");
    output.push_str(&format!(
        "sdkwork_payment_reconciliation_total{{service=\"{}\"}} {}\n",
        service, summary.total_count
    ));

    output.push_str(
        "# HELP sdkwork_payment_reconciliation_active_total Active reconciliation lines observed\n",
    );
    output.push_str("# TYPE sdkwork_payment_reconciliation_active_total gauge\n");
    output.push_str(&format!(
        "sdkwork_payment_reconciliation_active_total{{service=\"{}\"}} {}\n",
        service, summary.active_count
    ));

    output.push_str(
        "# HELP sdkwork_payment_reconciliation_resolved_total Resolved reconciliation lines observed\n",
    );
    output.push_str("# TYPE sdkwork_payment_reconciliation_resolved_total gauge\n");
    output.push_str(&format!(
        "sdkwork_payment_reconciliation_resolved_total{{service=\"{}\"}} {}\n",
        service, summary.resolved_count
    ));

    output.push_str(
        "# HELP sdkwork_payment_reconciliation_latest_updated_at_ms Latest reconciliation update timestamp in milliseconds\n",
    );
    output.push_str("# TYPE sdkwork_payment_reconciliation_latest_updated_at_ms gauge\n");
    if let Some(value) = summary.latest_updated_at_ms {
        output.push_str(&format!(
            "sdkwork_payment_reconciliation_latest_updated_at_ms{{service=\"{}\"}} {}\n",
            service, value
        ));
    }

    output.push_str(
        "# HELP sdkwork_payment_reconciliation_oldest_active_created_at_ms Oldest active reconciliation creation timestamp in milliseconds\n",
    );
    output.push_str("# TYPE sdkwork_payment_reconciliation_oldest_active_created_at_ms gauge\n");
    if let Some(value) = summary.oldest_active_created_at_ms {
        output.push_str(&format!(
            "sdkwork_payment_reconciliation_oldest_active_created_at_ms{{service=\"{}\"}} {}\n",
            service, value
        ));
    }

    output.push_str(
        "# HELP sdkwork_payment_reconciliation_active_reason_total Active reconciliation lines grouped by reason code\n",
    );
    output.push_str("# TYPE sdkwork_payment_reconciliation_active_reason_total gauge\n");
    for item in &summary.active_reason_breakdown {
        output.push_str(&format!(
            "sdkwork_payment_reconciliation_active_reason_total{{service=\"{}\",reason_code=\"{}\"}} {}\n",
            service,
            escape_prometheus_label_value(&item.reason_code),
            item.count
        ));
    }

    output
}

fn escape_prometheus_label_value(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

fn map_refund_request_action_error(error: anyhow::Error) -> StatusCode {
    let message = error.to_string();
    if message.contains("refund order not found") || message.contains("payment order not found") {
        StatusCode::NOT_FOUND
    } else {
        StatusCode::BAD_REQUEST
    }
}

async fn resolve_admin_payment_reconciliation_line(
    store: &dyn PaymentKernelStore,
    reconciliation_line_id: &str,
    resolved_at_ms: u64,
) -> anyhow::Result<Option<ReconciliationMatchSummaryRecord>> {
    resolve_payment_reconciliation_line_with_store(store, reconciliation_line_id, resolved_at_ms)
        .await
}

async fn resolve_payment_reconciliation_line_with_store<S>(
    store: &S,
    reconciliation_line_id: &str,
    resolved_at_ms: u64,
) -> anyhow::Result<Option<ReconciliationMatchSummaryRecord>>
where
    S: PaymentKernelStore + ?Sized,
{
    let Some(existing) = store
        .find_reconciliation_match_summary_record(reconciliation_line_id)
        .await?
    else {
        return Ok(None);
    };
    if matches!(existing.match_status, ReconciliationMatchStatus::Resolved) {
        return Ok(Some(existing));
    }

    let mut resolved = existing;
    resolved.match_status = ReconciliationMatchStatus::Resolved;
    resolved.updated_at_ms = resolved_at_ms.max(resolved.created_at_ms);
    store
        .insert_reconciliation_match_summary_record(&resolved)
        .await
        .map(Some)
}
