import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

import jiti from '../node_modules/.pnpm/jiti@2.6.1/node_modules/jiti/lib/jiti.mjs';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

function loadAccountServices() {
  const load = jiti(import.meta.url, { moduleCache: false });
  return load(
    path.join(
      appRoot,
      'packages',
      'sdkwork-router-portal-account',
      'src',
      'services',
      'index.ts',
    ),
  );
}

test('portal commercial workspace routes canonical account surfaces through repository, types, services, and pages', () => {
  const portalTypes = read('packages/sdkwork-router-portal-types/src/index.ts');
  const accountRepository = read('packages/sdkwork-router-portal-account/src/repository/index.ts');
  const accountTypes = read('packages/sdkwork-router-portal-account/src/types/index.ts');
  const accountServices = read('packages/sdkwork-router-portal-account/src/services/index.ts');
  const accountPage = read('packages/sdkwork-router-portal-account/src/pages/index.tsx');
  const billingRepository = read('packages/sdkwork-router-portal-billing/src/repository/index.ts');
  const billingTypes = read('packages/sdkwork-router-portal-billing/src/types/index.ts');
  const billingPage = read('packages/sdkwork-router-portal-billing/src/pages/index.tsx');

  assert.match(portalTypes, /export interface CommercialAccountRecord/);
  assert.match(portalTypes, /export interface CommercialAccountSummary/);
  assert.match(portalTypes, /export interface CommercialAccountBalanceSnapshot/);
  assert.match(portalTypes, /export interface CommercialAccountBenefitLotRecord/);
  assert.match(portalTypes, /export interface CommercialAccountHoldRecord/);
  assert.match(portalTypes, /export interface CommercialRequestSettlementRecord/);
  assert.match(portalTypes, /export interface CommercialAccountLedgerHistoryEntry/);
  assert.match(portalTypes, /export interface CommercialAccountHistorySnapshot/);
  assert.match(portalTypes, /export interface CommercialPricingPlanRecord/);
  assert.match(portalTypes, /export interface CommercialPricingRateRecord/);
  assert.match(portalTypes, /export interface PortalCommercePaymentEventRecord/);
  assert.match(portalTypes, /'refunded'/);
  assert.match(portalTypes, /export interface PortalCommerceReconciliationSummary/);
  assert.match(portalTypes, /export interface PortalCommerceOrderCenterResponse/);
  assert.match(portalTypes, /reconciliation: PortalCommerceReconciliationSummary \| null;/);

  assert.match(accountRepository, /getPortalCommercialAccount/);
  assert.match(accountRepository, /getPortalCommercialAccountHistory/);
  assert.match(accountRepository, /getPortalCommercialAccountBalance/);
  assert.match(accountRepository, /listPortalCommercialBenefitLots/);
  assert.match(accountRepository, /listPortalCommercialHolds/);
  assert.match(accountRepository, /listPortalCommercialRequestSettlements/);
  assert.match(accountRepository, /listPortalCommercialPricingPlans/);
  assert.match(accountRepository, /listPortalCommercialPricingRates/);
  assert.match(accountTypes, /commercial_posture:/);
  assert.match(accountServices, /commercial_posture/);
  assert.match(accountPage, /Commercial posture/);
  assert.match(accountPage, /Benefit lots/);
  assert.match(accountPage, /Settlement posture/);
  assert.match(accountPage, /Pricing posture/);
  assert.match(accountPage, /Billing method/);
  assert.match(accountPage, /Price unit/);
  assert.match(accountPage, /Input token/);
  assert.match(accountPage, /USD \/ 1M input tokens/);

  assert.match(billingRepository, /getPortalCommercialAccount/);
  assert.match(billingRepository, /getPortalCommercialAccountHistory/);
  assert.match(billingRepository, /getPortalCommerceOrderCenter/);
  assert.match(billingRepository, /getPortalCommerceOrder/);
  assert.match(billingRepository, /listPortalCommercePaymentMethods/);
  assert.match(billingRepository, /getPortalCommercePaymentAttempt/);
  assert.match(billingRepository, /commercial_reconciliation: order_center\.reconciliation/);
  assert.match(billingRepository, /commercial_history\.request_settlements/);
  assert.match(billingRepository, /listPortalCommercialPricingRates/);
  assert.match(billingTypes, /commercial_account:/);
  assert.match(billingTypes, /commercial_reconciliation:/);
  assert.match(billingPage, /Commercial account/);
  assert.match(billingPage, /Commerce reconciliation/);
  assert.match(billingPage, /Backlog orders/);
  assert.match(billingPage, /Settlement coverage/);
  assert.match(billingPage, /Pricing posture/);
});

