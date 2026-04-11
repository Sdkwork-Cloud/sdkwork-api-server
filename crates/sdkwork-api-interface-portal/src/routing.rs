use super::*;
use sdkwork_api_app_catalog::provider_integration_view;

#[derive(Debug, Deserialize)]
pub(crate) struct SaveRoutingPreferencesRequest {
    preset_id: String,
    strategy: RoutingStrategy,
    #[serde(default)]
    ordered_provider_ids: Vec<String>,
    #[serde(default)]
    default_provider_id: Option<String>,
    #[serde(default)]
    max_cost: Option<f64>,
    #[serde(default)]
    max_latency_ms: Option<u64>,
    #[serde(default)]
    require_healthy: bool,
    #[serde(default)]
    preferred_region: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct PortalRoutingPreviewRequest {
    capability: String,
    model: String,
    #[serde(default)]
    requested_region: Option<String>,
    #[serde(default)]
    selection_seed: Option<u64>,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Deserialize)]
pub(crate) struct CreatePortalRoutingProfileRequest {
    name: String,
    #[serde(default)]
    slug: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default = "default_true")]
    active: bool,
    #[serde(default)]
    strategy: Option<RoutingStrategy>,
    #[serde(default)]
    ordered_provider_ids: Vec<String>,
    #[serde(default)]
    default_provider_id: Option<String>,
    #[serde(default)]
    max_cost: Option<f64>,
    #[serde(default)]
    max_latency_ms: Option<u64>,
    #[serde(default)]
    require_healthy: bool,
    #[serde(default)]
    preferred_region: Option<String>,
}

#[derive(Debug, Serialize)]
struct PortalRoutingProviderOption {
    provider_id: String,
    display_name: String,
    channel_id: String,
    protocol_kind: String,
    integration: sdkwork_api_app_catalog::ProviderIntegrationView,
    credential_readiness: sdkwork_api_app_credential::ProviderCredentialReadinessView,
    #[serde(default)]
    preferred: bool,
    #[serde(default)]
    default_provider: bool,
}

#[derive(Debug, Serialize)]
pub(crate) struct PortalRoutingSummary {
    project_id: String,
    preferences: ProjectRoutingPreferences,
    latest_model_hint: String,
    preview: RoutingDecision,
    provider_options: Vec<PortalRoutingProviderOption>,
}


pub(crate) async fn get_routing_preferences_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<ProjectRoutingPreferences>, StatusCode> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub).await?;
    load_project_routing_preferences_or_default(state.store.as_ref(), &workspace.project.id)
        .await
        .map(Json)
}

pub(crate) async fn list_routing_profiles_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<Vec<RoutingProfileRecord>>, StatusCode> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub).await?;
    list_routing_profiles(state.store.as_ref())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
        .map(|profiles| {
            profiles
                .into_iter()
                .filter(|profile| {
                    profile.tenant_id == workspace.tenant.id
                        && profile.project_id == workspace.project.id
                })
                .collect::<Vec<_>>()
        })
        .map(Json)
}

pub(crate) async fn list_routing_snapshots_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<Vec<CompiledRoutingSnapshotRecord>>, StatusCode> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub).await?;
    list_compiled_routing_snapshots(state.store.as_ref())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
        .map(|snapshots| {
            snapshots
                .into_iter()
                .filter(|snapshot| {
                    snapshot.tenant_id.as_deref() == Some(workspace.tenant.id.as_str())
                        && snapshot.project_id.as_deref() == Some(workspace.project.id.as_str())
                })
                .collect::<Vec<_>>()
        })
        .map(Json)
}

pub(crate) async fn create_routing_profile_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
    Json(request): Json<CreatePortalRoutingProfileRequest>,
) -> Result<(StatusCode, Json<RoutingProfileRecord>), StatusCode> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub).await?;
    let normalized_name = normalize_portal_routing_profile_name(&request.name)?;
    let normalized_slug =
        normalize_portal_routing_profile_slug(&normalized_name, request.slug.as_deref())?;
    let profile_id = format!(
        "routing-profile-{}-{}",
        normalized_slug,
        current_time_millis()
    );

    let profile = create_routing_profile(CreateRoutingProfileInput {
        profile_id: &profile_id,
        tenant_id: &workspace.tenant.id,
        project_id: &workspace.project.id,
        name: &normalized_name,
        slug: &normalized_slug,
        description: request.description.as_deref(),
        active: request.active,
        strategy: request.strategy,
        ordered_provider_ids: &request.ordered_provider_ids,
        default_provider_id: request.default_provider_id.as_deref(),
        max_cost: request.max_cost,
        max_latency_ms: request.max_latency_ms,
        require_healthy: request.require_healthy,
        preferred_region: request.preferred_region.as_deref(),
    })
    .map_err(|_| StatusCode::BAD_REQUEST)?;
    let profile = persist_routing_profile(state.store.as_ref(), &profile)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok((StatusCode::CREATED, Json(profile)))
}

