use super::*;

const CONFIG_RELOAD_POLL_INTERVAL_SECS: u64 = 1;

async fn reload_standalone_runtime_config_pass(
    service_kind: StandaloneServiceKind,
    config_loader: &StandaloneConfigLoader,
    reload_handles: &StandaloneServiceReloadHandles,
    state: &mut StandaloneRuntimeState,
    force_reload: bool,
) -> Result<StandaloneRuntimeReloadOutcome> {
    let next_watch_state = config_loader.watch_state()?;
    if let Some(pending_restart) = state.pending_restart_required.as_ref() {
        if pending_restart.watch_state == next_watch_state {
            return Ok(StandaloneRuntimeReloadOutcome::restart_required(
                pending_restart.message.clone(),
            ));
        }
    }
    if !force_reload && state.previous_watch_state.as_ref() == Some(&next_watch_state) {
        return Ok(StandaloneRuntimeReloadOutcome::no_change());
    }

    let next_config = config_loader.reload()?;
    next_config.validate_security_posture()?;
    let restart_required_changes =
        restart_required_changed_fields(service_kind, &state.current_config, &next_config);
    let restart_required_message = (!restart_required_changes.is_empty()).then(|| {
        format!(
            "restart required for {}",
            restart_required_changes.join(", ")
        )
    });

    let next_dynamic = next_config.runtime_dynamic_config();
    let bind_changed = service_bind(service_kind, &state.current_config)
        != service_bind(service_kind, &next_config);
    let database_changed = state.current_config.database_url != next_config.database_url;
    let admin_jwt_changed = service_kind == StandaloneServiceKind::Admin
        && state.current_config.admin_jwt_signing_secret != next_config.admin_jwt_signing_secret;
    let portal_jwt_changed = service_kind == StandaloneServiceKind::Portal
        && state.current_config.portal_jwt_signing_secret != next_config.portal_jwt_signing_secret;
    let secret_manager_changed = secret_manager_config_changed(&state.current_config, &next_config);
    let dynamic_changed =
        service_runtime_dynamic_changed(service_kind, &state.current_dynamic, &next_dynamic);
    let pricing_lifecycle_sync_interval_changed = service_kind
        .supports_pricing_lifecycle_supervision()
        && state.current_dynamic.pricing_lifecycle_sync_interval_secs
            != next_dynamic.pricing_lifecycle_sync_interval_secs;

    if !database_changed
        && !admin_jwt_changed
        && !portal_jwt_changed
        && !secret_manager_changed
        && !bind_changed
        && !dynamic_changed
    {
        state.previous_watch_state = Some(next_watch_state.clone());
        update_pending_restart_required(
            state,
            next_watch_state,
            restart_required_message.as_deref(),
        );
        return Ok(match restart_required_message {
            Some(message) => StandaloneRuntimeReloadOutcome::restart_required(message),
            None => StandaloneRuntimeReloadOutcome::no_change(),
        });
    }

    let prepared_store_bundle = if database_changed {
        Some(build_admin_payment_store_handles_from_config(&next_config).await?)
    } else {
        None
    };

    let prepared_listener = if bind_changed {
        let listener = reload_handles.listener.as_ref().with_context(|| {
            format!(
                "runtime config reload failed because no listener handle is configured for bind change: service={} bind={}",
                service_kind.as_str(),
                service_bind(service_kind, &next_config),
            )
        })?;

        listener
            .prepare_rebind(service_bind(service_kind, &next_config))
            .await?
    } else {
        None
    };

    let prepared_secret_manager = if secret_manager_changed {
        let _ = reload_handles.secret_manager.as_ref().with_context(|| {
            format!(
                "runtime config reload failed because no secret manager handle is configured for secret-manager change: service={}",
                service_kind.as_str(),
            )
        })?;

        let next_secret_manager = build_secret_manager_from_config(&next_config);
        let validation_store = prepared_store_bundle
            .as_ref()
            .map(|handles| handles.admin_store.as_ref())
            .unwrap_or(state.current_store.as_ref());
        validate_secret_manager_for_store(validation_store, &next_secret_manager).await?;
        Some(next_secret_manager)
    } else {
        None
    };

    let extension_policy_changed = service_kind.supports_runtime_dynamic()
        && extension_runtime_policy_changed(&state.current_dynamic, &next_dynamic);
    if extension_policy_changed {
        let policy = extension_discovery_policy_from_config(&next_dynamic);
        reload_extension_host_with_policy(&policy)?;
    }

    if dynamic_changed {
        next_dynamic.apply_to_process_env();
    }

    if let Some(next_store_handles) = prepared_store_bundle {
        state.current_store = next_store_handles.admin_store.clone();
        reload_handles.store.replace(next_store_handles.admin_store);
        if let Some(live_gateway_commercial_billing) =
            reload_handles.gateway_commercial_billing.as_ref()
        {
            live_gateway_commercial_billing.replace(next_store_handles.gateway_commercial_billing);
        }
        if let Some(live_commercial_billing) = reload_handles.commercial_billing.as_ref() {
            live_commercial_billing.replace(next_store_handles.commercial_billing);
        }
        if let Some(live_payment_store) = reload_handles.payment_store.as_ref() {
            live_payment_store.replace(next_store_handles.payment_store);
        }
        if let Some(live_identity_store) = reload_handles.identity_store.as_ref() {
            live_identity_store.replace(next_store_handles.identity_store);
        }
    }

    if admin_jwt_changed {
        if let Some(live_jwt) = reload_handles.admin_jwt_signing_secret.as_ref() {
            live_jwt.replace(next_config.admin_jwt_signing_secret.clone());
        }
    }

    if portal_jwt_changed {
        if let Some(live_jwt) = reload_handles.portal_jwt_signing_secret.as_ref() {
            live_jwt.replace(next_config.portal_jwt_signing_secret.clone());
        }
    }

    if let Some(next_secret_manager) = prepared_secret_manager {
        if let Some(live_secret_manager) = reload_handles.secret_manager.as_ref() {
            live_secret_manager.replace(next_secret_manager);
        }
    }

    if let Some(prepared_listener) = prepared_listener {
        prepared_listener.activate();
    }

    if service_kind.supports_runtime_dynamic()
        && (database_changed
            || state.current_dynamic.runtime_snapshot_interval_secs
                != next_dynamic.runtime_snapshot_interval_secs)
    {
        state
            .snapshot_supervision
            .replace(start_provider_health_snapshot_supervision(
                state.current_store.clone(),
                next_dynamic.runtime_snapshot_interval_secs,
            ));
    }

    if service_kind.supports_runtime_dynamic()
        && (extension_policy_changed
            || state.current_dynamic.extension_hot_reload_interval_secs
                != next_dynamic.extension_hot_reload_interval_secs)
    {
        state.extension_hot_reload_supervision.replace(
            start_configured_extension_hot_reload_supervision(
                next_dynamic.extension_hot_reload_interval_secs,
            ),
        );
    }

    if pricing_lifecycle_sync_interval_changed {
        state
            .pricing_lifecycle_supervision
            .replace(start_service_pricing_lifecycle_supervision(
                service_kind,
                reload_handles.commercial_billing.clone(),
                next_dynamic.pricing_lifecycle_sync_interval_secs,
            ));
    }

    eprintln!(
        "runtime config reload applied: service={} bind_changed={} database_changed={} admin_jwt_changed={} portal_jwt_changed={} secret_manager_changed={} extension_policy_changed={} pricing_lifecycle_sync_interval_secs={} runtime_snapshot_interval_secs={} extension_hot_reload_interval_secs={} native_dynamic_shutdown_drain_timeout_ms={}",
        service_kind.as_str(),
        bind_changed,
        database_changed,
        admin_jwt_changed,
        portal_jwt_changed,
        secret_manager_changed,
        extension_policy_changed,
        next_dynamic.pricing_lifecycle_sync_interval_secs,
        next_dynamic.runtime_snapshot_interval_secs,
        next_dynamic.extension_hot_reload_interval_secs,
        next_dynamic.native_dynamic_shutdown_drain_timeout_ms
    );

    let applied_message = format!(
        "runtime config reload applied: bind_changed={bind_changed} database_changed={database_changed} admin_jwt_changed={admin_jwt_changed} portal_jwt_changed={portal_jwt_changed} secret_manager_changed={secret_manager_changed} extension_policy_changed={extension_policy_changed} pricing_lifecycle_sync_interval_changed={pricing_lifecycle_sync_interval_changed}"
    );
    let applied_config =
        merge_applied_service_config(service_kind, &state.current_config, &next_config);
    state.current_config = applied_config.clone();
    state.current_dynamic = applied_config.runtime_dynamic_config();
    state.previous_watch_state = Some(next_watch_state.clone());
    update_pending_restart_required(state, next_watch_state, restart_required_message.as_deref());

    Ok(match restart_required_message {
        Some(message) => StandaloneRuntimeReloadOutcome::restart_required(format!(
            "{applied_message}; {message}"
        )),
        None => StandaloneRuntimeReloadOutcome::applied(applied_message),
    })
}

