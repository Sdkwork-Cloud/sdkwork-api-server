import { useState } from 'react';
import type { FormEvent, ReactNode } from 'react';

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
  ToolbarField,
  ToolbarInline,
  ToolbarSearchField,
} from 'sdkwork-router-admin-commons';
import type {
  AdminPageProps,
  ChannelModelRecord,
  CredentialRecord,
  ModelPriceRecord,
  ProxyProviderRecord,
} from 'sdkwork-router-admin-types';

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
  onSaveChannelModel: (input: {
    channel_id: string;
    model_id: string;
    model_display_name: string;
    capabilities: string[];
    streaming: boolean;
    context_window?: number | null;
    description?: string;
  }) => Promise<void>;
  onSaveModelPrice: (input: {
    channel_id: string;
    model_id: string;
    proxy_provider_id: string;
    currency_code: string;
    price_unit: string;
    input_price: number;
    output_price: number;
    cache_read_price: number;
    cache_write_price: number;
    request_price: number;
    is_active: boolean;
  }) => Promise<void>;
  onDeleteChannel: (channelId: string) => Promise<void>;
  onDeleteProvider: (providerId: string) => Promise<void>;
  onDeleteCredential: (
    tenantId: string,
    providerId: string,
    keyReference: string,
  ) => Promise<void>;
  onDeleteModel: (externalName: string, providerId: string) => Promise<void>;
  onDeleteChannelModel: (channelId: string, modelId: string) => Promise<void>;
  onDeleteModelPrice: (
    channelId: string,
    modelId: string,
    proxyProviderId: string,
  ) => Promise<void>;
};

type PendingDelete =
  | { kind: 'channel'; label: string; channelId: string }
  | { kind: 'provider'; label: string; providerId: string }
  | { kind: 'credential'; label: string; tenantId: string; providerId: string; keyReference: string }
  | { kind: 'channel-model'; label: string; channelId: string; modelId: string }
  | {
      kind: 'model-price';
      label: string;
      channelId: string;
      modelId: string;
      proxyProviderId: string;
    }
  | { kind: 'model'; label: string; externalName: string; providerId: string }
  | null;

type ChannelSeedModelDraft = {
  draft_id: string;
  model_id: string;
  model_display_name: string;
  capabilities: string;
  streaming: boolean;
  context_window: string;
  description: string;
};

type ProviderDraft = {
  id: string;
  primary_channel_id: string;
  display_name: string;
  adapter_kind: string;
  base_url: string;
  extension_id: string;
  bound_channel_ids: string[];
};

type CredentialDraft = {
  tenant_id: string;
  provider_id: string;
  key_reference: string;
  secret_value: string;
};

type ChannelModelDraft = {
  channel_id: string;
  model_id: string;
  model_display_name: string;
  capabilities: string;
  streaming: boolean;
  context_window: string;
  description: string;
};

type ModelPriceDraft = {
  channel_id: string;
  model_id: string;
  proxy_provider_id: string;
  currency_code: string;
  price_unit: string;
  input_price: string;
  output_price: string;
  cache_read_price: string;
  cache_write_price: string;
  request_price: string;
  is_active: boolean;
};

type CatalogLane = 'channels' | 'providers' | 'credentials' | 'variants';

type CatalogWorkbenchRow =
  | (AdminPageProps['snapshot']['channels'][number] & { kind: 'channel' })
  | (ProxyProviderRecord & { kind: 'provider' })
  | (CredentialRecord & { kind: 'credential' })
  | (AdminPageProps['snapshot']['models'][number] & { kind: 'variant' });

const PRICE_UNIT_OPTIONS: Array<{ value: string; label: string; detail: string }> = [
  {
    value: 'per_1m_tokens',
    label: 'Million tokens',
    detail: 'Default large-model billing unit for most LLM providers.',
  },
  {
    value: 'per_1k_tokens',
    label: 'Thousand tokens',
    detail: 'Useful when an upstream provider publishes smaller token-rate ladders.',
  },
  {
    value: 'per_request',
    label: 'Request',
    detail: 'Use when the upstream API charges a flat amount for each call.',
  },
  {
    value: 'per_image',
    label: 'Image generated',
    detail: 'Use for image generation, edits, and visual asset APIs.',
  },
  {
    value: 'per_second_audio',
    label: 'Audio second',
    detail: 'Use for speech, audio transcription, or realtime audio billing.',
  },
  {
    value: 'per_minute_video',
    label: 'Video minute',
    detail: 'Use for video generation, processing, or hosted streaming workloads.',
  },
  {
    value: 'per_track',
    label: 'Music track',
    detail: 'Use for music generation or composition-style APIs.',
  },
];

