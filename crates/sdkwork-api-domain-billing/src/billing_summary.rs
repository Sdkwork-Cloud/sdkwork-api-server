use super::*;


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct LedgerEntry {
    pub project_id: String,
    pub units: u64,
    pub amount: f64,
}

impl LedgerEntry {
    pub fn new(project_id: impl Into<String>, units: u64, amount: f64) -> Self {
        Self {
            project_id: project_id.into(),
            units,
            amount,
        }
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, ToSchema,
    Default,
)]
#[serde(rename_all = "snake_case")]
pub enum BillingAccountingMode {
    #[default]
    PlatformCredit,
    Byok,
    Passthrough,
}

impl BillingAccountingMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PlatformCredit => "platform_credit",
            Self::Byok => "byok",
            Self::Passthrough => "passthrough",
        }
    }
}

impl FromStr for BillingAccountingMode {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "platform_credit" => Ok(Self::PlatformCredit),
            "byok" => Ok(Self::Byok),
            "passthrough" => Ok(Self::Passthrough),
            other => Err(format!("unknown billing accounting mode: {other}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct BillingEventRecord {
    pub event_id: String,
    pub tenant_id: String,
    pub project_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key_group_id: Option<String>,
    pub capability: String,
    pub route_key: String,
    pub usage_model: String,
    pub provider_id: String,
    pub accounting_mode: BillingAccountingMode,
    pub operation_kind: String,
    pub modality: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channel_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reference_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latency_ms: Option<u64>,
    #[serde(default)]
    pub units: u64,
    #[serde(default = "default_request_count")]
    pub request_count: u64,
    #[serde(default)]
    pub input_tokens: u64,
    #[serde(default)]
    pub output_tokens: u64,
    #[serde(default)]
    pub total_tokens: u64,
    #[serde(default)]
    pub cache_read_tokens: u64,
    #[serde(default)]
    pub cache_write_tokens: u64,
    #[serde(default)]
    pub image_count: u64,
    #[serde(default)]
    pub audio_seconds: f64,
    #[serde(default)]
    pub video_seconds: f64,
    #[serde(default)]
    pub music_seconds: f64,
    #[serde(default)]
    pub upstream_cost: f64,
    #[serde(default)]
    pub customer_charge: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub applied_routing_profile_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compiled_routing_snapshot_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fallback_reason: Option<String>,
    pub created_at_ms: u64,
}

impl BillingEventRecord {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        event_id: impl Into<String>,
        tenant_id: impl Into<String>,
        project_id: impl Into<String>,
        capability: impl Into<String>,
        route_key: impl Into<String>,
        usage_model: impl Into<String>,
        provider_id: impl Into<String>,
        accounting_mode: BillingAccountingMode,
        created_at_ms: u64,
    ) -> Self {
        Self {
            event_id: event_id.into(),
            tenant_id: tenant_id.into(),
            project_id: project_id.into(),
            api_key_group_id: None,
            capability: capability.into(),
            route_key: route_key.into(),
            usage_model: usage_model.into(),
            provider_id: provider_id.into(),
            accounting_mode,
            operation_kind: "request".to_owned(),
            modality: "text".to_owned(),
            api_key_hash: None,
            channel_id: None,
            reference_id: None,
            latency_ms: None,
            units: 0,
            request_count: default_request_count(),
            input_tokens: 0,
            output_tokens: 0,
            total_tokens: 0,
            cache_read_tokens: 0,
            cache_write_tokens: 0,
            image_count: 0,
            audio_seconds: 0.0,
            video_seconds: 0.0,
            music_seconds: 0.0,
            upstream_cost: 0.0,
            customer_charge: 0.0,
            applied_routing_profile_id: None,
            compiled_routing_snapshot_id: None,
            fallback_reason: None,
            created_at_ms,
        }
    }

    pub fn with_api_key_group_id(mut self, api_key_group_id: impl Into<String>) -> Self {
        self.api_key_group_id = Some(api_key_group_id.into());
        self
    }

    pub fn with_operation(
        mut self,
        operation_kind: impl Into<String>,
        modality: impl Into<String>,
    ) -> Self {
        self.operation_kind = operation_kind.into();
        self.modality = modality.into();
        self
    }

    pub fn with_request_facts(
        mut self,
        api_key_hash: Option<&str>,
        channel_id: Option<&str>,
        reference_id: Option<&str>,
        latency_ms: Option<u64>,
    ) -> Self {
        self.api_key_hash = api_key_hash.map(ToOwned::to_owned);
        self.channel_id = channel_id.map(ToOwned::to_owned);
        self.reference_id = reference_id.map(ToOwned::to_owned);
        self.latency_ms = latency_ms;
        self
    }

    pub fn with_units(mut self, units: u64) -> Self {
        self.units = units;
        self
    }

    pub fn with_request_count(mut self, request_count: u64) -> Self {
        self.request_count = request_count;
        self
    }

    pub fn with_token_usage(
        mut self,
        input_tokens: u64,
        output_tokens: u64,
        total_tokens: u64,
    ) -> Self {
        self.input_tokens = input_tokens;
        self.output_tokens = output_tokens;
        self.total_tokens = total_tokens;
        self
    }

    pub fn with_cache_token_usage(
        mut self,
        cache_read_tokens: u64,
        cache_write_tokens: u64,
    ) -> Self {
        self.cache_read_tokens = cache_read_tokens;
        self.cache_write_tokens = cache_write_tokens;
        self
    }

    pub fn with_media_usage(
        mut self,
        image_count: u64,
        audio_seconds: f64,
        video_seconds: f64,
        music_seconds: f64,
    ) -> Self {
        self.image_count = image_count;
        self.audio_seconds = audio_seconds;
        self.video_seconds = video_seconds;
        self.music_seconds = music_seconds;
        self
    }

    pub fn with_financials(mut self, upstream_cost: f64, customer_charge: f64) -> Self {
        self.upstream_cost = upstream_cost;
        self.customer_charge = customer_charge;
        self
    }

    pub fn with_routing_evidence(
        mut self,
        applied_routing_profile_id: Option<&str>,
        compiled_routing_snapshot_id: Option<&str>,
        fallback_reason: Option<&str>,
    ) -> Self {
        self.applied_routing_profile_id = applied_routing_profile_id.map(ToOwned::to_owned);
        self.compiled_routing_snapshot_id = compiled_routing_snapshot_id.map(ToOwned::to_owned);
        self.fallback_reason = fallback_reason.map(ToOwned::to_owned);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuotaPolicy {
    pub policy_id: String,
    pub project_id: String,
    pub max_units: u64,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

impl QuotaPolicy {
    pub fn new(
        policy_id: impl Into<String>,
        project_id: impl Into<String>,
        max_units: u64,
    ) -> Self {
        Self {
            policy_id: policy_id.into(),
            project_id: project_id.into(),
            max_units,
            enabled: true,
        }
    }

    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuotaCheckResult {
    pub allowed: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub policy_id: Option<String>,
    pub requested_units: u64,
    pub used_units: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit_units: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remaining_units: Option<u64>,
}

impl QuotaCheckResult {
    pub fn allowed_without_policy(requested_units: u64, used_units: u64) -> Self {
        Self {
            allowed: true,
            policy_id: None,
            requested_units,
            used_units,
            limit_units: None,
            remaining_units: None,
        }
    }

    pub fn from_policy(policy: &QuotaPolicy, used_units: u64, requested_units: u64) -> Self {
        let remaining_units = policy.max_units.saturating_sub(used_units);
        Self {
            allowed: used_units.saturating_add(requested_units) <= policy.max_units,
            policy_id: Some(policy.policy_id.clone()),
            requested_units,
            used_units,
            limit_units: Some(policy.max_units),
            remaining_units: Some(remaining_units),
        }
    }
}

fn default_enabled() -> bool {
    true
}

fn default_request_count() -> u64 {
    1
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct ProjectBillingSummary {
    pub project_id: String,
    pub entry_count: u64,
    pub used_units: u64,
    pub booked_amount: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quota_policy_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quota_limit_units: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remaining_units: Option<u64>,
    #[serde(default)]
    pub exhausted: bool,
}

impl ProjectBillingSummary {
    pub fn new(project_id: impl Into<String>) -> Self {
        Self {
            project_id: project_id.into(),
            entry_count: 0,
            used_units: 0,
            booked_amount: 0.0,
            quota_policy_id: None,
            quota_limit_units: None,
            remaining_units: None,
            exhausted: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct BillingSummary {
    pub total_entries: u64,
    pub project_count: u64,
    pub total_units: u64,
    pub total_amount: f64,
    pub active_quota_policy_count: u64,
    pub exhausted_project_count: u64,
    pub projects: Vec<ProjectBillingSummary>,
}

impl BillingSummary {
    pub fn empty() -> Self {
        Self {
            total_entries: 0,
            project_count: 0,
            total_units: 0,
            total_amount: 0.0,
            active_quota_policy_count: 0,
            exhausted_project_count: 0,
            projects: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct BillingEventProjectSummary {
    pub project_id: String,
    pub event_count: u64,
    pub request_count: u64,
    pub total_units: u64,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub total_tokens: u64,
    pub total_image_count: u64,
    pub total_audio_seconds: f64,
    pub total_video_seconds: f64,
    pub total_music_seconds: f64,
    pub total_upstream_cost: f64,
    pub total_customer_charge: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct BillingEventGroupSummary {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key_group_id: Option<String>,
    pub project_count: u64,
    pub event_count: u64,
    pub request_count: u64,
    pub total_upstream_cost: f64,
    pub total_customer_charge: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct BillingEventCapabilitySummary {
    pub capability: String,
    pub event_count: u64,
    pub request_count: u64,
    pub total_tokens: u64,
    pub image_count: u64,
    pub audio_seconds: f64,
    pub video_seconds: f64,
    pub music_seconds: f64,
    pub total_upstream_cost: f64,
    pub total_customer_charge: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct BillingEventAccountingModeSummary {
    pub accounting_mode: BillingAccountingMode,
    pub event_count: u64,
    pub request_count: u64,
    pub total_upstream_cost: f64,
    pub total_customer_charge: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct BillingEventSummary {
    pub total_events: u64,
    pub project_count: u64,
    pub group_count: u64,
    pub capability_count: u64,
    pub total_request_count: u64,
    pub total_units: u64,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub total_tokens: u64,
    pub total_image_count: u64,
    pub total_audio_seconds: f64,
    pub total_video_seconds: f64,
    pub total_music_seconds: f64,
    pub total_upstream_cost: f64,
    pub total_customer_charge: f64,
    pub projects: Vec<BillingEventProjectSummary>,
    pub groups: Vec<BillingEventGroupSummary>,
    pub capabilities: Vec<BillingEventCapabilitySummary>,
    pub accounting_modes: Vec<BillingEventAccountingModeSummary>,
}

impl BillingEventSummary {
    pub fn empty() -> Self {
        Self {
            total_events: 0,
            project_count: 0,
            group_count: 0,
            capability_count: 0,
            total_request_count: 0,
            total_units: 0,
            total_input_tokens: 0,
            total_output_tokens: 0,
            total_tokens: 0,
            total_image_count: 0,
            total_audio_seconds: 0.0,
            total_video_seconds: 0.0,
            total_music_seconds: 0.0,
            total_upstream_cost: 0.0,
            total_customer_charge: 0.0,
            projects: Vec::new(),
            groups: Vec::new(),
            capabilities: Vec::new(),
            accounting_modes: Vec::new(),
        }
    }
}