async fn process_standalone_config_rollout_work(
    coordination_store: &dyn AdminStore,
    service_kind: StandaloneServiceKind,
    node_id: &str,
    config_loader: &StandaloneConfigLoader,
    reload_handles: &StandaloneServiceReloadHandles,
    state: &mut StandaloneRuntimeState,
) -> Result<()> {
    let participants = coordination_store
        .list_pending_standalone_config_rollout_participants_for_node(node_id)
        .await?;

    for participant in participants {
        if participant.service_kind != service_kind.as_str() {
            continue;
        }

        let Some(rollout) = coordination_store
            .find_standalone_config_rollout(&participant.rollout_id)
            .await?
        else {
            continue;
        };

        if rollout.deadline_at_ms <= unix_timestamp_ms() {
            continue;
        }

        let applying_at_ms = unix_timestamp_ms();
        if !coordination_store
            .transition_standalone_config_rollout_participant(
                &participant.rollout_id,
                node_id,
                "pending",
                "applying",
                None,
                applying_at_ms,
            )
            .await?
        {
            continue;
        }

        let completed_at_ms = unix_timestamp_ms();
        match reload_standalone_runtime_config_pass(
            service_kind,
            config_loader,
            reload_handles,
            state,
            true,
        )
        .await
        {
            Ok(outcome) => {
                let next_status = if outcome.requires_restart() {
                    "failed"
                } else {
                    "succeeded"
                };
                coordination_store
                    .transition_standalone_config_rollout_participant(
                        &participant.rollout_id,
                        node_id,
                        "applying",
                        next_status,
                        Some(outcome.message()),
                        completed_at_ms,
                    )
                    .await?;
            }
            Err(error) => {
                let message = error.to_string();
                coordination_store
                    .transition_standalone_config_rollout_participant(
                        &participant.rollout_id,
                        node_id,
                        "applying",
                        "failed",
                        Some(message.as_str()),
                        completed_at_ms,
                    )
                    .await?;
            }
        }

        return Ok(());
    }

    Ok(())
}

