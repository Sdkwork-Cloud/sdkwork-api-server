import { useMemo, useState } from 'react';
import {
  Button,
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
  DataTable,
  StatusBadge,
  StatCard,
  type DataTableColumn,
} from '@sdkwork/ui-pc-react';
import {
  deleteCommercePaymentMethod,
  listCommercePaymentMethodCredentialBindings,
  replaceCommercePaymentMethodCredentialBindings,
  saveCommercePaymentMethod,
} from 'sdkwork-router-admin-admin-api';
import {
  embeddedAdminDataTableClassName,
  embeddedAdminDataTableSlotProps,
  formatAdminDateTime,
  useAdminI18n,
} from 'sdkwork-router-admin-core';
import type {
  CredentialRecord,
  PaymentMethodCredentialBindingRecord,
  PaymentMethodRecord,
} from 'sdkwork-router-admin-types';
import { PaymentCredentialBindingsDialog } from './paymentCredentialBindingsDialog';
import { PaymentMethodDialog } from './paymentMethodDialog';
import {
  ConfirmActionDialog,
  joinCommaSeparatedList,
  resolveStatusVariant,
} from './paymentShared';

type PaymentMethodManagerSectionProps = {
  credentials: CredentialRecord[];
  onRefresh: () => Promise<void>;
  paymentMethods: PaymentMethodRecord[];
  paymentMethodsError: string | null;
  paymentMethodsLoading: boolean;
};

