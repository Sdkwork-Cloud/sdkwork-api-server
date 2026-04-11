import {
  Button,
  Card,
  CardContent,
  Input,
  Label,
  SegmentedControl,
  StatCard,
} from '@sdkwork/ui-pc-react';
import { Search } from 'lucide-react';
import { type SaveProviderInput, useAdminI18n } from 'sdkwork-router-admin-core';
import type { AdminPageProps, ModelPriceTier } from 'sdkwork-router-admin-types';

import { CatalogDetailDrawer } from './page/CatalogDetailDrawer';
import {
  CatalogChannelDialog,
  CatalogChannelModelDialog,
  CatalogCredentialDialog,
  CatalogModelPriceDialog,
  CatalogProviderDialog,
} from './page/CatalogDialogs';
import { CatalogRegistrySection } from './page/CatalogRegistrySection';
import { ConfirmActionDialog } from './page/shared';
import { useCatalogWorkspaceState } from './page/useCatalogWorkspaceState';

type CatalogPageProps = AdminPageProps & {
  onSaveChannel: (input: { id: string; name: string }) => Promise<void>;
  onSaveProvider: (input: SaveProviderInput) => Promise<void>;
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

export function CatalogPage({
  snapshot,
  onSaveChannel,
  onSaveProvider,
  onSaveCredential,
  onSaveModel: _onSaveModel,
  onSaveChannelModel,
  onSaveModelPrice,
  onDeleteChannel,
  onDeleteProvider,
  onDeleteCredential,
  onDeleteModel,
  onDeleteChannelModel,
  onDeleteModelPrice,
}: CatalogPageProps) {
  const { formatNumber, t } = useAdminI18n();
  const {
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
    selectedModelPrices,
    selectedProviderModels,
    selectedProviderModelPrices,
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
    openEditChannelModelDialog,
    openNewCredentialDialog,
    openCredentialDialog,
    openEditModelPriceDialog,
    openEditProviderDialog,
    openNewChannelModelDialog,
    openNewModelPriceDialog,
    openNewProviderDialog,
    setChannelDraft,
    setPendingDelete,
    setProviderDraft,
    setCredentialDraft,
    setChannelModelDraft,
    setModelPriceDraft,
    setIsChannelDialogOpen,
    setIsProviderDialogOpen,
    setIsCredentialDialogOpen,
    setIsChannelModelEditorOpen,
    setIsModelPriceEditorOpen,
    handleChannelSubmit,
    handleProviderSubmit,
    handleCredentialSubmit,
    handleChannelModelSubmit,
    handleModelPriceSubmit,
    confirmDelete,
    handleDeleteDialogOpenChange,
  } = useCatalogWorkspaceState({
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
  });

  const catalogAreaLabel =
    catalogLane === 'channels'
      ? t('Channels')
      : catalogLane === 'providers'
        ? t('Providers')
        : catalogLane === 'credentials'
          ? t('Credentials')
          : t('Variants');

  function requestDeleteSelected() {
    if (catalogLane === 'channels' && selectedChannel) {
      setPendingDelete({
        kind: 'channel',
        label: `${selectedChannel.name} (${selectedChannel.id})`,
        channelId: selectedChannel.id,
      });
      return;
    }

    if (catalogLane === 'providers' && selectedProvider) {
      setPendingDelete({
        kind: 'provider',
        label: `${selectedProvider.display_name} (${selectedProvider.id})`,
        providerId: selectedProvider.id,
      });
      return;
    }

    if (catalogLane === 'credentials' && selectedCredential) {
      setPendingDelete({
        kind: 'credential',
        label: selectedCredential.key_reference,
        tenantId: selectedCredential.tenant_id,
        providerId: selectedCredential.provider_id,
        keyReference: selectedCredential.key_reference,
      });
      return;
    }

    if (catalogLane === 'variants' && selectedVariant) {
      setPendingDelete({
        kind: 'model',
        label: `${selectedVariant.external_name} / ${selectedVariant.provider_id}`,
        externalName: selectedVariant.external_name,
        providerId: selectedVariant.provider_id,
      });
    }
  }

  return (
    <>
      <div className="flex h-full min-h-0 flex-col gap-4 p-4 lg:p-5">
        <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
          <StatCard
            description={t('Public channels exposed by the router.')}
            label={t('Channels')}
            value={formatNumber(snapshot.channels.length)}
          />
          <StatCard
            description={t('Proxy providers bound into the catalog.')}
            label={t('Providers')}
            value={formatNumber(snapshot.providers.length)}
          />
          <StatCard
            description={t('Encrypted provider credentials on record.')}
            label={t('Credentials')}
            value={formatNumber(snapshot.credentials.length)}
          />
          <StatCard
            description={t('Provider-scoped variants available for publication.')}
            label={t('Variants')}
            value={formatNumber(snapshot.models.length)}
          />
        </div>

        <Card className="shrink-0">
          <CardContent className="p-4">
            <form
              className="flex flex-wrap items-center gap-3"
              onSubmit={(event) => event.preventDefault()}
            >
              <div className="min-w-[18rem] flex-[1.5]">
                <Label className="sr-only" htmlFor="catalog-search">
                  {t('Search catalog')}
                </Label>
                <div className="relative">
                  <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-[var(--sdk-color-text-muted)]" />
                  <Input
                    className="pl-9"
                    id="catalog-search"
                    onChange={(event) => handleSearchChange(event.target.value)}
                    placeholder={t('name, id, provider, credential')}
                    value={search}
                  />
                </div>
              </div>

              <div className="min-w-[22rem] flex-[1.1]">
                <div className="space-y-0">
                  <Label className="sr-only">{t('Catalog area')}</Label>
                  <SegmentedControl
                    onValueChange={handleCatalogLaneChange}
                    options={[
                      { label: t('Channels'), value: 'channels' },
                      { label: t('Providers'), value: 'providers' },
                      { label: t('Credentials'), value: 'credentials' },
                      { label: t('Variants'), value: 'variants' },
                    ]}
                    size="sm"
                    value={catalogLane}
                  />
                </div>
              </div>

              <div className="ml-auto flex flex-wrap items-center self-center gap-2">
                <div className="hidden text-sm text-[var(--sdk-color-text-secondary)] xl:block">
                  {t('{count} visible', {
                    count: formatNumber(
                      catalogLane === 'channels'
                        ? filteredChannels.length
                        : catalogLane === 'providers'
                          ? filteredProviders.length
                          : catalogLane === 'credentials'
                            ? filteredCredentials.length
                            : filteredVariants.length,
                    ),
                  })}
                  {' | '}
                  {catalogAreaLabel}
                </div>
                <Button onClick={clearSearch} type="button" variant="outline">
                  {t('Reset filters')}
                </Button>
                <Button onClick={handlePrimaryAction} type="button" variant="primary">
                  {primaryActionLabel}
                </Button>
              </div>
            </form>
          </CardContent>
        </Card>

        <div className="min-h-0 flex-1">
          <CatalogRegistrySection
            catalogLane={catalogLane}
            defaultChannelId={defaultChannelId}
            filteredChannels={filteredChannels}
            filteredCredentials={filteredCredentials}
            filteredProviders={filteredProviders}
            filteredVariants={filteredVariants}
            onDeleteItem={setPendingDelete}
            onEditChannel={openEditChannelDialog}
            onEditCredential={openCredentialDialog}
            onEditProvider={openEditProviderDialog}
            onOpenCredentialDialog={(providerId) => {
              openNewCredentialDialog(providerId);
            }}
            onOpenNewChannelModel={openNewChannelModelDialog}
            onOpenNewProvider={openNewProviderDialog}
            onSelectChannel={openChannelDetail}
            onSelectCredential={openCredentialDetail}
            onSelectProvider={openProviderDetail}
            onSelectVariant={openVariantDetail}
            selectedChannel={selectedChannel}
            selectedChannelId={isDetailDrawerOpen ? selectedChannel?.id ?? null : null}
            selectedCredentialKey={isDetailDrawerOpen ? selectedCredentialKey : null}
            selectedProviderId={isDetailDrawerOpen ? selectedProvider?.id ?? null : null}
            selectedVariantKey={isDetailDrawerOpen ? selectedVariantKey : null}
            snapshot={snapshot}
          />
        </div>
      </div>

      <CatalogDetailDrawer
        catalogLane={catalogLane}
        channelNameById={channelNameById}
        defaultChannelId={defaultChannelId}
        onDeleteItem={setPendingDelete}
        onDeleteSelected={requestDeleteSelected}
        onEditChannel={() => {
          if (!selectedChannel) {
            return;
          }
          handleDetailDrawerOpenChange(false);
          openEditChannelDialog(selectedChannel);
        }}
        onEditChannelModel={openEditChannelModelDialog}
        onEditModelPrice={openEditModelPriceDialog}
        onEditProvider={() => {
          if (!selectedProvider) {
            return;
          }
          handleDetailDrawerOpenChange(false);
          openEditProviderDialog(selectedProvider);
        }}
        onNewChannelModel={() => {
          if (!selectedChannel) {
            return;
          }
          handleDetailDrawerOpenChange(false);
          openNewChannelModelDialog(selectedChannel.id);
        }}
        onNewProvider={() => {
          handleDetailDrawerOpenChange(false);
          openNewProviderDialog(selectedChannel?.id ?? defaultChannelId);
        }}
        onOpenChange={handleDetailDrawerOpenChange}
        onPublishVariant={() => {
          if (!selectedVariant) {
            return;
          }
          handleDetailDrawerOpenChange(false);
          openNewChannelModelDialog(
            selectedChannel?.id ?? defaultChannelId,
            selectedVariant,
          );
        }}
        onRotateCredential={() => {
          handleDetailDrawerOpenChange(false);
          if (catalogLane === 'credentials' && selectedCredential) {
            openCredentialDialog(selectedCredential);
            return;
          }
          openNewCredentialDialog(selectedProvider?.id ?? undefined);
        }}
        onStartPricing={openNewModelPriceDialog}
        open={isDetailDrawerOpen}
        providerNameById={providerNameById}
        selectedChannel={selectedChannel}
        selectedChannelModels={selectedChannelModels}
        selectedChannelProviderCount={selectedChannelProviderCount}
        selectedCredential={selectedCredential}
        selectedModelPrices={selectedModelPrices}
        selectedProviderModels={selectedProviderModels}
        selectedProviderModelPrices={selectedProviderModelPrices}
        selectedProvider={selectedProvider}
        selectedPublication={selectedPublication}
        selectedVariant={selectedVariant}
      />

      <CatalogChannelDialog
        channelDraft={channelDraft}
        editingChannelId={editingChannelId}
        onOpenChange={setIsChannelDialogOpen}
        onSubmit={(event) => void handleChannelSubmit(event)}
        open={isChannelDialogOpen}
        setChannelDraft={setChannelDraft}
      />

      <CatalogProviderDialog
        editingProviderId={editingProviderId}
        onOpenChange={setIsProviderDialogOpen}
        onSubmit={(event) => void handleProviderSubmit(event)}
        open={isProviderDialogOpen}
        providerDraft={providerDraft}
        setProviderDraft={setProviderDraft}
        snapshot={snapshot}
      />

      <CatalogCredentialDialog
        credentialDraft={credentialDraft}
        onOpenChange={setIsCredentialDialogOpen}
        onSubmit={(event) => void handleCredentialSubmit(event)}
        open={isCredentialDialogOpen}
        setCredentialDraft={setCredentialDraft}
        snapshot={snapshot}
      />

      <CatalogChannelModelDialog
        channelModelDraft={channelModelDraft}
        editingChannelModelKey={editingChannelModelKey}
        onOpenChange={setIsChannelModelEditorOpen}
        onSubmit={(event) => void handleChannelModelSubmit(event)}
        open={isChannelModelEditorOpen}
        setChannelModelDraft={setChannelModelDraft}
        snapshot={snapshot}
      />

      <CatalogModelPriceDialog
        editingModelPriceKey={editingModelPriceKey}
        modelPriceDraft={modelPriceDraft}
        onOpenChange={setIsModelPriceEditorOpen}
        onSubmit={(event) => void handleModelPriceSubmit(event)}
        open={isModelPriceEditorOpen}
        setModelPriceDraft={setModelPriceDraft}
        snapshot={snapshot}
      />

      <ConfirmActionDialog
        confirmLabel={t('Delete')}
        description={deleteDialogDescription}
        onConfirm={() => void confirmDelete()}
        onOpenChange={handleDeleteDialogOpenChange}
        open={Boolean(pendingDelete)}
        title={t('Delete catalog item')}
      />
    </>
  );
}