pub(crate) async fn resolve_active_standalone_config_rollout_nodes(
    store: &dyn AdminStore,
    requested_service_kind: &Option<String>,
    now_ms: u64,
) -> Result<Vec<ServiceRuntimeNodeRecord>> {
    let active_after_ms = now_ms.saturating_sub(STANDALONE_CONFIG_ROLLOUT_NODE_FRESHNESS_WINDOW_MS);

    Ok(store
        .list_service_runtime_nodes()
        .await?
        .into_iter()
        .filter(|node| matches!(node.service_kind.as_str(), "gateway" | "admin" | "portal"))
        .filter(|node| node.last_seen_at_ms >= active_after_ms)
        .filter(|node| match requested_service_kind.as_deref() {
            Some(requested_service_kind) => node.service_kind == requested_service_kind,
            None => true,
        })
        .collect())
}

pub(crate) fn build_standalone_config_rollout_details(
    rollout: StandaloneConfigRolloutRecord,
    participants: Vec<StandaloneConfigRolloutParticipantRecord>,
    now_ms: u64,
) -> StandaloneConfigRolloutDetails {
    let status = aggregate_standalone_config_rollout_status(&rollout, &participants, now_ms);

    StandaloneConfigRolloutDetails {
        rollout_id: rollout.rollout_id,
        status,
        requested_service_kind: rollout.requested_service_kind,
        created_by: rollout.created_by,
        created_at_ms: rollout.created_at_ms,
        deadline_at_ms: rollout.deadline_at_ms,
        participant_count: participants.len(),
        participants,
    }
}

