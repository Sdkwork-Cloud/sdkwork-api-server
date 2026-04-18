import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

import jiti from 'jiti';

const appRoot = path.resolve(import.meta.dirname, '..');

function loadAdminApi() {
  const load = jiti(import.meta.url, { moduleCache: false });
  return load(
    path.join(
      appRoot,
      'packages',
      'sdkwork-router-admin-admin-api',
      'src',
      'index.ts',
    ),
  );
}

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

function installAdminApiTestEnvironment() {
  const requests = [];
  const previousFetch = globalThis.fetch;
  const previousLocalStorage = globalThis.localStorage;
  const previousWindow = globalThis.window;

  globalThis.localStorage = {
    getItem(key) {
      return key === 'sdkwork.router.admin.session-token' ? 'admin-session-token' : null;
    },
    setItem() {},
    removeItem() {},
  };
  globalThis.window = {
    location: {
      origin: 'http://127.0.0.1:3000',
      port: '3000',
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

test('admin pricing api client exposes canonical pricing governance methods and rich type surface', async () => {
  const adminApi = loadAdminApi();
  const types = read('packages/sdkwork-router-admin-types/src/index.ts');
  const env = installAdminApiTestEnvironment();

  assert.match(types, /export type CommercialPricingChargeUnit/);
  assert.match(types, /'input_token'/);
  assert.match(types, /'image'/);
  assert.match(types, /'video_minute'/);
  assert.match(types, /'music_track'/);
  assert.match(types, /export type CommercialPricingMethod/);
  assert.match(types, /'per_unit'/);
  assert.match(types, /'flat'/);
  assert.match(types, /'included_then_per_unit'/);
  assert.match(types, /capability_code\?: string \| null;/);
  assert.match(types, /charge_unit: CommercialPricingChargeUnit;/);
  assert.match(types, /pricing_method: CommercialPricingMethod;/);
  assert.match(types, /display_price_unit: string;/);
  assert.match(types, /rounding_mode: string;/);
  assert.match(types, /minimum_charge: number;/);
  assert.match(types, /included_quantity: number;/);
  assert.match(types, /priority: number;/);
  assert.match(types, /notes\?: string \| null;/);
  assert.match(types, /status: string;/);
  assert.match(types, /effective_from_ms: number;/);
  assert.match(types, /effective_to_ms\?: number \| null;/);
  assert.match(types, /export interface CommercialPricingLifecycleSynchronizationReport/);
  assert.match(types, /changed: boolean;/);
  assert.match(types, /due_group_count: number;/);
  assert.match(types, /activated_plan_count: number;/);
  assert.match(types, /archived_plan_count: number;/);
  assert.match(types, /activated_rate_count: number;/);
  assert.match(types, /archived_rate_count: number;/);
  assert.match(types, /skipped_plan_count: number;/);
  assert.match(types, /synchronized_at_ms: number;/);

  try {
    await adminApi.listCommercialPricingPlans();
    await adminApi.createCommercialPricingPlan({
      tenant_id: 1001,
      organization_id: 2002,
      plan_code: 'retail-pro',
      plan_version: 1,
      display_name: 'Retail Pro',
      currency_code: 'USD',
      credit_unit_code: 'credit',
      status: 'active',
      effective_from_ms: 1717171730000,
      effective_to_ms: 1719773730000,
    });
    await adminApi.updateCommercialPricingPlan(9101, {
      tenant_id: 1001,
      organization_id: 2002,
      plan_code: 'retail-pro',
      plan_version: 2,
      display_name: 'Retail Pro Updated',
      currency_code: 'USD',
      credit_unit_code: 'credit',
      status: 'draft',
      effective_from_ms: 1718035730000,
      effective_to_ms: 1720627730000,
    });
    await adminApi.cloneCommercialPricingPlan(9101);
    await adminApi.scheduleCommercialPricingPlan(9101);
    await adminApi.publishCommercialPricingPlan(9101);
    await adminApi.retireCommercialPricingPlan(9101);
    await adminApi.synchronizeCommercialPricingLifecycle();
    await adminApi.listCommercialPricingRates();
    await adminApi.createCommercialPricingRate({
      tenant_id: 1001,
      organization_id: 2002,
      pricing_plan_id: 9101,
      metric_code: 'token.input',
      capability_code: 'responses',
      model_code: 'gpt-4.1',
      provider_code: 'provider-openai-official',
      charge_unit: 'input_token',
      pricing_method: 'per_unit',
      quantity_step: 1000000,
      unit_price: 2.5,
      display_price_unit: 'USD / 1M input tokens',
      minimum_billable_quantity: 0,
      minimum_charge: 0,
      rounding_increment: 1,
      rounding_mode: 'ceil',
      included_quantity: 0,
      priority: 100,
      notes: 'Retail text input pricing',
      status: 'active',
    });
    await adminApi.updateCommercialPricingRate(9201, {
      tenant_id: 1001,
      organization_id: 2002,
      pricing_plan_id: 9101,
      metric_code: 'image.output',
      capability_code: 'images',
      model_code: 'gpt-image-1',
      provider_code: 'provider-openai-official',
      charge_unit: 'image',
      pricing_method: 'flat',
      quantity_step: 1,
      unit_price: 0.08,
      display_price_unit: 'USD / image',
      minimum_billable_quantity: 1,
      minimum_charge: 0.08,
      rounding_increment: 1,
      rounding_mode: 'ceil',
      included_quantity: 0,
      priority: 200,
      notes: 'Updated image pricing',
      status: 'draft',
    });

    assert.deepEqual(
      env.requests.map((request) => request.url),
      [
        '/api/admin/billing/pricing-plans',
        '/api/admin/billing/pricing-plans',
        '/api/admin/billing/pricing-plans/9101',
        '/api/admin/billing/pricing-plans/9101/clone',
        '/api/admin/billing/pricing-plans/9101/schedule',
        '/api/admin/billing/pricing-plans/9101/publish',
        '/api/admin/billing/pricing-plans/9101/retire',
        '/api/admin/billing/pricing-lifecycle/synchronize',
        '/api/admin/billing/pricing-rates',
        '/api/admin/billing/pricing-rates',
        '/api/admin/billing/pricing-rates/9201',
      ],
    );
    assert.deepEqual(
      env.requests.map((request) => request.method),
      ['GET', 'POST', 'PUT', 'POST', 'POST', 'POST', 'POST', 'POST', 'GET', 'POST', 'PUT'],
    );
    assert.deepEqual(
      env.requests.map((request) => request.authorization),
      Array(11).fill('Bearer admin-session-token'),
    );
  } finally {
    env.restore();
  }
});
