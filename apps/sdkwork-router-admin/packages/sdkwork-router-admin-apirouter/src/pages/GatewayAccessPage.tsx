import { useDeferredValue, useEffect, useMemo, useState } from 'react';
import type { FormEvent } from 'react';

import {
  AdminDialog,
  Checkbox,
  ConfirmDialog,
  DataTable,
  Dialog,
  DialogContent,
  DialogFooter,
  FormField,
  Input,
  InlineButton,
  PageToolbar,
  Pill,
  Select,
  Textarea,
  ToolbarInline,
  ToolbarSearchField,
} from 'sdkwork-router-admin-commons';
import type {
  AdminPageProps,
  CreatedGatewayApiKey,
  GatewayApiKeyRecord,
} from 'sdkwork-router-admin-types';

import {
  applyApiKeyQuickSetup,
  buildApiKeyCurlSnippet,
  buildApiKeyQuickSetupPlans,
  listApiKeyInstances,
  resolveGatewayBaseUrl,
  type ApiKeyQuickSetupPlan,
  type ApiKeySetupClientId,
  type ApiKeySetupInstance,
} from '../services/gatewayApiKeyAccessService';
import {
  clearGatewayApiKeyOverlay,
  clearGatewayApiKeyPlaintextReveal,
  listGatewayModelMappings,
  readGatewayApiKeyOverlay,
  readGatewayApiKeyPlaintextReveal,
  rememberGatewayApiKeyPlaintextReveal,
  saveGatewayApiKeyOverlay,
  type GatewayRouteMode,
} from '../services/gatewayOverlayStore';

type GatewayAccessPageProps = AdminPageProps & {
  onRefreshWorkspace: () => Promise<void>;
  onCreateApiKey: (input: {
    tenant_id: string;
    project_id: string;
    environment: string;
    label?: string;
    notes?: string;
    expires_at_ms?: number | null;
    plaintext_key?: string;
  }) => Promise<CreatedGatewayApiKey>;
  onUpdateApiKey: (input: {
    hashed_key: string;
    tenant_id: string;
    project_id: string;
    environment: string;
    label: string;
    notes?: string | null;
    expires_at_ms?: number | null;
  }) => Promise<void>;
  onUpdateApiKeyStatus: (hashedKey: string, active: boolean) => Promise<void>;
  onDeleteApiKey: (hashedKey: string) => Promise<void>;
};

type CreateDraft = {
  tenant_id: string;
  project_id: string;
  environment: string;
  label: string;
  notes: string;
  expires_at: string;
  plaintext_key: string;
  route_mode: GatewayRouteMode;
  route_provider_id: string;
  model_mapping_id: string;
};

type EditDraft = {
  label: string;
  notes: string;
  expires_at: string;
};

type RouteDraft = {
  source: 'system-generated' | 'custom';
  route_mode: GatewayRouteMode;
  route_provider_id: string;
  model_mapping_id: string;
};

const QUICK_SETUP_CLIENT_ORDER: ApiKeySetupClientId[] = [
  'codex',
  'claude-code',
  'opencode',
  'gemini',
  'openclaw',
];

const QUICK_SETUP_CLIENT_LABELS: Record<ApiKeySetupClientId, string> = {
  codex: 'Codex',
  'claude-code': 'Claude Code',
  opencode: 'OpenCode',
  gemini: 'Gemini',
  openclaw: 'OpenClaw',
};

function formatTimestamp(value?: number | null): string {
  if (!value) {
    return '-';
  }

  return new Date(value).toLocaleString();
}

