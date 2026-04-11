export type PortalAnonymousRouteKey = 'login' | 'register' | 'forgot-password';
export type PortalTopLevelRouteKey =
  | 'home'
  | 'console'
  | 'models'
  | 'api-reference'
  | 'docs'
  | 'downloads';
export type PortalRouteKey =
  | 'gateway'
  | 'dashboard'
  | 'routing'
  | 'api-keys'
  | 'usage'
  | 'user'
  | 'credits'
  | 'recharge'
  | 'billing'
  | 'settlements'
  | 'account';
export type PortalRouteGroupKey = 'operations' | 'access' | 'revenue';
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
  group: PortalRouteGroupKey;
  labelKey: string;
  eyebrowKey: string;
  detailKey: string;
  sidebarVisible?: boolean;
}

export type PortalRouteModuleId =
  | 'sdkwork-router-portal-gateway'
  | 'sdkwork-router-portal-dashboard'
  | 'sdkwork-router-portal-routing'
  | 'sdkwork-router-portal-api-keys'
  | 'sdkwork-router-portal-usage'
  | 'sdkwork-router-portal-user'
  | 'sdkwork-router-portal-credits'
  | 'sdkwork-router-portal-recharge'
  | 'sdkwork-router-portal-billing'
  | 'sdkwork-router-portal-settlements'
  | 'sdkwork-router-portal-account';

export interface PortalModuleLoadingPolicy {
  strategy: 'lazy';
  prefetch: 'none' | 'intent';
  chunkGroup?: string;
}

export interface PortalModuleNavigationDescriptor {
  group: PortalRouteGroupKey | 'public';
  order: number;
  sidebar: boolean;
}

export interface PortalProductModuleManifest {
  moduleId: PortalRouteModuleId;
  pluginId: PortalRouteModuleId;
  pluginKind: 'portal-module';
  packageName: PortalRouteModuleId;
  displayName: string;
  routeKeys: PortalRouteKey[];
  capabilityTags: string[];
  requiredPermissions: string[];
  navigation: PortalModuleNavigationDescriptor;
  loading: PortalModuleLoadingPolicy;
}

export interface PortalRouteManifestEntry extends PortalRouteDefinition {
  path: string;
  moduleId: PortalRouteModuleId;
  prefetchGroup?: string;
  productModule: PortalProductModuleManifest;
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
  api_key_group_id?: string | null;
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
  api_key_group_id?: string | null;
  label: string;
  notes?: string | null;
  created_at_ms: number;
  expires_at_ms?: number | null;
}

export interface ApiKeyGroupRecord {
  group_id: string;
  tenant_id: string;
  project_id: string;
  environment: string;
  name: string;
  slug: string;
  description?: string | null;
  color?: string | null;
  default_capability_scope?: string | null;
  default_routing_profile_id?: string | null;
  default_accounting_mode?: string | null;
  active: boolean;
  created_at_ms: number;
  updated_at_ms: number;
}

