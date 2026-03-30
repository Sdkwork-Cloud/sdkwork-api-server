export type AdminRouteKey =
  | 'overview'
  | 'users'
  | 'tenants'
  | 'coupons'
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
  raw_key?: string | null;
  label: string;
  notes?: string | null;
  created_at_ms: number;
  last_used_at_ms?: number | null;
  expires_at_ms?: number | null;
  active: boolean;
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

export interface RoutingDecisionLogRecord {
  decision_id: string;
  decision_source: string;
  capability: string;
  route_key: string;
  selected_provider_id: string;
  strategy?: string | null;
  selection_reason?: string | null;
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
  tenants: TenantRecord[];
  projects: ProjectRecord[];
  apiKeys: GatewayApiKeyRecord[];
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
  billingSummary: BillingSummary;
  routingLogs: RoutingDecisionLogRecord[];
  providerHealth: ProviderHealthSnapshot[];
  runtimeStatuses: RuntimeStatusRecord[];
  overviewMetrics: OverviewMetric[];
  alerts: AdminAlert[];
}

export interface AdminPageProps {
  snapshot: AdminWorkspaceSnapshot;
}
