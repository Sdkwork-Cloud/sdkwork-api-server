use super::*;

const EXTENSION_RUNTIME_ROLLOUT_POLL_INTERVAL_MS: u64 = 250;
const EXTENSION_RUNTIME_ROLLOUT_NODE_FRESHNESS_WINDOW_MS: u64 = 15_000;
pub(crate) const DEFAULT_EXTENSION_RUNTIME_ROLLOUT_TIMEOUT_SECS: u64 = 30;
pub(crate) const STANDALONE_CONFIG_ROLLOUT_NODE_FRESHNESS_WINDOW_MS: u64 = 15_000;
pub(crate) const DEFAULT_STANDALONE_CONFIG_ROLLOUT_TIMEOUT_SECS: u64 = 30;

pub(crate) static NEXT_EXTENSION_RUNTIME_ROLLOUT_ID: AtomicU64 = AtomicU64::new(1);
pub(crate) static NEXT_STANDALONE_CONFIG_ROLLOUT_ID: AtomicU64 = AtomicU64::new(1);

pub async fn create_extension_runtime_rollout(
    store: &dyn AdminStore,
    created_by: &str,
    scope: ConfiguredExtensionHostReloadScope,
    timeout_secs: u64,
) -> Result<ExtensionRuntimeRolloutDetails> {
    create_extension_runtime_rollout_with_request(
        store,
        created_by,
        CreateExtensionRuntimeRolloutRequest::new(scope, timeout_secs),
    )
    .await
}

pub async fn create_extension_runtime_rollout_with_request(
    store: &dyn AdminStore,
    created_by: &str,
    request: CreateExtensionRuntimeRolloutRequest,
) -> Result<ExtensionRuntimeRolloutDetails> {
    let now_ms = unix_timestamp_ms();
    let active_nodes = resolve_active_extension_runtime_rollout_nodes(store, now_ms).await?;
    if active_nodes.is_empty() {
        anyhow::bail!("no active gateway or admin nodes available for extension runtime rollout");
    }

    let timeout_secs = normalize_extension_runtime_rollout_timeout_secs(request.timeout_secs);
    let rollout = ExtensionRuntimeRolloutRecord::new(
        next_extension_runtime_rollout_id(now_ms),
        rollout_scope_name(&request.scope),
        request.requested_extension_id,
        request.requested_instance_id,
        request.resolved_extension_id,
        created_by,
        now_ms,
        now_ms.saturating_add(timeout_secs.saturating_mul(1_000)),
    );
    store.insert_extension_runtime_rollout(&rollout).await?;

    let mut participants = Vec::with_capacity(active_nodes.len());
    for node in active_nodes {
        let participant = ExtensionRuntimeRolloutParticipantRecord::new(
            rollout.rollout_id.clone(),
            node.node_id,
            node.service_kind,
            "pending",
            now_ms,
        );
        store
            .insert_extension_runtime_rollout_participant(&participant)
            .await?;
        participants.push(participant);
    }

    Ok(build_extension_runtime_rollout_details(
        rollout,
        participants,
        now_ms,
    ))
}

pub async fn list_extension_runtime_rollouts(
    store: &dyn AdminStore,
) -> Result<Vec<ExtensionRuntimeRolloutDetails>> {
    let now_ms = unix_timestamp_ms();
    let rollouts = store.list_extension_runtime_rollouts().await?;
    let mut details = Vec::with_capacity(rollouts.len());

    for rollout in rollouts {
        let participants = store
            .list_extension_runtime_rollout_participants(&rollout.rollout_id)
            .await?;
        details.push(build_extension_runtime_rollout_details(
            rollout,
            participants,
            now_ms,
        ));
    }

    Ok(details)
}

pub async fn find_extension_runtime_rollout(
    store: &dyn AdminStore,
    rollout_id: &str,
) -> Result<Option<ExtensionRuntimeRolloutDetails>> {
    let Some(rollout) = store.find_extension_runtime_rollout(rollout_id).await? else {
        return Ok(None);
    };
    let participants = store
        .list_extension_runtime_rollout_participants(&rollout.rollout_id)
        .await?;

    Ok(Some(build_extension_runtime_rollout_details(
        rollout,
        participants,
        unix_timestamp_ms(),
    )))
}