fn aggregate_standalone_config_rollout_status(
    rollout: &StandaloneConfigRolloutRecord,
    participants: &[StandaloneConfigRolloutParticipantRecord],
    now_ms: u64,
) -> String {
    if !participants.is_empty()
        && participants
            .iter()
            .all(|participant| participant.status == "succeeded")
    {
        return "succeeded".to_owned();
    }

    if participants
        .iter()
        .any(|participant| participant.status == "failed")
    {
        return "failed".to_owned();
    }

    if rollout.deadline_at_ms <= now_ms
        && participants
            .iter()
            .any(|participant| matches!(participant.status.as_str(), "pending" | "applying"))
    {
        return "timed_out".to_owned();
    }

    if participants
        .iter()
        .any(|participant| participant.status == "applying")
    {
        return "applying".to_owned();
    }

    "pending".to_owned()
}

pub(crate) fn rollout_gateway_scope(
    rollout: &ExtensionRuntimeRolloutRecord,
) -> Result<ConfiguredExtensionHostReloadScope> {
    match rollout.scope.as_str() {
        "all" => Ok(ConfiguredExtensionHostReloadScope::All),
        "extension" => {
            let extension_id = rollout
                .resolved_extension_id
                .clone()
                .or_else(|| rollout.requested_extension_id.clone())
                .context("extension rollout is missing a resolved extension id")?;
            Ok(ConfiguredExtensionHostReloadScope::Extension { extension_id })
        }
        "instance" => {
            let instance_id = rollout
                .requested_instance_id
                .clone()
                .context("instance rollout is missing a requested instance id")?;
            Ok(ConfiguredExtensionHostReloadScope::Instance { instance_id })
        }
        other => anyhow::bail!("unsupported extension runtime rollout scope: {other}"),
    }
}

pub(crate) fn rollout_request_fields_from_scope(
    scope: &ConfiguredExtensionHostReloadScope,
) -> (Option<String>, Option<String>, Option<String>) {
    match scope {
        ConfiguredExtensionHostReloadScope::All => (None, None, None),
        ConfiguredExtensionHostReloadScope::Extension { extension_id } => {
            (Some(extension_id.clone()), None, Some(extension_id.clone()))
        }
        ConfiguredExtensionHostReloadScope::Instance { instance_id } => {
            (None, Some(instance_id.clone()), None)
        }
    }
}

pub(crate) fn rollout_scope_name(scope: &ConfiguredExtensionHostReloadScope) -> &'static str {
    match scope {
        ConfiguredExtensionHostReloadScope::All => "all",
        ConfiguredExtensionHostReloadScope::Extension { .. } => "extension",
        ConfiguredExtensionHostReloadScope::Instance { .. } => "instance",
    }
}

pub(crate) fn next_extension_runtime_rollout_id(now_ms: u64) -> String {
    let sequence = NEXT_EXTENSION_RUNTIME_ROLLOUT_ID.fetch_add(1, Ordering::SeqCst);
    format!("runtime-rollout-{now_ms}-{sequence}")
}

