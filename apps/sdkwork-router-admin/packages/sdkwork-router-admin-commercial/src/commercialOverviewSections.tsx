import type { ChangeEvent, FormEvent, ReactNode } from 'react';
import {
  Button,
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
  DataTable,
  Input,
  Label,
  ManagementWorkbench,
  StatCard,
  StatusBadge,
  type DataTableColumn,
} from '@sdkwork/ui-pc-react';
import { embeddedAdminDataTableClassName, embeddedAdminDataTableSlotProps, formatAdminDateTime, useAdminI18n } from 'sdkwork-router-admin-core';
import type { CommercialRequestSettlementRecord } from 'sdkwork-router-admin-types';
import type { CommercialLedgerTimelineRow } from './ledgerTimeline';
import type { CommercialOrderPaymentAuditRow } from './orderPaymentAudit';
import type { CommercialFact, CommercialSummaryMetric } from './formatters';
import { formatLedgerEntryTypeLabel, formatOrderAuditEventLabel, formatStatusLabel } from './formatters';

type CommercialDashboardMainProps = {
  summaryCards: CommercialSummaryMetric[];
  accountFacts: CommercialFact[];
  settlementFacts: CommercialFact[];
  pricingFacts: CommercialFact[];
  settlementLedgerColumns: Array<DataTableColumn<CommercialLedgerTimelineRow>>;
  refundTimelineColumns: Array<DataTableColumn<CommercialLedgerTimelineRow>>;
  orderPaymentAuditColumns: Array<DataTableColumn<CommercialOrderPaymentAuditRow>>;
  orderRefundAuditColumns: Array<DataTableColumn<CommercialOrderPaymentAuditRow>>;
  recentLedgerTimeline: CommercialLedgerTimelineRow[];
  refundTimelineRows: CommercialLedgerTimelineRow[];
  recentOrderPaymentAuditRows: CommercialOrderPaymentAuditRow[];
  refundAuditRows: CommercialOrderPaymentAuditRow[];
  orderAuditLookupLabel: string;
  orderAuditLookupValue: string;
  orderAuditLookupError: string | null;
  orderAuditLookupHint: string;
  onOrderAuditLookupChange: (value: string) => void;
  onOrderAuditLookupSubmit: (event: FormEvent<HTMLFormElement>) => void;
  latestSettlements: CommercialRequestSettlementRecord[];
  formatCurrency: (value: number) => string;
  formatNumber: (value: number) => string;
  supplementarySections?: ReactNode;
};

type CommercialLatestSettlementsRailProps = {
  latestSettlements: CommercialRequestSettlementRecord[];
  formatCurrency: (value: number) => string;
  formatNumber: (value: number) => string;
};

