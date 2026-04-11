import {
  Card,
  DataTable,
  Pagination,
  PaginationContent,
  PaginationItem,
  PaginationLink,
  PaginationNext,
  PaginationPrevious,
  type DataTableColumn,
} from '@sdkwork/ui-pc-react';
import { useEffect, useState } from 'react';
import {
  buildEmbeddedAdminSingleSelectRowProps,
  embeddedAdminDataTableClassName,
  embeddedAdminDataTableSlotProps,
  useAdminI18n,
} from 'sdkwork-router-admin-core';
import type { CouponRecord } from 'sdkwork-router-admin-types';

type CouponsRegistrySectionProps = {
  activeCoupons: CouponRecord[];
  archivedCoupons: CouponRecord[];
  atRiskCoupons: CouponRecord[];
  columns: DataTableColumn<CouponRecord>[];
  coveredAudiencesCount: number;
  expiringSoonCoupons: CouponRecord[];
  filteredCoupons: CouponRecord[];
  nextExpiringCoupon: { coupon: CouponRecord; days: number | null } | null;
  onSelectCoupon: (coupon: CouponRecord) => void;
  remainingQuota: number;
  selectedCouponId: string | null;
};

export function CouponsRegistrySection({
  activeCoupons,
  archivedCoupons,
  atRiskCoupons,
  columns,
  coveredAudiencesCount,
  expiringSoonCoupons,
  filteredCoupons,
  nextExpiringCoupon,
  onSelectCoupon,
  remainingQuota,
  selectedCouponId,
}: CouponsRegistrySectionProps) {
  const { formatNumber, t } = useAdminI18n();
  const [page, setPage] = useState(1);
  const pageSize = 10;
  const total = filteredCoupons.length;
  const totalPages = Math.max(1, Math.ceil(total / pageSize));
  const startIndex = (page - 1) * pageSize;
  const endIndex = startIndex + pageSize;
  const paginatedCoupons = filteredCoupons.slice(startIndex, endIndex);

  useEffect(() => {
    setPage(1);
  }, [filteredCoupons]);

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
        emptyDescription={t('Try a different campaign state or broaden the search query.')}
        emptyTitle={t('No coupons match the current filter')}
        getRowId={(coupon: CouponRecord) => coupon.id}
        getRowProps={buildEmbeddedAdminSingleSelectRowProps(
          selectedCouponId,
          (coupon: CouponRecord) => coupon.id,
        )}
        onRowClick={onSelectCoupon}
        slotProps={embeddedAdminDataTableSlotProps}
        rows={paginatedCoupons}
        stickyHeader
      />
      <div className="flex flex-col gap-3 border-t border-[var(--sdk-color-border-default)] p-4">
        <div className="flex flex-wrap items-center justify-between gap-3">
          <div className="flex flex-wrap items-center gap-x-4 gap-y-1 text-sm text-[var(--sdk-color-text-secondary)]">
            <span>{t('{count} campaigns', { count: formatNumber(total) })}</span>
            <span>{t('{count} live', { count: formatNumber(activeCoupons.length) })}</span>
            <span>{t('{count} at risk', { count: formatNumber(atRiskCoupons.length) })}</span>
            <span>{t('{count} audiences', { count: formatNumber(coveredAudiencesCount) })}</span>
            <span>{t('{count} quota', { count: formatNumber(remainingQuota) })}</span>
          </div>
          <div className="text-xs uppercase tracking-[0.18em] text-[var(--sdk-color-text-muted)]">
            {nextExpiringCoupon
              ? t('{code} expires next', { code: nextExpiringCoupon.coupon.code })
              : t('Page {page} of {total}', {
                  page: formatNumber(page),
                  total: formatNumber(totalPages),
                })}
          </div>
        </div>
        {total > 0 ? (
          <div className="flex flex-wrap items-center justify-between gap-3">
            <div className="text-sm text-[var(--sdk-color-text-secondary)]">
              {t(
                'Showing {start} - {end} of {total} | {archived} archived | {expiringSoon} expiring soon',
                {
                  archived: formatNumber(archivedCoupons.length),
                  end: formatNumber(Math.min(endIndex, total)),
                  expiringSoon: formatNumber(expiringSoonCoupons.length),
                  start: formatNumber(total === 0 ? 0 : startIndex + 1),
                  total: formatNumber(total),
                },
              )}
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
