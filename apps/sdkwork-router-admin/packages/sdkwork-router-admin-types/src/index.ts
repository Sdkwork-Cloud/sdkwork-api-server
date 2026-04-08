export * from './commercePayments';

export type AdminRouteKey =
  | 'overview'
  | 'users'
  | 'tenants'
  | 'coupons'
  | 'commercial'
  | 'pricing'
  | 'api-keys'
  | 'rate-limits'
  | 'route-config'
  | 'model-mapping'
  | 'usage-records'
  | 'catalog'
  | 'traffic'
  | 'operations'
  | 'settings';

export type ThemeMode = 'light' | 'dark' | 'system';
export type ThemeColor =
  | 'tech-blue'
  | 'lobster'
  | 'green-tech'
  | 'zinc'
  | 'violet'
  | 'rose';

export type AdminSidebarItemKey = AdminRouteKey;

export interface AdminThemePreference {
  mode: ThemeMode;
  color: ThemeColor;
}

export type AdminDataSource = 'live';

export interface AdminSessionUser {
  id: string;
  email: string;
  display_name: string;
  active: boolean;
  created_at_ms: number;
}

export interface AdminAuthSession {
  token: string;
  claims: {
    sub: string;
    iss: string;
    aud: string;
    exp: number;
    iat: number;
  };
  user: AdminSessionUser;
}

export interface AdminRouteDefinition {
  key: AdminRouteKey;
  label: string;
  eyebrow: string;
  detail: string;
  group?: string;
}

export type AdminRouteModuleId =
  | 'sdkwork-router-admin-overview'
  | 'sdkwork-router-admin-users'
  | 'sdkwork-router-admin-tenants'
  | 'sdkwork-router-admin-coupons'
  | 'sdkwork-router-admin-commercial'
  | 'sdkwork-router-admin-pricing'
  | 'sdkwork-router-admin-apirouter'
  | 'sdkwork-router-admin-catalog'
  | 'sdkwork-router-admin-traffic'
  | 'sdkwork-router-admin-operations'
  | 'sdkwork-router-admin-settings';

export interface AdminModuleLoadingPolicy {
  strategy: 'lazy';
  prefetch: 'none' | 'intent';
  chunkGroup?: string;
}

export interface AdminModuleNavigationDescriptor {
  group: string;
  order: number;
  sidebar: boolean;
}

export interface AdminProductModuleManifest {
  moduleId: AdminRouteModuleId;
  pluginId: AdminRouteModuleId;
  pluginKind: 'admin-module';
  packageName: AdminRouteModuleId;
  displayName: string;
  routeKeys: AdminRouteKey[];
  capabilityTags: string[];
  requiredPermissions: string[];
  navigation: AdminModuleNavigationDescriptor;
  loading: AdminModuleLoadingPolicy;
}

export interface AdminRouteManifestEntry extends AdminRouteDefinition {
  path: string;
  moduleId: AdminRouteModuleId;
  prefetchGroup?: string;
  productModule: AdminProductModuleManifest;
}

export interface ManagedUser {
  id: string;
  email: string;
  display_name: string;
  role: 'operator' | 'portal';
  active: boolean;
  workspace_tenant_id?: string;
  workspace_project_id?: string;
  request_count: number;
  usage_units: number;
  total_tokens: number;
  source: AdminDataSource;
}

export interface OperatorUserRecord {
  id: string;
  email: string;
  display_name: string;
  active: boolean;
  created_at_ms: number;
}

export interface PortalUserRecord {
  id: string;
  email: string;
  display_name: string;
  workspace_tenant_id: string;
  workspace_project_id: string;
  active: boolean;
  created_at_ms: number;
}

export interface CouponRecord {
  id: string;
  code: string;
  discount_label: string;
  audience: string;
  remaining: number;
  active: boolean;
  note: string;
  expires_on: string;
}

export type MarketingBenefitKind = 'percentage_off' | 'fixed_amount_off' | 'grant_units';
export type MarketingStackingPolicy = 'exclusive' | 'stackable' | 'best_of_group';
export type MarketingSubjectScope = 'user' | 'project' | 'workspace' | 'account';
export type CouponTemplateStatus = 'draft' | 'active' | 'archived';
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

export interface TenantRecord {
  id: string;
  name: string;
}

export interface ProjectRecord {
  tenant_id: string;
  id: string;
  name: string;
}

export type CommerceOrderStatus =
  | 'pending_payment'
  | 'fulfilled'
  | 'canceled'
  | 'failed'
  | 'refunded';

export type CommerceSettlementStatus =
  | 'not_required'
  | 'pending'
  | 'requires_action'
  | 'authorized'
  | 'settled'
  | 'failed'
  | 'canceled'
  | 'partially_refunded'
  | 'refunded';