export function CommercialDashboardMain({
  summaryCards,
  accountFacts,
  settlementFacts,
  pricingFacts,
  settlementLedgerColumns,
  refundTimelineColumns,
  orderPaymentAuditColumns,
  orderRefundAuditColumns,
  recentLedgerTimeline,
  refundTimelineRows,
  recentOrderPaymentAuditRows,
  refundAuditRows,
  orderAuditLookupLabel,
  orderAuditLookupValue,
  orderAuditLookupError,
  orderAuditLookupHint,
  onOrderAuditLookupChange,
  onOrderAuditLookupSubmit,
  latestSettlements,
  formatCurrency,
  formatNumber,
  supplementarySections,
}: CommercialDashboardMainProps) {
  const { t } = useAdminI18n();

  return (
    <ManagementWorkbench
      description={t('Commercial accounts, settlement explorer, and pricing governance now live as a first-class admin module.')}
      eyebrow={t('Revenue')}
      main={{
        title: t('Commercial control plane'),
        description: t('Operators can audit commercial accounts, request settlement posture, and pricing governance without leaving a dedicated module.'),
        children: (
          <div className="space-y-6">
            <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-5">
              {summaryCards.map((metric) => (
                <StatCard
                  description={metric.description}
                  key={metric.label}
                  label={metric.label}
                  value={metric.value}
                />
              ))}
            </div>

            <div className="grid gap-4 xl:grid-cols-3">
              <CommercialFactPanel
                description={t('Account posture keeps status, held balance, and admission readiness visible in one surface.')}
                facts={accountFacts}
                title={t('Commercial accounts')}
              />
              <CommercialFactPanel
                description={t('Settlement explorer highlights open holds, captured requests, and correction posture from canonical settlement records.')}
                facts={settlementFacts}
                title={t('Settlement explorer')}
              />
              <CommercialFactPanel
                description={t('Pricing governance keeps commercial plan activation and metric-rate coverage visible for operator review.')}
                facts={pricingFacts}
                title={t('Pricing governance')}
              />
            </div>

            <div className="grid gap-4 xl:grid-cols-[1.15fr_0.85fr]">
              <CommercialTableCard
                columns={settlementLedgerColumns}
                description={t('Settlement ledger keeps capture and refund entries linked to request settlements so operators can audit credits, retail charge, and final correction posture without leaving the commercial module.')}
                emptyDescription={t('Settlement ledger entries will appear here once commercial account history begins landing for the selected control-plane slice.')}
                emptyTitle={t('No settlement ledger entries yet')}
                getRowId={(row: CommercialLedgerTimelineRow) => row.id}
                rows={recentLedgerTimeline}
                title={t('Settlement ledger')}
              />
              <CommercialTableCard
                columns={refundTimelineColumns}
                description={t('Refund timeline isolates correction entries so support and finance can verify credited quantity, linked request, and refund cost posture at a glance.')}
                emptyDescription={t('Refund activity will appear here once commercial refunds are posted into the account ledger history.')}
                emptyTitle={t('No refunds recorded yet')}
                getRowId={(row: CommercialLedgerTimelineRow) => row.id}
                rows={refundTimelineRows}
                title={t('Refund timeline')}
              />
            </div>

            <div className="grid gap-4 xl:grid-cols-[1.15fr_0.85fr]">
              <Card className="min-h-0 flex flex-col overflow-hidden p-0">
                <CardHeader className="space-y-4">
                  <CardTitle>{t('Order payment audit')}</CardTitle>
                  <CardDescription>
                    {t('Order payment audit keeps recent commercial orders linked to payment callbacks, provider evidence, and operator-visible processing posture without loading unbounded order history into the commercial module.')}
                  </CardDescription>
                  <form
                    className="flex flex-wrap items-end gap-3"
                    onSubmit={onOrderAuditLookupSubmit}
                  >
                    <div className="min-w-[18rem] flex-[1.3] space-y-2">
                      <Label htmlFor="commercial-order-audit-lookup">
                        {orderAuditLookupLabel}
                      </Label>
                      <Input
                        aria-invalid={orderAuditLookupError != null}
                        autoComplete="off"
                        id="commercial-order-audit-lookup"
                        onChange={(event: ChangeEvent<HTMLInputElement>) => {
                          onOrderAuditLookupChange(event.target.value);
                        }}
                        placeholder={t('Enter an order id to open order audit detail.')}
                        value={orderAuditLookupValue}
                      />
                      {orderAuditLookupError ? (
                        <div className="text-xs text-[var(--sdk-color-status-danger)]">
                          {orderAuditLookupError}
                        </div>
                      ) : (
                        <div className="text-xs text-[var(--sdk-color-text-secondary)]">
                          {orderAuditLookupHint}
                        </div>
                      )}
                    </div>

                    <Button type="submit" variant="outline">
                      {t('Inspect')}
                    </Button>
                  </form>
                </CardHeader>
                <CardContent className="min-h-0 flex-1 p-0">
                  <DataTable
                    className={embeddedAdminDataTableClassName}
                    columns={orderPaymentAuditColumns}
                    emptyDescription={t('Recent commerce orders will appear here once checkout, webhook, and settlement evidence starts landing in the commercial audit stream.')}
                    emptyTitle={t('No order payment evidence yet')}
                    getRowId={(row: CommercialOrderPaymentAuditRow) => row.id}
                    rows={recentOrderPaymentAuditRows}
                    slotProps={embeddedAdminDataTableSlotProps}
                    stickyHeader
                  />
                </CardContent>
              </Card>

              <CommercialTableCard
                columns={orderRefundAuditColumns}
                description={t('Order refund audit keeps explicit refund callbacks and refunded-order fallback evidence visible so operators can spot missing callback closure before it becomes a reconciliation blind spot.')}
                emptyDescription={t('Refund audit rows will appear here once commercial orders begin entering explicit refund or refunded-order-state correction flows.')}
                emptyTitle={t('No refund evidence yet')}
                getRowId={(row: CommercialOrderPaymentAuditRow) => row.id}
                rows={refundAuditRows}
                title={t('Order refund audit')}
              />
            </div>

            {supplementarySections}
          </div>
        ),
      }}
      detail={{
        title: t('Latest settlements'),
        description: t('The right rail keeps the most recent commercial settlement evidence in view for rapid operator triage.'),
        children: (
          <CommercialLatestSettlementsRail
            formatCurrency={formatCurrency}
            formatNumber={formatNumber}
            latestSettlements={latestSettlements}
          />
        ),
      }}
      title={t('Commercial')}
    />
  );
}

