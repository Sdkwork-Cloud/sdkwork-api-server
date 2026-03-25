export type GatewayRouteMode = 'sdkwork-remote' | 'custom';

export type GatewayModelMappingStatus = 'active' | 'disabled';

export interface GatewayModelMappingRule {
  id: string;
  source_channel_id: string;
  source_channel_name: string;
  source_model_id: string;
  source_model_name: string;
  target_channel_id: string;
  target_channel_name: string;
  target_model_id: string;
  target_model_name: string;
}

export interface GatewayModelMappingRecord {
  id: string;
  name: string;
  description: string;
  status: GatewayModelMappingStatus;
  effective_from: string;
  effective_to?: string | null;
  created_at: string;
  rules: GatewayModelMappingRule[];
}

export interface GatewayApiKeyOverlayRecord {
  source: 'system-generated' | 'custom';
  route_mode: GatewayRouteMode;
  route_provider_id?: string | null;
  model_mapping_id?: string | null;
  updated_at_ms: number;
}

export interface GatewayProviderSecretCacheRecord {
  tenant_id: string;
  provider_id: string;
  key_reference: string;
  secret_value: string;
  updated_at_ms: number;
}

export interface GatewayApiKeyPlaintextRevealRecord {
  plaintext_key: string;
  source: 'system-generated' | 'custom';
  updated_at_ms: number;
}

const API_ROUTER_MODEL_MAPPINGS_STORAGE_KEY =
  'sdkwork-router-admin.api-router.model-mappings';
const API_ROUTER_KEY_OVERLAYS_STORAGE_KEY =
  'sdkwork-router-admin.api-router.key-overlays';
const API_ROUTER_PROVIDER_SECRET_CACHE_STORAGE_KEY =
  'sdkwork-router-admin.api-router.provider-secrets';
const API_ROUTER_PLAINTEXT_REVEAL_STORAGE_KEY =
  'sdkwork-router-admin.api-router.plaintext-reveals';

let fallbackMappings: GatewayModelMappingRecord[] = [];
let fallbackKeyOverlays: Record<string, GatewayApiKeyOverlayRecord> = {};
let fallbackProviderSecrets: GatewayProviderSecretCacheRecord[] = [];
let fallbackPlaintextReveals: Record<string, GatewayApiKeyPlaintextRevealRecord> = {};

function storage(): Storage | null {
  if (typeof globalThis.localStorage !== 'undefined') {
    return globalThis.localStorage;
  }

  if (typeof window !== 'undefined' && window.localStorage) {
    return window.localStorage;
  }

  return null;
}

function clone<T>(value: T): T {
  return JSON.parse(JSON.stringify(value)) as T;
}

function readJson<T>(key: string, fallback: T): T {
  const currentStorage = storage();
  if (!currentStorage) {
    return clone(fallback);
  }

  const raw = currentStorage.getItem(key);
  if (!raw) {
    return clone(fallback);
  }

  try {
    return JSON.parse(raw) as T;
  } catch {
    return clone(fallback);
  }
}

function writeJson(key: string, value: unknown): void {
  const currentStorage = storage();
  if (!currentStorage) {
    return;
  }

  currentStorage.setItem(key, JSON.stringify(value));
}

function createUniqueSuffix(): string {
  return `${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 8)}`;
}

function createMappingId(): string {
  return `api-router-mapping-${createUniqueSuffix()}`;
}

function createRuleId(): string {
  return `api-router-rule-${createUniqueSuffix()}`;
}

function normalizeText(value?: string | null): string {
  return value?.trim() ?? '';
}

function normalizeOptionalText(value?: string | null): string | null {
  const normalized = value?.trim();
  return normalized ? normalized : null;
}

function normalizeRule(
  rule: Partial<GatewayModelMappingRule> & {
    source_channel_id: string;
    source_channel_name: string;
    source_model_id: string;
    source_model_name: string;
    target_channel_id: string;
    target_channel_name: string;
    target_model_id: string;
    target_model_name: string;
  },
): GatewayModelMappingRule {
  return {
    id: normalizeText(rule.id) || createRuleId(),
    source_channel_id: normalizeText(rule.source_channel_id),
    source_channel_name: normalizeText(rule.source_channel_name),
    source_model_id: normalizeText(rule.source_model_id),
    source_model_name: normalizeText(rule.source_model_name),
    target_channel_id: normalizeText(rule.target_channel_id),
    target_channel_name: normalizeText(rule.target_channel_name),
    target_model_id: normalizeText(rule.target_model_id),
    target_model_name: normalizeText(rule.target_model_name),
  };
}

function readMappings(): GatewayModelMappingRecord[] {
  fallbackMappings = clone(
    readJson<GatewayModelMappingRecord[]>(
      API_ROUTER_MODEL_MAPPINGS_STORAGE_KEY,
      fallbackMappings,
    ),
  );
  return clone(fallbackMappings);
}