pub(crate) async fn save_routing_preferences_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
    Json(request): Json<SaveRoutingPreferencesRequest>,
) -> Result<Json<ProjectRoutingPreferences>, StatusCode> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub).await?;
    let preferences = ProjectRoutingPreferences::new(workspace.project.id.clone())
        .with_preset_id(request.preset_id)
        .with_strategy(request.strategy)
        .with_ordered_provider_ids(request.ordered_provider_ids)
        .with_default_provider_id_option(request.default_provider_id)
        .with_max_cost_option(request.max_cost)
        .with_max_latency_ms_option(request.max_latency_ms)
        .with_require_healthy(request.require_healthy)
        .with_preferred_region_option(request.preferred_region)
        .with_updated_at_ms(current_time_millis());

    state
        .store
        .insert_project_routing_preferences(&preferences)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub(crate) async fn preview_routing_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
    Json(request): Json<PortalRoutingPreviewRequest>,
) -> Result<Json<RoutingDecision>, StatusCode> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub).await?;
    select_route_with_store_context(
        state.store.as_ref(),
        &request.capability,
        &request.model,
        portal_route_selection_context(
            &workspace,
            RoutingDecisionSource::PortalSimulation,
            request.requested_region.as_deref(),
            request.selection_seed,
        ),
    )
    .await
    .map(Json)
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub(crate) async fn list_routing_decision_logs_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<Vec<RoutingDecisionLog>>, StatusCode> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub).await?;
    load_project_routing_decision_logs(state.store.as_ref(), &workspace.project.id)
        .await
        .map(Json)
}

pub(crate) async fn routing_summary_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<PortalRoutingSummary>, StatusCode> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub).await?;
    let preferences =
        load_project_routing_preferences_or_default(state.store.as_ref(), &workspace.project.id)
            .await?;
    let (latest_capability_hint, latest_model_hint) =
        load_latest_route_hint(state.store.as_ref(), &workspace.project.id).await?;
    let preview = simulate_route_with_store_selection_context(
        state.store.as_ref(),
        &latest_capability_hint,
        &latest_model_hint,
        portal_route_selection_context(
            &workspace,
            RoutingDecisionSource::PortalSimulation,
            None,
            None,
        ),
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let provider_options =
        load_routing_provider_options(
            state.store.as_ref(),
            &workspace.tenant.id,
            &latest_model_hint,
            &preferences,
        )
        .await?;

    Ok(Json(PortalRoutingSummary {
        project_id: workspace.project.id,
        preferences,
        latest_model_hint,
        preview,
        provider_options,
    }))
}


fn portal_route_selection_context<'a>(
    workspace: &'a PortalWorkspaceSummary,
    decision_source: RoutingDecisionSource,
    requested_region: Option<&'a str>,
    selection_seed: Option<u64>,
) -> RouteSelectionContext<'a> {
    RouteSelectionContext::new(decision_source)
        .with_tenant_id_option(Some(workspace.tenant.id.as_str()))
        .with_project_id_option(Some(workspace.project.id.as_str()))
        .with_requested_region_option(requested_region)
        .with_selection_seed_option(selection_seed)
}

