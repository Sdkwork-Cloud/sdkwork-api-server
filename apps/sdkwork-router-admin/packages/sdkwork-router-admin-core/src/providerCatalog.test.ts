// @ts-nocheck
import test from 'node:test';
import assert from 'node:assert/strict';

import {
  buildProviderSaveInput,
  emptyProviderDraft,
  providerDraftFromRecord,
  recommendedModelPriceSourceKind,
  summarizeProviderPricingCoverage,
} from './providerCatalog.ts';
import type {
  ModelPriceRecord,
  ProviderCatalogRecord,
  ProviderRecordWithIntegration,
} from 'sdkwork-router-admin-types';

function createProviderRecord(
  overrides: Partial<ProviderCatalogRecord> = {},
): ProviderCatalogRecord {
  return {
    id: 'provider-openrouter-main',
    channel_id: 'openrouter',
    extension_id: 'sdkwork.provider.openrouter',
    adapter_kind: 'openrouter',
    protocol_kind: 'openai',
    base_url: 'https://openrouter.ai/api/v1',
    display_name: 'OpenRouter Main',
    channel_bindings: [{ channel_id: 'openrouter', is_primary: true }],
    integration: {
      mode: 'default_plugin',
      default_plugin_family: 'openrouter',
    },
    execution: {
      binding_kind: 'builtin',
      runtime: 'builtin',
      runtime_key: 'openrouter',
      passthrough_protocol: 'openai',
      supports_provider_adapter: true,
      supports_raw_plugin: false,
      fail_closed: true,
      route_readiness: {
        openai: { executable: true, supported: true },
        anthropic: { executable: false, supported: false },
        gemini: { executable: false, supported: false },
      },
      reason: null,
    },
    credential_readiness: {
      ready: true,
      state: 'ready',
    },
    ...overrides,
  };
}

test('emptyProviderDraft defaults to standard passthrough openai mode', () => {
  const draft = emptyProviderDraft('openai');

  assert.equal(draft.integration_mode, 'standard_passthrough');
  assert.equal(draft.standard_protocol, 'openai');
  assert.equal(draft.default_plugin_family, '');
  assert.equal(draft.adapter_kind, 'openai');
  assert.equal(draft.protocol_kind, 'openai');
  assert.deepEqual(draft.bound_channel_ids, ['openai']);
});

test('providerDraftFromRecord promotes default plugin family to first-class draft mode', () => {
  const draft = providerDraftFromRecord(createProviderRecord());

  assert.equal(draft.integration_mode, 'default_plugin');
  assert.equal(draft.default_plugin_family, 'openrouter');
  assert.equal(draft.standard_protocol, 'openai');
  assert.equal(draft.adapter_kind, 'openrouter');
  assert.equal(draft.protocol_kind, 'openai');
});

test('buildProviderSaveInput serializes standard passthrough providers without plugin fields', () => {
  const payload = buildProviderSaveInput({
    ...emptyProviderDraft('openai'),
    id: 'provider-gemini-main',
    display_name: 'Gemini Main',
    integration_mode: 'standard_passthrough',
    standard_protocol: 'gemini',
    base_url: 'https://generativelanguage.googleapis.com/v1beta/openai',
  });

  assert.deepEqual(payload, {
    id: 'provider-gemini-main',
    channel_id: 'openai',
    adapter_kind: 'gemini',
    base_url: 'https://generativelanguage.googleapis.com/v1beta/openai',
    display_name: 'Gemini Main',
    channel_bindings: [{ channel_id: 'openai', is_primary: true }],
  });
});

test('buildProviderSaveInput serializes default plugin providers with default_plugin_family only', () => {
  const payload = buildProviderSaveInput({
    ...emptyProviderDraft('openrouter'),
    id: 'provider-openrouter-main',
    display_name: 'OpenRouter Main',
    integration_mode: 'default_plugin',
    default_plugin_family: 'openrouter',
    adapter_kind: 'openrouter',
    protocol_kind: 'openai',
    extension_id: 'sdkwork.provider.openrouter',
    base_url: 'https://openrouter.ai/api/v1',
  });

  assert.deepEqual(payload, {
    id: 'provider-openrouter-main',
    channel_id: 'openrouter',
    default_plugin_family: 'openrouter',
    base_url: 'https://openrouter.ai/api/v1',
    display_name: 'OpenRouter Main',
    channel_bindings: [{ channel_id: 'openrouter', is_primary: true }],
  });
});