export function CommercialLatestSettlementsRail({
  latestSettlements,
  formatCurrency,
  formatNumber,
}: CommercialLatestSettlementsRailProps) {
  const { t } = useAdminI18n();

  return (
    <div className="space-y-4">
      {latestSettlements.length ? (
        latestSettlements.map((settlement) => (
          <Card key={settlement.request_settlement_id}>
            <CardHeader className="space-y-2">
              <div className="flex items-start justify-between gap-3">
                <div>
                  <CardTitle className="text-base">
                    {t('Request #{id}', { id: settlement.request_id })}
                  </CardTitle>
                  <CardDescription>
                    {t('Account #{id}', { id: settlement.account_id })}
                  </CardDescription>
                </div>
                <StatusBadge
                  showIcon
                  status={formatStatusLabel(settlement.status)}
                  variant={
                    settlement.status === 'captured'
                      ? 'success'
                      : settlement.status === 'failed'
                        ? 'danger'
                        : 'secondary'
                  }
                />
              </div>
            </CardHeader>
            <CardContent className="grid gap-1 text-sm text-[var(--sdk-color-text-secondary)]">
              <div>
                {t('Retail charge: {amount}', {
                  amount: formatCurrency(settlement.retail_charge_amount),
                })}
              </div>
              <div>
                {t('Provider cost: {amount}', {
                  amount: formatCurrency(settlement.provider_cost_amount),
                })}
              </div>
              <div>
                {t('Captured credits: {count}', {
                  count: formatNumber(settlement.captured_credit_amount),
                })}
              </div>
            </CardContent>
          </Card>
        ))
      ) : (
        <Card>
          <CardHeader>
            <CardTitle>{t('No settlement evidence yet')}</CardTitle>
            <CardDescription>
              {t('Latest settlements will appear here once request settlement records start landing from the canonical commercial kernel.')}
            </CardDescription>
          </CardHeader>
        </Card>
      )}
    </div>
  );
}

function CommercialFactPanel({
  title,
  description,
  facts,
}: {
  title: string;
  description: string;
  facts: CommercialFact[];
}) {
  return (
    <Card>
      <CardHeader>
        <CardTitle>{title}</CardTitle>
        <CardDescription>{description}</CardDescription>
      </CardHeader>
      <CardContent className="space-y-3">
        {facts.map((fact) => (
          <div className="flex items-start justify-between gap-3" key={fact.label}>
            <div className="space-y-1">
              <div className="text-sm font-medium text-[var(--sdk-color-text-primary)]">
                {fact.label}
              </div>
              <div className="text-sm text-[var(--sdk-color-text-secondary)]">
                {fact.detail}
              </div>
            </div>
            <StatusBadge
              showIcon
              status={fact.value}
              variant={fact.tone ?? 'secondary'}
            />
          </div>
        ))}
      </CardContent>
    </Card>
  );
}

