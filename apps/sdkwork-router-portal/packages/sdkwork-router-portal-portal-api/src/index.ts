import type {
  CreatedGatewayApiKey,
  GatewayApiKeyRecord,
  LedgerEntry,
  PortalAuthSession,
  PortalDashboardSummary,
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

type TauriWindowLike = Window & {
  __TAURI__?: unknown;
  __TAURI_INTERNALS__?: TauriInternalsLike;
  isTauri?: boolean;
};

type TauriInternalsLike = {
  invoke?: <T>(command: string, args?: Record<string, unknown>) => Promise<T>;
};

let cachedPortalDesktopBaseUrl: string | null = null;

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
