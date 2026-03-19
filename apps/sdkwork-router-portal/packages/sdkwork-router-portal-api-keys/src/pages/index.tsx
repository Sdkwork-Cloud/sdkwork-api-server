import { startTransition, useDeferredValue, useEffect, useMemo, useState } from 'react';
import type { FormEvent } from 'react';
import { InlineButton, copyText } from 'sdkwork-router-portal-commons';
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
  createEmptyPortalApiKeyFormState,
  resolvePortalApiKeyEnvironment,
  resolvePortalApiKeyExpiresAt,
  resolvePortalApiKeyNotes,
  resolvePortalApiKeyPlaintext,
} from '../services';
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
  const [status, setStatus] = useState('Loading issued keys...');
  const [copyStatus, setCopyStatus] = useState('Plaintext keys are only shown once at creation time.');
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

      await refresh();
      setCreatedKey(nextKey);
      setCopyStatus(
        formState.keyMode === 'custom'
          ? 'Custom key saved in write-only mode. Verify the plaintext value now before closing this screen.'
          : 'Copy the plaintext secret now. It will not be shown again after you leave this screen.',
      );
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
    if (!createdKey) {
      return;
    }

    const copied = await copyText(createdKey.plaintext);
    setCopyStatus(
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
    () => buildPortalApiKeysViewModel(apiKeys, createdKey, resolvedFilters),
    [apiKeys, createdKey, resolvedFilters],
  );
  const usagePreview = useMemo(
    () => (usageKey ? buildPortalApiKeyUsagePreview(usageKey, createdKey) : null),
    [createdKey, usageKey],
  );

  return (
    <div data-slot="api-router-page" className="h-full overflow-y-auto bg-zinc-50 dark:bg-zinc-950">
      <div className="flex w-full flex-col gap-4 px-4 py-4 sm:px-4 sm:py-6 xl:px-4 xl:py-6">
        <PortalApiKeyManagerToolbar
          environment={filters.environment}
          environmentOptions={viewModel.environment_options}
          onEnvironmentChange={(value) =>
            startTransition(() => {
              setFilters((current) => ({ ...current, environment: value }));
            })
          }
          onOpenCreate={() => setCreateDialogOpen(true)}
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

        <div className="flex flex-col gap-2 lg:flex-row lg:items-center lg:justify-between">
          <p className="text-sm text-zinc-500 dark:text-zinc-400">{status}</p>
          <div className="flex flex-wrap items-center gap-3">
            {createdKey ? (
              <>
                <span className="text-sm text-emerald-700 dark:text-emerald-300">
                  One-time plaintext available for {createdKey.environment}.
                </span>
                <button
                  type="button"
                  onClick={() => void handleCopyPlaintext()}
                  className="inline-flex h-9 items-center justify-center rounded-2xl border border-emerald-200 bg-emerald-50 px-3 text-sm font-medium text-emerald-700 transition hover:bg-emerald-100 dark:border-emerald-500/20 dark:bg-emerald-500/10 dark:text-emerald-300 dark:hover:bg-emerald-500/15"
                >
                  Copy plaintext
                </button>
                <span className="text-sm text-zinc-500 dark:text-zinc-400">{copyStatus}</span>
              </>
            ) : null}
            <InlineButton onClick={() => onNavigate('usage')} tone="secondary">
              Open usage
            </InlineButton>
          </div>
        </div>

        <PortalApiKeyTable
          items={viewModel.filtered_keys}
          latestCreatedKey={createdKey}
          mutatingKey={mutatingKey}
          onCopyLatestPlaintext={() => void handleCopyPlaintext()}
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
          submitting={submitting}
          usageKey={usageKey}
          usagePreview={usagePreview}
        />
      </div>
    </div>
  );
}
