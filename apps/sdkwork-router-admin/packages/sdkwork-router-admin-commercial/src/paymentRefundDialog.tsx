import { useEffect, useMemo, useState, type ChangeEvent, type FormEvent } from 'react';
import {
  Button,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  Input,
  Textarea,
} from '@sdkwork/ui-pc-react';
import { useAdminI18n } from 'sdkwork-router-admin-core';
import type {
  AdminCommerceRefundCreateRequest,
  CommerceOrderRecord,
  CommercePaymentAttemptRecord,
} from 'sdkwork-router-admin-types';
import { DialogField, SelectField, createLocalId } from './paymentShared';

type PaymentRefundDialogProps = {
  attempts: CommercePaymentAttemptRecord[];
  onOpenChange: (open: boolean) => void;
  onSubmit: (request: AdminCommerceRefundCreateRequest) => Promise<void>;
  open: boolean;
  order: CommerceOrderRecord | null;
};

type RefundDraft = {
  payment_attempt_id: string;
  amount_minor: string;
  reason: string;
  idempotency_key: string;
};

export function PaymentRefundDialog({
  attempts,
  onOpenChange,
  onSubmit,
  open,
  order,
}: PaymentRefundDialogProps) {
  const { formatNumber, t } = useAdminI18n();
  const defaultAttemptId = attempts[0]?.payment_attempt_id ?? '';
  const [draft, setDraft] = useState<RefundDraft>({
    payment_attempt_id: defaultAttemptId,
    amount_minor: '',
    reason: '',
    idempotency_key: '',
  });

  useEffect(() => {
    setDraft({
      payment_attempt_id: attempts[0]?.payment_attempt_id ?? '',
      amount_minor: '',
      reason: '',
      idempotency_key: '',
    });
  }, [attempts, open]);

  const attemptOptions = useMemo(
    () =>
      attempts.map((attempt) => ({
        label: `${attempt.payment_method_id} / ${attempt.status} / ${attempt.currency_code} ${attempt.amount_minor}`,
        value: attempt.payment_attempt_id,
      })),
    [attempts],
  );

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();

    await onSubmit({
      payment_attempt_id: draft.payment_attempt_id.trim() || undefined,
      amount_minor: draft.amount_minor.trim()
        ? Number(draft.amount_minor)
        : undefined,
      reason: draft.reason.trim() || undefined,
      idempotency_key: draft.idempotency_key.trim() || createLocalId('refund'),
    });
  }

  if (!order) {
    return null;
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="w-[min(92vw,40rem)]">
        <DialogHeader>
          <DialogTitle>{t('Create refund')}</DialogTitle>
          <DialogDescription>
            {t('Submit a provider-backed refund request and drive settlement status forward for the selected order.')}
          </DialogDescription>
        </DialogHeader>

        <form className="space-y-4" onSubmit={handleSubmit}>
          <div className="rounded-[var(--sdk-radius-card)] border border-[var(--sdk-color-border-default)] px-4 py-3 text-sm text-[var(--sdk-color-text-secondary)]">
            <div className="font-medium text-[var(--sdk-color-text-primary)]">
              {order.target_name}
            </div>
            <div>{order.order_id}</div>
            <div className="mt-2">
              {t('Refundable amount: {amount}', {
                amount: formatNumber(order.refundable_amount_minor),
              })}
            </div>
          </div>

          {attemptOptions.length ? (
            <SelectField
              label={t('Payment attempt')}
              onValueChange={(value) => setDraft((current) => ({ ...current, payment_attempt_id: value }))}
              options={attemptOptions}
              placeholder={t('Select payment attempt')}
              value={draft.payment_attempt_id || attemptOptions[0]?.value || ''}
            />
          ) : null}

          <DialogField
            htmlFor="refund-amount-minor"
            label={t('Refund amount minor')}
            description={t('Leave empty to refund the remaining refundable amount.')}
          >
            <Input
              id="refund-amount-minor"
              inputMode="numeric"
              onChange={(event: ChangeEvent<HTMLInputElement>) =>
                setDraft((current) => ({ ...current, amount_minor: event.target.value }))
              }
              placeholder={String(order.refundable_amount_minor)}
              type="number"
              value={draft.amount_minor}
            />
          </DialogField>

          <DialogField htmlFor="refund-reason" label={t('Reason')}>
            <Textarea
              id="refund-reason"
              onChange={(event: ChangeEvent<HTMLTextAreaElement>) =>
                setDraft((current) => ({ ...current, reason: event.target.value }))
              }
              rows={4}
              value={draft.reason}
            />
          </DialogField>

          <DialogField
            htmlFor="refund-idempotency-key"
            label={t('Idempotency key')}
            description={t('Optional operator-specified idempotency key for retry-safe refund creation.')}
          >
            <Input
              autoComplete="off"
              id="refund-idempotency-key"
              onChange={(event: ChangeEvent<HTMLInputElement>) =>
                setDraft((current) => ({ ...current, idempotency_key: event.target.value }))
              }
              placeholder={t('Leave blank to auto-generate')}
              value={draft.idempotency_key}
            />
          </DialogField>

          <DialogFooter>
            <Button onClick={() => onOpenChange(false)} type="button" variant="outline">
              {t('Cancel')}
            </Button>
            <Button type="submit" variant="primary">
              {t('Create refund')}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
