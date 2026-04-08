import type {
  ApiKeyGroupRecord,
  BillingEventRecord,
  BillingEventSummary,
  CampaignBudgetRecord,
  CommercePaymentAttemptRecord,
  CommercialAccountHistorySnapshot,
  CommercialAccountBalanceSnapshot,
  CommercialAccountBenefitLotRecord,
  CommercialAccountHoldRecord,
  CommercialAccountSummary,
  CommercialPricingPlanRecord,
  CommercialPricingRateRecord,
  CommercialRequestSettlementRecord,
  CouponCodeRecord,
  CouponRedemptionRecord,
  CouponReservationRecord,
  CouponRollbackRecord,
  CouponTemplateRecord,
  CreatedGatewayApiKey,
  GatewayApiKeyRecord,
  LedgerEntry,
  MarketingCampaignRecord,
  PaymentMethodRecord,
  PortalCommerceCheckoutSession,
  PortalCommercePaymentEventRequest,
  PortalCommercePaymentAttemptCreateRequest,
  PortalCommerceQuote,
  PortalCommerceOrder,
  PortalCommerceOrderCenterResponse,
  PortalCommerceMembership,
  PortalCommerceQuoteRequest,
  PortalCommerceCatalog,
  PortalCouponRedemptionConfirmRequest,
  PortalCouponRedemptionConfirmResponse,
  PortalCouponRedemptionRollbackRequest,
  PortalCouponRedemptionRollbackResponse,
  PortalCouponReservationRequest,
  PortalCouponReservationResponse,
  PortalCouponValidationRequest,
  PortalCouponValidationResponse,
  PortalDesktopRuntimeSnapshot,
  PortalMarketingCodesResponse,
  PortalMarketingRedemptionsResponse,
  PortalMarketingRewardHistoryItem,
  PortalRuntimeHealthSnapshot,
  PortalRuntimeServiceHealth,
  PortalAuthSession,
  PortalDashboardSummary,
  PortalGatewayRateLimitSnapshot,
  PortalCompiledRoutingSnapshotRecord,
  PortalRoutingDecision,
  PortalRoutingDecisionLog,
  PortalRoutingPreferences,
  RoutingProfileRecord,
  PortalRoutingSummary,
  PortalUserProfile,
  PortalWorkspaceSummary,
  ProjectBillingSummary,
  UsageRecord,
  UsageSummary,
} from 'sdkwork-router-portal-types';

const portalSessionTokenKey = 'sdkwork.router.portal.session-token';
const portalSessionExpiredEvent = 'sdkwork.router.portal.session-expired';
const portalProxyPrefix = '/api/portal';
const gatewayProxyPrefix = '/api';
const standalonePortalDevPorts = new Set(['4174', '5174']);
const standaloneGatewayBaseUrl = 'http://127.0.0.1:8080';

type TauriWindowLike = Window & {
  __TAURI__?: unknown;
  __TAURI_INTERNALS__?: TauriInternalsLike;
  isTauri?: boolean;
};

type TauriInternalsLike = {
  invoke?: <T>(command: string, args?: Record<string, unknown>) => Promise<T>;
};

let cachedPortalDesktopBaseUrl: string | null = null;
let cachedGatewayDesktopBaseUrl: string | null = null;
let cachedDesktopRuntimeSnapshot: PortalDesktopRuntimeSnapshot | null | undefined = undefined;

export class PortalApiError extends Error {
  constructor(message: string, readonly status: number) {
    super(message);
  }
}

export function portalBaseUrl(): string {
  return cachedPortalDesktopBaseUrl ?? portalProxyPrefix;
}

function resolveWindow(): TauriWindowLike | null {
  if (typeof window === 'undefined') {
    return null;
  }

  return window as TauriWindowLike;
}

function isDesktopRuntime(): boolean {
  const currentWindow = resolveWindow();
  return Boolean(
    currentWindow?.isTauri ||
      currentWindow?.__TAURI__ ||
      currentWindow?.__TAURI_INTERNALS__,
  );
}

function trimTrailingSlash(value: string): string {
  return value.replace(/\/+$/g, '');
}

function joinUrl(baseUrl: string, path: string): string {
  const normalizedBase = trimTrailingSlash(baseUrl);
  const normalizedPath = path.startsWith('/') ? path : `/${path}`;
  return `${normalizedBase}${normalizedPath}`;
}

