import { useState } from 'react';
import type { FormEvent } from 'react';

import {
  AdminDialog,
  ConfirmDialog,
  DataTable,
  Dialog,
  DialogContent,
  DialogFooter,
  DialogTrigger,
  FormField,
  InlineButton,
  PageToolbar,
  Pill,
  StatCard,
  Surface,
} from 'sdkwork-router-admin-commons';
import type { AdminPageProps, CredentialRecord } from 'sdkwork-router-admin-types';

type CatalogPageProps = AdminPageProps & {
  onSaveChannel: (input: { id: string; name: string }) => Promise<void>;
  onSaveProvider: (input: {
    id: string;
    channel_id: string;
    extension_id?: string;
    adapter_kind: string;
    base_url: string;
    display_name: string;
    channel_bindings: Array<{ channel_id: string; is_primary: boolean }>;
  }) => Promise<void>;
  onSaveCredential: (input: {
    tenant_id: string;
    provider_id: string;
    key_reference: string;
    secret_value: string;
  }) => Promise<void>;
  onSaveModel: (input: {
    external_name: string;
    provider_id: string;
    capabilities: string[];
    streaming: boolean;
    context_window?: number;
  }) => Promise<void>;
  onDeleteChannel: (channelId: string) => Promise<void>;
  onDeleteProvider: (providerId: string) => Promise<void>;
  onDeleteCredential: (
    tenantId: string,
    providerId: string,
    keyReference: string,
  ) => Promise<void>;
  onDeleteModel: (externalName: string, providerId: string) => Promise<void>;
};

type PendingDelete =
  | { kind: 'channel'; label: string; channelId: string }
  | { kind: 'provider'; label: string; providerId: string }
  | { kind: 'credential'; label: string; tenantId: string; providerId: string; keyReference: string }
  | { kind: 'model'; label: string; externalName: string; providerId: string }
  | null;

function credentialStorageLabel(credential: CredentialRecord): string {
  if (credential.secret_backend === 'local_encrypted_file') {
    return credential.secret_local_file ?? 'local encrypted file';
  }

  if (credential.secret_backend === 'os_keyring') {
    return credential.secret_keyring_service ?? 'os keyring';
  }

  return 'database envelope';
}

