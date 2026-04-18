import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

import jiti from 'jiti';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

function loadAccountServices() {
  const load = jiti(import.meta.url, { moduleCache: false });
  return load(
    path.join(
      appRoot,
      'packages',
      'sdkwork-router-portal-account',
      'src',
      'services',
      'index.ts',
    ),
  );
}

test('account workspace consumes billing event summary across repository, types, services, and page', () => {
  const repository = read('packages/sdkwork-router-portal-account/src/repository/index.ts');
  const accountTypes = read('packages/sdkwork-router-portal-account/src/types/index.ts');
  const accountServices = read('packages/sdkwork-router-portal-account/src/services/index.ts');
  const accountPage = read('packages/sdkwork-router-portal-account/src/pages/index.tsx');

  assert.match(repository, /getPortalBillingEventSummary/);

  assert.match(accountTypes, /BillingEventSummary/);
  assert.match(accountTypes, /billingEventSummary: BillingEventSummary;/);
  assert.match(accountTypes, /financial_breakdown:/);

  assert.match(accountServices, /financial_breakdown/);

  assert.match(accountPage, /Financial breakdown/);
  assert.match(accountPage, /Capability mix/);
  assert.match(accountPage, /Accounting mode mix/);
  assert.match(accountPage, /Multimodal usage/);
  assert.match(accountPage, /portal-account-financial-breakdown/);
});

test('account services derive capability, accounting mode, and multimodal financial breakdowns', () => {
  const { buildPortalAccountViewModel } = loadAccountServices();
  const now = new Date('2026-04-03T10:00:00.000Z').getTime();

  const viewModel = buildPortalAccountViewModel({
    summary: {
      project_id: 'project-demo',
      entry_count: 4,
      used_units: 9000,
      booked_amount: 240.5,
      quota_policy_id: 'quota-enterprise',
      quota_limit_units: 12000,
      remaining_units: 3000,
      exhausted: false,
    },
    membership: null,
    usageSummary: {
      total_requests: 120,
      project_count: 1,
      model_count: 3,
      provider_count: 2,
      projects: [{ project_id: 'project-demo', request_count: 120 }],
      providers: [],
      models: [],
    },
    usageRecords: [
      {
        project_id: 'project-demo',
        model: 'gpt-4.1',
        provider: 'provider-openai-official',
        units: 300,
        amount: 12,
        api_key_hash: 'key-live',
        channel_id: 'openai',
        input_tokens: 200,
        output_tokens: 100,
        total_tokens: 300,
        latency_ms: 400,
        reference_amount: 14,
        created_at_ms: now - 2 * 60 * 60 * 1000,
      },
    ],
    ledger: [
      { project_id: 'project-demo', units: 8000, amount: 180 },
      { project_id: 'project-linked-a', units: 650, amount: 35 },
    ],
    billingEventSummary: {
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
      total_upstream_cost: 9.1,
      total_customer_charge: 12.4,
      projects: [],
      groups: [],
      capabilities: [
        {
          capability: 'audio',
          event_count: 1,
          request_count: 2,
          total_tokens: 0,
          image_count: 0,
          audio_seconds: 92,
          video_seconds: 48,
          music_seconds: 25,
          total_upstream_cost: 1.8,
          total_customer_charge: 1.4,
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
          total_upstream_cost: 3.1,
          total_customer_charge: 4.1,
        },
        {
          capability: 'images',
          event_count: 1,
          request_count: 1,
          total_tokens: 0,
          image_count: 6,
          audio_seconds: 0,
          video_seconds: 0,
          music_seconds: 0,
          total_upstream_cost: 4.2,
          total_customer_charge: 6.9,
        },
      ],
      accounting_modes: [
        {
          accounting_mode: 'passthrough',
          event_count: 1,
          request_count: 2,
          total_upstream_cost: 1.8,
          total_customer_charge: 1.4,
        },
        {
          accounting_mode: 'platform_credit',
          event_count: 2,
          request_count: 4,
          total_upstream_cost: 3.1,
          total_customer_charge: 4.1,
        },
        {
          accounting_mode: 'byok',
          event_count: 1,
          request_count: 1,
          total_upstream_cost: 4.2,
          total_customer_charge: 6.9,
        },
      ],
    },
    searchQuery: '',
    historyView: 'all',
    page: 1,
    pageSize: 6,
    now,
  });

  assert.equal(viewModel.financial_breakdown.total_events, 4);
  assert.equal(viewModel.financial_breakdown.total_customer_charge, 12.4);
  assert.deepEqual(
    viewModel.financial_breakdown.top_capabilities.map((item) => item.capability),
    ['images', 'responses', 'audio'],
  );
  assert.deepEqual(
    viewModel.financial_breakdown.accounting_mode_mix.map((item) => item.accounting_mode),
    ['byok', 'platform_credit', 'passthrough'],
  );
  assert.deepEqual(viewModel.financial_breakdown.multimodal_totals, {
    image_count: 6,
    audio_seconds: 92,
    video_seconds: 48,
    music_seconds: 25,
  });
});
