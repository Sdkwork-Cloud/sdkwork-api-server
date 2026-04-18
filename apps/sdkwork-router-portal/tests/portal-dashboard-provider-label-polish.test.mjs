import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

import jiti from 'jiti';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

function loadDashboardServices() {
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
    },
  });
  return load(
    path.join(
      appRoot,
      'packages',
      'sdkwork-router-portal-dashboard',
      'src',
      'services',
      'index.ts',
    ),
  );
}

test('dashboard productizes provider identifiers before they reach routing posture and activity copy', () => {
  const dashboardServices = read('packages/sdkwork-router-portal-dashboard/src/services/index.ts');
  const { buildPortalDashboardViewModel } = loadDashboardServices();
  const now = Date.now();

  assert.match(dashboardServices, /selected_provider_id/);

  const viewModel = buildPortalDashboardViewModel(
    {
      workspace: {
        user: {
          id: 'portal-user',
          email: 'portal@example.com',
          display_name: 'Portal User',
          workspace_tenant_id: 'tenant-demo',
          workspace_project_id: 'project-demo',
          active: true,
          created_at_ms: now,
        },
        tenant: {
          id: 'tenant-demo',
          name: 'Tenant Demo',
        },
        project: {
          tenant_id: 'tenant-demo',
          id: 'project-demo',
          name: 'Project Demo',
        },
      },
      usage_summary: {
        total_requests: 18,
        project_count: 1,
        model_count: 2,
        provider_count: 2,
        projects: [{ project_id: 'project-demo', request_count: 18 }],
        providers: [
          { provider: 'openai-primary-eastus', request_count: 12, project_count: 1 },
          { provider: 'anthropic-fallback-router', request_count: 6, project_count: 1 },
        ],
        models: [
          { model: 'gpt-4o-mini', request_count: 12, provider_count: 1 },
          { model: 'claude-3-5-sonnet', request_count: 6, provider_count: 1 },
        ],
      },
      billing_summary: {
        project_id: 'project-demo',
        entry_count: 2,
        used_units: 1800,
        booked_amount: 32.4,
        quota_policy_id: 'quota-enterprise',
        quota_limit_units: 100000,
        remaining_units: 98200,
        exhausted: false,
      },
      recent_requests: [
        {
          project_id: 'project-demo',
          model: 'gpt-4o-mini',
          provider: 'openai-primary-eastus',
          units: 1200,
          amount: 21.6,
          api_key_hash: 'gwk_demo',
          channel_id: 'openai',
          input_tokens: 600,
          output_tokens: 300,
          total_tokens: 900,
          latency_ms: 420,
          reference_amount: 21.6,
          created_at_ms: now - 2_000,
        },
      ],
      api_key_count: 3,
    },
    {
      project_id: 'project-demo',
      latest_model_hint: 'gpt-4o-mini',
      preferences: {
        project_id: 'project-demo',
        preset_id: 'default',
        strategy: 'slo_aware',
        ordered_provider_ids: ['openai-primary-eastus', 'anthropic-fallback-router'],
        default_provider_id: 'openai-primary-eastus',
        max_cost: 0.1,
        max_latency_ms: 1200,
        require_healthy: true,
        preferred_region: 'eastus',
        updated_at_ms: now,
      },
      preview: {
        selected_provider_id: 'openai-primary-eastus',
        candidate_ids: ['openai-primary-eastus', 'anthropic-fallback-router'],
        matched_policy_id: 'policy-default',
        strategy: 'slo_aware',
        selection_seed: 7,
        selection_reason: 'Primary route stayed within the latency guardrail.',
        requested_region: 'eastus',
        slo_applied: true,
        slo_degraded: false,
        assessments: [],
      },
      provider_options: [],
    },
    [
      {
        decision_id: 'routing-demo-1',
        decision_source: 'live-request',
        tenant_id: 'tenant-demo',
        project_id: 'project-demo',
        capability: 'chat',
        route_key: 'chat.default',
        selected_provider_id: 'anthropic-fallback-router',
        matched_policy_id: 'policy-default',
        strategy: 'slo_aware',
        selection_seed: 11,
        selection_reason: 'Fallback provider kept the request inside the latency guardrail.',
        requested_region: 'eastus',
        slo_applied: true,
        slo_degraded: true,
        created_at_ms: now - 1_000,
        assessments: [],
      },
    ],
  );

  assert.equal(viewModel.routing_posture.selected_provider, 'OpenAI');
  assert.match(viewModel.routing_posture.detail, /Anthropic/);
  assert.doesNotMatch(viewModel.routing_posture.detail, /anthropic-fallback-router/);
  assert.equal(viewModel.provider_mix[0]?.label, 'OpenAI');
  assert.equal(viewModel.provider_mix[1]?.label, 'Anthropic');
  assert.equal(viewModel.provider_share_series[0]?.name, 'OpenAI');
  assert.equal(viewModel.provider_share_series[1]?.name, 'Anthropic');
  assert.match(viewModel.activity_feed[0]?.title ?? '', /Anthropic/);
  assert.match(viewModel.activity_feed[1]?.title ?? '', /OpenAI/);
  assert.doesNotMatch(viewModel.activity_feed[0]?.title ?? '', /anthropic-fallback-router/);
  assert.doesNotMatch(viewModel.activity_feed[1]?.title ?? '', /openai-primary-eastus/);
});