export function PaymentMethodManagerSection({
  credentials,
  onRefresh,
  paymentMethods,
  paymentMethodsError,
  paymentMethodsLoading,
}: PaymentMethodManagerSectionProps) {
  const { formatNumber, t } = useAdminI18n();
  const [dialogOpen, setDialogOpen] = useState(false);
  const [draft, setDraft] = useState<PaymentMethodRecord | null>(null);
  const [bindingsDialogOpen, setBindingsDialogOpen] = useState(false);
  const [selectedMethod, setSelectedMethod] = useState<PaymentMethodRecord | null>(null);
  const [bindings, setBindings] = useState<PaymentMethodCredentialBindingRecord[]>([]);
  const [deleteTarget, setDeleteTarget] = useState<PaymentMethodRecord | null>(null);
  const [error, setError] = useState<string | null>(null);

  const enabledCount = paymentMethods.filter((record) => record.enabled).length;
  const webhookBackedCount = paymentMethods.filter(
    (record) => record.callback_strategy.includes('webhook'),
  ).length;
  const refundCapableCount = paymentMethods.filter((record) =>
    record.capability_codes.includes('refund')
    || record.capability_codes.includes('partial_refund'),
  ).length;

  const columns = useMemo<Array<DataTableColumn<PaymentMethodRecord>>>(
    () => [
      {
        id: 'method',
        header: t('Payment method'),
        cell: (row) => (
          <div className="space-y-1">
            <div className="font-medium text-[var(--sdk-color-text-primary)]">
              {row.display_name}
            </div>
            <div className="text-sm text-[var(--sdk-color-text-secondary)]">
              {row.payment_method_id}
            </div>
          </div>
        ),
      },
      {
        id: 'provider',
        header: t('Provider'),
        cell: (row) => (
          <div className="space-y-1">
            <div className="font-medium text-[var(--sdk-color-text-primary)]">
              {row.provider}
            </div>
            <div className="text-sm text-[var(--sdk-color-text-secondary)]">
              {row.channel} / {row.mode}
            </div>
          </div>
        ),
        width: 220,
      },
      {
        id: 'capabilities',
        header: t('Capabilities'),
        cell: (row) => joinCommaSeparatedList(row.capability_codes) || t('None'),
        width: 260,
      },
      {
        id: 'market',
        header: t('Scope'),
        cell: (row) => (
          <div className="space-y-1 text-sm">
            <div>{joinCommaSeparatedList(row.supported_currency_codes) || t('All currencies')}</div>
            <div className="text-[var(--sdk-color-text-secondary)]">
              {joinCommaSeparatedList(row.supported_country_codes) || t('All countries')}
            </div>
          </div>
        ),
        width: 220,
      },
      {
        id: 'status',
        header: t('Status'),
        cell: (row) => (
          <StatusBadge
            showIcon
            status={row.enabled ? t('Enabled') : t('Disabled')}
            variant={row.enabled ? 'success' : 'secondary'}
          />
        ),
        width: 160,
      },
      {
        id: 'callback',
        header: t('Callback'),
        cell: (row) => (
          <div className="space-y-1 text-sm">
            <div>{row.callback_strategy}</div>
            <div className="text-[var(--sdk-color-text-secondary)]">
              {row.webhook_path || t('No webhook path')}
            </div>
          </div>
        ),
        width: 240,
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
              onClick={() => {
                setDraft({ ...row });
                setDialogOpen(true);
              }}
              size="sm"
              type="button"
              variant="outline"
            >
              {t('Edit')}
            </Button>
            <Button
              onClick={() => void openBindings(row)}
              size="sm"
              type="button"
              variant="outline"
            >
              {t('Bindings')}
            </Button>
            <Button
              onClick={() => setDeleteTarget(row)}
              size="sm"
              type="button"
              variant="danger"
            >
              {t('Delete')}
            </Button>
          </div>
        ),
        width: 240,
      },
    ],
    [t],
  );

  async function handleSave(nextDraft: PaymentMethodRecord) {
    try {
      setError(null);
      const now = Date.now();
      await saveCommercePaymentMethod(nextDraft.payment_method_id, {
        ...nextDraft,
        created_at_ms: nextDraft.created_at_ms || now,
        updated_at_ms: now,
      });
      setDialogOpen(false);
      setDraft(null);
      await onRefresh();
    } catch (nextError) {
      setError(
        nextError instanceof Error ? nextError.message : 'Failed to save payment method.',
      );
    }
  }

  async function openBindings(paymentMethod: PaymentMethodRecord) {
    try {
      setError(null);
      const records = await listCommercePaymentMethodCredentialBindings(
        paymentMethod.payment_method_id,
      );
      setSelectedMethod(paymentMethod);
      setBindings(records);
      setBindingsDialogOpen(true);
    } catch (nextError) {
      setError(
        nextError instanceof Error
          ? nextError.message
          : 'Failed to load payment method bindings.',
      );
    }
  }

  async function handleSaveBindings(
    nextBindings: PaymentMethodCredentialBindingRecord[],
  ) {
    if (!selectedMethod) {
      return;
    }

    try {
      setError(null);
      const savedBindings = await replaceCommercePaymentMethodCredentialBindings(
        selectedMethod.payment_method_id,
        nextBindings,
      );
      setBindings(savedBindings);
      setBindingsDialogOpen(false);
      await onRefresh();
    } catch (nextError) {
      setError(
        nextError instanceof Error
          ? nextError.message
          : 'Failed to save payment method bindings.',
      );
    }
  }

  async function handleDelete() {
    if (!deleteTarget) {
      return;
    }

    try {
      setError(null);
      await deleteCommercePaymentMethod(deleteTarget.payment_method_id);
      setDeleteTarget(null);
      await onRefresh();
    } catch (nextError) {
      setError(
        nextError instanceof Error ? nextError.message : 'Failed to delete payment method.',
      );
    }
  }

  return (
    <>
      <Card className="p-0">
        <CardHeader className="space-y-4">
          <div className="flex flex-wrap items-start justify-between gap-3">
            <div className="space-y-1">
              <CardTitle>{t('Payment method management')}</CardTitle>
              <CardDescription>
                {t('Control method availability, capability exposure, scope policy, callback behavior, and encrypted secret bindings for the payment control plane.')}
              </CardDescription>
            </div>
            <Button
              onClick={() => {
                const now = Date.now();
                setDraft({
                  payment_method_id: '',
                  display_name: '',
                  description: '',
                  provider: 'stripe',
                  channel: 'checkout',
                  mode: 'live',
                  enabled: true,
                  sort_order: paymentMethods.length,
                  capability_codes: ['checkout', 'refund', 'partial_refund', 'webhook'],
                  supported_currency_codes: ['USD'],
                  supported_country_codes: ['US'],
                  supported_order_kinds: ['subscription', 'recharge_pack', 'custom_recharge'],
                  callback_strategy: 'webhook_signed',
                  webhook_path: '',
                  webhook_tolerance_seconds: 300,
                  replay_window_seconds: 300,
                  max_retry_count: 8,
                  config_json: '{\n  "success_url_template": "https://example.com/pay/success",\n  "cancel_url_template": "https://example.com/pay/cancel"\n}',
                  created_at_ms: now,
                  updated_at_ms: now,
                });
                setDialogOpen(true);
              }}
              type="button"
              variant="primary"
            >
              {t('Add payment method')}
            </Button>
          </div>

          {error ? (
            <div className="text-sm text-[var(--sdk-color-status-danger)]">{error}</div>
          ) : null}

          <div className="grid gap-4 md:grid-cols-3">
            <StatCard
              description={t('Methods currently published into operator inventory.')}
              label={t('Total methods')}
              value={formatNumber(paymentMethods.length)}
            />
            <StatCard
              description={t('Methods still eligible for portal checkout exposure.')}
              label={t('Enabled methods')}
              value={formatNumber(enabledCount)}
            />
            <StatCard
              description={t('Methods that can close refund workflows without manual finance intervention.')}
              label={t('Refund capable')}
              value={formatNumber(refundCapableCount)}
            />
          </div>

          <div className="grid gap-4 md:grid-cols-3">
            <div className="rounded-[var(--sdk-radius-card)] border border-[var(--sdk-color-border-default)] px-4 py-3 text-sm">
              <div className="font-medium text-[var(--sdk-color-text-primary)]">
                {t('Webhook-backed methods')}
              </div>
              <div className="mt-1 text-[var(--sdk-color-text-secondary)]">
                {formatNumber(webhookBackedCount)} / {formatNumber(paymentMethods.length)}
              </div>
            </div>
            <div className="rounded-[var(--sdk-radius-card)] border border-[var(--sdk-color-border-default)] px-4 py-3 text-sm">
              <div className="font-medium text-[var(--sdk-color-text-primary)]">
                {t('Bound credentials')}
              </div>
              <div className="mt-1 text-[var(--sdk-color-text-secondary)]">
                {t('Secret bindings are edited per method and never stored in config_json.')}
              </div>
            </div>
            <div className="rounded-[var(--sdk-radius-card)] border border-[var(--sdk-color-border-default)] px-4 py-3 text-sm">
              <div className="font-medium text-[var(--sdk-color-text-primary)]">
                {t('Operational posture')}
              </div>
              <div className="mt-1 text-[var(--sdk-color-text-secondary)]">
                {error
                  ? error
                  : paymentMethodsError
                    ? paymentMethodsError
                    : paymentMethodsLoading
                      ? t('Refreshing payment method inventory...')
                      : t('Inventory synchronized from admin commerce endpoints.')}
              </div>
            </div>
          </div>
        </CardHeader>
        <CardContent className="p-0">
          <DataTable
            className={embeddedAdminDataTableClassName}
            columns={columns}
            emptyDescription={t('Create the first live payment method to expose a real provider-backed checkout path, callback policy, and refund capability.')}
            emptyTitle={t('No payment methods configured')}
            getRowId={(row) => row.payment_method_id}
            rows={paymentMethods}
            slotProps={embeddedAdminDataTableSlotProps}
            stickyHeader
          />
        </CardContent>
      </Card>

      <PaymentMethodDialog
        draft={draft}
        onOpenChange={(open) => {
          setDialogOpen(open);
          if (!open) {
            setDraft(null);
          }
        }}
        onSubmit={handleSave}
        open={dialogOpen}
      />

      <PaymentCredentialBindingsDialog
        credentials={credentials}
        existingBindings={bindings}
        onOpenChange={(open) => {
          setBindingsDialogOpen(open);
          if (!open) {
            setSelectedMethod(null);
            setBindings([]);
          }
        }}
        onSubmit={handleSaveBindings}
        open={bindingsDialogOpen}
        paymentMethod={selectedMethod}
      />

      <ConfirmActionDialog
        description={
          deleteTarget ? (
            <div className="space-y-2">
              <div>{deleteTarget.display_name}</div>
              <StatusBadge
                showIcon
                status={deleteTarget.callback_strategy}
                variant={resolveStatusVariant(deleteTarget.callback_strategy)}
              />
            </div>
          ) : (
            t('Delete the selected payment method.')
          )
        }
        onConfirm={() => void handleDelete()}
        onOpenChange={(open) => {
          if (!open) {
            setDeleteTarget(null);
          }
        }}
        open={deleteTarget != null}
        title={t('Delete payment method')}
      />
    </>
  );
}
