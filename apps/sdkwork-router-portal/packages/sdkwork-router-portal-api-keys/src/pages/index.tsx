import { startTransition, useDeferredValue, useEffect, useMemo, useState } from 'react';
import type { FormEvent } from 'react';
import { copyText } from 'sdkwork-router-portal-commons';
import { portalErrorMessage } from 'sdkwork-router-portal-portal-api';
import type { CreatedGatewayApiKey, GatewayApiKeyRecord } from 'sdkwork-router-portal-types';

import {
  PortalApiKeyDialogs,
  PortalApiKeyManagerToolbar,
  PortalApiKeyTable,
} from '../components';
import {
  issuePortalApiKey,
  loadPortalApiKeys,
  removePortalApiKey,
  setPortalApiKeyActive,
} from '../repository';
import {
  buildPortalApiKeyUsagePreview,
  buildPortalApiKeysViewModel,
  clearPortalApiKeyPlaintextReveal,
  createEmptyPortalApiKeyFormState,
  readPortalApiKeyPlaintextReveal,
  rememberPortalApiKeyPlaintextReveal,
  resolvePortalApiKeyEnvironment,
  resolvePortalApiKeyExpiresAt,
  resolvePortalApiKeyNotes,
  resolvePortalApiKeyPlaintext,
} from '../services';
import {
  applyApiKeyQuickSetup,
  buildApiKeyQuickSetupPlans,
  listApiKeyInstances,
  resolveGatewayBaseUrl,
  type ApiKeySetupClientId,
  type ApiKeySetupInstance,
} from '../services/quickSetup';
import type {
  PortalApiKeyCreateFormState,
  PortalApiKeyFilterState,
  PortalApiKeysPageProps,
} from '../types';

