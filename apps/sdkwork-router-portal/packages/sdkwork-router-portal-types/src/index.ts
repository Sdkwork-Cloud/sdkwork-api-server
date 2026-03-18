export type PortalAnonymousRouteKey = 'login' | 'register';
export type PortalRouteKey =
  | 'dashboard'
  | 'routing'
  | 'api-keys'
  | 'usage'
  | 'user'
  | 'credits'
  | 'billing'
  | 'account';
export type PortalThemeMode = 'light' | 'dark' | 'system';
export type PortalThemeColor =
  | 'tech-blue'
  | 'lobster'
  | 'green-tech'
  | 'zinc'
  | 'violet'
  | 'rose';
export type PortalDataSource = 'live' | 'workspace_seed';

export interface PortalRouteDefinition {
  key: PortalRouteKey;
  label: string;
  eyebrow: string;
  detail: string;
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
  workspace: {
    tenant_id: string;
    project_id: string;
  };
}

export interface PortalWorkspaceSummary {
  user: PortalUserProfile;
  tenant: TenantRecord;
  project: ProjectRecord;
}

export interface GatewayApiKeyRecord {
  tenant_id: string;
  project_id: string;
  environment: string;
  hashed_key: string;
  label: string;
  created_at_ms: number;
  last_used_at_ms?: number | null;
  expires_at_ms?: number | null;
  active: boolean;
}

export interface CreatedGatewayApiKey {
  plaintext: string;
  hashed: string;
  tenant_id: string;
  project_id: string;
  environment: string;
  label: string;
  created_at_ms: number;
  expires_at_ms?: number | null;
}

export interface UsageRecord {
  project_id: string;
  model: string;
  provider: string;
  units: number;
  amount: number;
  input_tokens: number;
  output_tokens: number;
  total_tokens: number;
  created_at_ms: number;
}

export interface UsageSummary {
  total_requests: number;
  project_count: number;
  model_count: number;
  provider_count: number;
  projects: Array<{ project_id: string; request_count: number }>;
  providers: Array<{ provider: string; request_count: number; project_count: number }>;
  models: Array<{ model: string; request_count: number; provider_count: number }>;
}

export interface ProjectBillingSummary {
  project_id: string;
  entry_count: number;
  used_units: number;
  booked_amount: number;
  quota_policy_id?: string | null;
  quota_limit_units?: number | null;
  remaining_units?: number | null;
  exhausted: boolean;
}

export interface LedgerEntry {
  project_id: string;
  units: number;
  amount: number;
}

export interface PortalDashboardSummary {
  workspace: PortalWorkspaceSummary;
  usage_summary: UsageSummary;
  billing_summary: ProjectBillingSummary;
  recent_requests: UsageRecord[];
  api_key_count: number;
}

export type PortalRoutingStrategy =
  | 'deterministic_priority'
  | 'weighted_random'
  | 'slo_aware'
  | 'geo_affinity';

export interface PortalRoutingPreferences {
  project_id: string;
  preset_id: string;
  strategy: PortalRoutingStrategy;
  ordered_provider_ids: string[];
  default_provider_id?: string | null;
  max_cost?: number | null;
  max_latency_ms?: number | null;
  require_healthy: boolean;
  preferred_region?: string | null;
  updated_at_ms: number;
}

export interface PortalRoutingAssessment {
  provider_id: string;
  available: boolean;
  health: 'healthy' | 'unhealthy' | 'unknown';
  policy_rank: number;
  weight?: number | null;
  cost?: number | null;
  latency_ms?: number | null;
  region?: string | null;
  region_match?: boolean | null;
  slo_eligible?: boolean | null;
  slo_violations: string[];
  reasons: string[];
}

export interface PortalRoutingDecision {
  selected_provider_id: string;
  candidate_ids: string[];
  matched_policy_id?: string | null;
  strategy?: string | null;
  selection_seed?: number | null;
  selection_reason?: string | null;
  requested_region?: string | null;
  slo_applied: boolean;
  slo_degraded: boolean;
  assessments: PortalRoutingAssessment[];
}

export interface PortalRoutingDecisionLog {
  decision_id: string;
  decision_source: string;
  tenant_id?: string | null;
  project_id?: string | null;
  capability: string;
  route_key: string;
  selected_provider_id: string;
  matched_policy_id?: string | null;
  strategy: string;
  selection_seed?: number | null;
  selection_reason?: string | null;
  requested_region?: string | null;
  slo_applied: boolean;
  slo_degraded: boolean;
  created_at_ms: number;
  assessments: PortalRoutingAssessment[];
}

export interface PortalRoutingProviderOption {
  provider_id: string;
  display_name: string;
  channel_id: string;
  preferred: boolean;
  default_provider: boolean;
}

export interface PortalRoutingSummary {
  project_id: string;
  preferences: PortalRoutingPreferences;
  latest_model_hint: string;
  preview: PortalRoutingDecision;
  provider_options: PortalRoutingProviderOption[];
}

export interface SubscriptionPlan {
  id: string;
  name: string;
  price_label: string;
  cadence: string;
  included_units: number;
  highlight: string;
  features: string[];
  cta: string;
  source: PortalDataSource;
}

export interface RechargePack {
  id: string;
  label: string;
  points: number;
  price_label: string;
  note: string;
  source: PortalDataSource;
}

export interface CouponOffer {
  code: string;
  title: string;
  benefit: string;
  description: string;
  bonus_units: number;
  source: PortalDataSource;
}