pub async fn create_standalone_config_rollout(
    store: &dyn AdminStore,
    created_by: &str,
    request: CreateStandaloneConfigRolloutRequest,
) -> Result<StandaloneConfigRolloutDetails> {
    let now_ms = unix_timestamp_ms();
    let active_nodes = resolve_active_standalone_config_rollout_nodes(
        store,
        &request.requested_service_kind,
        now_ms,
    )
    .await?;
    if active_nodes.is_empty() {
        anyhow::bail!("no active standalone nodes available for standalone config rollout");
    }

    let timeout_secs = normalize_standalone_config_rollout_timeout_secs(request.timeout_secs);
    let rollout = StandaloneConfigRolloutRecord::new(
        next_standalone_config_rollout_id(now_ms),
        request.requested_service_kind,
        created_by,
        now_ms,
        now_ms.saturating_add(timeout_secs.saturating_mul(1_000)),
    );
    store.insert_standalone_config_rollout(&rollout).await?;

    let mut participants = Vec::with_capacity(active_nodes.len());
    for node in active_nodes {
        let participant = StandaloneConfigRolloutParticipantRecord::new(
            rollout.rollout_id.clone(),
            node.node_id,
            node.service_kind,
            "pending",
            now_ms,
        );
        store
            .insert_standalone_config_rollout_participant(&participant)
            .await?;
        participants.push(participant);
    }

    Ok(build_standalone_config_rollout_details(
        rollout,
        participants,
        now_ms,
    ))
}

pub async fn list_standalone_config_rollouts(
    store: &dyn AdminStore,
) -> Result<Vec<StandaloneConfigRolloutDetails>> {
    let now_ms = unix_timestamp_ms();
    let rollouts = store.list_standalone_config_rollouts().await?;
    let mut details = Vec::with_capacity(rollouts.len());

    for rollout in rollouts {
        let participants = store
            .list_standalone_config_rollout_participants(&rollout.rollout_id)
            .await?;
        details.push(build_standalone_config_rollout_details(
            rollout,
            participants,
            now_ms,
        ));
    }

    Ok(details)
}

pub async fn find_standalone_config_rollout(
    store: &dyn AdminStore,
    rollout_id: &str,
) -> Result<Option<StandaloneConfigRolloutDetails>> {
    let Some(rollout) = store.find_standalone_config_rollout(rollout_id).await? else {
        return Ok(None);
    };
    let participants = store
        .list_standalone_config_rollout_participants(&rollout.rollout_id)
        .await?;

    Ok(Some(build_standalone_config_rollout_details(
        rollout,
        participants,
        unix_timestamp_ms(),
    )))
}

pub fn start_extension_runtime_rollout_supervision(
    service_kind: StandaloneServiceKind,
    node_id: impl Into<String>,
    live_store: Reloadable<Arc<dyn AdminStore>>,
) -> Result<JoinHandle<()>> {
    if !service_kind.supports_runtime_dynamic() {
        anyhow::bail!(
            "standalone service does not participate in extension runtime rollouts: {}",
            service_kind.as_str()
        );
    }

    let node_id = node_id.into();
    let started_at_ms = unix_timestamp_ms();

    Ok(tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_millis(
            EXTENSION_RUNTIME_ROLLOUT_POLL_INTERVAL_MS,
        ));
        interval.set_missed_tick_behavior(MissedTickBehavior::Delay);

        loop {
            interval.tick().await;

            let store = live_store.snapshot();
            let heartbeat = ServiceRuntimeNodeRecord::new(
                node_id.clone(),
                service_kind.as_str(),
                started_at_ms,
            )
            .with_last_seen_at_ms(unix_timestamp_ms());
            if let Err(error) = store.upsert_service_runtime_node(&heartbeat).await {
                eprintln!(
                    "extension runtime rollout heartbeat failed: service={} node_id={} error={error}",
                    service_kind.as_str(),
                    node_id
                );
                continue;
            }

            if let Err(error) =
                process_extension_runtime_rollout_work(store.as_ref(), service_kind, &node_id).await
            {
                eprintln!(
                    "extension runtime rollout processing failed: service={} node_id={} error={error}",
                    service_kind.as_str(),
                    node_id
                );
            }
        }
    }))
}

