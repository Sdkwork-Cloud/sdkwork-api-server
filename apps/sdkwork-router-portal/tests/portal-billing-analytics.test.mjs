import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

import jiti from 'jiti';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

function loadBillingServices() {
  const load = jiti(import.meta.url, {
    moduleCache: false,
    alias: {
      'sdkwork-router-portal-commons/format-core': path.join(
        appRoot,
        'packages',
        'sdkwork-router-portal-commons',
        'src',
        'format-core.ts',
      ),
      'sdkwork-router-portal-commons/i18n-core': path.join(
        appRoot,
        'packages',
        'sdkwork-router-portal-commons',
        'src',
        'i18n-core.ts',
      ),
    },
  });

  return load(
    path.join(
      appRoot,
      'packages',
      'sdkwork-router-portal-billing',
      'src',
      'services',
      'index.ts',
    ),
  );
}

test('billing workspace consumes billing event contracts across types, api client, repository, and page workbench', () => {
  const portalTypes = read('packages/sdkwork-router-portal-types/src/index.ts');
  const portalApi = read('packages/sdkwork-router-portal-portal-api/src/index.ts');
  const repository = read('packages/sdkwork-router-portal-billing/src/repository/index.ts');
  const billingTypes = read('packages/sdkwork-router-portal-billing/src/types/index.ts');
  const billingPage = read('packages/sdkwork-router-portal-billing/src/pages/index.tsx');
  const billingServices = read('packages/sdkwork-router-portal-billing/src/services/index.ts');

  assert.match(portalTypes, /export interface BillingEventRecord/);
  assert.match(portalTypes, /export interface BillingEventSummary/);
  assert.match(portalTypes, /export interface BillingEventGroupSummary/);
  assert.match(portalTypes, /export interface BillingEventCapabilitySummary/);
  assert.match(portalTypes, /export interface BillingEventAccountingModeSummary/);

  assert.match(portalApi, /getPortalBillingEvents/);
  assert.match(portalApi, /getPortalBillingEventSummary/);
  assert.match(portalApi, /\/billing\/events/);
  assert.match(portalApi, /\/billing\/events\/summary/);

  assert.match(repository, /getPortalBillingEvents/);
  assert.match(repository, /getPortalBillingEventSummary/);
  assert.match(repository, /billing_events/);
  assert.match(repository, /billing_event_summary/);

  assert.match(billingTypes, /billing_events: BillingEventRecord\[];/);
  assert.match(billingTypes, /billing_event_summary: BillingEventSummary;/);
  assert.match(billingTypes, /BillingEventAnalyticsViewModel/);

  assert.match(billingServices, /buildBillingEventAnalytics/);

  assert.match(billingPage, /Billing event analytics/);
  assert.match(billingPage, /Capability mix/);
  assert.match(billingPage, /API key group chargeback/);
  assert.match(billingPage, /Accounting mode mix/);
  assert.match(billingPage, /Recent billing events/);
});

