import { useEffect, useState } from 'react';
import {
  Button,
  Card,
  DataTable,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
  Pagination,
  PaginationContent,
  PaginationItem,
  PaginationLink,
  PaginationNext,
  PaginationPrevious,
  StatusBadge,
  type DataTableColumn,
} from '@sdkwork/ui-pc-react';
import { MoreHorizontal, Trash2 } from 'lucide-react';
import {
  buildEmbeddedAdminSingleSelectRowProps,
  describeProviderIntegration,
  embeddedAdminDataTableClassName,
  embeddedAdminDataTableSlotProps,
  summarizeProviderPricingCoverage,
  useAdminI18n,
} from 'sdkwork-router-admin-core';
import type {
  AdminPageProps,
  CredentialRecord,
  ProviderCatalogRecord,
} from 'sdkwork-router-admin-types';

import {
  providerChannelIds,
  type CatalogLane,
  type ChannelRecord,
  type PendingDelete,
  type VariantRecord,
} from './shared';

type CatalogRegistrySectionProps = {
  catalogLane: CatalogLane;
  defaultChannelId: string;
  filteredChannels: ChannelRecord[];
  filteredCredentials: CredentialRecord[];
  filteredProviders: ProviderCatalogRecord[];
  filteredVariants: VariantRecord[];
  onDeleteItem: (deleteTarget: NonNullable<PendingDelete>) => void;
  onEditChannel: (channel: ChannelRecord) => void;
  onEditCredential: (record: CredentialRecord) => void;
  onEditProvider: (provider: ProviderCatalogRecord) => void;
  onOpenCredentialDialog: (providerId?: string) => void;
  onOpenNewChannelModel: (channelId: string, variant?: VariantRecord) => void;
  onOpenNewProvider: (channelId?: string) => void;
  onSelectChannel: (channelId: string) => void;
  onSelectCredential: (key: string) => void;
  onSelectProvider: (providerId: string) => void;
  onSelectVariant: (key: string) => void;
  selectedChannel: ChannelRecord | null;
  selectedChannelId: string | null;
  selectedCredentialKey: string | null;
  selectedProviderId: string | null;
  selectedVariantKey: string | null;
  snapshot: AdminPageProps['snapshot'];
};

