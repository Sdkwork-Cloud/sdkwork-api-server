import { useEffect, useMemo, useState } from 'react';
import {
  Button,
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
  createCommerceRefund,
  listCommercePaymentAttempts,
  listCommerceRefunds,
} from 'sdkwork-router-admin-admin-api';
import {
  embeddedAdminDataTableClassName,
  embeddedAdminDataTableSlotProps,
  formatAdminDateTime,
  useAdminI18n,
} from 'sdkwork-router-admin-core';
import type {
  CommerceOrderRecord,
  CommercePaymentAttemptRecord,
  CommerceRefundRecord,
} from 'sdkwork-router-admin-types';
import { PaymentRefundDialog } from './paymentRefundDialog';
import { previewJson, resolveStatusVariant } from './paymentShared';

type PaymentOrderOperationsSectionProps = {
  onOpenOrderAudit: (orderId: string) => void;
  onRefreshWorkspace: () => Promise<void>;
  orders: CommerceOrderRecord[];
};

export function PaymentOrderOperationsSection({
  onOpenOrderAudit,
  onRefreshWorkspace,
  orders,
}: PaymentOrderOperationsSectionProps) {
  const { formatNumber, t } = useAdminI18n();
  const [selectedOrderId, setSelectedOrderId] = useState<string | null>(orders[0]?.order_id ?? null);
  const [attempts, setAttempts] = useState<CommercePaymentAttemptRecord[]>([]);
  const [refunds, setRefunds] = useState<CommerceRefundRecord[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [refundDialogOpen, setRefundDialogOpen] = useState(false);

  const selectedOrder = selectedOrderId
    ? orders.find((order) => order.order_id === selectedOrderId) ?? null
    : null;

  useEffect(() => {
    if (!orders.length) {
      setSelectedOrderId(null);
      return;
    }

    if (!selectedOrderId || !orders.some((order) => order.order_id === selectedOrderId)) {
      setSelectedOrderId(orders[0]?.order_id ?? null);
    }
  }, [orders, selectedOrderId]);

  useEffect(() => {
    if (!selectedOrderId) {
      setAttempts([]);
      setRefunds([]);
      setError(null);
      return;
    }

    let cancelled = false;
    setLoading(true);
    setError(null);

    Promise.all([
      listCommercePaymentAttempts(selectedOrderId),
      listCommerceRefunds(selectedOrderId),
    ])
      .then(([attemptRecords, refundRecords]) => {
        if (cancelled) {
          return;
        }
        setAttempts(attemptRecords);
        setRefunds(refundRecords);
      })
      .catch((nextError) => {
        if (cancelled) {
          return;
        }
        setError(
          nextError instanceof Error
            ? nextError.message
            : 'Failed to load payment attempts and refunds.',
        );
      })
      .finally(() => {
        if (!cancelled) {
          setLoading(false);
        }
      });

    return () => {
      cancelled = true;
    };
  }, [selectedOrderId]);

  const orderColumns = useMemo<Array<DataTableColumn<CommerceOrderRecord>>>(
    () => [
      {
        id: 'order',
        header: t('Order'),
        cell: (row) => (
          <div className="space-y-1">
            <div className="font-medium text-[var(--sdk-color-text-primary)]">
              {row.target_name}
            </div>
            <div className="text-sm text-[var(--sdk-color-text-secondary)]">
              {row.order_id}
            </div>
          </div>
        ),
      },
      {
        id: 'order_status',
        header: t('Order status'),
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
        id: 'settlement_status',
        header: t('Settlement'),
        cell: (row) => (
          <StatusBadge
            showIcon
            status={row.settlement_status}
            variant={resolveStatusVariant(row.settlement_status)}
          />
        ),
        width: 170,
      },
      {
        id: 'amount',
        header: t('Amount'),
        cell: (row) => (
          <div className="space-y-1 text-sm">
            <div>{row.payable_price_label}</div>
            <div className="text-[var(--sdk-color-text-secondary)]">
              {t('Refundable {count}', { count: formatNumber(row.refundable_amount_minor) })}
            </div>
          </div>
        ),
        width: 180,
      },
      {
        id: 'payment_method',
        header: t('Method'),
        cell: (row) => row.payment_method_id || t('Not chosen'),
        width: 180,
      },
      {
        id: 'updated',
        header: t('Updated'),
        cell: (row) => formatAdminDateTime(row.updated_at_ms),
        width: 180,
      },
      {
        id: 'actions',
        header: t('Actions'),
        cell: (row) => (
          <div className="flex flex-wrap gap-2">
            <Button
              onClick={() => setSelectedOrderId(row.order_id)}
              size="sm"
              type="button"
              variant={row.order_id === selectedOrderId ? 'primary' : 'outline'}
            >
              {t('Operate')}
            </Button>
            <Button
              onClick={() => onOpenOrderAudit(row.order_id)}
              size="sm"
              type="button"
              variant="outline"
            >
              {t('Audit')}
            </Button>
          </div>
        ),
        width: 190,
      },
    ],
    [formatNumber, onOpenOrderAudit, selectedOrderId, t],
  );

  const attemptColumns = useMemo<Array<DataTableColumn<CommercePaymentAttemptRecord>>>(
    () => [
      {
        id: 'attempt',
        header: t('Attempt'),
        cell: (row) => (
          <div className="space-y-1">
            <div className="font-medium text-[var(--sdk-color-text-primary)]">
              {row.payment_attempt_id}
            </div>
            <div className="text-sm text-[var(--sdk-color-text-secondary)]">
              {row.payment_method_id} / #{row.attempt_sequence}
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
        id: 'provider',
        header: t('Provider reference'),
        cell: (row) => (
          <div className="space-y-1 text-sm">
            <div>{row.provider_checkout_session_id || row.provider_payment_intent_id || t('n/a')}</div>
            <div className="text-[var(--sdk-color-text-secondary)]">
              {row.provider_reference || row.provider}
            </div>
          </div>
        ),
        width: 260,
      },
      {
        id: 'amount',
        header: t('Amounts'),
        cell: (row) => (
          <div className="space-y-1 text-sm">
            <div>
              {row.currency_code} {formatNumber(row.amount_minor)}
            </div>
            <div className="text-[var(--sdk-color-text-secondary)]">
              {t('Captured {captured} / Refunded {refunded}', {
                captured: formatNumber(row.captured_amount_minor),
                refunded: formatNumber(row.refunded_amount_minor),
              })}
            </div>
          </div>
        ),
        width: 220,
      },
      {
        id: 'payload',
        header: t('Payload'),
        cell: (row) => previewJson(row.response_payload_json),
        width: 260,
      },
      {
        id: 'time',
        header: t('Initiated'),
        cell: (row) => formatAdminDateTime(row.initiated_at_ms),
        width: 180,
      },
    ],
    [formatNumber, t],
  );

  const refundColumns = useMemo<Array<DataTableColumn<CommerceRefundRecord>>>(
    () => [
      {
        id: 'refund',
        header: t('Refund'),
        cell: (row) => (
          <div className="space-y-1">
            <div className="font-medium text-[var(--sdk-color-text-primary)]">
              {row.refund_id}
            </div>
            <div className="text-sm text-[var(--sdk-color-text-secondary)]">
              {row.payment_attempt_id || t('No linked attempt')}
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
        id: 'amount',
        header: t('Amount'),
        cell: (row) => `${row.currency_code} ${formatNumber(row.amount_minor)}`,
        width: 160,
      },
      {
        id: 'provider_refund',
        header: t('Provider refund'),
        cell: (row) => row.provider_refund_id || t('Pending provider id'),
        width: 220,
      },
      {
        id: 'reason',
        header: t('Reason'),
        cell: (row) => row.reason || t('No reason provided'),
        width: 220,
      },
      {
        id: 'updated',
        header: t('Updated'),
        cell: (row) => formatAdminDateTime(row.updated_at_ms),
        width: 180,
      },
    ],
    [formatNumber, t],
  );

  async function handleCreateRefund(request: {
    payment_attempt_id?: string | null;
    amount_minor?: number | null;
    reason?: string | null;
    idempotency_key?: string | null;
  }) {
    if (!selectedOrder) {
      return;
    }

    try {
      setError(null);
      await createCommerceRefund(selectedOrder.order_id, request);
      setRefundDialogOpen(false);
      await onRefreshWorkspace();

      const [attemptRecords, refundRecords] = await Promise.all([
        listCommercePaymentAttempts(selectedOrder.order_id),
        listCommerceRefunds(selectedOrder.order_id),
      ]);
      setAttempts(attemptRecords);
      setRefunds(refundRecords);
    } catch (nextError) {
      setError(
        nextError instanceof Error ? nextError.message : 'Failed to create refund.',
      );
    }
  }

  return (
    <>
      <Card className="p-0">
        <CardHeader className="space-y-2">
          <CardTitle>{t('Order operations and settlement closure')}</CardTitle>
          <CardDescription>
            {t('Inspect payment attempts, issue provider-backed refunds, and verify settlement progression from order to final financial closure.')}
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="rounded-[var(--sdk-radius-card)] border border-[var(--sdk-color-border-default)]">
            <DataTable
              className={embeddedAdminDataTableClassName}
              columns={orderColumns}
              emptyDescription={t('Recent orders will appear here after portal checkout starts producing real provider-backed attempts.')}
              emptyTitle={t('No recent orders')}
              getRowId={(row) => row.order_id}
              rows={orders}
              slotProps={embeddedAdminDataTableSlotProps}
              stickyHeader
            />
          </div>

          {selectedOrder ? (
            <div className="grid gap-4 xl:grid-cols-[1.15fr_0.85fr]">
              <Card className="min-h-0 flex flex-col p-0">
                <CardHeader className="space-y-3">
                  <div className="flex flex-wrap items-start justify-between gap-3">
                    <div>
                      <CardTitle>{t('Payment attempts')}</CardTitle>
                      <CardDescription>
                        {selectedOrder.target_name} / {selectedOrder.order_id}
                      </CardDescription>
                    </div>
                    <div className="flex flex-wrap gap-2">
                      <Button
                        onClick={() => onOpenOrderAudit(selectedOrder.order_id)}
                        type="button"
                        variant="outline"
                      >
                        {t('Open full audit')}
                      </Button>
                      <Button
                        disabled={selectedOrder.refundable_amount_minor <= 0}
                        onClick={() => setRefundDialogOpen(true)}
                        type="button"
                        variant="primary"
                      >
                        {t('Create refund')}
                      </Button>
                    </div>
                  </div>
                  <div className="text-sm text-[var(--sdk-color-text-secondary)]">
                    {loading
                      ? t('Loading attempts and refunds...')
                      : error || t('Attempts and refunds synchronized from admin commerce endpoints.')}
                  </div>
                </CardHeader>
                <CardContent className="min-h-0 flex-1 p-0">
                  <DataTable
                    className={embeddedAdminDataTableClassName}
                    columns={attemptColumns}
                    emptyDescription={t('Real checkout attempts for the selected order will appear here once the payment adapter starts creating provider sessions.')}
                    emptyTitle={t('No payment attempts')}
                    getRowId={(row) => row.payment_attempt_id}
                    rows={attempts}
                    slotProps={embeddedAdminDataTableSlotProps}
                    stickyHeader
                  />
                </CardContent>
              </Card>

              <Card className="min-h-0 flex flex-col p-0">
                <CardHeader>
                  <CardTitle>{t('Refund operations')}</CardTitle>
                  <CardDescription>
                    {t('Provider refund records stay visible here for finance, support, and reconciliation follow-up.')}
                  </CardDescription>
                </CardHeader>
                <CardContent className="min-h-0 flex-1 p-0">
                  <DataTable
                    className={embeddedAdminDataTableClassName}
                    columns={refundColumns}
                    emptyDescription={t('Refund records will appear here once the selected order enters a correction or partial refund flow.')}
                    emptyTitle={t('No refunds')}
                    getRowId={(row) => row.refund_id}
                    rows={refunds}
                    slotProps={embeddedAdminDataTableSlotProps}
                    stickyHeader
                  />
                </CardContent>
              </Card>
            </div>
          ) : null}
        </CardContent>
      </Card>

      <PaymentRefundDialog
        attempts={attempts}
        onOpenChange={setRefundDialogOpen}
        onSubmit={handleCreateRefund}
        open={refundDialogOpen}
        order={selectedOrder}
      />
    </>
  );
}
