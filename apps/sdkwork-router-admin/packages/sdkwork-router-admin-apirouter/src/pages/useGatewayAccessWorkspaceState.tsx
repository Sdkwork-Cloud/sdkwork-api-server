import { useDeferredValue, useEffect, useMemo, useState } from 'react';
import type { FormEvent } from 'react';
import type {
  AdminPageProps,
  ApiKeyGroupRecord,
  CreatedGatewayApiKey,
  GatewayApiKeyRecord,
} from 'sdkwork-router-admin-types';

import {
  applyApiKeyQuickSetup,
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
} from '../services/gatewayOverlayStore';
import {
  buildQuickSetupSummary,
  buildUsageKeyFromCreateResponse,
  createDraft,
  editDraftFromKey,
  filterApiKeyGroupsByScope,
  isExpiringSoon,
  parseExpiryInput,
  resolvePlaintextForKey,
  routeDraftFromKey,
  type CreateDraft,
  type EditDraft,
  type RouteDraft,
} from './access/shared';

type GatewayAccessWorkspaceOptions = {
  snapshot: AdminPageProps['snapshot'];
  onRefreshWorkspace: () => Promise<void>;
  onCreateApiKey: (input: {
    tenant_id: string;
    project_id: string;
    environment: string;
    label?: string;
    notes?: string;
    expires_at_ms?: number | null;
    plaintext_key?: string;
    api_key_group_id?: string | null;
  }) => Promise<CreatedGatewayApiKey>;
  onUpdateApiKey: (input: {
    hashed_key: string;
    tenant_id: string;
    project_id: string;
    environment: string;
    label: string;
    notes?: string | null;
    expires_at_ms?: number | null;
    api_key_group_id?: string | null;
  }) => Promise<void>;
  onUpdateApiKeyStatus: (hashedKey: string, active: boolean) => Promise<void>;
  onDeleteApiKey: (hashedKey: string) => Promise<void>;
};

