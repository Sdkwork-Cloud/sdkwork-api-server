import {
  Badge,
  Button,
  Drawer,
  DrawerBody,
  DrawerContent,
  DrawerDescription,
  DrawerFooter,
  DrawerHeader,
  DrawerTitle,
} from '@sdkwork/ui-pc-react';
import { Edit, Plus, Trash2 } from 'lucide-react';
import { useAdminI18n } from 'sdkwork-router-admin-core';
import type {
  ChannelModelRecord,
  CredentialRecord,
  ModelPriceRecord,
  ProviderCatalogRecord,
  ProviderModelRecord,
} from 'sdkwork-router-admin-types';

import {
  type PendingDelete,
  providerChannelIds,
  type CatalogLane,
  type ChannelRecord,
  type VariantRecord,
} from './shared';
import { CatalogDetailPanel } from './CatalogDetailPanel';

type CatalogDetailDrawerProps = {
  catalogLane: CatalogLane;
  channelNameById: Map<string, string>;
  defaultChannelId: string;
  onDeleteItem: (deleteTarget: NonNullable<PendingDelete>) => void;
  onDeleteSelected: () => void;
  onEditChannel: () => void;
  onEditChannelModel: (record: ChannelModelRecord) => void;
  onEditModelPrice: (record: ModelPriceRecord) => void;
  onEditProvider: () => void;
  onNewChannelModel: () => void;
  onNewProvider: () => void;
  onOpenChange: (open: boolean) => void;
  onPublishVariant: () => void;
  onRotateCredential: () => void;
  onStartPricing: (
    record: ChannelModelRecord,
    options?: {
      proxyProviderId?: string;
      priceSourceKind?: string;
    },
  ) => void;
  open: boolean;
  providerNameById: Map<string, string>;
  selectedChannel: ChannelRecord | null;
  selectedChannelModels: ChannelModelRecord[];
  selectedChannelProviderCount: number;
  selectedCredential: CredentialRecord | null;
  selectedModelPrices: ModelPriceRecord[];
  selectedProviderModels: ProviderModelRecord[];
  selectedProviderModelPrices: ModelPriceRecord[];
  selectedProvider: ProviderCatalogRecord | null;
  selectedPublication: ChannelModelRecord | null;
  selectedVariant: VariantRecord | null;
};