function formatExpiryInput(value?: number | null): string {
  if (!value) {
    return '';
  }

  const date = new Date(value);
  const pad = (item: number) => String(item).padStart(2, '0');
  return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}T${pad(date.getHours())}:${pad(date.getMinutes())}`;
}

function parseExpiryInput(value: string): number | null | undefined {
  const normalized = value.trim();
  if (!normalized) {
    return undefined;
  }

  const parsed = Date.parse(normalized);
  return Number.isFinite(parsed) ? parsed : null;
}

function maskKey(value: string): string {
  if (value.length <= 16) {
    return value;
  }

  return `${value.slice(0, 10)}••••••${value.slice(-4)}`;
}

function createDraft(
  tenantId: string,
  projectId: string,
  overrides: Partial<CreateDraft> = {},
): CreateDraft {
  return {
    tenant_id: tenantId,
    project_id: projectId,
    environment: 'live',
    label: '',
    notes: '',
    expires_at: '',
    plaintext_key: '',
    route_mode: 'sdkwork-remote',
    route_provider_id: '',
    model_mapping_id: '',
    ...overrides,
  };
}

function editDraftFromKey(key: GatewayApiKeyRecord): EditDraft {
  return {
    label: key.label,
    notes: key.notes ?? '',
    expires_at: formatExpiryInput(key.expires_at_ms),
  };
}

function routeDraftFromKey(key: GatewayApiKeyRecord): RouteDraft {
  const overlay = readGatewayApiKeyOverlay(key.hashed_key);
  return {
    source: overlay.source,
    route_mode: overlay.route_mode,
    route_provider_id: overlay.route_provider_id ?? '',
    model_mapping_id: overlay.model_mapping_id ?? '',
  };
}

async function copyToClipboard(value: string): Promise<void> {
  if (navigator.clipboard) {
    await navigator.clipboard.writeText(value);
  }
}

function resolvePlaintextForKey(key: GatewayApiKeyRecord): string | null {
  if (key.raw_key?.trim()) {
    return key.raw_key;
  }

  return readGatewayApiKeyPlaintextReveal(key.hashed_key)?.plaintext_key ?? null;
}

function buildUsageKeyFromCreateResponse(created: CreatedGatewayApiKey): GatewayApiKeyRecord {
  return {
    tenant_id: created.tenant_id,
    project_id: created.project_id,
    environment: created.environment,
    hashed_key: created.hashed,
    label: created.label,
    notes: created.notes,
    created_at_ms: created.created_at_ms,
    expires_at_ms: created.expires_at_ms,
    last_used_at_ms: null,
    active: true,
    raw_key: null,
  };
}

function buildQuickSetupSummary(result: Awaited<ReturnType<typeof applyApiKeyQuickSetup>>): string {
  if (result.updatedInstanceIds.length) {
    return `Applied setup to ${result.updatedInstanceIds.length} OpenClaw instance(s).`;
  }

  if (result.updatedEnvironments.length) {
    return `Applied setup and updated ${result.updatedEnvironments.length} environment target(s).`;
  }

  return `Applied setup and wrote ${result.writtenFiles.length} file(s).`;
}

export function GatewayAccessPage({
  snapshot,
  onRefreshWorkspace,
  onCreateApiKey,
  onUpdateApiKey,
  onUpdateApiKeyStatus,
  onDeleteApiKey,
}: GatewayAccessPageProps) {
  const defaultTenantId = snapshot.tenants[0]?.id ?? 'tenant_local_demo';
  const defaultProjectId = snapshot.projects[0]?.id ?? 'project_local_demo';
  const [search, setSearch] = useState('');
  const [isCreateOpen, setIsCreateOpen] = useState(false);
  const [createDraftState, setCreateDraftState] = useState<CreateDraft>(() =>
    createDraft(defaultTenantId, defaultProjectId),
  );
  const [editingKey, setEditingKey] = useState<GatewayApiKeyRecord | null>(null);
  const [editDraft, setEditDraft] = useState<EditDraft>({
    label: '',
    notes: '',
    expires_at: '',
  });
  const [routeKey, setRouteKey] = useState<GatewayApiKeyRecord | null>(null);
  const [routeDraft, setRouteDraft] = useState<RouteDraft>({
    source: 'system-generated',
    route_mode: 'sdkwork-remote',
    route_provider_id: '',
    model_mapping_id: '',
  });
  const [usageKey, setUsageKey] = useState<GatewayApiKeyRecord | null>(null);
  const [pendingDelete, setPendingDelete] = useState<GatewayApiKeyRecord | null>(null);
  const [overlayVersion, setOverlayVersion] = useState(0);
  const [gatewayBaseUrl, setGatewayBaseUrl] = useState('http://127.0.0.1:8080');
  const [openClawInstances, setOpenClawInstances] = useState<ApiKeySetupInstance[]>([]);
  const [loadingInstances, setLoadingInstances] = useState(false);
  const [selectedClientId, setSelectedClientId] = useState<ApiKeySetupClientId>('codex');
  const [selectedInstanceIds, setSelectedInstanceIds] = useState<string[]>([]);
  const [applyingClientId, setApplyingClientId] = useState<ApiKeySetupClientId | null>(null);
  const [usageStatus, setUsageStatus] = useState('');
  const deferredSearch = useDeferredValue(search.trim().toLowerCase());

  useEffect(() => {
    for (const key of snapshot.apiKeys) {
      if (key.raw_key?.trim()) {
        rememberGatewayApiKeyPlaintextReveal({
          hashed_key: key.hashed_key,
          plaintext_key: key.raw_key,
          source: readGatewayApiKeyOverlay(key.hashed_key).source,
        });
      }
    }
  }, [snapshot.apiKeys]);

  useEffect(() => {
    let cancelled = false;

    void resolveGatewayBaseUrl().then((baseUrl) => {
      if (!cancelled) {
        setGatewayBaseUrl(baseUrl);
      }
    });

    return () => {
      cancelled = true;
    };
  }, []);

  useEffect(() => {
    if (!usageKey) {
      setUsageStatus('');
      setSelectedClientId('codex');
      setSelectedInstanceIds([]);
      return;
    }

    let cancelled = false;
    setLoadingInstances(true);

    void Promise.all([resolveGatewayBaseUrl(), listApiKeyInstances()])
      .then(([baseUrl, instances]) => {
        if (cancelled) {
          return;
        }

        setGatewayBaseUrl(baseUrl);
        setOpenClawInstances(instances);
        setSelectedInstanceIds(instances.slice(0, 1).map((item) => item.id));
      })
      .finally(() => {
        if (!cancelled) {
          setLoadingInstances(false);
        }
      });

    return () => {
      cancelled = true;
    };
  }, [usageKey]);

  const modelMappings = useMemo(() => listGatewayModelMappings(), [overlayVersion]);
  const mappingById = useMemo(
    () => new Map(modelMappings.map((mapping) => [mapping.id, mapping])),
    [modelMappings],
  );
  const providerById = useMemo(
    () => new Map(snapshot.providers.map((provider) => [provider.id, provider])),
    [snapshot.providers],
  );

  const filteredKeys = useMemo(
    () =>
      [...snapshot.apiKeys]
        .sort((left, right) => right.created_at_ms - left.created_at_ms)
        .filter((key) => {
          if (!deferredSearch) {
            return true;
          }

          const overlay = readGatewayApiKeyOverlay(key.hashed_key);
          const reveal = readGatewayApiKeyPlaintextReveal(key.hashed_key);
          const haystack = [
            key.label,
            key.tenant_id,
            key.project_id,
            key.environment,
            key.notes ?? '',
            key.hashed_key,
            overlay.route_provider_id ?? '',
            overlay.model_mapping_id ?? '',
            reveal?.plaintext_key ?? '',
          ]
            .join(' ')
            .toLowerCase();

          return haystack.includes(deferredSearch);
        }),
    [deferredSearch, snapshot.apiKeys],
  );

  const availableCreateProjects = snapshot.projects.filter(
    (project) => project.tenant_id === createDraftState.tenant_id,
  );

  const usagePlaintext = usageKey ? resolvePlaintextForKey(usageKey) : null;
  const quickSetupPlans = useMemo(
    () =>
      usageKey
        ? buildApiKeyQuickSetupPlans({
            hashedKey: usageKey.hashed_key,
            label: usageKey.label,
            plaintextKey: usagePlaintext || '<api-key-not-visible-on-this-device>',
            gatewayBaseUrl,
            defaults: {
              openaiModel: snapshot.models[0]?.external_name,
              anthropicModel:
                snapshot.models.find((model) => model.external_name.includes('claude'))?.external_name ??
                snapshot.models[0]?.external_name,
              geminiModel:
                snapshot.models.find((model) => model.external_name.includes('gemini'))?.external_name ??
                'gemini-2.5-pro',
            },
          })
        : [],
    [gatewayBaseUrl, snapshot.models, usageKey, usagePlaintext],
  );
  const selectedPlan =
    quickSetupPlans.find((plan) => plan.id === selectedClientId) ?? quickSetupPlans[0] ?? null;
  const usageOverlay = usageKey ? readGatewayApiKeyOverlay(usageKey.hashed_key) : null;

  function refreshOverlay(): void {
    setOverlayVersion((value) => value + 1);
  }

  async function handleRefreshWorkspace(): Promise<void> {
    await onRefreshWorkspace();
    refreshOverlay();
  }

  function resetCreateDialog(): void {
    setCreateDraftState(createDraft(defaultTenantId, defaultProjectId));
    setIsCreateOpen(false);
  }

  function openEditDialog(key: GatewayApiKeyRecord): void {
    setEditingKey(key);
    setEditDraft(editDraftFromKey(key));
  }

  function openRouteDialog(key: GatewayApiKeyRecord): void {
    setRouteKey(key);
    setRouteDraft(routeDraftFromKey(key));
  }

  function openUsageDialog(key: GatewayApiKeyRecord): void {
    setUsageKey(key);
  }

  async function handleCreateSubmit(event: FormEvent<HTMLFormElement>): Promise<void> {
    event.preventDefault();
    const expiresAt = parseExpiryInput(createDraftState.expires_at);
    const plaintextKey = createDraftState.plaintext_key.trim();
    const created = await onCreateApiKey({
      tenant_id: createDraftState.tenant_id,
      project_id: createDraftState.project_id,
      environment: createDraftState.environment,
      label: createDraftState.label.trim() || undefined,
      notes: createDraftState.notes.trim() || undefined,
      expires_at_ms: expiresAt === null ? undefined : expiresAt,
      plaintext_key: plaintextKey || undefined,
    });

    rememberGatewayApiKeyPlaintextReveal({
      hashed_key: created.hashed,
      plaintext_key: created.plaintext,
      source: plaintextKey ? 'custom' : 'system-generated',
    });
    saveGatewayApiKeyOverlay(created.hashed, {
      source: plaintextKey ? 'custom' : 'system-generated',
      route_mode: createDraftState.route_mode,
      route_provider_id:
        createDraftState.route_mode === 'custom'
          ? createDraftState.route_provider_id || null
          : null,
      model_mapping_id: createDraftState.model_mapping_id || null,
    });
    refreshOverlay();
    resetCreateDialog();
    openUsageDialog(buildUsageKeyFromCreateResponse(created));
  }

  async function handleEditSubmit(event: FormEvent<HTMLFormElement>): Promise<void> {
    event.preventDefault();
    if (!editingKey) {
      return;
    }

    const expiresAt = parseExpiryInput(editDraft.expires_at);
    await onUpdateApiKey({
      hashed_key: editingKey.hashed_key,
      tenant_id: editingKey.tenant_id,
      project_id: editingKey.project_id,
      environment: editingKey.environment,
      label: editDraft.label.trim(),
      notes: editDraft.notes.trim() || null,
      expires_at_ms: expiresAt === null ? null : expiresAt ?? null,
    });
    setEditingKey(null);
  }

  async function handleRouteSubmit(event: FormEvent<HTMLFormElement>): Promise<void> {
    event.preventDefault();
    if (!routeKey) {
      return;
    }

    saveGatewayApiKeyOverlay(routeKey.hashed_key, {
      source: routeDraft.source,
      route_mode: routeDraft.route_mode,
      route_provider_id:
        routeDraft.route_mode === 'custom' ? routeDraft.route_provider_id || null : null,
      model_mapping_id: routeDraft.model_mapping_id || null,
    });
    refreshOverlay();
    setRouteKey(null);
  }

  async function confirmDelete(): Promise<void> {
    if (!pendingDelete) {
      return;
    }

    await onDeleteApiKey(pendingDelete.hashed_key);
    clearGatewayApiKeyOverlay(pendingDelete.hashed_key);
    clearGatewayApiKeyPlaintextReveal(pendingDelete.hashed_key);
    refreshOverlay();
    setPendingDelete(null);
  }

  async function handleApplySetup(plan: ApiKeyQuickSetupPlan): Promise<void> {
    if (!usagePlaintext) {
      setUsageStatus(
        'The plaintext Api key is no longer visible on this device. Create a replacement before applying setup.',
      );
      return;
    }

    if (plan.requiresInstances && !selectedInstanceIds.length) {
      setUsageStatus('Select at least one OpenClaw instance before applying setup.');
      return;
    }

    setApplyingClientId(plan.id);
    setUsageStatus(`Applying ${plan.label} setup...`);

    try {
      const result = await applyApiKeyQuickSetup({
        ...plan.request,
        provider: {
          ...plan.request.provider,
          apiKey: usagePlaintext,
        },
        openClaw: plan.requiresInstances ? { instanceIds: selectedInstanceIds } : undefined,
      });
      setUsageStatus(buildQuickSetupSummary(result));
    } catch (error) {
      setUsageStatus(error instanceof Error ? error.message : 'Failed to apply setup.');
    } finally {
      setApplyingClientId(null);
    }
  }

  return (
    <div className="adminx-page-grid">
      <PageToolbar
        compact
        actions={(
          <>
            <InlineButton tone="primary" onClick={() => setIsCreateOpen(true)}>
              Create Api key
            </InlineButton>
            <InlineButton onClick={() => void handleRefreshWorkspace()}>
              Refresh workspace
            </InlineButton>
          </>
        )}
      >
        <ToolbarInline>
          <ToolbarSearchField
            label="Search Api keys"
            value={search}
            onChange={(event) => setSearch(event.target.value)}
            placeholder="label, project, hashed key..."
          />
        </ToolbarInline>
      </PageToolbar>

      <DataTable
        columns={[
          {
            key: 'identity',
            label: 'Api key',
            render: (key) => {
              const plaintext = resolvePlaintextForKey(key);
              return (
                <div className="adminx-table-cell-stack">
                  <strong>{key.label || key.project_id}</strong>
                  <span>{plaintext ? maskKey(plaintext) : key.hashed_key}</span>
                </div>
              );
            },
          },
          {
            key: 'workspace',
            label: 'Workspace',
            render: (key) => (
              <div className="adminx-table-cell-stack">
                <strong>
                  {key.tenant_id} / {key.project_id}
                </strong>
                <span>{key.environment}</span>
              </div>
            ),
          },
          {
            key: 'route',
            label: 'Route config',
            render: (key) => {
              const overlay = readGatewayApiKeyOverlay(key.hashed_key);
              const provider = overlay.route_provider_id
                ? providerById.get(overlay.route_provider_id)
                : null;
              const mapping = overlay.model_mapping_id
                ? mappingById.get(overlay.model_mapping_id)
                : null;

              return (
                <div className="adminx-table-cell-stack">
                  <strong>
                    {overlay.route_mode === 'custom'
                      ? provider?.display_name ?? provider?.id ?? 'Custom provider'
                      : 'SDKWork gateway default'}
                  </strong>
                  <span>{mapping?.name ?? 'No model mapping'}</span>
                </div>
              );
            },
          },
          {
            key: 'usage',
            label: 'Usage',
            render: (key) => (
              <div className="adminx-table-cell-stack">
                <span>Created: {formatTimestamp(key.created_at_ms)}</span>
                <span>Last used: {formatTimestamp(key.last_used_at_ms)}</span>
                <span>Expires: {formatTimestamp(key.expires_at_ms)}</span>
              </div>
            ),
          },
          {
            key: 'status',
            label: 'Status',
            render: (key) => (
              <Pill tone={key.active ? 'live' : 'danger'}>{key.active ? 'active' : 'revoked'}</Pill>
            ),
          },
          {
            key: 'actions',
            label: 'Actions',
            render: (key) => {
              const plaintext = resolvePlaintextForKey(key);

              return (
                <div className="adminx-row">
                  <InlineButton onClick={() => openUsageDialog(key)}>Usage method</InlineButton>
                  <InlineButton onClick={() => openRouteDialog(key)}>Route config</InlineButton>
                  <InlineButton onClick={() => openEditDialog(key)}>Edit</InlineButton>
                  {plaintext ? (
                    <InlineButton onClick={() => void copyToClipboard(plaintext)}>
                      Copy Api key
                    </InlineButton>
                  ) : null}
                  <InlineButton onClick={() => void onUpdateApiKeyStatus(key.hashed_key, !key.active)}>
                    {key.active ? 'Revoke' : 'Restore'}
                  </InlineButton>
                  <InlineButton tone="danger" onClick={() => setPendingDelete(key)}>
                    Delete
                  </InlineButton>
                </div>
              );
            },
          },
        ]}
        rows={filteredKeys}
        empty="No Api keys match the current workspace filter."
        getKey={(key) => key.hashed_key}
      />

      <Dialog
        open={isCreateOpen}
        onOpenChange={(nextOpen) => (nextOpen ? setIsCreateOpen(true) : resetCreateDialog())}
      >
        <DialogContent size="large">
          <AdminDialog
            title="Create Api key"
            detail="Issue a new Api key, keep the table surface compact, and capture key-level route posture in the same workflow."
          >
            <form className="adminx-form-grid" onSubmit={(event) => void handleCreateSubmit(event)}>
              <FormField label="Tenant">
                {snapshot.tenants.length ? (
                  <Select
                    value={createDraftState.tenant_id}
                    onChange={(event) =>
                      setCreateDraftState((current) => ({
                        ...current,
                        tenant_id: event.target.value,
                        project_id:
                          snapshot.projects.find((project) => project.tenant_id === event.target.value)?.id ??
                          current.project_id,
                      }))
                    }
                  >
                    {snapshot.tenants.map((tenant) => (
                      <option key={tenant.id} value={tenant.id}>
                        {tenant.name} ({tenant.id})
                      </option>
                    ))}
                  </Select>
                ) : (
                  <Input
                    value={createDraftState.tenant_id}
                    onChange={(event) =>
                      setCreateDraftState((current) => ({ ...current, tenant_id: event.target.value }))
                    }
                    required
                  />
                )}
              </FormField>

              <FormField label="Project">
                {availableCreateProjects.length ? (
                  <Select
                    value={createDraftState.project_id}
                    onChange={(event) =>
                      setCreateDraftState((current) => ({ ...current, project_id: event.target.value }))
                    }
                  >
                    {availableCreateProjects.map((project) => (
                      <option key={project.id} value={project.id}>
                        {project.name} ({project.id})
                      </option>
                    ))}
                  </Select>
                ) : (
                  <Input
                    value={createDraftState.project_id}
                    onChange={(event) =>
                      setCreateDraftState((current) => ({ ...current, project_id: event.target.value }))
                    }
                    required
                  />
                )}
              </FormField>

              <FormField label="Environment">
                <Select
                  value={createDraftState.environment}
                  onChange={(event) =>
                    setCreateDraftState((current) => ({ ...current, environment: event.target.value }))
                  }
                >
                  <option value="live">Live</option>
                  <option value="staging">Staging</option>
                  <option value="test">Test</option>
                </Select>
              </FormField>

              <FormField label="Label">
                <Input
                  value={createDraftState.label}
                  onChange={(event) =>
                    setCreateDraftState((current) => ({ ...current, label: event.target.value }))
                  }
                  placeholder="Workspace default Api key"
                />
              </FormField>

              <FormField label="Expires at">
                <Input
                  type="datetime-local"
                  value={createDraftState.expires_at}
                  onChange={(event) =>
                    setCreateDraftState((current) => ({ ...current, expires_at: event.target.value }))
                  }
                />
              </FormField>

              <FormField label="Notes">
                <Textarea
                  rows={3}
                  value={createDraftState.notes}
                  onChange={(event) =>
                    setCreateDraftState((current) => ({ ...current, notes: event.target.value }))
                  }
                />
              </FormField>

              <FormField label="Custom Api key" hint="Leave empty to let the gateway generate the plaintext.">
                <Input
                  value={createDraftState.plaintext_key}
                  onChange={(event) =>
                    setCreateDraftState((current) => ({ ...current, plaintext_key: event.target.value }))
                  }
                  placeholder="sk-router-live-demo"
                />
              </FormField>

              <FormField label="Route mode">
                <Select
                  value={createDraftState.route_mode}
                  onChange={(event) =>
                    setCreateDraftState((current) => ({
                      ...current,
                      route_mode: event.target.value as GatewayRouteMode,
                    }))
                  }
                >
                  <option value="sdkwork-remote">SDKWork gateway default</option>
                  <option value="custom">Custom provider</option>
                </Select>
              </FormField>

              <FormField label="Pinned provider">
                <Select
                  value={createDraftState.route_provider_id}
                  onChange={(event) =>
                    setCreateDraftState((current) => ({ ...current, route_provider_id: event.target.value }))
                  }
                  disabled={createDraftState.route_mode !== 'custom'}
                >
                  <option value="">Gateway default</option>
                  {snapshot.providers.map((provider) => (
                    <option key={provider.id} value={provider.id}>
                      {provider.display_name} ({provider.id})
                    </option>
                  ))}
                </Select>
              </FormField>

              <FormField label="Model mapping">
                <Select
                  value={createDraftState.model_mapping_id}
                  onChange={(event) =>
                    setCreateDraftState((current) => ({ ...current, model_mapping_id: event.target.value }))
                  }
                >
                  <option value="">No mapping</option>
                  {modelMappings.map((mapping) => (
                    <option key={mapping.id} value={mapping.id}>
                      {mapping.name}
                    </option>
                  ))}
                </Select>
              </FormField>

              <DialogFooter>
                <InlineButton onClick={resetCreateDialog}>Cancel</InlineButton>
                <InlineButton tone="primary" type="submit">
                  Create Api key
                </InlineButton>
              </DialogFooter>
            </form>
          </AdminDialog>
        </DialogContent>
      </Dialog>

      <Dialog
        open={Boolean(editingKey)}
        onOpenChange={(nextOpen) => (nextOpen ? null : setEditingKey(null))}
      >
        <DialogContent size="medium">
          <AdminDialog title="Edit Api key" detail="Update display metadata without changing the compact table-first workbench.">
            {editingKey ? (
              <form className="adminx-form-grid" onSubmit={(event) => void handleEditSubmit(event)}>
                <FormField label="Workspace">
                  <Input
                    value={`${editingKey.tenant_id} / ${editingKey.project_id} / ${editingKey.environment}`}
                    disabled
                  />
                </FormField>
                <FormField label="Hashed key">
                  <Input value={editingKey.hashed_key} disabled />
                </FormField>
                <FormField label="Label">
                  <Input
                    value={editDraft.label}
                    onChange={(event) =>
                      setEditDraft((current) => ({ ...current, label: event.target.value }))
                    }
                    required
                  />
                </FormField>
                <FormField label="Expires at">
                  <Input
                    type="datetime-local"
                    value={editDraft.expires_at}
                    onChange={(event) =>
                      setEditDraft((current) => ({ ...current, expires_at: event.target.value }))
                    }
                  />
                </FormField>
                <FormField label="Notes">
                  <Textarea
                    rows={3}
                    value={editDraft.notes}
                    onChange={(event) =>
                      setEditDraft((current) => ({ ...current, notes: event.target.value }))
                    }
                  />
                </FormField>
                <DialogFooter>
                  <InlineButton onClick={() => setEditingKey(null)}>Cancel</InlineButton>
                  <InlineButton tone="primary" type="submit">
                    Save
                  </InlineButton>
                </DialogFooter>
              </form>
            ) : null}
          </AdminDialog>
        </DialogContent>
      </Dialog>

      <Dialog
        open={Boolean(routeKey)}
        onOpenChange={(nextOpen) => (nextOpen ? null : setRouteKey(null))}
      >
        <DialogContent size="medium">
          <AdminDialog
            title="Route config"
            detail="Keep per-key route mode, provider pinning, and model mapping aligned with claw-style local overlay behavior."
          >
            {routeKey ? (
              <form className="adminx-form-grid" onSubmit={(event) => void handleRouteSubmit(event)}>
                <FormField label="Api key">
                  <Input value={`${routeKey.label || routeKey.project_id} (${routeKey.environment})`} disabled />
                </FormField>
                <FormField label="Source">
                  <Select
                    value={routeDraft.source}
                    onChange={(event) =>
                      setRouteDraft((current) => ({
                        ...current,
                        source: event.target.value as 'system-generated' | 'custom',
                      }))
                    }
                  >
                    <option value="system-generated">System generated</option>
                    <option value="custom">Custom</option>
                  </Select>
                </FormField>
                <FormField label="Route mode">
                  <Select
                    value={routeDraft.route_mode}
                    onChange={(event) =>
                      setRouteDraft((current) => ({
                        ...current,
                        route_mode: event.target.value as GatewayRouteMode,
                      }))
                    }
                  >
                    <option value="sdkwork-remote">SDKWork gateway default</option>
                    <option value="custom">Custom provider</option>
                  </Select>
                </FormField>
                <FormField label="Pinned provider">
                  <Select
                    value={routeDraft.route_provider_id}
                    onChange={(event) =>
                      setRouteDraft((current) => ({ ...current, route_provider_id: event.target.value }))
                    }
                    disabled={routeDraft.route_mode !== 'custom'}
                  >
                    <option value="">Gateway default</option>
                    {snapshot.providers.map((provider) => (
                      <option key={provider.id} value={provider.id}>
                        {provider.display_name} ({provider.id})
                      </option>
                    ))}
                  </Select>
                </FormField>
                <FormField label="Model mapping">
                  <Select
                    value={routeDraft.model_mapping_id}
                    onChange={(event) =>
                      setRouteDraft((current) => ({ ...current, model_mapping_id: event.target.value }))
                    }
                  >
                    <option value="">No mapping</option>
                    {modelMappings.map((mapping) => (
                      <option key={mapping.id} value={mapping.id}>
                        {mapping.name}
                      </option>
                    ))}
                  </Select>
                </FormField>
                <DialogFooter>
                  <InlineButton onClick={() => setRouteKey(null)}>Cancel</InlineButton>
                  <InlineButton tone="primary" type="submit">
                    Save
                  </InlineButton>
                </DialogFooter>
              </form>
            ) : null}
          </AdminDialog>
        </DialogContent>
      </Dialog>

      <Dialog open={Boolean(usageKey)} onOpenChange={(nextOpen) => !nextOpen && setUsageKey(null)}>
        <DialogContent size="large">
          <AdminDialog
            title="Usage method"
            detail="Quick setup keeps the compact Api key workbench aligned with claw-studio while routing through the real gateway compatibility endpoints."
          >
            {usageKey ? (
              <div className="adminx-form-grid">
                <div className="adminx-note">
                  <strong>Api key</strong>
                  <p>{usageKey.label || usageKey.project_id}</p>
                </div>
                <div className="adminx-note">
                  <strong>Gateway endpoint</strong>
                  <p>{`${gatewayBaseUrl}/v1`}</p>
                </div>
                <div className="adminx-note">
                  <strong>Authorization header</strong>
                  <code>
                    {usagePlaintext
                      ? `Authorization: Bearer ${usagePlaintext}`
                      : 'Authorization: Bearer <rotate-to-reveal-a-new-api-key>'}
                  </code>
                </div>
                <div className="adminx-note">
                  <strong>Route config</strong>
                  <p>
                    {usageOverlay?.route_mode === 'custom'
                      ? providerById.get(usageOverlay.route_provider_id ?? '')?.display_name ??
                        'Custom provider'
                      : 'SDKWork gateway default'}
                    {' · '}
                    {usageOverlay?.model_mapping_id
                      ? mappingById.get(usageOverlay.model_mapping_id)?.name ?? usageOverlay.model_mapping_id
                      : 'No model mapping'}
                  </p>
                </div>

                <div className="adminx-note">
                  <strong>Quick setup</strong>
                  <div className="adminx-row">
                    {QUICK_SETUP_CLIENT_ORDER.map((clientId) => {
                      const plan = quickSetupPlans.find((item) => item.id === clientId);
                      if (!plan) {
                        return null;
                      }

                      return (
                        <InlineButton
                          key={plan.id}
                          tone={selectedClientId === plan.id ? 'primary' : 'secondary'}
                          onClick={() => setSelectedClientId(plan.id)}
                        >
                          {QUICK_SETUP_CLIENT_LABELS[plan.id] ?? plan.label}
                        </InlineButton>
                      );
                    })}
                  </div>
                </div>

                {selectedPlan ? (
                  <>
                    <div className="adminx-note">
                      <strong>{selectedPlan.label}</strong>
                      <p>{selectedPlan.description}</p>
                    </div>

                    {selectedPlan.requiresInstances ? (
                      <div className="adminx-note">
                        <strong>OpenClaw instances</strong>
                        {loadingInstances ? (
                          <p>Loading local instances...</p>
                        ) : openClawInstances.length ? (
                          <div className="adminx-form-grid">
                            {openClawInstances.map((instance) => (
                              <label key={instance.id} className="adminx-row">
                                <Checkbox
                                  checked={selectedInstanceIds.includes(instance.id)}
                                  onChange={(event) =>
                                    setSelectedInstanceIds((current) =>
                                      event.target.checked
                                        ? [...current, instance.id]
                                        : current.filter((item) => item !== instance.id),
                                    )
                                  }
                                />
                                <span>
                                  {instance.label}
                                  {instance.detail ? ` · ${instance.detail}` : ''}
                                </span>
                              </label>
                            ))}
                          </div>
                        ) : (
                          <p>No OpenClaw instances were detected on this machine yet.</p>
                        )}
                      </div>
                    ) : null}

                    {selectedPlan.snippets.map((snippet) => (
                      <div key={snippet.id} className="adminx-note">
                        <strong>{snippet.title}</strong>
                        <p>{snippet.target}</p>
                        <pre>
                          <code>{snippet.content}</code>
                        </pre>
                      </div>
                    ))}

                    <div className="adminx-note">
                      <strong>cURL</strong>
                      <pre>
                        <code>
                          {buildApiKeyCurlSnippet(
                            gatewayBaseUrl,
                            usagePlaintext || '<rotate-to-reveal-a-new-api-key>',
                          )}
                        </code>
                      </pre>
                    </div>

                    <DialogFooter>
                      {usagePlaintext ? (
                        <InlineButton onClick={() => void copyToClipboard(usagePlaintext)}>
                          Copy Api key
                        </InlineButton>
                      ) : null}
                      <InlineButton
                        tone="primary"
                        disabled={
                          applyingClientId === selectedPlan.id ||
                          !usagePlaintext ||
                          (selectedPlan.requiresInstances && !selectedInstanceIds.length)
                        }
                        onClick={() => void handleApplySetup(selectedPlan)}
                      >
                        {applyingClientId === selectedPlan.id ? 'Applying...' : 'Apply setup'}
                      </InlineButton>
                    </DialogFooter>
                  </>
                ) : null}

                {usageStatus ? (
                  <div className="adminx-note">
                    <strong>Status</strong>
                    <p>{usageStatus}</p>
                  </div>
                ) : null}
              </div>
            ) : null}
          </AdminDialog>
        </DialogContent>
      </Dialog>

      <ConfirmDialog
        open={Boolean(pendingDelete)}
        title="Delete Api key"
        detail={
          pendingDelete
            ? `Delete ${pendingDelete.label || pendingDelete.project_id}. This clears the key registry row, reveal cache, and local route overlay.`
            : ''
        }
        confirmLabel="Delete"
        onClose={() => setPendingDelete(null)}
        onConfirm={confirmDelete}
      />
    </div>
  );
}
