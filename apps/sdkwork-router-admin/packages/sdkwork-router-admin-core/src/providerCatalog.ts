import type {
  ChannelModelRecord,
  ModelPriceRecord,
  ProviderCatalogRecord,
  ProviderIntegrationMode,
  ProviderModelRecord,
  ProviderRecordWithIntegration,
  SaveProviderInput,
  SaveProviderSupportedModelInput,
} from 'sdkwork-router-admin-types';

export type StandardProviderProtocol = 'openai' | 'anthropic' | 'gemini';
export type DefaultPluginFamily = 'openrouter' | 'siliconflow' | 'ollama';

export type ProviderSupportedModelDraft = {
  channel_id: string;
  model_id: string;
  provider_model_id: string;
  provider_model_family: string;
  capabilities: string[];
  streaming?: boolean | null;
  context_window?: number | null;
  max_output_tokens?: number | null;
  supports_prompt_caching: boolean;
  supports_reasoning_usage: boolean;
  supports_tool_usage_metrics: boolean;
  is_default_route: boolean;
  is_active: boolean;
};

export type ProviderDraft = {
  id: string;
  display_name: string;
  integration_mode: ProviderIntegrationMode;
  standard_protocol: StandardProviderProtocol;
  default_plugin_family: string;
  adapter_kind: string;
  protocol_kind: string;
  extension_id: string;
  base_url: string;
  primary_channel_id: string;
  bound_channel_ids: string[];
  supported_models: ProviderSupportedModelDraft[];
};

export type ProviderPricingCoverageSummary = {
  active_model_count: number;
  priced_model_count: number;
  missing_price_count: number;
  default_route_count: number;
};

export const STANDARD_PROVIDER_PROTOCOL_OPTIONS: Array<{
  label: string;
  value: StandardProviderProtocol;
}> = [
  { label: 'OpenAI', value: 'openai' },
  { label: 'Anthropic', value: 'anthropic' },
  { label: 'Gemini', value: 'gemini' },
];

export const DEFAULT_PLUGIN_FAMILY_OPTIONS: Array<{
  label: string;
  value: DefaultPluginFamily;
}> = [
  { label: 'OpenRouter', value: 'openrouter' },
  { label: 'SiliconFlow', value: 'siliconflow' },
  { label: 'Ollama', value: 'ollama' },
];

export const CUSTOM_PLUGIN_PROTOCOL_OPTIONS: Array<{
  label: string;
  value: string;
}> = [
  { label: 'OpenAI', value: 'openai' },
  { label: 'Anthropic', value: 'anthropic' },
  { label: 'Gemini', value: 'gemini' },
  { label: 'Custom', value: 'custom' },
];

function normalizeText(value: string | null | undefined): string {
  return value?.trim() ?? '';
}

function normalizeProviderIntegrationMode(
  value: string | null | undefined,
): ProviderIntegrationMode {
  if (
    value === 'standard_passthrough'
    || value === 'default_plugin'
    || value === 'custom_plugin'
  ) {
    return value;
  }
  return 'custom_plugin';
}

function normalizeStandardProtocol(
  value: string | null | undefined,
): StandardProviderProtocol {
  const normalized = normalizeText(value).toLowerCase();
  if (
    normalized === 'openai'
    || normalized === 'anthropic'
    || normalized === 'gemini'
  ) {
    return normalized;
  }
  return 'openai';
}

function deriveDefaultPluginProtocol(
  defaultPluginFamily: string,
): StandardProviderProtocol | 'custom' {
  switch (normalizeText(defaultPluginFamily).toLowerCase()) {
    case 'openrouter':
    case 'siliconflow':
      return 'openai';
    case 'ollama':
      return 'custom';
    default:
      return 'custom';
  }
}

