import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

import jiti from 'jiti';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

function loadCreditsServices() {
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
      'sdkwork-router-portal-commons/i18n-core': path.join(
        appRoot,
        'packages',
        'sdkwork-router-portal-commons',
        'src',
        'i18n-core.ts',
      ),
    },
  });

  return load(
    path.join(
      appRoot,
      'packages',
      'sdkwork-router-portal-credits',
      'src',
      'services',
      'index.ts',
    ),
  );
}

test('credits workspace consumes billing event summary across repository, types, services, and page', () => {
  const repository = read('packages/sdkwork-router-portal-credits/src/repository/index.ts');
  const pageTypes = read('packages/sdkwork-router-portal-credits/src/types/index.ts');
  const services = read('packages/sdkwork-router-portal-credits/src/services/index.ts');
  const page = read('packages/sdkwork-router-portal-credits/src/pages/index.tsx');

  assert.match(repository, /getPortalBillingEventSummary/);
  assert.match(pageTypes, /billing_event_summary: BillingEventSummary;/);
  assert.match(pageTypes, /PortalCreditsFinanceProjection/);
  assert.match(services, /buildPortalCreditsFinanceProjection/);
  assert.match(page, /Redeem decision support/);
  assert.match(page, /Redemption coverage/);
  assert.match(page, /Leading accounting mode/);
  assert.match(page, /Leading capability/);
  assert.match(page, /Multimodal demand/);
  assert.match(page, /portal-redeem-decision-support/);
  assert.match(page, /portal-redeem-multimodal-demand/);
});

test('credits services derive finance projection from coupon redemption history and billing event evidence', () => {
  const { buildPortalCreditsFinanceProjection } = loadCreditsServices();

  const projection = buildPortalCreditsFinanceProjection({
    summary: {
      project_id: 'project-demo',
      entry_count: 8,
      used_units: 32000,
      booked_amount: 199,
      quota_policy_id: 'policy-1',
      quota_limit_units: 40000,
      remaining_units: 2800,
      exhausted: false,
    },
    orders: [
      {
        order_id: 'order-redeem-1',
        project_id: 'project-demo',
        user_id: 'user-1',
        target_kind: 'coupon_redemption',
        target_id: 'WELCOME100',
        target_name: 'Welcome 100',
        list_price_cents: 0,
        payable_price_cents: 0,
        list_price_label: '$0',
        payable_price_label: '$0',
        granted_units: 12000,
        bonus_units: 3000,
        applied_coupon_code: 'WELCOME100',
        status: 'fulfilled',
        source: 'workspace_seed',
        created_at_ms: 100,
      },
      {
        order_id: 'order-redeem-2',
        project_id: 'project-demo',
        user_id: 'user-1',
        target_kind: 'coupon_redemption',
        target_id: 'TEAMREADY',
        target_name: 'Team Ready',
        list_price_cents: 0,
        payable_price_cents: 0,
        list_price_label: '$0',
        payable_price_label: '$0',
        granted_units: 8000,
        bonus_units: 2000,
        applied_coupon_code: 'TEAMREADY',
        status: 'fulfilled',
        source: 'workspace_seed',
        created_at_ms: 200,
      },
      {
        order_id: 'order-pack-1',
        project_id: 'project-demo',
        user_id: 'user-1',
        target_kind: 'recharge_pack',
        target_id: 'pack-growth',
        target_name: 'Growth pack',
        list_price_cents: 9900,
        payable_price_cents: 9900,
        list_price_label: '$99',
        payable_price_label: '$99',
        granted_units: 18000,
        bonus_units: 0,
        status: 'fulfilled',
        source: 'workspace_seed',
        created_at_ms: 300,
      },
    ],
    billingEventSummary: {
      total_events: 5,
      project_count: 1,
      group_count: 2,
      capability_count: 3,
      total_request_count: 9,
      total_units: 650,
      total_input_tokens: 180,
      total_output_tokens: 140,
      total_tokens: 320,
      total_image_count: 4,
      total_audio_seconds: 81,
      total_video_seconds: 39,
      total_music_seconds: 12,
      total_upstream_cost: 9.2,
      total_customer_charge: 13.6,
      projects: [],
      groups: [],
      capabilities: [
        {
          capability: 'audio',
          event_count: 1,
          request_count: 3,
          total_tokens: 0,
          image_count: 0,
          audio_seconds: 81,
          video_seconds: 39,
          music_seconds: 12,
          total_upstream_cost: 2.4,
          total_customer_charge: 3.6,
        },
        {
          capability: 'responses',
          event_count: 2,
          request_count: 5,
          total_tokens: 320,
          image_count: 0,
          audio_seconds: 0,
          video_seconds: 0,
          music_seconds: 0,
          total_upstream_cost: 3.8,
          total_customer_charge: 4.8,
        },
        {
          capability: 'images',
          event_count: 2,
          request_count: 1,
          total_tokens: 0,
          image_count: 4,
          audio_seconds: 0,
          video_seconds: 0,
          music_seconds: 0,
          total_upstream_cost: 3,
          total_customer_charge: 5.2,
        },
      ],
      accounting_modes: [
        {
          accounting_mode: 'passthrough',
          event_count: 1,
          request_count: 1,
          total_upstream_cost: 1.2,
          total_customer_charge: 1.1,
        },
        {
          accounting_mode: 'platform_credit',
          event_count: 3,
          request_count: 7,
          total_upstream_cost: 5.4,
          total_customer_charge: 8.9,
        },
        {
          accounting_mode: 'byok',
          event_count: 1,
          request_count: 1,
          total_upstream_cost: 2.6,
          total_customer_charge: 3.6,
        },
      ],
    },
  });

  assert.equal(projection.redemption_coverage.fulfilled_redemptions, 2);
  assert.equal(projection.redemption_coverage.granted_units, 20000);
  assert.equal(projection.redemption_coverage.bonus_units, 5000);
  assert.equal(projection.redemption_coverage.next_funding_path, 'recharge');
  assert.equal(projection.leading_accounting_mode?.accounting_mode, 'platform_credit');
  assert.equal(projection.leading_capability?.capability, 'images');
  assert.deepEqual(projection.multimodal_totals, {
    image_count: 4,
    audio_seconds: 81,
    video_seconds: 39,
    music_seconds: 12,
  });
});