test('billing analytics service sorts recent multimodal billing evidence into commercial workbench slices', () => {
  const { buildBillingEventAnalytics } = loadBillingServices();

  const viewModel = buildBillingEventAnalytics(
    {
      total_events: 4,
      project_count: 1,
      group_count: 2,
      capability_count: 3,
      total_request_count: 7,
      total_units: 480,
      total_input_tokens: 160,
      total_output_tokens: 120,
      total_tokens: 280,
      total_image_count: 6,
      total_audio_seconds: 92,
      total_video_seconds: 48,
      total_music_seconds: 25,
      total_upstream_cost: 9.4,
      total_customer_charge: 14.8,
      projects: [],
      groups: [
        {
          api_key_group_id: 'group-enterprise',
          project_count: 1,
          event_count: 2,
          request_count: 3,
          total_upstream_cost: 6.2,
          total_customer_charge: 9.8,
        },
        {
          api_key_group_id: 'group-live',
          project_count: 1,
          event_count: 2,
          request_count: 4,
          total_upstream_cost: 3.2,
          total_customer_charge: 5.0,
        },
      ],
      capabilities: [
        {
          capability: 'images',
          event_count: 1,
          request_count: 1,
          total_tokens: 0,
          image_count: 6,
          audio_seconds: 0,
          video_seconds: 0,
          music_seconds: 0,
          total_upstream_cost: 4.4,
          total_customer_charge: 8.6,
        },
        {
          capability: 'responses',
          event_count: 2,
          request_count: 4,
          total_tokens: 280,
          image_count: 0,
          audio_seconds: 0,
          video_seconds: 0,
          music_seconds: 0,
          total_upstream_cost: 3.0,
          total_customer_charge: 4.6,
        },
        {
          capability: 'audio',
          event_count: 1,
          request_count: 2,
          total_tokens: 0,
          image_count: 0,
          audio_seconds: 92,
          video_seconds: 48,
          music_seconds: 25,
          total_upstream_cost: 2.0,
          total_customer_charge: 1.6,
        },
      ],
      accounting_modes: [
        {
          accounting_mode: 'byok',
          event_count: 1,
          request_count: 1,
          total_upstream_cost: 4.4,
          total_customer_charge: 8.6,
        },
        {
          accounting_mode: 'platform_credit',
          event_count: 3,
          request_count: 6,
          total_upstream_cost: 5.0,
          total_customer_charge: 6.2,
        },
      ],
    },
    [
      {
        event_id: 'evt_1',
        tenant_id: 'tenant-demo',
        project_id: 'project-demo',
        api_key_group_id: 'group-live',
        capability: 'responses',
        route_key: 'responses',
        usage_model: 'gpt-4.1',
        provider_id: 'provider-openrouter',
        accounting_mode: 'platform_credit',
        operation_kind: 'responses.create',
        modality: 'text',
        api_key_hash: 'key-live',
        channel_id: 'openai',
        reference_id: 'resp_1',
        latency_ms: 420,
        units: 180,
        request_count: 3,
        input_tokens: 120,
        output_tokens: 60,
        total_tokens: 180,
        cache_read_tokens: 0,
        cache_write_tokens: 0,
        image_count: 0,
        audio_seconds: 0,
        video_seconds: 0,
        music_seconds: 0,
        upstream_cost: 2.2,
        customer_charge: 3.8,
        applied_routing_profile_id: 'profile-live',
        compiled_routing_snapshot_id: 'snapshot-live',
        fallback_reason: null,
        created_at_ms: 100,
      },
      {
        event_id: 'evt_2',
        tenant_id: 'tenant-demo',
        project_id: 'project-demo',
        api_key_group_id: 'group-enterprise',
        capability: 'images',
        route_key: 'images',
        usage_model: 'gpt-image-1',
        provider_id: 'provider-openai',
        accounting_mode: 'byok',
        operation_kind: 'images.generate',
        modality: 'image',
        api_key_hash: 'key-enterprise',
        channel_id: 'openai',
        reference_id: 'img_1',
        latency_ms: 780,
        units: 120,
        request_count: 1,
        input_tokens: 0,
        output_tokens: 0,
        total_tokens: 0,
        cache_read_tokens: 0,
        cache_write_tokens: 0,
        image_count: 6,
        audio_seconds: 0,
        video_seconds: 0,
        music_seconds: 0,
        upstream_cost: 4.4,
        customer_charge: 8.6,
        applied_routing_profile_id: null,
        compiled_routing_snapshot_id: null,
        fallback_reason: null,
        created_at_ms: 400,
      },
      {
        event_id: 'evt_3',
        tenant_id: 'tenant-demo',
        project_id: 'project-demo',
        api_key_group_id: 'group-enterprise',
        capability: 'audio',
        route_key: 'audio',
        usage_model: 'gpt-4o-mini-transcribe',
        provider_id: 'provider-openai',
        accounting_mode: 'platform_credit',
        operation_kind: 'audio.transcriptions.create',
        modality: 'audio',
        api_key_hash: 'key-enterprise',
        channel_id: 'openai',
        reference_id: 'aud_1',
        latency_ms: 960,
        units: 90,
        request_count: 2,
        input_tokens: 0,
        output_tokens: 0,
        total_tokens: 0,
        cache_read_tokens: 0,
        cache_write_tokens: 0,
        image_count: 0,
        audio_seconds: 92,
        video_seconds: 48,
        music_seconds: 25,
        upstream_cost: 2.0,
        customer_charge: 1.6,
        applied_routing_profile_id: 'profile-audio',
        compiled_routing_snapshot_id: 'snapshot-audio',
        fallback_reason: 'latency_guardrail',
        created_at_ms: 300,
      },
      {
        event_id: 'evt_4',
        tenant_id: 'tenant-demo',
        project_id: 'project-demo',
        api_key_group_id: 'group-live',
        capability: 'responses',
        route_key: 'responses',
        usage_model: 'gpt-4.1-mini',
        provider_id: 'provider-openrouter',
        accounting_mode: 'platform_credit',
        operation_kind: 'responses.create',
        modality: 'text',
        api_key_hash: 'key-live',
        channel_id: 'openai',
        reference_id: 'resp_2',
        latency_ms: 380,
        units: 90,
        request_count: 1,
        input_tokens: 40,
        output_tokens: 20,
        total_tokens: 60,
        cache_read_tokens: 0,
        cache_write_tokens: 0,
        image_count: 0,
        audio_seconds: 0,
        video_seconds: 0,
        music_seconds: 0,
        upstream_cost: 0.8,
        customer_charge: 1.2,
        applied_routing_profile_id: 'profile-live',
        compiled_routing_snapshot_id: 'snapshot-live',
        fallback_reason: null,
        created_at_ms: 200,
      },
    ],
  );

  assert.equal(viewModel.totals.total_customer_charge, 14.8);
  assert.equal(viewModel.totals.total_tokens, 280);
  assert.equal(viewModel.totals.total_music_seconds, 25);
  assert.equal(viewModel.top_capabilities[0].capability, 'images');
  assert.equal(viewModel.top_capabilities[1].capability, 'responses');
  assert.equal(viewModel.group_chargeback[0].api_key_group_id, 'group-enterprise');
  assert.equal(viewModel.accounting_mode_mix[0].accounting_mode, 'byok');
  assert.equal(viewModel.recent_events[0].event_id, 'evt_2');
  assert.equal(viewModel.recent_events[1].event_id, 'evt_3');
  assert.equal(viewModel.routing_evidence.events_with_profile, 3);
  assert.equal(viewModel.routing_evidence.events_with_compiled_snapshot, 3);
  assert.equal(viewModel.routing_evidence.events_with_fallback_reason, 1);
});

