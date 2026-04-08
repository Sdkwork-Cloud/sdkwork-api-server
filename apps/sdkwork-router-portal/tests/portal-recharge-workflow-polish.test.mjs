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
  const load = jiti(import.meta.url, {
    moduleCache: false,
    alias: {
      'sdkwork-router-portal-commons': path.join(
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
      'sdkwork-router-portal-recharge',
      'src',
      'services',
      'index.ts',
    ),
  );
}

test('recharge page keeps pending payment queue visible and routes users back to the billing workbench', () => {
  const page = read('packages/sdkwork-router-portal-recharge/src/pages/index.tsx');

  assert.match(page, /PortalRechargePage\(\{ onNavigate \}: PortalRechargePageProps\)/);
  assert.doesNotMatch(page, /onNavigate:\s*_onNavigate/);
  assert.match(page, /Payment information/);
  assert.match(page, /Recommended/);
  assert.match(page, /Pending payment queue/);
  assert.match(page, /Open billing workbench/);
  assert.match(page, /onClick=\{\(\) => onNavigate\('billing'\)\}/);
  assert.doesNotMatch(page, /Recharge decision support/);
});

test('recharge services validate custom recharge amounts against server-managed policy bounds and increments', () => {
  const { validatePortalRechargeAmount } = loadRechargeServices();

  const policy = {
    enabled: true,
    min_amount_cents: 1000,
    max_amount_cents: 10000,
    step_amount_cents: 500,
    suggested_amount_cents: 2500,
    rules: [],
    source: 'live',
  };

  assert.equal(validatePortalRechargeAmount(500, policy), 'below_minimum');
  assert.equal(validatePortalRechargeAmount(10550, policy), 'above_maximum');
  assert.equal(validatePortalRechargeAmount(1250, policy), 'step_mismatch');
  assert.equal(validatePortalRechargeAmount(2500, policy), null);
  assert.equal(
    validatePortalRechargeAmount(2500, { ...policy, enabled: false }),
    'disabled',
  );
  assert.equal(validatePortalRechargeAmount(2500, null), null);
});