test('buildProviderSaveInput serializes supported canonical models for proxy providers', () => {
  const payload = buildProviderSaveInput({
    ...emptyProviderDraft('openai'),
    id: 'provider-openrouter-main',
    display_name: 'OpenRouter Main',
    integration_mode: 'default_plugin',
    default_plugin_family: 'openrouter',
    adapter_kind: 'openrouter',
    protocol_kind: 'openai',
    extension_id: 'sdkwork.provider.openrouter',
    base_url: 'https://openrouter.ai/api/v1',
    bound_channel_ids: ['openai', 'anthropic'],
    supported_models: [
      {
        channel_id: 'openai',
        model_id: 'gpt-4.1',
        provider_model_id: 'openai/gpt-4.1',
        provider_model_family: 'openai',
        capabilities: ['responses'],
        streaming: true,
        context_window: 128000,
        max_output_tokens: 32768,
        supports_prompt_caching: false,
        supports_reasoning_usage: true,
        supports_tool_usage_metrics: false,
        is_default_route: true,
        is_active: true,
      },
      {
        channel_id: 'anthropic',
        model_id: 'claude-3-7-sonnet',
        provider_model_id: 'anthropic/claude-3.7-sonnet',
        provider_model_family: 'anthropic',
        capabilities: ['responses'],
        streaming: true,
        context_window: 200000,
        max_output_tokens: 16384,
        supports_prompt_caching: true,
        supports_reasoning_usage: false,
        supports_tool_usage_metrics: false,
        is_default_route: false,
        is_active: true,
      },
    ],
  });

  assert.deepEqual(payload.supported_models, [
    {
      channel_id: 'openai',
      model_id: 'gpt-4.1',
      provider_model_id: 'openai/gpt-4.1',
      provider_model_family: 'openai',
      capabilities: ['responses'],
      streaming: true,
      context_window: 128000,
      max_output_tokens: 32768,
      supports_prompt_caching: false,
      supports_reasoning_usage: true,
      supports_tool_usage_metrics: false,
      is_default_route: true,
      is_active: true,
    },
    {
      channel_id: 'anthropic',
      model_id: 'claude-3-7-sonnet',
      provider_model_id: 'anthropic/claude-3.7-sonnet',
      provider_model_family: 'anthropic',
      capabilities: ['responses'],
      streaming: true,
      context_window: 200000,
      max_output_tokens: 16384,
      supports_prompt_caching: true,
      supports_reasoning_usage: false,
      supports_tool_usage_metrics: false,
      is_default_route: false,
      is_active: true,
    },
  ]);
});

test('buildProviderSaveInput serializes siliconflow as a default plugin family', () => {
  const payload = buildProviderSaveInput({
    ...emptyProviderDraft('deepseek'),
    id: 'provider-siliconflow-main',
    display_name: 'SiliconFlow Main',
    integration_mode: 'default_plugin',
    default_plugin_family: 'siliconflow',
    adapter_kind: 'siliconflow',
    protocol_kind: 'openai',
    extension_id: 'sdkwork.provider.siliconflow',
    base_url: 'https://api.siliconflow.cn/v1',
    bound_channel_ids: ['deepseek', 'qwen'],
  });

  assert.deepEqual(payload, {
    id: 'provider-siliconflow-main',
    channel_id: 'deepseek',
    default_plugin_family: 'siliconflow',
    base_url: 'https://api.siliconflow.cn/v1',
    display_name: 'SiliconFlow Main',
    channel_bindings: [
      { channel_id: 'deepseek', is_primary: true },
      { channel_id: 'qwen', is_primary: false },
    ],
  });
});

test('buildProviderSaveInput preserves advanced custom plugin fields', () => {
  const payload = buildProviderSaveInput({
    ...emptyProviderDraft('anthropic'),
    id: 'provider-claude-relay',
    display_name: 'Claude Relay',
    integration_mode: 'custom_plugin',
    adapter_kind: 'native-dynamic',
    protocol_kind: 'anthropic',
    extension_id: 'sdkwork.provider.claude.relay',
    base_url: 'https://relay.example.com',
  });

  assert.deepEqual(payload, {
    id: 'provider-claude-relay',
    channel_id: 'anthropic',
    adapter_kind: 'native-dynamic',
    protocol_kind: 'anthropic',
    extension_id: 'sdkwork.provider.claude.relay',
    base_url: 'https://relay.example.com',
    display_name: 'Claude Relay',
    channel_bindings: [{ channel_id: 'anthropic', is_primary: true }],
  });
});