export type CommercePaymentEventType =
  | 'settled'
  | 'failed'
  | 'canceled'
  | 'refunded';

export type CommercePaymentEventProcessingStatus =
  | 'received'
  | 'processed'
  | 'ignored'
  | 'rejected'
  | 'failed';

export interface CommerceOrderRecord {
  order_id: string;
  project_id: string;
  user_id: string;
  target_kind: string;
  target_id: string;
  target_name: string;
  list_price_cents: number;
  payable_price_cents: number;
  list_price_label: string;
  payable_price_label: string;
  granted_units: number;
  bonus_units: number;
  currency_code: string;
  pricing_plan_id?: string | null;
  pricing_plan_version?: number | null;
  pricing_snapshot_json: string;
  applied_coupon_code?: string | null;
  coupon_reservation_id?: string | null;
  coupon_redemption_id?: string | null;
  marketing_campaign_id?: string | null;
  subsidy_amount_minor: number;
  payment_method_id?: string | null;
  latest_payment_attempt_id?: string | null;
  status: CommerceOrderStatus;
  settlement_status: CommerceSettlementStatus;
  source: string;
  refundable_amount_minor: number;
  refunded_amount_minor: number;
  created_at_ms: number;
  updated_at_ms: number;
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

export interface PaymentMethodCredentialBindingRecord {
  binding_id: string;
  payment_method_id: string;
  usage_kind: string;
  credential_tenant_id: string;
  credential_provider_id: string;
  credential_key_reference: string;
  created_at_ms: number;
  updated_at_ms: number;
}

export interface CommercePaymentEventRecord {
  payment_event_id: string;
  order_id: string;
  project_id: string;
  user_id: string;
  provider: string;
  provider_event_id?: string | null;
  dedupe_key: string;
  event_type: CommercePaymentEventType;
  payload_json: string;
  processing_status: CommercePaymentEventProcessingStatus;
  processing_message?: string | null;
  received_at_ms: number;
  processed_at_ms?: number | null;
  order_status_after?: CommerceOrderStatus | null;
}

export interface CommerceOrderAuditRecord {
  order: CommerceOrderRecord;
  payment_events: CommercePaymentEventRecord[];
  coupon_reservation?: CouponReservationRecord | null;
  coupon_redemption?: CouponRedemptionRecord | null;
  coupon_rollbacks: CouponRollbackRecord[];
  coupon_code?: CouponCodeRecord | null;
  coupon_template?: CouponTemplateRecord | null;
  marketing_campaign?: MarketingCampaignRecord | null;
}

export interface GatewayApiKeyRecord {
  tenant_id: string;
  project_id: string;
  environment: string;
  hashed_key: string;
  api_key_group_id?: string | null;
  raw_key?: string | null;
  label: string;
  notes?: string | null;
  created_at_ms: number;
  last_used_at_ms?: number | null;
  expires_at_ms?: number | null;
  active: boolean;
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
export type CommercialAccountLedgerEntryType =
  | 'hold_create'
  | 'hold_release'
  | 'settlement_capture'
  | 'grant_issue'
  | 'manual_adjustment'
  | 'refund';

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

export interface CommercialPricingPlanCreateInput {
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
}

export interface CommercialPricingRateCreateInput {
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
}

export interface CommercialPricingLifecycleSynchronizationReport {
  changed: boolean;
  due_group_count: number;
  activated_plan_count: number;
  archived_plan_count: number;
  activated_rate_count: number;
  archived_rate_count: number;
  skipped_plan_count: number;
  synchronized_at_ms: number;
}

export type BillingAccountingMode = 'platform_credit' | 'byok' | 'passthrough';

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

export interface CompiledRoutingSnapshotRecord {
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

export interface RateLimitPolicyRecord {
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

export interface RateLimitWindowRecord {
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

export interface ChannelRecord {
  id: string;
  name: string;
}

export interface ProviderChannelBinding {
  channel_id: string;
  is_primary: boolean;
}

export interface ProxyProviderRecord {
  id: string;
  channel_id: string;
  extension_id?: string | null;
  adapter_kind: string;
  base_url: string;
  display_name: string;
  channel_bindings: ProviderChannelBinding[];
}

export interface ModelCatalogRecord {
  external_name: string;
  provider_id: string;
  capabilities: string[];
  streaming: boolean;
  context_window?: number | null;
}

export interface ChannelModelRecord {
  channel_id: string;
  model_id: string;
  model_display_name: string;
  capabilities: string[];
  streaming: boolean;
  context_window?: number | null;
  description?: string | null;
}

export interface ModelPriceRecord {
  channel_id: string;
  model_id: string;
  proxy_provider_id: string;
  currency_code: string;
  price_unit: string;
  input_price: number;
  output_price: number;
  cache_read_price: number;
  cache_write_price: number;
  request_price: number;
  is_active: boolean;
}

export interface CredentialRecord {
  tenant_id: string;
  provider_id: string;
  key_reference: string;
  secret_backend: string;
  secret_local_file?: string | null;
  secret_keyring_service?: string | null;
  secret_master_key_id?: string | null;
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

export interface BillingSummary {
  total_entries: number;
  project_count: number;
  total_units: number;
  total_amount: number;
  active_quota_policy_count: number;
  exhausted_project_count: number;
  projects: Array<{
    project_id: string;
    entry_count: number;
    used_units: number;
    booked_amount: number;
    quota_policy_id?: string | null;
    quota_limit_units?: number | null;
    remaining_units?: number | null;
    exhausted: boolean;
  }>;
}

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

export interface RoutingDecisionLogRecord {
  decision_id: string;
  decision_source: string;
  capability: string;
  route_key: string;
  selected_provider_id: string;
  strategy?: string | null;
  selection_reason?: string | null;
  compiled_routing_snapshot_id?: string | null;
  fallback_reason?: string | null;
  requested_region?: string | null;
  selection_seed?: number | null;
  slo_applied: boolean;
  slo_degraded: boolean;
  created_at_ms: number;
}

export interface ProviderHealthSnapshot {
  provider_id: string;
  status: string;
  healthy: boolean;
  message?: string | null;
  observed_at_ms: number;
}

export interface RuntimeStatusRecord {
  runtime: string;
  extension_id: string;
  instance_id?: string | null;
  display_name: string;
  running: boolean;
  healthy: boolean;
  message?: string | null;
}

export interface RuntimeReloadReport {
  scope: string;
  requested_extension_id?: string | null;
  requested_instance_id?: string | null;
  resolved_extension_id?: string | null;
  discovered_package_count: number;
  loadable_package_count: number;
  active_runtime_count: number;
  reloaded_at_ms: number;
  runtime_statuses: RuntimeStatusRecord[];
}

export interface OverviewMetric {
  label: string;
  value: string;
  detail: string;
}

export interface AdminAlert {
  id: string;
  title: string;
  detail: string;
  severity: 'high' | 'medium' | 'low';
}

export interface AdminWorkspaceSnapshot {
  sessionUser: AdminSessionUser | null;
  operatorUsers: ManagedUser[];
  portalUsers: ManagedUser[];
  coupons: CouponRecord[];
  couponTemplates: CouponTemplateRecord[];
  marketingCampaigns: MarketingCampaignRecord[];
  campaignBudgets: CampaignBudgetRecord[];
  couponCodes: CouponCodeRecord[];
  couponReservations: CouponReservationRecord[];
  couponRedemptions: CouponRedemptionRecord[];
  couponRollbacks: CouponRollbackRecord[];
  tenants: TenantRecord[];
  projects: ProjectRecord[];
  apiKeys: GatewayApiKeyRecord[];
  apiKeyGroups: ApiKeyGroupRecord[];
  routingProfiles: RoutingProfileRecord[];
  compiledRoutingSnapshots: CompiledRoutingSnapshotRecord[];
  rateLimitPolicies: RateLimitPolicyRecord[];
  rateLimitWindows: RateLimitWindowRecord[];
  channels: ChannelRecord[];
  providers: ProxyProviderRecord[];
  credentials: CredentialRecord[];
  models: ModelCatalogRecord[];
  channelModels: ChannelModelRecord[];
  modelPrices: ModelPriceRecord[];
  usageRecords: UsageRecord[];
  usageSummary: UsageSummary;
  billingEvents: BillingEventRecord[];
  billingEventSummary: BillingEventSummary;
  billingSummary: BillingSummary;
  commerceOrders: CommerceOrderRecord[];
  commercePaymentEvents: CommercePaymentEventRecord[];
  commercialAccounts: CommercialAccountSummary[];
  commercialAccountHolds: CommercialAccountHoldRecord[];
  commercialAccountLedger: CommercialAccountLedgerHistoryEntry[];
  commercialRequestSettlements: CommercialRequestSettlementRecord[];
  commercialPricingPlans: CommercialPricingPlanRecord[];
  commercialPricingRates: CommercialPricingRateRecord[];
  routingLogs: RoutingDecisionLogRecord[];
  providerHealth: ProviderHealthSnapshot[];
  runtimeStatuses: RuntimeStatusRecord[];
  overviewMetrics: OverviewMetric[];
  alerts: AdminAlert[];
}

export interface AdminPageProps {
  snapshot: AdminWorkspaceSnapshot;
}
