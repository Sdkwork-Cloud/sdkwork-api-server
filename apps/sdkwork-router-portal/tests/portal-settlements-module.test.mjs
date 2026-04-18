import assert from 'node:assert/strict';
import { existsSync, readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

import jiti from 'jiti';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

function loadSettlementsServices() {
  const load = jiti(import.meta.url, { moduleCache: false });
  return load(
    path.join(
      appRoot,
      'packages',
      'sdkwork-router-portal-settlements',
      'src',
      'services',
      'index.ts',
    ),
  );
}

test('portal settlements module is packaged as a first-class route module with console wiring', () => {
  const tsconfig = read('tsconfig.json');
  const portalTypes = read('packages/sdkwork-router-portal-types/src/index.ts');
  const corePackage = JSON.parse(read('packages/sdkwork-router-portal-core/package.json'));
  const consolePackage = JSON.parse(read('packages/sdkwork-router-portal-console/package.json'));
  const routes = read('packages/sdkwork-router-portal-core/src/routes.ts');
  const routePaths = read('packages/sdkwork-router-portal-core/src/application/router/routePaths.ts');
  const routeManifest = read('packages/sdkwork-router-portal-core/src/application/router/routeManifest.ts');
  const routePrefetch = read('packages/sdkwork-router-portal-core/src/application/router/routePrefetch.ts');
  const navigationRail = read('packages/sdkwork-router-portal-core/src/components/PortalNavigationRail.tsx');
  const appRoutes = read('packages/sdkwork-router-portal-core/src/application/router/AppRoutes.tsx');
  const consoleRoute = read('packages/sdkwork-router-portal-console/src/index.tsx');
  const settlementsPage = read('packages/sdkwork-router-portal-settlements/src/pages/index.tsx');

  assert.equal(
    existsSync(path.join(appRoot, 'packages', 'sdkwork-router-portal-settlements', 'package.json')),
    true,
  );
  assert.equal(
    corePackage.dependencies['sdkwork-router-portal-settlements'],
    'workspace:*',
  );
  assert.equal(
    consolePackage.dependencies['sdkwork-router-portal-settlements'],
    'workspace:*',
  );
  assert.match(tsconfig, /sdkwork-router-portal-settlements/);

  assert.match(portalTypes, /'settlements'/);
  assert.match(portalTypes, /'sdkwork-router-portal-settlements'/);
  assert.match(routes, /key:\s*'settlements'/);
  assert.match(routes, /labelKey:\s*'Settlements'/);
  assert.match(routePaths, /settlements:\s*'\/console\/settlements'/);
  assert.match(routeManifest, /moduleId:\s*'sdkwork-router-portal-settlements'/);
  assert.match(routeManifest, /displayName:\s*'Settlements'/);
  assert.match(routeManifest, /settlement-explorer/);
  assert.match(routeManifest, /credit-holds/);
  assert.match(routeManifest, /pricing-evidence/);
  assert.match(
    routePrefetch,
    /'sdkwork-router-portal-settlements': \(\) => import\('sdkwork-router-portal-settlements'\)/,
  );
  assert.match(navigationRail, /settlements:\s*ReceiptText/);
  assert.match(appRoutes, /case 'settlements':/);
  assert.match(appRoutes, /'settlements',/);
  assert.match(consoleRoute, /import \{ PortalSettlementsPage \} from 'sdkwork-router-portal-settlements';/);
  assert.match(consoleRoute, /case 'settlements':/);
  assert.match(consoleRoute, /<PortalSettlementsPage onNavigate=\{onNavigate\} \/>/);
  assert.match(settlementsPage, /data-slot="portal-settlements-toolbar"/);
  assert.match(settlementsPage, /Credit holds/);
  assert.match(settlementsPage, /Pricing evidence/);
  assert.match(settlementsPage, /Request settlements/);
  assert.match(settlementsPage, /Available balance/);
  assert.match(settlementsPage, /Billing method/);
  assert.match(settlementsPage, /Price unit/);
  assert.match(settlementsPage, /Input token/);
  assert.match(settlementsPage, /USD \/ 1M input tokens/);
  assert.doesNotMatch(settlementsPage, /title=\{t\('Settlement explorer'\)\}/);
  assert.doesNotMatch(settlementsPage, /Open billing/);
});

test('portal settlements services derive settlement explorer posture from canonical commercial evidence', () => {
  const { buildPortalSettlementsViewModel } = loadSettlementsServices();
  const now = new Date('2026-04-03T10:00:00.000Z').getTime();

  const viewModel = buildPortalSettlementsViewModel({
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
        created_at_ms: now - 86_400_000,
        updated_at_ms: now,
      },
      available_balance: 150,
      held_balance: 12,
      consumed_balance: 40,
      grant_balance: 60,
      active_lot_count: 2,
    },
    accountBalance: {
      account_id: 7001,
      available_balance: 150,
      held_balance: 12,
      consumed_balance: 40,
      grant_balance: 60,
      active_lot_count: 2,
      lots: [],
    },
    benefitLots: [
      {
        lot_id: 8101,
        tenant_id: 1001,
        organization_id: 2002,
        account_id: 7001,
        user_id: 9001,
        benefit_type: 'cash_credit',
        source_type: 'recharge',
        source_id: 1,
        scope_json: null,
        original_quantity: 200,
        remaining_quantity: 160,
        held_quantity: 12,
        priority: 10,
        acquired_unit_cost: 0.25,
        issued_at_ms: now - 86_400_000,
        expires_at_ms: now + 86_400_000,
        status: 'active',
        created_at_ms: now - 86_400_000,
        updated_at_ms: now,
      },
      {
        lot_id: 8102,
        tenant_id: 1001,
        organization_id: 2002,
        account_id: 7001,
        user_id: 9001,
        benefit_type: 'promo_credit',
        source_type: 'coupon',
        source_id: 2,
        scope_json: null,
        original_quantity: 40,
        remaining_quantity: 0,
        held_quantity: 0,
        priority: 5,
        acquired_unit_cost: null,
        issued_at_ms: now - 172_800_000,
        expires_at_ms: now - 1_000,
        status: 'expired',
        created_at_ms: now - 172_800_000,
        updated_at_ms: now - 1_000,
      },
    ],
    holds: [
      {
        hold_id: 9101,
        tenant_id: 1001,
        organization_id: 2002,
        account_id: 7001,
        user_id: 9001,
        request_id: 6001,
        status: 'held',
        estimated_quantity: 8,
        captured_quantity: 0,
        released_quantity: 0,
        expires_at_ms: now + 600_000,
        created_at_ms: now - 30_000,
        updated_at_ms: now,
      },
      {
        hold_id: 9102,
        tenant_id: 1001,
        organization_id: 2002,
        account_id: 7001,
        user_id: 9001,
        request_id: 6002,
        status: 'captured',
        estimated_quantity: 12,
        captured_quantity: 11,
        released_quantity: 1,
        expires_at_ms: now + 300_000,
        created_at_ms: now - 120_000,
        updated_at_ms: now - 60_000,
      },
    ],
    requestSettlements: [
      {
        request_settlement_id: 9301,
        tenant_id: 1001,
        organization_id: 2002,
        request_id: 6002,
        account_id: 7001,
        user_id: 9001,
        hold_id: 9102,
        status: 'captured',
        estimated_credit_hold: 12,
        released_credit_amount: 1,
        captured_credit_amount: 11,
        provider_cost_amount: 5,
        retail_charge_amount: 12,
        shortfall_amount: 0,
        refunded_amount: 0,
        settled_at_ms: now - 30_000,
        created_at_ms: now - 60_000,
        updated_at_ms: now - 30_000,
      },
      {
        request_settlement_id: 9302,
        tenant_id: 1001,
        organization_id: 2002,
        request_id: 6003,
        account_id: 7001,
        user_id: 9001,
        hold_id: null,
        status: 'refunded',
        estimated_credit_hold: 4,
        released_credit_amount: 0,
        captured_credit_amount: 0,
        provider_cost_amount: 0,
        retail_charge_amount: 0,
        shortfall_amount: 0,
        refunded_amount: 4,
        settled_at_ms: now - 10_000,
        created_at_ms: now - 20_000,
        updated_at_ms: now - 10_000,
      },
    ],
    pricingPlans: [
      {
        pricing_plan_id: 9401,
        tenant_id: 1001,
        organization_id: 2002,
        plan_code: 'workspace-retail',
        plan_version: 1,
        display_name: 'Workspace Retail',
        currency_code: 'USD',
        credit_unit_code: 'credit',
        status: 'active',
        created_at_ms: now - 86_400_000,
        updated_at_ms: now,
      },
    ],
    pricingRates: [
      {
        pricing_rate_id: 9501,
        tenant_id: 1001,
        organization_id: 2002,
        pricing_plan_id: 9401,
        metric_code: 'token.input',
        capability_code: 'responses',
        model_code: 'gpt-4.1',
        provider_code: 'openrouter',
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
        created_at_ms: now - 60_000,
        updated_at_ms: now,
      },
      {
        pricing_rate_id: 9502,
        tenant_id: 1001,
        organization_id: 2002,
        pricing_plan_id: 9401,
        metric_code: 'image.generate',
        capability_code: 'images',
        model_code: 'gpt-image-1',
        provider_code: 'openai',
        charge_unit: 'image',
        pricing_method: 'per_unit',
        quantity_step: 1,
        unit_price: 2,
        display_price_unit: 'USD / image',
        minimum_billable_quantity: 1,
        minimum_charge: 2,
        rounding_increment: 1,
        rounding_mode: 'ceil',
        included_quantity: 0,
        priority: 50,
        notes: 'Image generation pricing',
        status: 'active',
        created_at_ms: now - 40_000,
        updated_at_ms: now,
      },
    ],
  });

  assert.equal(viewModel.account_id, 7001);
  assert.equal(viewModel.account_status, 'active');
  assert.equal(viewModel.available_balance, 150);
  assert.equal(viewModel.held_balance, 12);
  assert.equal(viewModel.active_benefit_lot_count, 1);
  assert.equal(viewModel.expired_benefit_lot_count, 1);
  assert.equal(viewModel.open_hold_count, 2);
  assert.equal(viewModel.captured_settlement_count, 1);
  assert.equal(viewModel.refunded_settlement_count, 1);
  assert.equal(viewModel.captured_credit_amount, 11);
  assert.equal(viewModel.refunded_credit_amount, 4);
  assert.equal(viewModel.primary_plan_display_name, 'Workspace Retail');
  assert.equal(viewModel.primary_rate_metric_code, 'token.input');
  assert.equal(viewModel.primary_rate_charge_unit, 'input_token');
  assert.equal(viewModel.primary_rate_pricing_method, 'per_unit');
  assert.equal(viewModel.primary_rate_display_price_unit, 'USD / 1M input tokens');
  assert.equal(viewModel.priced_metric_count, 2);
  assert.equal(viewModel.latest_settlements[0]?.request_settlement_id, 9302);
  assert.equal(viewModel.open_holds[0]?.hold_id, 9101);
});

