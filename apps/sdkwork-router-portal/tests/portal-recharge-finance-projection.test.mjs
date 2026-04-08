import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

import jiti from '../node_modules/.pnpm/jiti@2.6.1/node_modules/jiti/lib/jiti.mjs';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

function loadRechargeServices() {
  const load = jiti(import.meta.url, { moduleCache: false });
  return load(
    path.join(
      appRoot,
      'packages',
      'sdkwork-router-portal-recharge',
      'src',
      'services',
      'index.ts',
    ),
  );
}

test('recharge workspace removes deleted finance-projection dependencies across repository, types, services, and page', () => {
  const repository = read('packages/sdkwork-router-portal-recharge/src/repository/index.ts');
  const pageTypes = read('packages/sdkwork-router-portal-recharge/src/types/index.ts');
  const services = read('packages/sdkwork-router-portal-recharge/src/services/index.ts');
  const page = read('packages/sdkwork-router-portal-recharge/src/pages/index.tsx');

  assert.doesNotMatch(repository, /getPortalCommerceMembership/);
  assert.doesNotMatch(repository, /getPortalBillingEventSummary/);
  assert.doesNotMatch(pageTypes, /membership: PortalCommerceMembership \| null;/);
  assert.doesNotMatch(pageTypes, /billing_event_summary: BillingEventSummary;/);
  assert.doesNotMatch(pageTypes, /PortalRechargeFinanceProjection/);
  assert.doesNotMatch(services, /buildPortalRechargeFinanceProjection/);
  assert.doesNotMatch(services, /buildPortalRechargeSummaryCards/);
  assert.match(services, /buildPortalRechargeQuoteSnapshot/);
  assert.match(page, /Payment information/);
  assert.doesNotMatch(page, /Recharge decision support/);
  assert.doesNotMatch(page, /Leading accounting mode/);
  assert.doesNotMatch(page, /Leading capability/);
  assert.doesNotMatch(page, /Multimodal demand/);
  assert.doesNotMatch(page, /portal-recharge-decision-support/);
  assert.doesNotMatch(page, /portal-recharge-multimodal-demand/);
});

test('recharge services build payment information snapshot from quote and current balance', () => {
  const { buildPortalRechargeQuoteSnapshot } = loadRechargeServices();

  const snapshot = buildPortalRechargeQuoteSnapshot({
    customRechargePolicy: null,
    quote: {
      amount_cents: 4900,
      list_price_cents: 4900,
      projected_remaining_units: 16400,
      granted_units: 12000,
      bonus_units: 400,
      effective_ratio_label: '248 units / USD',
      pricing_rule_label: 'Growth top-up',
    },
    summary: {
      remaining_units: 4000,
    },
    t: (text) => text,
  });

  assert.deepEqual(snapshot, {
    amountLabel: '$49.00',
    projectedBalanceLabel: '16,400',
    grantedUnitsLabel: '12,400',
    effectiveRatioLabel: '248 units / USD',
    pricingRuleLabel: 'Growth top-up',
  });
});

test('recharge data path tolerates missing array payloads so slice-based UI rendering does not crash', () => {
  const repository = read('packages/sdkwork-router-portal-recharge/src/repository/index.ts');
  const page = read('packages/sdkwork-router-portal-recharge/src/pages/index.tsx');
  const { buildPortalRechargeHistoryRows } = loadRechargeServices();

  assert.deepEqual(buildPortalRechargeHistoryRows(undefined), []);
  assert.match(
    repository,
    /rechargeOptions:\s*Array\.isArray\(catalog\.recharge_options\)\s*\?\s*catalog\.recharge_options\s*:\s*\[\]/,
  );
  assert.match(
    repository,
    /orders:\s*Array\.isArray\(orders\)\s*\?\s*orders\s*:\s*\[\]/,
  );
  assert.match(page, /\(options\s*\?\?\s*\[\]\)\s*\.slice\(\)/);
});
