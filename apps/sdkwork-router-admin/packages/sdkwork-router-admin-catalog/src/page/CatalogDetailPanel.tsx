import {
  Badge,
  Button,
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
  DescriptionDetails,
  DescriptionItem,
  DescriptionList,
  DescriptionTerm,
  InlineAlert,
} from '@sdkwork/ui-pc-react';
import {
  describeProviderIntegration,
  findProviderModelPrice,
  recommendedModelPriceSourceKind,
  summarizeProviderPricingCoverage,
  useAdminI18n,
} from 'sdkwork-router-admin-core';
import type {
  ChannelModelRecord,
  CredentialRecord,
  ModelPriceRecord,
  ModelPriceTier,
  ProviderCatalogRecord,
  ProviderModelRecord,
} from 'sdkwork-router-admin-types';

import {
  credentialStorageLabel,
  modelPriceSourceLabel,
  priceUnitLabel,
  providerChannelIds,
  type CatalogLane,
  type ChannelRecord,
  type PendingDelete,
  type VariantRecord,
} from './shared';

type CatalogDetailPanelProps = {
  catalogLane: CatalogLane;
  channelNameById: Map<string, string>;
  defaultChannelId: string;
  onDeleteItem: (deleteTarget: NonNullable<PendingDelete>) => void;
  onEditChannelModel: (record: ChannelModelRecord) => void;
  onEditModelPrice: (record: ModelPriceRecord) => void;
  onPublishVariant: (channelId: string, variant: VariantRecord) => void;
  onStartPricing: (
    record: ChannelModelRecord,
    options?: {
      proxyProviderId?: string;
      priceSourceKind?: string;
    },
  ) => void;
  providerNameById: Map<string, string>;
  selectedChannel: ChannelRecord | null;
  selectedChannelProviderCount: number;
  selectedChannelModels: ChannelModelRecord[];
  selectedCredential: CredentialRecord | null;
  selectedModelPrices: ModelPriceRecord[];
  selectedProviderModels: ProviderModelRecord[];
  selectedProviderModelPrices: ModelPriceRecord[];
  selectedProvider: ProviderCatalogRecord | null;
  selectedPublication: ChannelModelRecord | null;
  selectedVariant: VariantRecord | null;
};

function summarizeModelPriceTier(tier: ModelPriceTier) {
  const conditions: string[] = [];

  if (tier.condition_kind) {
    conditions.push(tier.condition_kind);
  }
  if (tier.min_input_tokens != null || tier.max_input_tokens != null) {
    const min = tier.min_input_tokens?.toString() ?? '0';
    const max = tier.max_input_tokens?.toString() ?? 'max';
    conditions.push(`${min}-${max} input tokens`);
  }
  if (tier.modality) {
    conditions.push(`modality: ${tier.modality}`);
  }
  if (tier.cache_ttl) {
    conditions.push(`cache TTL: ${tier.cache_ttl}`);
  }

  return conditions.join(' | ');
}

