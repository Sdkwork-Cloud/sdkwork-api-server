import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('dashboard exposes chart-driven analytics instead of card-only summaries', () => {
  const dashboardPage = read('packages/sdkwork-router-portal-dashboard/src/pages/index.tsx');
  const dashboardRepository = read('packages/sdkwork-router-portal-dashboard/src/repository/index.ts');
  const dashboardServices = read('packages/sdkwork-router-portal-dashboard/src/services/index.ts');

  assert.match(dashboardPage, /Request volume/);
  assert.match(dashboardPage, /Booked spend trend/);
  assert.match(dashboardPage, /Provider share/);
  assert.match(dashboardPage, /Model demand/);
  assert.match(dashboardPage, /ResponsiveContainer/);
  assert.match(dashboardPage, /AreaChart/);
  assert.match(dashboardPage, /BarChart/);
  assert.match(dashboardPage, /PieChart/);
  assert.match(dashboardRepository, /listPortalUsageRecords/);
  assert.match(dashboardServices, /request_volume_series/);
  assert.match(dashboardServices, /spend_series/);
});
