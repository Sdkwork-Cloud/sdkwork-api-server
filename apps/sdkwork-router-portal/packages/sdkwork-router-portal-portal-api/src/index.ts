import type {
  CreatedGatewayApiKey,
  GatewayApiKeyRecord,
  LedgerEntry,
  PortalCommerceCheckoutSession,
  PortalCommercePaymentEventRequest,
  PortalCommerceQuote,
  PortalCommerceOrder,
  PortalCommerceMembership,
  PortalCommerceQuoteRequest,
  PortalCommerceCatalog,
  PortalDesktopRuntimeSnapshot,
  PortalRuntimeHealthSnapshot,
  PortalRuntimeServiceHealth,
  PortalAuthSession,
  PortalDashboardSummary,
  PortalGatewayRateLimitSnapshot,
  PortalRoutingDecision,
  PortalRoutingDecisionLog,
  PortalRoutingPreferences,
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

export function listPortalUsageRecords(token?: string): Promise<UsageRecord[]> {
  return getJson<UsageRecord[]>('/usage/records', requiredPortalToken(token));
}

export function getPortalUsageSummary(token?: string): Promise<UsageSummary> {
  return getJson<UsageSummary>('/usage/summary', requiredPortalToken(token));
}

export function getPortalBillingSummary(token?: string): Promise<ProjectBillingSummary> {
  return getJson<ProjectBillingSummary>('/billing/summary', requiredPortalToken(token));
}

export function listPortalBillingLedger(token?: string): Promise<LedgerEntry[]> {
  return getJson<LedgerEntry[]>('/billing/ledger', requiredPortalToken(token));
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
