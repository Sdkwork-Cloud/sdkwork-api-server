import type {
  ChangeEvent,
  Dispatch,
  FormEvent,
  SetStateAction,
} from 'react';
import {
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
  STANDARD_PROVIDER_PROTOCOL_OPTIONS,
  type DefaultPluginFamily,
  useAdminI18n,
} from 'sdkwork-router-admin-core';
import type {
  AdminWorkspaceSnapshot,
  ProxyProviderRecord,
} from 'sdkwork-router-admin-types';

import { DialogField, SelectField } from '../shared';
import type { ProviderDraft } from './shared';

type GatewayProviderDialogProps = {
  editingProvider: ProxyProviderRecord | null;
  onOpenChange: (open: boolean) => void;
  onSubmit: (event: FormEvent<HTMLFormElement>) => void;
  open: boolean;
  providerDraft: ProviderDraft;
  setProviderDraft: Dispatch<SetStateAction<ProviderDraft>>;
  snapshot: AdminWorkspaceSnapshot;
};

export function GatewayProviderDialog({
  editingProvider,
  onOpenChange,
  onSubmit,
  open,
  providerDraft,
  setProviderDraft,
  snapshot,
}: GatewayProviderDialogProps) {
  const { t } = useAdminI18n();

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="w-[min(92vw,56rem)]">
        <DialogHeader>
          <DialogTitle>
            {editingProvider ? t('Edit route provider') : t('Create route provider')}
          </DialogTitle>
          <DialogDescription>
            {t(
              'Keep route posture focused on upstream connectivity and channel exposure. Credentials and model publication remain visible from the main workbench.',
            )}
          </DialogDescription>
        </DialogHeader>

        <form className="space-y-6" onSubmit={onSubmit}>
          <FormSection
            description={t('Capture the upstream endpoint and the public channel bindings used by the router.')}
            title={t('Provider profile')}
          >
            <FormGrid columns={2}>
              <DialogField htmlFor="gateway-provider-id" label={t('Provider id')}>
                <Input
                  disabled={Boolean(editingProvider)}
                  id="gateway-provider-id"
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
              <DialogField htmlFor="gateway-provider-name" label={t('Display name')}>
                <Input
                  id="gateway-provider-name"
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
                <DialogField htmlFor="gateway-provider-adapter" label={t('Adapter kind')}>
                  <Input
                    id="gateway-provider-adapter"
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
              <DialogField htmlFor="gateway-provider-base-url" label={t('Base URL')}>
                <Input
                  id="gateway-provider-base-url"
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
                <DialogField
                  htmlFor="gateway-provider-extension"
                  label={t('Extension id')}
                >
                  <Input
                    id="gateway-provider-extension"
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

          <FormSection
            description={t('Expose the provider on one or more public API channels without leaving the dialog.')}
            title={t('Channel bindings')}
          >
            <div className="grid gap-3 md:grid-cols-2">
              {snapshot.channels.map((channel) => {
                const checked = providerDraft.bound_channel_ids.includes(channel.id);

                return (
                  <label
                    className="flex items-start gap-3 rounded-[var(--sdk-radius-control)] border border-[var(--sdk-color-border-default)] bg-[var(--sdk-color-surface-panel-muted)] px-4 py-3"
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
                                  new Set([
                                    ...current.bound_channel_ids,
                                    channel.id,
                                  ]),
                                )
                              : current.bound_channel_ids.filter(
                                  (id) => id !== channel.id,
                                ),
                        }))
                      }
                    />
                    <div className="space-y-1">
                      <div className="font-medium text-[var(--sdk-color-text-primary)]">
                        {channel.name}
                      </div>
                      <div className="font-mono text-xs text-[var(--sdk-color-text-secondary)]">
                        {channel.id}
                      </div>
                    </div>
                  </label>
                );
              })}
            </div>
          </FormSection>

          <FormActions>
            <Button
              onClick={() => onOpenChange(false)}
              type="button"
              variant="outline"
            >
              {t('Cancel')}
            </Button>
            <Button type="submit" variant="primary">
              {editingProvider ? t('Save provider') : t('Create provider')}
            </Button>
          </FormActions>
        </form>
      </DialogContent>
    </Dialog>
  );
}