test('billing workspace exposes billing event csv export and route evidence columns for finance review', () => {
  const billingPage = read('packages/sdkwork-router-portal-billing/src/pages/index.tsx');
  const billingServices = read('packages/sdkwork-router-portal-billing/src/services/index.ts');

  assert.match(billingServices, /buildBillingEventCsvDocument/);
  assert.match(billingPage, /Export billing events CSV/);
  assert.match(billingPage, /buildBillingEventCsvDocument/);
  assert.match(billingPage, /sdkwork-router-billing-events\.csv/);
  assert.match(billingPage, /Applied routing profile/);
  assert.match(billingPage, /Compiled snapshot/);
  assert.match(billingPage, /Fallback reason/);
});

test('billing workspace surfaces routing evidence counters for tenant-facing finance review', () => {
  const billingTypes = read('packages/sdkwork-router-portal-billing/src/types/index.ts');
  const billingPage = read('packages/sdkwork-router-portal-billing/src/pages/index.tsx');
  const billingServices = read('packages/sdkwork-router-portal-billing/src/services/index.ts');

  assert.match(billingTypes, /routing_evidence:/);
  assert.match(billingServices, /routing_evidence:/);
  assert.match(billingPage, /Routing evidence/);
  assert.match(billingPage, /routing_evidence\.events_with_profile/);
  assert.match(billingPage, /routing_evidence\.events_with_compiled_snapshot/);
  assert.match(billingPage, /routing_evidence\.events_with_fallback_reason/);
});

test('billing event csv export keeps multimodal and routing evidence columns stable', () => {
  const { buildBillingEventCsvDocument } = loadBillingServices();

  const document = buildBillingEventCsvDocument([
    {
      event_id: 'evt_export_1',
      tenant_id: 'tenant-demo',
      project_id: 'project-demo',
      api_key_group_id: 'group-enterprise',
      capability: 'audio',
      route_key: 'audio',
      usage_model: 'gpt-4o-mini-transcribe',
      provider_id: 'provider-openai',
      accounting_mode: 'platform_credit',
      operation_kind: 'audio.transcriptions.create',
      modality: 'audio',
      api_key_hash: 'key-enterprise',
      channel_id: 'openai',
      reference_id: 'aud_1',
      latency_ms: 960,
      units: 90,
      request_count: 2,
      input_tokens: 0,
      output_tokens: 0,
      total_tokens: 0,
      cache_read_tokens: 0,
      cache_write_tokens: 0,
      image_count: 0,
      audio_seconds: 92,
      video_seconds: 48,
      music_seconds: 25,
      upstream_cost: 2.0,
      customer_charge: 1.6,
      applied_routing_profile_id: 'profile-audio',
      compiled_routing_snapshot_id: 'snapshot-audio',
      fallback_reason: 'latency_guardrail',
      created_at_ms: 300,
    },
  ]);

  assert.deepEqual(document.headers, [
    'event_id',
    'tenant_id',
    'project_id',
    'api_key_group_id',
    'capability',
    'route_key',
    'usage_model',
    'provider_id',
    'accounting_mode',
    'operation_kind',
    'modality',
    'api_key_hash',
    'channel_id',
    'reference_id',
    'latency_ms',
    'units',
    'request_count',
    'input_tokens',
    'output_tokens',
    'total_tokens',
    'cache_read_tokens',
    'cache_write_tokens',
    'image_count',
    'audio_seconds',
    'video_seconds',
    'music_seconds',
    'upstream_cost',
    'customer_charge',
    'applied_routing_profile_id',
    'compiled_routing_snapshot_id',
    'fallback_reason',
    'created_at',
  ]);
  assert.deepEqual(document.rows, [
    [
      'evt_export_1',
      'tenant-demo',
      'project-demo',
      'group-enterprise',
      'audio',
      'audio',
      'gpt-4o-mini-transcribe',
      'provider-openai',
      'platform_credit',
      'audio.transcriptions.create',
      'audio',
      'key-enterprise',
      'openai',
      'aud_1',
      960,
      90,
      2,
      0,
      0,
      0,
      0,
      0,
      0,
      92,
      48,
      25,
      '2.0000',
      '1.6000',
      'profile-audio',
      'snapshot-audio',
      'latency_guardrail',
      new Date(300).toISOString(),
    ],
  ]);
});