function bindAddrUrl(bindAddr: string, path: string): string {
  const normalized = bindAddr.trim();
  const baseUrl = /^https?:\/\//.test(normalized) ? normalized : `http://${normalized}`;
  return joinUrl(baseUrl, path);
}

async function invokeDesktopCommand<T>(
  command: string,
  args?: Record<string, unknown>,
): Promise<T> {
  const invoke = resolveWindow()?.__TAURI_INTERNALS__?.invoke;
  if (typeof invoke !== 'function') {
    throw new Error('Tauri invoke bridge is unavailable.');
  }

  return invoke<T>(command, args);
}

async function resolvePortalBaseUrl(): Promise<string> {
  if (cachedPortalDesktopBaseUrl) {
    return cachedPortalDesktopBaseUrl;
  }

  if (!isDesktopRuntime()) {
    return portalProxyPrefix;
  }

  try {
    const runtimeBaseUrl = await invokeDesktopCommand<string>('runtime_base_url');
    const normalizedBaseUrl = runtimeBaseUrl?.trim();
    if (normalizedBaseUrl) {
      cachedPortalDesktopBaseUrl = joinUrl(normalizedBaseUrl, portalProxyPrefix);
      return cachedPortalDesktopBaseUrl;
    }
  } catch {
    // Fall back to the browser-style relative proxy path when the desktop bridge is unavailable.
  }

  return portalProxyPrefix;
}

export async function resolveGatewayBaseUrl(): Promise<string> {
  if (cachedGatewayDesktopBaseUrl) {
    return cachedGatewayDesktopBaseUrl;
  }

  if (isDesktopRuntime()) {
    try {
      const runtimeBaseUrl = await invokeDesktopCommand<string>('runtime_base_url');
      const normalizedBaseUrl = runtimeBaseUrl?.trim();
      if (normalizedBaseUrl) {
        cachedGatewayDesktopBaseUrl = joinUrl(normalizedBaseUrl, gatewayProxyPrefix);
        return cachedGatewayDesktopBaseUrl;
      }
    } catch {
      // Fall back to the local standalone gateway bind when the desktop bridge is unavailable.
    }
  }

  const currentWindow = resolveWindow();
  const currentOrigin = currentWindow?.location?.origin?.trim();
  const currentPort = currentWindow?.location?.port?.trim();

  if (currentOrigin && currentPort && standalonePortalDevPorts.has(currentPort)) {
    return standaloneGatewayBaseUrl;
  }

  if (currentOrigin) {
    return joinUrl(currentOrigin, gatewayProxyPrefix);
  }

  return standaloneGatewayBaseUrl;
}

export async function getDesktopRuntimeSnapshot(): Promise<PortalDesktopRuntimeSnapshot | null> {
  if (cachedDesktopRuntimeSnapshot !== undefined) {
    return cachedDesktopRuntimeSnapshot;
  }

  if (!isDesktopRuntime()) {
    cachedDesktopRuntimeSnapshot = null;
    return cachedDesktopRuntimeSnapshot;
  }

  try {
    cachedDesktopRuntimeSnapshot = await invokeDesktopCommand<PortalDesktopRuntimeSnapshot>(
      'runtime_desktop_snapshot',
    );
    return cachedDesktopRuntimeSnapshot;
  } catch {
    cachedDesktopRuntimeSnapshot = null;
    return cachedDesktopRuntimeSnapshot;
  }
}

export async function restartDesktopRuntime(): Promise<PortalDesktopRuntimeSnapshot> {
  const snapshot = await invokeDesktopCommand<PortalDesktopRuntimeSnapshot>(
    'restart_product_runtime',
  );

  cachedDesktopRuntimeSnapshot = snapshot;

  const runtimeBaseUrl = snapshot.publicBaseUrl?.trim();
  if (runtimeBaseUrl) {
    cachedPortalDesktopBaseUrl = joinUrl(runtimeBaseUrl, portalProxyPrefix);
    cachedGatewayDesktopBaseUrl = joinUrl(runtimeBaseUrl, gatewayProxyPrefix);
  } else {
    cachedPortalDesktopBaseUrl = null;
    cachedGatewayDesktopBaseUrl = null;
  }

  return snapshot;
}

type ProductHealthTarget = {
  id: PortalRuntimeServiceHealth['id'];
  label: string;
  healthUrl: string;
  detail: string;
};

