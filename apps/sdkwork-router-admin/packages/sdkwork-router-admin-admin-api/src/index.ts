import type {
  AdminAuthSession,
  AdminSessionUser,
  BillingSummary,
  ChannelRecord,
  CouponRecord,
  CreatedGatewayApiKey,
  CredentialRecord,
  GatewayApiKeyRecord,
  ModelCatalogRecord,
  OperatorUserRecord,
  PortalUserRecord,
  ProjectRecord,
  ProviderHealthSnapshot,
  ProxyProviderRecord,
  RoutingDecisionLogRecord,
  RuntimeReloadReport,
  RuntimeStatusRecord,
  TenantRecord,
  UsageRecord,
  UsageSummary,
} from 'sdkwork-router-admin-types';

const adminSessionTokenKey = 'sdkwork.router.admin.session-token';

export class AdminApiError extends Error {
  constructor(message: string, readonly status: number) {
    super(message);
  }
}

export function adminBaseUrl(): string {
  return '/api/admin';
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

function requiredToken(token?: string): string {
  const sessionToken = token ?? readAdminSessionToken();
  if (!sessionToken) {
    throw new AdminApiError('Admin session token not found', 401);
  }
  return sessionToken;
}

async function getJson<T>(path: string, token?: string): Promise<T> {
  const response = await fetch(`${adminBaseUrl()}${path}`, {
    headers: {
      authorization: `Bearer ${requiredToken(token)}`,
    },
  });
  return readJson<T>(response);
}

async function postJson<TRequest, TResponse>(
  path: string,
  body: TRequest,
  token?: string,
): Promise<TResponse> {
  const response = await fetch(`${adminBaseUrl()}${path}`, {
    method: 'POST',
    headers: {
      'content-type': 'application/json',
      ...(token ? { authorization: `Bearer ${token}` } : {}),
    },
    body: JSON.stringify(body),
  });
  return readJson<TResponse>(response);
}

async function deleteEmpty(path: string, token?: string): Promise<void> {
  const response = await fetch(`${adminBaseUrl()}${path}`, {
    method: 'DELETE',
    headers: {
      authorization: `Bearer ${requiredToken(token)}`,
    },
  });

  if (!response.ok) {
    await readJson<never>(response);
  }
}

export function loginAdminUser(input: {
  email: string;
  password: string;
}): Promise<AdminAuthSession> {
  return postJson<typeof input, AdminAuthSession>('/auth/login', input);
}

export function getAdminMe(token?: string): Promise<AdminSessionUser> {
  return getJson<AdminSessionUser>('/auth/me', token);
}

export function listOperatorUsers(token?: string): Promise<OperatorUserRecord[]> {
  return getJson<OperatorUserRecord[]>('/users/operators', token);
}

export function saveOperatorUser(input: {
  id?: string;
  email: string;
  display_name: string;
  password?: string;
  active: boolean;
}): Promise<OperatorUserRecord> {
  return postJson<typeof input, OperatorUserRecord>('/users/operators', input, requiredToken());
}

export function updateOperatorUserStatus(
  userId: string,
  active: boolean,
): Promise<OperatorUserRecord> {
  return postJson<{ active: boolean }, OperatorUserRecord>(
    `/users/operators/${userId}/status`,
    { active },
    requiredToken(),
  );
}

export function resetOperatorUserPassword(
  userId: string,
  newPassword: string,
): Promise<OperatorUserRecord> {
  return postJson<{ new_password: string }, OperatorUserRecord>(
    `/users/operators/${userId}/password`,
    { new_password: newPassword },
    requiredToken(),
  );
}

export function deleteOperatorUser(userId: string): Promise<void> {
  return deleteEmpty(`/users/operators/${encodeURIComponent(userId)}`, requiredToken());
}

export function listPortalUsers(token?: string): Promise<PortalUserRecord[]> {
  return getJson<PortalUserRecord[]>('/users/portal', token);
}

export function listCoupons(token?: string): Promise<CouponRecord[]> {
  return getJson<CouponRecord[]>('/coupons', token);
}

export function saveCoupon(input: CouponRecord): Promise<CouponRecord> {
  return postJson<CouponRecord, CouponRecord>('/coupons', input, requiredToken());
}

export function deleteCoupon(couponId: string): Promise<void> {
  return deleteEmpty(`/coupons/${couponId}`, requiredToken());
}

export function savePortalUser(input: {
  id?: string;
  email: string;
  display_name: string;
  password?: string;
  workspace_tenant_id: string;
  workspace_project_id: string;
  active: boolean;
}): Promise<PortalUserRecord> {
  return postJson<typeof input, PortalUserRecord>('/users/portal', input, requiredToken());
}

export function updatePortalUserStatus(
  userId: string,
  active: boolean,
): Promise<PortalUserRecord> {
  return postJson<{ active: boolean }, PortalUserRecord>(
    `/users/portal/${userId}/status`,
    { active },
    requiredToken(),
  );
}

export function resetPortalUserPassword(
  userId: string,
  newPassword: string,
): Promise<PortalUserRecord> {
  return postJson<{ new_password: string }, PortalUserRecord>(
    `/users/portal/${userId}/password`,
    { new_password: newPassword },
    requiredToken(),
  );
}

export function deletePortalUser(userId: string): Promise<void> {
  return deleteEmpty(`/users/portal/${encodeURIComponent(userId)}`, requiredToken());
}

export function listTenants(token?: string): Promise<TenantRecord[]> {
  return getJson<TenantRecord[]>('/tenants', token);
}

export function saveTenant(input: {
  id: string;
  name: string;
}): Promise<TenantRecord> {
  return postJson<typeof input, TenantRecord>('/tenants', input, requiredToken());
}

export function deleteTenant(tenantId: string): Promise<void> {
  return deleteEmpty(`/tenants/${encodeURIComponent(tenantId)}`, requiredToken());
}

export function listProjects(token?: string): Promise<ProjectRecord[]> {
  return getJson<ProjectRecord[]>('/projects', token);
}

export function saveProject(input: {
  tenant_id: string;
  id: string;
  name: string;
}): Promise<ProjectRecord> {
  return postJson<typeof input, ProjectRecord>('/projects', input, requiredToken());
}

export function deleteProject(projectId: string): Promise<void> {
  return deleteEmpty(`/projects/${encodeURIComponent(projectId)}`, requiredToken());
}

export function listApiKeys(token?: string): Promise<GatewayApiKeyRecord[]> {
  return getJson<GatewayApiKeyRecord[]>('/api-keys', token);
}

export function createApiKey(input: {
  tenant_id: string;
  project_id: string;
  environment: string;
  label?: string;
  expires_at_ms?: number | null;
}): Promise<CreatedGatewayApiKey> {
  return postJson<typeof input, CreatedGatewayApiKey>('/api-keys', input, requiredToken());
}

export function updateApiKeyStatus(
  hashedKey: string,
  active: boolean,
): Promise<GatewayApiKeyRecord> {
  return postJson<{ active: boolean }, GatewayApiKeyRecord>(
    `/api-keys/${encodeURIComponent(hashedKey)}/status`,
    { active },
    requiredToken(),
  );
}

export function deleteApiKey(hashedKey: string): Promise<void> {
  return deleteEmpty(`/api-keys/${encodeURIComponent(hashedKey)}`, requiredToken());
}

export function listChannels(token?: string): Promise<ChannelRecord[]> {
  return getJson<ChannelRecord[]>('/channels', token);
}

export function saveChannel(input: {
  id: string;
  name: string;
}): Promise<ChannelRecord> {
  return postJson<typeof input, ChannelRecord>('/channels', input, requiredToken());
}

export function deleteChannel(channelId: string): Promise<void> {
  return deleteEmpty(`/channels/${encodeURIComponent(channelId)}`, requiredToken());
}

export function listProviders(token?: string): Promise<ProxyProviderRecord[]> {
  return getJson<ProxyProviderRecord[]>('/providers', token);
}

export function saveProvider(input: {
  id: string;
  channel_id: string;
  extension_id?: string;
  adapter_kind: string;
  base_url: string;
  display_name: string;
  channel_bindings: Array<{ channel_id: string; is_primary: boolean }>;
}): Promise<ProxyProviderRecord> {
  return postJson<typeof input, ProxyProviderRecord>('/providers', input, requiredToken());
}

export function deleteProvider(providerId: string): Promise<void> {
  return deleteEmpty(`/providers/${encodeURIComponent(providerId)}`, requiredToken());
}

export function listCredentials(token?: string): Promise<CredentialRecord[]> {
  return getJson<CredentialRecord[]>('/credentials', token);
}

export function saveCredential(input: {
  tenant_id: string;
  provider_id: string;
  key_reference: string;
  secret_value: string;
}): Promise<CredentialRecord> {
  return postJson<typeof input, CredentialRecord>('/credentials', input, requiredToken());
}

export function deleteCredential(
  tenantId: string,
  providerId: string,
  keyReference: string,
): Promise<void> {
  return deleteEmpty(
    `/credentials/${encodeURIComponent(tenantId)}/providers/${encodeURIComponent(providerId)}/keys/${encodeURIComponent(keyReference)}`,
    requiredToken(),
  );
}

export function listModels(token?: string): Promise<ModelCatalogRecord[]> {
  return getJson<ModelCatalogRecord[]>('/models', token);
}

export function saveModel(input: {
  external_name: string;
  provider_id: string;
  capabilities: string[];
  streaming: boolean;
  context_window?: number;
}): Promise<ModelCatalogRecord> {
  return postJson<typeof input, ModelCatalogRecord>('/models', input, requiredToken());
}

export function deleteModel(externalName: string, providerId: string): Promise<void> {
  return deleteEmpty(
    `/models/${encodeURIComponent(externalName)}/providers/${encodeURIComponent(providerId)}`,
    requiredToken(),
  );
}

export function listUsageRecords(token?: string): Promise<UsageRecord[]> {
  return getJson<UsageRecord[]>('/usage/records', token);
}

export function getUsageSummary(token?: string): Promise<UsageSummary> {
  return getJson<UsageSummary>('/usage/summary', token);
}

export function getBillingSummary(token?: string): Promise<BillingSummary> {
  return getJson<BillingSummary>('/billing/summary', token);
}

export function listRoutingDecisionLogs(token?: string): Promise<RoutingDecisionLogRecord[]> {
  return getJson<RoutingDecisionLogRecord[]>('/routing/decision-logs', token);
}

export function listProviderHealthSnapshots(
  token?: string,
): Promise<ProviderHealthSnapshot[]> {
  return getJson<ProviderHealthSnapshot[]>('/routing/health-snapshots', token);
}

export function listRuntimeStatuses(token?: string): Promise<RuntimeStatusRecord[]> {
  return getJson<RuntimeStatusRecord[]>('/extensions/runtime-statuses', token);
}

export function reloadExtensionRuntimes(input?: {
  extension_id?: string;
  instance_id?: string;
}): Promise<RuntimeReloadReport> {
  return postJson<typeof input, RuntimeReloadReport>(
    '/extensions/runtime-reloads',
    input ?? {},
    requiredToken(),
  );
}
