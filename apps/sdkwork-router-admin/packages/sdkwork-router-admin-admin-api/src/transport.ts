const adminSessionTokenKey = 'sdkwork.router.admin.session-token';
const adminProxyPrefix = '/api/admin';

type TauriWindowLike = Window & {
  __TAURI__?: unknown;
  __TAURI_INTERNALS__?: TauriInternalsLike;
  isTauri?: boolean;
};

type TauriInternalsLike = {
  invoke?: <T>(command: string, args?: Record<string, unknown>) => Promise<T>;
};

let cachedAdminDesktopBaseUrl: string | null = null;

export class AdminApiError extends Error {
  constructor(message: string, readonly status: number) {
    super(message);
  }
}

export function adminBaseUrl(): string {
  return cachedAdminDesktopBaseUrl ?? adminProxyPrefix;
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
    currentWindow?.isTauri
      || currentWindow?.__TAURI__
      || currentWindow?.__TAURI_INTERNALS__,
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

async function resolveAdminBaseUrl(): Promise<string> {
  if (cachedAdminDesktopBaseUrl) {
    return cachedAdminDesktopBaseUrl;
  }

  if (!isDesktopRuntime()) {
    return adminProxyPrefix;
  }

  try {
    const runtimeBaseUrl = await invokeDesktopCommand<string>('runtime_base_url');
    const normalizedBaseUrl = runtimeBaseUrl?.trim();
    if (normalizedBaseUrl) {
      cachedAdminDesktopBaseUrl = joinUrl(normalizedBaseUrl, adminProxyPrefix);
      return cachedAdminDesktopBaseUrl;
    }
  } catch {
    // Fall back to the browser-style relative proxy path when the desktop bridge is unavailable.
  }

  return adminProxyPrefix;
}

export function readAdminSessionToken(): string | null {
  return globalThis.localStorage?.getItem(adminSessionTokenKey) ?? null;
}

export function persistAdminSessionToken(token: string): void {
  globalThis.localStorage?.setItem(adminSessionTokenKey, token);
}

export function clearAdminSessionToken(): void {
  globalThis.localStorage?.removeItem(adminSessionTokenKey);
}

async function readJson<T>(response: Response): Promise<T> {
  if (!response.ok) {
    let message = `Admin request failed with status ${response.status}`;
    try {
      const payload = (await response.json()) as { error?: { message?: string } };
      message = payload.error?.message?.trim() || message;
    } catch {
      // Fall back to the generic transport status when the response is not JSON.
    }
    throw new AdminApiError(message, response.status);
  }

  return (await response.json()) as T;
}

export function requiredToken(token?: string): string {
  const sessionToken = token ?? readAdminSessionToken();
  if (!sessionToken) {
    throw new AdminApiError('Admin session token not found', 401);
  }
  return sessionToken;
}

export async function getJson<T>(path: string, token?: string): Promise<T> {
  const response = await fetch(`${await resolveAdminBaseUrl()}${path}`, {
    headers: {
      authorization: `Bearer ${requiredToken(token)}`,
    },
  });
  return readJson<T>(response);
}

export async function postJson<TRequest, TResponse>(
  path: string,
  body: TRequest,
  token?: string,
): Promise<TResponse> {
  const response = await fetch(`${await resolveAdminBaseUrl()}${path}`, {
    method: 'POST',
    headers: {
      'content-type': 'application/json',
      ...(token ? { authorization: `Bearer ${token}` } : {}),
    },
    body: JSON.stringify(body),
  });
  return readJson<TResponse>(response);
}

export async function patchJson<TRequest, TResponse>(
  path: string,
  body: TRequest,
  token?: string,
): Promise<TResponse> {
  const response = await fetch(`${await resolveAdminBaseUrl()}${path}`, {
    method: 'PATCH',
    headers: {
      'content-type': 'application/json',
      ...(token ? { authorization: `Bearer ${token}` } : {}),
    },
    body: JSON.stringify(body),
  });
  return readJson<TResponse>(response);
}

export async function putJson<TRequest, TResponse>(
  path: string,
  body: TRequest,
  token?: string,
): Promise<TResponse> {
  const response = await fetch(`${await resolveAdminBaseUrl()}${path}`, {
    method: 'PUT',
    headers: {
      'content-type': 'application/json',
      ...(token ? { authorization: `Bearer ${token}` } : {}),
    },
    body: JSON.stringify(body),
  });
  return readJson<TResponse>(response);
}

export async function deleteEmpty(path: string, token?: string): Promise<void> {
  const response = await fetch(`${await resolveAdminBaseUrl()}${path}`, {
    method: 'DELETE',
    headers: {
      authorization: `Bearer ${requiredToken(token)}`,
    },
  });

  if (!response.ok) {
    await readJson<never>(response);
  }
}