pub(crate) fn normalize_extension_runtime_rollout_timeout_secs(timeout_secs: u64) -> u64 {
    if timeout_secs == 0 {
        DEFAULT_EXTENSION_RUNTIME_ROLLOUT_TIMEOUT_SECS
    } else {
        timeout_secs
    }
}

pub(crate) fn next_standalone_config_rollout_id(now_ms: u64) -> String {
    let sequence = NEXT_STANDALONE_CONFIG_ROLLOUT_ID.fetch_add(1, Ordering::SeqCst);
    format!("config-rollout-{now_ms}-{sequence}")
}

pub(crate) fn normalize_standalone_config_rollout_timeout_secs(timeout_secs: u64) -> u64 {
    if timeout_secs == 0 {
        DEFAULT_STANDALONE_CONFIG_ROLLOUT_TIMEOUT_SECS
    } else {
        timeout_secs
    }
}

pub(crate) fn unix_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("unix time")
        .as_millis() as u64
}

pub fn start_standalone_runtime_supervision(
    service_kind: StandaloneServiceKind,
    config_loader: StandaloneConfigLoader,
    initial_config: StandaloneConfig,
    reload_handles: StandaloneServiceReloadHandles,
) -> StandaloneRuntimeSupervision {
    let initial_watch_state = match config_loader.watch_state() {
        Ok(state) => Some(state),
        Err(error) => {
            eprintln!("runtime config watch startup state capture failed: {error}");
            None
        }
    };
    let started_at_ms = unix_timestamp_ms();
    let coordination_store = reload_handles
        .coordination_store
        .clone()
        .unwrap_or_else(|| reload_handles.store.clone());

    StandaloneRuntimeSupervision {
        join_handle: tokio::spawn(async move {
            let current_dynamic = initial_config.runtime_dynamic_config();
            let current_store = reload_handles.store.snapshot();
            let snapshot_supervision =
                AbortOnDropHandle::new(if service_kind.supports_runtime_dynamic() {
                    start_provider_health_snapshot_supervision(
                        current_store.clone(),
                        current_dynamic.runtime_snapshot_interval_secs,
                    )
                } else {
                    None
                });
            let extension_hot_reload_supervision =
                AbortOnDropHandle::new(if service_kind.supports_runtime_dynamic() {
                    start_configured_extension_hot_reload_supervision(
                        current_dynamic.extension_hot_reload_interval_secs,
                    )
                } else {
                    None
                });
            let pricing_lifecycle_supervision =
                AbortOnDropHandle::new(start_service_pricing_lifecycle_supervision(
                    service_kind,
                    reload_handles.commercial_billing.clone(),
                    current_dynamic.pricing_lifecycle_sync_interval_secs,
                ));
            let mut state = StandaloneRuntimeState {
                current_config: initial_config,
                current_dynamic,
                current_store,
                snapshot_supervision,
                extension_hot_reload_supervision,
                pricing_lifecycle_supervision,
                previous_watch_state: initial_watch_state,
                pending_restart_required: None,
            };

            let mut interval =
                tokio::time::interval(Duration::from_secs(CONFIG_RELOAD_POLL_INTERVAL_SECS));
            interval.set_missed_tick_behavior(MissedTickBehavior::Delay);
            interval.tick().await;

            loop {
                interval.tick().await;

                if let Some(node_id) = reload_handles.node_id.as_deref() {
                    let coordination_store = coordination_store.snapshot();
                    let heartbeat = ServiceRuntimeNodeRecord::new(
                        node_id,
                        service_kind.as_str(),
                        started_at_ms,
                    )
                    .with_last_seen_at_ms(unix_timestamp_ms());
                    if let Err(error) = coordination_store
                        .upsert_service_runtime_node(&heartbeat)
                        .await
                    {
                        eprintln!(
                            "standalone config rollout heartbeat failed: service={} node_id={} error={error}",
                            service_kind.as_str(),
                            node_id,
                        );
                    }

                    if let Err(error) = process_standalone_config_rollout_work(
                        coordination_store.as_ref(),
                        service_kind,
                        node_id,
                        &config_loader,
                        &reload_handles,
                        &mut state,
                    )
                    .await
                    {
                        eprintln!(
                            "standalone config rollout processing failed: service={} node_id={} error={error}",
                            service_kind.as_str(),
                            node_id,
                        );
                    }
                }

                if let Err(error) = reload_standalone_runtime_config_pass(
                    service_kind,
                    &config_loader,
                    &reload_handles,
                    &mut state,
                    false,
                )
                .await
                {
                    eprintln!("runtime config reload failed: {error}");
                }
            }
        }),
    }
}