test('providerDraftFromRecord keeps custom plugin mode for non-default integrations', () => {
  const record: ProviderRecordWithIntegration = {
    id: 'provider-claude-relay',
    channel_id: 'anthropic',
    extension_id: 'sdkwork.provider.claude.relay',
    adapter_kind: 'native-dynamic',
    protocol_kind: 'anthropic',
    base_url: 'https://relay.example.com',
    display_name: 'Claude Relay',
    channel_bindings: [{ channel_id: 'anthropic', is_primary: true }],
    integration: {
      mode: 'custom_plugin',
      default_plugin_family: null,
    },
  };

  const draft = providerDraftFromRecord(record);

  assert.equal(draft.integration_mode, 'custom_plugin');
  assert.equal(draft.adapter_kind, 'native-dynamic');
  assert.equal(draft.protocol_kind, 'anthropic');
  assert.equal(draft.extension_id, 'sdkwork.provider.claude.relay');
});

test('recommendedModelPriceSourceKind classifies official, proxy, and local providers', () => {
  const official = createProviderRecord({
    id: 'provider-openai-official',
    channel_id: 'openai',
    adapter_kind: 'openai',
    protocol_kind: 'openai',
    integration: {
      mode: 'standard_passthrough',
      default_plugin_family: null,
    },
  });
  const proxy = createProviderRecord();
  const local = createProviderRecord({
    id: 'provider-ollama-local',
    channel_id: 'ollama',
    adapter_kind: 'ollama',
    protocol_kind: 'custom',
    integration: {
      mode: 'default_plugin',
      default_plugin_family: 'ollama',
    },
  });

  assert.equal(recommendedModelPriceSourceKind(official), 'official');
  assert.equal(recommendedModelPriceSourceKind(proxy), 'proxy');
  assert.equal(recommendedModelPriceSourceKind(local), 'local');
});

test('summarizeProviderPricingCoverage counts active priced and missing provider models', () => {
  const modelPrices: ModelPriceRecord[] = [
    {
      channel_id: 'openai',
      model_id: 'gpt-4.1',
      proxy_provider_id: 'provider-openrouter-main',
      currency_code: 'USD',
      price_unit: 'per_1m_tokens',
      input_price: 2.5,
      output_price: 10,
      cache_read_price: 0.4,
      cache_write_price: 1.2,
      request_price: 0,
      price_source_kind: 'proxy',
      billing_notes: 'Pass-through proxy pricing.',
      pricing_tiers: [],
      is_active: true,
    },
  ];

  const summary = summarizeProviderPricingCoverage(
    'provider-openrouter-main',
    [
      {
        proxy_provider_id: 'provider-openrouter-main',
        channel_id: 'openai',
        model_id: 'gpt-4.1',
        provider_model_id: 'openai/gpt-4.1',
        provider_model_family: 'openai',
        capabilities: ['responses'],
        streaming: true,
        context_window: 128000,
        max_output_tokens: 32768,
        supports_prompt_caching: false,
        supports_reasoning_usage: true,
        supports_tool_usage_metrics: false,
        is_default_route: true,
        is_active: true,
      },
      {
        proxy_provider_id: 'provider-openrouter-main',
        channel_id: 'anthropic',
        model_id: 'claude-3-7-sonnet',
        provider_model_id: 'anthropic/claude-3.7-sonnet',
        provider_model_family: 'anthropic',
        capabilities: ['responses'],
        streaming: true,
        context_window: 200000,
        max_output_tokens: 16384,
        supports_prompt_caching: true,
        supports_reasoning_usage: false,
        supports_tool_usage_metrics: false,
        is_default_route: false,
        is_active: true,
      },
      {
        proxy_provider_id: 'provider-openrouter-main',
        channel_id: 'gemini',
        model_id: 'gemini-2.5-pro',
        provider_model_id: 'google/gemini-2.5-pro',
        provider_model_family: 'google',
        capabilities: ['responses'],
        streaming: true,
        context_window: 1048576,
        max_output_tokens: 65536,
        supports_prompt_caching: false,
        supports_reasoning_usage: false,
        supports_tool_usage_metrics: false,
        is_default_route: false,
        is_active: false,
      },
    ],
    modelPrices,
  );

  assert.deepEqual(summary, {
    active_model_count: 2,
    priced_model_count: 1,
    missing_price_count: 1,
    default_route_count: 1,
  });
});