export interface RoutingProfileRecord {
  profile_id: string;
  tenant_id: string;
  project_id: string;
  name: string;
  slug: string;
  description?: string | null;
  active: boolean;
  strategy: string;
  ordered_provider_ids: string[];
  default_provider_id?: string | null;
  max_cost?: number | null;
  max_latency_ms?: number | null;
  require_healthy: boolean;
  preferred_region?: string | null;
  created_at_ms: number;
  updated_at_ms: number;
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

export type CommercialAccountType = 'primary' | 'grant' | 'postpaid';
export type CommercialAccountStatus = 'active' | 'suspended' | 'closed';
export type CommercialAccountBenefitType =
  | 'cash_credit'
  | 'promo_credit'
  | 'request_allowance'
  | 'token_allowance'
  | 'image_allowance'
  | 'audio_allowance'
  | 'video_allowance'
  | 'music_allowance';
export type CommercialAccountBenefitSourceType =
  | 'recharge'
  | 'coupon'
  | 'grant'
  | 'order'
  | 'manual_adjustment';
export type CommercialAccountBenefitLotStatus =
  | 'active'
  | 'exhausted'
  | 'expired'
  | 'disabled';
export type CommercialAccountHoldStatus =
  | 'held'
  | 'captured'
  | 'partially_released'
  | 'released'
  | 'expired'
  | 'failed';
export type CommercialRequestSettlementStatus =
  | 'pending'
  | 'captured'
  | 'partially_released'
  | 'released'
  | 'refunded'
  | 'failed';

export interface CommercialAccountRecord {
  account_id: number;
  tenant_id: number;
  organization_id: number;
  user_id: number;
  account_type: CommercialAccountType;
  currency_code: string;
  credit_unit_code: string;
  status: CommercialAccountStatus;
  allow_overdraft: boolean;
  overdraft_limit: number;
  created_at_ms: number;
  updated_at_ms: number;
}

export interface CommercialAccountLotBalanceSnapshot {
  lot_id: number;
  benefit_type: CommercialAccountBenefitType;
  scope_json?: string | null;
  expires_at_ms?: number | null;
  original_quantity: number;
  remaining_quantity: number;
  held_quantity: number;
  available_quantity: number;
}

export interface CommercialAccountBalanceSnapshot {
  account_id: number;
  available_balance: number;
  held_balance: number;
  consumed_balance: number;
  grant_balance: number;
  active_lot_count: number;
  lots: CommercialAccountLotBalanceSnapshot[];
}

export interface CommercialAccountSummary {
  account: CommercialAccountRecord;
  available_balance: number;
  held_balance: number;
  consumed_balance: number;
  grant_balance: number;
  active_lot_count: number;
}

export type CommercialAccountLedgerEntryType =
  | 'hold_create'
  | 'hold_release'
  | 'settlement_capture'
  | 'grant_issue'
  | 'manual_adjustment'
  | 'refund';

export interface CommercialAccountLedgerEntryRecord {
  ledger_entry_id: number;
  tenant_id: number;
  organization_id: number;
  account_id: number;
  user_id: number;
  request_id?: number | null;
  hold_id?: number | null;
  entry_type: CommercialAccountLedgerEntryType;
  benefit_type?: string | null;
  quantity: number;
  amount: number;
  created_at_ms: number;
}

export interface CommercialAccountLedgerAllocationRecord {
  ledger_allocation_id: number;
  tenant_id: number;
  organization_id: number;
  ledger_entry_id: number;
  lot_id: number;
  quantity_delta: number;
  created_at_ms: number;
}

export interface CommercialAccountLedgerHistoryEntry {
  entry: CommercialAccountLedgerEntryRecord;
  allocations: CommercialAccountLedgerAllocationRecord[];
}

export interface CommercialAccountHistorySnapshot {
  account: CommercialAccountRecord;
  balance: CommercialAccountBalanceSnapshot;
  benefit_lots: CommercialAccountBenefitLotRecord[];
  holds: CommercialAccountHoldRecord[];
  request_settlements: CommercialRequestSettlementRecord[];
  ledger: CommercialAccountLedgerHistoryEntry[];
}

export interface CommercialAccountBenefitLotRecord {
  lot_id: number;
  tenant_id: number;
  organization_id: number;
  account_id: number;
  user_id: number;
  benefit_type: CommercialAccountBenefitType;
  source_type: CommercialAccountBenefitSourceType;
  source_id?: number | null;
  scope_json?: string | null;
  original_quantity: number;
  remaining_quantity: number;
  held_quantity: number;
  priority: number;
  acquired_unit_cost?: number | null;
  issued_at_ms: number;
  expires_at_ms?: number | null;
  status: CommercialAccountBenefitLotStatus;
  created_at_ms: number;
  updated_at_ms: number;
}

export interface CommercialAccountHoldRecord {
  hold_id: number;
  tenant_id: number;
  organization_id: number;
  account_id: number;
  user_id: number;
  request_id: number;
  status: CommercialAccountHoldStatus;
  estimated_quantity: number;
  captured_quantity: number;
  released_quantity: number;
  expires_at_ms: number;
  created_at_ms: number;
  updated_at_ms: number;
}

export interface CommercialRequestSettlementRecord {
  request_settlement_id: number;
  tenant_id: number;
  organization_id: number;
  request_id: number;
  account_id: number;
  user_id: number;
  hold_id?: number | null;
  status: CommercialRequestSettlementStatus;
  estimated_credit_hold: number;
  released_credit_amount: number;
  captured_credit_amount: number;
  provider_cost_amount: number;
  retail_charge_amount: number;
  shortfall_amount: number;
  refunded_amount: number;
  settled_at_ms: number;
  created_at_ms: number;
  updated_at_ms: number;
}

export interface CommercialPricingPlanRecord {
  pricing_plan_id: number;
  tenant_id: number;
  organization_id: number;
  plan_code: string;
  plan_version: number;
  display_name: string;
  currency_code: string;
  credit_unit_code: string;
  status: string;
  effective_from_ms: number;
  effective_to_ms?: number | null;
  created_at_ms: number;
  updated_at_ms: number;
}

export type CommercialPricingChargeUnit =
  | 'input_token'
  | 'output_token'
  | 'cache_read_token'
  | 'cache_write_token'
  | 'request'
  | 'image'
  | 'audio_second'
  | 'audio_minute'
  | 'video_second'
  | 'video_minute'
  | 'music_track'
  | 'character'
  | 'storage_mb_day'
  | 'tool_call'
  | 'unit';

export type CommercialPricingMethod =
  | 'per_unit'
  | 'flat'
  | 'step'
  | 'included_then_per_unit';

export interface CommercialPricingRateRecord {
  pricing_rate_id: number;
  tenant_id: number;
  organization_id: number;
  pricing_plan_id: number;
  metric_code: string;
  capability_code?: string | null;
  model_code?: string | null;
  provider_code?: string | null;
  charge_unit: CommercialPricingChargeUnit;
  pricing_method: CommercialPricingMethod;
  quantity_step: number;
  unit_price: number;
  display_price_unit: string;
  minimum_billable_quantity: number;
  minimum_charge: number;
  rounding_increment: number;
  rounding_mode: string;
  included_quantity: number;
  priority: number;
  notes?: string | null;
  status: string;
  created_at_ms: number;
  updated_at_ms: number;
}

export type BillingAccountingMode = 'platform_credit' | 'byok' | 'passthrough';

export interface BillingEventRecord {
  event_id: string;
  tenant_id: string;
  project_id: string;
  api_key_group_id?: string | null;
  capability: string;
  route_key: string;
  usage_model: string;
  provider_id: string;
  accounting_mode: BillingAccountingMode;
  operation_kind: string;
  modality: string;
  api_key_hash?: string | null;
  channel_id?: string | null;
  reference_id?: string | null;
  latency_ms?: number | null;
  units: number;
  request_count: number;
  input_tokens: number;
  output_tokens: number;
  total_tokens: number;
  cache_read_tokens: number;
  cache_write_tokens: number;
  image_count: number;
  audio_seconds: number;
  video_seconds: number;
  music_seconds: number;
  upstream_cost: number;
  customer_charge: number;
  applied_routing_profile_id?: string | null;
  compiled_routing_snapshot_id?: string | null;
  fallback_reason?: string | null;
  created_at_ms: number;
}

export interface BillingEventProjectSummary {
  project_id: string;
  event_count: number;
  request_count: number;
  total_units: number;
  total_input_tokens: number;
  total_output_tokens: number;
  total_tokens: number;
  total_image_count: number;
  total_audio_seconds: number;
  total_video_seconds: number;
  total_music_seconds: number;
  total_upstream_cost: number;
  total_customer_charge: number;
}

export interface BillingEventGroupSummary {
  api_key_group_id?: string | null;
  project_count: number;
  event_count: number;
  request_count: number;
  total_upstream_cost: number;
  total_customer_charge: number;
}

export interface BillingEventCapabilitySummary {
  capability: string;
  event_count: number;
  request_count: number;
  total_tokens: number;
  image_count: number;
  audio_seconds: number;
  video_seconds: number;
  music_seconds: number;
  total_upstream_cost: number;
  total_customer_charge: number;
}

export interface BillingEventAccountingModeSummary {
  accounting_mode: BillingAccountingMode;
  event_count: number;
  request_count: number;
  total_upstream_cost: number;
  total_customer_charge: number;
}

export interface BillingEventSummary {
  total_events: number;
  project_count: number;
  group_count: number;
  capability_count: number;
  total_request_count: number;
  total_units: number;
  total_input_tokens: number;
  total_output_tokens: number;
  total_tokens: number;
  total_image_count: number;
  total_audio_seconds: number;
  total_video_seconds: number;
  total_music_seconds: number;
  total_upstream_cost: number;
  total_customer_charge: number;
  projects: BillingEventProjectSummary[];
  groups: BillingEventGroupSummary[];
  capabilities: BillingEventCapabilitySummary[];
  accounting_modes: BillingEventAccountingModeSummary[];
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
  applied_routing_profile_id?: string | null;
  compiled_routing_snapshot_id?: string | null;
  strategy?: string | null;
  selection_seed?: number | null;
  selection_reason?: string | null;
  fallback_reason?: string | null;
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
  api_key_group_id?: string | null;
  capability: string;
  route_key: string;
  selected_provider_id: string;
  matched_policy_id?: string | null;
  applied_routing_profile_id?: string | null;
  compiled_routing_snapshot_id?: string | null;
  strategy: string;
  selection_seed?: number | null;
  selection_reason?: string | null;
  fallback_reason?: string | null;
  requested_region?: string | null;
  slo_applied: boolean;
  slo_degraded: boolean;
  created_at_ms: number;
  assessments: PortalRoutingAssessment[];
}

export interface PortalCompiledRoutingSnapshotRecord {
  snapshot_id: string;
  tenant_id?: string | null;
  project_id?: string | null;
  api_key_group_id?: string | null;
  capability: string;
  route_key: string;
  matched_policy_id?: string | null;
  project_routing_preferences_project_id?: string | null;
  applied_routing_profile_id?: string | null;
  strategy: string;
  ordered_provider_ids: string[];
  default_provider_id?: string | null;
  max_cost?: number | null;
  max_latency_ms?: number | null;
  require_healthy: boolean;
  preferred_region?: string | null;
  created_at_ms: number;
  updated_at_ms: number;
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

export interface PortalRechargeOption {
  id: string;
  label: string;
  amount_cents: number;
  amount_label: string;
  granted_units: number;
  effective_ratio_label: string;
  note: string;
  recommended: boolean;
  source: PortalDataSource;
}

export interface PortalCustomRechargeRule {
  id: string;
  label: string;
  min_amount_cents: number;
  max_amount_cents: number;
  units_per_cent: number;
  effective_ratio_label: string;
  note: string;
}

export interface PortalCustomRechargePolicy {
  enabled: boolean;
  min_amount_cents: number;
  max_amount_cents: number;
  step_amount_cents: number;
  suggested_amount_cents: number;
  rules: PortalCustomRechargeRule[];
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

export interface PortalApiProduct {
  product_id: string;
  product_kind: ApiProductKind;
  target_id: string;
  display_name: string;
  source: PortalDataSource;
}

export interface PortalProductOffer {
  offer_id: string;
  product_id: string;
  product_kind: ApiProductKind;
  display_name: string;
  quote_kind: PortalQuoteKind;
  quote_target_kind: ApiProductKind;
  quote_target_id: string;
  publication_id?: string | null;
  publication_kind?: string | null;
  publication_status?: string | null;
  pricing_plan_id?: string | null;
  pricing_plan_version?: number | null;
  pricing_rate_id?: string | null;
  pricing_metric_code?: string | null;
  price_label?: string | null;
  source: PortalDataSource;
}

export interface PortalCommerceCatalog {
  products: PortalApiProduct[];
  offers: PortalProductOffer[];
  plans: SubscriptionPlan[];
  packs: RechargePack[];
  recharge_options: PortalRechargeOption[];
  custom_recharge_policy: PortalCustomRechargePolicy | null;
  coupons: PortalCommerceCoupon[];
}

export type MarketingBenefitKind = 'percentage_off' | 'fixed_amount_off' | 'grant_units';
export type MarketingStackingPolicy = 'exclusive' | 'stackable' | 'best_of_group';
export type MarketingSubjectScope = 'user' | 'project' | 'workspace' | 'account';
export type CouponTemplateStatus = 'draft' | 'scheduled' | 'active' | 'archived';
export type CouponDistributionKind = 'shared_code' | 'unique_code' | 'auto_claim';
export type MarketingCampaignStatus =
  | 'draft'
  | 'scheduled'
  | 'active'
  | 'paused'
  | 'ended'
  | 'archived';
export type CampaignBudgetStatus = 'draft' | 'active' | 'exhausted' | 'closed';
export type CouponCodeStatus = 'available' | 'reserved' | 'redeemed' | 'expired' | 'disabled';
export type CouponReservationStatus = 'reserved' | 'released' | 'confirmed' | 'expired';
export type CouponRedemptionStatus =
  | 'pending'
  | 'redeemed'
  | 'partially_rolled_back'
  | 'rolled_back'
  | 'failed';
export type CouponRollbackType = 'cancel' | 'refund' | 'partial_refund' | 'manual';
export type CouponRollbackStatus = 'pending' | 'completed' | 'failed';

export interface CouponBenefitSpec {
  benefit_kind: MarketingBenefitKind;
  subsidy_percent?: number | null;
  subsidy_amount_minor?: number | null;
  grant_units?: number | null;
  currency_code?: string | null;
}

export interface CouponRestrictionSpec {
  subject_scope: MarketingSubjectScope;
  min_order_amount_minor?: number | null;
  first_order_only: boolean;
  new_customer_only: boolean;
  exclusive_group?: string | null;
  stacking_policy: MarketingStackingPolicy;
  max_redemptions_per_subject?: number | null;
  eligible_target_kinds: string[];
}

export interface CouponTemplateRecord {
  coupon_template_id: string;
  template_key: string;
  display_name: string;
  status: CouponTemplateStatus;
  distribution_kind: CouponDistributionKind;
  benefit: CouponBenefitSpec;
  restriction: CouponRestrictionSpec;
  activation_at_ms?: number | null;
  created_at_ms: number;
  updated_at_ms: number;
}

export interface MarketingCampaignRecord {
  marketing_campaign_id: string;
  coupon_template_id: string;
  display_name: string;
  status: MarketingCampaignStatus;
  start_at_ms?: number | null;
  end_at_ms?: number | null;
  created_at_ms: number;
  updated_at_ms: number;
}

export interface CampaignBudgetRecord {
  campaign_budget_id: string;
  marketing_campaign_id: string;
  status: CampaignBudgetStatus;
  total_budget_minor: number;
  reserved_budget_minor: number;
  consumed_budget_minor: number;
  created_at_ms: number;
  updated_at_ms: number;
}

export interface CouponCodeRecord {
  coupon_code_id: string;
  coupon_template_id: string;
  code_value: string;
  status: CouponCodeStatus;
  claimed_subject_scope?: MarketingSubjectScope | null;
  claimed_subject_id?: string | null;
  expires_at_ms?: number | null;
  created_at_ms: number;
  updated_at_ms: number;
}

export interface CouponReservationRecord {
  coupon_reservation_id: string;
  coupon_code_id: string;
  subject_scope: MarketingSubjectScope;
  subject_id: string;
  reservation_status: CouponReservationStatus;
  budget_reserved_minor: number;
  expires_at_ms: number;
  created_at_ms: number;
  updated_at_ms: number;
}

export interface CouponRedemptionRecord {
  coupon_redemption_id: string;
  coupon_reservation_id: string;
  coupon_code_id: string;
  coupon_template_id: string;
  redemption_status: CouponRedemptionStatus;
  subsidy_amount_minor: number;
  order_id?: string | null;
  payment_event_id?: string | null;
  redeemed_at_ms: number;
  updated_at_ms: number;
}

export interface CouponRollbackRecord {
  coupon_rollback_id: string;
  coupon_redemption_id: string;
  rollback_type: CouponRollbackType;
  rollback_status: CouponRollbackStatus;
  restored_budget_minor: number;
  restored_inventory_count: number;
  created_at_ms: number;
  updated_at_ms: number;
}

export interface PortalCouponValidationRequest {
  coupon_code: string;
  subject_scope: MarketingSubjectScope;
  target_kind: PortalMarketingTargetKind;
  order_amount_minor: number;
  reserve_amount_minor: number;
}

export interface PortalCouponValidationDecisionResponse {
  eligible: boolean;
  rejection_reason?: string | null;
  reservable_budget_minor: number;
}

export interface PortalCouponValidationResponse {
  decision: PortalCouponValidationDecisionResponse;
  template: CouponTemplateRecord;
  campaign: MarketingCampaignRecord;
  budget: CampaignBudgetRecord;
  code: CouponCodeRecord;
}

export interface PortalCouponReservationRequest {
  coupon_code: string;
  subject_scope: MarketingSubjectScope;
  target_kind: PortalMarketingTargetKind;
  reserve_amount_minor: number;
  ttl_ms: number;
  idempotency_key?: string | null;
}

export interface PortalCouponReservationResponse {
  reservation: CouponReservationRecord;
  template: CouponTemplateRecord;
  campaign: MarketingCampaignRecord;
  budget: CampaignBudgetRecord;
  code: CouponCodeRecord;
}

export interface PortalCouponRedemptionConfirmRequest {
  coupon_reservation_id: string;
  subsidy_amount_minor: number;
  order_id?: string | null;
  payment_event_id?: string | null;
  idempotency_key?: string | null;
}

export interface PortalCouponRedemptionConfirmResponse {
  reservation: CouponReservationRecord;
  redemption: CouponRedemptionRecord;
  budget: CampaignBudgetRecord;
  code: CouponCodeRecord;
}

export interface PortalCouponRedemptionRollbackRequest {
  coupon_redemption_id: string;
  rollback_type: CouponRollbackType;
  restored_budget_minor: number;
  restored_inventory_count: number;
  idempotency_key?: string | null;
}

export interface PortalCouponRedemptionRollbackResponse {
  redemption: CouponRedemptionRecord;
  rollback: CouponRollbackRecord;
  budget: CampaignBudgetRecord;
  code: CouponCodeRecord;
}

export type PortalCouponEffectKind = 'checkout_discount' | 'account_entitlement';

export interface PortalCouponApplicabilitySummary {
  target_kinds: string[];
  all_target_kinds_eligible: boolean;
}

export interface PortalCouponEffectSummary {
  effect_kind: PortalCouponEffectKind;
  discount_percent?: number | null;
  discount_amount_minor?: number | null;
  grant_units?: number | null;
}

export interface PortalCouponOwnershipSummary {
  owned_by_current_subject: boolean;
  claimed_to_current_subject: boolean;
  claimed_subject_scope?: MarketingSubjectScope | null;
  claimed_subject_id?: string | null;
}

export interface PortalCouponAccountArrivalLotItem {
  lot_id: number;
  benefit_type: CommercialAccountBenefitType;
  source_type: CommercialAccountBenefitSourceType;
  source_id?: number | null;
  status: CommercialAccountBenefitLotStatus;
  original_quantity: number;
  remaining_quantity: number;
  issued_at_ms: number;
  expires_at_ms?: number | null;
  scope_order_id?: string | null;
}

export interface PortalCouponAccountArrivalSummary {
  order_id?: string | null;
  account_id?: number | null;
  benefit_lot_count: number;
  credited_quantity: number;
  benefit_lots: PortalCouponAccountArrivalLotItem[];
}

export interface PortalMarketingRedemptionSummary {
  total_count: number;
  redeemed_count: number;
  partially_rolled_back_count: number;
  rolled_back_count: number;
  failed_count: number;
}

export interface PortalMarketingCodeSummary {
  total_count: number;
  available_count: number;
  reserved_count: number;
  redeemed_count: number;
  disabled_count: number;
  expired_count: number;
}

export interface PortalMarketingCodeItem {
  code: CouponCodeRecord;
  template: CouponTemplateRecord;
  campaign: MarketingCampaignRecord;
  applicability: PortalCouponApplicabilitySummary;
  effect: PortalCouponEffectSummary;
  ownership: PortalCouponOwnershipSummary;
  latest_reservation?: CouponReservationRecord | null;
  latest_redemption?: CouponRedemptionRecord | null;
}

export interface PortalMarketingCodesResponse {
  summary: PortalMarketingCodeSummary;
  items: PortalMarketingCodeItem[];
}

export interface PortalMarketingRedemptionsResponse {
  summary: PortalMarketingRedemptionSummary;
  items: CouponRedemptionRecord[];
}

export interface PortalMarketingRewardHistoryItem {
  redemption: CouponRedemptionRecord;
  code: CouponCodeRecord;
  template: CouponTemplateRecord;
  campaign: MarketingCampaignRecord;
  applicability: PortalCouponApplicabilitySummary;
  effect: PortalCouponEffectSummary;
  ownership: PortalCouponOwnershipSummary;
  account_arrival: PortalCouponAccountArrivalSummary;
  rollbacks: CouponRollbackRecord[];
}

export type PortalCommerceTargetKind =
  | 'subscription_plan'
  | 'recharge_pack'
  | 'custom_recharge'
  | 'coupon_redemption';

export type PortalMarketingTargetKind = PortalCommerceTargetKind;

export type ApiProductKind = Exclude<PortalCommerceTargetKind, 'coupon_redemption'>;

export type PortalQuoteKind = 'product_purchase' | 'coupon_redemption';

export type CommercialTransactionKind = 'product_purchase' | 'coupon_redemption';

export type PortalCommerceQuoteKind = PortalCommerceTargetKind;

export interface PortalCommerceQuoteRequest {
  target_kind: PortalCommerceTargetKind;
  target_id: string;
  coupon_code?: string | null;
  current_remaining_units?: number | null;
  custom_amount_cents?: number | null;
}

export interface PortalAppliedCoupon {
  code: string;
  discount_label: string;
  source: PortalDataSource;
  discount_percent?: number | null;
  bonus_units: number;
}

export interface PortalCommerceQuote {
  target_kind: PortalCommerceTargetKind;
  product_kind?: ApiProductKind | null;
  quote_kind: PortalQuoteKind;
  target_id: string;
  target_name: string;
  product_id?: string | null;
  offer_id?: string | null;
  publication_id?: string | null;
  publication_kind?: string | null;
  publication_status?: string | null;
  list_price_cents: number;
  payable_price_cents: number;
  list_price_label: string;
  payable_price_label: string;
  granted_units: number;
  bonus_units: number;
  amount_cents?: number | null;
  projected_remaining_units?: number | null;
  pricing_plan_id?: string | null;
  pricing_plan_version?: number | null;
  pricing_rate_id?: string | null;
  pricing_metric_code?: string | null;
  applied_coupon?: PortalAppliedCoupon | null;
  pricing_rule_label?: string | null;
  effective_ratio_label?: string | null;
  source: PortalDataSource;
}

export type PortalCommerceOrderStatus =
  | 'pending_payment'
  | 'fulfilled'
  | 'canceled'
  | 'failed'
  | 'refunded';

export type PortalCommerceCheckoutSessionStatus =
  | 'open'
  | 'settled'
  | 'canceled'
  | 'failed'
  | 'refunded'
  | 'not_required'
  | 'closed';

export type PortalCommercePaymentEventType =
  | 'settled'
  | 'failed'
  | 'canceled'
  | 'refunded';

export type PortalCommercePaymentProvider =
  | 'manual_lab'
  | 'stripe'
  | 'alipay'
  | 'wechat_pay'
  | 'no_payment_required';

export type PortalCommerceCheckoutMethodChannel =
  | 'operator_settlement'
  | 'hosted_checkout'
  | 'scan_qr';

export type PortalCommerceCheckoutMethodSessionKind =
  | 'operator_action'
  | 'hosted_checkout'
  | 'qr_code';

export type PortalCommercePaymentEventProcessingStatus =
  | 'received'
  | 'processed'
  | 'ignored'
  | 'rejected'
  | 'failed';

export interface PortalCommercePaymentEventRequest {
  event_type: PortalCommercePaymentEventType;
  provider?: PortalCommercePaymentProvider | null;
  provider_event_id?: string | null;
  checkout_method_id?: string | null;
}

export interface PortalCommercePaymentAttemptCreateRequest {
  payment_method_id: string;
  idempotency_key?: string | null;
  success_url?: string | null;
  cancel_url?: string | null;
  country_code?: string | null;
  customer_email?: string | null;
}

export interface PortalCommercePaymentEventRecord {
  payment_event_id: string;
  order_id: string;
  project_id: string;
  user_id: string;
  provider: string;
  provider_event_id?: string | null;
  dedupe_key: string;
  event_type: PortalCommercePaymentEventType;
  payload_json: string;
  processing_status: PortalCommercePaymentEventProcessingStatus;
  processing_message?: string | null;
  received_at_ms: number;
  processed_at_ms?: number | null;
  order_status_after?: PortalCommerceOrderStatus | null;
}

export interface PaymentMethodRecord {
  payment_method_id: string;
  display_name: string;
  description: string;
  provider: string;
  channel: string;
  mode: string;
  enabled: boolean;
  sort_order: number;
  capability_codes: string[];
  supported_currency_codes: string[];
  supported_country_codes: string[];
  supported_order_kinds: string[];
  callback_strategy: string;
  webhook_path?: string | null;
  webhook_tolerance_seconds: number;
  replay_window_seconds: number;
  max_retry_count: number;
  config_json: string;
  created_at_ms: number;
  updated_at_ms: number;
}

export interface CommercePaymentAttemptRecord {
  payment_attempt_id: string;
  order_id: string;
  project_id: string;
  user_id: string;
  payment_method_id: string;
  provider: string;
  channel: string;
  status: string;
  idempotency_key: string;
  attempt_sequence: number;
  amount_minor: number;
  currency_code: string;
  captured_amount_minor: number;
  refunded_amount_minor: number;
  provider_payment_intent_id?: string | null;
  provider_checkout_session_id?: string | null;
  provider_reference?: string | null;
  checkout_url?: string | null;
  qr_code_payload?: string | null;
  request_payload_json: string;
  response_payload_json: string;
  error_code?: string | null;
  error_message?: string | null;
  initiated_at_ms: number;
  expires_at_ms?: number | null;
  completed_at_ms?: number | null;
  updated_at_ms: number;
}

export interface PortalCommerceCheckoutSessionMethod {
  id: string;
  label: string;
  detail: string;
  action: 'settle_order' | 'cancel_order' | 'provider_handoff';
  availability: 'available' | 'planned' | 'closed';
  provider: PortalCommercePaymentProvider;
  channel: PortalCommerceCheckoutMethodChannel;
  session_kind: PortalCommerceCheckoutMethodSessionKind;
  session_reference: string;
  qr_code_payload?: string | null;
  webhook_verification: string;
  supports_refund: boolean;
  supports_partial_refund: boolean;
  recommended: boolean;
  supports_webhook: boolean;
}

export interface PortalCommerceCheckoutSession {
  order_id: string;
  order_status: PortalCommerceOrderStatus;
  session_status: PortalCommerceCheckoutSessionStatus;
  provider: PortalCommercePaymentProvider;
  mode: string;
  reference: string;
  payable_price_label: string;
  guidance: string;
  payment_simulation_enabled: boolean;
  methods: PortalCommerceCheckoutSessionMethod[];
}

export interface PortalCommerceOrder {
  order_id: string;
  project_id: string;
  user_id: string;
  target_kind: PortalCommerceTargetKind;
  product_kind?: ApiProductKind | null;
  transaction_kind: CommercialTransactionKind;
  product_id?: string | null;
  offer_id?: string | null;
  publication_id?: string | null;
  publication_kind?: string | null;
  publication_status?: string | null;
  target_id: string;
  target_name: string;
  pricing_plan_id?: string | null;
  pricing_plan_version?: number | null;
  pricing_rate_id?: string | null;
  pricing_metric_code?: string | null;
  list_price_cents: number;
  payable_price_cents: number;
  list_price_label: string;
  payable_price_label: string;
  granted_units: number;
  bonus_units: number;
  applied_coupon_code?: string | null;
  coupon_reservation_id?: string | null;
  coupon_redemption_id?: string | null;
  marketing_campaign_id?: string | null;
  subsidy_amount_minor?: number;
  payment_method_id?: string | null;
  latest_payment_attempt_id?: string | null;
  status: PortalCommerceOrderStatus;
  settlement_status?: string;
  source: PortalDataSource;
  refundable_amount_minor?: number;
  refunded_amount_minor?: number;
  created_at_ms: number;
  updated_at_ms?: number;
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

export interface PortalCommerceOrderCenterEntry {
  order: PortalCommerceOrder;
  payment_events: PortalCommercePaymentEventRecord[];
  latest_payment_event?: PortalCommercePaymentEventRecord | null;
  checkout_session: PortalCommerceCheckoutSession;
}

export interface PortalCommerceReconciliationSummary {
  account_id: number;
  last_reconciled_order_id: string;
  last_reconciled_order_updated_at_ms: number;
  last_reconciled_order_created_at_ms: number;
  last_reconciled_at_ms: number;
  backlog_order_count: number;
  checkpoint_lag_ms: number;
  healthy: boolean;
}

export interface PortalCommerceOrderCenterResponse {
  project_id: string;
  payment_simulation_enabled: boolean;
  membership: PortalCommerceMembership | null;
  reconciliation: PortalCommerceReconciliationSummary | null;
  orders: PortalCommerceOrderCenterEntry[];
}
