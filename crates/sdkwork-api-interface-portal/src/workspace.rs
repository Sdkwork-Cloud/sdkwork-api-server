use super::*;

#[derive(Debug, Serialize)]
pub(crate) struct PortalDashboardSummary {
    workspace: PortalWorkspaceSummary,
    usage_summary: UsageSummary,
    billing_summary: ProjectBillingSummary,
    recent_requests: Vec<UsageRecord>,
    api_key_count: usize,
}

#[derive(Debug, Serialize)]
pub(crate) struct PortalGatewayRateLimitSnapshot {
    project_id: String,
    policy_count: usize,
    active_policy_count: usize,
    window_count: usize,
    exceeded_window_count: usize,
    headline: String,
    detail: String,
    generated_at_ms: u64,
    policies: Vec<RateLimitPolicy>,
    windows: Vec<RateLimitWindowSnapshot>,
}


pub(crate) async fn workspace_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<PortalWorkspaceSummary>, StatusCode> {
    load_workspace_for_user(state.store.as_ref(), &claims.claims().sub)
        .await
        .map(Json)
}


pub(crate) async fn dashboard_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<PortalDashboardSummary>, StatusCode> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub).await?;
    let usage_records =
        load_project_usage_records(state.store.as_ref(), &workspace.project.id).await?;
    let usage_summary = summarize_usage_records(&usage_records);
    let billing_summary =
        load_project_billing_summary(state.store.as_ref(), &workspace.project.id).await?;
    let api_key_count = list_portal_api_keys(state.store.as_ref(), &claims.claims().sub)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .len();

    let recent_requests = usage_records.iter().take(10).cloned().collect();

    Ok(Json(PortalDashboardSummary {
        workspace,
        usage_summary,
        billing_summary,
        recent_requests,
        api_key_count,
    }))
}

pub(crate) async fn gateway_rate_limit_snapshot_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<PortalGatewayRateLimitSnapshot>, StatusCode> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub).await?;
    let policies = state
        .store
        .list_rate_limit_policies_for_project(&workspace.project.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let windows = state
        .store
        .list_rate_limit_window_snapshots_for_project(&workspace.project.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let active_policy_count = policies.iter().filter(|policy| policy.enabled).count();
    let window_count = windows.len();
    let exceeded_window_count = windows.iter().filter(|window| window.exceeded).count();
    let headline = if policies.is_empty() {
        "No rate-limit policies configured yet".to_owned()
    } else if exceeded_window_count > 0 {
        "Rate-limit pressure is visible on the current project".to_owned()
    } else if active_policy_count > 0 {
        "Rate-limit posture is within configured headroom".to_owned()
    } else {
        "Rate-limit policies exist but are currently disabled".to_owned()
    };
    let detail = if policies.is_empty() {
        "The workspace has no visible project-scoped rate-limit policy yet, so the gateway still relies on the default protection surface.".to_owned()
    } else if exceeded_window_count > 0 {
        format!(
            "{} window(s) are currently over limit across {} policy record(s), so the portal is surfacing the live pressure state instead of waiting for a later audit.",
            exceeded_window_count,
            policies.len()
        )
    } else {
        format!(
            "{} active policy record(s) and {} live window snapshot(s) are currently within the configured limit posture for project {}.",
            active_policy_count, window_count, workspace.project.id
        )
    };

    Ok(Json(PortalGatewayRateLimitSnapshot {
        project_id: workspace.project.id,
        policy_count: policies.len(),
        active_policy_count,
        window_count,
        exceeded_window_count,
        headline,
        detail,
        generated_at_ms: current_time_millis(),
        policies,
        windows,
    }))
}


pub(crate) async fn list_usage_records_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<Vec<UsageRecord>>, StatusCode> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub).await?;
    load_project_usage_records(state.store.as_ref(), &workspace.project.id)
        .await
        .map(Json)
}

pub(crate) async fn usage_summary_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<UsageSummary>, StatusCode> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub).await?;
    let usage_records =
        load_project_usage_records(state.store.as_ref(), &workspace.project.id).await?;
    Ok(Json(summarize_usage_records(&usage_records)))
}