export function useGatewayAccessWorkspaceState({
  snapshot,
  onRefreshWorkspace,
  onCreateApiKey,
  onUpdateApiKey,
  onUpdateApiKeyStatus,
  onDeleteApiKey,
}: GatewayAccessWorkspaceOptions) {
  const defaultTenantId = snapshot.tenants[0]?.id ?? 'tenant_local_demo';
  const defaultProjectId = snapshot.projects[0]?.id ?? 'project_local_demo';

  const [search, setSearch] = useState('');
  const [selectedKeyId, setSelectedKeyId] = useState<string | null>(null);
  const [isDetailDrawerOpen, setIsDetailDrawerOpen] = useState(false);
  const [isGroupsDialogOpen, setIsGroupsDialogOpen] = useState(false);
  const [isCreateOpen, setIsCreateOpen] = useState(false);
  const [createDraftState, setCreateDraftState] = useState<CreateDraft>(() =>
    createDraft(defaultTenantId, defaultProjectId),
  );
  const [editingKey, setEditingKey] = useState<GatewayApiKeyRecord | null>(null);
  const [editDraft, setEditDraft] = useState<EditDraft>({
    label: '',
    notes: '',
    expires_at: '',
    api_key_group_id: '',
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
  const [openClawInstances, setOpenClawInstances] = useState<ApiKeySetupInstance[]>(
    [],
  );
  const [loadingInstances, setLoadingInstances] = useState(false);
  const [selectedClientId, setSelectedClientId] =
    useState<ApiKeySetupClientId>('codex');
  const [selectedInstanceIds, setSelectedInstanceIds] = useState<string[]>([]);
  const [applyingClientId, setApplyingClientId] =
    useState<ApiKeySetupClientId | null>(null);
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

  const modelMappings = useMemo(
    () => listGatewayModelMappings(),
    [overlayVersion],
  );
  const mappingById = useMemo(
    () => new Map(modelMappings.map((mapping) => [mapping.id, mapping])),
    [modelMappings],
  );
  const providerById = useMemo(
    () => new Map(snapshot.providers.map((provider) => [provider.id, provider])),
    [snapshot.providers],
  );
  const groupById = useMemo(
    () => new Map(snapshot.apiKeyGroups.map((group) => [group.group_id, group])),
    [snapshot.apiKeyGroups],
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
          const group = key.api_key_group_id ? groupById.get(key.api_key_group_id) : null;
          const haystack = [
            key.label,
            key.tenant_id,
            key.project_id,
            key.environment,
            key.notes ?? '',
            key.hashed_key,
            key.api_key_group_id ?? '',
            group?.name ?? '',
            group?.slug ?? '',
            overlay.route_provider_id ?? '',
            overlay.model_mapping_id ?? '',
            reveal?.plaintext_key ?? '',
          ]
            .join(' ')
            .toLowerCase();

          return haystack.includes(deferredSearch);
        }),
    [deferredSearch, groupById, snapshot.apiKeys],
  );

  useEffect(() => {
    if (!filteredKeys.length) {
      if (selectedKeyId !== null) {
        setSelectedKeyId(null);
      }
      if (isDetailDrawerOpen) {
        setIsDetailDrawerOpen(false);
      }
      return;
    }

    if (selectedKeyId && filteredKeys.some((key) => key.hashed_key === selectedKeyId)) {
      return;
    }

    setSelectedKeyId(filteredKeys[0]?.hashed_key ?? null);
    if (isDetailDrawerOpen) {
      setIsDetailDrawerOpen(false);
    }
  }, [filteredKeys, isDetailDrawerOpen, selectedKeyId]);

  const selectedKeyRecord =
    filteredKeys.find((key) => key.hashed_key === selectedKeyId)
    ?? filteredKeys[0]
    ?? null;
  const availableCreateProjects = snapshot.projects.filter(
    (project) => project.tenant_id === createDraftState.tenant_id,
  );
  const availableCreateGroups = useMemo(
    () =>
      filterApiKeyGroupsByScope(snapshot.apiKeyGroups, {
        tenant_id: createDraftState.tenant_id,
        project_id: createDraftState.project_id,
        environment: createDraftState.environment,
      }).filter((group) => group.active),
    [
      createDraftState.environment,
      createDraftState.project_id,
      createDraftState.tenant_id,
      snapshot.apiKeyGroups,
    ],
  );
  const availableEditGroups = useMemo(() => {
    if (!editingKey) {
      return [] as ApiKeyGroupRecord[];
    }

    return filterApiKeyGroupsByScope(snapshot.apiKeyGroups, {
      tenant_id: editingKey.tenant_id,
      project_id: editingKey.project_id,
      environment: editingKey.environment,
    }).filter(
      (group) =>
        group.active || group.group_id === editingKey.api_key_group_id,
    );
  }, [editingKey, snapshot.apiKeyGroups]);
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
                snapshot.models.find((model) =>
                  model.external_name.includes('claude'),
                )?.external_name ?? snapshot.models[0]?.external_name,
              geminiModel:
                snapshot.models.find((model) =>
                  model.external_name.includes('gemini'),
                )?.external_name ?? 'gemini-2.5-pro',
            },
          })
        : [],
    [gatewayBaseUrl, snapshot.models, usageKey, usagePlaintext],
  );
  const usageOverlay = usageKey
    ? readGatewayApiKeyOverlay(usageKey.hashed_key)
    : null;

  const totalKeys = snapshot.apiKeys.length;
  const activeKeys = snapshot.apiKeys.filter((key) => key.active).length;
  const customRouteCount = snapshot.apiKeys.filter(
    (key) => readGatewayApiKeyOverlay(key.hashed_key).route_mode === 'custom',
  ).length;
  const expiringSoonCount = snapshot.apiKeys.filter(isExpiringSoon).length;

  function refreshOverlay() {
    setOverlayVersion((value) => value + 1);
  }

  async function handleRefreshWorkspace() {
    await onRefreshWorkspace();
    refreshOverlay();
  }

  function handleSearchChange(value: string) {
    setSearch(value);
  }

  function clearSearch() {
    setSearch('');
  }

  function openCreateDialog() {
    setIsCreateOpen(true);
  }

  function resetCreateDialog() {
    setCreateDraftState(createDraft(defaultTenantId, defaultProjectId));
    setIsCreateOpen(false);
  }

  function handleCreateDialogOpenChange(nextOpen: boolean) {
    if (nextOpen) {
      setIsCreateOpen(true);
      return;
    }
    resetCreateDialog();
  }

  function openEditDialog(key: GatewayApiKeyRecord) {
    setEditingKey(key);
    setEditDraft(editDraftFromKey(key));
  }

  function handleEditDialogOpenChange(nextOpen: boolean) {
    if (!nextOpen) {
      setEditingKey(null);
    }
  }

  function openRouteDialog(key: GatewayApiKeyRecord) {
    setRouteKey(key);
    setRouteDraft(routeDraftFromKey(key));
  }

  function handleRouteDialogOpenChange(nextOpen: boolean) {
    if (!nextOpen) {
      setRouteKey(null);
    }
  }

  function openUsageDialog(key: GatewayApiKeyRecord) {
    setUsageKey(key);
  }

  function handleUsageDialogOpenChange(nextOpen: boolean) {
    if (!nextOpen) {
      setUsageKey(null);
    }
  }

  function handleDeleteDialogOpenChange(open: boolean) {
    if (!open) {
      setPendingDelete(null);
    }
  }

  function handleSelectKey(key: GatewayApiKeyRecord) {
    setSelectedKeyId(key.hashed_key);
  }

  function openDetailDrawer(key: GatewayApiKeyRecord) {
    setSelectedKeyId(key.hashed_key);
    setIsDetailDrawerOpen(true);
  }

  function handleDetailDrawerOpenChange(open: boolean) {
    setIsDetailDrawerOpen(open);
    if (!open) {
      setSelectedKeyId(null);
    }
  }

  async function handleCreateSubmit(event: FormEvent<HTMLFormElement>) {
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
      api_key_group_id: createDraftState.api_key_group_id || undefined,
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

  async function handleEditSubmit(event: FormEvent<HTMLFormElement>) {
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
      api_key_group_id: editDraft.api_key_group_id || null,
    });
    setEditingKey(null);
  }

  async function handleRouteSubmit(event: FormEvent<HTMLFormElement>) {
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

  async function confirmDelete() {
    if (!pendingDelete) {
      return;
    }

    await onDeleteApiKey(pendingDelete.hashed_key);
    clearGatewayApiKeyOverlay(pendingDelete.hashed_key);
    clearGatewayApiKeyPlaintextReveal(pendingDelete.hashed_key);
    refreshOverlay();
    if (selectedKeyId === pendingDelete.hashed_key) {
      setSelectedKeyId(null);
      setIsDetailDrawerOpen(false);
    }
    setPendingDelete(null);
  }

  async function handleApplySetup(plan: ApiKeyQuickSetupPlan) {
    if (!usagePlaintext) {
      setUsageStatus('The plaintext API key is no longer visible on this device.');
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
        provider: { ...plan.request.provider, apiKey: usagePlaintext },
        openClaw: plan.requiresInstances
          ? { instanceIds: selectedInstanceIds }
          : undefined,
      });
      setUsageStatus(buildQuickSetupSummary(result));
    } catch (error) {
      setUsageStatus(error instanceof Error ? error.message : 'Failed to apply setup.');
    } finally {
      setApplyingClientId(null);
    }
  }

  async function handleToggleKeyStatus(key: GatewayApiKeyRecord) {
    await onUpdateApiKeyStatus(key.hashed_key, !key.active);
  }

  const deleteDialogDescription = pendingDelete
    ? `Delete ${pendingDelete.label || pendingDelete.project_id}. This clears the key registry row, reveal cache, and local route overlay.`
    : '';

  return {
    search,
    isDetailDrawerOpen,
    isGroupsDialogOpen,
    isCreateOpen,
    createDraftState,
    editingKey,
    editDraft,
    routeKey,
    routeDraft,
    usageKey,
    pendingDelete,
    gatewayBaseUrl,
    openClawInstances,
    loadingInstances,
    selectedClientId,
    selectedInstanceIds,
    applyingClientId,
    usageStatus,
    modelMappings,
    mappingById,
    providerById,
    groupById,
    filteredKeys,
    selectedKeyRecord,
    availableCreateProjects,
    availableCreateGroups,
    availableEditGroups,
    usagePlaintext,
    quickSetupPlans,
    usageOverlay,
    totalKeys,
    activeKeys,
    customRouteCount,
    expiringSoonCount,
    deleteDialogDescription,
    openCreateDialog,
    handleRefreshWorkspace,
    handleSearchChange,
    clearSearch,
    resetCreateDialog,
    handleCreateDialogOpenChange,
    openEditDialog,
    handleEditDialogOpenChange,
    openRouteDialog,
    handleRouteDialogOpenChange,
    openUsageDialog,
    handleUsageDialogOpenChange,
    handleDeleteDialogOpenChange,
    handleSelectKey,
    openDetailDrawer,
    handleDetailDrawerOpenChange,
    handleCreateSubmit,
    handleEditSubmit,
    handleRouteSubmit,
    confirmDelete,
    handleApplySetup,
    handleToggleKeyStatus,
    setCreateDraftState,
    setEditDraft,
    setRouteDraft,
    setSelectedClientId,
    setSelectedInstanceIds,
    setIsGroupsDialogOpen,
    setPendingDelete,
  };
}