function desktopHealthTargets(snapshot: PortalDesktopRuntimeSnapshot): ProductHealthTarget[] {
  const publicBaseUrl =
    snapshot.publicBaseUrl?.trim()
    || (snapshot.publicBindAddr ? bindAddrUrl(snapshot.publicBindAddr, '/') : null)
    || resolveWindow()?.location?.origin?.trim()
    || 'http://127.0.0.1';

  return [
    {
      id: 'web',
      label: 'Web entrypoint',
      healthUrl: joinUrl(publicBaseUrl, '/'),
      detail:
        'The public web host is serving the integrated router product shell and public entrypoint.',
    },
    {
      id: 'gateway',
      label: 'Gateway',
      healthUrl: snapshot.gatewayBindAddr
        ? bindAddrUrl(snapshot.gatewayBindAddr, '/health')
        : joinUrl(publicBaseUrl, '/api/health'),
      detail:
        'The gateway role is responding directly on its runtime health route.',
    },
    {
      id: 'admin',
      label: 'Admin control plane',
      healthUrl: snapshot.adminBindAddr
        ? bindAddrUrl(snapshot.adminBindAddr, '/admin/health')
        : joinUrl(publicBaseUrl, '/api/admin/health'),
      detail:
        'The admin role is reachable and can accept operator traffic on the current runtime.',
    },
    {
      id: 'portal',
      label: 'Portal API',
      healthUrl: snapshot.portalBindAddr
        ? bindAddrUrl(snapshot.portalBindAddr, '/portal/health')
        : joinUrl(publicBaseUrl, '/api/portal/health'),
      detail:
        'The portal role is reachable for authentication, workspace reads, and commerce workflows.',
    },
  ];
}

async function browserHealthTargets(): Promise<ProductHealthTarget[]> {
  const currentOrigin = resolveWindow()?.location?.origin?.trim() ?? 'http://127.0.0.1:3001';
  const gatewayBaseUrl = await resolveGatewayBaseUrl();

  return [
    {
      id: 'web',
      label: 'Web entrypoint',
      healthUrl: joinUrl(currentOrigin, '/'),
      detail:
        'The public product entrypoint is serving the router shell for hosted or browser sessions.',
    },
    {
      id: 'gateway',
      label: 'Gateway',
      healthUrl: joinUrl(gatewayBaseUrl, '/health'),
      detail:
        'Gateway health is checked through the current public or standalone gateway surface.',
    },
    {
      id: 'admin',
      label: 'Admin control plane',
      healthUrl: joinUrl(currentOrigin, '/api/admin/health'),
      detail:
        'The admin role is checked through the public product host when the desktop bridge is unavailable.',
    },
    {
      id: 'portal',
      label: 'Portal API',
      healthUrl: joinUrl(currentOrigin, '/api/portal/health'),
      detail:
        'The portal role is checked through the public product host when the desktop bridge is unavailable.',
    },
  ];
}

async function probeProductHealthTarget(
  target: ProductHealthTarget,
): Promise<PortalRuntimeServiceHealth> {
  const startedAt = Date.now();
  const controller = typeof AbortController === 'function' ? new AbortController() : null;
  const timeoutId = controller
    ? globalThis.setTimeout(() => controller.abort(), 2_000)
    : null;

  try {
    const response = await fetch(target.healthUrl, {
      method: 'GET',
      signal: controller?.signal,
    });
    if (timeoutId !== null) {
      globalThis.clearTimeout(timeoutId);
    }

    return {
      id: target.id,
      label: target.label,
      status: response.ok ? 'healthy' : 'degraded',
      healthUrl: target.healthUrl,
      detail: response.ok
        ? target.detail
        : `${target.label} returned HTTP ${response.status} on its health route.`,
      httpStatus: response.status,
      responseTimeMs: Date.now() - startedAt,
    };
  } catch (error) {
    if (timeoutId !== null) {
      globalThis.clearTimeout(timeoutId);
    }

    return {
      id: target.id,
      label: target.label,
      status: 'unreachable',
      healthUrl: target.healthUrl,
      detail:
        error instanceof Error
          ? `${target.label} is unreachable from the current session: ${error.message}`
          : `${target.label} is unreachable from the current session.`,
      httpStatus: null,
      responseTimeMs: null,
    };
  }
}

export async function getProductRuntimeHealthSnapshot(): Promise<PortalRuntimeHealthSnapshot> {
  const desktopRuntime = await getDesktopRuntimeSnapshot();
  const targets = desktopRuntime
    ? desktopHealthTargets(desktopRuntime)
    : await browserHealthTargets();

  return {
    mode: desktopRuntime?.mode ?? 'browser',
    checkedAtMs: Date.now(),
    services: await Promise.all(targets.map((target) => probeProductHealthTarget(target))),
  };
}

