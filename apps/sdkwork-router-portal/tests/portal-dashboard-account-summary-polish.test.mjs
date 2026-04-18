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

test('dashboard promotes account-style balance and financial metric cards with time breakdowns', () => {
  const dashboardPage = read('packages/sdkwork-router-portal-dashboard/src/pages/index.tsx');
  const dashboardComponents = read('packages/sdkwork-router-portal-dashboard/src/components/index.tsx');

  assert.match(dashboardComponents, /DashboardBalanceCard/);
  assert.match(dashboardComponents, /DashboardMetricCard/);
  assert.match(dashboardPage, /Balance/);
  assert.match(dashboardPage, /Revenue/);
  assert.match(dashboardPage, /Total requests/);
  assert.match(dashboardPage, /Average booked spend/);
  assert.match(dashboardPage, /Today/);
  assert.match(dashboardPage, /7 days/);
  assert.match(dashboardPage, /This month/);
  assert.doesNotMatch(dashboardPage, /title=\{t\('Traffic posture'\)\}/);
  assert.doesNotMatch(dashboardPage, /title=\{t\('Cost and quota'\)\}/);
  assert.doesNotMatch(dashboardPage, /title=\{t\('Workspace readiness'\)\}/);
});

test('dashboard view model computes balance and today / 7 day / month summaries from live usage records', () => {
  const { buildPortalDashboardViewModel } = loadDashboardServices();
  const now = Date.UTC(2026, 3, 3, 12, 0, 0);
  const oneDay = 86_400_000;

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
          created_at_ms: now - oneDay,
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
        total_requests: 4,
        project_count: 1,
        model_count: 2,
        provider_count: 2,
        projects: [{ project_id: 'project-demo', request_count: 4 }],
        providers: [
          { provider: 'openai-primary-eastus', request_count: 3, project_count: 1 },
          { provider: 'anthropic-fallback-router', request_count: 1, project_count: 1 },
        ],
        models: [
          { model: 'gpt-4o-mini', request_count: 3, provider_count: 1 },
          { model: 'claude-3-5-sonnet', request_count: 1, provider_count: 1 },
        ],
      },
      billing_summary: {
        project_id: 'project-demo',
        entry_count: 4,
        used_units: 2000,
        booked_amount: 42,
        quota_policy_id: 'quota-enterprise',
        quota_limit_units: 8000,
        remaining_units: 6000,
        exhausted: false,
      },
      recent_requests: [],
      api_key_count: 2,
    },
    undefined,
    [],
    [
      {
        project_id: 'project-demo',
        model: 'gpt-4o-mini',
        provider: 'openai-primary-eastus',
        units: 500,
        amount: 10,
        api_key_hash: 'k1',
        channel_id: 'openai',
        input_tokens: 250,
        output_tokens: 125,
        total_tokens: 375,
        latency_ms: 400,
        reference_amount: 10,
        created_at_ms: now - 2 * 60 * 60 * 1000,
      },
      {
        project_id: 'project-demo',
        model: 'gpt-4o-mini',
        provider: 'openai-primary-eastus',
        units: 700,
        amount: 14,
        api_key_hash: 'k2',
        channel_id: 'openai',
        input_tokens: 300,
        output_tokens: 180,
        total_tokens: 480,
        latency_ms: 380,
        reference_amount: 14,
        created_at_ms: now - 3 * oneDay,
      },
      {
        project_id: 'project-demo',
        model: 'claude-3-5-sonnet',
        provider: 'anthropic-fallback-router',
        units: 300,
        amount: 6,
        api_key_hash: 'k3',
        channel_id: 'anthropic',
        input_tokens: 120,
        output_tokens: 90,
        total_tokens: 210,
        latency_ms: 510,
        reference_amount: 6,
        created_at_ms: now - 6 * oneDay,
      },
      {
        project_id: 'project-demo',
        model: 'gpt-4o-mini',
        provider: 'openai-primary-eastus',
        units: 500,
        amount: 12,
        api_key_hash: 'k4',
        channel_id: 'openai',
        input_tokens: 260,
        output_tokens: 140,
        total_tokens: 400,
        latency_ms: 390,
        reference_amount: 12,
        created_at_ms: now - 15 * oneDay,
      },
    ],
    null,
    now,
  );

  assert.equal(viewModel.balance.remaining_units, 6000);
  assert.equal(viewModel.balance.quota_limit_units, 8000);
  assert.equal(viewModel.balance.used_units, 2000);
  assert.equal(viewModel.totals.revenue, 42);
  assert.equal(viewModel.totals.request_count, 4);
  assert.equal(viewModel.today.revenue, 10);
  assert.equal(viewModel.today.request_count, 1);
  assert.equal(viewModel.trailing_7d.revenue, 30);
  assert.equal(viewModel.trailing_7d.request_count, 3);
  assert.equal(viewModel.current_month.revenue, 10);
  assert.equal(viewModel.current_month.request_count, 1);
});
