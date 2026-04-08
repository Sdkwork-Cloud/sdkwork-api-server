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
} from '@sdkwork/ui-pc-react';
import { useAdminI18n } from 'sdkwork-router-admin-core';
import type {
  CredentialRecord,
  PaymentMethodCredentialBindingRecord,
  PaymentMethodRecord,
} from 'sdkwork-router-admin-types';
import { DialogField, SelectField, createLocalId } from './paymentShared';

type PaymentCredentialBindingsDialogProps = {
  credentials: CredentialRecord[];
  existingBindings: PaymentMethodCredentialBindingRecord[];
  onOpenChange: (open: boolean) => void;
  onSubmit: (bindings: PaymentMethodCredentialBindingRecord[]) => Promise<void>;
  open: boolean;
  paymentMethod: PaymentMethodRecord | null;
};

const defaultUsageKinds = [
  'api_secret',
  'publishable_key',
  'webhook_secret',
  'refund_secret',
] as const;

export function PaymentCredentialBindingsDialog({
  credentials,
  existingBindings,
  onOpenChange,
  onSubmit,
  open,
  paymentMethod,
}: PaymentCredentialBindingsDialogProps) {
  const { t } = useAdminI18n();
  const [bindings, setBindings] = useState<PaymentMethodCredentialBindingRecord[]>([]);

  useEffect(() => {
    setBindings(existingBindings);
  }, [existingBindings]);

  const credentialOptions = useMemo(
    () =>
      credentials.map((credential) => ({
        label: `${credential.tenant_id} / ${credential.provider_id} / ${credential.key_reference}`,
        value: buildCredentialKey(credential),
      })),
    [credentials],
  );

  function addBinding() {
    if (!paymentMethod) {
      return;
    }

    const now = Date.now();
    const firstCredential = credentials[0];
    setBindings((current) => [
      ...current,
      {
        binding_id: createLocalId('payment_binding'),
        payment_method_id: paymentMethod.payment_method_id,
        usage_kind: defaultUsageKinds[0],
        credential_tenant_id: firstCredential?.tenant_id ?? '',
        credential_provider_id: firstCredential?.provider_id ?? '',
        credential_key_reference: firstCredential?.key_reference ?? '',
        created_at_ms: now,
        updated_at_ms: now,
      },
    ]);
  }

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    await onSubmit(
      bindings.map((binding) => ({
        ...binding,
        payment_method_id: paymentMethod?.payment_method_id ?? binding.payment_method_id,
        updated_at_ms: Date.now(),
      })),
    );
  }

  if (!paymentMethod) {
    return null;
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="w-[min(96vw,70rem)]">
        <DialogHeader>
          <DialogTitle>{t('Payment method credential bindings')}</DialogTitle>
          <DialogDescription>
            {t('Bind non-plaintext credentials to concrete usage kinds so checkout, webhook verification, and refund execution can each read the minimum secret they need.')}
          </DialogDescription>
        </DialogHeader>

        <form className="space-y-4" onSubmit={handleSubmit}>
          <div className="rounded-[var(--sdk-radius-card)] border border-[var(--sdk-color-border-default)] px-4 py-3 text-sm text-[var(--sdk-color-text-secondary)]">
            <div className="font-medium text-[var(--sdk-color-text-primary)]">
              {paymentMethod.display_name}
            </div>
            <div>{paymentMethod.payment_method_id}</div>
          </div>

          <div className="space-y-4">
            {bindings.map((binding, index) => (
              <div
                className="grid gap-4 rounded-[var(--sdk-radius-card)] border border-[var(--sdk-color-border-default)] p-4 md:grid-cols-2 xl:grid-cols-[0.8fr_1.3fr_auto]"
                key={binding.binding_id}
              >
                <DialogField
                  htmlFor={`binding-usage-${binding.binding_id}`}
                  label={t('Usage kind')}
                  description={t('Examples: api_secret, webhook_secret, publishable_key, refund_secret.')}
                >
                  <Input
                    autoComplete="off"
                    id={`binding-usage-${binding.binding_id}`}
                    onChange={(event: ChangeEvent<HTMLInputElement>) =>
                      setBindings((current) =>
                        current.map((record) =>
                          record.binding_id === binding.binding_id
                            ? { ...record, usage_kind: event.target.value }
                            : record,
                        ),
                      )
                    }
                    value={binding.usage_kind}
                  />
                </DialogField>

                {credentialOptions.length ? (
                  <SelectField
                    label={t('Credential')}
                    onValueChange={(value) => {
                      const [
                        credentialTenantId,
                        credentialProviderId,
                        credentialKeyReference,
                      ] = value.split('::');
                      setBindings((current) =>
                        current.map((record) =>
                          record.binding_id === binding.binding_id
                            ? {
                                ...record,
                                credential_tenant_id: credentialTenantId ?? '',
                                credential_provider_id: credentialProviderId ?? '',
                                credential_key_reference: credentialKeyReference ?? '',
                              }
                            : record,
                        ),
                      );
                    }}
                    options={credentialOptions}
                    placeholder={t('Select credential')}
                    value={resolveCredentialSelectionValue(binding, credentialOptions)}
                  />
                ) : (
                  <div className="grid gap-3 md:grid-cols-3">
                    <DialogField label={t('Tenant id')}>
                      <Input
                        autoComplete="off"
                        onChange={(event: ChangeEvent<HTMLInputElement>) =>
                          setBindings((current) =>
                            current.map((record) =>
                              record.binding_id === binding.binding_id
                                ? {
                                    ...record,
                                    credential_tenant_id: event.target.value,
                                  }
                                : record,
                            ),
                          )
                        }
                        value={binding.credential_tenant_id}
                      />
                    </DialogField>
                    <DialogField label={t('Provider id')}>
                      <Input
                        autoComplete="off"
                        onChange={(event: ChangeEvent<HTMLInputElement>) =>
                          setBindings((current) =>
                            current.map((record) =>
                              record.binding_id === binding.binding_id
                                ? {
                                    ...record,
                                    credential_provider_id: event.target.value,
                                  }
                                : record,
                            ),
                          )
                        }
                        value={binding.credential_provider_id}
                      />
                    </DialogField>
                    <DialogField label={t('Key reference')}>
                      <Input
                        autoComplete="off"
                        onChange={(event: ChangeEvent<HTMLInputElement>) =>
                          setBindings((current) =>
                            current.map((record) =>
                              record.binding_id === binding.binding_id
                                ? {
                                    ...record,
                                    credential_key_reference: event.target.value,
                                  }
                                : record,
                            ),
                          )
                        }
                        value={binding.credential_key_reference}
                      />
                    </DialogField>
                  </div>
                )}

                <div className="flex items-end">
                  <Button
                    onClick={() =>
                      setBindings((current) =>
                        current.filter((record) => record.binding_id !== binding.binding_id),
                      )
                    }
                    type="button"
                    variant="danger"
                  >
                    {t('Remove')}
                  </Button>
                </div>

                <div className="md:col-span-2 xl:col-span-3">
                  <div className="text-xs text-[var(--sdk-color-text-secondary)]">
                    {t('Binding #{index}', { index: index + 1 })} / {binding.binding_id}
                  </div>
                </div>
              </div>
            ))}
          </div>

          {!bindings.length ? (
            <div className="rounded-[var(--sdk-radius-card)] border border-dashed border-[var(--sdk-color-border-default)] px-4 py-6 text-sm text-[var(--sdk-color-text-secondary)]">
              {t('No credential bindings yet. Add at least one binding so the payment adapter can resolve secrets for checkout or webhook verification.')}
            </div>
          ) : null}

          <DialogFooter className="flex flex-wrap gap-3">
            <Button onClick={addBinding} type="button" variant="outline">
              {t('Add binding')}
            </Button>
            <div className="flex-1" />
            <Button onClick={() => onOpenChange(false)} type="button" variant="outline">
              {t('Cancel')}
            </Button>
            <Button type="submit" variant="primary">
              {t('Save bindings')}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}

function buildCredentialKey(credential: CredentialRecord): string {
  return buildCredentialValue(
    credential.tenant_id,
    credential.provider_id,
    credential.key_reference,
  );
}

function buildCredentialValue(
  credentialTenantId: string,
  credentialProviderId: string,
  credentialKeyReference: string,
): string {
  return [credentialTenantId, credentialProviderId, credentialKeyReference].join('::');
}

function resolveCredentialSelectionValue(
  binding: PaymentMethodCredentialBindingRecord,
  credentialOptions: Array<{ label: string; value: string }>,
): string {
  const candidate = buildCredentialValue(
    binding.credential_tenant_id,
    binding.credential_provider_id,
    binding.credential_key_reference,
  );
  if (credentialOptions.some((option) => option.value === candidate)) {
    return candidate;
  }
  return credentialOptions[0]?.value ?? '';
}