function writeMappings(items: GatewayModelMappingRecord[]): void {
  fallbackMappings = clone(items);
  writeJson(API_ROUTER_MODEL_MAPPINGS_STORAGE_KEY, fallbackMappings);
}

function readKeyOverlays(): Record<string, GatewayApiKeyOverlayRecord> {
  fallbackKeyOverlays = clone(
    readJson<Record<string, GatewayApiKeyOverlayRecord>>(
      API_ROUTER_KEY_OVERLAYS_STORAGE_KEY,
      fallbackKeyOverlays,
    ),
  );
  return clone(fallbackKeyOverlays);
}

function writeKeyOverlays(items: Record<string, GatewayApiKeyOverlayRecord>): void {
  fallbackKeyOverlays = clone(items);
  writeJson(API_ROUTER_KEY_OVERLAYS_STORAGE_KEY, fallbackKeyOverlays);
}

function readProviderSecrets(): GatewayProviderSecretCacheRecord[] {
  fallbackProviderSecrets = clone(
    readJson<GatewayProviderSecretCacheRecord[]>(
      API_ROUTER_PROVIDER_SECRET_CACHE_STORAGE_KEY,
      fallbackProviderSecrets,
    ),
  );
  return clone(fallbackProviderSecrets);
}

function writeProviderSecrets(items: GatewayProviderSecretCacheRecord[]): void {
  fallbackProviderSecrets = clone(items);
  writeJson(API_ROUTER_PROVIDER_SECRET_CACHE_STORAGE_KEY, fallbackProviderSecrets);
}

function readPlaintextReveals(): Record<string, GatewayApiKeyPlaintextRevealRecord> {
  fallbackPlaintextReveals = clone(
    readJson<Record<string, GatewayApiKeyPlaintextRevealRecord>>(
      API_ROUTER_PLAINTEXT_REVEAL_STORAGE_KEY,
      fallbackPlaintextReveals,
    ),
  );
  return clone(fallbackPlaintextReveals);
}

function writePlaintextReveals(
  items: Record<string, GatewayApiKeyPlaintextRevealRecord>,
): void {
  fallbackPlaintextReveals = clone(items);
  writeJson(API_ROUTER_PLAINTEXT_REVEAL_STORAGE_KEY, fallbackPlaintextReveals);
}

export function listGatewayModelMappings(): GatewayModelMappingRecord[] {
  return readMappings().sort((left, right) => {
    const createdDelta = right.created_at.localeCompare(left.created_at);
    if (createdDelta !== 0) {
      return createdDelta;
    }

    return left.name.localeCompare(right.name);
  });
}

export function createGatewayModelMapping(input: {
  name: string;
  description?: string;
  effective_from: string;
  effective_to?: string | null;
  rules: Array<
    Partial<GatewayModelMappingRule> & {
      source_channel_id: string;
      source_channel_name: string;
      source_model_id: string;
      source_model_name: string;
      target_channel_id: string;
      target_channel_name: string;
      target_model_id: string;
      target_model_name: string;
    }
  >;
}): GatewayModelMappingRecord {
  const created: GatewayModelMappingRecord = {
    id: createMappingId(),
    name: normalizeText(input.name),
    description: normalizeText(input.description),
    status: 'active',
    effective_from: normalizeText(input.effective_from),
    effective_to: normalizeOptionalText(input.effective_to),
    created_at: new Date().toISOString(),
    rules: input.rules.map(normalizeRule),
  };

  writeMappings([created, ...readMappings()]);
  return created;
}

export function updateGatewayModelMapping(
  mappingId: string,
  update: Partial<{
    name: string;
    description: string;
    status: GatewayModelMappingStatus;
    effective_from: string;
    effective_to?: string | null;
    rules: Array<
      Partial<GatewayModelMappingRule> & {
        source_channel_id: string;
        source_channel_name: string;
        source_model_id: string;
        source_model_name: string;
        target_channel_id: string;
        target_channel_name: string;
        target_model_id: string;
        target_model_name: string;
      }
    >;
  }>,
): GatewayModelMappingRecord {
  const items = readMappings();
  const index = items.findIndex((item) => item.id === mappingId);
  if (index < 0) {
    throw new Error('Gateway model mapping not found.');
  }

  const current = items[index];
  const next: GatewayModelMappingRecord = {
    ...current,
    name: update.name !== undefined ? normalizeText(update.name) : current.name,
    description:
      update.description !== undefined
        ? normalizeText(update.description)
        : current.description,
    status: update.status ?? current.status,
    effective_from:
      update.effective_from !== undefined
        ? normalizeText(update.effective_from)
        : current.effective_from,
    effective_to:
      update.effective_to !== undefined
        ? normalizeOptionalText(update.effective_to)
        : current.effective_to,
    rules: update.rules ? update.rules.map(normalizeRule) : current.rules,
  };

  items[index] = next;
  writeMappings(items);
  return next;
}

export function updateGatewayModelMappingStatus(
  mappingId: string,
  status: GatewayModelMappingStatus,
): GatewayModelMappingRecord {
  return updateGatewayModelMapping(mappingId, { status });
}

