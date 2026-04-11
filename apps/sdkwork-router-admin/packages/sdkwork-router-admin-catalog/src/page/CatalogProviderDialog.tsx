import type {
  ChangeEvent,
  Dispatch,
  FormEvent,
  SetStateAction,
} from 'react';
import {
  Badge,
  Button,
  Checkbox,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  FormActions,
  FormGrid,
  FormSection,
  Input,
} from '@sdkwork/ui-pc-react';
import {
  applyProviderDefaultPluginFamily,
  applyProviderIntegrationMode,
  applyProviderStandardProtocol,
  CUSTOM_PLUGIN_PROTOCOL_OPTIONS,
  DEFAULT_PLUGIN_FAMILY_OPTIONS,
  providerSupportedModelDraftFromChannelModel,
  providerSupportedModelKey,
  STANDARD_PROVIDER_PROTOCOL_OPTIONS,
  type DefaultPluginFamily,
  type ProviderSupportedModelDraft,
  useAdminI18n,
} from 'sdkwork-router-admin-core';
import type { AdminPageProps } from 'sdkwork-router-admin-types';

import { DialogField, SelectField, type ProviderDraft } from './shared';

type CatalogProviderDialogProps = {
  editingProviderId: string | null;
  onOpenChange: (open: boolean) => void;
  onSubmit: (event: FormEvent<HTMLFormElement>) => void;
  open: boolean;
  providerDraft: ProviderDraft;
  setProviderDraft: Dispatch<SetStateAction<ProviderDraft>>;
  snapshot: AdminPageProps['snapshot'];
};

