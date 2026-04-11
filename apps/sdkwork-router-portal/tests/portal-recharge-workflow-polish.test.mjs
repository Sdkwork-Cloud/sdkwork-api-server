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
  assert.match(page, /Preview amount/);
  assert.match(page, /Pending payment queue/);
  assert.match(page, /Open billing workbench/);
  assert.match(page, /onClick=\{\(\) => onNavigate\('billing'\)\}/);
  assert.doesNotMatch(page, /Current runway/);
  assert.doesNotMatch(page, /Balance updates after payment settles/);
  assert.doesNotMatch(page, /Secure billing handoff/);
  assert.doesNotMatch(page, /Order creation only/);
  assert.doesNotMatch(page, /Settle later in billing/);
  assert.doesNotMatch(page, /Recharge decision support/);
});

test('recharge page keeps helper copy and i18n surface compact after the simplification pass', () => {
  const page = read('packages/sdkwork-router-portal-recharge/src/pages/index.tsx');

  assert.doesNotMatch(page, /Select a preset package or enter a custom amount\./);
  assert.doesNotMatch(page, /Recent orders and payment status\./);
  assert.doesNotMatch(page, /Create a recharge order to start history\./);
  assert.doesNotMatch(page, /Select or preview an amount to continue\./);
  assert.doesNotMatch(page, /No amount selected/);
  assert.doesNotMatch(page, /Selected package/);
  assert.doesNotMatch(page, /t\('Selected'\)/);
});

test('recharge payment card exposes a compact checkout structure instead of text-heavy empty states', () => {
  const page = read('packages/sdkwork-router-portal-recharge/src/pages/index.tsx');

  assert.match(page, /data-slot="portal-recharge-quote-hero"/);
  assert.match(page, /data-slot="portal-recharge-quote-metrics"/);
  assert.match(page, /data-slot="portal-recharge-quote-skeleton"/);
  assert.match(page, /data-slot="portal-recharge-primary-cta"/);
});

test('recharge amount picker uses a grid-based option matrix and theme-driven palette', () => {
  const page = read('packages/sdkwork-router-portal-recharge/src/pages/index.tsx');

  assert.match(page, /data-slot="portal-recharge-options-grid"/);
  assert.match(page, /data-slot="portal-recharge-custom-tile"/);
  assert.match(page, /lg:grid-cols-3/);
  assert.doesNotMatch(page, /border-dashed/);
  assert.doesNotMatch(page, /from-white/);
  assert.doesNotMatch(page, /to-white/);
  assert.doesNotMatch(page, /sky-/);
  assert.match(page, /var\(--theme-primary-/);
  assert.match(page, /buildPortalRechargePickerOptions/);
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

test('recharge services build the fixed eight-amount commercial picker before the custom tile', () => {
  const { buildPortalRechargePickerOptions } = loadRechargeServices();

  const picker = buildPortalRechargePickerOptions(
    [
      {
        id: 'pack-25',
        label: '25',
        amount_cents: 2500,
        amount_label: '$25.00',
        granted_units: 6000,
        effective_ratio_label: '240 units / USD',
        note: '',
        recommended: false,
        source: 'live',
      },
      {
        id: 'pack-50',
        label: '50',
        amount_cents: 5000,
        amount_label: '$50.00',
        granted_units: 12500,
        effective_ratio_label: '250 units / USD',
        note: '',
        recommended: true,
        source: 'live',
      },
      {
        id: 'pack-100',
        label: '100',
        amount_cents: 10000,
        amount_label: '$100.00',
        granted_units: 26000,
        effective_ratio_label: '260 units / USD',
        note: '',
        recommended: false,
        source: 'live',
      },
    ],
    {
      enabled: true,
      min_amount_cents: 1000,
      max_amount_cents: 500000,
      step_amount_cents: 2500,
      suggested_amount_cents: 50000,
      rules: [
        {
          id: 'rule-1',
          label: 'Growth',
          min_amount_cents: 1000,
          max_amount_cents: 500000,
          units_per_cent: 3,
          effective_ratio_label: '300 units / USD',
          note: '',
        },
      ],
      source: 'live',
    },
  );

  assert.deepEqual(
    picker.map((option) => option.amount_cents),
    [1000, 5000, 10000, 20000, 50000, 100000, 200000, 500000],
  );
  assert.equal(new Set(picker.map((option) => option.amount_cents)).size, picker.length);
  assert.equal(picker.length, 8);
  assert.equal(picker[4]?.amount_cents, 50000);
  assert.equal(picker[4]?.recommended, true);
});
