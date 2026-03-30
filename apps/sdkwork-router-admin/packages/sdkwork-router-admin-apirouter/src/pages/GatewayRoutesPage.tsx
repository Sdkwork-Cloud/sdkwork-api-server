import { useDeferredValue, useMemo, useState } from 'react';
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
  Surface,
  ToolbarInline,
  ToolbarSearchField,
} from 'sdkwork-router-admin-commons';
import type {
  AdminPageProps,
  ProxyProviderRecord,
} from 'sdkwork-router-admin-types';

import { buildGatewayRouteInventory } from '../services/gatewayViewService';

type GatewayRoutesPageProps = AdminPageProps & {
  onRefreshWorkspace: () => Promise<void>;
  onSaveProvider: (input: {
    id: string;
    channel_id: string;
    extension_id?: string;
    adapter_kind: string;
    base_url: string;
    display_name: string;
    channel_bindings: Array<{ channel_id: string; is_primary: boolean }>;
  }) => Promise<void>;
  onDeleteProvider: (providerId: string) => Promise<void>;
};

type ProviderDraft = {
  id: string;
  display_name: string;
  adapter_kind: string;
  base_url: string;
  extension_id: string;
  primary_channel_id: string;
  bound_channel_ids: string[];
};

function collectProviderChannelIds(provider: ProxyProviderRecord): string[] {
  const ids = new Set<string>([provider.channel_id]);
  for (const binding of provider.channel_bindings) {
    ids.add(binding.channel_id);
  }

  return Array.from(ids);
}

function emptyProviderDraft(defaultChannelId: string): ProviderDraft {
  return {
    id: '',
    display_name: '',
    adapter_kind: 'openai',
    base_url: '',
    extension_id: '',
    primary_channel_id: defaultChannelId,
    bound_channel_ids: defaultChannelId ? [defaultChannelId] : [],
  };
}

function providerDraftFromRecord(provider: ProxyProviderRecord): ProviderDraft {
  return {
    id: provider.id,
    display_name: provider.display_name,
    adapter_kind: provider.adapter_kind,
    base_url: provider.base_url,
    extension_id: provider.extension_id ?? '',
    primary_channel_id: provider.channel_id,
    bound_channel_ids: collectProviderChannelIds(provider),
  };
}

