import type {
  ChannelRecord,
  GatewayApiKeyRecord,
  LedgerEntry,
  ModelCatalogRecord,
  ProjectRecord,
  ProxyProviderRecord,
  RoutingSimulationResult,
  TenantRecord,
  UsageRecord,
} from 'sdkwork-api-types';

export class AdminApiError extends Error {
  constructor(message: string, readonly status: number) {
    super(message);
  }
}

export function adminBaseUrl(): string {
  return '/admin';
}

async function readJson<T>(response: Response): Promise<T> {
  if (!response.ok) {
    throw new AdminApiError(`Admin request failed with status ${response.status}`, response.status);
  }

  return (await response.json()) as T;
}

async function getJson<T>(path: string): Promise<T> {
  const response = await fetch(`${adminBaseUrl()}${path}`);
  return readJson<T>(response);
}

async function postJson<TRequest, TResponse>(path: string, body: TRequest): Promise<TResponse> {
  const response = await fetch(`${adminBaseUrl()}${path}`, {
    method: 'POST',
    headers: {
      'content-type': 'application/json',
    },
    body: JSON.stringify(body),
  });

  return readJson<TResponse>(response);
}

export function listTenants(): Promise<TenantRecord[]> {
  return getJson<TenantRecord[]>('/tenants');
}

export function listProjects(): Promise<ProjectRecord[]> {
  return getJson<ProjectRecord[]>('/projects');
}

export function listApiKeys(): Promise<GatewayApiKeyRecord[]> {
  return getJson<GatewayApiKeyRecord[]>('/api-keys');
}

export function listChannels(): Promise<ChannelRecord[]> {
  return getJson<ChannelRecord[]>('/channels');
}

export function listProviders(): Promise<ProxyProviderRecord[]> {
  return getJson<ProxyProviderRecord[]>('/providers');
}

export function listModels(): Promise<ModelCatalogRecord[]> {
  return getJson<ModelCatalogRecord[]>('/models');
}

export function listUsageRecords(): Promise<UsageRecord[]> {
  return getJson<UsageRecord[]>('/usage/records');
}

export function listLedgerEntries(): Promise<LedgerEntry[]> {
  return getJson<LedgerEntry[]>('/billing/ledger');
}

export function simulateRoute(
  capability: string,
  model: string,
  selectionSeed?: number,
): Promise<RoutingSimulationResult> {
  return postJson<{ capability: string; model: string; selection_seed?: number }, RoutingSimulationResult>(
    '/routing/simulations',
    { capability, model, selection_seed: selectionSeed },
  );
}
