import {
  Button,
  Card,
  DataTable,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
  InlineAlert,
  Pagination,
  PaginationContent,
  PaginationItem,
  PaginationLink,
  PaginationNext,
  PaginationPrevious,
  type DataTableColumn,
} from '@sdkwork/ui-pc-react';
import { MoreHorizontal, Trash2 } from 'lucide-react';
import { useEffect, useState } from 'react';
import {
  buildEmbeddedAdminSingleSelectRowProps,
  embeddedAdminDataTableClassName,
  embeddedAdminDataTableSlotProps,
  useAdminI18n,
} from 'sdkwork-router-admin-core';
import type { ProviderCatalogRecord } from 'sdkwork-router-admin-types';

import type { GatewayRouteInventoryRow } from '../../services/gatewayViewService';

type GatewayRoutesRegistrySectionProps = {
  columns: DataTableColumn<GatewayRouteInventoryRow>[];
  degradedCount: number;
  filteredInventory: GatewayRouteInventoryRow[];
  inventory: GatewayRouteInventoryRow[];
  onDeleteProvider: (provider: ProviderCatalogRecord) => void;
  onEditProvider: (provider: ProviderCatalogRecord) => void;
  onSelectProvider: (row: GatewayRouteInventoryRow) => void;
  selectedRow: GatewayRouteInventoryRow | null;
};

export function GatewayRoutesRegistrySection({
  columns,
  degradedCount,
  filteredInventory,
  inventory,
  onDeleteProvider,
  onEditProvider,
  onSelectProvider,
  selectedRow,
}: GatewayRoutesRegistrySectionProps) {
  const { formatNumber, t } = useAdminI18n();
  const [page, setPage] = useState(1);
  const pageSize = 10;
  const total = filteredInventory.length;
  const totalPages = Math.max(1, Math.ceil(total / pageSize));
  const startIndex = (page - 1) * pageSize;
  const endIndex = startIndex + pageSize;
  const paginatedInventory = filteredInventory.slice(startIndex, endIndex);

  useEffect(() => {
    setPage(1);
  }, [filteredInventory]);

  useEffect(() => {
    if (page > totalPages) {
      setPage(totalPages);
    }
  }, [page, totalPages]);

  return (
    <Card className="h-full flex flex-col overflow-hidden p-0">
      <DataTable
        className={embeddedAdminDataTableClassName}
        columns={columns}
        emptyDescription={t('Try a broader search or reset the channel and health filters.')}
        emptyTitle={t('No route providers match the current filter')}
        getRowId={(row: GatewayRouteInventoryRow) => row.provider.id}
        getRowProps={buildEmbeddedAdminSingleSelectRowProps(
          selectedRow?.provider.id ?? null,
          (row: GatewayRouteInventoryRow) => row.provider.id,
        )}
        onRowClick={onSelectProvider}
        slotProps={embeddedAdminDataTableSlotProps}
        rowActions={(row: GatewayRouteInventoryRow) => (
          <div className="flex items-center justify-end gap-2">
            <Button
              onClick={(event) => {
                event.stopPropagation();
                onEditProvider(row.provider);
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
                  onClick={() => onDeleteProvider(row.provider)}
                >
                  <Trash2 className="w-3.5 h-3.5 mr-2" />
                  {t('Delete')}
                </DropdownMenuItem>
              </DropdownMenuContent>
            </DropdownMenu>
          </div>
        )}
        rows={paginatedInventory}
        stickyHeader
      />
      <div className="flex flex-col gap-3 border-t border-[var(--sdk-color-border-default)] p-4">
        <div className="flex flex-wrap items-center justify-between gap-3">
          <div className="flex flex-wrap items-center gap-x-4 gap-y-1 text-sm text-[var(--sdk-color-text-secondary)]">
            <span>{t('{count} providers', { count: formatNumber(inventory.length) })}</span>
            <span>{t('{count} degraded', { count: formatNumber(degradedCount) })}</span>
            <span>
              {t('{count} credentials', {
                count: formatNumber(inventory.reduce((sum, row) => sum + row.credentials.length, 0)),
              })}
            </span>
            <span>
              {t('{count} pricing rows', {
                count: formatNumber(inventory.reduce((sum, row) => sum + row.price_count, 0)),
              })}
            </span>
          </div>
          <div className="text-xs uppercase tracking-[0.18em] text-[var(--sdk-color-text-muted)]">
            {t('Page {page} of {totalPages}', {
              page: formatNumber(page),
              totalPages: formatNumber(totalPages),
            })}
          </div>
        </div>
        {degradedCount > 0 ? (
          <InlineAlert
            description={t(
              'Review the degraded providers in the table and use the inspector rail to understand channel and credential impact before editing.',
            )}
            title={t(
              degradedCount === 1
                ? '{count} provider needs attention'
                : '{count} providers need attention',
              { count: formatNumber(degradedCount) },
            )}
            tone="warning"
          />
        ) : null}
        {total > 0 ? (
          <div className="flex flex-wrap items-center justify-between gap-3">
            <div className="text-sm text-[var(--sdk-color-text-secondary)]">
              {t('Showing {start} - {end} of {total}', {
                start: total === 0 ? 0 : formatNumber(startIndex + 1),
                end: formatNumber(Math.min(endIndex, total)),
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
