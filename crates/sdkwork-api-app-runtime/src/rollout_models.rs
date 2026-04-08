use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateExtensionRuntimeRolloutRequest {
    pub scope: ConfiguredExtensionHostReloadScope,
    pub requested_extension_id: Option<String>,
    pub requested_instance_id: Option<String>,
    pub resolved_extension_id: Option<String>,
    pub timeout_secs: u64,
}

impl CreateExtensionRuntimeRolloutRequest {
    pub fn new(scope: ConfiguredExtensionHostReloadScope, timeout_secs: u64) -> Self {
        let (requested_extension_id, requested_instance_id, resolved_extension_id) =
            rollout_request_fields_from_scope(&scope);

        Self {
            scope,
            requested_extension_id,
            requested_instance_id,
            resolved_extension_id,
            timeout_secs,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtensionRuntimeRolloutDetails {
    pub rollout_id: String,
    pub status: String,
    pub scope: String,
    pub requested_extension_id: Option<String>,
    pub requested_instance_id: Option<String>,
    pub resolved_extension_id: Option<String>,
    pub created_by: String,
    pub created_at_ms: u64,
    pub deadline_at_ms: u64,
    pub participant_count: usize,
    pub participants: Vec<ExtensionRuntimeRolloutParticipantRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateStandaloneConfigRolloutRequest {
    pub requested_service_kind: Option<String>,
    pub timeout_secs: u64,
}

impl CreateStandaloneConfigRolloutRequest {
    pub fn new(requested_service_kind: Option<String>, timeout_secs: u64) -> Self {
        Self {
            requested_service_kind,
            timeout_secs,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StandaloneConfigRolloutDetails {
    pub rollout_id: String,
    pub status: String,
    pub requested_service_kind: Option<String>,
    pub created_by: String,
    pub created_at_ms: u64,
    pub deadline_at_ms: u64,
    pub participant_count: usize,
    pub participants: Vec<StandaloneConfigRolloutParticipantRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum StandaloneRuntimeReloadOutcome {
    NoChange { message: String },
    Applied { message: String },
    RestartRequired { message: String },
}

impl StandaloneRuntimeReloadOutcome {
    pub(crate) fn no_change() -> Self {
        Self::NoChange {
            message: "no effective config changes detected".to_owned(),
        }
    }

    pub(crate) fn applied(message: String) -> Self {
        Self::Applied { message }
    }

    pub(crate) fn restart_required(message: String) -> Self {
        Self::RestartRequired { message }
    }

    pub(crate) fn message(&self) -> &str {
        match self {
            Self::NoChange { message }
            | Self::Applied { message }
            | Self::RestartRequired { message } => message,
        }
    }

    pub(crate) fn requires_restart(&self) -> bool {
        matches!(self, Self::RestartRequired { .. })
    }
}