async fn load_project_routing_preferences_or_default(
    store: &dyn AdminStore,
    project_id: &str,
) -> Result<ProjectRoutingPreferences, StatusCode> {
    store
        .find_project_routing_preferences(project_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .map(Ok)
        .unwrap_or_else(|| {
            Ok(ProjectRoutingPreferences::new(project_id.to_owned())
                .with_preset_id("platform_default"))
        })
}

async fn load_project_routing_decision_logs(
    store: &dyn AdminStore,
    project_id: &str,
) -> Result<Vec<RoutingDecisionLog>, StatusCode> {
    let logs = store
        .list_routing_decision_logs_for_project(project_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(logs)
}

async fn load_latest_route_hint(
    store: &dyn AdminStore,
    project_id: &str,
) -> Result<(String, String), StatusCode> {
    if let Some(log) = store
        .find_latest_routing_decision_log_for_project(project_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    {
        return Ok((log.capability.clone(), log.route_key.clone()));
    }

    if let Some(record) = store
        .find_latest_usage_record_for_project(project_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    {
        return Ok(("chat_completion".to_owned(), record.model.clone()));
    }

    if let Some(model) = store
        .find_any_model()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    {
        return Ok(("chat_completion".to_owned(), model.external_name.clone()));
    }

    Ok(("chat_completion".to_owned(), "gpt-4.1".to_owned()))
}

async fn load_routing_provider_options(
    store: &dyn AdminStore,
    tenant_id: &str,
    model: &str,
    preferences: &ProjectRoutingPreferences,
) -> Result<Vec<PortalRoutingProviderOption>, StatusCode> {
    let mut providers = store
        .list_providers_for_model(model)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .into_iter()
        .collect::<Vec<_>>();

    if providers.is_empty() {
        providers = store
            .list_providers()
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    let preference_ranks = provider_preference_ranks(preferences);
    let preferred_provider_ids = preferences
        .ordered_provider_ids
        .iter()
        .cloned()
        .collect::<HashSet<_>>();
    let configured_provider_ids =
        sdkwork_api_app_credential::list_configured_provider_ids_for_tenant(store, tenant_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    sort_routing_provider_options(&mut providers, &preference_ranks);

    Ok(providers
        .into_iter()
        .map(|provider| {
            let protocol_kind = provider.protocol_kind().to_owned();
            let integration = provider_integration_view(&provider);
            let credential_readiness =
                sdkwork_api_app_credential::provider_credential_readiness_view(
                    configured_provider_ids.contains(&provider.id),
                );
            PortalRoutingProviderOption {
                preferred: preferred_provider_ids.contains(&provider.id),
                default_provider: preferences.default_provider_id.as_deref() == Some(&provider.id),
                provider_id: provider.id,
                display_name: provider.display_name,
                channel_id: provider.channel_id,
                protocol_kind,
                integration,
                credential_readiness,
            }
        })
        .collect())
}

fn sort_routing_provider_options(
    providers: &mut [ProxyProvider],
    preference_ranks: &HashMap<String, usize>,
) {
    providers.sort_by(|left, right| {
        provider_preference_rank(preference_ranks, &left.id)
            .cmp(&provider_preference_rank(preference_ranks, &right.id))
            .then_with(|| left.display_name.cmp(&right.display_name))
            .then_with(|| left.id.cmp(&right.id))
    });
}

fn provider_preference_ranks(preferences: &ProjectRoutingPreferences) -> HashMap<String, usize> {
    preferences
        .ordered_provider_ids
        .iter()
        .enumerate()
        .map(|(index, provider_id)| (provider_id.clone(), index))
        .collect()
}

fn provider_preference_rank(preference_ranks: &HashMap<String, usize>, provider_id: &str) -> usize {
    preference_ranks
        .get(provider_id)
        .copied()
        .unwrap_or(usize::MAX)
}

fn normalize_portal_routing_profile_name(name: &str) -> Result<String, StatusCode> {
    let normalized = name.trim();
    if normalized.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    Ok(normalized.to_owned())
}

fn normalize_portal_routing_profile_slug(
    name: &str,
    slug: Option<&str>,
) -> Result<String, StatusCode> {
    let source = normalize_portal_routing_profile_optional_value(slug).unwrap_or(name.to_owned());
    let mut normalized = String::new();
    let mut previous_was_dash = false;

    for ch in source.chars() {
        if ch.is_ascii_alphanumeric() {
            normalized.push(ch.to_ascii_lowercase());
            previous_was_dash = false;
        } else if !normalized.is_empty() && !previous_was_dash {
            normalized.push('-');
            previous_was_dash = true;
        }
    }

    while normalized.ends_with('-') {
        normalized.pop();
    }

    if normalized.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    Ok(normalized)
}

fn normalize_portal_routing_profile_optional_value(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}