pub fn resolve_service_runtime_node_id(service_kind: StandaloneServiceKind) -> String {
    if let Ok(node_id) = std::env::var("SDKWORK_SERVICE_INSTANCE_ID") {
        let node_id = node_id.trim();
        if !node_id.is_empty() {
            return node_id.to_owned();
        }
    }

    format!(
        "{}-{}-{}",
        service_kind.as_str(),
        std::process::id(),
        unix_timestamp_ms()
    )
}

async fn process_extension_runtime_rollout_work(
    store: &dyn AdminStore,
    service_kind: StandaloneServiceKind,
    node_id: &str,
) -> Result<()> {
    let participants = store
        .list_pending_extension_runtime_rollout_participants_for_node(node_id)
        .await?;

    for participant in participants {
        if participant.service_kind != service_kind.as_str() {
            continue;
        }

        let Some(rollout) = store
            .find_extension_runtime_rollout(&participant.rollout_id)
            .await?
        else {
            continue;
        };

        if rollout.deadline_at_ms <= unix_timestamp_ms() {
            continue;
        }

        let applying_at_ms = unix_timestamp_ms();
        if !store
            .transition_extension_runtime_rollout_participant(
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
        match apply_extension_runtime_rollout(&rollout) {
            Ok(()) => {
                store
                    .transition_extension_runtime_rollout_participant(
                        &participant.rollout_id,
                        node_id,
                        "applying",
                        "succeeded",
                        None,
                        completed_at_ms,
                    )
                    .await?;
            }
            Err(error) => {
                let message = error.to_string();
                store
                    .transition_extension_runtime_rollout_participant(
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

fn apply_extension_runtime_rollout(rollout: &ExtensionRuntimeRolloutRecord) -> Result<()> {
    let scope = rollout_gateway_scope(rollout)?;
    reload_extension_host_with_scope(&scope)
        .map(|_| ())
        .with_context(|| {
            format!(
                "failed to apply extension runtime rollout {}",
                rollout.rollout_id
            )
        })
}

async fn resolve_active_extension_runtime_rollout_nodes(
    store: &dyn AdminStore,
    now_ms: u64,
) -> Result<Vec<ServiceRuntimeNodeRecord>> {
    let active_after_ms = now_ms.saturating_sub(EXTENSION_RUNTIME_ROLLOUT_NODE_FRESHNESS_WINDOW_MS);

    Ok(store
        .list_service_runtime_nodes()
        .await?
        .into_iter()
        .filter(|node| matches!(node.service_kind.as_str(), "gateway" | "admin"))
        .filter(|node| node.last_seen_at_ms >= active_after_ms)
        .collect())
}

fn build_extension_runtime_rollout_details(
    rollout: ExtensionRuntimeRolloutRecord,
    participants: Vec<ExtensionRuntimeRolloutParticipantRecord>,
    now_ms: u64,
) -> ExtensionRuntimeRolloutDetails {
    let status = aggregate_extension_runtime_rollout_status(&rollout, &participants, now_ms);

    ExtensionRuntimeRolloutDetails {
        rollout_id: rollout.rollout_id,
        status,
        scope: rollout.scope,
        requested_extension_id: rollout.requested_extension_id,
        requested_instance_id: rollout.requested_instance_id,
        resolved_extension_id: rollout.resolved_extension_id,
        created_by: rollout.created_by,
        created_at_ms: rollout.created_at_ms,
        deadline_at_ms: rollout.deadline_at_ms,
        participant_count: participants.len(),
        participants,
    }
}

fn aggregate_extension_runtime_rollout_status(
    rollout: &ExtensionRuntimeRolloutRecord,
    participants: &[ExtensionRuntimeRolloutParticipantRecord],
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
