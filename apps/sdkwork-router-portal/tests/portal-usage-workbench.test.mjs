import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

import jiti from 'jiti';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

function loadUsageServices() {
  const load = jiti(import.meta.url, { moduleCache: false });
  return load(
    path.join(
      appRoot,
      'packages',
      'sdkwork-router-portal-usage',
      'src',
      'services',
      'index.ts',
    ),
  );
}

test('portal usage contracts and page copy expose the billing-grade usage workbench surface', () => {
  const types = read('packages/sdkwork-router-portal-types/src/index.ts');
  const usagePage = read('packages/sdkwork-router-portal-usage/src/pages/index.tsx');

  assert.match(types, /api_key_hash\?: string \| null;/);
  assert.match(types, /channel_id\?: string \| null;/);
  assert.match(types, /latency_ms\?: number \| null;/);
  assert.match(types, /reference_amount\?: number \| null;/);

  assert.match(usagePage, /Total requests/);
  assert.match(usagePage, /Total tokens/);
  assert.match(usagePage, /Total spend/);
  assert.match(usagePage, /Average latency/);
  assert.match(usagePage, /API key/);
  assert.match(usagePage, /Channel/);
  assert.match(usagePage, /Model/);
  assert.match(usagePage, /Previous page/);
  assert.match(usagePage, /Next page/);
});

test('portal usage services aggregate filtered spend, tokens, latency, and pagination for the table workbench', () => {
  const { buildPortalUsageViewModel } = loadUsageServices();
  const now = Date.now();

  const apiKeys = [
    {
      tenant_id: 'tenant-demo',
      project_id: 'project-demo',
      environment: 'live',
      hashed_key: 'key-live',
      label: 'Live browser key',
      notes: null,
      created_at_ms: now,
      last_used_at_ms: now,
      expires_at_ms: null,
      active: true,
    },
    {
      tenant_id: 'tenant-demo',
      project_id: 'project-demo',
      environment: 'server',
      hashed_key: 'key-server',
      label: 'Server workload key',
      notes: null,
      created_at_ms: now,
      last_used_at_ms: now,
      expires_at_ms: null,
      active: true,
    },
  ];

  const records = [
    {
      project_id: 'project-demo',
      model: 'gpt-4.1',
      provider: 'provider-openai-official',
      units: 1500,
      amount: 1.25,
      reference_amount: 1.5,
      api_key_hash: 'key-live',
      channel_id: 'openai',
      input_tokens: 1000,
      output_tokens: 500,
      total_tokens: 1500,
      latency_ms: 800,
      created_at_ms: now - 2 * 60 * 60 * 1000,
    },
    {
      project_id: 'project-demo',
      model: 'gpt-4.1-mini',
      provider: 'provider-openai-official',
      units: 500,
      amount: 0.5,
      reference_amount: 0.75,
      api_key_hash: 'key-live',
      channel_id: 'openai',
      input_tokens: 400,
      output_tokens: 100,
      total_tokens: 500,
      latency_ms: 400,
      created_at_ms: now - 60 * 60 * 1000,
    },
    {
      project_id: 'project-demo',
      model: 'claude-3-7-sonnet',
      provider: 'provider-anthropic',
      units: 1000,
      amount: 2,
      reference_amount: 2.4,
      api_key_hash: 'key-server',
      channel_id: 'anthropic',
      input_tokens: 300,
      output_tokens: 700,
      total_tokens: 1000,
      latency_ms: 1200,
      created_at_ms: now - 30 * 60 * 1000,
    },
  ];

  const viewModel = buildPortalUsageViewModel({
    records,
    apiKeys,
    filters: {
      api_key_hash: 'key-live',
      channel_id: 'openai',
      model: '',
      time_range: 'all',
    },
    page: 1,
    page_size: 1,
  });

  assert.equal(viewModel.summary.total_requests, 2);
  assert.equal(viewModel.summary.total_tokens, 2000);
  assert.equal(viewModel.summary.input_tokens, 1400);
  assert.equal(viewModel.summary.output_tokens, 600);
  assert.equal(viewModel.summary.actual_amount, 1.75);
  assert.equal(viewModel.summary.reference_amount, 2.25);
  assert.equal(viewModel.summary.average_latency_ms, 600);

  assert.deepEqual(
    viewModel.filter_options.api_keys.map((option) => option.value),
    ['all', 'key-live', 'key-server'],
  );
  assert.deepEqual(viewModel.filter_options.channels, ['all', 'anthropic', 'openai']);
  assert.deepEqual(
    viewModel.filter_options.models,
    ['all', 'claude-3-7-sonnet', 'gpt-4.1', 'gpt-4.1-mini'],
  );

  assert.equal(viewModel.pagination.total_items, 2);
  assert.equal(viewModel.pagination.total_pages, 2);
  assert.equal(viewModel.rows.length, 1);
  assert.equal(viewModel.rows[0].api_key_label, 'Live browser key');
  assert.equal(viewModel.rows[0].model, 'gpt-4.1-mini');
});