function CommercialTableCard<Row>({
  title,
  description,
  columns,
  rows,
  getRowId,
  emptyTitle,
  emptyDescription,
}: {
  title: string;
  description: string;
  columns: Array<DataTableColumn<Row>>;
  rows: Row[];
  getRowId: (row: Row) => string;
  emptyTitle: string;
  emptyDescription: string;
}) {
  return (
    <Card className="min-h-0 flex flex-col overflow-hidden p-0">
      <CardHeader>
        <CardTitle>{title}</CardTitle>
        <CardDescription>{description}</CardDescription>
      </CardHeader>
      <CardContent className="min-h-0 flex-1 p-0">
        <DataTable
          className={embeddedAdminDataTableClassName}
          columns={columns}
          emptyDescription={emptyDescription}
          emptyTitle={emptyTitle}
          getRowId={getRowId}
          rows={rows}
          slotProps={embeddedAdminDataTableSlotProps}
          stickyHeader
        />
      </CardContent>
    </Card>
  );
}

export function buildSettlementLedgerColumns(
  formatNumber: (value: number) => string,
  formatCurrency: (value: number) => string,
  t: (key: string, values?: Record<string, unknown>) => string,
): Array<DataTableColumn<CommercialLedgerTimelineRow>> {
  return [
    {
      id: 'account',
      header: t('Account'),
      cell: (row) => (
        <div className="space-y-1">
          <div className="font-medium text-[var(--sdk-color-text-primary)]">
            {t('Account #{id}', { id: row.account_id })}
          </div>
          <div className="text-sm text-[var(--sdk-color-text-secondary)]">
            {row.request_id != null
              ? t('Request #{id}', { id: row.request_id })
              : t('No linked request')}
          </div>
        </div>
      ),
    },
    {
      id: 'entry_type',
      header: t('Entry'),
      cell: (row) => (
        <StatusBadge
          showIcon
          status={formatLedgerEntryTypeLabel(row.entry_type)}
          variant={row.entry_type === 'refund' ? 'warning' : 'secondary'}
        />
      ),
      width: 180,
    },
    {
      id: 'credits',
      header: t('Credits'),
      cell: (row) => formatNumber(row.amount),
      width: 120,
    },
    {
      id: 'settlement',
      header: t('Settlement'),
      cell: (row) => (
        <StatusBadge
          showIcon
          status={row.settlement_status ? formatStatusLabel(row.settlement_status) : t('Unlinked')}
          variant={
            row.settlement_status === 'captured'
              ? 'success'
              : row.settlement_status === 'refunded'
                ? 'warning'
                : 'secondary'
          }
        />
      ),
      width: 160,
    },
    {
      id: 'retail_charge',
      header: t('Retail charge'),
      cell: (row) =>
        row.request_settlement_id != null
          ? formatCurrency(row.retail_charge_amount)
          : t('n/a'),
      width: 140,
    },
    {
      id: 'observed',
      header: t('Observed'),
      cell: (row) => formatAdminDateTime(row.created_at_ms),
      width: 180,
    },
  ];
}

export function buildRefundTimelineColumns(
  formatNumber: (value: number) => string,
  formatCurrency: (value: number) => string,
  t: (key: string, values?: Record<string, unknown>) => string,
): Array<DataTableColumn<CommercialLedgerTimelineRow>> {
  return [
    {
      id: 'account',
      header: t('Account'),
      cell: (row) => (
        <div className="space-y-1">
          <div className="font-medium text-[var(--sdk-color-text-primary)]">
            {t('Account #{id}', { id: row.account_id })}
          </div>
          <div className="text-sm text-[var(--sdk-color-text-secondary)]">
            {row.hold_id != null
              ? t('Hold #{id}', { id: row.hold_id })
              : t('No linked hold')}
          </div>
        </div>
      ),
    },
    {
      id: 'request',
      header: t('Request'),
      cell: (row) =>
        row.request_id != null ? t('Request #{id}', { id: row.request_id }) : t('Unlinked'),
      width: 140,
    },
    {
      id: 'refund_credits',
      header: t('Refund credits'),
      cell: (row) => formatNumber(row.refunded_amount || row.amount),
      width: 140,
    },
    {
      id: 'retail_charge',
      header: t('Retail charge'),
      cell: (row) =>
        row.request_settlement_id != null
          ? formatCurrency(row.retail_charge_amount)
          : t('n/a'),
      width: 140,
    },
    {
      id: 'provider_cost',
      header: t('Provider cost'),
      cell: (row) =>
        row.request_settlement_id != null
          ? formatCurrency(row.provider_cost_amount)
          : t('n/a'),
      width: 140,
    },
    {
      id: 'observed',
      header: t('Observed'),
      cell: (row) => formatAdminDateTime(row.created_at_ms),
      width: 180,
    },
  ];
}