function deriveDefaultPluginExtensionId(defaultPluginFamily: string): string {
  switch (normalizeText(defaultPluginFamily).toLowerCase()) {
    case 'openrouter':
      return 'sdkwork.provider.openrouter';
    case 'siliconflow':
      return 'sdkwork.provider.siliconflow';
    case 'ollama':
      return 'sdkwork.provider.ollama';
    default:
      return '';
  }
}

export function providerSupportedModelKey(model: {
  channel_id: string;
  model_id: string;
}) {
  return `${model.channel_id}:${model.model_id}`;
}

function normalizeCapabilities(capabilities: string[]) {
  return Array.from(
    new Set(
      capabilities
        .map((capability) => capability.trim())
        .filter(Boolean),
    ),
  );
}

function providerModelPriceKey(model: {
  proxy_provider_id: string;
  channel_id: string;
  model_id: string;
}) {
  return `${model.proxy_provider_id}:${model.channel_id}:${model.model_id}`;
}

export function providerSupportedModelDraftFromChannelModel(
  channelModel: ChannelModelRecord,
  existing?: ProviderModelRecord | Partial<ProviderSupportedModelDraft> | null,
): ProviderSupportedModelDraft {
  return {
    channel_id: channelModel.channel_id,
    model_id: channelModel.model_id,
    provider_model_id:
      normalizeText(existing?.provider_model_id) || channelModel.model_id,
    provider_model_family: normalizeText(existing?.provider_model_family),
    capabilities:
      normalizeCapabilities(existing?.capabilities ?? channelModel.capabilities),
    streaming: existing?.streaming ?? channelModel.streaming,
    context_window: existing?.context_window ?? channelModel.context_window ?? null,
    max_output_tokens: existing?.max_output_tokens ?? null,
    supports_prompt_caching: existing?.supports_prompt_caching ?? false,
    supports_reasoning_usage: existing?.supports_reasoning_usage ?? false,
    supports_tool_usage_metrics: existing?.supports_tool_usage_metrics ?? false,
    is_default_route: existing?.is_default_route ?? false,
    is_active: existing?.is_active ?? true,
  };
}

function normalizeSupportedModelInput(
  model: ProviderSupportedModelDraft,
): SaveProviderSupportedModelInput {
  const providerModelId = normalizeText(model.provider_model_id) || model.model_id;
  const providerModelFamily = normalizeText(model.provider_model_family);
  return {
    channel_id: normalizeText(model.channel_id),
    model_id: normalizeText(model.model_id),
    provider_model_id: providerModelId,
    provider_model_family: providerModelFamily || undefined,
    capabilities: normalizeCapabilities(model.capabilities),
    streaming: model.streaming ?? undefined,
    context_window: model.context_window ?? undefined,
    max_output_tokens: model.max_output_tokens ?? undefined,
    supports_prompt_caching: model.supports_prompt_caching,
    supports_reasoning_usage: model.supports_reasoning_usage,
    supports_tool_usage_metrics: model.supports_tool_usage_metrics,
    is_default_route: model.is_default_route,
    is_active: model.is_active,
  };
}

function collectProviderChannelIds(
  provider: ProviderRecordWithIntegration | ProviderCatalogRecord,
): string[] {
  const ids = new Set<string>([provider.channel_id]);
  for (const binding of provider.channel_bindings) {
    ids.add(binding.channel_id);
  }
  return Array.from(ids);
}

function baseProviderSaveInput(draft: ProviderDraft): SaveProviderInput {
  const primaryChannelId = normalizeText(draft.primary_channel_id);
  const bindingIds = Array.from(
    new Set(
      [primaryChannelId, ...draft.bound_channel_ids]
        .map((value) => value.trim())
        .filter(Boolean),
    ),
  );
  const supportedModels = draft.supported_models
    .map(normalizeSupportedModelInput)
    .filter((model) => model.channel_id && model.model_id);

  return {
    id: normalizeText(draft.id),
    channel_id: primaryChannelId,
    base_url: normalizeText(draft.base_url),
    display_name: normalizeText(draft.display_name),
    channel_bindings: bindingIds.map((channelId) => ({
      channel_id: channelId,
      is_primary: channelId === primaryChannelId,
    })),
    ...(supportedModels.length > 0 ? { supported_models: supportedModels } : {}),
  };
}

