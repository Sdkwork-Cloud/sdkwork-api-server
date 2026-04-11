import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

import jiti from '../node_modules/.pnpm/jiti@2.6.1/node_modules/jiti/lib/jiti.mjs';

const appRoot = path.resolve(import.meta.dirname, '..');

function loadPortalApi() {
  const load = jiti(import.meta.url, { moduleCache: false });
  return load(
    path.join(
      appRoot,
      'packages',
      'sdkwork-router-portal-portal-api',
      'src',
      'index.ts',
    ),
  );
}

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

function installPortalApiTestEnvironment() {
  const requests = [];
  const previousFetch = globalThis.fetch;
  const previousLocalStorage = globalThis.localStorage;
  const previousWindow = globalThis.window;

  globalThis.localStorage = {
    getItem(key) {
      return key === 'sdkwork.router.portal.session-token' ? 'portal-session-token' : null;
    },
    setItem() {},
    removeItem() {},
  };
  globalThis.window = {
    location: {
      origin: 'http://127.0.0.1:3001',
      port: '3001',
    },
  };
  globalThis.fetch = async (input, init) => {
    requests.push({
      url: String(input),
      method: init?.method ?? 'GET',
      authorization: init?.headers?.authorization ?? init?.headers?.Authorization ?? null,
    });

    return {
      ok: true,
      status: 200,
      async json() {
        return {};
      },
    };
  };

  return {
    requests,
    restore() {
      globalThis.fetch = previousFetch;
      globalThis.localStorage = previousLocalStorage;
      globalThis.window = previousWindow;
    },
  };
}

