export type PortalAnonymousRouteKey = 'login' | 'register' | 'forgot-password';
export type PortalRouteKey =
  | 'gateway'
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
  notes?: string | null;
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
  notes?: string | null;
  created_at_ms: number;
  expires_at_ms?: number | null;
}

export interface UsageRecord {
  project_id: string;
  model: string;
  provider: string;
  units: number;
  amount: number;
  api_key_hash?: string | null;
  channel_id?: string | null;
  input_tokens: number;
  output_tokens: number;
  total_tokens: number;
  latency_ms?: number | null;
  reference_amount?: number | null;
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

export interface PortalRateLimitPolicySnapshot {
  policy_id: string;
  project_id: string;
  api_key_hash?: string | null;
  route_key?: string | null;
  model_name?: string | null;
  requests_per_window: number;
  window_seconds: number;
  burst_requests: number;
  limit_requests: number;
  enabled: boolean;
  notes?: string | null;
  created_at_ms: number;
  updated_at_ms: number;
}

export interface PortalRateLimitWindowSnapshot {
  policy_id: string;
  project_id: string;
  api_key_hash?: string | null;
  route_key?: string | null;
  model_name?: string | null;
  requests_per_window: number;
  window_seconds: number;
  burst_requests: number;
  limit_requests: number;
  request_count: number;
  remaining_requests: number;
  window_start_ms: number;
  window_end_ms: number;
  updated_at_ms: number;
  enabled: boolean;
  exceeded: boolean;
}

export interface LedgerEntry {
  project_id: string;
  units: number;
  amount: number;
}

export interface PortalGatewayRateLimitSnapshot {
  project_id: string;
  policy_count: number;
  active_policy_count: number;
  window_count: number;
  exceeded_window_count: number;
  headline: string;
  detail: string;
  generated_at_ms: number;
  policies: PortalRateLimitPolicySnapshot[];
  windows: PortalRateLimitWindowSnapshot[];
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

export type PortalProductRuntimeRole = 'web' | 'gateway' | 'admin' | 'portal';
export type PortalProductRuntimeMode = 'desktop' | 'server';

export interface PortalDesktopRuntimeSnapshot {
  mode: PortalProductRuntimeMode;
  roles: PortalProductRuntimeRole[];
  publicBaseUrl?: string | null;
  publicBindAddr?: string | null;
  gatewayBindAddr?: string | null;
  adminBindAddr?: string | null;
  portalBindAddr?: string | null;
}

export type PortalRuntimeHealthStatus = 'healthy' | 'degraded' | 'unreachable';

export interface PortalRuntimeServiceHealth {
  id: PortalProductRuntimeRole;
  label: string;
  status: PortalRuntimeHealthStatus;
  healthUrl: string;
  detail: string;
  httpStatus?: number | null;
  responseTimeMs?: number | null;
}

export interface PortalRuntimeHealthSnapshot {
  mode: PortalProductRuntimeMode | 'browser';
  checkedAtMs: number;
  services: PortalRuntimeServiceHealth[];
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

export interface PortalCommerceCoupon {
  id: string;
  code: string;
  discount_label: string;
  audience: string;
  remaining: number;
  active: boolean;
  note: string;
  expires_on: string;
  source: PortalDataSource;
  discount_percent?: number | null;
  bonus_units: number;
}

export interface PortalCommerceCatalog {
  plans: SubscriptionPlan[];
  packs: RechargePack[];
  coupons: PortalCommerceCoupon[];
}

export type PortalCommerceQuoteKind =
  | 'subscription_plan'
  | 'recharge_pack'
  | 'coupon_redemption';

export interface PortalCommerceQuoteRequest {
  target_kind: PortalCommerceQuoteKind;
  target_id: string;
  coupon_code?: string | null;
  current_remaining_units?: number | null;
}

export interface PortalAppliedCoupon {
  code: string;
  discount_label: string;
  source: PortalDataSource;
  discount_percent?: number | null;
  bonus_units: number;
}

export interface PortalCommerceQuote {
  target_kind: PortalCommerceQuoteKind;
  target_id: string;
  target_name: string;
  list_price_cents: number;
  payable_price_cents: number;
  list_price_label: string;
  payable_price_label: string;
  granted_units: number;
  bonus_units: number;
  projected_remaining_units?: number | null;
  applied_coupon?: PortalAppliedCoupon | null;
  source: PortalDataSource;
}

export type PortalCommerceOrderStatus =
  | 'pending_payment'
  | 'fulfilled'
  | 'canceled'
  | 'failed';

export type PortalCommerceCheckoutSessionStatus =
  | 'open'
  | 'settled'
  | 'canceled'
  | 'failed'
  | 'not_required'
  | 'closed';

export type PortalCommercePaymentEventType = 'settled' | 'failed' | 'canceled';

export interface PortalCommercePaymentEventRequest {
  event_type: PortalCommercePaymentEventType;
}

export interface PortalCommerceCheckoutSessionMethod {
  id: string;
  label: string;
  detail: string;
  action: 'settle_order' | 'cancel_order' | 'provider_handoff';
  availability: 'available' | 'planned' | 'closed';
}

export interface PortalCommerceCheckoutSession {
  order_id: string;
  order_status: PortalCommerceOrderStatus;
  session_status: PortalCommerceCheckoutSessionStatus;
  provider: string;
  mode: string;
  reference: string;
  payable_price_label: string;
  guidance: string;
  methods: PortalCommerceCheckoutSessionMethod[];
}

export interface PortalCommerceOrder {
  order_id: string;
  project_id: string;
  user_id: string;
  target_kind: PortalCommerceQuoteKind;
  target_id: string;
  target_name: string;
  list_price_cents: number;
  payable_price_cents: number;
  list_price_label: string;
  payable_price_label: string;
  granted_units: number;
  bonus_units: number;
  applied_coupon_code?: string | null;
  status: PortalCommerceOrderStatus;
  source: PortalDataSource;
  created_at_ms: number;
}

export interface PortalCommerceMembership {
  membership_id: string;
  project_id: string;
  user_id: string;
  plan_id: string;
  plan_name: string;
  price_cents: number;
  price_label: string;
  cadence: string;
  included_units: number;
  status: string;
  source: PortalDataSource;
  activated_at_ms: number;
  updated_at_ms: number;
}
