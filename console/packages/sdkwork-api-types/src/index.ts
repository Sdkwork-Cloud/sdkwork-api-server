export type RuntimeMode = 'server' | 'embedded';

export interface ConsoleSection {
  id: string;
  title: string;
}

export interface TenantRecord {
  id: string;
  name: string;
}

export interface ProjectRecord {
  tenant_id: string;
  id: string;
  name: string;
}

export interface GatewayApiKeyRecord {
  tenant_id: string;
  project_id: string;
  environment: string;
  hashed_key: string;
  active: boolean;
}

export interface CreatedGatewayApiKey {
  plaintext: string;
  hashed: string;
  tenant_id: string;
  project_id: string;
  environment: string;
}

export interface PortalWorkspaceScope {
  tenant_id: string;
  project_id: string;
}

export interface PortalUserProfile {
  id: string;
  email: string;
  display_name: string;
  workspace_tenant_id: string;
  workspace_project_id: string;
  active: boolean;
  created_at_ms: number;
}

export interface PortalAuthSession {
  token: string;
  user: PortalUserProfile;
  workspace: PortalWorkspaceScope;
}

export interface PortalWorkspaceSummary {
  user: PortalUserProfile;
  tenant: TenantRecord;
  project: ProjectRecord;
}

export interface ChannelRecord {
  id: string;
  name: string;
}

export interface ProxyProviderRecord {
  id: string;
  channel_id: string;
  adapter_kind: string;
  base_url: string;
  display_name: string;
}

export interface ModelCatalogRecord {
  external_name: string;
  provider_id: string;
}

export type RoutingStrategy = 'deterministic_priority' | 'weighted_random' | 'slo_aware';
export type RoutingCandidateHealth = 'healthy' | 'unhealthy' | 'unknown';
export type RoutingDecisionSource = 'gateway' | 'admin_simulation';

export interface RoutingCandidateAssessment {
  provider_id: string;
  available: boolean;
  health: RoutingCandidateHealth;
  policy_rank: number;
  weight?: number;
  cost?: number;
  latency_ms?: number;
  slo_eligible?: boolean;
  slo_violations: string[];
  reasons: string[];
}

export interface RoutingSimulationResult {
  selected_provider_id: string;
  candidate_ids: string[];
  matched_policy_id?: string;
  strategy?: string;
  selection_seed?: number;
  selection_reason?: string;
  slo_applied: boolean;
  slo_degraded: boolean;
  assessments: RoutingCandidateAssessment[];
}

export interface RoutingDecisionLog {
  decision_id: string;
  decision_source: RoutingDecisionSource;
  tenant_id?: string;
  project_id?: string;
  capability: string;
  route_key: string;
  selected_provider_id: string;
  matched_policy_id?: string;
  strategy: string;
  selection_seed?: number;
  selection_reason?: string;
  slo_applied: boolean;
  slo_degraded: boolean;
  created_at_ms: number;
  assessments: RoutingCandidateAssessment[];
}

export interface RoutingPolicyRecord {
  policy_id: string;
  capability: string;
  model_pattern: string;
  enabled: boolean;
  priority: number;
  strategy: RoutingStrategy;
  ordered_provider_ids: string[];
  default_provider_id?: string;
  max_cost?: number;
  max_latency_ms?: number;
  require_healthy: boolean;
}

export interface ProviderHealthSnapshot {
  provider_id: string;
  extension_id: string;
  runtime: string;
  observed_at_ms: number;
  instance_id?: string;
  running: boolean;
  healthy: boolean;
  message?: string;
}

export interface UsageRecord {
  project_id: string;
  model: string;
  provider: string;
}

export interface UsageProjectSummary {
  project_id: string;
  request_count: number;
}

export interface UsageProviderSummary {
  provider: string;
  request_count: number;
  project_count: number;
}

export interface UsageModelSummary {
  model: string;
  request_count: number;
  provider_count: number;
}

export interface UsageSummary {
  total_requests: number;
  project_count: number;
  model_count: number;
  provider_count: number;
  projects: UsageProjectSummary[];
  providers: UsageProviderSummary[];
  models: UsageModelSummary[];
}

export interface LedgerEntry {
  project_id: string;
  units: number;
  amount: number;
}

export interface QuotaPolicyRecord {
  policy_id: string;
  project_id: string;
  max_units: number;
  enabled: boolean;
}

export interface ProjectBillingSummary {
  project_id: string;
  entry_count: number;
  used_units: number;
  booked_amount: number;
  quota_policy_id?: string;
  quota_limit_units?: number;
  remaining_units?: number;
  exhausted: boolean;
}

export interface BillingSummary {
  total_entries: number;
  project_count: number;
  total_units: number;
  total_amount: number;
  active_quota_policy_count: number;
  exhausted_project_count: number;
  projects: ProjectBillingSummary[];
}
