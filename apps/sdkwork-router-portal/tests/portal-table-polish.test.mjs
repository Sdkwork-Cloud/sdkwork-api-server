import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('portal shared table exposes shadcn-style slots and keeps header visible for empty data', () => {
  const commons = read('packages/sdkwork-router-portal-commons/src/index.tsx');
  const apiKeyTable = read('packages/sdkwork-router-portal-api-keys/src/components/PortalApiKeyTable.tsx');
  const usagePage = read('packages/sdkwork-router-portal-usage/src/pages/index.tsx');
  const creditsPage = read('packages/sdkwork-router-portal-credits/src/pages/index.tsx');
  const accountPage = read('packages/sdkwork-router-portal-account/src/pages/index.tsx');
  const billingPage = read('packages/sdkwork-router-portal-billing/src/pages/index.tsx');
  const dashboardPage = read('packages/sdkwork-router-portal-dashboard/src/pages/index.tsx');
  const routingPage = read('packages/sdkwork-router-portal-routing/src/pages/index.tsx');
  const routingComponents = read('packages/sdkwork-router-portal-routing/src/components/index.tsx');
  const gatewayPage = read('packages/sdkwork-router-portal-gateway/src/pages/index.tsx');
  const gatewayComponents = read('packages/sdkwork-router-portal-gateway/src/components/index.tsx');
  const billingTableCount = billingPage.match(/<DataTable/g)?.length ?? 0;
  const dashboardTableCount = dashboardPage.match(/<DataTable/g)?.length ?? 0;
  const routingTableCount =
    (routingPage.match(/<DataTable/g)?.length ?? 0)
    + (routingComponents.match(/<DataTable/g)?.length ?? 0);
  const gatewayTableCount =
    (gatewayPage.match(/<DataTable/g)?.length ?? 0)
    + (gatewayComponents.match(/<DataTable/g)?.length ?? 0);

  assert.match(commons, /data-slot="table-container"/);
  assert.match(commons, /data-slot="table-header"/);
  assert.match(commons, /data-slot="table-empty"/);
  assert.match(apiKeyTable, /DataTable/);
  assert.doesNotMatch(apiKeyTable, /if \(!items.length\)/);
  assert.doesNotMatch(usagePage, /visibleRecords.length \?/);
  assert.doesNotMatch(creditsPage, /rows.length \?/);
  assert.doesNotMatch(accountPage, /visibleLedger.length \?/);
  assert.equal(billingTableCount, 1);
  assert.equal(dashboardTableCount, 3);
  assert.doesNotMatch(billingPage, /pendingOrders.length \?/);
  assert.doesNotMatch(billingPage, /timelineOrders.length \?/);
  assert.match(dashboardPage, /DataTable/);
  assert.match(dashboardPage, /Action detail/);
  assert.match(dashboardPage, /Request posture/);
  assert.doesNotMatch(dashboardPage, /<table className="w-full/);
  assert.equal(routingTableCount, 1);
  assert.match(routingPage, /data-slot="portal-routing-filter-bar"/);
  assert.match(routingPage, /Workbench lane/);
  assert.match(routingPage, /Operational focus/);
  assert.doesNotMatch(routingPage, /viewModel\.evidence.length \?/);
  assert.doesNotMatch(routingPage, /<Tabs/);
  assert.equal(gatewayTableCount, 1);
  assert.match(gatewayPage, /data-slot="portal-gateway-filter-bar"/);
  assert.match(gatewayPage, /Workbench lane/);
  assert.match(gatewayPage, /Operational focus/);
  assert.doesNotMatch(gatewayPage, /Compatibility matrix/);
  assert.doesNotMatch(gatewayPage, /Desktop runtime controls/);
});