export function GatewayRoutesPage({
  snapshot,
  onRefreshWorkspace,
  onSaveProvider,
  onDeleteProvider,
}: GatewayRoutesPageProps) {
  const defaultChannelId = snapshot.channels[0]?.id ?? '';
  const [search, setSearch] = useState('');
  const [channelFilter, setChannelFilter] = useState('all');
  const [healthFilter, setHealthFilter] = useState<'all' | 'healthy' | 'degraded'>('all');
  const [editingProvider, setEditingProvider] = useState<ProxyProviderRecord | null>(null);
  const [providerDraft, setProviderDraft] = useState<ProviderDraft>(() =>
    emptyProviderDraft(defaultChannelId),
  );
  const [isProviderDialogOpen, setIsProviderDialogOpen] = useState(false);
  const [pendingDelete, setPendingDelete] = useState<ProxyProviderRecord | null>(null);
  const deferredSearch = useDeferredValue(search.trim().toLowerCase());

  const inventory = useMemo(() => buildGatewayRouteInventory(snapshot), [snapshot]);
  const filteredInventory = inventory.filter((row) => {
    if (channelFilter !== 'all' && !row.channels.some((channel) => channel.id === channelFilter)) {
      return false;
    }
    if (healthFilter === 'healthy' && !row.healthy) {
      return false;
    }
    if (healthFilter === 'degraded' && row.healthy) {
      return false;
    }

    if (!deferredSearch) {
      return true;
    }

    const haystack = [
      row.provider.id,
      row.provider.display_name,
      row.provider.adapter_kind,
      row.provider.base_url,
      ...row.channels.map((channel) => `${channel.id} ${channel.name}`),
      ...row.credentials.map((credential) => credential.key_reference),
    ]
      .join(' ')
      .toLowerCase();

    return haystack.includes(deferredSearch);
  });

  function resetProviderDialog(): void {
    setEditingProvider(null);
    setProviderDraft(emptyProviderDraft(defaultChannelId));
    setIsProviderDialogOpen(false);
  }

  function openNewProviderDialog(): void {
    setEditingProvider(null);
    setProviderDraft(emptyProviderDraft(defaultChannelId));
    setIsProviderDialogOpen(true);
  }

  function openEditProviderDialog(provider: ProxyProviderRecord): void {
    setEditingProvider(provider);
    setProviderDraft(providerDraftFromRecord(provider));
    setIsProviderDialogOpen(true);
  }

  async function handleProviderSubmit(event: FormEvent<HTMLFormElement>): Promise<void> {
    event.preventDefault();
    const bindingIds = Array.from(
      new Set(
        [providerDraft.primary_channel_id, ...providerDraft.bound_channel_ids].filter(Boolean),
      ),
    );
    await onSaveProvider({
      id: providerDraft.id.trim(),
      channel_id: providerDraft.primary_channel_id,
      extension_id: providerDraft.extension_id.trim() || undefined,
      adapter_kind: providerDraft.adapter_kind.trim(),
      base_url: providerDraft.base_url.trim(),
      display_name: providerDraft.display_name.trim(),
      channel_bindings: bindingIds.map((channelId) => ({
        channel_id: channelId,
        is_primary: channelId === providerDraft.primary_channel_id,
      })),
    });
    resetProviderDialog();
  }

  async function confirmDelete(): Promise<void> {
    if (!pendingDelete) {
      return;
    }

    await onDeleteProvider(pendingDelete.id);
    setPendingDelete(null);
  }

  return (
    <div className="adminx-page-grid">
      <PageToolbar
        compact
        actions={(
          <>
            <InlineButton tone="primary" onClick={openNewProviderDialog}>
              New route provider
            </InlineButton>
            <InlineButton onClick={() => void onRefreshWorkspace()}>
              Refresh workspace
            </InlineButton>
          </>
        )}
      >
        <ToolbarInline>
          <ToolbarSearchField
            label="Search providers"
            value={search}
            onChange={(event) => setSearch(event.target.value)}
            placeholder="provider, base url, channel..."
          />
        </ToolbarInline>
      </PageToolbar>

      <DataTable
        columns={[
          {
            key: 'provider',
            label: 'Provider',
            render: (row) => (
              <div className="adminx-table-cell-stack">
                <strong>{row.provider.display_name}</strong>
                <span>
                  {row.provider.id} / {row.provider.adapter_kind}
                </span>
              </div>
            ),
          },
          {
            key: 'channels',
            label: 'Channels',
            render: (row) =>
              row.channels.map((channel) => channel.name).join(', ') || row.primary_channel_id,
          },
          {
            key: 'base_url',
            label: 'Base URL',
            render: (row) => row.provider.base_url,
          },
          {
            key: 'credentials',
            label: 'Credential inventory',
            render: (row) =>
              row.credentials.length
                ? row.credentials.map((credential) => credential.key_reference).join(', ')
                : 'No credentials',
          },
          {
            key: 'coverage',
            label: 'Models / pricing',
            render: (row) => `${row.model_count} / ${row.price_count}`,
          },
          {
            key: 'health',
            label: 'Health',
            render: (row) => (
              <Pill tone={row.healthy ? 'live' : 'danger'}>
                {row.health_status}
              </Pill>
            ),
          },
          {
            key: 'actions',
            label: 'Actions',
            render: (row) => (
              <div className="adminx-row">
                <InlineButton onClick={() => openEditProviderDialog(row.provider)}>
                  Edit route
                </InlineButton>
                <InlineButton
                  tone="danger"
                  onClick={() => setPendingDelete(row.provider)}
                >
                  Delete
                </InlineButton>
              </div>
            ),
          },
        ]}
        rows={filteredInventory}
        empty="No route providers match the current filter."
        getKey={(row) => row.provider.id}
      />

      <Dialog
        open={isProviderDialogOpen}
        onOpenChange={(nextOpen) => (nextOpen ? setIsProviderDialogOpen(true) : resetProviderDialog())}
      >
        <DialogContent size="large">
          <AdminDialog
            title={editingProvider ? 'Edit route provider' : 'New route provider'}
            detail="Keep route config focused on provider posture and upstream connectivity. Credential lifecycle and model publication stay in Catalog."
          >
            <form className="adminx-form-grid" onSubmit={(event) => void handleProviderSubmit(event)}>
              <FormField label="Provider id">
                <Input
                  value={providerDraft.id}
                  onChange={(event) =>
                    setProviderDraft((current) => ({ ...current, id: event.target.value }))
                  }
                  disabled={Boolean(editingProvider)}
                  required
                />
              </FormField>
              <FormField label="Display name">
                <Input
                  value={providerDraft.display_name}
                  onChange={(event) =>
                    setProviderDraft((current) => ({
                      ...current,
                      display_name: event.target.value,
                    }))
                  }
                  required
                />
              </FormField>
              <FormField label="Adapter kind">
                <Input
                  value={providerDraft.adapter_kind}
                  onChange={(event) =>
                    setProviderDraft((current) => ({
                      ...current,
                      adapter_kind: event.target.value,
                    }))
                  }
                  required
                />
              </FormField>
              <FormField label="Base URL">
                <Input
                  value={providerDraft.base_url}
                  onChange={(event) =>
                    setProviderDraft((current) => ({
                      ...current,
                      base_url: event.target.value,
                    }))
                  }
                  required
                />
              </FormField>
              <FormField label="Extension id">
                <Input
                  value={providerDraft.extension_id}
                  onChange={(event) =>
                    setProviderDraft((current) => ({
                      ...current,
                      extension_id: event.target.value,
                    }))
                  }
                />
              </FormField>
              <FormField label="Primary channel">
                <Select
                  value={providerDraft.primary_channel_id}
                  onChange={(event) =>
                    setProviderDraft((current) => ({
                      ...current,
                      primary_channel_id: event.target.value,
                      bound_channel_ids: current.bound_channel_ids.includes(event.target.value)
                        ? current.bound_channel_ids
                        : [...current.bound_channel_ids, event.target.value],
                    }))
                  }
                >
                  {snapshot.channels.map((channel) => (
                    <option key={channel.id} value={channel.id}>
                      {channel.name} ({channel.id})
                    </option>
                  ))}
                </Select>
              </FormField>
              <Surface
                title="Bound channels"
                detail="Expose the provider on one or more public API channels."
              >
                <div className="adminx-form-grid">
                  {snapshot.channels.map((channel) => {
                    const checked = providerDraft.bound_channel_ids.includes(channel.id);
                    return (
                      <label key={channel.id} className="adminx-field">
                        <span>{channel.name}</span>
                        <Checkbox
                          checked={checked}
                          onChange={(event) =>
                            setProviderDraft((current) => ({
                              ...current,
                              bound_channel_ids: event.target.checked
                                ? [...current.bound_channel_ids, channel.id]
                                : current.bound_channel_ids.filter((id) => id !== channel.id),
                            }))
                          }
                        />
                      </label>
                    );
                  })}
                </div>
              </Surface>
              <DialogFooter>
                <InlineButton onClick={resetProviderDialog}>Cancel</InlineButton>
                <InlineButton tone="primary" type="submit">
                  {editingProvider ? 'Save provider' : 'Create provider'}
                </InlineButton>
              </DialogFooter>
            </form>
          </AdminDialog>
        </DialogContent>
      </Dialog>

      <ConfirmDialog
        open={Boolean(pendingDelete)}
        title="Delete route provider"
        detail={
          pendingDelete
            ? `Delete ${pendingDelete.display_name}. Route bindings, model pricing, and downstream route selection should be reviewed before removing the provider.`
            : ''
        }
        confirmLabel="Delete provider"
        onClose={() => setPendingDelete(null)}
        onConfirm={confirmDelete}
      />
    </div>
  );
}