export function CatalogDetailPanel({
  catalogLane,
  channelNameById,
  defaultChannelId,
  onDeleteItem,
  onEditChannelModel,
  onEditModelPrice,
  onPublishVariant,
  onStartPricing,
  providerNameById,
  selectedChannel,
  selectedChannelProviderCount,
  selectedChannelModels,
  selectedCredential,
  selectedModelPrices,
  selectedProviderModels,
  selectedProviderModelPrices,
  selectedProvider,
  selectedPublication,
  selectedVariant,
}: CatalogDetailPanelProps) {
  const { formatNumber, t } = useAdminI18n();

  if (catalogLane === 'channels' && selectedChannel) {
    return (
      <div className="space-y-4">
        <DescriptionList columns={2}>
          <DescriptionItem>
            <DescriptionTerm>{t('Channel id')}</DescriptionTerm>
            <DescriptionDetails mono>{selectedChannel.id}</DescriptionDetails>
          </DescriptionItem>
          <DescriptionItem>
            <DescriptionTerm>{t('Providers')}</DescriptionTerm>
            <DescriptionDetails>{formatNumber(selectedChannelProviderCount)}</DescriptionDetails>
          </DescriptionItem>
        </DescriptionList>
        <Card>
          <CardHeader>
            <CardTitle className="text-base">{t('Published models')}</CardTitle>
            <CardDescription>
              {t('Channel model publications and their pricing rows.')}
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-3">
            {selectedChannelModels.map((model) => (
              <div
                className="rounded-[var(--sdk-radius-control)] border border-[var(--sdk-color-border-default)] p-3"
                key={`${model.channel_id}:${model.model_id}`}
              >
                <div className="flex flex-wrap items-center justify-between gap-2">
                  <div>
                    <div className="font-medium">{model.model_display_name}</div>
                    <div className="text-sm text-[var(--sdk-color-text-secondary)]">
                      {model.model_id}
                    </div>
                  </div>
                  <div className="flex items-center gap-2">
                    <Button
                      onClick={() => onEditChannelModel(model)}
                      size="sm"
                      type="button"
                      variant="ghost"
                    >
                      {t('Edit')}
                    </Button>
                    <Button
                      onClick={() => onStartPricing(model)}
                      size="sm"
                      type="button"
                      variant="outline"
                    >
                      {t('Add pricing')}
                    </Button>
                    <Button
                      onClick={() =>
                        onDeleteItem({
                          kind: 'channel-model',
                          label: `${model.model_display_name} / ${model.model_id}`,
                          channelId: model.channel_id,
                          modelId: model.model_id,
                        })
                      }
                      size="sm"
                      type="button"
                      variant="danger"
                    >
                      {t('Delete')}
                    </Button>
                  </div>
                </div>
              </div>
            ))}
            {selectedChannelModels.length === 0 ? (
              <InlineAlert
                description={t('Publish a provider model into this channel to start exposing it to router consumers.')}
                title={t('No channel publications yet')}
                tone="info"
              />
            ) : null}
          </CardContent>
        </Card>
        {selectedPublication ? (
          <Card>
            <CardHeader>
              <CardTitle className="text-base">
                {t('Pricing for {name}', { name: selectedPublication.model_display_name })}
              </CardTitle>
              <CardDescription>
                {t('Provider-specific billing rows for the selected publication.')}
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-3">
              {selectedModelPrices.map((record) => (
                <div
                  className="space-y-3 rounded-[var(--sdk-radius-control)] border border-[var(--sdk-color-border-default)] p-3"
                  key={`${record.channel_id}:${record.model_id}:${record.proxy_provider_id}`}
                >
                  <div className="flex flex-wrap items-center justify-between gap-2">
                    <div>
                      <div className="font-medium">
                        {providerNameById.get(record.proxy_provider_id)
                          ?? record.proxy_provider_id}
                      </div>
                      <div className="text-sm text-[var(--sdk-color-text-secondary)]">
                        {record.currency_code} / {priceUnitLabel(record.price_unit)}
                      </div>
                    </div>
                    <div className="flex items-center gap-2">
                      <Button
                        onClick={() => onEditModelPrice(record)}
                        size="sm"
                        type="button"
                        variant="ghost"
                      >
                        {t('Edit')}
                      </Button>
                      <Button
                        onClick={() =>
                          onDeleteItem({
                            kind: 'model-price',
                            label: `${record.model_id} / ${record.proxy_provider_id}`,
                            channelId: record.channel_id,
                            modelId: record.model_id,
                            proxyProviderId: record.proxy_provider_id,
                          })
                        }
                        size="sm"
                        type="button"
                        variant="danger"
                      >
                        {t('Delete')}
                      </Button>
                    </div>
                  </div>
                  <div className="flex flex-wrap items-center gap-2">
                    <Badge variant="secondary">
                      {t('Price source')}
                      {': '}
                      {modelPriceSourceLabel(record.price_source_kind)}
                    </Badge>
                    <Badge variant="outline">
                      {record.currency_code}
                      {' / '}
                      {priceUnitLabel(record.price_unit)}
                    </Badge>
                    <Badge variant="outline">
                      {record.pricing_tiers.length > 0
                        ? t('{count} tiers', { count: record.pricing_tiers.length })
                        : t('Flat pricing')}
                    </Badge>
                  </div>
                  <DescriptionList columns={2}>
                    <DescriptionItem>
                      <DescriptionTerm>{t('Input price')}</DescriptionTerm>
                      <DescriptionDetails>{String(record.input_price)}</DescriptionDetails>
                    </DescriptionItem>
                    <DescriptionItem>
                      <DescriptionTerm>{t('Output price')}</DescriptionTerm>
                      <DescriptionDetails>{String(record.output_price)}</DescriptionDetails>
                    </DescriptionItem>
                    <DescriptionItem>
                      <DescriptionTerm>{t('Cache read')}</DescriptionTerm>
                      <DescriptionDetails>{String(record.cache_read_price)}</DescriptionDetails>
                    </DescriptionItem>
                    <DescriptionItem>
                      <DescriptionTerm>{t('Cache write')}</DescriptionTerm>
                      <DescriptionDetails>{String(record.cache_write_price)}</DescriptionDetails>
                    </DescriptionItem>
                    <DescriptionItem>
                      <DescriptionTerm>{t('Request price')}</DescriptionTerm>
                      <DescriptionDetails>{String(record.request_price)}</DescriptionDetails>
                    </DescriptionItem>
                    <DescriptionItem>
                      <DescriptionTerm>{t('Tiered pricing')}</DescriptionTerm>
                      <DescriptionDetails>
                        {record.pricing_tiers.length > 0
                          ? t('{count} configured', { count: record.pricing_tiers.length })
                          : t('Not configured')}
                      </DescriptionDetails>
                    </DescriptionItem>
                  </DescriptionList>
                  {record.billing_notes ? (
                    <div className="space-y-1">
                      <div className="text-xs font-medium text-[var(--sdk-color-text-secondary)]">
                        {t('Billing notes')}
                      </div>
                      <div className="text-sm text-[var(--sdk-color-text-secondary)]">
                        {record.billing_notes}
                      </div>
                    </div>
                  ) : null}
                  {record.pricing_tiers.length > 0 ? (
                    <div className="space-y-2">
                      <div className="text-xs font-medium text-[var(--sdk-color-text-secondary)]">
                        {t('Tiered pricing')}
                      </div>
                      {record.pricing_tiers.map((tier) => (
                        <div
                          className="space-y-2 rounded-[var(--sdk-radius-control)] bg-[var(--sdk-color-background-subtle)] p-3"
                          key={tier.tier_id}
                        >
                          <div className="flex flex-wrap items-start justify-between gap-2">
                            <div>
                              <div className="font-medium">
                                {tier.display_name || tier.tier_id}
                              </div>
                              <div className="text-sm text-[var(--sdk-color-text-secondary)]">
                                {summarizeModelPriceTier(tier) || t('Default')}
                              </div>
                            </div>
                            <div className="text-xs text-[var(--sdk-color-text-secondary)]">
                              {tier.currency_code}
                              {' / '}
                              {priceUnitLabel(tier.price_unit)}
                            </div>
                          </div>
                          <DescriptionList columns={2}>
                            <DescriptionItem>
                              <DescriptionTerm>{t('Input price')}</DescriptionTerm>
                              <DescriptionDetails>{String(tier.input_price)}</DescriptionDetails>
                            </DescriptionItem>
                            <DescriptionItem>
                              <DescriptionTerm>{t('Output price')}</DescriptionTerm>
                              <DescriptionDetails>{String(tier.output_price)}</DescriptionDetails>
                            </DescriptionItem>
                            <DescriptionItem>
                              <DescriptionTerm>{t('Cache read')}</DescriptionTerm>
                              <DescriptionDetails>{String(tier.cache_read_price)}</DescriptionDetails>
                            </DescriptionItem>
                            <DescriptionItem>
                              <DescriptionTerm>{t('Cache write')}</DescriptionTerm>
                              <DescriptionDetails>{String(tier.cache_write_price)}</DescriptionDetails>
                            </DescriptionItem>
                            <DescriptionItem>
                              <DescriptionTerm>{t('Request price')}</DescriptionTerm>
                              <DescriptionDetails>{String(tier.request_price)}</DescriptionDetails>
                            </DescriptionItem>
                          </DescriptionList>
                          {tier.notes ? (
                            <div className="text-sm text-[var(--sdk-color-text-secondary)]">
                              {tier.notes}
                            </div>
                          ) : null}
                        </div>
                      ))}
                    </div>
                  ) : null}
                </div>
              ))}
              {selectedModelPrices.length === 0 ? (
                <InlineAlert
                  description={t('No provider pricing rows exist for the selected publication.')}
                  title={t('Pricing is empty')}
                  tone="warning"
                />
              ) : null}
            </CardContent>
          </Card>
        ) : null}
      </div>
    );
  }

  if (catalogLane === 'providers' && selectedProvider) {
    const pricingCoverage = summarizeProviderPricingCoverage(
      selectedProvider.id,
      selectedProviderModels,
      selectedProviderModelPrices,
    );
    const recommendedPriceSourceKind =
      recommendedModelPriceSourceKind(selectedProvider);
    return (
      <div className="space-y-4">
        <DescriptionList columns={2}>
          <DescriptionItem>
            <DescriptionTerm>{t('Provider id')}</DescriptionTerm>
            <DescriptionDetails mono>{selectedProvider.id}</DescriptionDetails>
          </DescriptionItem>
          <DescriptionItem>
            <DescriptionTerm>{t('Adapter')}</DescriptionTerm>
            <DescriptionDetails>
              {describeProviderIntegration(selectedProvider)}
            </DescriptionDetails>
          </DescriptionItem>
          <DescriptionItem>
            <DescriptionTerm>{t('Primary channel')}</DescriptionTerm>
            <DescriptionDetails>
              {channelNameById.get(selectedProvider.channel_id) ?? selectedProvider.channel_id}
            </DescriptionDetails>
          </DescriptionItem>
          <DescriptionItem>
            <DescriptionTerm>{t('Base URL')}</DescriptionTerm>
            <DescriptionDetails mono>{selectedProvider.base_url}</DescriptionDetails>
          </DescriptionItem>
        </DescriptionList>
        <InlineAlert
          description={providerChannelIds(selectedProvider)
            .map((channelId) => channelNameById.get(channelId) ?? channelId)
            .join(', ')}
          title={t('Bound channels')}
          tone="info"
        />
        <InlineAlert
          description={pricingCoverage.active_model_count > 0
            ? t(
                '{priced} priced / {active} active · {missing} missing pricing · {defaults} default routes',
                {
                  priced: pricingCoverage.priced_model_count,
                  active: pricingCoverage.active_model_count,
                  missing: pricingCoverage.missing_price_count,
                  defaults: pricingCoverage.default_route_count,
                },
              )
            : t('No active provider-model records exist yet, so pricing cannot be attached.')}
          title={t('Pricing coverage')}
          tone={pricingCoverage.missing_price_count > 0 ? 'warning' : 'info'}
        />
        <Card>
          <CardHeader>
            <CardTitle className="text-base">{t('Supported canonical models')}</CardTitle>
            <CardDescription>
              {t('Canonical channel-model pairs currently exposed by this provider, including provider-model metadata and pricing coverage.' )}
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-3">
            {selectedProviderModels.map((record) => {
              const priceRecord = findProviderModelPrice(
                record,
                selectedProviderModelPrices,
              );
              const publication: ChannelModelRecord = {
                channel_id: record.channel_id,
                model_id: record.model_id,
                model_display_name: record.model_id,
                capabilities: record.capabilities,
                streaming: record.streaming,
                context_window: record.context_window,
                description: null,
              };

              return (
                <div
                  className="space-y-3 rounded-[var(--sdk-radius-control)] border border-[var(--sdk-color-border-default)] p-3"
                  key={`${record.proxy_provider_id}:${record.channel_id}:${record.model_id}`}
                >
                  <div className="flex flex-wrap items-center justify-between gap-2">
                    <div>
                      <div className="font-medium">
                        {channelNameById.get(record.channel_id) ?? record.channel_id}
                        {' / '}
                        {record.model_id}
                      </div>
                      <div className="text-sm text-[var(--sdk-color-text-secondary)]">
                        {record.provider_model_id}
                      </div>
                    </div>
                    <div className="flex flex-wrap items-center gap-2">
                      <Badge variant={record.is_default_route ? 'secondary' : 'outline'}>
                        {record.is_default_route ? t('Default route') : t('Optional route')}
                      </Badge>
                      <Badge variant={record.is_active ? 'outline' : 'secondary'}>
                        {record.is_active ? t('Active') : t('Inactive')}
                      </Badge>
                      <Badge variant={priceRecord ? 'outline' : 'secondary'}>
                        {priceRecord ? t('Priced') : t('Missing pricing')}
                      </Badge>
                    </div>
                  </div>
                  <DescriptionList columns={2}>
                    <DescriptionItem>
                      <DescriptionTerm>{t('Provider model id')}</DescriptionTerm>
                      <DescriptionDetails mono>{record.provider_model_id}</DescriptionDetails>
                    </DescriptionItem>
                    <DescriptionItem>
                      <DescriptionTerm>{t('Provider model family')}</DescriptionTerm>
                      <DescriptionDetails>
                        {record.provider_model_family || t('Not set')}
                      </DescriptionDetails>
                    </DescriptionItem>
                    <DescriptionItem>
                      <DescriptionTerm>{t('Context window')}</DescriptionTerm>
                      <DescriptionDetails>{record.context_window ?? '-'}</DescriptionDetails>
                    </DescriptionItem>
                    <DescriptionItem>
                      <DescriptionTerm>{t('Max output tokens')}</DescriptionTerm>
                      <DescriptionDetails>{record.max_output_tokens ?? '-'}</DescriptionDetails>
                    </DescriptionItem>
                  </DescriptionList>
                  <div className="flex flex-wrap items-center gap-2">
                    <Badge variant="outline">
                      {record.capabilities.join(', ') || t('general')}
                    </Badge>
                    {record.supports_prompt_caching ? (
                      <Badge variant="outline">{t('Prompt caching')}</Badge>
                    ) : null}
                    {record.supports_reasoning_usage ? (
                      <Badge variant="outline">{t('Reasoning usage')}</Badge>
                    ) : null}
                    {record.supports_tool_usage_metrics ? (
                      <Badge variant="outline">{t('Tool usage metrics')}</Badge>
                    ) : null}
                  </div>
                  {priceRecord ? (
                    <div className="space-y-2 rounded-[var(--sdk-radius-control)] bg-[var(--sdk-color-background-subtle)] p-3">
                      <div className="flex flex-wrap items-center justify-between gap-2">
                        <div className="font-medium">{t('Pricing coverage')}</div>
                        <div className="flex flex-wrap items-center gap-2">
                          <Badge variant="secondary">
                            {modelPriceSourceLabel(priceRecord.price_source_kind)}
                          </Badge>
                          <Badge variant="outline">
                            {priceRecord.currency_code}
                            {' / '}
                            {priceUnitLabel(priceRecord.price_unit)}
                          </Badge>
                        </div>
                      </div>
                      <DescriptionList columns={2}>
                        <DescriptionItem>
                          <DescriptionTerm>{t('Input price')}</DescriptionTerm>
                          <DescriptionDetails>{String(priceRecord.input_price)}</DescriptionDetails>
                        </DescriptionItem>
                        <DescriptionItem>
                          <DescriptionTerm>{t('Output price')}</DescriptionTerm>
                          <DescriptionDetails>{String(priceRecord.output_price)}</DescriptionDetails>
                        </DescriptionItem>
                      </DescriptionList>
                      <div className="flex flex-wrap items-center justify-end gap-2">
                        <Button
                          onClick={() => onEditModelPrice(priceRecord)}
                          size="sm"
                          type="button"
                          variant="outline"
                        >
                          {t('Edit pricing')}
                        </Button>
                      </div>
                    </div>
                  ) : (
                    <InlineAlert
                      description={t(
                        'This provider-model is active but has no pricing row yet. Add pricing before using it in cost-aware routing, reporting, or tenant billing comparisons.',
                      )}
                      title={t('Missing pricing')}
                      tone="warning"
                    />
                  )}
                  {!priceRecord ? (
                    <div className="flex flex-wrap items-center justify-end gap-2">
                      <Button
                        onClick={() =>
                          onStartPricing(publication, {
                            proxyProviderId: record.proxy_provider_id,
                            priceSourceKind: recommendedPriceSourceKind,
                          })
                        }
                        size="sm"
                        type="button"
                        variant="primary"
                      >
                        {t('Add pricing')}
                      </Button>
                    </div>
                  ) : null}
                </div>
              );
            })}
            {selectedProviderModels.length === 0 ? (
              <InlineAlert
                description={t('No canonical channel models are bound to this provider yet.')}
                title={t('Model support is empty')}
                tone="warning"
              />
            ) : null}
          </CardContent>
        </Card>
      </div>
    );
  }

  if (catalogLane === 'credentials' && selectedCredential) {
    return (
      <DescriptionList columns={2}>
        <DescriptionItem>
          <DescriptionTerm>{t('Tenant')}</DescriptionTerm>
          <DescriptionDetails>{selectedCredential.tenant_id}</DescriptionDetails>
        </DescriptionItem>
        <DescriptionItem>
          <DescriptionTerm>{t('Provider')}</DescriptionTerm>
          <DescriptionDetails>
            {providerNameById.get(selectedCredential.provider_id)
              ?? selectedCredential.provider_id}
          </DescriptionDetails>
        </DescriptionItem>
        <DescriptionItem>
          <DescriptionTerm>{t('Backend')}</DescriptionTerm>
          <DescriptionDetails>{selectedCredential.secret_backend}</DescriptionDetails>
        </DescriptionItem>
        <DescriptionItem>
          <DescriptionTerm>{t('Storage')}</DescriptionTerm>
          <DescriptionDetails>
            {credentialStorageLabel(selectedCredential)}
          </DescriptionDetails>
        </DescriptionItem>
      </DescriptionList>
    );
  }

  if (catalogLane === 'variants' && selectedVariant) {
    return (
      <div className="space-y-4">
        <DescriptionList columns={2}>
          <DescriptionItem>
            <DescriptionTerm>{t('Provider')}</DescriptionTerm>
            <DescriptionDetails>
              {providerNameById.get(selectedVariant.provider_id) ?? selectedVariant.provider_id}
            </DescriptionDetails>
          </DescriptionItem>
          <DescriptionItem>
            <DescriptionTerm>{t('Capabilities')}</DescriptionTerm>
            <DescriptionDetails>{selectedVariant.capabilities.join(', ') || '-'}</DescriptionDetails>
          </DescriptionItem>
          <DescriptionItem>
            <DescriptionTerm>{t('Streaming')}</DescriptionTerm>
            <DescriptionDetails>{selectedVariant.streaming ? t('Enabled') : t('Disabled')}</DescriptionDetails>
          </DescriptionItem>
          <DescriptionItem>
            <DescriptionTerm>{t('Context window')}</DescriptionTerm>
            <DescriptionDetails>{selectedVariant.context_window ?? '-'}</DescriptionDetails>
          </DescriptionItem>
        </DescriptionList>
        <Button
          onClick={() => onPublishVariant(selectedChannel?.id ?? defaultChannelId, selectedVariant)}
          type="button"
          variant="primary"
        >
          {t('Publish to channel')}
        </Button>
      </div>
    );
  }

  return null;
}