pub(crate) struct AbortOnDropHandle(Option<JoinHandle<()>>);

impl AbortOnDropHandle {
    fn new(handle: Option<JoinHandle<()>>) -> Self {
        Self(handle)
    }

    fn replace(&mut self, handle: Option<JoinHandle<()>>) {
        self.abort();
        self.0 = handle;
    }

    fn abort(&mut self) {
        if let Some(handle) = self.0.take() {
            handle.abort();
        }
    }
}

impl Drop for AbortOnDropHandle {
    fn drop(&mut self) {
        self.abort();
    }
}

fn start_service_pricing_lifecycle_supervision(
    service_kind: StandaloneServiceKind,
    live_commercial_billing: Option<Reloadable<Arc<dyn CommercialBillingAdminKernel>>>,
    interval_secs: u64,
) -> Option<JoinHandle<()>> {
    if !service_kind.supports_pricing_lifecycle_supervision() || interval_secs == 0 {
        return None;
    }

    let Some(live_commercial_billing) = live_commercial_billing else {
        eprintln!(
            "pricing lifecycle supervision disabled because no commercial billing handle is configured: service={} interval_secs={interval_secs}",
            service_kind.as_str(),
        );
        return None;
    };

    start_pricing_lifecycle_supervision(live_commercial_billing, interval_secs)
}

fn start_pricing_lifecycle_supervision(
    live_commercial_billing: Reloadable<Arc<dyn CommercialBillingAdminKernel>>,
    interval_secs: u64,
) -> Option<JoinHandle<()>> {
    if interval_secs == 0 {
        return None;
    }

    Some(tokio::spawn(async move {
        let synchronize_once = |billing: Arc<dyn CommercialBillingAdminKernel>,
                                stage: &'static str| async move {
            let report = synchronize_due_pricing_plan_lifecycle_with_report(
                billing.as_ref(),
                unix_timestamp_ms(),
            )
            .await;
            match report {
                Ok(report) if report.changed => {
                    eprintln!(
                        "pricing lifecycle synchronization applied: stage={} activated_plans={} archived_plans={} activated_rates={} archived_rates={}",
                        stage,
                        report.activated_plan_count,
                        report.archived_plan_count,
                        report.activated_rate_count,
                        report.archived_rate_count,
                    );
                }
                Ok(_) => {}
                Err(error) => {
                    eprintln!(
                        "pricing lifecycle synchronization failed: stage={} error={error}",
                        stage
                    );
                }
            }
        };

        synchronize_once(live_commercial_billing.snapshot(), "startup").await;

        let mut interval = tokio::time::interval(Duration::from_secs(interval_secs));
        interval.set_missed_tick_behavior(MissedTickBehavior::Delay);
        interval.tick().await;

        loop {
            interval.tick().await;
            synchronize_once(live_commercial_billing.snapshot(), "tick").await;
        }
    }))
}

pub(crate) fn restart_required_changed_fields(
    service_kind: StandaloneServiceKind,
    current: &StandaloneConfig,
    next: &StandaloneConfig,
) -> Vec<&'static str> {
    current
        .non_reloadable_changed_fields(next)
        .into_iter()
        .filter(|field| service_relevant_field(service_kind, field))
        .filter(|field| !service_reloadable_field(service_kind, field))
        .collect()
}