export function CatalogPage({
  snapshot,
  onSaveChannel,
  onSaveProvider,
  onSaveCredential,
  onSaveModel,
  onDeleteChannel,
  onDeleteProvider,
  onDeleteCredential,
  onDeleteModel,
}: CatalogPageProps) {
  const [channelDraft, setChannelDraft] = useState({
    id: snapshot.channels[0]?.id ?? 'openai',
    name: snapshot.channels[0]?.name ?? 'OpenAI',
  });
  const [providerDraft, setProviderDraft] = useState({
    id: snapshot.providers[0]?.id ?? 'provider-openai-official',
    channel_id: snapshot.providers[0]?.channel_id ?? snapshot.channels[0]?.id ?? 'openai',
    display_name: snapshot.providers[0]?.display_name ?? 'OpenAI Official',
    adapter_kind: snapshot.providers[0]?.adapter_kind ?? 'openai',
    base_url: snapshot.providers[0]?.base_url ?? 'https://api.openai.com',
    extension_id: snapshot.providers[0]?.extension_id ?? 'sdkwork.provider.openai.official',
  });
  const [credentialDraft, setCredentialDraft] = useState({
    tenant_id: snapshot.credentials[0]?.tenant_id ?? snapshot.tenants[0]?.id ?? 'tenant-local',
    provider_id:
      snapshot.credentials[0]?.provider_id
      ?? snapshot.providers[0]?.id
      ?? 'provider-openai-official',
    key_reference: snapshot.credentials[0]?.key_reference ?? 'cred-openai-primary',
    secret_value: '',
  });
  const [modelDraft, setModelDraft] = useState({
    external_name: snapshot.models[0]?.external_name ?? 'gpt-4.1',
    provider_id:
      snapshot.models[0]?.provider_id ?? snapshot.providers[0]?.id ?? 'provider-openai-official',
    capabilities: snapshot.models[0]?.capabilities.join(', ') ?? 'responses, chat_completions',
    streaming: snapshot.models[0]?.streaming ?? true,
    context_window: String(snapshot.models[0]?.context_window ?? 128000),
  });
  const [isChannelDialogOpen, setIsChannelDialogOpen] = useState(false);
  const [isProviderDialogOpen, setIsProviderDialogOpen] = useState(false);
  const [isCredentialDialogOpen, setIsCredentialDialogOpen] = useState(false);
  const [isModelDialogOpen, setIsModelDialogOpen] = useState(false);
  const [pendingDelete, setPendingDelete] = useState<PendingDelete>(null);

  const selectedProvider = snapshot.providers.find((provider) => provider.id === modelDraft.provider_id);
  const selectedCredential = snapshot.credentials.find(
    (credential) => (
      credential.tenant_id === credentialDraft.tenant_id
      && credential.provider_id === credentialDraft.provider_id
      && credential.key_reference === credentialDraft.key_reference
    ),
  );
  const channelsWithProviders = new Set(
    snapshot.providers.flatMap((provider) => [
      provider.channel_id,
      ...provider.channel_bindings.map((binding) => binding.channel_id),
    ]),
  );
  const providersWithModels = new Set(snapshot.models.map((model) => model.provider_id));
  const providersWithCredentials = new Set(
    snapshot.credentials.map((credential) => credential.provider_id),
  );
  const providersWithoutCredentials = snapshot.providers.filter(
    (provider) => !providersWithCredentials.has(provider.id),
  );
  const orphanCredentials = snapshot.credentials.filter(
    (credential) => !snapshot.providers.some((provider) => provider.id === credential.provider_id),
  );

  async function handleChannel(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    await onSaveChannel(channelDraft);
    setIsChannelDialogOpen(false);
  }

  async function handleProvider(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    await onSaveProvider({
      id: providerDraft.id,
      channel_id: providerDraft.channel_id,
      extension_id: providerDraft.extension_id || undefined,
      adapter_kind: providerDraft.adapter_kind,
      base_url: providerDraft.base_url,
      display_name: providerDraft.display_name,
      channel_bindings: [{ channel_id: providerDraft.channel_id, is_primary: true }],
    });
    setIsProviderDialogOpen(false);
  }

  async function handleCredential(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    await onSaveCredential(credentialDraft);
    setCredentialDraft((current) => ({ ...current, secret_value: '' }));
    setIsCredentialDialogOpen(false);
  }

  async function handleModel(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    await onSaveModel({
      external_name: modelDraft.external_name,
      provider_id: modelDraft.provider_id,
      capabilities: modelDraft.capabilities.split(',').map((value) => value.trim()).filter(Boolean),
      streaming: modelDraft.streaming,
      context_window: Number(modelDraft.context_window),
    });
    setIsModelDialogOpen(false);
  }

  async function confirmDelete() {
    if (!pendingDelete) {
      return;
    }

    if (pendingDelete.kind === 'channel') {
      await onDeleteChannel(pendingDelete.channelId);
    }

    if (pendingDelete.kind === 'provider') {
      await onDeleteProvider(pendingDelete.providerId);
    }

    if (pendingDelete.kind === 'credential') {
      await onDeleteCredential(
        pendingDelete.tenantId,
        pendingDelete.providerId,
        pendingDelete.keyReference,
      );
    }

    if (pendingDelete.kind === 'model') {
      await onDeleteModel(pendingDelete.externalName, pendingDelete.providerId);
    }

    setPendingDelete(null);
  }

  return (
    <div className="adminx-page-grid">
      <section className="adminx-stat-grid">
        <StatCard label="Channels" value={String(snapshot.channels.length)} detail="Adapter or protocol surfaces exposed to the router." />
        <StatCard label="Providers" value={String(snapshot.providers.length)} detail="Proxy provider records and base URL definitions." />
        <StatCard label="Credentials" value={String(snapshot.credentials.length)} detail="Upstream secret references tracked by the control plane." />
        <StatCard label="Models" value={String(snapshot.models.length)} detail="Published model entries currently available for routing." />
      </section>

      <PageToolbar
        title="Catalog workbench"
        detail="Review the registries first, then open a dedicated dialog only when you need to create or edit configuration."
        actions={(
          <>
            <Dialog open={isChannelDialogOpen} onOpenChange={setIsChannelDialogOpen}>
              <DialogTrigger asChild>
                <InlineButton tone="primary" onClick={() => setIsChannelDialogOpen(true)}>New channel</InlineButton>
              </DialogTrigger>
              <DialogContent size="medium">
                <AdminDialog title="New channel" detail="Channels define the surface that providers attach to.">
                  <form className="adminx-form-grid" onSubmit={(event) => void handleChannel(event)}>
                    <FormField label="Channel id"><input value={channelDraft.id} onChange={(event) => setChannelDraft((current) => ({ ...current, id: event.target.value }))} required /></FormField>
                    <FormField label="Channel name"><input value={channelDraft.name} onChange={(event) => setChannelDraft((current) => ({ ...current, name: event.target.value }))} required /></FormField>
                    <DialogFooter>
                      <InlineButton onClick={() => setIsChannelDialogOpen(false)}>Cancel</InlineButton>
                      <InlineButton tone="primary" type="submit">Save channel</InlineButton>
                    </DialogFooter>
                  </form>
                </AdminDialog>
              </DialogContent>
            </Dialog>

            <Dialog open={isProviderDialogOpen} onOpenChange={setIsProviderDialogOpen}>
              <DialogTrigger asChild>
                <InlineButton onClick={() => setIsProviderDialogOpen(true)}>New provider</InlineButton>
              </DialogTrigger>
              <DialogContent size="large">
                <AdminDialog title="New provider" detail="Providers hold adapter metadata, extension identity, and upstream endpoint details.">
                  <form className="adminx-form-grid" onSubmit={(event) => void handleProvider(event)}>
                    <FormField label="Provider id"><input value={providerDraft.id} onChange={(event) => setProviderDraft((current) => ({ ...current, id: event.target.value }))} required /></FormField>
                    <FormField label="Channel id">
                      {snapshot.channels.length ? (
                        <select value={providerDraft.channel_id} onChange={(event) => setProviderDraft((current) => ({ ...current, channel_id: event.target.value }))}>
                          {snapshot.channels.map((channel) => (
                            <option key={channel.id} value={channel.id}>{channel.name} ({channel.id})</option>
                          ))}
                        </select>
                      ) : (
                        <input value={providerDraft.channel_id} onChange={(event) => setProviderDraft((current) => ({ ...current, channel_id: event.target.value }))} required />
                      )}
                    </FormField>
                    <FormField label="Display name"><input value={providerDraft.display_name} onChange={(event) => setProviderDraft((current) => ({ ...current, display_name: event.target.value }))} required /></FormField>
                    <FormField label="Adapter kind"><input value={providerDraft.adapter_kind} onChange={(event) => setProviderDraft((current) => ({ ...current, adapter_kind: event.target.value }))} required /></FormField>
                    <FormField label="Base URL"><input value={providerDraft.base_url} onChange={(event) => setProviderDraft((current) => ({ ...current, base_url: event.target.value }))} required /></FormField>
                    <FormField label="Extension id"><input value={providerDraft.extension_id} onChange={(event) => setProviderDraft((current) => ({ ...current, extension_id: event.target.value }))} /></FormField>
                    <DialogFooter>
                      <InlineButton onClick={() => setIsProviderDialogOpen(false)}>Cancel</InlineButton>
                      <InlineButton tone="primary" type="submit">Save provider</InlineButton>
                    </DialogFooter>
                  </form>
                </AdminDialog>
              </DialogContent>
            </Dialog>

            <Dialog open={isCredentialDialogOpen} onOpenChange={setIsCredentialDialogOpen}>
              <DialogTrigger asChild>
                <InlineButton onClick={() => setIsCredentialDialogOpen(true)}>Rotate credential</InlineButton>
              </DialogTrigger>
              <DialogContent size="large">
                <AdminDialog title="Rotate credential" detail="Secrets remain write-only. Use this flow to add or rotate upstream credentials without crowding the inventory view.">
                  <form className="adminx-form-grid" onSubmit={(event) => void handleCredential(event)}>
                    <FormField label="Tenant">
                      {snapshot.tenants.length ? (
                        <select value={credentialDraft.tenant_id} onChange={(event) => setCredentialDraft((current) => ({ ...current, tenant_id: event.target.value }))}>
                          {snapshot.tenants.map((tenant) => (
                            <option key={tenant.id} value={tenant.id}>{tenant.name} ({tenant.id})</option>
                          ))}
                        </select>
                      ) : (
                        <input value={credentialDraft.tenant_id} onChange={(event) => setCredentialDraft((current) => ({ ...current, tenant_id: event.target.value }))} required />
                      )}
                    </FormField>
                    <FormField label="Provider">
                      {snapshot.providers.length ? (
                        <select value={credentialDraft.provider_id} onChange={(event) => setCredentialDraft((current) => ({ ...current, provider_id: event.target.value }))}>
                          {snapshot.providers.map((provider) => (
                            <option key={provider.id} value={provider.id}>{provider.display_name} ({provider.id})</option>
                          ))}
                        </select>
                      ) : (
                        <input value={credentialDraft.provider_id} onChange={(event) => setCredentialDraft((current) => ({ ...current, provider_id: event.target.value }))} required />
                      )}
                    </FormField>
                    <FormField label="Key reference"><input value={credentialDraft.key_reference} onChange={(event) => setCredentialDraft((current) => ({ ...current, key_reference: event.target.value }))} required /></FormField>
                    <FormField label="Secret value" hint="The cleartext secret is never returned by the admin API."><input value={credentialDraft.secret_value} onChange={(event) => setCredentialDraft((current) => ({ ...current, secret_value: event.target.value }))} type="password" required /></FormField>
                    <div className="adminx-note">
                      <strong>Credential posture</strong>
                      <p>
                        Backend: {selectedCredential?.secret_backend ?? 'server-managed default'}
                        {' | '}
                        Storage: {selectedCredential ? credentialStorageLabel(selectedCredential) : 'selected by the control plane'}
                      </p>
                    </div>
                    <DialogFooter>
                      <InlineButton onClick={() => setIsCredentialDialogOpen(false)}>Cancel</InlineButton>
                      <InlineButton tone="primary" type="submit">Save credential</InlineButton>
                    </DialogFooter>
                  </form>
                </AdminDialog>
              </DialogContent>
            </Dialog>

            <Dialog open={isModelDialogOpen} onOpenChange={setIsModelDialogOpen}>
              <DialogTrigger asChild>
                <InlineButton onClick={() => setIsModelDialogOpen(true)}>New model</InlineButton>
              </DialogTrigger>
              <DialogContent size="large">
                <AdminDialog title="New model" detail="Models stay close to the routing mesh, but editing belongs in a dedicated dialog.">
                  <form className="adminx-form-grid" onSubmit={(event) => void handleModel(event)}>
                    <FormField label="Model name"><input value={modelDraft.external_name} onChange={(event) => setModelDraft((current) => ({ ...current, external_name: event.target.value }))} required /></FormField>
                    <FormField label="Provider">
                      {snapshot.providers.length ? (
                        <select value={modelDraft.provider_id} onChange={(event) => setModelDraft((current) => ({ ...current, provider_id: event.target.value }))}>
                          {snapshot.providers.map((provider) => (
                            <option key={provider.id} value={provider.id}>{provider.display_name} ({provider.id})</option>
                          ))}
                        </select>
                      ) : (
                        <input value={modelDraft.provider_id} onChange={(event) => setModelDraft((current) => ({ ...current, provider_id: event.target.value }))} required />
                      )}
                    </FormField>
                    <FormField label="Capabilities"><input value={modelDraft.capabilities} onChange={(event) => setModelDraft((current) => ({ ...current, capabilities: event.target.value }))} required /></FormField>
                    <FormField label="Context window"><input value={modelDraft.context_window} onChange={(event) => setModelDraft((current) => ({ ...current, context_window: event.target.value }))} type="number" /></FormField>
                    <FormField label="Streaming">
                      <select value={modelDraft.streaming ? 'true' : 'false'} onChange={(event) => setModelDraft((current) => ({ ...current, streaming: event.target.value === 'true' }))}>
                        <option value="true">Enabled</option>
                        <option value="false">Disabled</option>
                      </select>
                    </FormField>
                    <div className="adminx-note">
                      <strong>Selected provider posture</strong>
                      <p>
                        Channel: {selectedProvider?.channel_id ?? '-'}
                        {' | '}
                        Base URL: {selectedProvider?.base_url ?? '-'}
                      </p>
                    </div>
                    <DialogFooter>
                      <InlineButton onClick={() => setIsModelDialogOpen(false)}>Cancel</InlineButton>
                      <InlineButton tone="primary" type="submit">Save model</InlineButton>
                    </DialogFooter>
                  </form>
                </AdminDialog>
              </DialogContent>
            </Dialog>
          </>
        )}
      >
        <div className="adminx-form-grid">
          <div className="adminx-note">
            <strong>Dependency order</strong>
            <p>Channels feed providers, providers feed models, and credentials should be rotated before live traffic is expanded.</p>
          </div>
          <div className="adminx-note">
            <strong>Delete posture</strong>
            <p>Delete actions stay behind confirmation so downstream impact is explicit before any registry record is removed.</p>
          </div>
        </div>
      </PageToolbar>

      <Surface title="Coverage posture" detail="Quickly see which providers are ready for live upstream traffic.">
        <div className="adminx-card-grid">
          <article className="adminx-mini-card">
            <div className="adminx-row"><strong>Covered providers</strong><Pill tone="live">{providersWithCredentials.size}</Pill></div>
            <p>Providers with at least one credential record.</p>
          </article>
          <article className="adminx-mini-card">
            <div className="adminx-row"><strong>Missing coverage</strong><Pill tone={providersWithoutCredentials.length ? 'danger' : 'default'}>{providersWithoutCredentials.length}</Pill></div>
            <p>{providersWithoutCredentials.length ? providersWithoutCredentials.map((provider) => provider.display_name).join(', ') : 'All providers currently have credential coverage.'}</p>
          </article>
          <article className="adminx-mini-card">
            <div className="adminx-row"><strong>Orphan credentials</strong><Pill tone={orphanCredentials.length ? 'danger' : 'default'}>{orphanCredentials.length}</Pill></div>
            <p>{orphanCredentials.length ? 'Credential rows exist for providers that are no longer present in the registry.' : 'No orphaned credential rows detected.'}</p>
          </article>
        </div>
      </Surface>

      <Surface title="Channel registry" detail="Live channel catalog from the admin API.">
        <DataTable
          columns={[
            { key: 'id', label: 'Channel id', render: (channel) => <strong>{channel.id}</strong> },
            { key: 'name', label: 'Name', render: (channel) => channel.name },
            {
              key: 'actions',
              label: 'Actions',
              render: (channel) => (
                <div className="adminx-row">
                  <InlineButton onClick={() => { setChannelDraft({ id: channel.id, name: channel.name }); setIsChannelDialogOpen(true); }}>Edit channel</InlineButton>
                  <InlineButton tone="danger" disabled={channelsWithProviders.has(channel.id)} onClick={() => setPendingDelete({ kind: 'channel', label: channel.name, channelId: channel.id })}>Delete</InlineButton>
                </div>
              ),
            },
          ]}
          rows={snapshot.channels}
          empty="No channels available."
          getKey={(channel) => channel.id}
        />
      </Surface>

      <Surface title="Provider registry" detail="Provider records, bound channels, and credential coverage.">
        <DataTable
          columns={[
            { key: 'id', label: 'Provider id', render: (provider) => <strong>{provider.id}</strong> },
            { key: 'channel', label: 'Channel', render: (provider) => provider.channel_id },
            { key: 'display', label: 'Display', render: (provider) => provider.display_name },
            { key: 'base', label: 'Base URL', render: (provider) => provider.base_url },
            { key: 'credentials', label: 'Credentials', render: (provider) => <Pill tone={providersWithCredentials.has(provider.id) ? 'live' : 'danger'}>{snapshot.credentials.filter((credential) => credential.provider_id === provider.id).length}</Pill> },
            {
              key: 'actions',
              label: 'Actions',
              render: (provider) => (
                <div className="adminx-row">
                  <InlineButton onClick={() => { setProviderDraft({ id: provider.id, channel_id: provider.channel_id, display_name: provider.display_name, adapter_kind: provider.adapter_kind, base_url: provider.base_url, extension_id: provider.extension_id ?? '' }); setIsProviderDialogOpen(true); }}>Edit provider</InlineButton>
                  <InlineButton onClick={() => { setCredentialDraft((current) => ({ ...current, provider_id: provider.id, secret_value: '' })); setIsCredentialDialogOpen(true); }}>Rotate secret</InlineButton>
                  <InlineButton tone="danger" disabled={providersWithModels.has(provider.id)} onClick={() => setPendingDelete({ kind: 'provider', label: provider.display_name, providerId: provider.id })}>Delete</InlineButton>
                </div>
              ),
            },
          ]}
          rows={snapshot.providers}
          empty="No providers available."
          getKey={(provider) => provider.id}
        />
      </Surface>

      <Surface title="Credential inventory" detail="Write-only upstream secret inventory and backend placement metadata.">
        <DataTable
          columns={[
            { key: 'tenant', label: 'Tenant', render: (credential) => <strong>{credential.tenant_id}</strong> },
            { key: 'provider', label: 'Provider', render: (credential) => credential.provider_id },
            { key: 'reference', label: 'Key reference', render: (credential) => credential.key_reference },
            { key: 'backend', label: 'Backend', render: (credential) => <Pill tone={credential.secret_backend === 'database_encrypted' ? 'live' : 'default'}>{credential.secret_backend}</Pill> },
            { key: 'storage', label: 'Storage', render: (credential) => credentialStorageLabel(credential) },
            {
              key: 'actions',
              label: 'Actions',
              render: (credential) => (
                <div className="adminx-row">
                  <InlineButton onClick={() => { setCredentialDraft({ tenant_id: credential.tenant_id, provider_id: credential.provider_id, key_reference: credential.key_reference, secret_value: '' }); setIsCredentialDialogOpen(true); }}>Rotate secret</InlineButton>
                  <InlineButton tone="danger" onClick={() => setPendingDelete({ kind: 'credential', label: credential.key_reference, tenantId: credential.tenant_id, providerId: credential.provider_id, keyReference: credential.key_reference })}>Delete</InlineButton>
                </div>
              ),
            },
          ]}
          rows={snapshot.credentials}
          empty="No provider credentials available."
          getKey={(credential) => `${credential.tenant_id}:${credential.provider_id}:${credential.key_reference}`}
        />
      </Surface>

      <Surface title="Model registry" detail="Live model catalog used by the routing layer.">
        <DataTable
          columns={[
            { key: 'name', label: 'Model', render: (model) => <strong>{model.external_name}</strong> },
            { key: 'provider', label: 'Provider', render: (model) => model.provider_id },
            { key: 'caps', label: 'Capabilities', render: (model) => model.capabilities.join(', ') || '-' },
            { key: 'streaming', label: 'Streaming', render: (model) => String(model.streaming) },
            {
              key: 'actions',
              label: 'Actions',
              render: (model) => (
                <div className="adminx-row">
                  <InlineButton onClick={() => { setModelDraft({ external_name: model.external_name, provider_id: model.provider_id, capabilities: model.capabilities.join(', '), streaming: model.streaming, context_window: String(model.context_window ?? '') }); setIsModelDialogOpen(true); }}>Edit model</InlineButton>
                  <InlineButton tone="danger" onClick={() => setPendingDelete({ kind: 'model', label: model.external_name, externalName: model.external_name, providerId: model.provider_id })}>Delete</InlineButton>
                </div>
              ),
            },
          ]}
          rows={snapshot.models}
          empty="No models available."
          getKey={(model) => `${model.external_name}:${model.provider_id}`}
        />
      </Surface>

      <ConfirmDialog
        open={Boolean(pendingDelete)}
        title={
          pendingDelete?.kind === 'channel'
            ? 'Delete channel'
            : pendingDelete?.kind === 'provider'
              ? 'Delete provider'
              : pendingDelete?.kind === 'credential'
                ? 'Delete credential'
                : 'Delete model'
        }
        detail={
          pendingDelete?.kind === 'channel'
            ? `Delete ${pendingDelete.label}. Remove dependent providers first so the routing mesh stays coherent.`
            : pendingDelete?.kind === 'provider'
              ? `Delete ${pendingDelete.label}. Remove dependent models before retiring the provider record.`
              : pendingDelete?.kind === 'credential'
                ? `Delete credential ${pendingDelete.label}. This removes the stored provider secret reference from the control plane.`
                : pendingDelete?.kind === 'model'
                  ? `Delete model ${pendingDelete.label}. This removes the routeable catalog entry for the selected provider.`
                  : ''
        }
        confirmLabel="Delete now"
        onClose={() => setPendingDelete(null)}
        onConfirm={confirmDelete}
      />
    </div>
  );
}