export function emptyProviderDraft(defaultChannelId: string): ProviderDraft {
  return {
    id: '',
    display_name: '',
    integration_mode: 'standard_passthrough',
    standard_protocol: 'openai',
    default_plugin_family: '',
    adapter_kind: 'openai',
    protocol_kind: 'openai',
    extension_id: '',
    base_url: '',
    primary_channel_id: defaultChannelId,
    bound_channel_ids: defaultChannelId ? [defaultChannelId] : [],
    supported_models: [],
  };
}

export function providerDraftFromRecord(
  provider: ProviderRecordWithIntegration | ProviderCatalogRecord,
  providerModels: ProviderModelRecord[] = [],
): ProviderDraft {
  const integrationMode = normalizeProviderIntegrationMode(provider.integration.mode);
  const standardProtocol = normalizeStandardProtocol(
    integrationMode === 'standard_passthrough'
      ? provider.protocol_kind || provider.adapter_kind
      : provider.protocol_kind,
  );

  return {
    id: provider.id,
    display_name: provider.display_name,
    integration_mode: integrationMode,
    standard_protocol: standardProtocol,
    default_plugin_family: normalizeText(provider.integration.default_plugin_family),
    adapter_kind: provider.adapter_kind,
    protocol_kind: normalizeText(provider.protocol_kind),
    extension_id: normalizeText(provider.extension_id),
    base_url: provider.base_url,
    primary_channel_id: provider.channel_id,
    bound_channel_ids: collectProviderChannelIds(provider),
    supported_models: providerModels
      .filter((record) => record.proxy_provider_id === provider.id)
      .map((record) =>
        providerSupportedModelDraftFromChannelModel(
          {
            channel_id: record.channel_id,
            model_id: record.model_id,
            model_display_name: record.model_id,
            capabilities: record.capabilities,
            streaming: record.streaming,
            context_window: record.context_window,
            description: null,
          },
          record,
        ),
      ),
  };
}

export function buildProviderSaveInput(draft: ProviderDraft): SaveProviderInput {
  const base = baseProviderSaveInput(draft);

  if (draft.integration_mode === 'default_plugin') {
    return {
      ...base,
      default_plugin_family: normalizeText(draft.default_plugin_family),
    };
  }

  if (draft.integration_mode === 'custom_plugin') {
    const payload: SaveProviderInput = {
      ...base,
      adapter_kind: normalizeText(draft.adapter_kind),
    };
    const protocolKind = normalizeText(draft.protocol_kind);
    const extensionId = normalizeText(draft.extension_id);

    if (protocolKind) {
      payload.protocol_kind = protocolKind;
    }
    if (extensionId) {
      payload.extension_id = extensionId;
    }

    return payload;
  }

  return {
    ...base,
    adapter_kind: draft.standard_protocol,
  };
}

export function findProviderModelPrice(
  providerModel: Pick<ProviderModelRecord, 'proxy_provider_id' | 'channel_id' | 'model_id'>,
  modelPrices: ModelPriceRecord[],
): ModelPriceRecord | null {
  return modelPrices.find(
    (record) =>
      providerModelPriceKey(record) === providerModelPriceKey(providerModel),
  ) ?? null;
}

export function summarizeProviderPricingCoverage(
  providerId: string,
  providerModels: ProviderModelRecord[],
  modelPrices: ModelPriceRecord[],
): ProviderPricingCoverageSummary {
  const activeProviderModels = providerModels.filter(
    (record) => record.proxy_provider_id === providerId && record.is_active,
  );
  const activePriceKeys = new Set(
    modelPrices
      .filter((record) => record.proxy_provider_id === providerId && record.is_active)
      .map((record) => providerModelPriceKey(record)),
  );

  const pricedModelCount = activeProviderModels.filter((record) =>
    activePriceKeys.has(providerModelPriceKey(record)),
  ).length;

  return {
    active_model_count: activeProviderModels.length,
    priced_model_count: pricedModelCount,
    missing_price_count: Math.max(activeProviderModels.length - pricedModelCount, 0),
    default_route_count: activeProviderModels.filter((record) => record.is_default_route)
      .length,
  };
}