export function buildOrderPaymentAuditColumns(
  onOpenOrderAudit: (orderId: string) => void,
  t: (key: string, values?: Record<string, unknown>) => string,
): Array<DataTableColumn<CommercialOrderPaymentAuditRow>> {
  return [
    {
      id: 'order',
      header: t('Order'),
      cell: (row) => (
        <div className="space-y-1">
          <div className="font-medium text-[var(--sdk-color-text-primary)]">
            {t('Order #{id}', { id: row.order_id })}
          </div>
          <div className="text-sm text-[var(--sdk-color-text-secondary)]">
            {row.target_name}
          </div>
        </div>
      ),
    },
    {
      id: 'event',
      header: t('Event'),
      cell: (row) => (
        <StatusBadge
          showIcon
          status={formatOrderAuditEventLabel(row)}
          variant={row.event_type === 'refunded' ? 'warning' : 'secondary'}
        />
      ),
      width: 180,
    },
    {
      id: 'provider',
      header: t('Provider'),
      cell: (row) => (
        <div className="space-y-1">
          <div className="font-medium text-[var(--sdk-color-text-primary)]">
            {formatStatusLabel(row.provider)}
          </div>
          <div className="text-sm text-[var(--sdk-color-text-secondary)]">
            {row.provider_event_id ?? t('No provider event id')}
          </div>
        </div>
      ),
      width: 220,
    },
    {
      id: 'processing',
      header: t('Processing'),
      cell: (row) => (
        <StatusBadge
          showIcon
          status={row.processing_status ? formatStatusLabel(row.processing_status) : t('Pending evidence')}
          variant={
            row.processing_status === 'processed'
              ? 'success'
              : row.processing_status === 'rejected' || row.processing_status === 'failed'
                ? 'danger'
                : 'secondary'
          }
        />
      ),
      width: 180,
    },
    {
      id: 'amount',
      header: t('Amount'),
      cell: (row) => row.payable_price_label,
      width: 140,
    },
    {
      id: 'observed',
      header: t('Observed'),
      cell: (row) => formatAdminDateTime(row.observed_at_ms),
      width: 180,
    },
    {
      id: 'detail',
      header: t('Investigation'),
      cell: (row) => (
        <Button
          onClick={() => onOpenOrderAudit(row.order_id)}
          size="sm"
          type="button"
          variant="outline"
        >
          {t('View order audit')}
        </Button>
      ),
      width: 180,
    },
  ];
}

export function buildOrderRefundAuditColumns(
  onOpenOrderAudit: (orderId: string) => void,
  t: (key: string, values?: Record<string, unknown>) => string,
): Array<DataTableColumn<CommercialOrderPaymentAuditRow>> {
  return [
    {
      id: 'order',
      header: t('Order'),
      cell: (row) => (
        <div className="space-y-1">
          <div className="font-medium text-[var(--sdk-color-text-primary)]">
            {t('Order #{id}', { id: row.order_id })}
          </div>
          <div className="text-sm text-[var(--sdk-color-text-secondary)]">
            {row.target_name}
          </div>
        </div>
      ),
    },
    {
      id: 'provider',
      header: t('Provider'),
      cell: (row) => formatStatusLabel(row.provider),
      width: 160,
    },
    {
      id: 'refund_state',
      header: t('Refund state'),
      cell: (row) => (
        <StatusBadge
          showIcon
          status={formatOrderAuditEventLabel(row)}
          variant="warning"
        />
      ),
      width: 160,
    },
    {
      id: 'amount',
      header: t('Amount'),
      cell: (row) => row.payable_price_label,
      width: 140,
    },
    {
      id: 'observed',
      header: t('Observed'),
      cell: (row) => formatAdminDateTime(row.observed_at_ms),
      width: 180,
    },
    {
      id: 'detail',
      header: t('Investigation'),
      cell: (row) => (
        <Button
          onClick={() => onOpenOrderAudit(row.order_id)}
          size="sm"
          type="button"
          variant="outline"
        >
          {t('View order audit')}
        </Button>
      ),
      width: 180,
    },
  ];
}