test('portal account services derive canonical commercial posture for balance, lots, holds, settlements, and pricing', () => {
  const { buildPortalAccountViewModel } = loadAccountServices();
  const now = new Date('2026-04-03T10:00:00.000Z').getTime();

  const viewModel = buildPortalAccountViewModel({
    summary: {
      project_id: 'project-demo',
      entry_count: 1,
      used_units: 1200,
      booked_amount: 42,
      quota_limit_units: 2000,
      remaining_units: 800,
      exhausted: false,
    },
    membership: null,
    usageSummary: {
      total_requests: 12,
      project_count: 1,
      model_count: 2,
      provider_count: 1,
      projects: [{ project_id: 'project-demo', request_count: 12 }],
      providers: [],
      models: [],
    },
    usageRecords: [],
    ledger: [],
    billingEventSummary: {
      total_events: 0,
      project_count: 0,
      group_count: 0,
      capability_count: 0,
      total_request_count: 0,
      total_units: 0,
      total_input_tokens: 0,
      total_output_tokens: 0,
      total_tokens: 0,
      total_image_count: 0,
      total_audio_seconds: 0,
      total_video_seconds: 0,
      total_music_seconds: 0,
      total_upstream_cost: 0,
      total_customer_charge: 0,
      projects: [],
      groups: [],
      capabilities: [],
      accounting_modes: [],
    },
    commercialAccount: {
      account: {
        account_id: 7001,
        tenant_id: 1001,
        organization_id: 2002,
        user_id: 9001,
        account_type: 'primary',
        currency_code: 'USD',
        credit_unit_code: 'credit',
        status: 'active',
        allow_overdraft: false,
        overdraft_limit: 0,
        created_at_ms: now - 1000,
        updated_at_ms: now,
      },
      available_balance: 150,
      held_balance: 10,
      consumed_balance: 40,
      grant_balance: 240,
      active_lot_count: 1,
    },
    accountBalance: {
      account_id: 7001,
      available_balance: 150,
      held_balance: 10,
      consumed_balance: 40,
      grant_balance: 240,
      active_lot_count: 1,
      lots: [
        {
          lot_id: 8001,
          benefit_type: 'cash_credit',
          scope_json: null,
          expires_at_ms: now + 3 * 24 * 60 * 60 * 1000,
          original_quantity: 200,
          remaining_quantity: 160,
          held_quantity: 10,
          available_quantity: 150,
        },
      ],
    },
    benefitLots: [
      {
        lot_id: 8001,
        tenant_id: 1001,
        organization_id: 2002,
        account_id: 7001,
        user_id: 9001,
        benefit_type: 'cash_credit',
        source_type: 'recharge',
        source_id: null,
        scope_json: null,
        original_quantity: 200,
        remaining_quantity: 160,
        held_quantity: 10,
        priority: 10,
        acquired_unit_cost: 0.25,
        issued_at_ms: now - 2 * 24 * 60 * 60 * 1000,
        expires_at_ms: now + 3 * 24 * 60 * 60 * 1000,
        status: 'active',
        created_at_ms: now - 2 * 24 * 60 * 60 * 1000,
        updated_at_ms: now,
      },
      {
        lot_id: 8002,
        tenant_id: 1001,
        organization_id: 2002,
        account_id: 7001,
        user_id: 9001,
        benefit_type: 'promo_credit',
        source_type: 'coupon',
        source_id: null,
        scope_json: null,
        original_quantity: 40,
        remaining_quantity: 0,
        held_quantity: 0,
        priority: 5,
        acquired_unit_cost: null,
        issued_at_ms: now - 8 * 24 * 60 * 60 * 1000,
        expires_at_ms: now - 1000,
        status: 'expired',
        created_at_ms: now - 8 * 24 * 60 * 60 * 1000,
        updated_at_ms: now - 1000,
      },
    ],
    holds: [
      {
        hold_id: 8101,
        tenant_id: 1001,
        organization_id: 2002,
        account_id: 7001,
        user_id: 9001,
        request_id: 6001,
        status: 'captured',
        estimated_quantity: 10,
        captured_quantity: 10,
        released_quantity: 0,
        expires_at_ms: now + 10 * 60 * 1000,
        created_at_ms: now - 60 * 1000,
        updated_at_ms: now,
      },
    ],
    requestSettlements: [
      {
        request_settlement_id: 8301,
        tenant_id: 1001,
        organization_id: 2002,
        request_id: 6001,
        account_id: 7001,
        user_id: 9001,
        hold_id: 8101,
        status: 'captured',
        estimated_credit_hold: 10,
        released_credit_amount: 0,
        captured_credit_amount: 10,
        provider_cost_amount: 5,
        retail_charge_amount: 10,
        shortfall_amount: 0,
        refunded_amount: 0,
        settled_at_ms: now,
        created_at_ms: now - 60 * 1000,
        updated_at_ms: now,
      },
    ],
    pricingPlans: [
      {
        pricing_plan_id: 9101,
        tenant_id: 1001,
        organization_id: 2002,
        plan_code: 'workspace-retail',
        plan_version: 1,
        display_name: 'Workspace Retail',
        currency_code: 'USD',
        credit_unit_code: 'credit',
        status: 'active',
        effective_from_ms: now - 86_400_000,
        effective_to_ms: now + 86_400_000,
        created_at_ms: now - 2000,
        updated_at_ms: now,
      },
      {
        pricing_plan_id: 9102,
        tenant_id: 1001,
        organization_id: 2002,
        plan_code: 'workspace-retail',
        plan_version: 2,
        display_name: 'Workspace Retail Future',
        currency_code: 'USD',
        credit_unit_code: 'credit',
        status: 'active',
        effective_from_ms: now + 86_400_000,
        effective_to_ms: null,
        created_at_ms: now - 1000,
        updated_at_ms: now + 1000,
      },
    ],
    pricingRates: [
      {
        pricing_rate_id: 9201,
        tenant_id: 1001,
        organization_id: 2002,
        pricing_plan_id: 9101,
        metric_code: 'token.input',
        capability_code: 'responses',
        model_code: 'gpt-4.1',
        provider_code: 'provider-openrouter',
        charge_unit: 'input_token',
        pricing_method: 'per_unit',
        quantity_step: 1000000,
        unit_price: 0.25,
        display_price_unit: 'USD / 1M input tokens',
        minimum_billable_quantity: 0,
        minimum_charge: 0,
        rounding_increment: 1,
        rounding_mode: 'ceil',
        included_quantity: 0,
        priority: 100,
        notes: 'Retail input pricing',
        status: 'active',
        created_at_ms: now - 1000,
        updated_at_ms: now,
      },
    ],
    searchQuery: '',
    historyView: 'all',
    page: 1,
    pageSize: 8,
    now,
  });

  assert.equal(viewModel.commercial_posture.account_id, 7001);
  assert.equal(viewModel.commercial_posture.available_balance, 150);
  assert.equal(viewModel.commercial_posture.held_balance, 10);
  assert.equal(viewModel.commercial_posture.consumed_balance, 40);
  assert.equal(viewModel.commercial_posture.active_lot_count, 1);
  assert.equal(viewModel.commercial_posture.benefit_lot_count, 2);
  assert.equal(viewModel.commercial_posture.active_benefit_lot_count, 1);
  assert.equal(viewModel.commercial_posture.expired_benefit_lot_count, 1);
  assert.equal(viewModel.commercial_posture.open_hold_count, 1);
  assert.equal(viewModel.commercial_posture.settlement_count, 1);
  assert.equal(viewModel.commercial_posture.captured_settlement_amount, 10);
  assert.equal(viewModel.commercial_posture.pricing_plan_count, 1);
  assert.equal(viewModel.commercial_posture.pricing_rate_count, 1);
  assert.equal(viewModel.commercial_posture.primary_plan_display_name, 'Workspace Retail');
  assert.equal(viewModel.commercial_posture.primary_rate_metric_code, 'token.input');
  assert.equal(viewModel.commercial_posture.primary_rate_charge_unit, 'input_token');
  assert.equal(viewModel.commercial_posture.primary_rate_pricing_method, 'per_unit');
  assert.equal(
    viewModel.commercial_posture.primary_rate_display_price_unit,
    'USD / 1M input tokens',
  );
});