export function recommendedModelPriceSourceKind(
  provider: ProviderRecordWithIntegration | ProviderCatalogRecord | null | undefined,
): 'official' | 'proxy' | 'local' | 'reference' {
  if (!provider) {
    return 'reference';
  }

  const adapterKind = normalizeText(provider.adapter_kind).toLowerCase();
  const defaultPluginFamily = normalizeText(
    provider.integration.default_plugin_family,
  ).toLowerCase();
  const primaryChannelId = normalizeText(provider.channel_id).toLowerCase();

  if (
    defaultPluginFamily === 'ollama'
    || adapterKind === 'ollama'
    || primaryChannelId === 'ollama'
  ) {
    return 'local';
  }

  if (provider.integration.mode === 'standard_passthrough') {
    return 'official';
  }

  if (provider.integration.mode === 'default_plugin') {
    return 'proxy';
  }

  return 'proxy';
}

export function applyProviderIntegrationMode(
  draft: ProviderDraft,
  integrationMode: ProviderIntegrationMode,
): ProviderDraft {
  if (integrationMode === 'default_plugin') {
    const defaultPluginFamily =
      normalizeText(draft.default_plugin_family) || 'openrouter';
    const protocolKind = deriveDefaultPluginProtocol(defaultPluginFamily);

    return {
      ...draft,
      integration_mode: integrationMode,
      default_plugin_family: defaultPluginFamily,
      adapter_kind: defaultPluginFamily,
      protocol_kind: protocolKind === 'custom' ? 'custom' : protocolKind,
      extension_id: deriveDefaultPluginExtensionId(defaultPluginFamily),
    };
  }

  if (integrationMode === 'custom_plugin') {
    return {
      ...draft,
      integration_mode: integrationMode,
      adapter_kind: normalizeText(draft.adapter_kind) || 'native-dynamic',
      protocol_kind: normalizeText(draft.protocol_kind) || 'custom',
    };
  }

  return {
    ...draft,
    integration_mode: integrationMode,
    default_plugin_family: '',
    adapter_kind: draft.standard_protocol,
    protocol_kind: draft.standard_protocol,
    extension_id: '',
  };
}

export function applyProviderStandardProtocol(
  draft: ProviderDraft,
  protocol: StandardProviderProtocol,
): ProviderDraft {
  return {
    ...draft,
    standard_protocol: protocol,
    adapter_kind: protocol,
    protocol_kind: protocol,
    extension_id: '',
  };
}

export function applyProviderDefaultPluginFamily(
  draft: ProviderDraft,
  defaultPluginFamily: DefaultPluginFamily,
): ProviderDraft {
  const protocolKind = deriveDefaultPluginProtocol(defaultPluginFamily);

  return {
    ...draft,
    default_plugin_family: defaultPluginFamily,
    adapter_kind: defaultPluginFamily,
    protocol_kind: protocolKind === 'custom' ? 'custom' : protocolKind,
    extension_id: deriveDefaultPluginExtensionId(defaultPluginFamily),
  };
}

export function describeProviderIntegration(
  provider: ProviderRecordWithIntegration | ProviderCatalogRecord,
): string {
  if (provider.integration.mode === 'default_plugin') {
    return `default-plugin/${provider.integration.default_plugin_family ?? provider.adapter_kind}`;
  }
  if (provider.integration.mode === 'standard_passthrough') {
    return `standard/${provider.protocol_kind}`;
  }
  return `custom/${provider.adapter_kind}`;
}