export function CatalogDetailDrawer({
  catalogLane,
  channelNameById,
  defaultChannelId,
  onDeleteItem,
  onDeleteSelected,
  onEditChannel,
  onEditChannelModel,
  onEditModelPrice,
  onEditProvider,
  onNewChannelModel,
  onNewProvider,
  onOpenChange,
  onPublishVariant,
  onRotateCredential,
  onStartPricing,
  open,
  providerNameById,
  selectedChannel,
  selectedChannelModels,
  selectedChannelProviderCount,
  selectedCredential,
  selectedModelPrices,
  selectedProviderModels,
  selectedProviderModelPrices,
  selectedProvider,
  selectedPublication,
  selectedVariant,
}: CatalogDetailDrawerProps) {
  const { t } = useAdminI18n();

  const detailPanel = (
    <CatalogDetailPanel
      catalogLane={catalogLane}
      channelNameById={channelNameById}
      defaultChannelId={defaultChannelId}
      onDeleteItem={onDeleteItem}
      onEditChannelModel={onEditChannelModel}
      onEditModelPrice={onEditModelPrice}
      onPublishVariant={(_channelId, _variant) => onPublishVariant()}
      onStartPricing={onStartPricing}
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
  );

  let title: string | null = null;
  let description: string | null = null;
  let footerCopy: string | null = null;
  let badges: Array<{ label: string; variant: 'secondary' | 'outline' }> = [];

  if (catalogLane === 'channels' && selectedChannel) {
    title = selectedChannel.name;
    description = t('Channel publications, provider coverage, and pricing stay attached to the selected directory record.');
    footerCopy = t('Channel operations stay table-first while publication and pricing work remains attached to the selected drawer context.');
    badges = [
      { label: t('Channel'), variant: 'secondary' },
      { label: selectedChannel.id, variant: 'outline' },
      {
        label: t('{count} publications', { count: selectedChannelModels.length }),
        variant: 'outline',
      },
    ];
  } else if (catalogLane === 'providers' && selectedProvider) {
    title = selectedProvider.display_name;
    description = selectedProvider.base_url;
    footerCopy = t('Provider posture stays connected to channel bindings, adapter configuration, and credential rotation.');
    badges = [
      { label: t('Provider'), variant: 'secondary' },
      { label: selectedProvider.id, variant: 'outline' },
      {
        label: t('{count} channels', {
          count: providerChannelIds(selectedProvider).length,
        }),
        variant: 'outline',
      },
    ];
  } else if (catalogLane === 'credentials' && selectedCredential) {
    title = selectedCredential.key_reference;
    description = t('{tenant} / {provider}', {
      tenant: selectedCredential.tenant_id,
      provider:
        providerNameById.get(selectedCredential.provider_id)
        ?? selectedCredential.provider_id,
    });
    footerCopy = t('Credential lifecycle remains scoped to the selected provider reference without leaving the directory workflow.');
    badges = [
      { label: t('Credential'), variant: 'secondary' },
      { label: selectedCredential.secret_backend, variant: 'outline' },
      { label: selectedCredential.provider_id, variant: 'outline' },
    ];
  } else if (catalogLane === 'variants' && selectedVariant) {
    title = selectedVariant.external_name;
    description =
      providerNameById.get(selectedVariant.provider_id) ?? selectedVariant.provider_id;
    footerCopy = t('Provider variants can be inspected and published into the active channel without leaving the directory table.');
    badges = [
      { label: t('Variant'), variant: 'secondary' },
      { label: selectedVariant.provider_id, variant: 'outline' },
      {
        label: selectedVariant.streaming ? t('Streaming') : t('Non-streaming'),
        variant: 'outline',
      },
    ];
  }

  return (
    <Drawer open={open} onOpenChange={onOpenChange}>
      <DrawerContent side="right" size="xl">
        {title ? (
          <>
            <DrawerHeader>
              <div className="space-y-3">
                <div className="flex flex-wrap items-start justify-between gap-3">
                  <div className="space-y-1">
                    <DrawerTitle>{title}</DrawerTitle>
                    <DrawerDescription>{description}</DrawerDescription>
                  </div>
                  <div className="flex flex-wrap gap-2">
                    {badges.map((badge) => (
                      <Badge key={`${badge.variant}:${badge.label}`} variant={badge.variant}>
                        {badge.label}
                      </Badge>
                    ))}
                  </div>
                </div>
              </div>
            </DrawerHeader>

            <DrawerBody className="space-y-4">{detailPanel}</DrawerBody>

            <DrawerFooter className="flex flex-wrap items-center justify-between gap-3">
              <div className="text-xs text-[var(--sdk-color-text-secondary)]">
                {footerCopy}
              </div>
              <div className="flex flex-wrap items-center gap-2">
                {catalogLane === 'channels' && selectedChannel ? (
                  <>
                    <Button onClick={onEditChannel} size="sm" type="button" variant="outline">
                      <Edit className="h-4 w-4" />
                      {t('Edit')}
                    </Button>
                    <Button onClick={onNewChannelModel} size="sm" type="button" variant="primary">
                      <Plus className="h-4 w-4" />
                      {t('New model')}
                    </Button>
                    <Button onClick={onNewProvider} size="sm" type="button" variant="outline">
                      <Plus className="h-4 w-4" />
                      {t('New provider')}
                    </Button>
                  </>
                ) : null}

                {catalogLane === 'providers' && selectedProvider ? (
                  <>
                    <Button onClick={onEditProvider} size="sm" type="button" variant="outline">
                      <Edit className="h-4 w-4" />
                      {t('Edit')}
                    </Button>
                    <Button
                      onClick={onRotateCredential}
                      size="sm"
                      type="button"
                      variant="primary"
                    >
                      <Plus className="h-4 w-4" />
                      {t('Rotate credential')}
                    </Button>
                  </>
                ) : null}

                {catalogLane === 'credentials' && selectedCredential ? (
                  <Button
                    onClick={onRotateCredential}
                    size="sm"
                    type="button"
                    variant="primary"
                  >
                    <Plus className="h-4 w-4" />
                    {t('Rotate credential')}
                  </Button>
                ) : null}

                {catalogLane === 'variants' && selectedVariant ? (
                  <Button
                    onClick={onPublishVariant}
                    size="sm"
                    type="button"
                    variant="primary"
                  >
                    <Plus className="h-4 w-4" />
                    {t('Publish to channel')}
                  </Button>
                ) : null}

                <Button onClick={onDeleteSelected} size="sm" type="button" variant="danger">
                  <Trash2 className="h-4 w-4" />
                  {t('Delete')}
                </Button>
              </div>
            </DrawerFooter>
          </>
        ) : null}
      </DrawerContent>
    </Drawer>
  );
}
