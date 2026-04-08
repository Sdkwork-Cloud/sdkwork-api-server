import { useEffect, useMemo, useState, type ChangeEvent, type FormEvent } from 'react';
import {
  Button,
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
  DataTable,
  Input,
  StatusBadge,
  type DataTableColumn,
} from '@sdkwork/ui-pc-react';
import {
  createCommerceReconciliationRun,
  listCommerceReconciliationItems,
  listCommerceReconciliationRuns,
} from 'sdkwork-router-admin-admin-api';
import {
  embeddedAdminDataTableClassName,
  embeddedAdminDataTableSlotProps,
  formatAdminDateTime,
  useAdminI18n,
} from 'sdkwork-router-admin-core';
import type {
  CommerceReconciliationItemRecord,
  CommerceReconciliationRunRecord,
  PaymentMethodRecord,
} from 'sdkwork-router-admin-types';
import { DialogField, SelectField, previewJson, resolveStatusVariant } from './paymentShared';

type PaymentReconciliationSectionProps = {
  paymentMethods: PaymentMethodRecord[];
};

const allPaymentMethodsValue = '__all_payment_methods__';

export function PaymentReconciliationSection({
  paymentMethods,
}: PaymentReconciliationSectionProps) {
  const { formatNumber, t } = useAdminI18n();
  const [runs, setRuns] = useState<CommerceReconciliationRunRecord[]>([]);
  const [selectedRunId, setSelectedRunId] = useState<string | null>(null);
  const [items, setItems] = useState<CommerceReconciliationItemRecord[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [draft, setDraft] = useState(() => {
    const now = Date.now();
    return {
      provider: 'stripe',
      payment_method_id: allPaymentMethodsValue,
      scope_started_at: toDateTimeLocalValue(now - 24 * 60 * 60 * 1000),
      scope_ended_at: toDateTimeLocalValue(now),
    };
  });

  useEffect(() => {
    let cancelled = false;
    listCommerceReconciliationRuns()
      .then((records) => {
        if (cancelled) {
          return;
        }
        setRuns(records);
        setSelectedRunId((current) => current ?? records[0]?.reconciliation_run_id ?? null);
      })
      .catch((nextError) => {
        if (!cancelled) {
          setError(
            nextError instanceof Error
              ? nextError.message
              : 'Failed to load reconciliation runs.',
          );
        }
      });

    return () => {
      cancelled = true;
    };
  }, []);

  useEffect(() => {
    if (!selectedRunId) {
      setItems([]);
      return;
    }

    let cancelled = false;
    listCommerceReconciliationItems(selectedRunId)
      .then((records) => {
        if (!cancelled) {
          setItems(records);
        }
      })
      .catch((nextError) => {
        if (!cancelled) {
          setError(
            nextError instanceof Error
              ? nextError.message
              : 'Failed to load reconciliation items.',
          );
        }
      });

    return () => {
      cancelled = true;
    };
  }, [selectedRunId]);

  const openItemCount = items.filter((item) => item.status === 'open').length;
  const mismatchCount = items.filter((item) =>
    resolveStatusVariant(item.discrepancy_type) !== 'success',
  ).length;

  const runColumns = useMemo<Array<DataTableColumn<CommerceReconciliationRunRecord>>>(
    () => [
      {
        id: 'run',
        header: t('Run'),
        cell: (row) => (
          <div className="space-y-1">
            <div className="font-medium text-[var(--sdk-color-text-primary)]">
              {row.reconciliation_run_id}
            </div>
            <div className="text-sm text-[var(--sdk-color-text-secondary)]">
              {row.provider} / {row.payment_method_id || t('All methods')}
            </div>
          </div>
        ),
      },
      {
        id: 'status',
        header: t('Status'),
        cell: (row) => (
          <StatusBadge
            showIcon
            status={row.status}
            variant={resolveStatusVariant(row.status)}
          />
        ),
        width: 150,
      },
      {
        id: 'window',
        header: t('Scope window'),
        cell: (row) => (
          <div className="space-y-1 text-sm">
            <div>{formatAdminDateTime(row.scope_started_at_ms)}</div>
            <div className="text-[var(--sdk-color-text-secondary)]">
              {formatAdminDateTime(row.scope_ended_at_ms)}
            </div>
          </div>
        ),
        width: 220,
      },
      {
        id: 'summary',
        header: t('Summary'),
        cell: (row) => previewJson(row.summary_json),
        width: 280,
      },
      {
        id: 'created',
        header: t('Created'),
        cell: (row) => formatAdminDateTime(row.created_at_ms),
        width: 180,
      },
    ],
    [t],
  );

  const itemColumns = useMemo<Array<DataTableColumn<CommerceReconciliationItemRecord>>>(
    () => [
      {
        id: 'item',
        header: t('Item'),
        cell: (row) => (
          <div className="space-y-1">
            <div className="font-medium text-[var(--sdk-color-text-primary)]">
              {row.reconciliation_item_id}
            </div>
            <div className="text-sm text-[var(--sdk-color-text-secondary)]">
              {row.external_reference || row.payment_attempt_id || row.refund_id || row.order_id || t('No reference')}
            </div>
          </div>
        ),
      },
      {
        id: 'discrepancy_type',
        header: t('Discrepancy'),
        cell: (row) => (
          <StatusBadge
            showIcon
            status={row.discrepancy_type}
            variant={resolveStatusVariant(row.discrepancy_type)}
          />
        ),
        width: 170,
      },
      {
        id: 'status',
        header: t('Status'),
        cell: (row) => (
          <StatusBadge
            showIcon
            status={row.status}
            variant={resolveStatusVariant(row.status)}
          />
        ),
        width: 150,
      },
      {
        id: 'expected_amount',
        header: t('Expected'),
        cell: (row) => formatNumber(row.expected_amount_minor),
        width: 130,
      },
      {
        id: 'provider_amount',
        header: t('Provider'),
        cell: (row) =>
          row.provider_amount_minor != null
            ? formatNumber(row.provider_amount_minor)
            : t('n/a'),
        width: 130,
      },
      {
        id: 'detail',
        header: t('Detail'),
        cell: (row) => previewJson(row.detail_json),
        width: 280,
      },
    ],
    [formatNumber, t],
  );

  async function handleCreateRun(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();

    try {
      setError(null);
      const run = await createCommerceReconciliationRun({
        provider: draft.provider.trim(),
        payment_method_id:
          draft.payment_method_id === allPaymentMethodsValue
            ? undefined
            : draft.payment_method_id.trim() || undefined,
        scope_started_at_ms: parseDateTimeLocalValue(draft.scope_started_at),
        scope_ended_at_ms: parseDateTimeLocalValue(draft.scope_ended_at),
      });
      const nextRuns = await listCommerceReconciliationRuns();
      setRuns(nextRuns);
      setSelectedRunId(run.reconciliation_run_id);
    } catch (nextError) {
      setError(
        nextError instanceof Error
          ? nextError.message
          : 'Failed to create reconciliation run.',
      );
    }
  }

  return (
    <div className="grid gap-4 xl:grid-cols-[0.95fr_1.05fr]">
      <Card className="p-0">
        <CardHeader className="space-y-3">
          <CardTitle>{t('Reconciliation runs')}</CardTitle>
          <CardDescription>
            {t('Launch provider-vs-local reconciliation scans, isolate mismatches, and keep a concrete discrepancy ledger for finance closure.')}
          </CardDescription>
          {error ? (
            <div className="text-sm text-[var(--sdk-color-status-danger)]">{error}</div>
          ) : null}
          <form className="grid gap-4 md:grid-cols-2" onSubmit={handleCreateRun}>
            <DialogField htmlFor="reconciliation-provider" label={t('Provider')}>
              <Input
                autoComplete="off"
                id="reconciliation-provider"
                onChange={(event: ChangeEvent<HTMLInputElement>) =>
                  setDraft((current) => ({ ...current, provider: event.target.value }))
                }
                placeholder={t('stripe')}
                value={draft.provider}
              />
            </DialogField>

            <SelectField
              label={t('Payment method')}
              onValueChange={(value) =>
                setDraft((current) => ({ ...current, payment_method_id: value }))
              }
              options={[
                { label: t('All methods'), value: allPaymentMethodsValue },
                ...paymentMethods.map((method) => ({
                  label: `${method.display_name} (${method.payment_method_id})`,
                  value: method.payment_method_id,
                })),
              ]}
              value={draft.payment_method_id}
            />

            <DialogField htmlFor="reconciliation-scope-start" label={t('Scope started at')}>
              <Input
                id="reconciliation-scope-start"
                onChange={(event: ChangeEvent<HTMLInputElement>) =>
                  setDraft((current) => ({ ...current, scope_started_at: event.target.value }))
                }
                type="datetime-local"
                value={draft.scope_started_at}
              />
            </DialogField>

            <DialogField htmlFor="reconciliation-scope-end" label={t('Scope ended at')}>
              <Input
                id="reconciliation-scope-end"
                onChange={(event: ChangeEvent<HTMLInputElement>) =>
                  setDraft((current) => ({ ...current, scope_ended_at: event.target.value }))
                }
                type="datetime-local"
                value={draft.scope_ended_at}
              />
            </DialogField>

            <div className="md:col-span-2">
              <Button type="submit" variant="primary">
                {t('Create reconciliation run')}
              </Button>
            </div>
          </form>
        </CardHeader>
        <CardContent className="space-y-4 p-0">
          <div className="grid gap-4 px-6 md:grid-cols-3">
            <div className="rounded-[var(--sdk-radius-card)] border border-[var(--sdk-color-border-default)] px-4 py-3 text-sm">
              <div className="font-medium text-[var(--sdk-color-text-primary)]">{t('Run count')}</div>
              <div className="mt-1 text-[var(--sdk-color-text-secondary)]">{formatNumber(runs.length)}</div>
            </div>
            <div className="rounded-[var(--sdk-radius-card)] border border-[var(--sdk-color-border-default)] px-4 py-3 text-sm">
              <div className="font-medium text-[var(--sdk-color-text-primary)]">{t('Open items')}</div>
              <div className="mt-1 text-[var(--sdk-color-text-secondary)]">{formatNumber(openItemCount)}</div>
            </div>
            <div className="rounded-[var(--sdk-radius-card)] border border-[var(--sdk-color-border-default)] px-4 py-3 text-sm">
              <div className="font-medium text-[var(--sdk-color-text-primary)]">{t('Mismatches')}</div>
              <div className="mt-1 text-[var(--sdk-color-text-secondary)]">{formatNumber(mismatchCount)}</div>
            </div>
          </div>

          <DataTable
            className={embeddedAdminDataTableClassName}
            columns={runColumns}
            emptyDescription={t('Create the first reconciliation run to compare local payment attempts and refunds against the provider source of truth.')}
            emptyTitle={t('No reconciliation runs')}
            getRowId={(row) => row.reconciliation_run_id}
            getRowProps={(row) => ({
              onClick: () => setSelectedRunId(row.reconciliation_run_id),
            })}
            rows={runs}
            slotProps={embeddedAdminDataTableSlotProps}
            stickyHeader
          />
        </CardContent>
      </Card>

      <Card className="min-h-0 flex flex-col p-0">
        <CardHeader>
          <CardTitle>{t('Reconciliation items')}</CardTitle>
          <CardDescription>
            {selectedRunId
              ? t('Discrepancy ledger for run {id}.', { id: selectedRunId })
              : t('Select a reconciliation run to inspect its discrepancy items.')}
          </CardDescription>
        </CardHeader>
        <CardContent className="min-h-0 flex-1 p-0">
          <DataTable
            className={embeddedAdminDataTableClassName}
            columns={itemColumns}
            emptyDescription={t('Discrepancy items will appear here when the selected reconciliation run detects local-provider divergence.')}
            emptyTitle={t('No reconciliation items')}
            getRowId={(row) => row.reconciliation_item_id}
            rows={items}
            slotProps={embeddedAdminDataTableSlotProps}
            stickyHeader
          />
        </CardContent>
      </Card>
    </div>
  );
}

function toDateTimeLocalValue(timestampMs: number): string {
  const date = new Date(timestampMs);
  const timezoneOffsetMs = date.getTimezoneOffset() * 60 * 1000;
  return new Date(timestampMs - timezoneOffsetMs).toISOString().slice(0, 16);
}

function parseDateTimeLocalValue(value: string): number {
  const parsed = Date.parse(value);
  return Number.isFinite(parsed) ? parsed : Date.now();
}