test('portal marketing api client exposes coupon validation, reservation, redemption, and inventory methods', async () => {
  const portalApi = loadPortalApi();
  const types = read('packages/sdkwork-router-portal-types/src/index.ts');
  const env = installPortalApiTestEnvironment();

  assert.match(types, /export interface PortalCouponValidationRequest/);
  assert.match(types, /export interface PortalCouponValidationResponse/);
  assert.match(types, /export interface PortalCouponReservationRequest/);
  assert.match(types, /export interface PortalCouponReservationResponse/);
  assert.match(types, /export type PortalCommerceTargetKind =/);
  assert.match(types, /export type PortalMarketingTargetKind = PortalCommerceTargetKind;/);
  assert.match(types, /export interface PortalCouponValidationRequest[\s\S]*target_kind: PortalMarketingTargetKind;/);
  assert.match(types, /export interface PortalCouponReservationRequest[\s\S]*target_kind: PortalMarketingTargetKind;/);
  assert.match(types, /export interface PortalCouponRedemptionConfirmRequest/);
  assert.match(types, /export interface PortalCouponRedemptionConfirmResponse/);
  assert.match(types, /export interface PortalCouponRedemptionRollbackRequest/);
  assert.match(types, /export interface PortalCouponRedemptionRollbackResponse/);
  assert.match(types, /export interface PortalCouponReservationRequest[\s\S]*idempotency_key\?: string \| null;/);
  assert.match(types, /export interface PortalCouponRedemptionConfirmRequest[\s\S]*idempotency_key\?: string \| null;/);
  assert.match(types, /export interface PortalCouponRedemptionRollbackRequest[\s\S]*idempotency_key\?: string \| null;/);
  assert.match(types, /export type PortalCouponEffectKind = 'checkout_discount' \| 'account_entitlement';/);
  assert.match(types, /export interface PortalCouponApplicabilitySummary/);
  assert.match(types, /export interface PortalCouponEffectSummary/);
  assert.match(types, /export interface PortalCouponOwnershipSummary/);
  assert.match(types, /export interface PortalCouponAccountArrivalSummary/);
  assert.match(types, /export interface PortalCouponAccountArrivalLotItem/);
  assert.match(types, /export interface PortalMarketingCodesResponse/);
  assert.match(types, /export interface PortalMarketingCodeItem[\s\S]*template: CouponTemplateRecord;/);
  assert.match(types, /export interface PortalMarketingCodeItem[\s\S]*campaign: MarketingCampaignRecord;/);
  assert.match(types, /export interface PortalMarketingCodeItem[\s\S]*applicability: PortalCouponApplicabilitySummary;/);
  assert.match(types, /export interface PortalMarketingCodeItem[\s\S]*effect: PortalCouponEffectSummary;/);
  assert.match(types, /export interface PortalMarketingCodeItem[\s\S]*ownership: PortalCouponOwnershipSummary;/);
  assert.match(types, /export interface PortalMarketingRedemptionsResponse/);
  assert.match(types, /export interface PortalMarketingRewardHistoryItem/);
  assert.match(types, /export interface PortalMarketingRewardHistoryItem[\s\S]*template: CouponTemplateRecord;/);
  assert.match(types, /export interface PortalMarketingRewardHistoryItem[\s\S]*campaign: MarketingCampaignRecord;/);
  assert.match(types, /export interface PortalMarketingRewardHistoryItem[\s\S]*effect: PortalCouponEffectSummary;/);
  assert.match(types, /export interface PortalMarketingRewardHistoryItem[\s\S]*ownership: PortalCouponOwnershipSummary;/);
  assert.match(types, /export interface PortalMarketingRewardHistoryItem[\s\S]*account_arrival: PortalCouponAccountArrivalSummary;/);
  assert.match(types, /export interface PortalCouponAccountArrivalLotItem[\s\S]*benefit_type: CommercialAccountBenefitType;/);
  assert.match(types, /export interface PortalCouponAccountArrivalLotItem[\s\S]*source_type: CommercialAccountBenefitSourceType;/);
  assert.match(types, /export interface PortalCouponAccountArrivalLotItem[\s\S]*status: CommercialAccountBenefitLotStatus;/);
  assert.match(types, /export interface CouponCodeRecord/);
  assert.match(types, /export interface CouponReservationRecord/);
  assert.match(types, /export interface CouponRedemptionRecord/);
  assert.match(types, /export interface CouponRollbackRecord/);

  try {
    await portalApi.validatePortalCoupon({
      coupon_code: 'LAUNCHA',
      subject_scope: 'project',
      target_kind: 'coupon_redemption',
      order_amount_minor: 5000,
      reserve_amount_minor: 5000,
    });
    await portalApi.reservePortalCouponRedemption({
      coupon_code: 'LAUNCHA',
      subject_scope: 'project',
      target_kind: 'coupon_redemption',
      reserve_amount_minor: 5000,
      ttl_ms: 300000,
      idempotency_key: 'reserve_launcha_project_1',
    });
    await portalApi.confirmPortalCouponRedemption({
      coupon_reservation_id: 'reservation_launch',
      subsidy_amount_minor: 5000,
      order_id: 'order_launch',
      payment_event_id: 'pay_launch',
      idempotency_key: 'confirm_launcha_project_1',
    });
    await portalApi.rollbackPortalCouponRedemption({
      coupon_redemption_id: 'redemption_launch',
      rollback_type: 'refund',
      restored_budget_minor: 5000,
      restored_inventory_count: 1,
      idempotency_key: 'rollback_launcha_project_1_refund',
    });
    await portalApi.listPortalMarketingMyCoupons();
    await portalApi.listPortalMarketingRewardHistory();
    await portalApi.listPortalMarketingRedemptions();
    await portalApi.listPortalMarketingCodes();

    assert.deepEqual(
      env.requests.map((request) => request.url),
      [
        '/api/portal/marketing/coupon-validations',
        '/api/portal/marketing/coupon-reservations',
        '/api/portal/marketing/coupon-redemptions/confirm',
        '/api/portal/marketing/coupon-redemptions/rollback',
        '/api/portal/marketing/my-coupons',
        '/api/portal/marketing/reward-history',
        '/api/portal/marketing/redemptions',
        '/api/portal/marketing/codes',
      ],
    );
    assert.deepEqual(
      env.requests.map((request) => request.method),
      ['POST', 'POST', 'POST', 'POST', 'GET', 'GET', 'GET', 'GET'],
    );
    assert.deepEqual(
      env.requests.map((request) => request.authorization),
      Array(8).fill('Bearer portal-session-token'),
    );
  } finally {
    env.restore();
  }
});
