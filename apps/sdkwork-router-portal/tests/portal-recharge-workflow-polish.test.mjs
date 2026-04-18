import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

import jiti from 'jiti';

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
  const presentation = read('packages/sdkwork-router-portal-recharge/src/pages/presentation.ts');
  const pageContract = `${page}\n${presentation}`;

  assert.match(page, /PortalRechargePage\(\{ onNavigate \}: PortalRechargePageProps\)/);
  assert.doesNotMatch(page, /onNavigate:\s*_onNavigate/);
  assert.match(page, /data-slot="portal-recharge-selection-hero"/);
  assert.match(page, /data-slot="portal-recharge-posture-strip"/);
  assert.match(page, /data-slot="portal-recharge-guidance-band"/);
  assert.match(page, /data-slot="portal-recharge-flow-tracker"/);
  assert.match(page, /data-slot="portal-recharge-selection-story"/);
  assert.match(page, /data-slot="portal-recharge-quote-breakdown"/);
  assert.match(page, /data-slot="portal-recharge-next-step-callout"/);
  assert.match(page, /data-slot="portal-recharge-post-order-handoff"/);
  assert.match(page, /data-slot="portal-recharge-mobile-cta"/);
  assert.match(pageContract, /Payment information/);
  assert.match(pageContract, /Funding flow/);
  assert.match(pageContract, /Choose amount/);
  assert.match(pageContract, /Create order/);
  assert.match(pageContract, /Complete payment in billing/);
  assert.match(pageContract, /Checkout summary/);
  assert.match(pageContract, /Selection story/);
  assert.match(pageContract, /Recommended/);
  assert.match(pageContract, /Best fit for steady usage/);
  assert.match(pageContract, /Pending settlement queue/);
  assert.match(pageContract, /Latest pending order/);
  assert.match(pageContract, /Open billing to complete payment/);
  assert.match(pageContract, /Order ready for payment/);
  assert.match(pageContract, /Continue in billing/);
  assert.match(pageContract, /Create another order/);
  assert.match(page, /buildPortalRechargePrimaryActionState/);
  assert.match(page, /buildPortalRechargeMobileActionState/);
  assert.match(page, /buildPortalRechargeFlowTrackerState/);
  assert.match(page, /resolvePortalRechargePostOrderHandoffActive/);
  assert.match(page, /Pending payment queue/);
  assert.match(page, /Open billing workbench/);
  assert.match(page, /onClick=\{\(\) => onNavigate\('billing'\)\}/);
  assert.match(pageContract, /Current balance/);
  assert.match(pageContract, /Pending follow-up/);
  assert.match(pageContract, /Recommended next top-up/);
  assert.match(page, /data-slot="portal-recharge-quote-note"/);
  assert.match(pageContract, /Create order in billing/);
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

test('recharge services derive purchase merchandising guidance from recharge option posture', () => {
  const { buildPortalRechargeOptionMerchandising } = loadRechargeServices();

  const options = [
    {
      id: 'starter',
      label: 'Starter',
      amount_cents: 1500,
      amount_label: '$15',
      granted_units: 150000,
      effective_ratio_label: '10k units/USD',
      note: 'Fast top-up',
      recommended: false,
      source: 'live',
    },
    {
      id: 'growth',
      label: 'Growth',
      amount_cents: 3000,
      amount_label: '$30',
      granted_units: 330000,
      effective_ratio_label: '11k units/USD',
      note: 'Balanced option',
      recommended: true,
      source: 'live',
    },
    {
      id: 'reserve',
      label: 'Reserve',
      amount_cents: 9000,
      amount_label: '$90',
      granted_units: 1080000,
      effective_ratio_label: '12k units/USD',
      note: 'Bulk reserve',
      recommended: false,
      source: 'live',
    },
  ];
  const t = (text) => text;

  assert.deepEqual(
    buildPortalRechargeOptionMerchandising({ option: options[0], options, t }),
    {
      badge: 'Quick coverage',
      intentLabel: 'Best for immediate runway',
      supportLabel: 'Keep service continuity covered without overfunding the workspace.',
    },
  );
  assert.deepEqual(
    buildPortalRechargeOptionMerchandising({ option: options[1], options, t }),
    {
      badge: 'Recommended default',
      intentLabel: 'Best fit for steady usage',
      supportLabel: 'The safest default when you want clean value and low decision friction.',
    },
  );
  assert.deepEqual(
    buildPortalRechargeOptionMerchandising({ option: options[2], options, t }),
    {
      badge: 'Reserve build',
      intentLabel: 'Built for scale planning',
      supportLabel: 'Use the larger reserve when you want longer runway and fewer manual top-ups.',
    },
  );
});

test('recharge services spotlight the newest pending payment order for settlement follow-up', () => {
  const { buildPortalRechargePendingPaymentSpotlight } = loadRechargeServices();
  const t = (text, values) => {
    if (!values) {
      return text;
    }

    return Object.entries(values).reduce(
      (current, [key, value]) => current.replace(`{${key}}`, String(value)),
      text,
    );
  };
  const orders = [
    {
      order_id: 'paid',
      project_id: 'project-1',
      user_id: 'user-1',
      target_kind: 'custom_recharge',
      target_id: 'custom',
      target_name: 'Recharge',
      list_price_cents: 3000,
      payable_price_cents: 3000,
      list_price_label: '$30.00',
      payable_price_label: '$30.00',
      granted_units: 330000,
      bonus_units: 0,
      status: 'fulfilled',
      source: 'live',
      created_at_ms: 100,
    },
    {
      order_id: 'pending-older',
      project_id: 'project-1',
      user_id: 'user-1',
      target_kind: 'custom_recharge',
      target_id: 'custom',
      target_name: 'Recharge',
      list_price_cents: 4000,
      payable_price_cents: 4000,
      list_price_label: '$40.00',
      payable_price_label: '$40.00',
      granted_units: 440000,
      bonus_units: 0,
      status: 'pending_payment',
      source: 'live',
      created_at_ms: 200,
    },
    {
      order_id: 'pending-latest',
      project_id: 'project-1',
      user_id: 'user-1',
      target_kind: 'recharge_pack',
      target_id: 'pack',
      target_name: 'Recharge pack',
      list_price_cents: 9000,
      payable_price_cents: 9000,
      list_price_label: '$90.00',
      payable_price_label: '$90.00',
      granted_units: 1080000,
      bonus_units: 0,
      status: 'pending_payment',
      source: 'live',
      created_at_ms: 300,
    },
  ];

  assert.deepEqual(
    buildPortalRechargePendingPaymentSpotlight({ orders, t }),
    {
      headline: 'Pending settlement queue',
      detail: '2 orders waiting for payment completion in billing.',
      latestOrderLabel: 'Latest pending order',
      ctaLabel: 'Open billing to complete payment',
      count: 2,
      latestOrder: orders[2],
    },
  );
  assert.equal(
    buildPortalRechargePendingPaymentSpotlight({
      orders: orders.filter((order) => order.status !== 'pending_payment'),
      t,
    }),
    null,
  );
});