export function CatalogProviderDialog({
  editingProviderId,
  onOpenChange,
  onSubmit,
  open,
  providerDraft,
  setProviderDraft,
  snapshot,
}: CatalogProviderDialogProps) {
  const { t } = useAdminI18n();
  const boundChannelIds = Array.from(
    new Set(
      [providerDraft.primary_channel_id, ...providerDraft.bound_channel_ids].filter(Boolean),
    ),
  );
  const selectedModelKeys = new Set(
    providerDraft.supported_models.map((model) => providerSupportedModelKey(model)),
  );
  const availableModels = snapshot.channelModels.filter((model) =>
    boundChannelIds.includes(model.channel_id),
  );
  const selectedSupportedModels = providerDraft.supported_models
    .slice()
    .sort((left, right) =>
      providerSupportedModelKey(left).localeCompare(providerSupportedModelKey(right)),
    );
  const pricingByModelKey = new Map(
    snapshot.modelPrices
      .filter((record) => record.proxy_provider_id === providerDraft.id)
      .map((record) => [providerSupportedModelKey(record), record]),
  );

  function parseOptionalWholeNumber(value: string) {
    const trimmed = value.trim();
    if (!trimmed) {
      return null;
    }
    const parsed = Number(trimmed);
    return Number.isFinite(parsed) ? parsed : null;
  }

  function updateSupportedModel(
    key: string,
    updater: (model: ProviderSupportedModelDraft) => ProviderSupportedModelDraft,
  ) {
    setProviderDraft((current) => ({
      ...current,
      supported_models: current.supported_models.map((model) =>
        providerSupportedModelKey(model) === key ? updater(model) : model,
      ),
    }));
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="w-[min(92vw,56rem)]">
        <DialogHeader>
          <DialogTitle>{editingProviderId ? t('Edit provider') : t('Create provider')}</DialogTitle>
          <DialogDescription>
            {t('Capture upstream connectivity and channel bindings with the shared form primitives.')}
          </DialogDescription>
        </DialogHeader>
        <form className="space-y-6" onSubmit={onSubmit}>
          <FormSection title={t('Provider profile')}>
            <FormGrid columns={2}>
              <DialogField htmlFor="provider-id" label={t('Provider id')}>
                <Input
                  disabled={Boolean(editingProviderId)}
                  id="provider-id"
                  onChange={(event: ChangeEvent<HTMLInputElement>) =>
                    setProviderDraft((current) => ({
                      ...current,
                      id: event.target.value,
                    }))
                  }
                  required
                  value={providerDraft.id}
                />
              </DialogField>
              <DialogField htmlFor="provider-name" label={t('Display name')}>
                <Input
                  id="provider-name"
                  onChange={(event: ChangeEvent<HTMLInputElement>) =>
                    setProviderDraft((current) => ({
                      ...current,
                      display_name: event.target.value,
                    }))
                  }
                  required
                  value={providerDraft.display_name}
                />
              </DialogField>
              <SelectField
                label={t('Integration mode')}
                onValueChange={(value) =>
                  setProviderDraft((current) =>
                    applyProviderIntegrationMode(current, value),
                  )
                }
                options={[
                  { label: t('Standard passthrough'), value: 'standard_passthrough' },
                  { label: t('Default plugin'), value: 'default_plugin' },
                  { label: t('Custom plugin'), value: 'custom_plugin' },
                ]}
                value={providerDraft.integration_mode}
              />
              {providerDraft.integration_mode === 'standard_passthrough' ? (
                <SelectField
                  label={t('Provider standard')}
                  onValueChange={(value) =>
                    setProviderDraft((current) =>
                      applyProviderStandardProtocol(current, value),
                    )
                  }
                  options={STANDARD_PROVIDER_PROTOCOL_OPTIONS.map((option) => ({
                    label: t(option.label),
                    value: option.value,
                  }))}
                  value={providerDraft.standard_protocol}
                />
              ) : null}
              {providerDraft.integration_mode === 'default_plugin' ? (
                <SelectField<DefaultPluginFamily>
                  label={t('Default plugin family')}
                  onValueChange={(value) =>
                    setProviderDraft((current) =>
                      applyProviderDefaultPluginFamily(current, value),
                    )
                  }
                  options={DEFAULT_PLUGIN_FAMILY_OPTIONS.map((option) => ({
                    label: t(option.label),
                    value: option.value,
                  }))}
                  value={
                    (providerDraft.default_plugin_family || 'openrouter') as DefaultPluginFamily
                  }
                />
              ) : null}
              {providerDraft.integration_mode === 'custom_plugin' ? (
                <DialogField htmlFor="provider-adapter" label={t('Adapter kind')}>
                  <Input
                    id="provider-adapter"
                    onChange={(event: ChangeEvent<HTMLInputElement>) =>
                      setProviderDraft((current) => ({
                        ...current,
                        adapter_kind: event.target.value,
                      }))
                    }
                    required
                    value={providerDraft.adapter_kind}
                  />
                </DialogField>
              ) : null}
              <DialogField htmlFor="provider-url" label={t('Base URL')}>
                <Input
                  id="provider-url"
                  onChange={(event: ChangeEvent<HTMLInputElement>) =>
                    setProviderDraft((current) => ({
                      ...current,
                      base_url: event.target.value,
                    }))
                  }
                  required
                  value={providerDraft.base_url}
                />
              </DialogField>
              {providerDraft.integration_mode === 'custom_plugin' ? (
                <SelectField
                  label={t('External protocol')}
                  onValueChange={(value) =>
                    setProviderDraft((current) => ({
                      ...current,
                      protocol_kind: value,
                    }))
                  }
                  options={CUSTOM_PLUGIN_PROTOCOL_OPTIONS.map((option) => ({
                    label: t(option.label),
                    value: option.value,
                  }))}
                  value={providerDraft.protocol_kind || 'custom'}
                />
              ) : null}
              {providerDraft.integration_mode === 'custom_plugin' ? (
                <DialogField htmlFor="provider-extension" label={t('Extension id')}>
                  <Input
                    id="provider-extension"
                    onChange={(event: ChangeEvent<HTMLInputElement>) =>
                      setProviderDraft((current) => ({
                        ...current,
                        extension_id: event.target.value,
                      }))
                    }
                    value={providerDraft.extension_id}
                  />
                </DialogField>
              ) : null}
              <SelectField
                label={t('Primary channel')}
                onValueChange={(value) =>
                  setProviderDraft((current) => ({
                    ...current,
                    primary_channel_id: value,
                    bound_channel_ids: current.bound_channel_ids.includes(value)
                      ? current.bound_channel_ids
                      : [...current.bound_channel_ids, value],
                  }))
                }
                options={snapshot.channels.map((channel) => ({
                  label: `${channel.name} (${channel.id})`,
                  value: channel.id,
                }))}
                value={providerDraft.primary_channel_id}
              />
            </FormGrid>
          </FormSection>

          <FormSection title={t('Bound channels')}>
            <div className="grid gap-3 md:grid-cols-2">
              {snapshot.channels.map((channel) => {
                const checked = providerDraft.bound_channel_ids.includes(channel.id);

                return (
                  <label
                    className="flex items-start gap-3 rounded-[var(--sdk-radius-control)] border border-[var(--sdk-color-border-default)] px-4 py-3"
                    key={channel.id}
                  >
                    <Checkbox
                      checked={checked}
                      onCheckedChange={(nextChecked: boolean | 'indeterminate') =>
                        setProviderDraft((current) => ({
                          ...current,
                          bound_channel_ids:
                            nextChecked === true
                              ? Array.from(
                                  new Set([...current.bound_channel_ids, channel.id]),
                                )
                              : current.bound_channel_ids.filter(
                                  (id) => id !== channel.id,
                                ),
                          supported_models:
                            nextChecked === true
                              ? current.supported_models
                              : current.supported_models.filter(
                                  (model) => model.channel_id !== channel.id,
                                ),
                        }))
                      }
                    />
                    <div>
                      <div className="font-medium">{channel.name}</div>
                      <div className="text-xs text-[var(--sdk-color-text-secondary)]">
                        {channel.id}
                      </div>
                    </div>
                  </label>
                );
              })}
            </div>
          </FormSection>

          <FormSection title={t('Supported models')}>
            <div className="space-y-3">
              <div className="text-sm text-[var(--sdk-color-text-secondary)]">
                {t('Pick the canonical channel models this provider can actually serve. Official pricing and proxy pricing will attach to these canonical records.')}
              </div>
              <div className="grid gap-3 md:grid-cols-2">
                {availableModels.map((model) => {
                  const key = providerSupportedModelKey(model);
                  const checked = selectedModelKeys.has(key);

                  return (
                    <label
                      className="flex items-start gap-3 rounded-[var(--sdk-radius-control)] border border-[var(--sdk-color-border-default)] px-4 py-3"
                      key={key}
                    >
                      <Checkbox
                        checked={checked}
                        onCheckedChange={(nextChecked: boolean | 'indeterminate') =>
                          setProviderDraft((current) => {
                            const existing = current.supported_models.find(
                              (record) => providerSupportedModelKey(record) === key,
                            );
                            return {
                              ...current,
                              supported_models:
                                nextChecked === true
                                  ? existing
                                    ? current.supported_models
                                    : [
                                        ...current.supported_models,
                                        providerSupportedModelDraftFromChannelModel(
                                          model,
                                        ),
                                      ]
                                  : current.supported_models.filter(
                                      (record) =>
                                        providerSupportedModelKey(record) !== key,
                                    ),
                            };
                          })
                        }
                      />
                      <div className="space-y-1">
                        <div className="font-medium">{model.model_display_name}</div>
                        <div className="text-xs text-[var(--sdk-color-text-secondary)]">
                          {model.channel_id} / {model.model_id}
                        </div>
                        <div className="text-xs text-[var(--sdk-color-text-muted)]">
                          {model.capabilities.join(', ') || t('general')}
                        </div>
                      </div>
                    </label>
                  );
                })}
              </div>
              {availableModels.length === 0 ? (
                <div className="text-sm text-[var(--sdk-color-text-secondary)]">
                  {t('Bind at least one channel before selecting supported models.')}
                </div>
              ) : null}
              {selectedSupportedModels.length > 0 ? (
                <div className="space-y-3">
                  <div className="text-sm font-medium">
                    {t('Selected provider-model configuration')}
                  </div>
                  {selectedSupportedModels.map((model) => {
                    const key = providerSupportedModelKey(model);
                    const pricingRecord = pricingByModelKey.get(key);

                    return (
                      <div
                        className="space-y-4 rounded-[var(--sdk-radius-control)] border border-[var(--sdk-color-border-default)] p-4"
                        key={key}
                      >
                        <div className="flex flex-wrap items-center justify-between gap-3">
                          <div>
                            <div className="font-medium">
                              {model.channel_id}
                              {' / '}
                              {model.model_id}
                            </div>
                            <div className="text-xs text-[var(--sdk-color-text-secondary)]">
                              {t('Canonical model subset exposed by this provider.')}
                            </div>
                          </div>
                          <div className="flex flex-wrap items-center gap-2">
                            <Badge variant={model.is_default_route ? 'secondary' : 'outline'}>
                              {model.is_default_route ? t('Default route') : t('Optional route')}
                            </Badge>
                            <Badge variant={model.is_active ? 'outline' : 'secondary'}>
                              {model.is_active ? t('Active') : t('Inactive')}
                            </Badge>
                            <Badge variant={pricingRecord ? 'outline' : 'secondary'}>
                              {pricingRecord ? t('Pricing coverage') : t('Missing pricing')}
                            </Badge>
                          </div>
                        </div>

                        <FormGrid columns={2}>
                          <DialogField
                            htmlFor={`provider-model-id-${key}`}
                            label={t('Provider model id')}
                            description={t('Outbound requests use this provider-native model id.')}
                          >
                            <Input
                              id={`provider-model-id-${key}`}
                              onChange={(event: ChangeEvent<HTMLInputElement>) =>
                                updateSupportedModel(key, (current) => ({
                                  ...current,
                                  provider_model_id: event.target.value,
                                }))
                              }
                              required
                              value={model.provider_model_id}
                            />
                          </DialogField>
                          <DialogField
                            htmlFor={`provider-model-family-${key}`}
                            label={t('Provider model family')}
                            description={t('Optional upstream family or vendor grouping label.')}
                          >
                            <Input
                              id={`provider-model-family-${key}`}
                              onChange={(event: ChangeEvent<HTMLInputElement>) =>
                                updateSupportedModel(key, (current) => ({
                                  ...current,
                                  provider_model_family: event.target.value,
                                }))
                              }
                              value={model.provider_model_family}
                            />
                          </DialogField>
                          <DialogField
                            htmlFor={`provider-model-capabilities-${key}`}
                            label={t('Supported capabilities')}
                            description={t('Use a comma-separated subset when the provider does not expose the full canonical surface.')}
                          >
                            <Input
                              id={`provider-model-capabilities-${key}`}
                              onChange={(event: ChangeEvent<HTMLInputElement>) =>
                                updateSupportedModel(key, (current) => ({
                                  ...current,
                                  capabilities: event.target.value
                                    .split(',')
                                    .map((value) => value.trim())
                                    .filter(Boolean),
                                }))
                              }
                              value={model.capabilities.join(', ')}
                            />
                          </DialogField>
                          <DialogField
                            htmlFor={`provider-model-context-window-${key}`}
                            label={t('Context window')}
                          >
                            <Input
                              id={`provider-model-context-window-${key}`}
                              onChange={(event: ChangeEvent<HTMLInputElement>) =>
                                updateSupportedModel(key, (current) => ({
                                  ...current,
                                  context_window: parseOptionalWholeNumber(
                                    event.target.value,
                                  ),
                                }))
                              }
                              type="number"
                              value={model.context_window ?? ''}
                            />
                          </DialogField>
                          <DialogField
                            htmlFor={`provider-model-max-output-${key}`}
                            label={t('Max output tokens')}
                          >
                            <Input
                              id={`provider-model-max-output-${key}`}
                              onChange={(event: ChangeEvent<HTMLInputElement>) =>
                                updateSupportedModel(key, (current) => ({
                                  ...current,
                                  max_output_tokens: parseOptionalWholeNumber(
                                    event.target.value,
                                  ),
                                }))
                              }
                              type="number"
                              value={model.max_output_tokens ?? ''}
                            />
                          </DialogField>
                        </FormGrid>

                        <div className="grid gap-3 md:grid-cols-2 xl:grid-cols-3">
                          {[
                            {
                              checked: model.streaming ?? false,
                              label: t('Streaming'),
                              apply: (
                                current: ProviderSupportedModelDraft,
                                checked: boolean,
                              ): ProviderSupportedModelDraft => ({
                                ...current,
                                streaming: checked,
                              }),
                            },
                            {
                              checked: model.supports_prompt_caching,
                              label: t('Prompt caching'),
                              apply: (
                                current: ProviderSupportedModelDraft,
                                checked: boolean,
                              ): ProviderSupportedModelDraft => ({
                                ...current,
                                supports_prompt_caching: checked,
                              }),
                            },
                            {
                              checked: model.supports_reasoning_usage,
                              label: t('Reasoning usage'),
                              apply: (
                                current: ProviderSupportedModelDraft,
                                checked: boolean,
                              ): ProviderSupportedModelDraft => ({
                                ...current,
                                supports_reasoning_usage: checked,
                              }),
                            },
                            {
                              checked: model.supports_tool_usage_metrics,
                              label: t('Tool usage metrics'),
                              apply: (
                                current: ProviderSupportedModelDraft,
                                checked: boolean,
                              ): ProviderSupportedModelDraft => ({
                                ...current,
                                supports_tool_usage_metrics: checked,
                              }),
                            },
                            {
                              checked: model.is_default_route,
                              label: t('Default route'),
                              apply: (
                                current: ProviderSupportedModelDraft,
                                checked: boolean,
                              ): ProviderSupportedModelDraft => ({
                                ...current,
                                is_default_route: checked,
                              }),
                            },
                            {
                              checked: model.is_active,
                              label: t('Active'),
                              apply: (
                                current: ProviderSupportedModelDraft,
                                checked: boolean,
                              ): ProviderSupportedModelDraft => ({
                                ...current,
                                is_active: checked,
                              }),
                            },
                          ].map((option) => (
                            <label
                              className="flex items-start gap-3 rounded-[var(--sdk-radius-control)] border border-[var(--sdk-color-border-default)] px-4 py-3"
                              key={`${key}:${option.label}`}
                            >
                              <Checkbox
                                checked={option.checked}
                                onCheckedChange={(nextChecked: boolean | 'indeterminate') =>
                                  updateSupportedModel(
                                    key,
                                    (current) => option.apply(current, nextChecked === true),
                                  )
                                }
                              />
                              <div>
                                <div className="font-medium">{option.label}</div>
                              </div>
                            </label>
                          ))}
                        </div>
                      </div>
                    );
                  })}
                </div>
              ) : null}
            </div>
          </FormSection>

          <FormActions>
            <Button onClick={() => onOpenChange(false)} type="button" variant="outline">
              {t('Cancel')}
            </Button>
            <Button type="submit" variant="primary">
              {editingProviderId ? t('Save provider') : t('Create provider')}
            </Button>
          </FormActions>
        </form>
      </DialogContent>
    </Dialog>
  );
}