export function readPortalSessionToken(): string | null {
  return globalThis.localStorage?.getItem(portalSessionTokenKey) ?? null;
}

export function persistPortalSessionToken(token: string): void {
  globalThis.localStorage?.setItem(portalSessionTokenKey, token);
}

export function clearPortalSessionToken(): void {
  globalThis.localStorage?.removeItem(portalSessionTokenKey);
}

async function readJson<T>(response: Response): Promise<T> {
  if (!response.ok) {
    let message = `Portal request failed with status ${response.status}`;

    try {
      const payload = (await response.json()) as { error?: { message?: string } };
      message = payload.error?.message ?? message;
    } catch {
      // Fall back to the status-based message for non-JSON failures.
    }

    if (response.status === 401) {
      globalThis.dispatchEvent?.(new CustomEvent(portalSessionExpiredEvent));
    }

    throw new PortalApiError(message, response.status);
  }

  return (await response.json()) as T;
}

export function onPortalSessionExpired(handler: () => void): () => void {
  const listener = () => handler();
  globalThis.addEventListener?.(portalSessionExpiredEvent, listener);
  return () => globalThis.removeEventListener?.(portalSessionExpiredEvent, listener);
}

function requiredPortalToken(providedToken?: string): string {
  const token = providedToken ?? readPortalSessionToken();
  if (!token) {
    throw new PortalApiError('Portal session token not found', 401);
  }
  return token;
}

async function getJson<T>(path: string, token?: string): Promise<T> {
  const response = await fetch(`${await resolvePortalBaseUrl()}${path}`, {
    headers: token
      ? {
          authorization: `Bearer ${token}`,
        }
      : undefined,
  });
  return readJson<T>(response);
}

async function postJson<TRequest, TResponse>(
  path: string,
  body: TRequest,
  token?: string,
): Promise<TResponse> {
  const headers: Record<string, string> = {
    'content-type': 'application/json',
  };
  if (token) {
    headers.authorization = `Bearer ${token}`;
  }

  const response = await fetch(`${await resolvePortalBaseUrl()}${path}`, {
    method: 'POST',
    headers,
    body: JSON.stringify(body),
  });

  return readJson<TResponse>(response);
}

async function patchJson<TRequest, TResponse>(
  path: string,
  body: TRequest,
  token?: string,
): Promise<TResponse> {
  const headers: Record<string, string> = {
    'content-type': 'application/json',
  };
  if (token) {
    headers.authorization = `Bearer ${token}`;
  }

  const response = await fetch(`${await resolvePortalBaseUrl()}${path}`, {
    method: 'PATCH',
    headers,
    body: JSON.stringify(body),
  });

  return readJson<TResponse>(response);
}

async function deleteEmpty(path: string, token?: string): Promise<void> {
  const headers: Record<string, string> = {};
  if (token) {
    headers.authorization = `Bearer ${token}`;
  }

  const response = await fetch(`${await resolvePortalBaseUrl()}${path}`, {
    method: 'DELETE',
    headers,
  });

  if (!response.ok) {
    await readJson(response);
  }
}

export function portalErrorMessage(error: unknown): string {
  if (error instanceof PortalApiError) {
    return error.message;
  }
  if (error instanceof Error) {
    return error.message;
  }
  return 'Portal request failed.';
}

export function registerPortalUser(input: {
  email: string;
  password: string;
  display_name: string;
}): Promise<PortalAuthSession> {
  return postJson<typeof input, PortalAuthSession>('/auth/register', input);
}

export function loginPortalUser(input: {
  email: string;
  password: string;
}): Promise<PortalAuthSession> {
  return postJson<typeof input, PortalAuthSession>('/auth/login', input);
}

export function getPortalMe(token?: string): Promise<PortalUserProfile> {
  return getJson<PortalUserProfile>('/auth/me', requiredPortalToken(token));
}

export function changePortalPassword(
  input: { current_password: string; new_password: string },
  token?: string,
): Promise<PortalUserProfile> {
  return postJson<typeof input, PortalUserProfile>(
    '/auth/change-password',
    input,
    requiredPortalToken(token),
  );
}

