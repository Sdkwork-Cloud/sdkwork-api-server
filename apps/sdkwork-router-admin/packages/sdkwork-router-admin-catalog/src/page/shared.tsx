import type { ReactNode } from 'react';
import {
  Button,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  Label,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@sdkwork/ui-pc-react';
import {
  emptyProviderDraft as createEmptyProviderDraft,
  providerDraftFromRecord as createProviderDraftFromRecord,
  translateAdminText,
  type ProviderDraft,
} from 'sdkwork-router-admin-core';
import type {
  AdminPageProps,
  ChannelModelRecord,
  CredentialRecord,
  ModelPriceRecord,
  ModelPriceTier,
  ProviderCatalogRecord,
  ProviderModelRecord,
  ProxyProviderRecord,
} from 'sdkwork-router-admin-types';

export type CatalogLane = 'channels' | 'providers' | 'credentials' | 'variants';
export type { ProviderDraft } from 'sdkwork-router-admin-core';
export type ChannelRecord = AdminPageProps['snapshot']['channels'][number];
export type VariantRecord = AdminPageProps['snapshot']['models'][number];
export type ChannelDraft = { id: string; name: string };
export type CredentialDraft = {
  tenant_id: string;
  provider_id: string;
  key_reference: string;
  secret_value: string;
};
export type ChannelModelDraft = {
  channel_id: string;
  model_id: string;
  model_display_name: string;
  capabilities: string;
  streaming: boolean;
  context_window: string;
  description: string;
};
export type ModelPriceDraft = {
  channel_id: string;
  model_id: string;
  proxy_provider_id: string;
  currency_code: string;
  price_unit: string;
  input_price: string;
  output_price: string;
  cache_read_price: string;
  cache_write_price: string;
  request_price: string;
  price_source_kind: string;
  billing_notes: string;
  pricing_tiers_json: string;
  is_active: boolean;
};
export type PendingDelete =
  | { kind: 'channel'; label: string; channelId: string }
  | { kind: 'provider'; label: string; providerId: string }
  | {
      kind: 'credential';
      label: string;
      tenantId: string;
      providerId: string;
      keyReference: string;
    }
  | { kind: 'channel-model'; label: string; channelId: string; modelId: string }
  | {
      kind: 'model-price';
      label: string;
      channelId: string;
      modelId: string;
      proxyProviderId: string;
    }
  | { kind: 'model'; label: string; externalName: string; providerId: string }
  | null;

export const PRICE_UNIT_OPTIONS = [
  { value: 'per_1m_tokens', label: 'Million tokens' },
  { value: 'per_1k_tokens', label: 'Thousand tokens' },
  { value: 'per_request', label: 'Request' },
  { value: 'per_image', label: 'Image generated' },
  { value: 'per_second_audio', label: 'Audio second' },
  { value: 'per_minute_video', label: 'Video minute' },
  { value: 'per_track', label: 'Music track' },
] as const;

export const MODEL_PRICE_SOURCE_OPTIONS = [
  { value: 'official', label: 'Official pricing' },
  { value: 'proxy', label: 'Proxy provider pricing' },
  { value: 'local', label: 'Local runtime reference' },
  { value: 'reference', label: 'Reference pricing' },
] as const;

export function DialogField({
  children,
  description,
  htmlFor,
  label,
}: {
  children: ReactNode;
  description?: ReactNode;
  htmlFor?: string;
  label: ReactNode;
}) {
  return (
    <div className="space-y-2">
      <Label htmlFor={htmlFor}>{label}</Label>
      {children}
      {description ? (
        <div className="text-xs text-[var(--sdk-color-text-secondary)]">
          {description}
        </div>
      ) : null}
    </div>
  );
}

export function SelectField<T extends string>({
  disabled,
  label,
  onValueChange,
  options,
  placeholder,
  value,
}: {
  disabled?: boolean;
  label: ReactNode;
  onValueChange: (value: T) => void;
  options: Array<{ label: ReactNode; value: T }>;
  placeholder?: string;
  value: T;
}) {
  return (
    <div className="space-y-2">
      <Label>{label}</Label>
      <Select
        disabled={disabled}
        onValueChange={(nextValue: string) => onValueChange(nextValue as T)}
        value={value}
      >
        <SelectTrigger>
          <SelectValue placeholder={placeholder ?? String(label)} />
        </SelectTrigger>
        <SelectContent>
          {options.map((option) => (
            <SelectItem key={option.value} value={option.value}>
              {option.label}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
    </div>
  );
}

export function ConfirmActionDialog({
  confirmLabel = translateAdminText('Confirm'),
  description,
  onConfirm,
  onOpenChange,
  open,
  title,
}: {
  confirmLabel?: string;
  description: ReactNode;
  onConfirm: () => void | Promise<void>;
  onOpenChange: (open: boolean) => void;
  open: boolean;
  title: ReactNode;
}) {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="w-[min(92vw,28rem)]">
        <DialogHeader>
          <DialogTitle>{title}</DialogTitle>
          <DialogDescription>{description}</DialogDescription>
        </DialogHeader>
        <DialogFooter>
          <Button onClick={() => onOpenChange(false)} type="button" variant="outline">
            {translateAdminText('Cancel')}
          </Button>
          <Button onClick={() => void onConfirm()} type="button" variant="danger">
            {confirmLabel}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

export function priceUnitLabel(value: string) {
  return translateAdminText(
    PRICE_UNIT_OPTIONS.find((option) => option.value === value)?.label ?? value,
  );
}

export function modelPriceSourceLabel(value: string) {
  return translateAdminText(
    MODEL_PRICE_SOURCE_OPTIONS.find((option) => option.value === value)?.label ?? value,
  );
}

export function splitCapabilities(value: string) {
  return value
    .split(',')
    .map((item) => item.trim())
    .filter(Boolean);
}

export function parseOptionalNumber(value: string) {
  const trimmed = value.trim();
  if (!trimmed) return null;
  const parsed = Number(trimmed);
  return Number.isFinite(parsed) ? parsed : null;
}

export function parseRequiredNumber(value: string) {
  const parsed = Number(value.trim() || '0');
  return Number.isFinite(parsed) ? parsed : 0;
}

export function formatPricingTiersJson(pricingTiers: ModelPriceTier[]) {
  if (pricingTiers.length === 0) {
    return '';
  }

  return JSON.stringify(pricingTiers, null, 2);
}

export function parsePricingTiersJson(value: string): ModelPriceTier[] {
  const trimmed = value.trim();
  if (!trimmed) {
    return [];
  }

  const parsed = JSON.parse(trimmed);
  if (!Array.isArray(parsed)) {
    throw new Error(translateAdminText('Pricing tiers JSON must be an array.'));
  }

  return parsed as ModelPriceTier[];
}

export function providerChannelIds(provider: ProxyProviderRecord) {
  const ids = new Set<string>([provider.channel_id]);
  for (const binding of provider.channel_bindings) {
    ids.add(binding.channel_id);
  }
  return Array.from(ids);
}

export function credentialStorageLabel(credential: CredentialRecord) {
  if (credential.secret_backend === 'local_encrypted_file') {
    return credential.secret_local_file ?? translateAdminText('local encrypted file');
  }

  if (credential.secret_backend === 'os_keyring') {
    return credential.secret_keyring_service ?? translateAdminText('os keyring');
  }

  return translateAdminText('database envelope');
}

export function emptyProviderDraft(channelId = ''): ProviderDraft {
  return createEmptyProviderDraft(channelId);
}

export function providerDraftFromRecord(
  record: ProviderCatalogRecord,
  providerModels: ProviderModelRecord[] = [],
): ProviderDraft {
  return createProviderDraftFromRecord(record, providerModels);
}

export function emptyCredentialDraft(tenantId = '', providerId = ''): CredentialDraft {
  return {
    tenant_id: tenantId,
    provider_id: providerId,
    key_reference: '',
    secret_value: '',
  };
}

export function credentialDraftFromRecord(record: CredentialRecord): CredentialDraft {
  return {
    tenant_id: record.tenant_id,
    provider_id: record.provider_id,
    key_reference: record.key_reference,
    secret_value: '',
  };
}

export function emptyChannelModelDraft(
  channelId = '',
  modelId = '',
  displayName = '',
): ChannelModelDraft {
  return {
    channel_id: channelId,
    model_id: modelId,
    model_display_name: displayName || modelId,
    capabilities: 'chat',
    streaming: true,
    context_window: '',
    description: '',
  };
}

export function channelModelDraftFromRecord(
  record: ChannelModelRecord,
): ChannelModelDraft {
  return {
    channel_id: record.channel_id,
    model_id: record.model_id,
    model_display_name: record.model_display_name,
    capabilities: record.capabilities.join(', '),
    streaming: record.streaming,
    context_window: String(record.context_window ?? ''),
    description: record.description ?? '',
  };
}

export function emptyModelPriceDraft(
  channelId = '',
  modelId = '',
  proxyProviderId = '',
  priceSourceKind = 'reference',
): ModelPriceDraft {
  return {
    channel_id: channelId,
    model_id: modelId,
    proxy_provider_id: proxyProviderId,
    currency_code: 'USD',
    price_unit: 'per_1m_tokens',
    input_price: '0',
    output_price: '0',
    cache_read_price: '0',
    cache_write_price: '0',
    request_price: '0',
    price_source_kind: priceSourceKind,
    billing_notes: '',
    pricing_tiers_json: '',
    is_active: true,
  };
}

export function modelPriceDraftFromRecord(record: ModelPriceRecord): ModelPriceDraft {
  return {
    channel_id: record.channel_id,
    model_id: record.model_id,
    proxy_provider_id: record.proxy_provider_id,
    currency_code: record.currency_code,
    price_unit: record.price_unit,
    input_price: String(record.input_price),
    output_price: String(record.output_price),
    cache_read_price: String(record.cache_read_price),
    cache_write_price: String(record.cache_write_price),
    request_price: String(record.request_price),
    price_source_kind: record.price_source_kind,
    billing_notes: record.billing_notes ?? '',
    pricing_tiers_json: formatPricingTiersJson(record.pricing_tiers),
    is_active: record.is_active,
  };
}
