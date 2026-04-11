import { useDeferredValue, useEffect, useMemo, useState } from 'react';
import type { FormEvent } from 'react';
import type {
  AdminPageProps,
  ChannelModelRecord,
  CredentialRecord,
  ModelPriceRecord,
  ModelPriceTier,
  ProviderCatalogRecord,
  SaveProviderInput,
} from 'sdkwork-router-admin-types';
import {
  buildProviderSaveInput,
  describeProviderIntegration,
  providerSupportedModelKey,
  recommendedModelPriceSourceKind,
  translateAdminText,
} from 'sdkwork-router-admin-core';

import {
  channelModelDraftFromRecord,
  credentialDraftFromRecord,
  emptyChannelModelDraft,
  emptyCredentialDraft,
  emptyModelPriceDraft,
  emptyProviderDraft,
  modelPriceDraftFromRecord,
  parseOptionalNumber,
  parsePricingTiersJson,
  parseRequiredNumber,
  providerChannelIds,
  providerDraftFromRecord,
  splitCapabilities,
  type CatalogLane,
  type ChannelDraft,
  type ChannelModelDraft,
  type CredentialDraft,
  type ModelPriceDraft,
  type PendingDelete,
  type ProviderDraft,
  type VariantRecord,
} from './shared';

export type CatalogWorkspaceActions = {
  onSaveChannel: (input: { id: string; name: string }) => Promise<void>;
  onSaveProvider: (input: SaveProviderInput) => Promise<void>;
  onSaveCredential: (input: {
    tenant_id: string;
    provider_id: string;
    key_reference: string;
    secret_value: string;
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
    price_source_kind: string;
    billing_notes?: string | null;
    pricing_tiers: ModelPriceTier[];
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

type CatalogWorkspaceOptions = CatalogWorkspaceActions & {
  snapshot: AdminPageProps['snapshot'];
};

export function useCatalogWorkspaceState({
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
}: CatalogWorkspaceOptions) {
  const defaultChannelId = snapshot.channels[0]?.id ?? 'openai';
  const defaultProviderId = snapshot.providers[0]?.id ?? '';
  const defaultTenantId = snapshot.tenants[0]?.id ?? 'tenant-local';

  const [catalogLane, setCatalogLane] = useState<CatalogLane>('channels');
  const [search, setSearch] = useState('');
  const deferredSearch = useDeferredValue(search.trim().toLowerCase());

  const [activeChannelId, setActiveChannelId] = useState(defaultChannelId);
  const [selectedProviderId, setSelectedProviderId] = useState<string | null>(null);
  const [selectedCredentialKey, setSelectedCredentialKey] = useState<string | null>(null);
  const [selectedVariantKey, setSelectedVariantKey] = useState<string | null>(null);
  const [selectedPublicationKey, setSelectedPublicationKey] = useState<string | null>(null);

  const [channelDraft, setChannelDraft] = useState<ChannelDraft>({
    id: defaultChannelId,
    name: snapshot.channels[0]?.name ?? 'OpenAI',
  });
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
  const [editingChannelModelKey, setEditingChannelModelKey] = useState<string | null>(
    null,
  );
  const [modelPriceDraft, setModelPriceDraft] = useState<ModelPriceDraft>(
    emptyModelPriceDraft(defaultChannelId),
  );
  const [editingModelPriceKey, setEditingModelPriceKey] = useState<string | null>(null);
  const [pendingDelete, setPendingDelete] = useState<PendingDelete>(null);
  const [isDetailDrawerOpen, setIsDetailDrawerOpen] = useState(false);

  const [isChannelDialogOpen, setIsChannelDialogOpen] = useState(false);
  const [isProviderDialogOpen, setIsProviderDialogOpen] = useState(false);
  const [isCredentialDialogOpen, setIsCredentialDialogOpen] = useState(false);
  const [isChannelModelEditorOpen, setIsChannelModelEditorOpen] = useState(false);
  const [isModelPriceEditorOpen, setIsModelPriceEditorOpen] = useState(false);

  const providerNameById = useMemo(
    () =>
      new Map(snapshot.providers.map((provider) => [provider.id, provider.display_name])),
    [snapshot.providers],
  );
  const channelNameById = useMemo(
    () => new Map(snapshot.channels.map((channel) => [channel.id, channel.name])),
    [snapshot.channels],
  );

  const filteredChannels = useMemo(
    () =>
      snapshot.channels.filter(
        (channel) =>
          !deferredSearch
          || [channel.id, channel.name].join(' ').toLowerCase().includes(deferredSearch),
      ),
    [deferredSearch, snapshot.channels],
  );
  const filteredProviders = useMemo(
    () =>
      snapshot.providers.filter(
        (provider) =>
          !deferredSearch
          || [
            provider.id,
            provider.display_name,
            provider.channel_id,
            provider.adapter_kind,
            provider.protocol_kind,
            provider.base_url,
            provider.extension_id ?? '',
            provider.integration.default_plugin_family ?? '',
            describeProviderIntegration(provider),
            providerChannelIds(provider).join(' '),
            snapshot.providerModels
              .filter((record) => record.proxy_provider_id === provider.id)
              .map((record) => providerSupportedModelKey(record))
              .join(' '),
          ]
            .join(' ')
            .toLowerCase()
            .includes(deferredSearch),
      ),
    [deferredSearch, snapshot.providerModels, snapshot.providers],
  );
  const filteredCredentials = useMemo(
    () =>
      snapshot.credentials.filter(
        (credential) =>
          !deferredSearch
          || [
            credential.tenant_id,
            credential.provider_id,
            credential.key_reference,
            credential.secret_backend,
            credential.secret_local_file ?? '',
            credential.secret_keyring_service ?? '',
            credential.secret_master_key_id ?? '',
          ]
            .join(' ')
            .toLowerCase()
            .includes(deferredSearch),
      ),
    [deferredSearch, snapshot.credentials],
  );
  const filteredVariants = useMemo(
    () =>
      snapshot.models.filter(
        (model) =>
          !deferredSearch
          || [model.external_name, model.provider_id, model.capabilities.join(' ')]
            .join(' ')
            .toLowerCase()
            .includes(deferredSearch),
      ),
    [deferredSearch, snapshot.models],
  );

  const selectedChannel =
    snapshot.channels.find((channel) => channel.id === activeChannelId)
    ?? filteredChannels[0]
    ?? null;
  const selectedProvider =
    filteredProviders.find((provider) => provider.id === selectedProviderId)
    ?? filteredProviders[0]
    ?? null;
  const selectedCredential =
    filteredCredentials.find(
      (credential) =>
        `${credential.tenant_id}:${credential.provider_id}:${credential.key_reference}`
        === selectedCredentialKey,
    )
    ?? filteredCredentials[0]
    ?? null;
  const selectedVariant =
    filteredVariants.find(
      (variant) =>
        `${variant.provider_id}:${variant.external_name}` === selectedVariantKey,
    )
    ?? filteredVariants[0]
    ?? null;
  const selectedChannelModels = snapshot.channelModels.filter(
    (model) => model.channel_id === (selectedChannel?.id ?? defaultChannelId),
  );
  const selectedProviderModels = snapshot.providerModels.filter(
    (record) => record.proxy_provider_id === selectedProvider?.id,
  );
  const selectedProviderModelPrices = snapshot.modelPrices.filter(
    (record) => record.proxy_provider_id === selectedProvider?.id,
  );
  const selectedPublication =
    selectedChannelModels.find(
      (model) => `${model.channel_id}:${model.model_id}` === selectedPublicationKey,
    )
    ?? selectedChannelModels[0]
    ?? null;
  const selectedModelPrices = selectedPublication
    ? snapshot.modelPrices.filter(
        (record) =>
          record.channel_id === selectedPublication.channel_id
          && record.model_id === selectedPublication.model_id,
      )
    : [];
  const selectedChannelProviderCount = snapshot.providers.filter((provider) =>
    selectedChannel ? providerChannelIds(provider).includes(selectedChannel.id) : false,
  ).length;

  useEffect(() => {
    if (selectedChannel && selectedChannel.id !== activeChannelId) {
      setActiveChannelId(selectedChannel.id);
    }
  }, [activeChannelId, selectedChannel]);

  useEffect(() => {
    if (selectedProvider && selectedProvider.id !== selectedProviderId) {
      setSelectedProviderId(selectedProvider.id);
    }
  }, [selectedProvider, selectedProviderId]);

  useEffect(() => {
    if (selectedCredential) {
      const key = `${selectedCredential.tenant_id}:${selectedCredential.provider_id}:${selectedCredential.key_reference}`;
      if (key !== selectedCredentialKey) {
        setSelectedCredentialKey(key);
      }
    }
  }, [selectedCredential, selectedCredentialKey]);

  useEffect(() => {
    if (selectedVariant) {
      const key = `${selectedVariant.provider_id}:${selectedVariant.external_name}`;
      if (key !== selectedVariantKey) {
        setSelectedVariantKey(key);
      }
    }
  }, [selectedVariant, selectedVariantKey]);

  useEffect(() => {
    if (selectedPublication) {
      const key = `${selectedPublication.channel_id}:${selectedPublication.model_id}`;
      if (key !== selectedPublicationKey) {
        setSelectedPublicationKey(key);
      }
    }
  }, [selectedPublication, selectedPublicationKey]);

  function handleCatalogLaneChange(value: string) {
    setCatalogLane(value as CatalogLane);
    setIsDetailDrawerOpen(false);
  }

  function handleSearchChange(value: string) {
    setSearch(value);
    setIsDetailDrawerOpen(false);
  }

  function clearSearch() {
    setSearch('');
    setIsDetailDrawerOpen(false);
  }

  function handleDetailDrawerOpenChange(open: boolean) {
    setIsDetailDrawerOpen(open);
  }

  function openChannelDetail(channelId: string) {
    setActiveChannelId(channelId);
    setIsDetailDrawerOpen(true);
  }

  function openProviderDetail(providerId: string) {
    setSelectedProviderId(providerId);
    setIsDetailDrawerOpen(true);
  }

  function openCredentialDetail(key: string) {
    setSelectedCredentialKey(key);
    setIsDetailDrawerOpen(true);
  }

  function openVariantDetail(key: string) {
    setSelectedVariantKey(key);
    setIsDetailDrawerOpen(true);
  }

  function openNewChannelDialog() {
    setEditingChannelId(null);
    setChannelDraft({ id: '', name: '' });
    setIsChannelDialogOpen(true);
  }

  function openEditChannelDialog(channel: { id: string; name: string }) {
    setEditingChannelId(channel.id);
    setChannelDraft({ id: channel.id, name: channel.name });
    setIsChannelDialogOpen(true);
  }

  function openNewProviderDialog(channelId = selectedChannel?.id ?? defaultChannelId) {
    setEditingProviderId(null);
    setProviderDraft(emptyProviderDraft(channelId));
    setIsProviderDialogOpen(true);
  }

  function openEditProviderDialog(provider: ProviderCatalogRecord) {
    setEditingProviderId(provider.id);
    setProviderDraft(providerDraftFromRecord(provider, snapshot.providerModels));
    setIsProviderDialogOpen(true);
  }

  function openNewCredentialDialog(providerId = selectedProvider?.id ?? defaultProviderId) {
    setCredentialDraft(emptyCredentialDraft(defaultTenantId, providerId));
    setIsCredentialDialogOpen(true);
  }

  function openCredentialDialog(record?: CredentialRecord) {
    setCredentialDraft(
      record
        ? credentialDraftFromRecord(record)
        : emptyCredentialDraft(defaultTenantId, selectedProvider?.id ?? defaultProviderId),
    );
    setIsCredentialDialogOpen(true);
  }

  function openNewChannelModelDialog(
    channelId = selectedChannel?.id ?? defaultChannelId,
    variant?: VariantRecord,
  ) {
    setEditingChannelModelKey(null);
    setChannelModelDraft(
      emptyChannelModelDraft(
        channelId,
        variant?.external_name ?? '',
        variant?.external_name ?? '',
      ),
    );
    setIsChannelModelEditorOpen(true);
  }

  function openEditChannelModelDialog(record: ChannelModelRecord) {
    setEditingChannelModelKey(`${record.channel_id}:${record.model_id}`);
    setChannelModelDraft(channelModelDraftFromRecord(record));
    setIsChannelModelEditorOpen(true);
  }

  function openNewModelPriceDialog(
    publication = selectedPublication,
    options: {
      proxyProviderId?: string;
      priceSourceKind?: string;
    } = {},
  ) {
    if (!publication) {
      return;
    }

    const provider = options.proxyProviderId
      ? snapshot.providers.find((entry) => entry.id === options.proxyProviderId) ?? null
      : null;
    setEditingModelPriceKey(null);
    setModelPriceDraft(
      emptyModelPriceDraft(
        publication.channel_id,
        publication.model_id,
        options.proxyProviderId ?? '',
        options.priceSourceKind ?? recommendedModelPriceSourceKind(provider),
      ),
    );
    setSelectedPublicationKey(`${publication.channel_id}:${publication.model_id}`);
    setIsModelPriceEditorOpen(true);
  }

  function openEditModelPriceDialog(record: ModelPriceRecord) {
    setEditingModelPriceKey(
      `${record.channel_id}:${record.model_id}:${record.proxy_provider_id}`,
    );
    setModelPriceDraft(modelPriceDraftFromRecord(record));
    setSelectedPublicationKey(`${record.channel_id}:${record.model_id}`);
    setIsModelPriceEditorOpen(true);
  }

  function handlePrimaryAction() {
    if (catalogLane === 'channels') {
      openNewChannelDialog();
      return;
    }
    if (catalogLane === 'providers') {
      openNewProviderDialog();
      return;
    }
    if (catalogLane === 'credentials') {
      openNewCredentialDialog();
      return;
    }
    openNewChannelModelDialog(
      selectedChannel?.id ?? defaultChannelId,
      selectedVariant ?? undefined,
    );
  }

  async function handleChannelSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    await onSaveChannel(channelDraft);
    setActiveChannelId(channelDraft.id.trim());
    setIsChannelDialogOpen(false);
  }

  async function handleProviderSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    await onSaveProvider(buildProviderSaveInput(providerDraft));
    setIsProviderDialogOpen(false);
  }

  async function handleCredentialSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    await onSaveCredential({
      tenant_id: credentialDraft.tenant_id.trim(),
      provider_id: credentialDraft.provider_id.trim(),
      key_reference: credentialDraft.key_reference.trim(),
      secret_value: credentialDraft.secret_value,
    });
    setIsCredentialDialogOpen(false);
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
    setActiveChannelId(channelModelDraft.channel_id.trim());
    setIsChannelModelEditorOpen(false);
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
      price_source_kind: modelPriceDraft.price_source_kind.trim(),
      billing_notes: modelPriceDraft.billing_notes.trim() || null,
      pricing_tiers: parsePricingTiersJson(modelPriceDraft.pricing_tiers_json),
      is_active: modelPriceDraft.is_active,
    });
    setIsModelPriceEditorOpen(false);
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

  function handleDeleteDialogOpenChange(open: boolean) {
    if (!open) {
      setPendingDelete(null);
    }
  }

  const primaryActionLabel =
    catalogLane === 'channels'
      ? translateAdminText('New channel')
      : catalogLane === 'providers'
        ? translateAdminText('New provider')
        : catalogLane === 'credentials'
          ? translateAdminText('New credential')
          : translateAdminText('Publish variant');

  const deleteDialogDescription = pendingDelete
    ? translateAdminText(
        'Delete {label}. This action removes the selected catalog record.',
        {
          label: pendingDelete.label,
        },
      )
    : '';

  return {
    catalogLane,
    search,
    defaultChannelId,
    primaryActionLabel,
    filteredChannels,
    filteredCredentials,
    filteredProviders,
    filteredVariants,
    channelNameById,
    providerNameById,
    selectedChannel,
    selectedChannelModels,
    selectedChannelProviderCount,
    selectedCredential,
    selectedCredentialKey,
    selectedProviderModels,
    selectedProviderModelPrices,
    selectedModelPrices,
    selectedPublication,
    selectedProvider,
    selectedVariant,
    selectedVariantKey,
    channelDraft,
    editingChannelId,
    providerDraft,
    editingProviderId,
    credentialDraft,
    channelModelDraft,
    editingChannelModelKey,
    modelPriceDraft,
    editingModelPriceKey,
    pendingDelete,
    deleteDialogDescription,
    isDetailDrawerOpen,
    isChannelDialogOpen,
    isProviderDialogOpen,
    isCredentialDialogOpen,
    isChannelModelEditorOpen,
    isModelPriceEditorOpen,
    handleCatalogLaneChange,
    handleSearchChange,
    clearSearch,
    handleDetailDrawerOpenChange,
    handlePrimaryAction,
    openChannelDetail,
    openProviderDetail,
    openCredentialDetail,
    openVariantDetail,
    openEditChannelDialog,
    openNewCredentialDialog,
    openCredentialDialog,
    openEditChannelModelDialog,
    openEditModelPriceDialog,
    openEditProviderDialog,
    openNewChannelModelDialog,
    openNewModelPriceDialog,
    openNewProviderDialog,
    setActiveChannelId,
    setSelectedCredentialKey,
    setSelectedProviderId,
    setSelectedVariantKey,
    setChannelDraft,
    setProviderDraft,
    setCredentialDraft,
    setChannelModelDraft,
    setModelPriceDraft,
    setIsChannelDialogOpen,
    setIsProviderDialogOpen,
    setIsCredentialDialogOpen,
    setIsChannelModelEditorOpen,
    setIsModelPriceEditorOpen,
    setPendingDelete,
    handleChannelSubmit,
    handleProviderSubmit,
    handleCredentialSubmit,
    handleChannelModelSubmit,
    handleModelPriceSubmit,
    confirmDelete,
    handleDeleteDialogOpenChange,
  };
}