export function getPortalWorkspace(token?: string): Promise<PortalWorkspaceSummary> {
  return getJson<PortalWorkspaceSummary>('/workspace', requiredPortalToken(token));
}

export function getPortalDashboard(token?: string): Promise<PortalDashboardSummary> {
  return getJson<PortalDashboardSummary>('/dashboard', requiredPortalToken(token));
}

export function getPortalGatewayRateLimitSnapshot(
  token?: string,
): Promise<PortalGatewayRateLimitSnapshot> {
  return getJson<PortalGatewayRateLimitSnapshot>(
    '/gateway/rate-limit-snapshot',
    requiredPortalToken(token),
  );
}

export function listPortalApiKeys(token?: string): Promise<GatewayApiKeyRecord[]> {
  return getJson<GatewayApiKeyRecord[]>('/api-keys', requiredPortalToken(token));
}

export function createPortalApiKey(
  input: {
    environment: string;
    label: string;
    api_key?: string | null;
    api_key_group_id?: string | null;
    notes?: string | null;
    expires_at_ms?: number | null;
  },
  token?: string,
): Promise<CreatedGatewayApiKey> {
  return postJson<typeof input, CreatedGatewayApiKey>(
    '/api-keys',
    input,
    requiredPortalToken(token),
  );
}

export function updatePortalApiKeyStatus(
  hashedKey: string,
  active: boolean,
  token?: string,
): Promise<GatewayApiKeyRecord> {
  return postJson<{ active: boolean }, GatewayApiKeyRecord>(
    `/api-keys/${encodeURIComponent(hashedKey)}/status`,
    { active },
    requiredPortalToken(token),
  );
}

export function deletePortalApiKey(hashedKey: string, token?: string): Promise<void> {
  return deleteEmpty(`/api-keys/${encodeURIComponent(hashedKey)}`, requiredPortalToken(token));
}

type PortalApiKeyGroupMutationInput = {
  environment: string;
  name: string;
  slug?: string | null;
  description?: string | null;
  color?: string | null;
  default_capability_scope?: string | null;
  default_accounting_mode?: string | null;
  default_routing_profile_id?: string | null;
};

export function listPortalApiKeyGroups(token?: string): Promise<ApiKeyGroupRecord[]> {
  return getJson<ApiKeyGroupRecord[]>('/api-key-groups', requiredPortalToken(token));
}

export function createPortalApiKeyGroup(
  input: PortalApiKeyGroupMutationInput,
  token?: string,
): Promise<ApiKeyGroupRecord> {
  return postJson<PortalApiKeyGroupMutationInput, ApiKeyGroupRecord>(
    '/api-key-groups',
    input,
    requiredPortalToken(token),
  );
}

export function updatePortalApiKeyGroup(
  groupId: string,
  input: PortalApiKeyGroupMutationInput,
  token?: string,
): Promise<ApiKeyGroupRecord> {
  return patchJson<PortalApiKeyGroupMutationInput, ApiKeyGroupRecord>(
    `/api-key-groups/${encodeURIComponent(groupId)}`,
    input,
    requiredPortalToken(token),
  );
}

export function updatePortalApiKeyGroupStatus(
  groupId: string,
  active: boolean,
  token?: string,
): Promise<ApiKeyGroupRecord> {
  return postJson<{ active: boolean }, ApiKeyGroupRecord>(
    `/api-key-groups/${encodeURIComponent(groupId)}/status`,
    { active },
    requiredPortalToken(token),
  );
}

export function deletePortalApiKeyGroup(groupId: string, token?: string): Promise<void> {
  return deleteEmpty(
    `/api-key-groups/${encodeURIComponent(groupId)}`,
    requiredPortalToken(token),
  );
}

type RoutingProfileWireRecord = Omit<RoutingProfileRecord, 'active' | 'require_healthy'> & {
  active?: boolean;
  require_healthy?: boolean;
};

function normalizeRoutingProfileRecord(profile: RoutingProfileWireRecord): RoutingProfileRecord {
  return {
    ...profile,
    active: profile.active ?? false,
    require_healthy: profile.require_healthy ?? false,
  };
}

type PortalCompiledRoutingSnapshotWireRecord = Omit<
  PortalCompiledRoutingSnapshotRecord,
  'require_healthy'
> & {
  require_healthy?: boolean;
};

function normalizeCompiledRoutingSnapshotRecord(
  snapshot: PortalCompiledRoutingSnapshotWireRecord,
): PortalCompiledRoutingSnapshotRecord {
  return {
    ...snapshot,
    require_healthy: snapshot.require_healthy ?? false,
  };
}

