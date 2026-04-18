import assert from 'node:assert/strict';
import { existsSync, readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

import jiti from 'jiti';

const appRoot = path.resolve(import.meta.dirname, '..');
const overviewPagePath = path.join(
  appRoot,
  'packages',
  'sdkwork-router-admin-overview',
  'src',
  'index.tsx',
);
const overviewViewModelPath = path.join(
  appRoot,
  'packages',
  'sdkwork-router-admin-overview',
  'src',
  'view-model.ts',
);

function loadOverviewViewModel() {
  const load = jiti(import.meta.url, { moduleCache: false });
  return load(overviewViewModelPath);
}

test('admin overview page delegates snapshot shaping to a dedicated view model module', () => {
  const overviewPage = readFileSync(overviewPagePath, 'utf8');
  const overviewViewModelSource = existsSync(overviewViewModelPath)
    ? readFileSync(overviewViewModelPath, 'utf8')
    : '';

  assert.match(overviewPage, /buildAdminOverviewViewModel/);
  assert.match(overviewViewModelSource, /export function buildAdminOverviewViewModel/);
});

test('admin overview view model tolerates missing snapshot collections from live payloads', () => {
  if (!existsSync(overviewViewModelPath)) {
    assert.fail('admin overview view model module is missing');
  }

  const { buildAdminOverviewViewModel } = loadOverviewViewModel();

  const viewModel = buildAdminOverviewViewModel({
    usageSummary: {
      total_requests: 0,
      project_count: 0,
      model_count: 0,
      provider_count: 0,
    },
    billingSummary: {
      total_entries: 0,
      project_count: 0,
      total_units: 0,
      total_amount: 0,
      active_quota_policy_count: 0,
      exhausted_project_count: 0,
    },
  });

  assert.deepEqual(viewModel.snapshot.portalUsers, []);
  assert.deepEqual(viewModel.snapshot.projects, []);
  assert.deepEqual(viewModel.snapshot.usageRecords, []);
  assert.deepEqual(viewModel.snapshot.usageSummary.projects, []);
  assert.deepEqual(viewModel.snapshot.billingSummary.projects, []);
  assert.deepEqual(viewModel.snapshot.commerceOrders, []);
  assert.deepEqual(viewModel.snapshot.commercePaymentEvents, []);
  assert.deepEqual(viewModel.snapshot.commercialAccountLedger, []);
  assert.deepEqual(viewModel.metrics, []);
  assert.deepEqual(viewModel.alerts, []);
  assert.deepEqual(viewModel.rankedUsers, []);
  assert.deepEqual(viewModel.rankedProjects, []);
});
