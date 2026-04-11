import type {
  ChangeEvent,
  Dispatch,
  FormEvent,
  SetStateAction,
} from 'react';
import {
  Button,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  FormActions,
  FormGrid,
  FormSection,
  Input,
  Textarea,
} from '@sdkwork/ui-pc-react';
import {
  recommendedModelPriceSourceKind,
  useAdminI18n,
} from 'sdkwork-router-admin-core';
import type { AdminPageProps } from 'sdkwork-router-admin-types';

import {
  DialogField,
  MODEL_PRICE_SOURCE_OPTIONS,
  PRICE_UNIT_OPTIONS,
  SelectField,
  type ModelPriceDraft,
} from './shared';

type CatalogModelPriceDialogProps = {
  editingModelPriceKey: string | null;
  modelPriceDraft: ModelPriceDraft;
  onOpenChange: (open: boolean) => void;
  onSubmit: (event: FormEvent<HTMLFormElement>) => void;
  open: boolean;
  setModelPriceDraft: Dispatch<SetStateAction<ModelPriceDraft>>;
  snapshot: AdminPageProps['snapshot'];
};

export function CatalogModelPriceDialog({
  editingModelPriceKey,
  modelPriceDraft,
  onOpenChange,
  onSubmit,
  open,
  setModelPriceDraft,
  snapshot,
}: CatalogModelPriceDialogProps) {
  const { t } = useAdminI18n();
  const eligibleProviderIds = new Set(
    snapshot.providerModels
      .filter(
        (record) =>
          record.channel_id === modelPriceDraft.channel_id
          && record.model_id === modelPriceDraft.model_id
          && record.is_active,
      )
      .map((record) => record.proxy_provider_id),
  );
  const providerOptions = snapshot.providers
    .filter((provider) =>
      eligibleProviderIds.size === 0 || eligibleProviderIds.has(provider.id),
    )
    .map((provider) => ({
      label: `${provider.display_name} (${provider.id})`,
      value: provider.id,
    }));
  const selectedProvider = snapshot.providers.find(
    (provider) => provider.id === modelPriceDraft.proxy_provider_id,
  );
  const selectedProviderModel = snapshot.providerModels.find(
    (record) =>
      record.proxy_provider_id === modelPriceDraft.proxy_provider_id
      && record.channel_id === modelPriceDraft.channel_id
      && record.model_id === modelPriceDraft.model_id,
  );

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="w-[min(92vw,64rem)]">
        <DialogHeader>
          <DialogTitle>
            {editingModelPriceKey ? t('Edit model pricing') : t('Add model pricing')}
          </DialogTitle>
          <DialogDescription>
            {t('Provider-specific pricing rows stay aligned with provider-supported canonical publications and keep official, proxy, and local pricing posture explicit.')}
          </DialogDescription>
        </DialogHeader>
        <form className="space-y-6" onSubmit={onSubmit}>
          <FormSection title={t('Pricing row')}>
            <FormGrid columns={2}>
              <DialogField label={t('Channel')}>
                <Input disabled value={modelPriceDraft.channel_id} />
              </DialogField>
              <DialogField label={t('Model')}>
                <Input disabled value={modelPriceDraft.model_id} />
              </DialogField>
              <SelectField
                label={t('Provider')}
                onValueChange={(value) =>
                  setModelPriceDraft((current) => {
                    const provider = snapshot.providers.find((entry) => entry.id === value);
                    return {
                      ...current,
                      proxy_provider_id: value,
                      price_source_kind:
                        recommendedModelPriceSourceKind(provider),
                    };
                  })
                }
                options={providerOptions}
                value={modelPriceDraft.proxy_provider_id}
              />
              <DialogField
                label={t('Provider model mapping')}
                description={selectedProviderModel
                  ? t('Provider model id: {id}', {
                      id: selectedProviderModel.provider_model_id,
                    })
                  : t('Pricing can only be attached to provider-model records that already exist.')}
              >
                <Input
                  disabled
                  value={selectedProviderModel?.provider_model_family || t('Provider model family not set')}
                />
              </DialogField>
              <DialogField htmlFor="price-currency" label={t('Currency code')}>
                <Input
                  id="price-currency"
                  onChange={(event: ChangeEvent<HTMLInputElement>) =>
                    setModelPriceDraft((current) => ({
                      ...current,
                      currency_code: event.target.value,
                    }))
                  }
                  required
                  value={modelPriceDraft.currency_code}
                />
              </DialogField>
              <SelectField
                label={t('Price unit')}
                onValueChange={(value) =>
                  setModelPriceDraft((current) => ({
                    ...current,
                    price_unit: value,
                  }))
                }
                options={PRICE_UNIT_OPTIONS.map((option) => ({
                  label: t(option.label),
                  value: option.value,
                }))}
                value={modelPriceDraft.price_unit}
              />
              <SelectField
                label={t('Price source')}
                onValueChange={(value) =>
                  setModelPriceDraft((current) => ({
                    ...current,
                    price_source_kind: value,
                  }))
                }
                options={MODEL_PRICE_SOURCE_OPTIONS.map((option) => ({
                  label: t(option.label),
                  value: option.value,
                }))}
                value={modelPriceDraft.price_source_kind}
              />
              <DialogField
                label={t('Pricing guidance')}
                description={selectedProvider
                  ? t(
                      'Recommended source kind for this provider is {kind}. Official providers usually stay official, proxy providers stay proxy, and Ollama/local runtimes stay local.',
                      {
                        kind: recommendedModelPriceSourceKind(selectedProvider),
                      },
                    )
                  : t('Select a provider-model to inherit the recommended pricing posture.')}
              >
                <Input
                  disabled
                  value={selectedProvider?.display_name ?? t('No provider selected')}
                />
              </DialogField>
              <DialogField htmlFor="price-input" label={t('Input price')}>
                <Input
                  id="price-input"
                  onChange={(event: ChangeEvent<HTMLInputElement>) =>
                    setModelPriceDraft((current) => ({
                      ...current,
                      input_price: event.target.value,
                    }))
                  }
                  required
                  type="number"
                  value={modelPriceDraft.input_price}
                />
              </DialogField>
              <DialogField htmlFor="price-output" label={t('Output price')}>
                <Input
                  id="price-output"
                  onChange={(event: ChangeEvent<HTMLInputElement>) =>
                    setModelPriceDraft((current) => ({
                      ...current,
                      output_price: event.target.value,
                    }))
                  }
                  required
                  type="number"
                  value={modelPriceDraft.output_price}
                />
              </DialogField>
              <DialogField htmlFor="price-cache-read" label={t('Cache read')}>
                <Input
                  id="price-cache-read"
                  onChange={(event: ChangeEvent<HTMLInputElement>) =>
                    setModelPriceDraft((current) => ({
                      ...current,
                      cache_read_price: event.target.value,
                    }))
                  }
                  required
                  type="number"
                  value={modelPriceDraft.cache_read_price}
                />
              </DialogField>
              <DialogField htmlFor="price-cache-write" label={t('Cache write')}>
                <Input
                  id="price-cache-write"
                  onChange={(event: ChangeEvent<HTMLInputElement>) =>
                    setModelPriceDraft((current) => ({
                      ...current,
                      cache_write_price: event.target.value,
                    }))
                  }
                  required
                  type="number"
                  value={modelPriceDraft.cache_write_price}
                />
              </DialogField>
              <DialogField htmlFor="price-request" label={t('Request price')}>
                <Input
                  id="price-request"
                  onChange={(event: ChangeEvent<HTMLInputElement>) =>
                    setModelPriceDraft((current) => ({
                      ...current,
                      request_price: event.target.value,
                    }))
                  }
                  required
                  type="number"
                  value={modelPriceDraft.request_price}
                />
              </DialogField>
              <SelectField<'active' | 'inactive'>
                label={t('Status')}
                onValueChange={(value) =>
                  setModelPriceDraft((current) => ({
                    ...current,
                    is_active: value === 'active',
                  }))
                }
                options={[
                  { label: t('Active'), value: 'active' },
                  { label: t('Inactive'), value: 'inactive' },
                ]}
                value={modelPriceDraft.is_active ? 'active' : 'inactive'}
              />
              <DialogField
                htmlFor="price-billing-notes"
                label={t('Billing notes')}
                description={t('Describe official billing posture, pass-through proxy policy, or local cost-allocation assumptions.')}
              >
                <Textarea
                  id="price-billing-notes"
                  onChange={(event: ChangeEvent<HTMLTextAreaElement>) =>
                    setModelPriceDraft((current) => ({
                      ...current,
                      billing_notes: event.target.value,
                    }))
                  }
                  rows={4}
                  value={modelPriceDraft.billing_notes}
                />
              </DialogField>
              <DialogField
                htmlFor="price-pricing-tiers-json"
                label={t('Pricing tiers JSON')}
                description={t('Optional JSON array for prompt-length, modality, cache-window, or request-specific price tiers.')}
              >
                <Textarea
                  id="price-pricing-tiers-json"
                  onChange={(event: ChangeEvent<HTMLTextAreaElement>) =>
                    setModelPriceDraft((current) => ({
                      ...current,
                      pricing_tiers_json: event.target.value,
                    }))
                  }
                  placeholder={`[
  {
    "tier_id": "default",
    "display_name": "Default",
    "condition_kind": "default",
    "currency_code": "USD",
    "price_unit": "per_1m_tokens",
    "input_price": 2.5,
    "output_price": 10,
    "cache_read_price": 0.3,
    "cache_write_price": 1,
    "request_price": 0
  }
]`}
                  rows={10}
                  value={modelPriceDraft.pricing_tiers_json}
                />
              </DialogField>
            </FormGrid>
          </FormSection>
          <FormActions>
            <Button onClick={() => onOpenChange(false)} type="button" variant="outline">
              {t('Cancel')}
            </Button>
            <Button type="submit" variant="primary">
              {editingModelPriceKey ? t('Save pricing') : t('Add pricing')}
            </Button>
          </FormActions>
        </form>
      </DialogContent>
    </Dialog>
  );
}