export function listPortalRoutingProfiles(token?: string): Promise<RoutingProfileRecord[]> {
  return getJson<RoutingProfileWireRecord[]>(
    '/routing/profiles',
    requiredPortalToken(token),
  ).then((profiles) => profiles.map(normalizeRoutingProfileRecord));
}

export function listPortalRoutingSnapshots(
  token?: string,
): Promise<PortalCompiledRoutingSnapshotRecord[]> {
  return getJson<PortalCompiledRoutingSnapshotWireRecord[]>(
    '/routing/snapshots',
    requiredPortalToken(token),
  ).then((snapshots) => snapshots.map(normalizeCompiledRoutingSnapshotRecord));
}

export function createPortalRoutingProfile(
  input: {
    name: string;
    slug?: string | null;
    description?: string | null;
    active?: boolean;
    strategy?: string;
    ordered_provider_ids?: string[];
    default_provider_id?: string | null;
    max_cost?: number | null;
    max_latency_ms?: number | null;
    require_healthy?: boolean;
    preferred_region?: string | null;
  },
  token?: string,
): Promise<RoutingProfileRecord> {
  return postJson<typeof input, RoutingProfileWireRecord>(
    '/routing/profiles',
    input,
    requiredPortalToken(token),
  ).then(normalizeRoutingProfileRecord);
}

export function listPortalUsageRecords(token?: string): Promise<UsageRecord[]> {
  return getJson<UsageRecord[]>('/usage/records', requiredPortalToken(token));
}

export function getPortalUsageSummary(token?: string): Promise<UsageSummary> {
  return getJson<UsageSummary>('/usage/summary', requiredPortalToken(token));
}

export function getPortalBillingSummary(token?: string): Promise<ProjectBillingSummary> {
  return getJson<ProjectBillingSummary>('/billing/summary', requiredPortalToken(token));
}

export function getPortalCommercialAccount(
  token?: string,
): Promise<CommercialAccountSummary> {
  return getJson<CommercialAccountSummary>('/billing/account', requiredPortalToken(token));
}

export function getPortalCommercialAccountHistory(
  token?: string,
): Promise<CommercialAccountHistorySnapshot> {
  return getJson<CommercialAccountHistorySnapshot>(
    '/billing/account-history',
    requiredPortalToken(token),
  );
}

export function getPortalCommercialAccountBalance(
  token?: string,
): Promise<CommercialAccountBalanceSnapshot> {
  return getJson<CommercialAccountBalanceSnapshot>(
    '/billing/account/balance',
    requiredPortalToken(token),
  );
}

export function listPortalCommercialBenefitLots(
  token?: string,
): Promise<CommercialAccountBenefitLotRecord[]> {
  return getJson<CommercialAccountBenefitLotRecord[]>(
    '/billing/account/benefit-lots',
    requiredPortalToken(token),
  );
}

export function listPortalCommercialHolds(
  token?: string,
): Promise<CommercialAccountHoldRecord[]> {
  return getJson<CommercialAccountHoldRecord[]>(
    '/billing/account/holds',
    requiredPortalToken(token),
  );
}

export function listPortalCommercialRequestSettlements(
  token?: string,
): Promise<CommercialRequestSettlementRecord[]> {
  return getJson<CommercialRequestSettlementRecord[]>(
    '/billing/account/request-settlements',
    requiredPortalToken(token),
  );
}

export function listPortalCommercialPricingPlans(
  token?: string,
): Promise<CommercialPricingPlanRecord[]> {
  return getJson<CommercialPricingPlanRecord[]>(
    '/billing/pricing-plans',
    requiredPortalToken(token),
  );
}

export function listPortalCommercialPricingRates(
  token?: string,
): Promise<CommercialPricingRateRecord[]> {
  return getJson<CommercialPricingRateRecord[]>(
    '/billing/pricing-rates',
    requiredPortalToken(token),
  );
}

export function getPortalBillingEvents(token?: string): Promise<BillingEventRecord[]> {
  return getJson<BillingEventRecord[]>('/billing/events', requiredPortalToken(token));
}

export function getPortalBillingEventSummary(token?: string): Promise<BillingEventSummary> {
  return getJson<BillingEventSummary>('/billing/events/summary', requiredPortalToken(token));
}

export function listPortalBillingLedger(token?: string): Promise<LedgerEntry[]> {
  return getJson<LedgerEntry[]>('/billing/ledger', requiredPortalToken(token));
}

