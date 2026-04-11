import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

import jiti from '../node_modules/.pnpm/jiti@2.6.1/node_modules/jiti/lib/jiti.mjs';

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

test('credits workspace routes canonical marketing wallet inventory and reward history through repository, types, services, and page', () => {
  const repository = read('packages/sdkwork-router-portal-credits/src/repository/index.ts');
  const pageTypes = read('packages/sdkwork-router-portal-credits/src/types/index.ts');
  const services = read('packages/sdkwork-router-portal-credits/src/services/index.ts');
  const page = read('packages/sdkwork-router-portal-credits/src/pages/index.tsx');

  assert.match(repository, /listPortalMarketingMyCoupons/);
  assert.match(repository, /listPortalMarketingRewardHistory/);
  assert.match(repository, /listPortalMarketingRedemptions/);
  assert.match(repository, /validatePortalCoupon/);
  assert.match(repository, /reservePortalCouponRedemption/);
  assert.match(repository, /confirmPortalCouponRedemption/);

  assert.match(pageTypes, /marketing_codes: PortalMarketingCodesResponse;/);
  assert.match(pageTypes, /marketing_reward_history: PortalMarketingRewardHistoryItem\[];/);
  assert.match(pageTypes, /marketing_redemptions: PortalMarketingRedemptionsResponse;/);
  assert.match(services, /buildPortalCouponSelfServiceDecision/);

  assert.match(page, /My coupons/);
  assert.match(page, /Reward history/);
  assert.match(page, /Apply during checkout/);
  assert.match(page, /portal-redeem-wallet-table/);
  assert.match(page, /portal-redeem-reward-history-table/);
  assert.match(page, /Arrived to account/);
  assert.match(page, /No linked account lot evidence yet/);
  assert.match(page, /No account arrival for checkout discount/);
});

test('credits services classify marketing coupon validation into self-service, checkout-only, and blocked flows', () => {
  const { buildPortalCouponSelfServiceDecision } = loadCreditsServices();

  const baseResponse = {
    campaign: {
      marketing_campaign_id: 'campaign_launch',
      coupon_template_id: 'template_launch',
      display_name: 'Launch Campaign',
      status: 'active',
      start_at_ms: null,
      end_at_ms: null,
      created_at_ms: 1,
      updated_at_ms: 1,
    },
    budget: {
      campaign_budget_id: 'budget_launch',
      marketing_campaign_id: 'campaign_launch',
      status: 'active',
      total_budget_minor: 5000,
      reserved_budget_minor: 0,
      consumed_budget_minor: 0,
      created_at_ms: 1,
      updated_at_ms: 1,
    },
    code: {
      coupon_code_id: 'code_launch',
      coupon_template_id: 'template_launch',
      code_value: 'LAUNCH20',
      status: 'available',
      claimed_subject_scope: null,
      claimed_subject_id: null,
      expires_at_ms: null,
      created_at_ms: 1,
      updated_at_ms: 1,
    },
  };

  const selfService = buildPortalCouponSelfServiceDecision({
    ...baseResponse,
    decision: {
      eligible: true,
      rejection_reason: null,
      reservable_budget_minor: 0,
    },
    template: {
      coupon_template_id: 'template_launch',
      template_key: 'launch-grant',
      display_name: 'Launch Grant',
      status: 'active',
      distribution_kind: 'shared_code',
      benefit: {
        benefit_kind: 'grant_units',
        grant_units: 12000,
        currency_code: null,
      },
      restriction: {
        subject_scope: 'project',
        min_order_amount_minor: null,
        first_order_only: false,
        new_customer_only: false,
        exclusive_group: null,
        stacking_policy: 'exclusive',
        max_redemptions_per_subject: 1,
        eligible_target_kinds: ['coupon_redemption'],
      },
      created_at_ms: 1,
      updated_at_ms: 1,
    },
  });

  assert.equal(selfService.flow, 'grant_self_service');
  assert.match(selfService.message, /redeem/i);

  const checkoutOnly = buildPortalCouponSelfServiceDecision({
    ...baseResponse,
    decision: {
      eligible: true,
      rejection_reason: null,
      reservable_budget_minor: 1200,
    },
    template: {
      coupon_template_id: 'template_discount',
      template_key: 'launch-discount',
      display_name: 'Launch Discount',
      status: 'active',
      distribution_kind: 'shared_code',
      benefit: {
        benefit_kind: 'fixed_amount_off',
        subsidy_amount_minor: 1200,
        currency_code: 'USD',
      },
      restriction: {
        subject_scope: 'project',
        min_order_amount_minor: 5000,
        first_order_only: false,
        new_customer_only: false,
        exclusive_group: null,
        stacking_policy: 'exclusive',
        max_redemptions_per_subject: 1,
        eligible_target_kinds: ['subscription_plan'],
      },
      created_at_ms: 1,
      updated_at_ms: 1,
    },
  });

  assert.equal(checkoutOnly.flow, 'checkout_only');
  assert.match(checkoutOnly.message, /checkout/i);

  const blocked = buildPortalCouponSelfServiceDecision({
    ...baseResponse,
    decision: {
      eligible: false,
      rejection_reason: 'Coupon expired',
      reservable_budget_minor: 0,
    },
    template: {
      coupon_template_id: 'template_expired',
      template_key: 'expired-coupon',
      display_name: 'Expired Coupon',
      status: 'archived',
      distribution_kind: 'shared_code',
      benefit: {
        benefit_kind: 'grant_units',
        grant_units: 12000,
        currency_code: null,
      },
      restriction: {
        subject_scope: 'project',
        min_order_amount_minor: null,
        first_order_only: false,
        new_customer_only: false,
        exclusive_group: null,
        stacking_policy: 'exclusive',
        max_redemptions_per_subject: 1,
        eligible_target_kinds: ['coupon_redemption'],
      },
      created_at_ms: 1,
      updated_at_ms: 1,
    },
  });

  assert.equal(blocked.flow, 'blocked');
  assert.match(blocked.message, /expired/i);
});