function createDraftId(): string {
  return `${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;
}

function splitCapabilities(value: string): string[] {
  return value
    .split(',')
    .map((item) => item.trim())
    .filter(Boolean);
}

function parseOptionalNumber(value: string): number | null {
  const trimmed = value.trim();
  if (!trimmed) {
    return null;
  }

  const parsed = Number(trimmed);
  return Number.isFinite(parsed) ? parsed : null;
}

function parseRequiredNumber(value: string): number {
  const parsed = Number(value.trim() || '0');
  return Number.isFinite(parsed) ? parsed : 0;
}

function priceUnitLabel(value: string): string {
  return PRICE_UNIT_OPTIONS.find((option) => option.value === value)?.label ?? value;
}

function priceUnitDetail(value: string): string {
  return PRICE_UNIT_OPTIONS.find((option) => option.value === value)?.detail ?? value;
}

function credentialStorageLabel(credential: CredentialRecord): string {
  if (credential.secret_backend === 'local_encrypted_file') {
    return credential.secret_local_file ?? 'local encrypted file';
  }

  if (credential.secret_backend === 'os_keyring') {
    return credential.secret_keyring_service ?? 'os keyring';
  }

  return 'database envelope';
}

function providerChannelIds(provider: ProxyProviderRecord): string[] {
  const ids = new Set<string>();
  ids.add(provider.channel_id);

  for (const binding of provider.channel_bindings) {
    ids.add(binding.channel_id);
  }

  return Array.from(ids);
}

function emptySeedModelDraft(): ChannelSeedModelDraft {
  return {
    draft_id: createDraftId(),
    model_id: '',
    model_display_name: '',
    capabilities: 'chat',
    streaming: true,
    context_window: '',
    description: '',
  };
}

function seedModelDraftFromRecord(record: ChannelModelRecord): ChannelSeedModelDraft {
  return {
    draft_id: createDraftId(),
    model_id: record.model_id,
    model_display_name: record.model_display_name,
    capabilities: record.capabilities.join(', '),
    streaming: record.streaming,
    context_window: String(record.context_window ?? ''),
    description: record.description ?? '',
  };
}

function emptyProviderDraft(channelId?: string): ProviderDraft {
  return {
    id: '',
    primary_channel_id: channelId ?? '',
    display_name: '',
    adapter_kind: 'openai',
    base_url: '',
    extension_id: '',
    bound_channel_ids: channelId ? [channelId] : [],
  };
}

function providerDraftFromRecord(record: ProxyProviderRecord): ProviderDraft {
  return {
    id: record.id,
    primary_channel_id: record.channel_id,
    display_name: record.display_name,
    adapter_kind: record.adapter_kind,
    base_url: record.base_url,
    extension_id: record.extension_id ?? '',
    bound_channel_ids: providerChannelIds(record),
  };
}

function emptyCredentialDraft(tenantId?: string, providerId?: string): CredentialDraft {
  return {
    tenant_id: tenantId ?? '',
    provider_id: providerId ?? '',
    key_reference: '',
    secret_value: '',
  };
}

function credentialDraftFromRecord(record: CredentialRecord): CredentialDraft {
  return {
    tenant_id: record.tenant_id,
    provider_id: record.provider_id,
    key_reference: record.key_reference,
    secret_value: '',
  };
}

function emptyChannelModelDraft(channelId: string): ChannelModelDraft {
  return {
    channel_id: channelId,
    model_id: '',
    model_display_name: '',
    capabilities: 'chat',
    streaming: true,
    context_window: '',
    description: '',
  };
}

function channelModelDraftFromRecord(record: ChannelModelRecord): ChannelModelDraft {
  return {
    channel_id: record.channel_id,
    model_id: record.model_id,
    model_display_name: record.model_display_name,
    capabilities: record.capabilities.join(', '),
    streaming: record.streaming,
    context_window: String(record.context_window ?? ''),
    description: record.description ?? '',
  };
}

function emptyModelPriceDraft(channelId: string, modelId: string): ModelPriceDraft {
  return {
    channel_id: channelId,
    model_id: modelId,
    proxy_provider_id: '',
    currency_code: 'USD',
    price_unit: 'per_1m_tokens',
    input_price: '0',
    output_price: '0',
    cache_read_price: '0',
    cache_write_price: '0',
    request_price: '0',
    is_active: true,
  };
}

function modelPriceDraftFromRecord(record: ModelPriceRecord): ModelPriceDraft {
  return {
    channel_id: record.channel_id,
    model_id: record.model_id,
    proxy_provider_id: record.proxy_provider_id,
    currency_code: record.currency_code,
    price_unit: record.price_unit,
    input_price: String(record.input_price),
    output_price: String(record.output_price),
    cache_read_price: String(record.cache_read_price),
    cache_write_price: String(record.cache_write_price),
    request_price: String(record.request_price),
    is_active: record.is_active,
  };
}

export function CatalogPage({
  snapshot,
  onSaveChannel,
  onSaveProvider,
  onSaveCredential,
  onSaveChannelModel,
  onSaveModelPrice,
  onDeleteChannel,
  onDeleteProvider,
  onDeleteCredential,
  onDeleteModel,
  onDeleteChannelModel,
  onDeleteModelPrice,
}: CatalogPageProps) {
  const defaultChannelId = snapshot.channels[0]?.id ?? 'openai';
  const defaultProviderId = snapshot.providers[0]?.id ?? '';
  const defaultTenantId = snapshot.tenants[0]?.id ?? 'tenant-local';
  const [channelDraft, setChannelDraft] = useState({
    id: defaultChannelId,
    name: snapshot.channels[0]?.name ?? 'OpenAI',
  });
  const [channelSeedModels, setChannelSeedModels] = useState<ChannelSeedModelDraft[]>([
    emptySeedModelDraft(),
  ]);
  const [editingChannelId, setEditingChannelId] = useState<string | null>(null);
  const [providerDraft, setProviderDraft] = useState<ProviderDraft>(
    emptyProviderDraft(defaultChannelId),
  );
  const [editingProviderId, setEditingProviderId] = useState<string | null>(null);
  const [credentialDraft, setCredentialDraft] = useState<CredentialDraft>(
    emptyCredentialDraft(defaultTenantId, defaultProviderId),
  );
  const [channelModelDraft, setChannelModelDraft] = useState<ChannelModelDraft>(
    emptyChannelModelDraft(defaultChannelId),
  );
  const [editingChannelModelKey, setEditingChannelModelKey] = useState<string | null>(null);
  const [modelPriceDraft, setModelPriceDraft] = useState<ModelPriceDraft>(
    emptyModelPriceDraft(defaultChannelId, ''),
  );
  const [editingModelPriceKey, setEditingModelPriceKey] = useState<string | null>(null);
  const [activeChannelId, setActiveChannelId] = useState<string>(defaultChannelId);
  const [catalogLane, setCatalogLane] = useState<CatalogLane>('channels');
  const [search, setSearch] = useState('');
  const [pricingTarget, setPricingTarget] = useState<{
    channel_id: string;
    model_id: string;
    model_display_name: string;
  } | null>(null);
  const [isChannelDialogOpen, setIsChannelDialogOpen] = useState(false);
  const [isProviderDialogOpen, setIsProviderDialogOpen] = useState(false);
  const [isCredentialDialogOpen, setIsCredentialDialogOpen] = useState(false);
  const [isChannelModelsDialogOpen, setIsChannelModelsDialogOpen] = useState(false);
  const [isChannelModelEditorOpen, setIsChannelModelEditorOpen] = useState(false);
  const [isPricingDialogOpen, setIsPricingDialogOpen] = useState(false);
  const [isModelPriceEditorOpen, setIsModelPriceEditorOpen] = useState(false);
  const [pendingDelete, setPendingDelete] = useState<PendingDelete>(null);

  const selectedChannel =
    snapshot.channels.find((channel) => channel.id === activeChannelId) ?? null;
  const selectedChannelModels = snapshot.channelModels.filter(
    (model) => model.channel_id === activeChannelId,
  );
  const selectedModelPrices = pricingTarget
    ? snapshot.modelPrices.filter(
        (record) =>
          record.channel_id === pricingTarget.channel_id &&
          record.model_id === pricingTarget.model_id,
      )
    : [];
  const channelsWithProviders = new Set(
    snapshot.providers.flatMap((provider) => providerChannelIds(provider)),
  );
  const channelsWithModels = new Set(
    snapshot.channelModels.map((model) => model.channel_id),
  );
  const providerNameById = new Map(
    snapshot.providers.map((provider) => [provider.id, provider.display_name]),
  );
  const normalizedSearch = search.trim().toLowerCase();
  const filteredChannels = snapshot.channels.filter((channel) =>
    !normalizedSearch
    || [channel.id, channel.name].join(' ').toLowerCase().includes(normalizedSearch));
  const filteredProviders = snapshot.providers.filter((provider) =>
    !normalizedSearch
    || [
      provider.id,
      provider.display_name,
      provider.channel_id,
      provider.adapter_kind,
      provider.base_url,
      provider.extension_id ?? '',
      providerChannelIds(provider).join(' '),
    ].join(' ').toLowerCase().includes(normalizedSearch));
  const filteredCredentials = snapshot.credentials.filter((credential) =>
    !normalizedSearch
    || [
      credential.tenant_id,
      credential.provider_id,
      credential.key_reference,
      credential.secret_backend,
      credential.secret_local_file ?? '',
      credential.secret_keyring_service ?? '',
      credential.secret_master_key_id ?? '',
    ].join(' ').toLowerCase().includes(normalizedSearch));
  const filteredModels = snapshot.models.filter((model) =>
    !normalizedSearch
    || [
      model.external_name,
      model.provider_id,
      model.capabilities.join(' '),
    ].join(' ').toLowerCase().includes(normalizedSearch));
  const filteredSelectedChannelModels = selectedChannelModels.filter((model) =>
    !normalizedSearch
    || [
      model.channel_id,
      model.model_id,
      model.model_display_name ?? '',
      model.capabilities.join(' '),
    ].join(' ').toLowerCase().includes(normalizedSearch));
  const filteredSelectedModelPrices = selectedModelPrices.filter((record) =>
    !normalizedSearch
    || [
      record.channel_id,
      record.model_id,
      record.proxy_provider_id,
      record.currency_code,
      String(record.input_price),
      String(record.output_price),
    ].join(' ').toLowerCase().includes(normalizedSearch));
  const activeSelectedModelPrices = selectedModelPrices.filter((record) => record.is_active);
  const pricingUnitCoverage = Array.from(
    new Set(selectedModelPrices.map((record) => priceUnitLabel(record.price_unit))),
  );
  const activeCredentialCount = snapshot.credentials.length;
  const catalogLaneLabel =
    catalogLane === 'channels'
      ? 'Channels'
      : catalogLane === 'providers'
        ? 'Providers'
        : catalogLane === 'credentials'
          ? 'Credentials'
          : 'Variants';
  const catalogWorkbenchDetail =
    catalogLane === 'channels'
      ? 'Manage public channel surfaces and open channel-level model maintenance from one directory.'
      : catalogLane === 'providers'
        ? 'Review proxy provider bindings, adapters, and credential rotation coverage in one place.'
        : catalogLane === 'credentials'
          ? 'Track encrypted router secret coverage by tenant, provider, and storage backend.'
          : 'Inspect provider-scoped model variants before channel exposure and downstream pricing are attached.';
  let catalogRows: CatalogWorkbenchRow[] = filteredChannels.map((channel) => ({
    ...channel,
    kind: 'channel',
  }));
  let catalogEmpty = 'No channels available.';
  let catalogHealthLabel = `${filteredChannels.length} visible`;
  let catalogHealthTone: 'default' | 'live' | 'seed' | 'danger' = 'default';
  let catalogColumns: Array<{
    key: string;
    label: string;
    render: (row: CatalogWorkbenchRow) => ReactNode;
  }> = [
    {
      key: 'id',
      label: 'Channel id',
      render: (row) =>
        row.kind === 'channel' ? <strong>{row.id}</strong> : null,
    },
    {
      key: 'name',
      label: 'Channel name',
      render: (row) => (row.kind === 'channel' ? row.name : null),
    },
    {
      key: 'models',
      label: 'Models',
      render: (row) =>
        row.kind === 'channel' ? (
          <Pill tone={channelsWithModels.has(row.id) ? 'live' : 'default'}>
            {snapshot.channelModels.filter((model) => model.channel_id === row.id).length}
          </Pill>
        ) : null,
    },
    {
      key: 'providers',
      label: 'Providers',
      render: (row) =>
        row.kind === 'channel' ? (
          <Pill tone={channelsWithProviders.has(row.id) ? 'seed' : 'default'}>
            {snapshot.providers.filter((provider) => providerChannelIds(provider).includes(row.id)).length}
          </Pill>
        ) : null,
    },
    {
      key: 'actions',
      label: 'Actions',
      render: (row) =>
        row.kind === 'channel' ? (
          <div className="adminx-row">
            <InlineButton onClick={() => openEditChannelDialog(row.id)}>
              Edit channel
            </InlineButton>
            <InlineButton onClick={() => openChannelModelsDialog(row.id)}>
              Manage models
            </InlineButton>
            <InlineButton
              tone="danger"
              disabled={channelsWithProviders.has(row.id)}
              onClick={() =>
                setPendingDelete({
                  kind: 'channel',
                  label: `${row.name} (${row.id})`,
                  channelId: row.id,
                })
              }
            >
              Delete
            </InlineButton>
          </div>
        ) : null,
    },
  ];

  if (catalogLane === 'providers') {
    catalogRows = filteredProviders.map((provider) => ({
      ...provider,
      kind: 'provider',
    }));
    catalogEmpty = 'No proxy providers available.';
    catalogHealthLabel = `${filteredProviders.length} visible`;
    catalogHealthTone = filteredProviders.length ? 'seed' : 'default';
    catalogColumns = [
      {
        key: 'id',
        label: 'Provider id',
        render: (row) => (row.kind === 'provider' ? <strong>{row.id}</strong> : null),
      },
      {
        key: 'display_name',
        label: 'Display name',
        render: (row) => (row.kind === 'provider' ? row.display_name : null),
      },
      {
        key: 'primary_channel',
        label: 'Primary channel',
        render: (row) => (row.kind === 'provider' ? row.channel_id : null),
      },
      {
        key: 'bound_channels',
        label: 'Bound channels',
        render: (row) => (row.kind === 'provider' ? providerChannelIds(row).join(', ') : null),
      },
      {
        key: 'adapter_kind',
        label: 'Adapter',
        render: (row) => (row.kind === 'provider' ? row.adapter_kind : null),
      },
      {
        key: 'actions',
        label: 'Actions',
        render: (row) =>
          row.kind === 'provider' ? (
            <div className="adminx-row">
              <InlineButton onClick={() => openEditProviderDialog(row.id)}>
                Edit provider
              </InlineButton>
              <InlineButton onClick={() => openNewCredentialDialog(row.id)}>
                Rotate credential
              </InlineButton>
              <InlineButton
                tone="danger"
                onClick={() =>
                  setPendingDelete({
                    kind: 'provider',
                    label: `${row.display_name} (${row.id})`,
                    providerId: row.id,
                  })
                }
              >
                Delete
              </InlineButton>
            </div>
          ) : null,
      },
    ];
  } else if (catalogLane === 'credentials') {
    catalogRows = filteredCredentials.map((credential) => ({
      ...credential,
      kind: 'credential',
    }));
    catalogEmpty = 'No router credentials available.';
    catalogHealthLabel = `${filteredCredentials.length} visible`;
    catalogHealthTone = filteredCredentials.length ? 'live' : 'default';
    catalogColumns = [
      {
        key: 'tenant',
        label: 'Tenant',
        render: (row) =>
          row.kind === 'credential' ? <strong>{row.tenant_id}</strong> : null,
      },
      {
        key: 'provider',
        label: 'Proxy provider',
        render: (row) =>
          row.kind === 'credential'
            ? providerNameById.get(row.provider_id) ?? row.provider_id
            : null,
      },
      {
        key: 'reference',
        label: 'Key reference',
        render: (row) => (row.kind === 'credential' ? row.key_reference : null),
      },
      {
        key: 'backend',
        label: 'Backend',
        render: (row) => (row.kind === 'credential' ? row.secret_backend : null),
      },
      {
        key: 'storage',
        label: 'Storage',
        render: (row) =>
          row.kind === 'credential' ? credentialStorageLabel(row) : null,
      },
      {
        key: 'actions',
        label: 'Actions',
        render: (row) =>
          row.kind === 'credential' ? (
            <div className="adminx-row">
              <InlineButton onClick={() => openEditCredentialDialog(row)}>
                Rotate credential
              </InlineButton>
              <InlineButton
                tone="danger"
                onClick={() =>
                  setPendingDelete({
                    kind: 'credential',
                    label: row.key_reference,
                    tenantId: row.tenant_id,
                    providerId: row.provider_id,
                    keyReference: row.key_reference,
                  })
                }
              >
                Delete
              </InlineButton>
            </div>
          ) : null,
      },
    ];
  } else if (catalogLane === 'variants') {
    catalogRows = filteredModels.map((model) => ({
      ...model,
      kind: 'variant',
    }));
    catalogEmpty = 'No provider-scoped model variants available.';
    catalogHealthLabel = `${filteredModels.length} visible`;
    catalogHealthTone = filteredModels.length ? 'seed' : 'default';
    catalogColumns = [
      {
        key: 'model',
        label: 'Model',
        render: (row) => (row.kind === 'variant' ? <strong>{row.external_name}</strong> : null),
      },
      {
        key: 'provider',
        label: 'Provider',
        render: (row) =>
          row.kind === 'variant'
            ? providerNameById.get(row.provider_id) ?? row.provider_id
            : null,
      },
      {
        key: 'capabilities',
        label: 'Capabilities',
        render: (row) =>
          row.kind === 'variant' ? row.capabilities.join(', ') || '-' : null,
      },
      {
        key: 'streaming',
        label: 'Streaming',
        render: (row) => (row.kind === 'variant' ? String(row.streaming) : null),
      },
      {
        key: 'actions',
        label: 'Actions',
        render: (row) =>
          row.kind === 'variant' ? (
            <div className="adminx-row">
              <InlineButton
                tone="danger"
                onClick={() =>
                  setPendingDelete({
                    kind: 'model',
                    label: `${row.external_name} / ${row.provider_id}`,
                    externalName: row.external_name,
                    providerId: row.provider_id,
                  })
                }
              >
                Delete variant
              </InlineButton>
            </div>
          ) : null,
      },
    ];
  }

  function resetChannelDialog() {
    setEditingChannelId(null);
    setChannelDraft({
      id: snapshot.channels[0]?.id ?? 'openai',
      name: snapshot.channels[0]?.name ?? 'OpenAI',
    });
    setChannelSeedModels([emptySeedModelDraft()]);
    setIsChannelDialogOpen(false);
  }

  function openNewChannelDialog() {
    setEditingChannelId(null);
    setChannelDraft({ id: '', name: '' });
    setChannelSeedModels([emptySeedModelDraft()]);
    setIsChannelDialogOpen(true);
  }

  function openEditChannelDialog(channelId: string) {
    const channel = snapshot.channels.find((item) => item.id === channelId);
    if (!channel) {
      return;
    }

    const existingModels = snapshot.channelModels
      .filter((model) => model.channel_id === channelId)
      .map(seedModelDraftFromRecord);

    setEditingChannelId(channel.id);
    setChannelDraft({ id: channel.id, name: channel.name });
    setChannelSeedModels(existingModels.length ? existingModels : [emptySeedModelDraft()]);
    setIsChannelDialogOpen(true);
  }

  function resetProviderDialog() {
    setEditingProviderId(null);
    setProviderDraft(emptyProviderDraft(snapshot.channels[0]?.id ?? ''));
    setIsProviderDialogOpen(false);
  }

  function openNewProviderDialog(channelId?: string) {
    setEditingProviderId(null);
    setProviderDraft(emptyProviderDraft(channelId ?? snapshot.channels[0]?.id ?? ''));
    setIsProviderDialogOpen(true);
  }

  function openEditProviderDialog(providerId: string) {
    const provider = snapshot.providers.find((item) => item.id === providerId);
    if (!provider) {
      return;
    }

    setEditingProviderId(provider.id);
    setProviderDraft(providerDraftFromRecord(provider));
    setIsProviderDialogOpen(true);
  }

  function resetCredentialDialog() {
    setCredentialDraft(
      emptyCredentialDraft(
        snapshot.tenants[0]?.id ?? 'tenant-local',
        snapshot.providers[0]?.id ?? '',
      ),
    );
    setIsCredentialDialogOpen(false);
  }

  function openNewCredentialDialog(providerId?: string) {
    setCredentialDraft(
      emptyCredentialDraft(
        snapshot.tenants[0]?.id ?? 'tenant-local',
        providerId ?? snapshot.providers[0]?.id ?? '',
      ),
    );
    setIsCredentialDialogOpen(true);
  }

  function openEditCredentialDialog(record?: CredentialRecord) {
    if (!record) {
      openNewCredentialDialog();
      return;
    }

    setCredentialDraft(credentialDraftFromRecord(record));
    setIsCredentialDialogOpen(true);
  }

  function openChannelModelsDialog(channelId: string) {
    setActiveChannelId(channelId);
    setIsChannelModelsDialogOpen(true);
  }

  function resetChannelModelEditor() {
    setEditingChannelModelKey(null);
    setChannelModelDraft(
      emptyChannelModelDraft(activeChannelId || (snapshot.channels[0]?.id ?? '')),
    );
    setIsChannelModelEditorOpen(false);
  }

  function openNewChannelModelDialog(channelId: string) {
    setActiveChannelId(channelId);
    setEditingChannelModelKey(null);
    setChannelModelDraft(emptyChannelModelDraft(channelId));
    setIsChannelModelEditorOpen(true);
  }

  function openEditChannelModelDialog(record: ChannelModelRecord) {
    setActiveChannelId(record.channel_id);
    setEditingChannelModelKey(`${record.channel_id}:${record.model_id}`);
    setChannelModelDraft(channelModelDraftFromRecord(record));
    setIsChannelModelEditorOpen(true);
  }

  function resetModelPriceEditor() {
    setEditingModelPriceKey(null);
    setModelPriceDraft(
      emptyModelPriceDraft(
        pricingTarget?.channel_id ?? activeChannelId,
        pricingTarget?.model_id ?? '',
      ),
    );
    setIsModelPriceEditorOpen(false);
  }

  function openPricingDialog(record: ChannelModelRecord) {
    setActiveChannelId(record.channel_id);
    setPricingTarget({
      channel_id: record.channel_id,
      model_id: record.model_id,
      model_display_name: record.model_display_name,
    });
    setIsPricingDialogOpen(true);
  }

  function openNewModelPriceDialog() {
    if (!pricingTarget) {
      return;
    }

    setEditingModelPriceKey(null);
    setModelPriceDraft(
      emptyModelPriceDraft(pricingTarget.channel_id, pricingTarget.model_id),
    );
    setIsModelPriceEditorOpen(true);
  }

  function openEditModelPriceDialog(record: ModelPriceRecord) {
    setPricingTarget({
      channel_id: record.channel_id,
      model_id: record.model_id,
      model_display_name:
        snapshot.channelModels.find(
          (item) =>
            item.channel_id === record.channel_id && item.model_id === record.model_id,
        )?.model_display_name ?? record.model_id,
    });
    setEditingModelPriceKey(
      `${record.channel_id}:${record.model_id}:${record.proxy_provider_id}`,
    );
    setModelPriceDraft(modelPriceDraftFromRecord(record));
    setIsModelPriceEditorOpen(true);
  }

  async function handleChannelSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    await onSaveChannel(channelDraft);

    const channelId = channelDraft.id.trim();
    const existingModelIds = new Set(
      snapshot.channelModels
        .filter((model) => model.channel_id === channelId)
        .map((model) => model.model_id),
    );
    const nextModelIds = new Set<string>();

    for (const draft of channelSeedModels) {
      const modelId = draft.model_id.trim();
      const modelDisplayName = draft.model_display_name.trim();

      if (!modelId || !modelDisplayName) {
        continue;
      }

      nextModelIds.add(modelId);
      await onSaveChannelModel({
        channel_id: channelId,
        model_id: modelId,
        model_display_name: modelDisplayName,
        capabilities: splitCapabilities(draft.capabilities),
        streaming: draft.streaming,
        context_window: parseOptionalNumber(draft.context_window),
        description: draft.description.trim() || undefined,
      });
    }

    if (editingChannelId === channelId) {
      for (const existingModelId of existingModelIds) {
        if (!nextModelIds.has(existingModelId)) {
          await onDeleteChannelModel(channelId, existingModelId);
        }
      }
    }

    setActiveChannelId(channelId);
    resetChannelDialog();
  }

  async function handleProviderSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();

    const bindingIds = Array.from(
      new Set(
        [providerDraft.primary_channel_id, ...providerDraft.bound_channel_ids]
          .map((value) => value.trim())
          .filter(Boolean),
      ),
    );

    await onSaveProvider({
      id: providerDraft.id.trim(),
      channel_id: providerDraft.primary_channel_id.trim(),
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

  async function handleCredentialSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    await onSaveCredential({
      tenant_id: credentialDraft.tenant_id.trim(),
      provider_id: credentialDraft.provider_id.trim(),
      key_reference: credentialDraft.key_reference.trim(),
      secret_value: credentialDraft.secret_value,
    });
    resetCredentialDialog();
  }

  async function handleChannelModelSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    await onSaveChannelModel({
      channel_id: channelModelDraft.channel_id.trim(),
      model_id: channelModelDraft.model_id.trim(),
      model_display_name: channelModelDraft.model_display_name.trim(),
      capabilities: splitCapabilities(channelModelDraft.capabilities),
      streaming: channelModelDraft.streaming,
      context_window: parseOptionalNumber(channelModelDraft.context_window),
      description: channelModelDraft.description.trim() || undefined,
    });
    setActiveChannelId(channelModelDraft.channel_id);
    resetChannelModelEditor();
  }

  async function handleModelPriceSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    await onSaveModelPrice({
      channel_id: modelPriceDraft.channel_id.trim(),
      model_id: modelPriceDraft.model_id.trim(),
      proxy_provider_id: modelPriceDraft.proxy_provider_id.trim(),
      currency_code: modelPriceDraft.currency_code.trim(),
      price_unit: modelPriceDraft.price_unit.trim(),
      input_price: parseRequiredNumber(modelPriceDraft.input_price),
      output_price: parseRequiredNumber(modelPriceDraft.output_price),
      cache_read_price: parseRequiredNumber(modelPriceDraft.cache_read_price),
      cache_write_price: parseRequiredNumber(modelPriceDraft.cache_write_price),
      request_price: parseRequiredNumber(modelPriceDraft.request_price),
      is_active: modelPriceDraft.is_active,
    });
    resetModelPriceEditor();
  }

  async function confirmDelete() {
    if (!pendingDelete) {
      return;
    }

    if (pendingDelete.kind === 'channel') {
      await onDeleteChannel(pendingDelete.channelId);
    } else if (pendingDelete.kind === 'provider') {
      await onDeleteProvider(pendingDelete.providerId);
    } else if (pendingDelete.kind === 'credential') {
      await onDeleteCredential(
        pendingDelete.tenantId,
        pendingDelete.providerId,
        pendingDelete.keyReference,
      );
    } else if (pendingDelete.kind === 'channel-model') {
      await onDeleteChannelModel(pendingDelete.channelId, pendingDelete.modelId);
    } else if (pendingDelete.kind === 'model-price') {
      await onDeleteModelPrice(
        pendingDelete.channelId,
        pendingDelete.modelId,
        pendingDelete.proxyProviderId,
      );
    } else if (pendingDelete.kind === 'model') {
      await onDeleteModel(pendingDelete.externalName, pendingDelete.providerId);
    }

    setPendingDelete(null);
  }

  return (
    <div className="adminx-page-grid">
      <PageToolbar
        compact
        actions={
          <>
            <InlineButton tone="primary" onClick={openNewChannelDialog}>
              New channel
            </InlineButton>
            <InlineButton onClick={() => openNewProviderDialog()}>
              New provider
            </InlineButton>
            <InlineButton onClick={() => openNewCredentialDialog()}>
              Rotate secret
            </InlineButton>
            <InlineButton onClick={() => openChannelModelsDialog(activeChannelId)}>
              Manage channel models
            </InlineButton>
          </>
        }
      >
        <ToolbarInline>
          <ToolbarSearchField
            label="Search catalog"
            value={search}
            onChange={(event) => setSearch(event.target.value)}
            placeholder="channel, provider, model, credential"
          />
          <ToolbarField label="Catalog lane">
            <Select
              value={catalogLane}
              onChange={(event) => setCatalogLane(event.target.value as CatalogLane)}
            >
              <option value="channels">Channels</option>
              <option value="providers">Providers</option>
              <option value="credentials">Credentials</option>
              <option value="variants">Variants</option>
            </Select>
          </ToolbarField>
          <ToolbarField label="Channel focus">
            <Select
              value={activeChannelId}
              onChange={(event) => setActiveChannelId(event.target.value)}
            >
              {snapshot.channels.map((channel) => (
                <option key={channel.id} value={channel.id}>
                  {channel.name}
                </option>
              ))}
            </Select>
          </ToolbarField>
        </ToolbarInline>
      </PageToolbar>

      <Dialog
        open={isChannelDialogOpen}
        onOpenChange={(nextOpen) =>
          nextOpen ? setIsChannelDialogOpen(true) : resetChannelDialog()
        }
      >
        <DialogContent size="large">
          <AdminDialog
            title={editingChannelId ? 'Edit channel' : 'New channel'}
            detail="Channel records can define a starter set of supported models. For ongoing maintenance you can also open Manage models from the channel registry."
          >
            <form className="adminx-form-grid" onSubmit={(event) => void handleChannelSubmit(event)}>
              <FormField
                label="Channel id"
                hint="Stable lowercase identifiers are recommended."
              >
                <Input
                  value={channelDraft.id}
                  onChange={(event) =>
                    setChannelDraft((current) => ({
                      ...current,
                      id: event.target.value,
                    }))
                  }
                  disabled={Boolean(editingChannelId)}
                  required
                />
              </FormField>
              <FormField label="Channel name">
                <Input
                  value={channelDraft.name}
                  onChange={(event) =>
                    setChannelDraft((current) => ({
                      ...current,
                      name: event.target.value,
                    }))
                  }
                  required
                />
              </FormField>
              <div className="adminx-note">
                <strong>Supported models</strong>
                <p>
                  The channel modal supports dynamic model rows, including model ID and model
                  display name, so initial channel configuration can be completed in one pass.
                </p>
              </div>
              {channelSeedModels.map((draft, index) => (
                <div key={draft.draft_id} className="adminx-form-grid">
                  <div className="adminx-row">
                    <strong>Model #{index + 1}</strong>
                    <InlineButton
                      disabled={channelSeedModels.length === 1}
                      onClick={() =>
                        setChannelSeedModels((current) =>
                          current.length === 1
                            ? current
                            : current.filter((item) => item.draft_id !== draft.draft_id),
                        )
                      }
                    >
                      Remove
                    </InlineButton>
                  </div>
                  <FormField label="Model ID">
                    <Input
                      value={draft.model_id}
                      onChange={(event) =>
                        setChannelSeedModels((current) =>
                          current.map((item) =>
                            item.draft_id === draft.draft_id
                              ? { ...item, model_id: event.target.value }
                              : item,
                          ),
                        )
                      }
                    />
                  </FormField>
                  <FormField label="Model display name">
                    <Input
                      value={draft.model_display_name}
                      onChange={(event) =>
                        setChannelSeedModels((current) =>
                          current.map((item) =>
                            item.draft_id === draft.draft_id
                              ? { ...item, model_display_name: event.target.value }
                              : item,
                          ),
                        )
                      }
                    />
                  </FormField>
                  <FormField label="Capabilities">
                    <Input
                      value={draft.capabilities}
                      onChange={(event) =>
                        setChannelSeedModels((current) =>
                          current.map((item) =>
                            item.draft_id === draft.draft_id
                              ? { ...item, capabilities: event.target.value }
                              : item,
                          ),
                        )
                      }
                    />
                  </FormField>
                  <FormField label="Context window">
                    <Input
                      value={draft.context_window}
                      onChange={(event) =>
                        setChannelSeedModels((current) =>
                          current.map((item) =>
                            item.draft_id === draft.draft_id
                              ? { ...item, context_window: event.target.value }
                              : item,
                          ),
                        )
                      }
                      type="number"
                    />
                  </FormField>
                  <FormField label="Streaming">
                    <Select
                      value={draft.streaming ? 'true' : 'false'}
                      onChange={(event) =>
                        setChannelSeedModels((current) =>
                          current.map((item) =>
                            item.draft_id === draft.draft_id
                              ? { ...item, streaming: event.target.value === 'true' }
                              : item,
                          ),
                        )
                      }
                    >
                      <option value="true">Enabled</option>
                      <option value="false">Disabled</option>
                    </Select>
                  </FormField>
                  <FormField label="Description">
                    <Textarea
                      rows={3}
                      value={draft.description}
                      onChange={(event) =>
                        setChannelSeedModels((current) =>
                          current.map((item) =>
                            item.draft_id === draft.draft_id
                              ? { ...item, description: event.target.value }
                              : item,
                          ),
                        )
                      }
                    />
                  </FormField>
                </div>
              ))}
              <DialogFooter>
                <InlineButton
                  onClick={() =>
                    setChannelSeedModels((current) => [...current, emptySeedModelDraft()])
                  }
                >
                  Add supported model
                </InlineButton>
                <InlineButton onClick={resetChannelDialog}>Cancel</InlineButton>
                <InlineButton tone="primary" type="submit">
                  {editingChannelId ? 'Save channel' : 'Create channel'}
                </InlineButton>
              </DialogFooter>
            </form>
          </AdminDialog>
        </DialogContent>
      </Dialog>

      <Dialog
        open={isProviderDialogOpen}
        onOpenChange={(nextOpen) =>
          nextOpen ? setIsProviderDialogOpen(true) : resetProviderDialog()
        }
      >
        <DialogContent size="large">
          <AdminDialog
            title={editingProviderId ? 'Edit proxy provider' : 'New proxy provider'}
            detail="Provider records define adapter identity, endpoint information, and bound channels."
          >
            <form className="adminx-form-grid" onSubmit={(event) => void handleProviderSubmit(event)}>
              <FormField label="Provider id">
                <Input
                  value={providerDraft.id}
                  onChange={(event) =>
                    setProviderDraft((current) => ({
                      ...current,
                      id: event.target.value,
                    }))
                  }
                  disabled={Boolean(editingProviderId)}
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
                {snapshot.channels.length ? (
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
                ) : (
                  <Input
                    value={providerDraft.primary_channel_id}
                    onChange={(event) =>
                      setProviderDraft((current) => ({
                        ...current,
                        primary_channel_id: event.target.value,
                      }))
                    }
                    required
                  />
                )}
              </FormField>
              <div className="adminx-note">
                <strong>Bound channels</strong>
                <p>
                  Choose every API surface this proxy provider should expose. The primary channel
                  becomes the default runtime affinity.
                </p>
              </div>
              {snapshot.channels.length ? (
                <div className="adminx-form-grid">
                  {snapshot.channels.map((channel) => (
                    <label key={channel.id} className="adminx-row">
                      <Checkbox
                        checked={providerDraft.bound_channel_ids.includes(channel.id)}
                        onChange={(event) =>
                          setProviderDraft((current) => ({
                            ...current,
                            bound_channel_ids: event.target.checked
                              ? Array.from(
                                  new Set([...current.bound_channel_ids, channel.id]),
                                )
                              : current.bound_channel_ids.filter(
                                  (value) => value !== channel.id,
                                ),
                          }))
                        }
                      />
                      <span>
                        {channel.name} ({channel.id})
                      </span>
                    </label>
                  ))}
                </div>
              ) : null}
              <DialogFooter>
                <InlineButton onClick={resetProviderDialog}>Cancel</InlineButton>
                <InlineButton tone="primary" type="submit">
                  {editingProviderId ? 'Save provider' : 'Create provider'}
                </InlineButton>
              </DialogFooter>
            </form>
          </AdminDialog>
        </DialogContent>
      </Dialog>

      <Dialog
        open={isCredentialDialogOpen}
        onOpenChange={(nextOpen) =>
          nextOpen ? setIsCredentialDialogOpen(true) : resetCredentialDialog()
        }
      >
        <DialogContent size="large">
          <AdminDialog
            title="Rotate credential"
            detail="Router credentials are stored as encrypted records in ai_router_credential_records. Cleartext input is write-only."
          >
            <form className="adminx-form-grid" onSubmit={(event) => void handleCredentialSubmit(event)}>
              <FormField label="Tenant">
                {snapshot.tenants.length ? (
                  <Select
                    value={credentialDraft.tenant_id}
                    onChange={(event) =>
                      setCredentialDraft((current) => ({
                        ...current,
                        tenant_id: event.target.value,
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
                    value={credentialDraft.tenant_id}
                    onChange={(event) =>
                      setCredentialDraft((current) => ({
                        ...current,
                        tenant_id: event.target.value,
                      }))
                    }
                    required
                  />
                )}
              </FormField>
              <FormField label="Proxy provider">
                {snapshot.providers.length ? (
                  <Select
                    value={credentialDraft.provider_id}
                    onChange={(event) =>
                      setCredentialDraft((current) => ({
                        ...current,
                        provider_id: event.target.value,
                      }))
                    }
                  >
                    {snapshot.providers.map((provider) => (
                      <option key={provider.id} value={provider.id}>
                        {provider.display_name} ({provider.id})
                      </option>
                    ))}
                  </Select>
                ) : (
                  <Input
                    value={credentialDraft.provider_id}
                    onChange={(event) =>
                      setCredentialDraft((current) => ({
                        ...current,
                        provider_id: event.target.value,
                      }))
                    }
                    required
                  />
                )}
              </FormField>
              <FormField label="Key reference">
                <Input
                  value={credentialDraft.key_reference}
                  onChange={(event) =>
                    setCredentialDraft((current) => ({
                      ...current,
                      key_reference: event.target.value,
                    }))
                  }
                  required
                />
              </FormField>
              <FormField
                label="Secret value"
                hint="Only encrypted ciphertext and secret metadata are retained in storage."
              >
                <Input
                  value={credentialDraft.secret_value}
                  onChange={(event) =>
                    setCredentialDraft((current) => ({
                      ...current,
                      secret_value: event.target.value,
                    }))
                  }
                  type="password"
                  required
                />
              </FormField>
              <DialogFooter>
                <InlineButton onClick={resetCredentialDialog}>Cancel</InlineButton>
                <InlineButton tone="primary" type="submit">
                  Save credential
                </InlineButton>
              </DialogFooter>
            </form>
          </AdminDialog>
        </DialogContent>
      </Dialog>

      <section className="adminx-page-grid">
        <div className="adminx-row">
          <div className="adminx-row">
            <strong>Catalog workbench</strong>
            <Pill tone="default">{catalogLaneLabel}</Pill>
            <Pill tone={catalogHealthTone}>{catalogHealthLabel}</Pill>
          </div>
          <Pill tone={activeCredentialCount ? 'live' : 'danger'}>
            {activeCredentialCount} credentials
          </Pill>
        </div>
        <p className="text-sm text-zinc-500 dark:text-zinc-400">
          {catalogWorkbenchDetail}
        </p>
        <DataTable
          columns={catalogColumns}
          rows={catalogRows}
          empty={catalogEmpty}
          getKey={(row) =>
            row.kind === 'channel'
              ? row.id
              : row.kind === 'provider'
                ? row.id
                : row.kind === 'credential'
                  ? `${row.tenant_id}:${row.provider_id}:${row.key_reference}`
                  : `${row.external_name}:${row.provider_id}`
          }
        />
      </section>

      <Dialog open={isChannelModelsDialogOpen} onOpenChange={setIsChannelModelsDialogOpen}>
        <DialogContent size="large">
          <AdminDialog
            title={selectedChannel ? `Manage models · ${selectedChannel.name}` : 'Manage models'}
            detail="Maintain the channel-level API model list. Each channel model can open its own pricing registry by proxy provider."
          >
            <div className="adminx-page-grid">
              <div className="adminx-row">
                <InlineButton tone="primary" onClick={() => openNewChannelModelDialog(activeChannelId)}>
                  New channel model
                </InlineButton>
              </div>
              <section className="adminx-page-grid">
                <div className="adminx-row">
                  <strong>Channel model roster</strong>
                  <Pill tone="default">{filteredSelectedChannelModels.length} models</Pill>
                </div>

                {filteredSelectedChannelModels.length ? (
                  filteredSelectedChannelModels.map((model) => {
                    const modelPricingCount = snapshot.modelPrices.filter(
                      (record) =>
                        record.channel_id === model.channel_id && record.model_id === model.model_id,
                    ).length;

                    return (
                      <article
                        key={`${model.channel_id}:${model.model_id}`}
                        className="rounded-[24px] border border-zinc-200/80 bg-zinc-50/80 p-5 dark:border-zinc-800/80 dark:bg-zinc-900/70"
                      >
                        <div className="flex flex-col gap-4 border-b border-zinc-200/80 pb-4 dark:border-zinc-800/80 md:flex-row md:items-start md:justify-between">
                          <div className="space-y-1">
                            <h3 className="text-base font-semibold tracking-tight text-zinc-950 dark:text-zinc-50">
                              {model.model_display_name}
                            </h3>
                            <p className="text-sm text-zinc-500 dark:text-zinc-400">
                              {model.model_id}
                            </p>
                          </div>
                          <div className="flex flex-wrap gap-2">
                            <Pill tone="seed">{model.capabilities.join(', ') || '-'}</Pill>
                            <Pill tone={modelPricingCount ? 'live' : 'danger'}>
                              {modelPricingCount} pricing rows
                            </Pill>
                          </div>
                        </div>

                        <div className="mt-4 flex flex-col gap-4 md:flex-row md:items-end md:justify-between">
                          <div className="space-y-1">
                            <span className="text-[11px] font-semibold uppercase tracking-[0.18em] text-zinc-500 dark:text-zinc-400">
                              Capabilities
                            </span>
                            <p className="text-sm text-zinc-600 dark:text-zinc-300">
                              {model.capabilities.join(', ') || 'No capabilities declared'}
                            </p>
                          </div>
                          <div className="adminx-row">
                            <InlineButton onClick={() => openEditChannelModelDialog(model)}>
                              Edit
                            </InlineButton>
                            <InlineButton onClick={() => openPricingDialog(model)}>
                              Manage pricing
                            </InlineButton>
                            <InlineButton
                              tone="danger"
                              onClick={() =>
                                setPendingDelete({
                                  kind: 'channel-model',
                                  label: `${model.model_display_name} (${model.model_id})`,
                                  channelId: model.channel_id,
                                  modelId: model.model_id,
                                })
                              }
                            >
                              Delete
                            </InlineButton>
                          </div>
                        </div>
                      </article>
                    );
                  })
                ) : (
                  <div className="rounded-[24px] border border-dashed border-zinc-200/80 bg-zinc-50/50 p-6 text-sm text-zinc-500 dark:border-zinc-800/80 dark:bg-zinc-900/40 dark:text-zinc-400">
                    Add a channel model to start exposing this channel through the catalog workbench.
                  </div>
                )}
              </section>
            </div>
          </AdminDialog>
        </DialogContent>
      </Dialog>

      <Dialog
        open={isChannelModelEditorOpen}
        onOpenChange={(nextOpen) =>
          nextOpen ? setIsChannelModelEditorOpen(true) : resetChannelModelEditor()
        }
      >
        <DialogContent size="large">
          <AdminDialog
            title={editingChannelModelKey ? 'Edit channel model' : 'New channel model'}
            detail="Channel model rows define what the API catalog exposes before pricing and provider availability are attached."
          >
            <form
              className="adminx-form-grid"
              onSubmit={(event) => void handleChannelModelSubmit(event)}
            >
              <FormField label="Channel">
                {snapshot.channels.length ? (
                  <Select
                    value={channelModelDraft.channel_id}
                    onChange={(event) =>
                      setChannelModelDraft((current) => ({
                        ...current,
                        channel_id: event.target.value,
                      }))
                    }
                  >
                    {snapshot.channels.map((channel) => (
                      <option key={channel.id} value={channel.id}>
                        {channel.name} ({channel.id})
                      </option>
                    ))}
                  </Select>
                ) : (
                  <Input
                    value={channelModelDraft.channel_id}
                    onChange={(event) =>
                      setChannelModelDraft((current) => ({
                        ...current,
                        channel_id: event.target.value,
                      }))
                    }
                    required
                  />
                )}
              </FormField>
              <FormField label="Model ID">
                <Input
                  value={channelModelDraft.model_id}
                  onChange={(event) =>
                    setChannelModelDraft((current) => ({
                      ...current,
                      model_id: event.target.value,
                    }))
                  }
                  disabled={Boolean(editingChannelModelKey)}
                  required
                />
              </FormField>
              <FormField label="Model display name">
                <Input
                  value={channelModelDraft.model_display_name}
                  onChange={(event) =>
                    setChannelModelDraft((current) => ({
                      ...current,
                      model_display_name: event.target.value,
                    }))
                  }
                  required
                />
              </FormField>
              <FormField label="Capabilities">
                <Input
                  value={channelModelDraft.capabilities}
                  onChange={(event) =>
                    setChannelModelDraft((current) => ({
                      ...current,
                      capabilities: event.target.value,
                    }))
                  }
                  required
                />
              </FormField>
              <FormField label="Context window">
                <Input
                  value={channelModelDraft.context_window}
                  onChange={(event) =>
                    setChannelModelDraft((current) => ({
                      ...current,
                      context_window: event.target.value,
                    }))
                  }
                  type="number"
                />
              </FormField>
              <FormField label="Streaming">
                <Select
                  value={channelModelDraft.streaming ? 'true' : 'false'}
                  onChange={(event) =>
                    setChannelModelDraft((current) => ({
                      ...current,
                      streaming: event.target.value === 'true',
                    }))
                  }
                >
                  <option value="true">Enabled</option>
                  <option value="false">Disabled</option>
                </Select>
              </FormField>
              <FormField label="Description">
                <Textarea
                  rows={3}
                  value={channelModelDraft.description}
                  onChange={(event) =>
                    setChannelModelDraft((current) => ({
                      ...current,
                      description: event.target.value,
                    }))
                  }
                />
              </FormField>
              <DialogFooter>
                <InlineButton onClick={resetChannelModelEditor}>Cancel</InlineButton>
                <InlineButton tone="primary" type="submit">
                  {editingChannelModelKey ? 'Save channel model' : 'Create channel model'}
                </InlineButton>
              </DialogFooter>
            </form>
          </AdminDialog>
        </DialogContent>
      </Dialog>

      <Dialog open={isPricingDialogOpen} onOpenChange={setIsPricingDialogOpen}>
        <DialogContent size="large">
          <AdminDialog
            title={
              pricingTarget
                ? `Manage pricing · ${pricingTarget.model_display_name}`
                : 'Manage pricing'
            }
            detail="Maintain provider-specific pricing for input, output, cache, and per-request cost dimensions."
          >
            <div className="adminx-page-grid">
              <div className="grid gap-3 md:grid-cols-2 xl:grid-cols-4">
                <article className="rounded-3xl border border-zinc-200/80 bg-white/92 p-4 shadow-sm dark:border-zinc-800/80 dark:bg-zinc-950/85">
                  <span className="text-[11px] font-semibold uppercase tracking-[0.18em] text-zinc-500 dark:text-zinc-400">
                    Default large-model billing unit
                  </span>
                  <strong className="mt-3 block text-lg font-semibold tracking-tight text-zinc-950 dark:text-zinc-50">
                    Million tokens
                  </strong>
                  <p className="mt-2 text-sm leading-6 text-zinc-500 dark:text-zinc-400">
                    Use token-scale pricing for most text and reasoning models unless the provider
                    publishes a different meter.
                  </p>
                </article>
                <article className="rounded-3xl border border-zinc-200/80 bg-white/92 p-4 shadow-sm dark:border-zinc-800/80 dark:bg-zinc-950/85">
                  <span className="text-[11px] font-semibold uppercase tracking-[0.18em] text-zinc-500 dark:text-zinc-400">
                    Pricing rows
                  </span>
                  <strong className="mt-3 block text-lg font-semibold tracking-tight text-zinc-950 dark:text-zinc-50">
                    {selectedModelPrices.length}
                  </strong>
                  <p className="mt-2 text-sm leading-6 text-zinc-500 dark:text-zinc-400">
                    Provider-specific price mappings currently registered for this channel model.
                  </p>
                </article>
                <article className="rounded-3xl border border-zinc-200/80 bg-white/92 p-4 shadow-sm dark:border-zinc-800/80 dark:bg-zinc-950/85">
                  <span className="text-[11px] font-semibold uppercase tracking-[0.18em] text-zinc-500 dark:text-zinc-400">
                    Active provider prices
                  </span>
                  <strong className="mt-3 block text-lg font-semibold tracking-tight text-zinc-950 dark:text-zinc-50">
                    {activeSelectedModelPrices.length}
                  </strong>
                  <p className="mt-2 text-sm leading-6 text-zinc-500 dark:text-zinc-400">
                    Active rows are eligible for downstream billing, quoting, and cost analytics.
                  </p>
                </article>
                <article className="rounded-3xl border border-zinc-200/80 bg-white/92 p-4 shadow-sm dark:border-zinc-800/80 dark:bg-zinc-950/85">
                  <span className="text-[11px] font-semibold uppercase tracking-[0.18em] text-zinc-500 dark:text-zinc-400">
                    Charge dimensions
                  </span>
                  <strong className="mt-3 block text-lg font-semibold tracking-tight text-zinc-950 dark:text-zinc-50">
                    {pricingUnitCoverage.join(', ') || 'Pending pricing'}
                  </strong>
                  <p className="mt-2 text-sm leading-6 text-zinc-500 dark:text-zinc-400">
                    Input, output, cache, and per-request charges can be mixed under one provider row.
                  </p>
                </article>
              </div>
              <div className="adminx-row">
                <InlineButton tone="primary" onClick={openNewModelPriceDialog}>
                  New model pricing
                </InlineButton>
              </div>
              <section className="adminx-page-grid">
                <div className="adminx-row">
                  <strong>Pricing roster</strong>
                  <Pill tone="default">{filteredSelectedModelPrices.length} price rows</Pill>
                </div>

                {filteredSelectedModelPrices.length ? (
                  filteredSelectedModelPrices.map((record) => (
                    <article
                      key={`${record.channel_id}:${record.model_id}:${record.proxy_provider_id}`}
                      className="rounded-[24px] border border-zinc-200/80 bg-zinc-50/80 p-5 dark:border-zinc-800/80 dark:bg-zinc-900/70"
                    >
                      <div className="flex flex-col gap-4 border-b border-zinc-200/80 pb-4 dark:border-zinc-800/80 md:flex-row md:items-start md:justify-between">
                        <div className="space-y-1">
                          <h3 className="text-base font-semibold tracking-tight text-zinc-950 dark:text-zinc-50">
                            {providerNameById.get(record.proxy_provider_id) ?? record.proxy_provider_id}
                          </h3>
                          <p className="text-sm text-zinc-500 dark:text-zinc-400">
                            {record.currency_code} / {priceUnitLabel(record.price_unit)}
                          </p>
                        </div>
                        <div className="flex flex-wrap gap-2">
                          <Pill tone={record.is_active ? 'live' : 'danger'}>
                            {record.is_active ? 'active' : 'inactive'}
                          </Pill>
                          <Pill tone="seed">{record.price_unit}</Pill>
                        </div>
                      </div>

                      <div className="mt-4 grid gap-3 md:grid-cols-2 xl:grid-cols-4">
                        <div className="rounded-[20px] border border-zinc-200/80 bg-white/90 p-4 dark:border-zinc-800/80 dark:bg-zinc-950/70">
                          <span className="text-[11px] font-semibold uppercase tracking-[0.18em] text-zinc-500 dark:text-zinc-400">
                            Input / Output
                          </span>
                          <strong className="mt-2 block text-sm font-semibold text-zinc-950 dark:text-zinc-50">
                            {record.input_price} / {record.output_price}
                          </strong>
                        </div>
                        <div className="rounded-[20px] border border-zinc-200/80 bg-white/90 p-4 dark:border-zinc-800/80 dark:bg-zinc-950/70">
                          <span className="text-[11px] font-semibold uppercase tracking-[0.18em] text-zinc-500 dark:text-zinc-400">
                            Cache read / write
                          </span>
                          <strong className="mt-2 block text-sm font-semibold text-zinc-950 dark:text-zinc-50">
                            {record.cache_read_price} / {record.cache_write_price}
                          </strong>
                        </div>
                        <div className="rounded-[20px] border border-zinc-200/80 bg-white/90 p-4 dark:border-zinc-800/80 dark:bg-zinc-950/70">
                          <span className="text-[11px] font-semibold uppercase tracking-[0.18em] text-zinc-500 dark:text-zinc-400">
                            Request price
                          </span>
                          <strong className="mt-2 block text-sm font-semibold text-zinc-950 dark:text-zinc-50">
                            {record.request_price}
                          </strong>
                        </div>
                        <div className="rounded-[20px] border border-zinc-200/80 bg-white/90 p-4 dark:border-zinc-800/80 dark:bg-zinc-950/70">
                          <span className="text-[11px] font-semibold uppercase tracking-[0.18em] text-zinc-500 dark:text-zinc-400">
                            Billing unit
                          </span>
                          <strong className="mt-2 block text-sm font-semibold text-zinc-950 dark:text-zinc-50">
                            {priceUnitLabel(record.price_unit)}
                          </strong>
                        </div>
                      </div>

                      <div className="mt-4 flex justify-end">
                        <div className="adminx-row">
                          <InlineButton onClick={() => openEditModelPriceDialog(record)}>
                            Edit
                          </InlineButton>
                          <InlineButton
                            tone="danger"
                            onClick={() =>
                              setPendingDelete({
                                kind: 'model-price',
                                label: `${record.model_id} / ${record.proxy_provider_id}`,
                                channelId: record.channel_id,
                                modelId: record.model_id,
                                proxyProviderId: record.proxy_provider_id,
                              })
                            }
                          >
                            Delete
                          </InlineButton>
                        </div>
                      </div>
                    </article>
                  ))
                ) : (
                  <div className="rounded-[24px] border border-dashed border-zinc-200/80 bg-zinc-50/50 p-6 text-sm text-zinc-500 dark:border-zinc-800/80 dark:bg-zinc-900/40 dark:text-zinc-400">
                    Add provider-specific pricing to activate billing and commercial visibility for this channel model.
                  </div>
                )}
              </section>
            </div>
          </AdminDialog>
        </DialogContent>
      </Dialog>

      <Dialog
        open={isModelPriceEditorOpen}
        onOpenChange={(nextOpen) =>
          nextOpen ? setIsModelPriceEditorOpen(true) : resetModelPriceEditor()
        }
      >
        <DialogContent size="large">
          <AdminDialog
            title={editingModelPriceKey ? 'Edit model pricing' : 'New model pricing'}
            detail="Pricing rows connect a channel model to a proxy provider and define input, output, cache, and request costs."
          >
            <form
              className="adminx-form-grid"
              onSubmit={(event) => void handleModelPriceSubmit(event)}
            >
              <FormField label="Channel">
                <Input value={modelPriceDraft.channel_id} disabled />
              </FormField>
              <FormField label="Model">
                <Input value={modelPriceDraft.model_id} disabled />
              </FormField>
              <FormField label="Proxy provider">
                {snapshot.providers.length ? (
                  <Select
                    value={modelPriceDraft.proxy_provider_id}
                    onChange={(event) =>
                      setModelPriceDraft((current) => ({
                        ...current,
                        proxy_provider_id: event.target.value,
                      }))
                    }
                  >
                    <option value="">Select provider</option>
                    {snapshot.providers.map((provider) => (
                      <option key={provider.id} value={provider.id}>
                        {provider.display_name} ({provider.id})
                      </option>
                    ))}
                  </Select>
                ) : (
                  <Input
                    value={modelPriceDraft.proxy_provider_id}
                    onChange={(event) =>
                      setModelPriceDraft((current) => ({
                        ...current,
                        proxy_provider_id: event.target.value,
                      }))
                    }
                    required
                  />
                )}
              </FormField>
              <FormField label="Currency code">
                <Input
                  value={modelPriceDraft.currency_code}
                  onChange={(event) =>
                    setModelPriceDraft((current) => ({
                      ...current,
                      currency_code: event.target.value,
                    }))
                  }
                  required
                />
              </FormField>
              <FormField
                label="Price unit"
                hint={`${priceUnitDetail(modelPriceDraft.price_unit)} Default large-model billing unit is Million tokens.`}
              >
                <Select
                  value={modelPriceDraft.price_unit}
                  onChange={(event) =>
                    setModelPriceDraft((current) => ({
                      ...current,
                      price_unit: event.target.value,
                    }))
                  }
                  required
                >
                  {PRICE_UNIT_OPTIONS.map((option) => (
                    <option key={option.value} value={option.value}>
                      {option.label}
                    </option>
                  ))}
                </Select>
              </FormField>
              <FormField
                label="Input price"
                hint="Charge applied to inbound prompt tokens or the equivalent upstream input meter."
              >
                <Input
                  value={modelPriceDraft.input_price}
                  onChange={(event) =>
                    setModelPriceDraft((current) => ({
                      ...current,
                      input_price: event.target.value,
                    }))
                  }
                  type="number"
                  step="0.000001"
                  required
                />
              </FormField>
              <FormField
                label="Output price"
                hint="Charge applied to generated tokens or the equivalent upstream output meter."
              >
                <Input
                  value={modelPriceDraft.output_price}
                  onChange={(event) =>
                    setModelPriceDraft((current) => ({
                      ...current,
                      output_price: event.target.value,
                    }))
                  }
                  type="number"
                  step="0.000001"
                  required
                />
              </FormField>
              <FormField
                label="Cache read price"
                hint="Use when a provider exposes discounted read pricing for prompt cache hits."
              >
                <Input
                  value={modelPriceDraft.cache_read_price}
                  onChange={(event) =>
                    setModelPriceDraft((current) => ({
                      ...current,
                      cache_read_price: event.target.value,
                    }))
                  }
                  type="number"
                  step="0.000001"
                  required
                />
              </FormField>
              <FormField
                label="Cache write price"
                hint="Use when cache population is billed separately from normal input tokens."
              >
                <Input
                  value={modelPriceDraft.cache_write_price}
                  onChange={(event) =>
                    setModelPriceDraft((current) => ({
                      ...current,
                      cache_write_price: event.target.value,
                    }))
                  }
                  type="number"
                  step="0.000001"
                  required
                />
              </FormField>
              <FormField
                label="Request price"
                hint="Optional flat surcharge for hosted inference calls, media jobs, or routing overhead."
              >
                <Input
                  value={modelPriceDraft.request_price}
                  onChange={(event) =>
                    setModelPriceDraft((current) => ({
                      ...current,
                      request_price: event.target.value,
                    }))
                  }
                  type="number"
                  step="0.000001"
                  required
                />
              </FormField>
              <div className="rounded-3xl border border-zinc-200/80 bg-white/92 p-5 shadow-sm dark:border-zinc-800/80 dark:bg-zinc-950/85 md:col-span-2">
                <p className="text-[11px] font-semibold uppercase tracking-[0.18em] text-zinc-500 dark:text-zinc-400">
                  Charge dimensions
                </p>
                <p className="mt-3 text-sm leading-6 text-zinc-600 dark:text-zinc-300">
                  Input, output, cache, and per-request charges can be mixed under one provider row.
                  Use token-based units for mainstream LLM pricing, then switch to Image generated,
                  Audio second, Video minute, or Music track when the upstream API is media-metered
                  instead of token-metered.
                </p>
              </div>
              <FormField label="Status">
                <Select
                  value={modelPriceDraft.is_active ? 'active' : 'inactive'}
                  onChange={(event) =>
                    setModelPriceDraft((current) => ({
                      ...current,
                      is_active: event.target.value === 'active',
                    }))
                  }
                >
                  <option value="active">Active</option>
                  <option value="inactive">Inactive</option>
                </Select>
              </FormField>
              <DialogFooter>
                <InlineButton onClick={resetModelPriceEditor}>Cancel</InlineButton>
                <InlineButton tone="primary" type="submit">
                  {editingModelPriceKey ? 'Save model pricing' : 'Create model pricing'}
                </InlineButton>
              </DialogFooter>
            </form>
          </AdminDialog>
        </DialogContent>
      </Dialog>

      <ConfirmDialog
        open={Boolean(pendingDelete)}
        title={
          pendingDelete?.kind === 'channel'
            ? 'Delete channel'
            : pendingDelete?.kind === 'provider'
              ? 'Delete proxy provider'
              : pendingDelete?.kind === 'credential'
                ? 'Delete credential'
                : pendingDelete?.kind === 'channel-model'
                  ? 'Delete channel model'
                  : pendingDelete?.kind === 'model-price'
                    ? 'Delete model pricing'
                    : 'Delete model variant'
        }
        detail={
          pendingDelete?.kind === 'channel'
            ? `Delete ${pendingDelete.label}. Remove or rebind dependent proxy providers before retiring the channel.`
            : pendingDelete?.kind === 'provider'
              ? `Delete ${pendingDelete.label}. This removes the provider record and its pricing or credential associations from the active registry.`
              : pendingDelete?.kind === 'credential'
                ? `Delete credential ${pendingDelete.label}. The encrypted router secret record will be removed from storage.`
                : pendingDelete?.kind === 'channel-model'
                  ? `Delete ${pendingDelete.label}. Associated pricing rows for this channel model will also be retired.`
                  : pendingDelete?.kind === 'model-price'
                    ? `Delete pricing row ${pendingDelete.label}. This only removes the selected provider price mapping.`
                    : pendingDelete?.kind === 'model'
                      ? `Delete provider model variant ${pendingDelete.label}.`
                      : ''
        }
        confirmLabel="Delete now"
        onClose={() => setPendingDelete(null)}
        onConfirm={confirmDelete}
      />
    </div>
  );
}