export function validatePortalCoupon(
  input: PortalCouponValidationRequest,
  token?: string,
): Promise<PortalCouponValidationResponse> {
  return postJson<PortalCouponValidationRequest, PortalCouponValidationResponse>(
    '/marketing/coupon-validations',
    input,
    requiredPortalToken(token),
  );
}

export function reservePortalCouponRedemption(
  input: PortalCouponReservationRequest,
  token?: string,
): Promise<PortalCouponReservationResponse> {
  return postJson<PortalCouponReservationRequest, PortalCouponReservationResponse>(
    '/marketing/coupon-reservations',
    input,
    requiredPortalToken(token),
  );
}

export function confirmPortalCouponRedemption(
  input: PortalCouponRedemptionConfirmRequest,
  token?: string,
): Promise<PortalCouponRedemptionConfirmResponse> {
  return postJson<
    PortalCouponRedemptionConfirmRequest,
    PortalCouponRedemptionConfirmResponse
  >(
    '/marketing/coupon-redemptions/confirm',
    input,
    requiredPortalToken(token),
  );
}

export function rollbackPortalCouponRedemption(
  input: PortalCouponRedemptionRollbackRequest,
  token?: string,
): Promise<PortalCouponRedemptionRollbackResponse> {
  return postJson<
    PortalCouponRedemptionRollbackRequest,
    PortalCouponRedemptionRollbackResponse
  >(
    '/marketing/coupon-redemptions/rollback',
    input,
    requiredPortalToken(token),
  );
}

export function listPortalMarketingMyCoupons(
  token?: string,
): Promise<PortalMarketingCodesResponse> {
  return getJson<PortalMarketingCodesResponse>(
    '/marketing/my-coupons',
    requiredPortalToken(token),
  );
}

export function listPortalMarketingRewardHistory(
  token?: string,
): Promise<PortalMarketingRewardHistoryItem[]> {
  return getJson<PortalMarketingRewardHistoryItem[]>(
    '/marketing/reward-history',
    requiredPortalToken(token),
  );
}

export function listPortalMarketingRedemptions(
  token?: string,
): Promise<PortalMarketingRedemptionsResponse> {
  return getJson<PortalMarketingRedemptionsResponse>(
    '/marketing/redemptions',
    requiredPortalToken(token),
  );
}

export function listPortalMarketingCodes(
  token?: string,
): Promise<PortalMarketingCodesResponse> {
  return getJson<PortalMarketingCodesResponse>(
    '/marketing/codes',
    requiredPortalToken(token),
  );
}

export function getPortalCommerceCatalog(token?: string): Promise<PortalCommerceCatalog> {
  return getJson<PortalCommerceCatalog>('/commerce/catalog', requiredPortalToken(token));
}

export function previewPortalCommerceQuote(
  input: PortalCommerceQuoteRequest,
  token?: string,
): Promise<PortalCommerceQuote> {
  return postJson<PortalCommerceQuoteRequest, PortalCommerceQuote>(
    '/commerce/quote',
    input,
    requiredPortalToken(token),
  );
}

export function createPortalCommerceOrder(
  input: PortalCommerceQuoteRequest,
  token?: string,
): Promise<PortalCommerceOrder> {
  return postJson<PortalCommerceQuoteRequest, PortalCommerceOrder>(
    '/commerce/orders',
    input,
    requiredPortalToken(token),
  );
}

export function getPortalCommerceOrder(
  orderId: string,
  token?: string,
): Promise<PortalCommerceOrder> {
  return getJson<PortalCommerceOrder>(
    `/commerce/orders/${encodeURIComponent(orderId)}`,
    requiredPortalToken(token),
  );
}

export function settlePortalCommerceOrder(
  orderId: string,
  token?: string,
): Promise<PortalCommerceOrder> {
  return postJson<Record<string, never>, PortalCommerceOrder>(
    `/commerce/orders/${encodeURIComponent(orderId)}/settle`,
    {},
    requiredPortalToken(token),
  );
}

export function cancelPortalCommerceOrder(
  orderId: string,
  token?: string,
): Promise<PortalCommerceOrder> {
  return postJson<Record<string, never>, PortalCommerceOrder>(
    `/commerce/orders/${encodeURIComponent(orderId)}/cancel`,
    {},
    requiredPortalToken(token),
  );
}

