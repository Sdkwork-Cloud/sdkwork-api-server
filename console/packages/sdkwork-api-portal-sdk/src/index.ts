import type {
  CreatedGatewayApiKey,
  GatewayApiKeyRecord,
  PortalAuthSession,
  PortalUserProfile,
  PortalWorkspaceSummary,
} from 'sdkwork-api-types';

const portalSessionTokenKey = 'sdkwork.portal.session-token';

export class PortalApiError extends Error {
  constructor(message: string, readonly status: number) {
    super(message);
  }
}

export function portalBaseUrl(): string {
  return '/portal';
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
      // Ignore non-JSON error bodies and fall back to the status-based message.
    }

    throw new PortalApiError(message, response.status);
  }

  return (await response.json()) as T;
}

async function getJson<T>(path: string, token?: string): Promise<T> {
  const response = await fetch(`${portalBaseUrl()}${path}`, {
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

  const response = await fetch(`${portalBaseUrl()}${path}`, {
    method: 'POST',
    headers,
    body: JSON.stringify(body),
  });

  return readJson<TResponse>(response);
}

function requiredPortalToken(providedToken?: string): string {
  const token = providedToken ?? readPortalSessionToken();
  if (!token) {
    throw new PortalApiError('Portal session token not found', 401);
  }
  return token;
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

export function getPortalWorkspace(token?: string): Promise<PortalWorkspaceSummary> {
  return getJson<PortalWorkspaceSummary>('/workspace', requiredPortalToken(token));
}

export function listPortalApiKeys(token?: string): Promise<GatewayApiKeyRecord[]> {
  return getJson<GatewayApiKeyRecord[]>('/api-keys', requiredPortalToken(token));
}

export function createPortalApiKey(
  environment: string,
  token?: string,
): Promise<CreatedGatewayApiKey> {
  return postJson<{ environment: string }, CreatedGatewayApiKey>(
    '/api-keys',
    { environment },
    requiredPortalToken(token),
  );
}
