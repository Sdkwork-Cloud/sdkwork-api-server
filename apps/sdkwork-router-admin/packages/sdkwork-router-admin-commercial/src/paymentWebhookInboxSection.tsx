import { useEffect, useMemo, useState } from 'react';
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
  DataTable,
  StatusBadge,
  type DataTableColumn,
} from '@sdkwork/ui-pc-react';
import {
  listCommerceWebhookDeliveryAttempts,
  listCommerceWebhookInbox,
} from 'sdkwork-router-admin-admin-api';
import {
  embeddedAdminDataTableClassName,
  embeddedAdminDataTableSlotProps,
  formatAdminDateTime,
  useAdminI18n,
} from 'sdkwork-router-admin-core';
import type {
  CommerceWebhookDeliveryAttemptRecord,
  CommerceWebhookInboxRecord,
} from 'sdkwork-router-admin-types';
import { previewJson, resolveStatusVariant } from './paymentShared';

export function PaymentWebhookInboxSection() {
  const { formatNumber, t } = useAdminI18n();
  const [inboxRecords, setInboxRecords] = useState<CommerceWebhookInboxRecord[]>([]);
  const [selectedInboxId, setSelectedInboxId] = useState<string | null>(null);
  const [deliveryAttempts, setDeliveryAttempts] = useState<CommerceWebhookDeliveryAttemptRecord[]>([]);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;

    listCommerceWebhookInbox()
      .then((records) => {
        if (cancelled) {
          return;
        }
        setInboxRecords(records);
        setSelectedInboxId((current) => current ?? records[0]?.webhook_inbox_id ?? null);
      })
      .catch((nextError) => {
        if (!cancelled) {
          setError(
            nextError instanceof Error ? nextError.message : 'Failed to load webhook inbox.',
          );
        }
      });

    return () => {
      cancelled = true;
    };
  }, []);

  useEffect(() => {
    if (!selectedInboxId) {
      setDeliveryAttempts([]);
      return;
    }

    let cancelled = false;
    listCommerceWebhookDeliveryAttempts(selectedInboxId)
      .then((records) => {
        if (!cancelled) {
          setDeliveryAttempts(records);
        }
      })
      .catch((nextError) => {
        if (!cancelled) {
          setError(
            nextError instanceof Error
              ? nextError.message
              : 'Failed to load webhook delivery attempts.',
          );
        }
      });

    return () => {
      cancelled = true;
    };
  }, [selectedInboxId]);

  const openRetries = inboxRecords.filter((record) =>
    record.processing_status.includes('retry')
    || record.next_retry_at_ms != null,
  ).length;
  const failedRecords = inboxRecords.filter((record) =>
    resolveStatusVariant(record.processing_status) === 'danger',
  ).length;

  const inboxColumns = useMemo<Array<DataTableColumn<CommerceWebhookInboxRecord>>>(
    () => [
      {
        id: 'event',
        header: t('Webhook event'),
        cell: (row) => (
          <div className="space-y-1">
            <div className="font-medium text-[var(--sdk-color-text-primary)]">
              {row.provider_event_id || t('No provider event id')}
            </div>
            <div className="text-sm text-[var(--sdk-color-text-secondary)]">
              {row.webhook_inbox_id}
            </div>
          </div>
        ),
      },
      {
        id: 'provider',
        header: t('Provider'),
        cell: (row) => (
          <div className="space-y-1 text-sm">
            <div>{row.provider}</div>
            <div className="text-[var(--sdk-color-text-secondary)]">
              {row.payment_method_id || t('No payment method')}
            </div>
          </div>
        ),
        width: 180,
      },
      {
        id: 'status',
        header: t('Processing'),
        cell: (row) => (
          <StatusBadge
            showIcon
            status={row.processing_status}
            variant={resolveStatusVariant(row.processing_status)}
          />
        ),
        width: 160,
      },
      {
        id: 'retry',
        header: t('Retries'),
        cell: (row) =>
          t('{current} / {max}', {
            current: formatNumber(row.retry_count),
            max: formatNumber(row.max_retry_count),
          }),
        width: 120,
      },
      {
        id: 'payload',
        header: t('Payload'),
        cell: (row) => previewJson(row.payload_json),
        width: 260,
      },
      {
        id: 'received',
        header: t('Received'),
        cell: (row) => formatAdminDateTime(row.last_received_at_ms),
        width: 180,
      },
    ],
    [formatNumber, t],
  );

  const attemptColumns = useMemo<
    Array<DataTableColumn<CommerceWebhookDeliveryAttemptRecord>>
  >(
    () => [
      {
        id: 'delivery_attempt',
        header: t('Delivery attempt'),
        cell: (row) => row.delivery_attempt_id,
      },
      {
        id: 'status',
        header: t('Status'),
        cell: (row) => (
          <StatusBadge
            showIcon
            status={row.processing_status}
            variant={resolveStatusVariant(row.processing_status)}
          />
        ),
        width: 150,
      },
      {
        id: 'response_code',
        header: t('Response'),
        cell: (row) => row.response_code != null ? String(row.response_code) : t('n/a'),
        width: 120,
      },
      {
        id: 'error_message',
        header: t('Error'),
        cell: (row) => row.error_message || t('No error'),
        width: 240,
      },
      {
        id: 'started',
        header: t('Started'),
        cell: (row) => formatAdminDateTime(row.started_at_ms),
        width: 180,
      },
      {
        id: 'finished',
        header: t('Finished'),
        cell: (row) => row.finished_at_ms ? formatAdminDateTime(row.finished_at_ms) : t('Running'),
        width: 180,
      },
    ],
    [t],
  );

  return (
    <div className="grid gap-4 xl:grid-cols-[1.15fr_0.85fr]">
      <Card className="min-h-0 flex flex-col p-0">
        <CardHeader className="space-y-3">
          <CardTitle>{t('Webhook inbox and replay defense')}</CardTitle>
          <CardDescription>
            {t('Track provider callbacks after signature verification, dedupe, replay-window checks, and dead-letter retry scheduling.')}
          </CardDescription>
          <div className="grid gap-4 md:grid-cols-3">
            <div className="rounded-[var(--sdk-radius-card)] border border-[var(--sdk-color-border-default)] px-4 py-3 text-sm">
              <div className="font-medium text-[var(--sdk-color-text-primary)]">{t('Inbox records')}</div>
              <div className="mt-1 text-[var(--sdk-color-text-secondary)]">{formatNumber(inboxRecords.length)}</div>
            </div>
            <div className="rounded-[var(--sdk-radius-card)] border border-[var(--sdk-color-border-default)] px-4 py-3 text-sm">
              <div className="font-medium text-[var(--sdk-color-text-primary)]">{t('Open retries')}</div>
              <div className="mt-1 text-[var(--sdk-color-text-secondary)]">{formatNumber(openRetries)}</div>
            </div>
            <div className="rounded-[var(--sdk-radius-card)] border border-[var(--sdk-color-border-default)] px-4 py-3 text-sm">
              <div className="font-medium text-[var(--sdk-color-text-primary)]">{t('Failed records')}</div>
              <div className="mt-1 text-[var(--sdk-color-text-secondary)]">{formatNumber(failedRecords)}</div>
            </div>
          </div>
          {error ? (
            <div className="text-sm text-[var(--sdk-color-status-danger)]">{error}</div>
          ) : null}
        </CardHeader>
        <CardContent className="min-h-0 flex-1 p-0">
          <DataTable
            className={embeddedAdminDataTableClassName}
            columns={inboxColumns}
            emptyDescription={t('Webhook inbox records will appear here after provider callbacks start landing in the secure webhook pipeline.')}
            emptyTitle={t('No webhook inbox records')}
            getRowId={(row) => row.webhook_inbox_id}
            getRowProps={(row) => ({
              onClick: () => setSelectedInboxId(row.webhook_inbox_id),
            })}
            rows={inboxRecords}
            slotProps={embeddedAdminDataTableSlotProps}
            stickyHeader
          />
        </CardContent>
      </Card>

      <Card className="min-h-0 flex flex-col p-0">
        <CardHeader>
          <CardTitle>{t('Delivery attempts')}</CardTitle>
          <CardDescription>
            {selectedInboxId
              ? t('Attempt history for webhook inbox {id}.', { id: selectedInboxId })
              : t('Select an inbox record to inspect retry history.')}
          </CardDescription>
        </CardHeader>
        <CardContent className="min-h-0 flex-1 p-0">
          <DataTable
            className={embeddedAdminDataTableClassName}
            columns={attemptColumns}
            emptyDescription={t('Delivery attempts will appear here when a webhook record is selected.')}
            emptyTitle={t('No delivery attempts')}
            getRowId={(row) => row.delivery_attempt_id}
            rows={deliveryAttempts}
            slotProps={embeddedAdminDataTableSlotProps}
            stickyHeader
          />
        </CardContent>
      </Card>
    </div>
  );
}
