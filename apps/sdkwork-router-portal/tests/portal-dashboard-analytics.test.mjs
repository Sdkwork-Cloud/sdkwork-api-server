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

test('dashboard exposes a claw-style analytics workbench instead of the earlier split operational board', () => {
  const dashboardPage = read('packages/sdkwork-router-portal-dashboard/src/pages/index.tsx');
  const dashboardComponents = read('packages/sdkwork-router-portal-dashboard/src/components/index.tsx');
  const dashboardRepository = read('packages/sdkwork-router-portal-dashboard/src/repository/index.ts');
  const dashboardServices = read('packages/sdkwork-router-portal-dashboard/src/services/index.ts');
  const dualColumnSectionCount = (
    dashboardPage.match(/xl:grid-cols-\[1\.35fr_0\.95fr\]/g) ?? []
  ).length;

  assert.match(dashboardComponents, /DashboardSummaryCard/);
  assert.match(dashboardComponents, /StatusBadge/);
  assert.doesNotMatch(dashboardComponents, /DashboardStatusPill/);
  assert.match(dashboardComponents, /DashboardRevenueTrendChart/);
  assert.match(dashboardComponents, /DashboardTokenTrendChart/);
  assert.match(dashboardComponents, /DashboardDistributionRingChart/);
  assert.match(dashboardComponents, /DashboardModelDistributionChart/);
  assert.doesNotMatch(dashboardPage, /SectionHeader/);
  assert.match(dashboardPage, /WorkspacePanel/);
  assert.match(dashboardPage, /ManagementWorkbench/);
  assert.match(dashboardPage, /StatusBadge/);
  assert.doesNotMatch(dashboardPage, /DashboardStatusPill/);
  assert.match(dashboardPage, /DashboardBalanceCard/);
  assert.match(dashboardPage, /DashboardMetricCard/);
  assert.match(dashboardPage, /Balance/);
  assert.match(dashboardPage, /Revenue/);
  assert.match(dashboardPage, /Total requests/);
  assert.match(dashboardPage, /Average booked spend/);
  assert.match(dashboardPage, /Today/);
  assert.match(dashboardPage, /7 days/);
  assert.match(dashboardPage, /This month/);
  assert.doesNotMatch(dashboardPage, /Portal overview/);
  assert.doesNotMatch(dashboardPage, /Workspace command center/);
  assert.doesNotMatch(dashboardPage, /title=\{t\('Traffic posture'\)\}/);
  assert.doesNotMatch(dashboardPage, /title=\{t\('Cost and quota'\)\}/);
  assert.doesNotMatch(dashboardPage, /title=\{t\('Workspace readiness'\)\}/);
  assert.match(dashboardPage, /Traffic trend/);
  assert.match(dashboardPage, /Spend trend/);
  assert.match(dashboardPage, /Provider distribution/);
  assert.match(dashboardPage, /Model distribution/);
  assert.match(dashboardPage, /Analytics workbench/);
  assert.match(dashboardPage, /Routing evidence/);
  assert.match(dashboardPage, /Module posture/);
  assert.match(dashboardPage, /Next actions/);
  assert.doesNotMatch(dashboardPage, /const surfaceClass =/);
  assert.ok(
    dualColumnSectionCount >= 2,
    'dashboard should use the same repeated two-column analytics rows as claw-studio',
  );
  assert.match(dashboardPage, /data-slot="portal-dashboard-workbench-tabs"/);
  assert.doesNotMatch(dashboardPage, /portalx-dashboard-grid/);
  assert.doesNotMatch(dashboardPage, /portalx-dashboard-main/);
  assert.doesNotMatch(dashboardPage, /portalx-dashboard-side/);
  assert.doesNotMatch(dashboardPage, /ResponsiveContainer/);
  assert.doesNotMatch(dashboardPage, /AreaChart/);
  assert.doesNotMatch(dashboardPage, /BarChart/);
  assert.doesNotMatch(dashboardPage, /PieChart/);
  assert.match(dashboardRepository, /listPortalUsageRecords/);
  assert.match(dashboardServices, /request_volume_series/);
  assert.match(dashboardServices, /spend_series/);
});

test('dashboard view model tolerates missing usage collections from live payloads', () => {
  const { buildPortalDashboardViewModel } = loadDashboardServices();
  const now = Date.now();

  const viewModel = buildPortalDashboardViewModel({
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
      total_requests: 0,
      project_count: 0,
      model_count: 0,
      provider_count: 0,
    },
    billing_summary: {
      project_id: 'project-demo',
      entry_count: 0,
      used_units: 0,
      booked_amount: 0,
      quota_policy_id: null,
      quota_limit_units: null,
      remaining_units: 20000,
      exhausted: false,
    },
    api_key_count: 0,
  });

  assert.deepEqual(viewModel.snapshot.recent_requests, []);
  assert.deepEqual(viewModel.snapshot.usage_summary.projects, []);
  assert.deepEqual(viewModel.snapshot.usage_summary.providers, []);
  assert.deepEqual(viewModel.snapshot.usage_summary.models, []);
  assert.deepEqual(viewModel.provider_mix, []);
  assert.deepEqual(viewModel.model_mix, []);
  assert.deepEqual(viewModel.activity_feed, []);
  assert.deepEqual(viewModel.traffic_trend_points, []);
  assert.deepEqual(viewModel.spend_trend_points, []);
});
