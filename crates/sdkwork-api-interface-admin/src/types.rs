use super::*;

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct LoginRequest {
    pub(crate) email: String,
    pub(crate) password: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct LoginResponse {
    pub(crate) token: String,
    pub(crate) claims: Claims,
    pub(crate) user: AdminUserProfile,
}

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct ChangePasswordRequest {
    pub(crate) current_password: String,
    pub(crate) new_password: String,
}

fn default_user_active() -> bool {
    true
}

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct UpsertOperatorUserRequest {
    #[serde(default)]
    pub(crate) id: Option<String>,
    pub(crate) email: String,
    pub(crate) display_name: String,
    #[serde(default)]
    pub(crate) password: Option<String>,
    #[serde(default = "default_user_active")]
    pub(crate) active: bool,
}

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct UpsertPortalUserRequest {
    #[serde(default)]
    pub(crate) id: Option<String>,
    pub(crate) email: String,
    pub(crate) display_name: String,
    #[serde(default)]
    pub(crate) password: Option<String>,
    pub(crate) workspace_tenant_id: String,
    pub(crate) workspace_project_id: String,
    #[serde(default = "default_user_active")]
    pub(crate) active: bool,
}

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct UpdateUserStatusRequest {
    pub(crate) active: bool,
}

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct UpdateCouponTemplateStatusRequest {
    pub(crate) status: CouponTemplateStatus,
}

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct UpdateMarketingCampaignStatusRequest {
    pub(crate) status: MarketingCampaignStatus,
}

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct UpdateCampaignBudgetStatusRequest {
    pub(crate) status: CampaignBudgetStatus,
}

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct UpdateCouponCodeStatusRequest {
    pub(crate) status: CouponCodeStatus,
}

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct ResetUserPasswordRequest {
    pub(crate) new_password: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct ErrorResponse {
    pub(crate) error: ErrorBody,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct ErrorBody {
    pub(crate) message: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct CommercialAccountSummaryResponse {
    pub(crate) account: AccountRecord,
    pub(crate) available_balance: f64,
    pub(crate) held_balance: f64,
    pub(crate) consumed_balance: f64,
    pub(crate) grant_balance: f64,
    pub(crate) active_lot_count: u64,
}

impl CommercialAccountSummaryResponse {
    pub(crate) fn from_balance(account: AccountRecord, balance: &AccountBalanceSnapshot) -> Self {
        Self {
            account,
            available_balance: balance.available_balance,
            held_balance: balance.held_balance,
            consumed_balance: balance.consumed_balance,
            grant_balance: balance.grant_balance,
            active_lot_count: balance.active_lot_count,
        }
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct CreateChannelRequest {
    pub(crate) id: String,
    pub(crate) name: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct CreateProviderRequest {
    pub(crate) id: String,
    pub(crate) channel_id: String,
    #[serde(default)]
    pub(crate) extension_id: Option<String>,
    #[serde(default)]
    pub(crate) channel_bindings: Vec<CreateProviderChannelBindingRequest>,
    pub(crate) adapter_kind: String,
    pub(crate) base_url: String,
    pub(crate) display_name: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct CreateProviderChannelBindingRequest {
    pub(crate) channel_id: String,
    #[serde(default)]
    pub(crate) is_primary: bool,
}

#[derive(Debug, Deserialize)]
pub(crate) struct CreateCredentialRequest {
    pub(crate) tenant_id: String,
    pub(crate) provider_id: String,
    pub(crate) key_reference: String,
    pub(crate) secret_value: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct CreateModelRequest {
    pub(crate) external_name: String,
    pub(crate) provider_id: String,
    #[serde(default)]
    pub(crate) capabilities: Vec<ModelCapability>,
    #[serde(default)]
    pub(crate) streaming: bool,
    pub(crate) context_window: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct CreateChannelModelRequest {
    pub(crate) channel_id: String,
    pub(crate) model_id: String,
    pub(crate) model_display_name: String,
    #[serde(default)]
    pub(crate) capabilities: Vec<ModelCapability>,
    #[serde(default)]
    pub(crate) streaming: bool,
    #[serde(default)]
    pub(crate) context_window: Option<u64>,
    #[serde(default)]
    pub(crate) description: Option<String>,
}

fn default_currency_code() -> String {
    "USD".to_owned()
}

fn default_credit_unit_code() -> String {
    "credit".to_owned()
}

fn default_price_unit() -> String {
    "per_1m_tokens".to_owned()
}

fn default_charge_unit() -> String {
    "unit".to_owned()
}

fn default_pricing_method() -> String {
    "per_unit".to_owned()
}

fn default_rounding_increment() -> f64 {
    1.0
}

fn default_rounding_mode() -> String {
    "none".to_owned()
}

fn default_pricing_status() -> String {
    "draft".to_owned()
}

#[derive(Debug, Deserialize)]
pub(crate) struct CreateModelPriceRequest {
    pub(crate) channel_id: String,
    pub(crate) model_id: String,
    pub(crate) proxy_provider_id: String,
    #[serde(default = "default_currency_code")]
    pub(crate) currency_code: String,
    #[serde(default = "default_price_unit")]
    pub(crate) price_unit: String,
    #[serde(default)]
    pub(crate) input_price: f64,
    #[serde(default)]
    pub(crate) output_price: f64,
    #[serde(default)]
    pub(crate) cache_read_price: f64,
    #[serde(default)]
    pub(crate) cache_write_price: f64,
    #[serde(default)]
    pub(crate) request_price: f64,
    #[serde(default = "default_true")]
    pub(crate) is_active: bool,
}

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct CreateTenantRequest {
    pub(crate) id: String,
    pub(crate) name: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct CreateProjectRequest {
    pub(crate) tenant_id: String,
    pub(crate) id: String,
    pub(crate) name: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct CreateCouponRequest {
    pub(crate) id: String,
    pub(crate) code: String,
    pub(crate) discount_label: String,
    pub(crate) audience: String,
    pub(crate) remaining: u64,
    pub(crate) active: bool,
    pub(crate) note: String,
    pub(crate) expires_on: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct CreateApiKeyRequest {
    pub(crate) tenant_id: String,
    pub(crate) project_id: String,
    pub(crate) environment: String,
    #[serde(default)]
    pub(crate) label: Option<String>,
    #[serde(default)]
    pub(crate) notes: Option<String>,
    #[serde(default)]
    pub(crate) expires_at_ms: Option<u64>,
    #[serde(default)]
    pub(crate) plaintext_key: Option<String>,
    #[serde(default)]
    pub(crate) api_key_group_id: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct UpdateApiKeyRequest {
    pub(crate) tenant_id: String,
    pub(crate) project_id: String,
    pub(crate) environment: String,
    pub(crate) label: String,
    #[serde(default)]
    pub(crate) notes: Option<String>,
    #[serde(default)]
    pub(crate) expires_at_ms: Option<u64>,
    #[serde(default)]
    pub(crate) api_key_group_id: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct CreateApiKeyGroupRequest {
    pub(crate) tenant_id: String,
    pub(crate) project_id: String,
    pub(crate) environment: String,
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) slug: Option<String>,
    #[serde(default)]
    pub(crate) description: Option<String>,
    #[serde(default)]
    pub(crate) color: Option<String>,
    #[serde(default)]
    pub(crate) default_capability_scope: Option<String>,
    #[serde(default)]
    pub(crate) default_accounting_mode: Option<String>,
    #[serde(default)]
    pub(crate) default_routing_profile_id: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct UpdateApiKeyGroupRequest {
    pub(crate) tenant_id: String,
    pub(crate) project_id: String,
    pub(crate) environment: String,
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) slug: Option<String>,
    #[serde(default)]
    pub(crate) description: Option<String>,
    #[serde(default)]
    pub(crate) color: Option<String>,
    #[serde(default)]
    pub(crate) default_capability_scope: Option<String>,
    #[serde(default)]
    pub(crate) default_accounting_mode: Option<String>,
    #[serde(default)]
    pub(crate) default_routing_profile_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct CreateExtensionInstallationRequest {
    pub(crate) installation_id: String,
    pub(crate) extension_id: String,
    pub(crate) runtime: ExtensionRuntime,
    pub(crate) enabled: bool,
    pub(crate) entrypoint: Option<String>,
    #[serde(default)]
    pub(crate) config: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub(crate) struct CreateExtensionInstanceRequest {
    pub(crate) instance_id: String,
    pub(crate) installation_id: String,
    pub(crate) extension_id: String,
    pub(crate) enabled: bool,
    pub(crate) base_url: Option<String>,
    pub(crate) credential_ref: Option<String>,
    #[serde(default)]
    pub(crate) config: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub(crate) struct CreateRoutingPolicyRequest {
    pub(crate) policy_id: String,
    pub(crate) capability: String,
    pub(crate) model_pattern: String,
    #[serde(default = "default_true")]
    pub(crate) enabled: bool,
    #[serde(default)]
    pub(crate) priority: i32,
    #[serde(default)]
    pub(crate) strategy: Option<RoutingStrategy>,
    #[serde(default)]
    pub(crate) ordered_provider_ids: Vec<String>,
    #[serde(default)]
    pub(crate) default_provider_id: Option<String>,
    #[serde(default)]
    pub(crate) max_cost: Option<f64>,
    #[serde(default)]
    pub(crate) max_latency_ms: Option<u64>,
    #[serde(default)]
    pub(crate) require_healthy: bool,
    #[serde(default = "default_true")]
    pub(crate) execution_failover_enabled: bool,
    #[serde(default)]
    pub(crate) upstream_retry_max_attempts: Option<u32>,
    #[serde(default)]
    pub(crate) upstream_retry_base_delay_ms: Option<u64>,
    #[serde(default)]
    pub(crate) upstream_retry_max_delay_ms: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct CreateRoutingProfileRequest {
    pub(crate) profile_id: String,
    pub(crate) tenant_id: String,
    pub(crate) project_id: String,
    pub(crate) name: String,
    pub(crate) slug: String,
    #[serde(default)]
    pub(crate) description: Option<String>,
    #[serde(default = "default_true")]
    pub(crate) active: bool,
    #[serde(default)]
    pub(crate) strategy: Option<RoutingStrategy>,
    #[serde(default)]
    pub(crate) ordered_provider_ids: Vec<String>,
    #[serde(default)]
    pub(crate) default_provider_id: Option<String>,
    #[serde(default)]
    pub(crate) max_cost: Option<f64>,
    #[serde(default)]
    pub(crate) max_latency_ms: Option<u64>,
    #[serde(default)]
    pub(crate) require_healthy: bool,
    #[serde(default)]
    pub(crate) preferred_region: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct CreateQuotaPolicyRequest {
    pub(crate) policy_id: String,
    pub(crate) project_id: String,
    pub(crate) max_units: u64,
    #[serde(default = "default_true")]
    pub(crate) enabled: bool,
}

#[derive(Debug, Deserialize)]
pub(crate) struct CreateRateLimitPolicyRequest {
    pub(crate) policy_id: String,
    pub(crate) project_id: String,
    pub(crate) requests_per_window: u64,
    #[serde(default = "default_window_seconds")]
    pub(crate) window_seconds: u64,
    #[serde(default)]
    pub(crate) burst_requests: u64,
    #[serde(default = "default_true")]
    pub(crate) enabled: bool,
    #[serde(default)]
    pub(crate) route_key: Option<String>,
    #[serde(default)]
    pub(crate) api_key_hash: Option<String>,
    #[serde(default)]
    pub(crate) model_name: Option<String>,
    #[serde(default)]
    pub(crate) notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct CreateCommercialPricingPlanRequest {
    pub(crate) tenant_id: u64,
    #[serde(default)]
    pub(crate) organization_id: u64,
    pub(crate) plan_code: String,
    pub(crate) plan_version: u64,
    pub(crate) display_name: String,
    #[serde(default = "default_currency_code")]
    pub(crate) currency_code: String,
    #[serde(default = "default_credit_unit_code")]
    pub(crate) credit_unit_code: String,
    #[serde(default = "default_pricing_status")]
    pub(crate) status: String,
    #[serde(default)]
    pub(crate) effective_from_ms: u64,
    #[serde(default)]
    pub(crate) effective_to_ms: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct CloneCommercialPricingPlanRequest {
    #[serde(default)]
    pub(crate) plan_version: Option<u64>,
    #[serde(default)]
    pub(crate) display_name: Option<String>,
    #[serde(default = "default_pricing_status")]
    pub(crate) status: String,
}

#[derive(Debug, Deserialize, Default)]
pub(crate) struct PublishCommercialPricingPlanRequest {}

#[derive(Debug, Deserialize, Default)]
pub(crate) struct ScheduleCommercialPricingPlanRequest {}

#[derive(Debug, Deserialize, Default)]
pub(crate) struct RetireCommercialPricingPlanRequest {}

#[derive(Debug, Deserialize)]
pub(crate) struct CreateCommercialPricingRateRequest {
    pub(crate) tenant_id: u64,
    #[serde(default)]
    pub(crate) organization_id: u64,
    pub(crate) pricing_plan_id: u64,
    pub(crate) metric_code: String,
    pub(crate) capability_code: Option<String>,
    pub(crate) model_code: Option<String>,
    pub(crate) provider_code: Option<String>,
    #[serde(default = "default_charge_unit")]
    pub(crate) charge_unit: String,
    #[serde(default = "default_pricing_method")]
    pub(crate) pricing_method: String,
    #[serde(default = "default_rounding_increment")]
    pub(crate) quantity_step: f64,
    #[serde(default)]
    pub(crate) unit_price: f64,
    #[serde(default)]
    pub(crate) display_price_unit: String,
    #[serde(default)]
    pub(crate) minimum_billable_quantity: f64,
    #[serde(default)]
    pub(crate) minimum_charge: f64,
    #[serde(default = "default_rounding_increment")]
    pub(crate) rounding_increment: f64,
    #[serde(default = "default_rounding_mode")]
    pub(crate) rounding_mode: String,
    #[serde(default)]
    pub(crate) included_quantity: f64,
    #[serde(default)]
    pub(crate) priority: u64,
    pub(crate) notes: Option<String>,
    #[serde(default = "default_pricing_status")]
    pub(crate) status: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RoutingSimulationRequest {
    pub(crate) capability: String,
    pub(crate) model: String,
    #[serde(default)]
    pub(crate) tenant_id: Option<String>,
    #[serde(default)]
    pub(crate) project_id: Option<String>,
    #[serde(default)]
    pub(crate) api_key_group_id: Option<String>,
    #[serde(default)]
    pub(crate) requested_region: Option<String>,
    #[serde(default)]
    pub(crate) selection_seed: Option<u64>,
}

#[derive(Debug, Serialize)]
pub(crate) struct RoutingSimulationResponse {
    pub(crate) selected_provider_id: String,
    pub(crate) candidate_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) matched_policy_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) applied_routing_profile_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) compiled_routing_snapshot_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) strategy: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) selection_seed: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) selection_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) fallback_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) requested_region: Option<String>,
    #[serde(default)]
    pub(crate) slo_applied: bool,
    #[serde(default)]
    pub(crate) slo_degraded: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) selected_candidate: Option<RoutingCandidateAssessment>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) rejected_candidates: Vec<RoutingCandidateAssessment>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) assessments: Vec<RoutingCandidateAssessment>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ExtensionRuntimeReloadScope {
    All,
    Extension,
    Instance,
}

#[derive(Debug, Deserialize, Default)]
pub(crate) struct ExtensionRuntimeReloadRequest {
    #[serde(default)]
    pub(crate) extension_id: Option<String>,
    #[serde(default)]
    pub(crate) instance_id: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub(crate) struct ExtensionRuntimeRolloutCreateRequest {
    #[serde(default)]
    pub(crate) extension_id: Option<String>,
    #[serde(default)]
    pub(crate) instance_id: Option<String>,
    #[serde(default)]
    pub(crate) timeout_secs: Option<u64>,
}

#[derive(Debug, Serialize)]
pub(crate) struct ExtensionRuntimeReloadResponse {
    pub(crate) scope: ExtensionRuntimeReloadScope,
    pub(crate) requested_extension_id: Option<String>,
    pub(crate) requested_instance_id: Option<String>,
    pub(crate) resolved_extension_id: Option<String>,
    pub(crate) discovered_package_count: usize,
    pub(crate) loadable_package_count: usize,
    pub(crate) active_runtime_count: usize,
    pub(crate) reloaded_at_ms: u64,
    pub(crate) runtime_statuses: Vec<sdkwork_api_app_extension::ExtensionRuntimeStatusRecord>,
}

pub(crate) struct ResolvedExtensionRuntimeReloadRequest {
    pub(crate) scope: ExtensionRuntimeReloadScope,
    pub(crate) requested_extension_id: Option<String>,
    pub(crate) requested_instance_id: Option<String>,
    pub(crate) resolved_extension_id: Option<String>,
    pub(crate) gateway_scope: ConfiguredExtensionHostReloadScope,
}

#[derive(Debug, Serialize)]
pub(crate) struct ExtensionRuntimeRolloutParticipantResponse {
    pub(crate) node_id: String,
    pub(crate) service_kind: String,
    pub(crate) status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) message: Option<String>,
    pub(crate) updated_at_ms: u64,
}

#[derive(Debug, Serialize)]
pub(crate) struct ExtensionRuntimeRolloutResponse {
    pub(crate) rollout_id: String,
    pub(crate) status: String,
    pub(crate) scope: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) requested_extension_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) requested_instance_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) resolved_extension_id: Option<String>,
    pub(crate) created_by: String,
    pub(crate) created_at_ms: u64,
    pub(crate) deadline_at_ms: u64,
    pub(crate) participant_count: usize,
    pub(crate) participants: Vec<ExtensionRuntimeRolloutParticipantResponse>,
}

impl From<ExtensionRuntimeRolloutDetails> for ExtensionRuntimeRolloutResponse {
    fn from(value: ExtensionRuntimeRolloutDetails) -> Self {
        Self {
            rollout_id: value.rollout_id,
            status: value.status,
            scope: value.scope,
            requested_extension_id: value.requested_extension_id,
            requested_instance_id: value.requested_instance_id,
            resolved_extension_id: value.resolved_extension_id,
            created_by: value.created_by,
            created_at_ms: value.created_at_ms,
            deadline_at_ms: value.deadline_at_ms,
            participant_count: value.participant_count,
            participants: value
                .participants
                .into_iter()
                .map(|participant| ExtensionRuntimeRolloutParticipantResponse {
                    node_id: participant.node_id,
                    service_kind: participant.service_kind,
                    status: participant.status,
                    message: participant.message,
                    updated_at_ms: participant.updated_at_ms,
                })
                .collect(),
        }
    }
}

#[derive(Debug, Deserialize, Default)]
pub(crate) struct StandaloneConfigRolloutCreateRequest {
    #[serde(default)]
    pub(crate) service_kind: Option<String>,
    #[serde(default)]
    pub(crate) timeout_secs: Option<u64>,
}

#[derive(Debug, Serialize)]
pub(crate) struct StandaloneConfigRolloutParticipantResponse {
    pub(crate) node_id: String,
    pub(crate) service_kind: String,
    pub(crate) status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) message: Option<String>,
    pub(crate) updated_at_ms: u64,
}

#[derive(Debug, Serialize)]
pub(crate) struct StandaloneConfigRolloutResponse {
    pub(crate) rollout_id: String,
    pub(crate) status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) requested_service_kind: Option<String>,
    pub(crate) created_by: String,
    pub(crate) created_at_ms: u64,
    pub(crate) deadline_at_ms: u64,
    pub(crate) participant_count: usize,
    pub(crate) participants: Vec<StandaloneConfigRolloutParticipantResponse>,
}

impl From<StandaloneConfigRolloutDetails> for StandaloneConfigRolloutResponse {
    fn from(value: StandaloneConfigRolloutDetails) -> Self {
        Self {
            rollout_id: value.rollout_id,
            status: value.status,
            requested_service_kind: value.requested_service_kind,
            created_by: value.created_by,
            created_at_ms: value.created_at_ms,
            deadline_at_ms: value.deadline_at_ms,
            participant_count: value.participant_count,
            participants: value
                .participants
                .into_iter()
                .map(|participant| StandaloneConfigRolloutParticipantResponse {
                    node_id: participant.node_id,
                    service_kind: participant.service_kind,
                    status: participant.status,
                    message: participant.message,
                    updated_at_ms: participant.updated_at_ms,
                })
                .collect(),
        }
    }
}

fn default_true() -> bool {
    true
}

fn default_window_seconds() -> u64 {
    60
}
