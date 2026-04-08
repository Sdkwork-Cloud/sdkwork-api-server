import { useEffect, useState, type ChangeEvent, type FormEvent } from 'react';
import {
  Button,
  Checkbox,
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
import type { PaymentMethodRecord } from 'sdkwork-router-admin-types';
import {
  DialogField,
  parseCommaSeparatedList,
  joinCommaSeparatedList,
} from './paymentShared';

type PaymentMethodDialogProps = {
  draft: PaymentMethodRecord | null;
  onOpenChange: (open: boolean) => void;
  onSubmit: (draft: PaymentMethodRecord) => Promise<void>;
  open: boolean;
};

export function PaymentMethodDialog({
  draft,
  onOpenChange,
  onSubmit,
  open,
}: PaymentMethodDialogProps) {
  const { t } = useAdminI18n();
  const [localDraft, setLocalDraft] = useState<PaymentMethodRecord | null>(draft);

  useEffect(() => {
    setLocalDraft(draft);
  }, [draft]);

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!localDraft) {
      return;
    }

    await onSubmit(localDraft);
  }

  function updateDraft(patch: Partial<PaymentMethodRecord>) {
    setLocalDraft((current) => (current ? { ...current, ...patch } : current));
  }

  if (!localDraft) {
    return null;
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="w-[min(96vw,72rem)]">
        <DialogHeader>
          <DialogTitle>
            {localDraft.payment_method_id ? t('Edit payment method') : t('New payment method')}
          </DialogTitle>
          <DialogDescription>
            {t('Configure provider capabilities, market scope, callback policy, and raw provider configuration for a live payment method.')}
          </DialogDescription>
        </DialogHeader>

        <form className="space-y-6" onSubmit={handleSubmit}>
          <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
            <DialogField
              htmlFor="payment-method-id"
              label={t('Payment method id')}
              description={t('Stable identifier used by portal checkout and provider webhook routing.')}
            >
              <Input
                autoComplete="off"
                id="payment-method-id"
                onChange={(event: ChangeEvent<HTMLInputElement>) =>
                  updateDraft({ payment_method_id: event.target.value })
                }
                placeholder={t('stripe_checkout')}
                value={localDraft.payment_method_id}
              />
            </DialogField>

            <DialogField htmlFor="payment-method-name" label={t('Display name')}>
              <Input
                autoComplete="off"
                id="payment-method-name"
                onChange={(event: ChangeEvent<HTMLInputElement>) =>
                  updateDraft({ display_name: event.target.value })
                }
                placeholder={t('Stripe Checkout')}
                value={localDraft.display_name}
              />
            </DialogField>

            <DialogField htmlFor="payment-method-description" label={t('Description')}>
              <Input
                autoComplete="off"
                id="payment-method-description"
                onChange={(event: ChangeEvent<HTMLInputElement>) =>
                  updateDraft({ description: event.target.value })
                }
                placeholder={t('Primary card checkout for self-serve recharge and subscription orders.')}
                value={localDraft.description}
              />
            </DialogField>

            <DialogField htmlFor="payment-method-provider" label={t('Provider')}>
              <Input
                autoComplete="off"
                id="payment-method-provider"
                onChange={(event: ChangeEvent<HTMLInputElement>) =>
                  updateDraft({ provider: event.target.value })
                }
                placeholder={t('stripe')}
                value={localDraft.provider}
              />
            </DialogField>

            <DialogField htmlFor="payment-method-channel" label={t('Channel')}>
              <Input
                autoComplete="off"
                id="payment-method-channel"
                onChange={(event: ChangeEvent<HTMLInputElement>) =>
                  updateDraft({ channel: event.target.value })
                }
                placeholder={t('checkout')}
                value={localDraft.channel}
              />
            </DialogField>

            <DialogField
              htmlFor="payment-method-mode"
              label={t('Mode')}
              description={t('Use live for production traffic and test for sandbox or shadow routing.')}
            >
              <Input
                autoComplete="off"
                id="payment-method-mode"
                onChange={(event: ChangeEvent<HTMLInputElement>) =>
                  updateDraft({ mode: event.target.value })
                }
                placeholder={t('live')}
                value={localDraft.mode}
              />
            </DialogField>

            <DialogField htmlFor="payment-method-sort-order" label={t('Sort order')}>
              <Input
                id="payment-method-sort-order"
                inputMode="numeric"
                onChange={(event: ChangeEvent<HTMLInputElement>) =>
                  updateDraft({ sort_order: Number(event.target.value || 0) })
                }
                type="number"
                value={String(localDraft.sort_order)}
              />
            </DialogField>

            <DialogField htmlFor="payment-method-callback-strategy" label={t('Callback strategy')}>
              <Input
                autoComplete="off"
                id="payment-method-callback-strategy"
                onChange={(event: ChangeEvent<HTMLInputElement>) =>
                  updateDraft({ callback_strategy: event.target.value })
                }
                placeholder={t('webhook_signed')}
                value={localDraft.callback_strategy}
              />
            </DialogField>

            <DialogField htmlFor="payment-method-webhook-path" label={t('Webhook path')}>
              <Input
                autoComplete="off"
                id="payment-method-webhook-path"
                onChange={(event: ChangeEvent<HTMLInputElement>) =>
                  updateDraft({ webhook_path: event.target.value.trim() || null })
                }
                placeholder={t('/portal/commerce/webhooks/stripe/stripe_checkout')}
                value={localDraft.webhook_path ?? ''}
              />
            </DialogField>

            <DialogField htmlFor="payment-method-tolerance" label={t('Webhook tolerance seconds')}>
              <Input
                id="payment-method-tolerance"
                inputMode="numeric"
                onChange={(event: ChangeEvent<HTMLInputElement>) =>
                  updateDraft({
                    webhook_tolerance_seconds: Number(event.target.value || 0),
                  })
                }
                type="number"
                value={String(localDraft.webhook_tolerance_seconds)}
              />
            </DialogField>

            <DialogField htmlFor="payment-method-replay-window" label={t('Replay window seconds')}>
              <Input
                id="payment-method-replay-window"
                inputMode="numeric"
                onChange={(event: ChangeEvent<HTMLInputElement>) =>
                  updateDraft({
                    replay_window_seconds: Number(event.target.value || 0),
                  })
                }
                type="number"
                value={String(localDraft.replay_window_seconds)}
              />
            </DialogField>

            <DialogField htmlFor="payment-method-max-retry" label={t('Max retry count')}>
              <Input
                id="payment-method-max-retry"
                inputMode="numeric"
                onChange={(event: ChangeEvent<HTMLInputElement>) =>
                  updateDraft({ max_retry_count: Number(event.target.value || 0) })
                }
                type="number"
                value={String(localDraft.max_retry_count)}
              />
            </DialogField>
          </div>

          <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
            <DialogField
              htmlFor="payment-method-capabilities"
              label={t('Capabilities')}
              description={t('Comma-separated capability codes such as checkout, refund, partial_refund, webhook.')}
            >
              <Input
                autoComplete="off"
                id="payment-method-capabilities"
                onChange={(event: ChangeEvent<HTMLInputElement>) =>
                  updateDraft({
                    capability_codes: parseCommaSeparatedList(event.target.value),
                  })
                }
                placeholder={t('checkout, refund, partial_refund, webhook')}
                value={joinCommaSeparatedList(localDraft.capability_codes)}
              />
            </DialogField>

            <DialogField
              htmlFor="payment-method-currencies"
              label={t('Currencies')}
              description={t('Comma-separated ISO currency codes. Leave blank to allow all configured currencies.')}
            >
              <Input
                autoComplete="off"
                id="payment-method-currencies"
                onChange={(event: ChangeEvent<HTMLInputElement>) =>
                  updateDraft({
                    supported_currency_codes: parseCommaSeparatedList(event.target.value),
                  })
                }
                placeholder={t('USD, CNY, HKD')}
                value={joinCommaSeparatedList(localDraft.supported_currency_codes)}
              />
            </DialogField>

            <DialogField
              htmlFor="payment-method-countries"
              label={t('Countries')}
              description={t('Comma-separated ISO country codes to restrict method availability.')}
            >
              <Input
                autoComplete="off"
                id="payment-method-countries"
                onChange={(event: ChangeEvent<HTMLInputElement>) =>
                  updateDraft({
                    supported_country_codes: parseCommaSeparatedList(event.target.value),
                  })
                }
                placeholder={t('US, CN, HK')}
                value={joinCommaSeparatedList(localDraft.supported_country_codes)}
              />
            </DialogField>

            <DialogField
              htmlFor="payment-method-order-kinds"
              label={t('Order kinds')}
              description={t('Comma-separated order kinds such as subscription, recharge_pack, custom_recharge.')}
            >
              <Input
                autoComplete="off"
                id="payment-method-order-kinds"
                onChange={(event: ChangeEvent<HTMLInputElement>) =>
                  updateDraft({
                    supported_order_kinds: parseCommaSeparatedList(event.target.value),
                  })
                }
                placeholder={t('subscription, recharge_pack, custom_recharge')}
                value={joinCommaSeparatedList(localDraft.supported_order_kinds)}
              />
            </DialogField>

            <div className="flex items-start gap-3 rounded-[var(--sdk-radius-control)] border border-[var(--sdk-color-border-default)] px-4 py-3 md:col-span-2 xl:col-span-2">
              <Checkbox
                checked={localDraft.enabled}
                onCheckedChange={(checked: boolean | 'indeterminate') =>
                  updateDraft({ enabled: checked === true })
                }
              />
              <div className="space-y-1">
                <div className="font-medium text-[var(--sdk-color-text-primary)]">
                  {t('Enabled for live selection')}
                </div>
                <div className="text-sm text-[var(--sdk-color-text-secondary)]">
                  {t('Disabled methods stay in operator inventory and keep historical bindings, but portal checkout will stop offering them.')}
                </div>
              </div>
            </div>
          </div>

          <DialogField
            htmlFor="payment-method-config-json"
            label={t('Provider configuration JSON')}
            description={t('Raw provider-specific configuration passed into the commerce payment adapter. Keep only non-secret metadata here and bind secrets separately.')}
          >
            <Textarea
              id="payment-method-config-json"
              onChange={(event: ChangeEvent<HTMLTextAreaElement>) =>
                updateDraft({ config_json: event.target.value })
              }
              rows={10}
              value={localDraft.config_json}
            />
          </DialogField>

          <DialogFooter>
            <Button onClick={() => onOpenChange(false)} type="button" variant="outline">
              {t('Cancel')}
            </Button>
            <Button
              disabled={
                !localDraft.payment_method_id.trim()
                || !localDraft.display_name.trim()
                || !localDraft.provider.trim()
                || !localDraft.channel.trim()
              }
              type="submit"
              variant="primary"
            >
              {t('Save payment method')}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