fn service_relevant_field(service_kind: StandaloneServiceKind, field: &str) -> bool {
    match field {
        "gateway_bind" => service_kind == StandaloneServiceKind::Gateway,
        "admin_bind" => service_kind == StandaloneServiceKind::Admin,
        "portal_bind" => service_kind == StandaloneServiceKind::Portal,
        "database_url" => true,
        "cache_backend" | "cache_url" => service_kind != StandaloneServiceKind::Portal,
        "admin_jwt_signing_secret" => service_kind == StandaloneServiceKind::Admin,
        "portal_jwt_signing_secret" => service_kind == StandaloneServiceKind::Portal,
        "metrics_bearer_token" => true,
        "browser_allowed_origins" => service_kind != StandaloneServiceKind::Admin,
        "pricing_lifecycle_sync_interval_secs" => {
            service_kind.supports_pricing_lifecycle_supervision()
        }
        "secret_backend"
        | "credential_master_key"
        | "credential_legacy_master_keys"
        | "secret_local_file"
        | "secret_keyring_service" => true,
        _ => false,
    }
}

fn service_reloadable_field(service_kind: StandaloneServiceKind, field: &str) -> bool {
    match field {
        "gateway_bind" => service_kind == StandaloneServiceKind::Gateway,
        "admin_bind" => service_kind == StandaloneServiceKind::Admin,
        "portal_bind" => service_kind == StandaloneServiceKind::Portal,
        "database_url" => true,
        "admin_jwt_signing_secret" => service_kind == StandaloneServiceKind::Admin,
        "portal_jwt_signing_secret" => service_kind == StandaloneServiceKind::Portal,
        "pricing_lifecycle_sync_interval_secs" => {
            service_kind.supports_pricing_lifecycle_supervision()
        }
        "secret_backend"
        | "credential_master_key"
        | "credential_legacy_master_keys"
        | "secret_local_file"
        | "secret_keyring_service" => true,
        _ => false,
    }
}

fn service_runtime_dynamic_changed(
    service_kind: StandaloneServiceKind,
    current: &StandaloneRuntimeDynamicConfig,
    next: &StandaloneRuntimeDynamicConfig,
) -> bool {
    if !service_kind.supports_runtime_dynamic() {
        return false;
    }

    current.extension_paths != next.extension_paths
        || current.enable_connector_extensions != next.enable_connector_extensions
        || current.enable_native_dynamic_extensions != next.enable_native_dynamic_extensions
        || current.extension_hot_reload_interval_secs != next.extension_hot_reload_interval_secs
        || current.extension_trusted_signers != next.extension_trusted_signers
        || current.require_signed_connector_extensions != next.require_signed_connector_extensions
        || current.require_signed_native_dynamic_extensions
            != next.require_signed_native_dynamic_extensions
        || current.native_dynamic_shutdown_drain_timeout_ms
            != next.native_dynamic_shutdown_drain_timeout_ms
        || current.runtime_snapshot_interval_secs != next.runtime_snapshot_interval_secs
        || (service_kind.supports_pricing_lifecycle_supervision()
            && current.pricing_lifecycle_sync_interval_secs
                != next.pricing_lifecycle_sync_interval_secs)
}

fn secret_manager_config_changed(current: &StandaloneConfig, next: &StandaloneConfig) -> bool {
    current.secret_backend != next.secret_backend
        || current.credential_master_key != next.credential_master_key
        || current.credential_legacy_master_keys != next.credential_legacy_master_keys
        || current.secret_local_file != next.secret_local_file
        || current.secret_keyring_service != next.secret_keyring_service
}

fn service_bind(service_kind: StandaloneServiceKind, config: &StandaloneConfig) -> &str {
    match service_kind {
        StandaloneServiceKind::Gateway => &config.gateway_bind,
        StandaloneServiceKind::Admin => &config.admin_bind,
        StandaloneServiceKind::Portal => &config.portal_bind,
    }
}

