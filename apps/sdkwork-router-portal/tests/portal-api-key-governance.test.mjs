import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

import jiti from 'jiti';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

function loadApiKeyServices() {
  const load = jiti(import.meta.url, { moduleCache: false });
  return load(
    path.join(
      appRoot,
      'packages',
      'sdkwork-router-portal-api-keys',
      'src',
      'services',
      'index.ts',
    ),
  );
}

test('api key workspace consumes billing event summary across repository, types, services, and page governance surface', () => {
  const repository = read('packages/sdkwork-router-portal-api-keys/src/repository/index.ts');
  const pageTypes = read('packages/sdkwork-router-portal-api-keys/src/types/index.ts');
  const services = read('packages/sdkwork-router-portal-api-keys/src/services/index.ts');
  const page = read('packages/sdkwork-router-portal-api-keys/src/pages/index.tsx');

  assert.match(repository, /getPortalBillingEventSummary/);
  assert.match(pageTypes, /billing_event_summary: BillingEventSummary;/);
  assert.match(pageTypes, /PortalApiKeyGovernanceViewModel/);
  assert.match(services, /buildPortalApiKeyGovernanceViewModel/);
  assert.match(page, /Group governance/);
  assert.match(page, /Leading chargeback group/);
  assert.match(page, /Grouping posture/);
  assert.match(page, /Default accounting mode/);
  assert.match(page, /portal-api-key-governance/);
});

test('api key services derive governance evidence from keys, groups, and billing chargeback summaries', () => {
  const { buildPortalApiKeyGovernanceViewModel } = loadApiKeyServices();

  const viewModel = buildPortalApiKeyGovernanceViewModel({
    apiKeys: [
      {
        tenant_id: 'tenant-1',
        project_id: 'project-1',
        environment: 'live',
        hashed_key: 'hashed-group-1',
        api_key_group_id: 'group-live',
        label: 'Primary live key',
        created_at_ms: 3,
        active: true,
      },
      {
        tenant_id: 'tenant-1',
        project_id: 'project-1',
        environment: 'live',
        hashed_key: 'hashed-group-2',
        api_key_group_id: 'group-live',
        label: 'Secondary live key',
        created_at_ms: 2,
        active: true,
      },
      {
        tenant_id: 'tenant-1',
        project_id: 'project-1',
        environment: 'staging',
        hashed_key: 'hashed-group-3',
        api_key_group_id: 'group-batch',
        label: 'Batch staging key',
        created_at_ms: 1,
        active: true,
      },
      {
        tenant_id: 'tenant-1',
        project_id: 'project-1',
        environment: 'live',
        hashed_key: 'hashed-none',
        api_key_group_id: null,
        label: 'Standalone key',
        created_at_ms: 0,
        active: true,
      },
    ],
    groups: [
      {
        group_id: 'group-live',
        tenant_id: 'tenant-1',
        project_id: 'project-1',
        environment: 'live',
        name: 'Live traffic',
        slug: 'live-traffic',
        description: 'Primary traffic',
        color: 'blue',
        default_capability_scope: 'responses',
        default_routing_profile_id: 'profile-premium',
        default_accounting_mode: 'byok',
        active: true,
        created_at_ms: 1,
        updated_at_ms: 2,
      },
      {
        group_id: 'group-batch',
        tenant_id: 'tenant-1',
        project_id: 'project-1',
        environment: 'staging',
        name: 'Batch jobs',
        slug: 'batch-jobs',
        description: 'Offline batch',
        color: 'amber',
        default_capability_scope: 'audio',
        default_routing_profile_id: null,
        default_accounting_mode: null,
        active: true,
        created_at_ms: 1,
        updated_at_ms: 2,
      },
    ],
    billingEventSummary: {
      total_events: 5,
      project_count: 1,
      group_count: 2,
      capability_count: 3,
      total_request_count: 14,
      total_units: 800,
      total_input_tokens: 300,
      total_output_tokens: 220,
      total_tokens: 520,
      total_image_count: 2,
      total_audio_seconds: 44,
      total_video_seconds: 0,
      total_music_seconds: 0,
      total_upstream_cost: 11.3,
      total_customer_charge: 16.7,
      projects: [],
      groups: [
        {
          api_key_group_id: 'group-live',
          project_count: 1,
          event_count: 3,
          request_count: 9,
          total_upstream_cost: 6.8,
          total_customer_charge: 10.9,
        },
        {
          api_key_group_id: null,
          project_count: 1,
          event_count: 2,
          request_count: 5,
          total_upstream_cost: 4.5,
          total_customer_charge: 5.8,
        },
      ],
      capabilities: [],
      accounting_modes: [],
    },
  });

  assert.equal(viewModel.summary.active_group_count, 2);
  assert.equal(viewModel.summary.grouped_key_count, 3);
  assert.equal(viewModel.summary.ungrouped_key_count, 1);
  assert.equal(viewModel.summary.routing_profile_bound_group_count, 1);
  assert.equal(viewModel.leading_chargeback_group?.api_key_group_id, 'group-live');
  assert.equal(viewModel.leading_chargeback_group?.group_name, 'Live traffic');
  assert.equal(viewModel.leading_chargeback_group?.default_accounting_mode, 'byok');
  assert.equal(viewModel.leading_chargeback_group?.default_routing_profile_id, 'profile-premium');
  assert.equal(viewModel.dominant_default_accounting_mode, 'byok');
});