export function deleteGatewayModelMapping(mappingId: string): boolean {
  const items = readMappings();
  const next = items.filter((item) => item.id !== mappingId);
  if (next.length === items.length) {
    return false;
  }

  writeMappings(next);

  const overlays = readKeyOverlays();
  let changed = false;
  for (const [hashedKey, overlay] of Object.entries(overlays)) {
    if (overlay.model_mapping_id === mappingId) {
      overlays[hashedKey] = {
        ...overlay,
        model_mapping_id: null,
        updated_at_ms: Date.now(),
      };
      changed = true;
    }
  }

  if (changed) {
    writeKeyOverlays(overlays);
  }

  return true;
}

export function readGatewayApiKeyOverlay(hashedKey: string): GatewayApiKeyOverlayRecord {
  const overlay = readKeyOverlays()[hashedKey];
  return {
    source: overlay?.source ?? 'system-generated',
    route_mode: overlay?.route_mode ?? 'sdkwork-remote',
    route_provider_id: overlay?.route_provider_id ?? null,
    model_mapping_id: overlay?.model_mapping_id ?? null,
    updated_at_ms: overlay?.updated_at_ms ?? 0,
  };
}

export function saveGatewayApiKeyOverlay(
  hashedKey: string,
  update: Partial<GatewayApiKeyOverlayRecord>,
): GatewayApiKeyOverlayRecord {
  const overlays = readKeyOverlays();
  const current = readGatewayApiKeyOverlay(hashedKey);
  const routeMode = update.route_mode ?? current.route_mode;
  const next: GatewayApiKeyOverlayRecord = {
    source: update.source ?? current.source,
    route_mode: routeMode,
    route_provider_id:
      routeMode === 'custom'
        ? normalizeOptionalText(update.route_provider_id ?? current.route_provider_id)
        : null,
    model_mapping_id: normalizeOptionalText(
      update.model_mapping_id ?? current.model_mapping_id,
    ),
    updated_at_ms: Date.now(),
  };

  overlays[hashedKey] = next;
  writeKeyOverlays(overlays);
  return next;
}

export function clearGatewayApiKeyOverlay(hashedKey: string): void {
  const overlays = readKeyOverlays();
  if (!overlays[hashedKey]) {
    return;
  }

  delete overlays[hashedKey];
  writeKeyOverlays(overlays);
}

export function readGatewayApiKeyPlaintextReveal(
  hashedKey: string,
): GatewayApiKeyPlaintextRevealRecord | null {
  const reveal = readPlaintextReveals()[hashedKey];
  if (!reveal?.plaintext_key?.trim()) {
    return null;
  }

  return {
    plaintext_key: reveal.plaintext_key,
    source: reveal.source === 'custom' ? 'custom' : 'system-generated',
    updated_at_ms: reveal.updated_at_ms ?? 0,
  };
}

export function rememberGatewayApiKeyPlaintextReveal(input: {
  hashed_key: string;
  plaintext_key: string;
  source?: 'system-generated' | 'custom' | null;
}): GatewayApiKeyPlaintextRevealRecord {
  const reveals = readPlaintextReveals();
  const next: GatewayApiKeyPlaintextRevealRecord = {
    plaintext_key: input.plaintext_key,
    source: input.source === 'custom' ? 'custom' : 'system-generated',
    updated_at_ms: Date.now(),
  };

  reveals[input.hashed_key] = next;
  writePlaintextReveals(reveals);
  return next;
}

export function clearGatewayApiKeyPlaintextReveal(hashedKey: string): void {
  const reveals = readPlaintextReveals();
  if (!reveals[hashedKey]) {
    return;
  }

  delete reveals[hashedKey];
  writePlaintextReveals(reveals);
}

export function readGatewayProviderSecretCache(
  tenantId: string,
  providerId: string,
  keyReference: string,
): GatewayProviderSecretCacheRecord | null {
  return (
    readProviderSecrets().find(
      (record) =>
        record.tenant_id === tenantId &&
        record.provider_id === providerId &&
        record.key_reference === keyReference,
    ) ?? null
  );
}

export function saveGatewayProviderSecretCache(input: {
  tenant_id: string;
  provider_id: string;
  key_reference: string;
  secret_value: string;
}): GatewayProviderSecretCacheRecord {
  const items = readProviderSecrets();
  const nextRecord: GatewayProviderSecretCacheRecord = {
    tenant_id: normalizeText(input.tenant_id),
    provider_id: normalizeText(input.provider_id),
    key_reference: normalizeText(input.key_reference),
    secret_value: input.secret_value,
    updated_at_ms: Date.now(),
  };

  const nextItems = items.filter(
    (record) =>
      !(
        record.tenant_id === nextRecord.tenant_id &&
        record.provider_id === nextRecord.provider_id &&
        record.key_reference === nextRecord.key_reference
      ),
  );
  nextItems.unshift(nextRecord);
  writeProviderSecrets(nextItems);
  return nextRecord;
}