test('portal settlements services keep unsafe settlement and pricing ids ordered without number coercion', () => {
  const { buildPortalSettlementsViewModel } = loadSettlementsServices();

  const viewModel = buildPortalSettlementsViewModel({
    commercialAccount: {
      account: {
        account_id: '9007199254740993',
        tenant_id: 1001,
        organization_id: 2002,
        user_id: 9001,
        account_type: 'primary',
        currency_code: 'USD',
        credit_unit_code: 'credit',
        status: 'active',
        allow_overdraft: false,
        overdraft_limit: 0,
        created_at_ms: 1,
        updated_at_ms: 2,
      },
      available_balance: 150,
      held_balance: 12,
      consumed_balance: 40,
      grant_balance: 60,
      active_lot_count: 2,
    },
    accountBalance: {
      account_id: '9007199254740993',
      available_balance: 150,
      held_balance: 12,
      consumed_balance: 40,
      grant_balance: 60,
      active_lot_count: 2,
      lots: [],
    },
    benefitLots: [
      {
        lot_id: '9007199254740992',
        tenant_id: 1001,
        organization_id: 2002,
        account_id: '9007199254740993',
        user_id: 9001,
        benefit_type: 'cash_credit',
        source_type: 'recharge',
        source_id: null,
        scope_json: null,
        original_quantity: 200,
        remaining_quantity: 160,
        held_quantity: 12,
        priority: 10,
        acquired_unit_cost: 0.25,
        issued_at_ms: 1,
        expires_at_ms: null,
        status: 'active',
        created_at_ms: 1,
        updated_at_ms: 2,
      },
      {
        lot_id: '9007199254740993',
        tenant_id: 1001,
        organization_id: 2002,
        account_id: '9007199254740993',
        user_id: 9001,
        benefit_type: 'cash_credit',
        source_type: 'recharge',
        source_id: null,
        scope_json: null,
        original_quantity: 200,
        remaining_quantity: 160,
        held_quantity: 12,
        priority: 10,
        acquired_unit_cost: 0.25,
        issued_at_ms: 1,
        expires_at_ms: null,
        status: 'active',
        created_at_ms: 1,
        updated_at_ms: 2,
      },
    ],
    holds: [
      {
        hold_id: '9007199254740992',
        tenant_id: 1001,
        organization_id: 2002,
        account_id: '9007199254740993',
        user_id: 9001,
        request_id: '9007199254740993',
        status: 'held',
        estimated_quantity: 8,
        captured_quantity: 0,
        released_quantity: 0,
        expires_at_ms: 3,
        created_at_ms: 10,
        updated_at_ms: 11,
      },
      {
        hold_id: '9007199254740993',
        tenant_id: 1001,
        organization_id: 2002,
        account_id: '9007199254740993',
        user_id: 9001,
        request_id: '9007199254740994',
        status: 'held',
        estimated_quantity: 8,
        captured_quantity: 0,
        released_quantity: 0,
        expires_at_ms: 3,
        created_at_ms: 10,
        updated_at_ms: 11,
      },
    ],
    requestSettlements: [
      {
        request_settlement_id: '9007199254740992',
        tenant_id: 1001,
        organization_id: 2002,
        request_id: '9007199254740993',
        account_id: '9007199254740993',
        user_id: 9001,
        hold_id: '9007199254740992',
        status: 'captured',
        estimated_credit_hold: 12,
        released_credit_amount: 1,
        captured_credit_amount: 11,
        provider_cost_amount: 5,
        retail_charge_amount: 12,
        shortfall_amount: 0,
        refunded_amount: 0,
        settled_at_ms: 20,
        created_at_ms: 19,
        updated_at_ms: 20,
      },
      {
        request_settlement_id: '9007199254740993',
        tenant_id: 1001,
        organization_id: 2002,
        request_id: '9007199254740994',
        account_id: '9007199254740993',
        user_id: 9001,
        hold_id: '9007199254740993',
        status: 'refunded',
        estimated_credit_hold: 4,
        released_credit_amount: 0,
        captured_credit_amount: 0,
        provider_cost_amount: 0,
        retail_charge_amount: 0,
        shortfall_amount: 0,
        refunded_amount: 4,
        settled_at_ms: 20,
        created_at_ms: 19,
        updated_at_ms: 20,
      },
    ],
    pricingPlans: [
      {
        pricing_plan_id: '9007199254740992',
        tenant_id: 1001,
        organization_id: 2002,
        plan_code: 'workspace-retail',
        plan_version: 1,
        display_name: 'Workspace Retail Older Id',
        currency_code: 'USD',
        credit_unit_code: 'credit',
        status: 'active',
        effective_from_ms: 1,
        effective_to_ms: null,
        created_at_ms: 1,
        updated_at_ms: 2,
      },
      {
        pricing_plan_id: '9007199254740993',
        tenant_id: 1001,
        organization_id: 2002,
        plan_code: 'workspace-retail',
        plan_version: 1,
        display_name: 'Workspace Retail Higher Id',
        currency_code: 'USD',
        credit_unit_code: 'credit',
        status: 'active',
        effective_from_ms: 1,
        effective_to_ms: null,
        created_at_ms: 1,
        updated_at_ms: 2,
      },
    ],
    pricingRates: [
      {
        pricing_rate_id: '9007199254740992',
        tenant_id: 1001,
        organization_id: 2002,
        pricing_plan_id: '9007199254740993',
        metric_code: 'token.input',
        capability_code: 'responses',
        model_code: 'gpt-5.4',
        provider_code: 'openrouter',
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
        notes: null,
        status: 'active',
        created_at_ms: 1,
        updated_at_ms: 2,
      },
      {
        pricing_rate_id: '9007199254740993',
        tenant_id: 1001,
        organization_id: 2002,
        pricing_plan_id: '9007199254740993',
        metric_code: 'token.output',
        capability_code: 'responses',
        model_code: 'gpt-5.4',
        provider_code: 'openrouter',
        charge_unit: 'output_token',
        pricing_method: 'per_unit',
        quantity_step: 1000000,
        unit_price: 0.3,
        display_price_unit: 'USD / 1M output tokens',
        minimum_billable_quantity: 0,
        minimum_charge: 0,
        rounding_increment: 1,
        rounding_mode: 'ceil',
        included_quantity: 0,
        priority: 100,
        notes: null,
        status: 'active',
        created_at_ms: 1,
        updated_at_ms: 2,
      },
    ],
  });

  assert.equal(viewModel.account_id, '9007199254740993');
  assert.equal(viewModel.latest_settlements[0]?.request_settlement_id, '9007199254740993');
  assert.equal(viewModel.open_holds[0]?.hold_id, '9007199254740993');
  assert.equal(viewModel.active_benefit_lots[0]?.lot_id, '9007199254740993');
  assert.equal(viewModel.primary_plan_display_name, 'Workspace Retail Higher Id');
  assert.equal(viewModel.primary_rate_metric_code, 'token.output');
});