export function PortalApiKeysPage({ onNavigate }: PortalApiKeysPageProps) {
  const [apiKeys, setApiKeys] = useState<GatewayApiKeyRecord[]>([]);
  const [createdKey, setCreatedKey] = useState<CreatedGatewayApiKey | null>(null);
  const [filters, setFilters] = useState<PortalApiKeyFilterState>({
    searchQuery: '',
    environment: 'all',
  });
  const [createDialogOpen, setCreateDialogOpen] = useState(false);
  const [usageKey, setUsageKey] = useState<GatewayApiKeyRecord | null>(null);
  const [formState, setFormState] = useState<PortalApiKeyCreateFormState>(
    createEmptyPortalApiKeyFormState,
  );
  const [gatewayBaseUrl, setGatewayBaseUrl] = useState('http://127.0.0.1:8080');
  const [openClawInstances, setOpenClawInstances] = useState<ApiKeySetupInstance[]>([]);
  const [loadingInstances, setLoadingInstances] = useState(false);
  const [selectedClientId, setSelectedClientId] = useState<ApiKeySetupClientId>('codex');
  const [selectedInstanceIds, setSelectedInstanceIds] = useState<string[]>([]);
  const [applyingClientId, setApplyingClientId] = useState<ApiKeySetupClientId | null>(null);
  const [usageStatus, setUsageStatus] = useState('');
  const [, setStatus] = useState('Loading issued keys...');
  const [submitting, setSubmitting] = useState(false);
  const [mutatingKey, setMutatingKey] = useState<string | null>(null);
  const deferredSearchQuery = useDeferredValue(filters.searchQuery);

  async function refresh() {
    const keys = await loadPortalApiKeys();
    setApiKeys(keys);
  }

  useEffect(() => {
    let cancelled = false;

    void refresh()
      .then(() => {
        if (!cancelled) {
          setStatus('Credential inventory is synced with the latest project key state.');
        }
      })
      .catch((error) => {
        if (!cancelled) {
          setStatus(portalErrorMessage(error));
        }
      });

    return () => {
      cancelled = true;
    };
  }, []);

  async function handleCreate(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();

    if (!formState.label.trim()) {
      setStatus('Key label is required so credentials remain auditable after creation.');
      return;
    }

    const environment = resolvePortalApiKeyEnvironment(formState);
    if (!environment) {
      setStatus('Custom environment is required when the custom environment option is selected.');
      return;
    }

    const expiresAtMs = resolvePortalApiKeyExpiresAt(formState);
    if (formState.expiresAt.trim() && !expiresAtMs) {
      setStatus('Expires at must be a valid date before the credential can be created.');
      return;
    }

    const customKey = resolvePortalApiKeyPlaintext(formState);
    if (formState.keyMode === 'custom' && !customKey) {
      setStatus('Custom key mode requires a plaintext key before the credential can be created.');
      return;
    }

    const notes = resolvePortalApiKeyNotes(formState);

    setSubmitting(true);
    setStatus(
      formState.keyMode === 'custom'
        ? `Registering a custom ${environment} key for this workspace...`
        : `Issuing a Portal-managed ${environment} key for this workspace...`,
    );

    try {
      const nextKey = await issuePortalApiKey({
        environment,
        label: formState.label,
        api_key: customKey,
        notes,
        expires_at_ms: expiresAtMs,
      });

      rememberPortalApiKeyPlaintextReveal(nextKey.hashed, nextKey.plaintext);
      await refresh();
      setCreatedKey(nextKey);
      setStatus(
        formState.keyMode === 'custom'
          ? `Custom key stored for ${environment}. Verify the plaintext value before leaving this page.`
          : `Portal-managed key issued for ${environment}. Copy the plaintext secret before leaving this page.`,
      );
      setCreateDialogOpen(false);
      setFormState(createEmptyPortalApiKeyFormState());
      setUsageKey(null);
    } catch (error) {
      setStatus(portalErrorMessage(error));
    } finally {
      setSubmitting(false);
    }
  }

  async function handleCopyPlaintext() {
    const plaintext = usagePlaintext ?? createdKey?.plaintext ?? null;
    if (!plaintext) {
      return;
    }

    const copied = await copyText(plaintext);
    setStatus(
      copied
        ? 'Plaintext key copied to clipboard.'
        : 'Clipboard copy is unavailable in this browser context.',
    );
  }

  async function handleKeyStatusChange(key: GatewayApiKeyRecord, active: boolean) {
    setMutatingKey(key.hashed_key);
    setStatus(`${active ? 'Restoring' : 'Revoking'} ${key.label}...`);

    try {
      await setPortalApiKeyActive(key.hashed_key, active);
      await refresh();
      setStatus(
        active
          ? `${key.label} is active again and can authenticate gateway traffic.`
          : `${key.label} has been revoked and will no longer authenticate requests.`,
      );
    } catch (error) {
      setStatus(portalErrorMessage(error));
    } finally {
      setMutatingKey(null);
    }
  }

  async function handleDeleteKey(key: GatewayApiKeyRecord) {
    setMutatingKey(key.hashed_key);
    setStatus(`Deleting ${key.label}...`);

    try {
      await removePortalApiKey(key.hashed_key);
      clearPortalApiKeyPlaintextReveal(key.hashed_key);
      await refresh();

      if (createdKey?.hashed === key.hashed_key) {
        setCreatedKey(null);
      }

      if (usageKey?.hashed_key === key.hashed_key) {
        setUsageKey(null);
      }

      setStatus(`${key.label} was deleted from this workspace.`);
    } catch (error) {
      setStatus(portalErrorMessage(error));
    } finally {
      setMutatingKey(null);
    }
  }

  const resolvedFilters = useMemo(
    () => ({
      ...filters,
      searchQuery: deferredSearchQuery,
    }),
    [deferredSearchQuery, filters],
  );

  const viewModel = useMemo(
    () => buildPortalApiKeysViewModel(apiKeys, createdKey, resolvedFilters, gatewayBaseUrl),
    [apiKeys, createdKey, gatewayBaseUrl, resolvedFilters],
  );
  const usagePlaintext = usageKey
    ? readPortalApiKeyPlaintextReveal(usageKey.hashed_key) ??
      (createdKey?.hashed === usageKey.hashed_key ? createdKey.plaintext : null)
    : null;
  const usagePreview = useMemo(
    () => (usageKey ? buildPortalApiKeyUsagePreview(usageKey, usagePlaintext, gatewayBaseUrl) : null),
    [gatewayBaseUrl, usageKey, usagePlaintext],
  );
  const quickSetupPlans = useMemo(
    () =>
      usageKey
        ? buildApiKeyQuickSetupPlans({
            hashedKey: usageKey.hashed_key,
            label: usageKey.label,
            plaintextKey: usagePlaintext || '<api-key-not-visible-on-this-device>',
            gatewayBaseUrl,
          })
        : [],
    [gatewayBaseUrl, usageKey, usagePlaintext],
  );
  const selectedPlan =
    quickSetupPlans.find((plan) => plan.id === selectedClientId) ?? quickSetupPlans[0] ?? null;

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
      setSelectedClientId('codex');
      setSelectedInstanceIds([]);
      setUsageStatus('');
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

  async function handleApplySetup() {
    if (!selectedPlan) {
      return;
    }

    if (!usagePlaintext) {
      setUsageStatus('Plaintext Api key is no longer visible on this device. Create a replacement first.');
      return;
    }

    if (selectedPlan.requiresInstances && !selectedInstanceIds.length) {
      setUsageStatus('Select at least one OpenClaw instance before applying setup.');
      return;
    }

    setApplyingClientId(selectedPlan.id);
    setUsageStatus(`Applying ${selectedPlan.label} setup...`);

    try {
      const result = await applyApiKeyQuickSetup({
        ...selectedPlan.request,
        provider: {
          ...selectedPlan.request.provider,
          apiKey: usagePlaintext,
        },
        openClaw: selectedPlan.requiresInstances
          ? {
              instanceIds: selectedInstanceIds,
            }
          : undefined,
      });
      setUsageStatus(
        result.updatedInstanceIds.length
          ? `Applied setup to ${result.updatedInstanceIds.length} OpenClaw instance(s).`
          : `Applied setup and wrote ${result.writtenFiles.length} file(s).`,
      );
    } catch (error) {
      setUsageStatus(portalErrorMessage(error));
    } finally {
      setApplyingClientId(null);
    }
  }

  return (
    <div data-slot="api-router-page" className="h-full overflow-y-auto bg-zinc-50 dark:bg-zinc-950">
      <div className="flex w-full flex-col gap-4 px-4 py-4 sm:px-4 sm:py-6 xl:px-4 xl:py-6">
        <PortalApiKeyManagerToolbar
          onOpenCreate={() => setCreateDialogOpen(true)}
          onOpenUsage={() => onNavigate('usage')}
          onRefresh={() => {
            void refresh().then(() =>
              setStatus('Credential inventory is synced with the latest project key state.'),
            );
          }}
          onSearchChange={(value) =>
            startTransition(() => {
              setFilters((current) => ({ ...current, searchQuery: value }));
            })
          }
          searchQuery={filters.searchQuery}
        />

        <PortalApiKeyTable
          items={viewModel.filtered_keys}
          latestCreatedKey={createdKey}
          mutatingKey={mutatingKey}
          onCopyLatestPlaintext={() => void handleCopyPlaintext()}
          onCopyPlaintext={(item) => {
            const plaintext =
              readPortalApiKeyPlaintextReveal(item.hashed_key) ??
              (createdKey?.hashed === item.hashed_key ? createdKey.plaintext : null);
            if (plaintext) {
              void copyText(plaintext);
            }
          }}
          resolvePlaintext={(item) =>
            readPortalApiKeyPlaintextReveal(item.hashed_key) ??
            (createdKey?.hashed === item.hashed_key ? createdKey.plaintext : null)
          }
          onDelete={(item) => void handleDeleteKey(item)}
          onOpenUsage={setUsageKey}
          onToggleStatus={(item) => void handleKeyStatusChange(item, !item.active)}
        />
        <span className="sr-only">Usage method</span>

        <PortalApiKeyDialogs
          createFormState={formState}
          createOpen={createDialogOpen}
          createdKey={createdKey}
          onChangeForm={(updater) => setFormState((current) => updater(current))}
          onCloseCreate={() => setCreateDialogOpen(false)}
          onCloseUsage={() => setUsageKey(null)}
          onCopyPlaintext={() => void handleCopyPlaintext()}
          onCreate={(event) => void handleCreate(event)}
          applyingClientId={applyingClientId}
          gatewayBaseUrl={gatewayBaseUrl}
          loadingInstances={loadingInstances}
          onApplySetup={() => void handleApplySetup()}
          onChangeInstanceSelection={setSelectedInstanceIds}
          onSelectClient={setSelectedClientId}
          submitting={submitting}
          openClawInstances={openClawInstances}
          quickSetupPlans={quickSetupPlans}
          selectedClientId={selectedClientId}
          selectedInstanceIds={selectedInstanceIds}
          selectedPlan={selectedPlan}
          usagePlaintext={usagePlaintext}
          usageStatus={usageStatus}
          usageKey={usageKey}
          usagePreview={usagePreview}
        />
      </div>
    </div>
  );
}