export function CatalogRegistrySection({
  catalogLane,
  defaultChannelId,
  filteredChannels,
  filteredCredentials,
  filteredProviders,
  filteredVariants,
  onDeleteItem,
  onEditChannel,
  onEditCredential,
  onEditProvider,
  onOpenCredentialDialog,
  onOpenNewChannelModel,
  onOpenNewProvider,
  onSelectChannel,
  onSelectCredential,
  onSelectProvider,
  onSelectVariant,
  selectedChannel,
  selectedChannelId,
  selectedCredentialKey,
  selectedProviderId,
  selectedVariantKey,
  snapshot,
}: CatalogRegistrySectionProps) {
  const { formatNumber, t } = useAdminI18n();
  const [page, setPage] = useState(1);
  const pageSize = 10;

  const channelNameById = new Map(
    snapshot.channels.map((channel) => [channel.id, channel.name]),
  );
  const providerNameById = new Map(
    snapshot.providers.map((provider) => [provider.id, provider.display_name]),
  );

  const channelColumns: DataTableColumn<ChannelRecord>[] = [
    {
      id: 'channel',
      header: t('Channel'),
      cell: (row) => (
        <div className="space-y-1">
          <div className="font-medium text-[var(--sdk-color-text-primary)]">{row.name}</div>
          <div className="text-sm text-[var(--sdk-color-text-secondary)]">{row.id}</div>
        </div>
      ),
    },
    {
      id: 'models',
      align: 'right',
      header: t('Published models'),
      cell: (row) =>
        formatNumber(
          snapshot.channelModels.filter((model) => model.channel_id === row.id).length,
        ),
      width: 140,
    },
    {
      id: 'providers',
      align: 'right',
      header: t('Providers'),
      cell: (row) =>
        formatNumber(
          snapshot.providers.filter((provider) =>
            providerChannelIds(provider).includes(row.id),
          ).length,
        ),
      width: 120,
    },
  ];

  const providerColumns: DataTableColumn<ProviderCatalogRecord>[] = [
    {
      id: 'provider',
      header: t('Provider'),
      cell: (row) => (
        <div className="space-y-1">
          <div className="font-medium text-[var(--sdk-color-text-primary)]">
            {row.display_name}
          </div>
          <div className="text-sm text-[var(--sdk-color-text-secondary)]">
            {row.id} / {describeProviderIntegration(row)}
          </div>
        </div>
      ),
    },
    {
      id: 'channel',
      header: t('Primary channel'),
      cell: (row) => channelNameById.get(row.channel_id) ?? row.channel_id,
    },
    {
      id: 'adapter',
      header: t('Adapter'),
      cell: (row) => describeProviderIntegration(row),
      width: 220,
    },
    {
      id: 'bindings',
      align: 'right',
      header: t('Bound channels'),
      cell: (row) => formatNumber(providerChannelIds(row).length),
      width: 140,
    },
    {
      id: 'models',
      align: 'right',
      header: t('Supported models'),
      cell: (row) =>
        formatNumber(
          snapshot.providerModels.filter(
            (record) => record.proxy_provider_id === row.id && record.is_active,
          ).length,
      ),
      width: 150,
    },
    {
      id: 'pricing',
      header: t('Pricing coverage'),
      cell: (row) => {
        const summary = summarizeProviderPricingCoverage(
          row.id,
          snapshot.providerModels,
          snapshot.modelPrices,
        );
        if (summary.active_model_count === 0) {
          return t('0 / 0 priced');
        }
        if (summary.missing_price_count > 0) {
          return t('{priced} / {active} priced · {missing} missing', {
            priced: summary.priced_model_count,
            active: summary.active_model_count,
            missing: summary.missing_price_count,
          });
        }
        return t('{priced} / {active} priced', {
          priced: summary.priced_model_count,
          active: summary.active_model_count,
        });
      },
      width: 220,
    },
  ];

  const credentialColumns: DataTableColumn<CredentialRecord>[] = [
    {
      id: 'reference',
      header: t('Key reference'),
      cell: (row) => (
        <div className="space-y-1">
          <div className="font-medium text-[var(--sdk-color-text-primary)]">
            {row.key_reference}
          </div>
          <div className="text-sm text-[var(--sdk-color-text-secondary)]">
            {row.tenant_id}
          </div>
        </div>
      ),
    },
    {
      id: 'provider',
      header: t('Provider'),
      cell: (row) => providerNameById.get(row.provider_id) ?? row.provider_id,
    },
    {
      id: 'backend',
      header: t('Backend'),
      cell: (row) => row.secret_backend,
      width: 180,
    },
  ];

  const variantColumns: DataTableColumn<VariantRecord>[] = [
    {
      id: 'model',
      header: t('Model'),
      cell: (row) => (
        <div className="space-y-1">
          <div className="font-medium text-[var(--sdk-color-text-primary)]">
            {row.external_name}
          </div>
          <div className="text-sm text-[var(--sdk-color-text-secondary)]">
            {providerNameById.get(row.provider_id) ?? row.provider_id}
          </div>
        </div>
      ),
    },
    {
      id: 'capabilities',
      header: t('Capabilities'),
      cell: (row) => row.capabilities.join(', ') || '-',
    },
    {
      id: 'streaming',
      header: t('Streaming'),
      cell: (row) => (
        <StatusBadge
          label={row.streaming ? t('Enabled') : t('Disabled')}
          showIcon
          status={row.streaming ? 'active' : 'disabled'}
          variant={row.streaming ? 'success' : 'secondary'}
        />
      ),
      width: 140,
    },
  ];

  const total =
    catalogLane === 'channels'
      ? filteredChannels.length
      : catalogLane === 'providers'
        ? filteredProviders.length
        : catalogLane === 'credentials'
          ? filteredCredentials.length
          : filteredVariants.length;
  const totalPages = Math.max(1, Math.ceil(total / pageSize));
  const startIndex = (page - 1) * pageSize;
  const endIndex = startIndex + pageSize;
  const laneLabel =
    catalogLane === 'channels'
      ? t('Channels')
      : catalogLane === 'providers'
        ? t('Providers')
        : catalogLane === 'credentials'
          ? t('Credentials')
          : t('Variants');

  useEffect(() => {
    setPage(1);
  }, [catalogLane, filteredChannels, filteredCredentials, filteredProviders, filteredVariants]);

  useEffect(() => {
    if (page > totalPages) {
      setPage(totalPages);
    }
  }, [page, totalPages]);

  return (
    <Card className="h-full flex flex-col overflow-hidden p-0">
      {catalogLane === 'channels' ? (
        <DataTable
          className={embeddedAdminDataTableClassName}
          columns={channelColumns}
          emptyDescription={t('Try a broader query or add a new channel.')}
          emptyTitle={t('No channels match the search')}
          getRowId={(row: ChannelRecord) => row.id}
          getRowProps={buildEmbeddedAdminSingleSelectRowProps(
            selectedChannelId,
            (row: ChannelRecord) => row.id,
          )}
          onRowClick={(row: ChannelRecord) => onSelectChannel(row.id)}
          slotProps={embeddedAdminDataTableSlotProps}
          rowActions={(row: ChannelRecord) => (
            <div className="flex items-center justify-end gap-2">
              <Button
                onClick={(event) => {
                  event.stopPropagation();
                  onEditChannel(row);
                }}
                size="sm"
                type="button"
                variant="ghost"
              >
                {t('Edit')}
              </Button>
              <Button
                onClick={(event) => {
                  event.stopPropagation();
                  onOpenNewChannelModel(row.id);
                }}
                size="sm"
                type="button"
                variant="outline"
              >
                {t('New model')}
              </Button>
              <DropdownMenu>
                <DropdownMenuTrigger asChild>
                  <Button size="sm" type="button" variant="ghost">
                    <MoreHorizontal className="w-4 h-4" />
                  </Button>
                </DropdownMenuTrigger>
                <DropdownMenuContent align="end">
                  <DropdownMenuItem onClick={() => onOpenNewProvider(row.id)}>
                    {t('New provider')}
                  </DropdownMenuItem>
                  <DropdownMenuItem
                    className="text-[var(--sdk-color-state-danger)]"
                    onClick={() =>
                      onDeleteItem({
                        kind: 'channel',
                        label: `${row.name} (${row.id})`,
                        channelId: row.id,
                      })
                    }
                  >
                    <Trash2 className="w-3.5 h-3.5 mr-2" />
                    {t('Delete')}
                  </DropdownMenuItem>
                </DropdownMenuContent>
              </DropdownMenu>
            </div>
          )}
          rows={filteredChannels.slice(startIndex, endIndex)}
          stickyHeader
        />
      ) : null}

      {catalogLane === 'providers' ? (
        <DataTable
          className={embeddedAdminDataTableClassName}
          columns={providerColumns}
          emptyDescription={t('Try a broader query or add a new provider.')}
          emptyTitle={t('No providers match the search')}
          getRowId={(row: ProviderCatalogRecord) => row.id}
          getRowProps={buildEmbeddedAdminSingleSelectRowProps(
            selectedProviderId,
            (row: ProviderCatalogRecord) => row.id,
          )}
          onRowClick={(row: ProviderCatalogRecord) => onSelectProvider(row.id)}
          slotProps={embeddedAdminDataTableSlotProps}
          rowActions={(row: ProviderCatalogRecord) => (
            <div className="flex items-center justify-end gap-2">
              <Button
                onClick={(event) => {
                  event.stopPropagation();
                  onEditProvider(row);
                }}
                size="sm"
                type="button"
                variant="ghost"
              >
                {t('Edit')}
              </Button>
              <Button
                onClick={(event) => {
                  event.stopPropagation();
                  onOpenCredentialDialog(row.id);
                }}
                size="sm"
                type="button"
                variant="outline"
              >
                {t('Rotate credential')}
              </Button>
              <DropdownMenu>
                <DropdownMenuTrigger asChild>
                  <Button size="sm" type="button" variant="ghost">
                    <MoreHorizontal className="w-4 h-4" />
                  </Button>
                </DropdownMenuTrigger>
                <DropdownMenuContent align="end">
                  <DropdownMenuItem
                    className="text-[var(--sdk-color-state-danger)]"
                    onClick={() =>
                      onDeleteItem({
                        kind: 'provider',
                        label: `${row.display_name} (${row.id})`,
                        providerId: row.id,
                      })
                    }
                  >
                    <Trash2 className="w-3.5 h-3.5 mr-2" />
                    {t('Delete')}
                  </DropdownMenuItem>
                </DropdownMenuContent>
              </DropdownMenu>
            </div>
          )}
          rows={filteredProviders.slice(startIndex, endIndex)}
          stickyHeader
        />
      ) : null}

      {catalogLane === 'credentials' ? (
        <DataTable
          className={embeddedAdminDataTableClassName}
          columns={credentialColumns}
          emptyDescription={t('Try a broader query or add a new credential.')}
          emptyTitle={t('No credentials match the search')}
          getRowId={(row: CredentialRecord) =>
            `${row.tenant_id}:${row.provider_id}:${row.key_reference}`
          }
          getRowProps={buildEmbeddedAdminSingleSelectRowProps(
            selectedCredentialKey,
            (row: CredentialRecord) =>
              `${row.tenant_id}:${row.provider_id}:${row.key_reference}`,
          )}
          onRowClick={(row: CredentialRecord) =>
            onSelectCredential(
              `${row.tenant_id}:${row.provider_id}:${row.key_reference}`,
            )
          }
          slotProps={embeddedAdminDataTableSlotProps}
          rowActions={(row: CredentialRecord) => (
            <div className="flex items-center justify-end gap-2">
              <Button
                onClick={(event) => {
                  event.stopPropagation();
                  onEditCredential(row);
                }}
                size="sm"
                type="button"
                variant="ghost"
              >
                {t('Edit')}
              </Button>
              <DropdownMenu>
                <DropdownMenuTrigger asChild>
                  <Button size="sm" type="button" variant="ghost">
                    <MoreHorizontal className="w-4 h-4" />
                  </Button>
                </DropdownMenuTrigger>
                <DropdownMenuContent align="end">
                  <DropdownMenuItem
                    className="text-[var(--sdk-color-state-danger)]"
                    onClick={() =>
                      onDeleteItem({
                        kind: 'credential',
                        label: row.key_reference,
                        tenantId: row.tenant_id,
                        providerId: row.provider_id,
                        keyReference: row.key_reference,
                      })
                    }
                  >
                    <Trash2 className="w-3.5 h-3.5 mr-2" />
                    {t('Delete')}
                  </DropdownMenuItem>
                </DropdownMenuContent>
              </DropdownMenu>
            </div>
          )}
          rows={filteredCredentials.slice(startIndex, endIndex)}
          stickyHeader
        />
      ) : null}

      {catalogLane === 'variants' ? (
        <DataTable
          className={embeddedAdminDataTableClassName}
          columns={variantColumns}
          emptyDescription={t('Try a broader query or publish a new provider variant upstream.')}
          emptyTitle={t('No variants match the search')}
          getRowId={(row: VariantRecord) => `${row.provider_id}:${row.external_name}`}
          getRowProps={buildEmbeddedAdminSingleSelectRowProps(
            selectedVariantKey,
            (row: VariantRecord) => `${row.provider_id}:${row.external_name}`,
          )}
          onRowClick={(row: VariantRecord) =>
            onSelectVariant(`${row.provider_id}:${row.external_name}`)
          }
          slotProps={embeddedAdminDataTableSlotProps}
          rowActions={(row: VariantRecord) => (
            <div className="flex items-center justify-end gap-2">
              <Button
                onClick={(event) => {
                  event.stopPropagation();
                  onOpenNewChannelModel(selectedChannel?.id ?? defaultChannelId, row);
                }}
                size="sm"
                type="button"
                variant="outline"
              >
                {t('Publish')}
              </Button>
              <DropdownMenu>
                <DropdownMenuTrigger asChild>
                  <Button size="sm" type="button" variant="ghost">
                    <MoreHorizontal className="w-4 h-4" />
                  </Button>
                </DropdownMenuTrigger>
                <DropdownMenuContent align="end">
                  <DropdownMenuItem
                    className="text-[var(--sdk-color-state-danger)]"
                    onClick={() =>
                      onDeleteItem({
                        kind: 'model',
                        label: `${row.external_name} / ${row.provider_id}`,
                        externalName: row.external_name,
                        providerId: row.provider_id,
                      })
                    }
                  >
                    <Trash2 className="w-3.5 h-3.5 mr-2" />
                    {t('Delete')}
                  </DropdownMenuItem>
                </DropdownMenuContent>
              </DropdownMenu>
            </div>
          )}
          rows={filteredVariants.slice(startIndex, endIndex)}
          stickyHeader
        />
      ) : null}

      <div className="flex flex-col gap-3 border-t border-[var(--sdk-color-border-default)] p-4">
        <div className="flex flex-wrap items-center justify-between gap-3">
          <div className="flex flex-wrap items-center gap-x-4 gap-y-1 text-sm text-[var(--sdk-color-text-secondary)]">
            <span>{t('{count} visible', { count: formatNumber(total) })}</span>
            <span>{laneLabel}</span>
          </div>
          <div className="text-xs uppercase tracking-[0.18em] text-[var(--sdk-color-text-muted)]">
            {t('Page {page} of {total}', {
              page: formatNumber(page),
              total: formatNumber(totalPages),
            })}
          </div>
        </div>
        {total > 0 ? (
          <div className="flex flex-wrap items-center justify-between gap-3">
            <div className="text-sm text-[var(--sdk-color-text-secondary)]">
              {t('Showing {start} - {end} of {total}', {
                end: formatNumber(Math.min(endIndex, total)),
                start: formatNumber(total === 0 ? 0 : startIndex + 1),
                total: formatNumber(total),
              })}
            </div>
            <Pagination>
              <PaginationContent>
                <PaginationItem>
                  <PaginationPrevious
                    className={page <= 1 ? 'pointer-events-none opacity-50' : 'cursor-pointer'}
                    onClick={() => setPage((current) => Math.max(1, current - 1))}
                  />
                </PaginationItem>
                {Array.from({ length: Math.min(5, totalPages) }, (_, index) => {
                  let pageNumber: number;

                  if (totalPages <= 5) {
                    pageNumber = index + 1;
                  } else if (page <= 3) {
                    pageNumber = index + 1;
                  } else if (page >= totalPages - 2) {
                    pageNumber = totalPages - 4 + index;
                  } else {
                    pageNumber = page - 2 + index;
                  }

                  return (
                    <PaginationItem key={pageNumber}>
                      <PaginationLink
                        className="cursor-pointer"
                        isActive={page === pageNumber}
                        onClick={() => setPage(pageNumber)}
                      >
                        {pageNumber}
                      </PaginationLink>
                    </PaginationItem>
                  );
                })}
                <PaginationItem>
                  <PaginationNext
                    className={page >= totalPages ? 'pointer-events-none opacity-50' : 'cursor-pointer'}
                    onClick={() => setPage((current) => Math.min(totalPages, current + 1))}
                  />
                </PaginationItem>
              </PaginationContent>
            </Pagination>
          </div>
        ) : null}
      </div>
    </Card>
  );
}