export function sendPortalCommercePaymentEvent(
  orderId: string,
  input: PortalCommercePaymentEventRequest,
  token?: string,
): Promise<PortalCommerceOrder> {
  return postJson<PortalCommercePaymentEventRequest, PortalCommerceOrder>(
    `/commerce/orders/${encodeURIComponent(orderId)}/payment-events`,
    input,
    requiredPortalToken(token),
  );
}

export function getPortalCommerceCheckoutSession(
  orderId: string,
  token?: string,
): Promise<PortalCommerceCheckoutSession> {
  return getJson<PortalCommerceCheckoutSession>(
    `/commerce/orders/${encodeURIComponent(orderId)}/checkout-session`,
    requiredPortalToken(token),
  );
}

export function listPortalCommerceOrders(token?: string): Promise<PortalCommerceOrder[]> {
  return getJson<PortalCommerceOrder[]>('/commerce/orders', requiredPortalToken(token));
}

export function listPortalCommercePaymentMethods(
  orderId: string,
  token?: string,
): Promise<PaymentMethodRecord[]> {
  return getJson<PaymentMethodRecord[]>(
    `/commerce/orders/${encodeURIComponent(orderId)}/payment-methods`,
    requiredPortalToken(token),
  );
}

export function listPortalCommercePaymentAttempts(
  orderId: string,
  token?: string,
): Promise<CommercePaymentAttemptRecord[]> {
  return getJson<CommercePaymentAttemptRecord[]>(
    `/commerce/orders/${encodeURIComponent(orderId)}/payment-attempts`,
    requiredPortalToken(token),
  );
}

export function createPortalCommercePaymentAttempt(
  orderId: string,
  input: PortalCommercePaymentAttemptCreateRequest,
  token?: string,
): Promise<CommercePaymentAttemptRecord> {
  return postJson<PortalCommercePaymentAttemptCreateRequest, CommercePaymentAttemptRecord>(
    `/commerce/orders/${encodeURIComponent(orderId)}/payment-attempts`,
    input,
    requiredPortalToken(token),
  );
}

export function getPortalCommerceOrderCenter(
  token?: string,
): Promise<PortalCommerceOrderCenterResponse> {
  return getJson<PortalCommerceOrderCenterResponse>(
    '/commerce/order-center',
    requiredPortalToken(token),
  );
}

export function getPortalCommercePaymentAttempt(
  paymentAttemptId: string,
  token?: string,
): Promise<CommercePaymentAttemptRecord> {
  return getJson<CommercePaymentAttemptRecord>(
    `/commerce/payment-attempts/${encodeURIComponent(paymentAttemptId)}`,
    requiredPortalToken(token),
  );
}

export function getPortalCommerceMembership(
  token?: string,
): Promise<PortalCommerceMembership | null> {
  return getJson<PortalCommerceMembership | null>(
    '/commerce/membership',
    requiredPortalToken(token),
  );
}

export function getPortalRoutingSummary(token?: string): Promise<PortalRoutingSummary> {
  return getJson<PortalRoutingSummary>('/routing/summary', requiredPortalToken(token));
}

export function getPortalRoutingPreferences(token?: string): Promise<PortalRoutingPreferences> {
  return getJson<PortalRoutingPreferences>('/routing/preferences', requiredPortalToken(token));
}

export function savePortalRoutingPreferences(
  input: {
    preset_id: string;
    strategy: PortalRoutingPreferences['strategy'];
    ordered_provider_ids: string[];
    default_provider_id?: string | null;
    max_cost?: number | null;
    max_latency_ms?: number | null;
    require_healthy: boolean;
    preferred_region?: string | null;
  },
  token?: string,
): Promise<PortalRoutingPreferences> {
  return postJson<typeof input, PortalRoutingPreferences>(
    '/routing/preferences',
    input,
    requiredPortalToken(token),
  );
}

export function previewPortalRouting(
  input: {
    capability: string;
    model: string;
    requested_region?: string | null;
    selection_seed?: number | null;
  },
  token?: string,
): Promise<PortalRoutingDecision> {
  return postJson<typeof input, PortalRoutingDecision>(
    '/routing/preview',
    input,
    requiredPortalToken(token),
  );
}

export function listPortalRoutingDecisionLogs(
  token?: string,
): Promise<PortalRoutingDecisionLog[]> {
  return getJson<PortalRoutingDecisionLog[]>(
    '/routing/decision-logs',
    requiredPortalToken(token),
  );
}