pub(crate) fn merge_applied_service_config(
    service_kind: StandaloneServiceKind,
    current: &StandaloneConfig,
    next: &StandaloneConfig,
) -> StandaloneConfig {
    let mut applied = current.clone();

    match service_kind {
        StandaloneServiceKind::Gateway => {
            applied.gateway_bind = next.gateway_bind.clone();
        }
        StandaloneServiceKind::Admin => {
            applied.admin_bind = next.admin_bind.clone();
            applied.admin_jwt_signing_secret = next.admin_jwt_signing_secret.clone();
        }
        StandaloneServiceKind::Portal => {
            applied.portal_bind = next.portal_bind.clone();
            applied.portal_jwt_signing_secret = next.portal_jwt_signing_secret.clone();
        }
    }

    applied.database_url = next.database_url.clone();

    if service_kind.supports_runtime_dynamic() {
        applied.extension_paths = next.extension_paths.clone();
        applied.enable_connector_extensions = next.enable_connector_extensions;
        applied.enable_native_dynamic_extensions = next.enable_native_dynamic_extensions;
        applied.extension_hot_reload_interval_secs = next.extension_hot_reload_interval_secs;
        applied.extension_trusted_signers = next.extension_trusted_signers.clone();
        applied.require_signed_connector_extensions = next.require_signed_connector_extensions;
        applied.require_signed_native_dynamic_extensions =
            next.require_signed_native_dynamic_extensions;
        applied.native_dynamic_shutdown_drain_timeout_ms =
            next.native_dynamic_shutdown_drain_timeout_ms;
        applied.runtime_snapshot_interval_secs = next.runtime_snapshot_interval_secs;
    }

    if service_kind.supports_pricing_lifecycle_supervision() {
        applied.pricing_lifecycle_sync_interval_secs = next.pricing_lifecycle_sync_interval_secs;
    }

    applied.secret_backend = next.secret_backend;
    applied.credential_master_key = next.credential_master_key.clone();
    applied.credential_legacy_master_keys = next.credential_legacy_master_keys.clone();
    applied.secret_local_file = next.secret_local_file.clone();
    applied.secret_keyring_service = next.secret_keyring_service.clone();

    applied
}

fn update_pending_restart_required(
    state: &mut StandaloneRuntimeState,
    watch_state: StandaloneConfigWatchState,
    message: Option<&str>,
) {
    match message {
        Some(message) => {
            eprintln!("runtime config reload requires restart: {message}");
            state.pending_restart_required = Some(PendingStandaloneRuntimeRestartRequired {
                watch_state,
                message: message.to_owned(),
            });
        }
        None => {
            state.pending_restart_required = None;
        }
    }
}

fn extension_runtime_policy_changed(
    current: &StandaloneRuntimeDynamicConfig,
    next: &StandaloneRuntimeDynamicConfig,
) -> bool {
    current.extension_paths != next.extension_paths
        || current.enable_connector_extensions != next.enable_connector_extensions
        || current.enable_native_dynamic_extensions != next.enable_native_dynamic_extensions
        || current.extension_trusted_signers != next.extension_trusted_signers
        || current.require_signed_connector_extensions != next.require_signed_connector_extensions
        || current.require_signed_native_dynamic_extensions
            != next.require_signed_native_dynamic_extensions
}

fn extension_discovery_policy_from_config(
    config: &StandaloneRuntimeDynamicConfig,
) -> ExtensionDiscoveryPolicy {
    let mut policy = ExtensionDiscoveryPolicy::new(
        config
            .extension_paths
            .iter()
            .map(PathBuf::from)
            .collect::<Vec<_>>(),
    )
    .with_connector_extensions(config.enable_connector_extensions)
    .with_native_dynamic_extensions(config.enable_native_dynamic_extensions)
    .with_required_signatures_for_connector_extensions(config.require_signed_connector_extensions)
    .with_required_signatures_for_native_dynamic_extensions(
        config.require_signed_native_dynamic_extensions,
    );
    for (publisher, public_key) in &config.extension_trusted_signers {
        policy = policy.with_trusted_signer(publisher.clone(), public_key.clone());
    }
    policy
}
