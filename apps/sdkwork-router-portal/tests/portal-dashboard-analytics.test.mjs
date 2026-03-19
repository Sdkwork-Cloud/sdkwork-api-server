import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
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
  assert.match(dashboardComponents, /DashboardSectionHeader/);
  assert.match(dashboardComponents, /DashboardStatusPill/);
  assert.match(dashboardComponents, /DashboardRevenueTrendChart/);
  assert.match(dashboardComponents, /DashboardTokenTrendChart/);
  assert.match(dashboardComponents, /DashboardDistributionRingChart/);
  assert.match(dashboardComponents, /DashboardModelDistributionChart/);
  assert.match(dashboardPage, /Traffic posture/);
  assert.match(dashboardPage, /Cost and quota/);
  assert.match(dashboardPage, /Workspace readiness/);
  assert.match(dashboardPage, /Traffic trend/);
  assert.match(dashboardPage, /Spend trend/);
  assert.match(dashboardPage, /Provider distribution/);
  assert.match(dashboardPage, /Model distribution/);
  assert.match(dashboardPage, /Analytics workbench/);
  assert.match(dashboardPage, /Routing evidence/);
  assert.match(dashboardPage, /Module posture/);
  assert.match(dashboardPage, /Next actions/);
  assert.match(dashboardPage, /const surfaceClass =/);
  assert.ok(
    dualColumnSectionCount >= 2,
    'dashboard should use the same repeated two-column analytics rows as claw-studio',
  );
  assert.match(dashboardPage, /data-slot="portal-dashboard-workbench-tabs"/);
  assert.match(
    dashboardPage,
    /chartFrameClass|overflow-hidden rounded-\[1\.5rem\] border border-\[color:var\(--portal-chart-grid\)\]/,
  );
  assert.doesNotMatch(dashboardPage, /portalx-dashboard-grid/);
  assert.doesNotMatch(dashboardPage, /portalx-dashboard-main/);
  assert.doesNotMatch(dashboardPage, /portalx-dashboard-side/);
  assert.doesNotMatch(dashboardPage, /ResponsiveContainer/);
  assert.doesNotMatch(dashboardPage, /AreaChart/);
  assert.doesNotMatch(dashboardPage, /BarChart/);
  assert.doesNotMatch(dashboardPage, /PieChart/);
  assert.doesNotMatch(dashboardPage, /Surface/);
  assert.match(dashboardRepository, /listPortalUsageRecords/);
  assert.match(dashboardServices, /request_volume_series/);
  assert.match(dashboardServices, /spend_series/);
});
